use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ─── Character ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    pub character_id: String,
    pub world_id: String,
    pub display_name: String,
    pub identity: String,
    pub voice_rules: Value,
    pub boundaries: Value,
    pub backstory_facts: Value,
    pub relationships: Value,
    pub state: Value,
    pub avatar_color: String,
    pub sex: String,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub fn create_character(conn: &Connection, ch: &Character) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO characters (character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, sex, is_archived, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![ch.character_id, ch.world_id, ch.display_name, ch.identity,
            ch.voice_rules.to_string(), ch.boundaries.to_string(),
            ch.backstory_facts.to_string(), ch.relationships.to_string(),
            ch.state.to_string(), ch.avatar_color, ch.sex, ch.is_archived, ch.created_at, ch.updated_at],
    )?;
    Ok(())
}

pub fn get_character(conn: &Connection, character_id: &str) -> Result<Character, rusqlite::Error> {
    conn.query_row(
        "SELECT character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, sex, is_archived, created_at, updated_at FROM characters WHERE character_id = ?1",
        params![character_id],
        row_to_character,
    )
}

pub fn list_characters(conn: &Connection, world_id: &str) -> Result<Vec<Character>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT c.character_id, c.world_id, c.display_name, c.identity, c.voice_rules, c.boundaries, c.backstory_facts, c.relationships, c.state, c.avatar_color, c.sex, c.is_archived, c.created_at, c.updated_at
         FROM characters c
         LEFT JOIN threads t ON t.character_id = c.character_id
         LEFT JOIN (SELECT thread_id, MAX(created_at) AS last_msg FROM messages GROUP BY thread_id) m ON m.thread_id = t.thread_id
         WHERE c.world_id = ?1 AND c.is_archived = 0
         ORDER BY m.last_msg DESC NULLS LAST, c.created_at"
    )?;
    let rows = stmt.query_map(params![world_id], row_to_character)?;
    rows.collect()
}

pub fn list_archived_characters(conn: &Connection, world_id: &str) -> Result<Vec<Character>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, sex, is_archived, created_at, updated_at FROM characters WHERE world_id = ?1 AND is_archived = 1 ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map(params![world_id], row_to_character)?;
    rows.collect()
}

pub fn archive_character(conn: &Connection, character_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET is_archived = 1, updated_at = datetime('now') WHERE character_id = ?1",
        params![character_id],
    )?;
    Ok(())
}

pub fn unarchive_character(conn: &Connection, character_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET is_archived = 0, updated_at = datetime('now') WHERE character_id = ?1",
        params![character_id],
    )?;
    Ok(())
}

pub fn update_character(conn: &Connection, ch: &Character) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET display_name=?2, identity=?3, voice_rules=?4, boundaries=?5, backstory_facts=?6, relationships=?7, state=?8, avatar_color=?9, sex=?10, updated_at=datetime('now') WHERE character_id=?1",
        params![ch.character_id, ch.display_name, ch.identity,
            ch.voice_rules.to_string(), ch.boundaries.to_string(),
            ch.backstory_facts.to_string(), ch.relationships.to_string(),
            ch.state.to_string(), ch.avatar_color, ch.sex],
    )?;
    Ok(())
}

/// Delete all chat-related data for a character's threads: messages, FTS, embeddings,
/// memory artifacts, reactions, and message count trackers.
/// Does NOT delete the threads themselves or the character.
/// Returns (illustration_file_names, all_message_ids) for disk cleanup.
fn purge_thread_data(conn: &Connection, character_id: &str, thread_ids: &[String]) -> Result<(Vec<String>, Vec<String>), rusqlite::Error> {
    let mut illustration_files: Vec<String> = Vec::new();
    let mut all_message_ids: Vec<String> = Vec::new();

    for tid in thread_ids {
        // Collect all message IDs before deletion (for audio file cleanup)
        {
            let mut stmt = conn.prepare("SELECT message_id FROM messages WHERE thread_id = ?1")?;
            let ids: Vec<String> = stmt.query_map(params![tid], |row| row.get(0))?
                .filter_map(|r| r.ok()).collect();
            all_message_ids.extend(ids);
        }

        // Find illustration messages and clean up their gallery entries
        let mut stmt = conn.prepare(
            "SELECT message_id FROM messages WHERE thread_id = ?1 AND role = 'illustration'"
        )?;
        let illus_ids: Vec<String> = stmt.query_map(params![tid], |row| row.get(0))?
            .filter_map(|r| r.ok()).collect();
        for illus_id in &illus_ids {
            let file_name: Option<String> = conn.query_row(
                "SELECT file_name FROM world_images WHERE image_id = ?1",
                params![illus_id], |r| r.get(0),
            ).ok();
            conn.execute("DELETE FROM world_images WHERE image_id = ?1", params![illus_id])?;
            if let Some(f) = file_name {
                illustration_files.push(f);
            }
        }

        // FTS entries
        conn.execute("DELETE FROM messages_fts WHERE thread_id = ?1", params![tid])?;

        // Reactions cascade from messages, but delete messages explicitly since
        // we're not deleting the thread itself in the clear_chat_history case.
        conn.execute("DELETE FROM messages WHERE thread_id = ?1", params![tid])?;

        // Memory artifacts (thread summaries etc.)
        conn.execute("DELETE FROM memory_artifacts WHERE subject_id = ?1", params![tid])?;

        // Message count tracker
        conn.execute("DELETE FROM message_count_tracker WHERE thread_id = ?1", params![tid])?;
    }

    // Delete all vector embeddings for this character's messages in one pass
    let rowids: Vec<i64> = {
        let mut stmt = conn.prepare(
            "SELECT rowid FROM chunk_metadata WHERE character_id = ?1 AND source_type = 'message'"
        )?;
        let rows = stmt.query_map(params![character_id], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    for rowid in &rowids {
        conn.execute("DELETE FROM vec_chunks WHERE rowid = ?1", params![rowid])?;
    }
    if !rowids.is_empty() {
        conn.execute(
            "DELETE FROM chunk_metadata WHERE character_id = ?1 AND source_type = 'message'",
            params![character_id],
        )?;
    }

    Ok((illustration_files, all_message_ids))
}

/// Returns (illustration_file_names, message_ids) for disk cleanup.
pub fn delete_character(conn: &Connection, character_id: &str) -> Result<(Vec<String>, Vec<String>), rusqlite::Error> {
    let thread_ids: Vec<String> = {
        let mut stmt = conn.prepare("SELECT thread_id FROM threads WHERE character_id = ?1")?;
        let rows = stmt.query_map(params![character_id], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let (illustration_files, message_ids) = purge_thread_data(conn, character_id, &thread_ids)?;

    // Delete the character — cascade handles threads, portraits (DB rows),
    // chat_backgrounds, character_mood.
    conn.execute("DELETE FROM characters WHERE character_id = ?1", params![character_id])?;
    Ok((illustration_files, message_ids))
}

/// Clear all chat history for a character while preserving the character and thread.
/// Returns (illustration_file_names, message_ids) for disk cleanup.
pub fn clear_chat_history(conn: &Connection, character_id: &str) -> Result<(Vec<String>, Vec<String>), rusqlite::Error> {
    let thread_ids: Vec<String> = {
        let mut stmt = conn.prepare("SELECT thread_id FROM threads WHERE character_id = ?1")?;
        let rows = stmt.query_map(params![character_id], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let (illustration_files, message_ids) = purge_thread_data(conn, character_id, &thread_ids)?;

    // Reset mood history (preserve current mood values, just clear history)
    conn.execute(
        "UPDATE character_mood SET history = '[]' WHERE character_id = ?1",
        params![character_id],
    )?;

    Ok((illustration_files, message_ids))
}

/// Delete all messages strictly after the given message (by created_at) in the same thread.
/// Also cleans up FTS entries, reactions (cascaded), vector embeddings, and illustration gallery entries.
/// Returns the IDs, roles, and any illustration file names to delete from disk.
pub fn delete_messages_after(conn: &Connection, thread_id: &str, character_id: &str, after_message_id: &str) -> Result<(Vec<(String, String)>, Vec<String>), rusqlite::Error> {
    // Use rowid to get reliable insertion order — timestamps can collide
    let anchor_rowid: i64 = conn.query_row(
        "SELECT rowid FROM messages WHERE message_id = ?1",
        params![after_message_id],
        |r| r.get(0),
    )?;

    // Find all messages inserted after the anchor in this thread
    let mut stmt = conn.prepare(
        "SELECT message_id, role FROM messages WHERE thread_id = ?1 AND rowid > ?2 ORDER BY rowid"
    )?;
    let deleted: Vec<(String, String)> = stmt.query_map(params![thread_id, anchor_rowid], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?.filter_map(|r| r.ok()).collect();

    if deleted.is_empty() {
        return Ok((deleted, vec![]));
    }

    let mut illustration_files: Vec<String> = Vec::new();

    for (msg_id, role) in &deleted {
        // FTS
        conn.execute("DELETE FROM messages_fts WHERE message_id = ?1", params![msg_id])?;
        // Message (reactions cascade via FK)
        conn.execute("DELETE FROM messages WHERE message_id = ?1", params![msg_id])?;
        // Vector embeddings
        let rowid: Option<i64> = conn.query_row(
            "SELECT rowid FROM chunk_metadata WHERE chunk_id = ?1",
            params![msg_id],
            |r| r.get(0),
        ).ok();
        if let Some(rid) = rowid {
            conn.execute("DELETE FROM vec_chunks WHERE rowid = ?1", params![rid])?;
            conn.execute("DELETE FROM chunk_metadata WHERE rowid = ?1", params![rid])?;
        }
        // Illustration cleanup: delete world_image entry (linked by message_id = image_id)
        if role == "illustration" {
            let file_name: Option<String> = conn.query_row(
                "SELECT file_name FROM world_images WHERE image_id = ?1",
                params![msg_id],
                |r| r.get(0),
            ).ok();
            conn.execute("DELETE FROM world_images WHERE image_id = ?1", params![msg_id])?;
            if let Some(f) = file_name {
                illustration_files.push(f);
            }
        }
    }

    // Sweep for any orphaned embeddings whose source messages no longer exist
    let orphaned_rowids: Vec<i64> = {
        let mut stmt = conn.prepare(
            "SELECT cm.rowid FROM chunk_metadata cm
             WHERE cm.character_id = ?1 AND cm.source_type = 'message'
             AND NOT EXISTS (SELECT 1 FROM messages m WHERE m.message_id = cm.source_id)"
        )?;
        let rows = stmt.query_map(params![character_id], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };
    for rid in &orphaned_rowids {
        conn.execute("DELETE FROM vec_chunks WHERE rowid = ?1", params![rid])?;
    }
    if !orphaned_rowids.is_empty() {
        conn.execute(
            &format!(
                "DELETE FROM chunk_metadata WHERE rowid IN ({})",
                orphaned_rowids.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(",")
            ),
            [],
        )?;
    }

    // Reset memory maintenance counter since we've changed the history
    conn.execute("DELETE FROM message_count_tracker WHERE thread_id = ?1", params![thread_id])?;

    // Invalidate thread summary since context has changed
    conn.execute(
        "DELETE FROM memory_artifacts WHERE subject_id = ?1 AND artifact_type = 'thread_summary'",
        params![thread_id],
    )?;

    Ok((deleted, illustration_files))
}

/// Clear all world events and their FTS entries for a world.

fn row_to_character(row: &rusqlite::Row) -> Result<Character, rusqlite::Error> {
    Ok(Character {
        character_id: row.get(0)?,
        world_id: row.get(1)?,
        display_name: row.get(2)?,
        identity: row.get(3)?,
        voice_rules: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
        boundaries: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
        backstory_facts: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
        relationships: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
        state: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
        avatar_color: row.get(9)?,
        sex: row.get::<_, Option<String>>(10)?.unwrap_or_else(|| "male".to_string()),
        is_archived: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}



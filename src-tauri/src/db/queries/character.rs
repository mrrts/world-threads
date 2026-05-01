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
    /// Short honest physical description of the character as seen in
    /// their active portrait. Populated by the vision-describe command;
    /// other characters read it so they "know what this person looks
    /// like." Empty when no portrait has been described yet.
    #[serde(default)]
    pub visual_description: String,
    /// The portrait_id that generated `visual_description`. Cache key:
    /// if the currently-active portrait's id matches this, the
    /// description is still fresh and we skip the vision call.
    #[serde(default)]
    pub visual_description_portrait_id: Option<String>,
    /// Current kept-in-hand inventory for this character. Stored as a
    /// JSON array of { name, description } objects. Max 3 entries.
    /// Refreshed by a memory-tier LLM call on world-day rollover.
    #[serde(default)]
    pub inventory: Value,
    /// World-day index (World.state.time.day_index) the inventory was
    /// last refreshed against. NULL = never seeded — next focus on the
    /// character triggers an initial seed pass.
    #[serde(default)]
    pub last_inventory_day: Option<i64>,
    /// Optional single-emoji signature. Rendered into the prompt's
    /// identity block when non-empty with usage guidance: drop it in
    /// RARELY, only on beats where the character feels especially
    /// themselves. Empty string = disabled.
    #[serde(default)]
    pub signature_emoji: String,
    /// How often this character uses italicized stage directions
    /// (*leans back*, *looks out the window*) in their replies.
    /// One of "low" | "normal" | "high". Overrides the global
    /// ~1-in-3-replies-no-beat baseline per-character. Quiet types
    /// like John (older, soft-spoken) read more measured on "low";
    /// alert, in-motion types like Darren benefit from "high."
    /// Defaults to "normal" for backward-compatible behavior.
    #[serde(default = "default_action_beat_density")]
    pub action_beat_density: String,
    /// Documentary formula-shorthand derivation of F = (R, C) for
    /// this character. Authored via `worldcli derive-character` (or,
    /// eventually, AI-trigger-on-save). When present, injected at
    /// the head of the IDENTITY section in dialogue prompts (per the
    /// layered-substrate design: derivation = tuning, prose =
    /// vocabulary). NULL for characters not yet derivation-populated.
    #[serde(default)]
    pub derived_formula: Option<String>,
    /// Full Empiricon document injected into prompts when true.
    #[serde(default)]
    pub has_read_empiricon: bool,
}

fn default_action_beat_density() -> String { "normal".to_string() }

const CHAR_COLS: &str = "character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, sex, is_archived, created_at, updated_at, visual_description, visual_description_portrait_id, inventory, last_inventory_day, signature_emoji, action_beat_density, derived_formula, has_read_empiricon";

pub fn create_character(conn: &Connection, ch: &Character) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO characters (character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, sex, is_archived, created_at, updated_at, visual_description, visual_description_portrait_id, inventory, last_inventory_day, signature_emoji, action_beat_density, has_read_empiricon)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
        params![ch.character_id, ch.world_id, ch.display_name, ch.identity,
            ch.voice_rules.to_string(), ch.boundaries.to_string(),
            ch.backstory_facts.to_string(), ch.relationships.to_string(),
            ch.state.to_string(), ch.avatar_color, ch.sex, ch.is_archived, ch.created_at, ch.updated_at,
            ch.visual_description, ch.visual_description_portrait_id,
            ch.inventory.to_string(), ch.last_inventory_day, ch.signature_emoji, ch.action_beat_density,
            ch.has_read_empiricon as i32],
    )?;
    Ok(())
}

pub fn get_character(conn: &Connection, character_id: &str) -> Result<Character, rusqlite::Error> {
    conn.query_row(
        &format!("SELECT {CHAR_COLS} FROM characters WHERE character_id = ?1"),
        params![character_id],
        row_to_character,
    )
}

pub fn list_characters(conn: &Connection, world_id: &str) -> Result<Vec<Character>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT c.character_id, c.world_id, c.display_name, c.identity, c.voice_rules, c.boundaries, c.backstory_facts, c.relationships, c.state, c.avatar_color, c.sex, c.is_archived, c.created_at, c.updated_at, c.visual_description, c.visual_description_portrait_id, c.inventory, c.last_inventory_day, c.signature_emoji, c.action_beat_density, c.derived_formula, c.has_read_empiricon
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
        &format!("SELECT {CHAR_COLS} FROM characters WHERE world_id = ?1 AND is_archived = 1 ORDER BY updated_at DESC")
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
    // visual_description is persisted here (user-editable on the settings
    // page) but visual_description_portrait_id is NOT — it's a cache key
    // owned by the vision-describe command. Preserving it means a user
    // edit survives as long as the active portrait doesn't change; when
    // a new portrait is generated the cache key mismatches and the
    // backfill sweep will regenerate, cleanly overwriting the edit.
    conn.execute(
        "UPDATE characters SET display_name=?2, identity=?3, voice_rules=?4, boundaries=?5, backstory_facts=?6, relationships=?7, state=?8, avatar_color=?9, sex=?10, visual_description=?11, signature_emoji=?12, action_beat_density=?13, has_read_empiricon=?14, updated_at=datetime('now') WHERE character_id=?1",
        params![ch.character_id, ch.display_name, ch.identity,
            ch.voice_rules.to_string(), ch.boundaries.to_string(),
            ch.backstory_facts.to_string(), ch.relationships.to_string(),
            ch.state.to_string(), ch.avatar_color, ch.sex, ch.visual_description, ch.signature_emoji, ch.action_beat_density,
            ch.has_read_empiricon as i32],
    )?;
    Ok(())
}

/// Delete all chat-related data for a character's threads: messages, FTS, embeddings,
/// memory artifacts, reactions, and message count trackers.
/// Does NOT delete the threads themselves or the character.
/// When `keep_media` is true, illustration messages and their world_images entries
/// (including linked videos) are preserved, as are novel_entries for each thread.
/// Returns (illustration_file_names, deleted_message_ids) for disk cleanup.
fn purge_thread_data(conn: &Connection, character_id: &str, thread_ids: &[String], keep_media: bool) -> Result<(Vec<String>, Vec<String>), rusqlite::Error> {
    let mut illustration_files: Vec<String> = Vec::new();
    let mut deleted_message_ids: Vec<String> = Vec::new();

    for tid in thread_ids {
        // Collect deletable (non-illustration if keeping media) message IDs
        // up front for audio/disk cleanup.
        {
            let sql = if keep_media {
                "SELECT message_id FROM messages WHERE thread_id = ?1 AND role != 'illustration'"
            } else {
                "SELECT message_id FROM messages WHERE thread_id = ?1"
            };
            let mut stmt = conn.prepare(sql)?;
            let ids: Vec<String> = stmt.query_map(params![tid], |row| row.get(0))?
                .filter_map(|r| r.ok()).collect();
            deleted_message_ids.extend(ids);
        }

        if !keep_media {
            // Find illustration messages and clean up their gallery entries (+ files on disk)
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
        }

        // FTS entries — illustrations are never indexed in FTS (message.rs:79), so
        // blanket-deleting by thread_id only removes text messages either way.
        conn.execute("DELETE FROM messages_fts WHERE thread_id = ?1", params![tid])?;

        // Messages: delete everything, or everything except illustrations.
        if keep_media {
            conn.execute(
                "DELETE FROM messages WHERE thread_id = ?1 AND role != 'illustration'",
                params![tid],
            )?;
        } else {
            conn.execute("DELETE FROM messages WHERE thread_id = ?1", params![tid])?;
        }

        // Memory artifacts (thread summaries etc.)
        conn.execute("DELETE FROM memory_artifacts WHERE subject_id = ?1", params![tid])?;

        // Message count tracker
        conn.execute("DELETE FROM message_count_tracker WHERE thread_id = ?1", params![tid])?;

        // Novel entries: preserve when keeping media, otherwise wipe them too.
        if !keep_media {
            conn.execute("DELETE FROM novel_entries WHERE thread_id = ?1", params![tid])?;
        }
    }

    // Delete vector embeddings. Illustrations aren't embedded, so purging all of
    // the character's message embeddings is correct in both branches.
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

    Ok((illustration_files, deleted_message_ids))
}

/// Returns (illustration_file_names, message_ids) for disk cleanup.
pub fn delete_character(conn: &Connection, character_id: &str) -> Result<(Vec<String>, Vec<String>), rusqlite::Error> {
    let thread_ids: Vec<String> = {
        let mut stmt = conn.prepare("SELECT thread_id FROM threads WHERE character_id = ?1")?;
        let rows = stmt.query_map(params![character_id], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let (illustration_files, message_ids) = purge_thread_data(conn, character_id, &thread_ids, false)?;

    // Delete the character — cascade handles threads, portraits (DB rows),
    // chat_backgrounds, character_mood.
    conn.execute("DELETE FROM characters WHERE character_id = ?1", params![character_id])?;
    Ok((illustration_files, message_ids))
}

/// Clear all chat history for a character while preserving the character and thread.
/// When `keep_media` is true, illustration messages, their linked videos, and
/// novel_entries for each thread are preserved — the rest (user/assistant/narrative
/// messages, summaries, embeddings, FTS) is wiped.
/// Returns (illustration_file_names, deleted_message_ids) for disk cleanup.
pub fn clear_chat_history(conn: &Connection, character_id: &str, keep_media: bool) -> Result<(Vec<String>, Vec<String>), rusqlite::Error> {
    let thread_ids: Vec<String> = {
        let mut stmt = conn.prepare("SELECT thread_id FROM threads WHERE character_id = ?1")?;
        let rows = stmt.query_map(params![character_id], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let (illustration_files, message_ids) = purge_thread_data(conn, character_id, &thread_ids, keep_media)?;

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
        visual_description: row.get::<_, Option<String>>(14)?.unwrap_or_default(),
        visual_description_portrait_id: row.get(15).ok(),
        inventory: row.get::<_, Option<String>>(16)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| Value::Array(vec![])),
        last_inventory_day: row.get(17).ok(),
        signature_emoji: row.get::<_, Option<String>>(18).ok().flatten().unwrap_or_default(),
        action_beat_density: row.get::<_, Option<String>>(19).ok().flatten().unwrap_or_else(|| "normal".to_string()),
        derived_formula: row.get::<_, Option<String>>(20).unwrap_or(None),
        has_read_empiricon: row
            .get::<_, Option<i64>>(21)
            .ok()
            .flatten()
            .map(|n| n != 0)
            .unwrap_or(false),
    })
}

/// Update only the visual description fields without touching identity /
/// voice / backstory / etc. Used by the vision-describe command so a
/// refresh doesn't inadvertently squash other fields the user may be
/// editing concurrently.
pub fn set_visual_description(
    conn: &Connection,
    character_id: &str,
    description: &str,
    source_portrait_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET visual_description = ?2, visual_description_portrait_id = ?3, updated_at = datetime('now') WHERE character_id = ?1",
        params![character_id, description, source_portrait_id],
    )?;
    Ok(())
}

/// Replace the character's inventory array. Caller is responsible for
/// capping to max 3 items; this blindly writes whatever is passed (the
/// JSON column holds arbitrary structure). `world_day_stamp` records
/// which world-day index this inventory is current as of, so the next
/// focus-trigger can decide whether a refresh is due.
pub fn set_character_inventory(
    conn: &Connection,
    character_id: &str,
    inventory: &Value,
    world_day_stamp: Option<i64>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET inventory = ?2, last_inventory_day = ?3, updated_at = datetime('now') WHERE character_id = ?1",
        params![character_id, inventory.to_string(), world_day_stamp],
    )?;
    Ok(())
}



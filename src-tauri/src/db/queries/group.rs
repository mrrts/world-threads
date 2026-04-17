use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use super::message::{Message, MSG_COLS, row_to_message};

// ─── Group Chats ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupChat {
    pub group_chat_id: String,
    pub world_id: String,
    pub character_ids: serde_json::Value,
    pub thread_id: String,
    pub display_name: String,
    pub created_at: String,
}

pub fn create_group_chat(conn: &Connection, gc: &GroupChat) -> Result<(), rusqlite::Error> {
    // Create thread for the group (character_id is NULL for group threads)
    conn.execute(
        "INSERT INTO threads (thread_id, character_id, world_id) VALUES (?1, NULL, ?2)",
        params![gc.thread_id, gc.world_id],
    )?;
    conn.execute(
        "INSERT INTO group_chats (group_chat_id, world_id, character_ids, thread_id, display_name, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![gc.group_chat_id, gc.world_id, gc.character_ids.to_string(), gc.thread_id, gc.display_name, gc.created_at],
    )?;
    Ok(())
}

pub fn list_group_chats(conn: &Connection, world_id: &str) -> Result<Vec<GroupChat>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT group_chat_id, world_id, character_ids, thread_id, display_name, created_at FROM group_chats WHERE world_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![world_id], |row| {
        let ids_str: String = row.get(2)?;
        Ok(GroupChat {
            group_chat_id: row.get(0)?,
            world_id: row.get(1)?,
            character_ids: serde_json::from_str(&ids_str).unwrap_or(serde_json::Value::Array(vec![])),
            thread_id: row.get(3)?,
            display_name: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn get_group_chat(conn: &Connection, group_chat_id: &str) -> Result<GroupChat, rusqlite::Error> {
    conn.query_row(
        "SELECT group_chat_id, world_id, character_ids, thread_id, display_name, created_at FROM group_chats WHERE group_chat_id = ?1",
        params![group_chat_id],
        |row| {
            let ids_str: String = row.get(2)?;
            Ok(GroupChat {
                group_chat_id: row.get(0)?,
                world_id: row.get(1)?,
                character_ids: serde_json::from_str(&ids_str).unwrap_or(serde_json::Value::Array(vec![])),
                thread_id: row.get(3)?,
                display_name: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    )
}

pub fn delete_group_chat(conn: &Connection, group_chat_id: &str) -> Result<(), rusqlite::Error> {
    // Get thread_id to cascade delete messages
    let thread_id: String = conn.query_row(
        "SELECT thread_id FROM group_chats WHERE group_chat_id = ?1",
        params![group_chat_id], |r| r.get(0),
    )?;
    conn.execute("DELETE FROM group_messages_fts WHERE thread_id = ?1", params![thread_id])?;
    conn.execute("DELETE FROM group_messages WHERE thread_id = ?1", params![thread_id])?;
    conn.execute("DELETE FROM memory_artifacts WHERE subject_id = ?1", params![thread_id])?;
    conn.execute("DELETE FROM message_count_tracker WHERE thread_id = ?1", params![thread_id])?;
    conn.execute("DELETE FROM group_chats WHERE group_chat_id = ?1", params![group_chat_id])?;
    conn.execute("DELETE FROM threads WHERE thread_id = ?1", params![thread_id])?;
    Ok(())
}

/// Find an existing group chat with exactly the same set of characters.
pub fn find_group_chat_by_members(conn: &Connection, world_id: &str, character_ids: &[String]) -> Option<GroupChat> {
    let mut sorted = character_ids.to_vec();
    sorted.sort();
    let sorted_json = serde_json::to_string(&sorted).unwrap_or_default();

    let group_chats = list_group_chats(conn, world_id).ok()?;
    for gc in group_chats {
        let mut gc_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        gc_ids.sort();
        if serde_json::to_string(&gc_ids).unwrap_or_default() == sorted_json {
            return Some(gc);
        }
    }
    None
}


// ─── Group Messages ─────────────────────────────────────────────────────────

pub fn create_group_message(conn: &Connection, m: &Message) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO group_messages (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time, address_to) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![m.message_id, m.thread_id, m.role, m.content, m.tokens_estimate, m.sender_character_id, m.created_at, m.world_day, m.world_time, m.address_to],
    )?;
    if m.role != "illustration" && m.role != "video" {
        conn.execute(
            "INSERT INTO group_messages_fts (message_id, thread_id, content) VALUES (?1, ?2, ?3)",
            params![m.message_id, m.thread_id, m.content],
        ).ok();
    }
    Ok(())
}

pub fn list_group_messages(conn: &Connection, thread_id: &str, limit: i64) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        &format!("SELECT {MSG_COLS} FROM group_messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT ?2")
    )?;
    let rows = stmt.query_map(params![thread_id, limit], row_to_message)?;
    let mut msgs: Vec<Message> = rows.collect::<Result<Vec<_>, _>>()?;
    msgs.reverse();
    Ok(msgs)
}

pub fn get_all_group_messages(conn: &Connection, thread_id: &str) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        &format!("SELECT {MSG_COLS} FROM group_messages WHERE thread_id = ?1 ORDER BY created_at ASC")
    )?;
    let rows = stmt.query_map(params![thread_id], row_to_message)?;
    rows.collect()
}

pub fn count_group_messages(conn: &Connection, thread_id: &str) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT count(*) FROM group_messages WHERE thread_id = ?1",
        params![thread_id],
        |r| r.get(0),
    )
}

pub fn delete_group_messages_after(conn: &Connection, thread_id: &str, after_message_id: &str) -> Result<(Vec<(String, String)>, Vec<String>), rusqlite::Error> {
    let anchor_rowid: i64 = conn.query_row(
        "SELECT rowid FROM group_messages WHERE message_id = ?1",
        params![after_message_id],
        |r| r.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT message_id, role FROM group_messages WHERE thread_id = ?1 AND rowid > ?2 ORDER BY rowid"
    )?;
    let deleted: Vec<(String, String)> = stmt.query_map(params![thread_id, anchor_rowid], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?.filter_map(|r| r.ok()).collect();

    if deleted.is_empty() {
        return Ok((deleted, vec![]));
    }

    let mut illustration_files: Vec<String> = Vec::new();

    for (msg_id, role) in &deleted {
        conn.execute("DELETE FROM group_messages_fts WHERE message_id = ?1", params![msg_id])?;
        conn.execute("DELETE FROM group_messages WHERE message_id = ?1", params![msg_id])?;

        if role == "illustration" {
            let file_name: Option<String> = conn.query_row(
                "SELECT file_name FROM world_images WHERE image_id = ?1",
                params![msg_id], |r| r.get(0),
            ).ok();
            conn.execute("DELETE FROM world_images WHERE image_id = ?1", params![msg_id])?;
            if let Some(f) = file_name {
                illustration_files.push(f);
            }
        }
    }

    conn.execute("DELETE FROM message_count_tracker WHERE thread_id = ?1", params![thread_id])?;
    conn.execute(
        "DELETE FROM memory_artifacts WHERE subject_id = ?1 AND artifact_type = 'thread_summary'",
        params![thread_id],
    )?;

    Ok((deleted, illustration_files))
}



use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// ─── Thread ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thread {
    pub thread_id: String,
    pub character_id: String,
    pub world_id: String,
    pub created_at: String,
}

pub fn create_thread(conn: &Connection, t: &Thread) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO threads (thread_id, character_id, world_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![t.thread_id, t.character_id, t.world_id, t.created_at],
    )?;
    Ok(())
}

pub fn get_thread_for_character(conn: &Connection, character_id: &str) -> Result<Thread, rusqlite::Error> {
    conn.query_row(
        "SELECT thread_id, character_id, world_id, created_at FROM threads WHERE character_id = ?1",
        params![character_id],
        |row| Ok(Thread { thread_id: row.get(0)?, character_id: row.get(1)?, world_id: row.get(2)?, created_at: row.get(3)? }),
    )
}


// ─── Message ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub message_id: String,
    pub thread_id: String,
    pub role: String,
    pub content: String,
    pub tokens_estimate: i64,
    pub sender_character_id: Option<String>,
    pub created_at: String,
    pub world_day: Option<i64>,
    pub world_time: Option<String>,
}

pub fn update_message_content(conn: &Connection, message_id: &str, content: &str, tokens_estimate: i64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE messages SET content = ?2, tokens_estimate = ?3 WHERE message_id = ?1",
        params![message_id, content, tokens_estimate],
    )?;
    // Update FTS
    conn.execute("DELETE FROM messages_fts WHERE message_id = ?1", params![message_id]).ok();
    conn.execute(
        "INSERT INTO messages_fts (message_id, thread_id, content) SELECT message_id, thread_id, ?2 FROM messages WHERE message_id = ?1",
        params![message_id, content],
    ).ok();
    Ok(())
}

pub fn update_group_message_content(conn: &Connection, message_id: &str, content: &str, tokens_estimate: i64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE group_messages SET content = ?2, tokens_estimate = ?3 WHERE message_id = ?1",
        params![message_id, content, tokens_estimate],
    )?;
    conn.execute("DELETE FROM group_messages_fts WHERE message_id = ?1", params![message_id]).ok();
    conn.execute(
        "INSERT INTO group_messages_fts (message_id, thread_id, content) SELECT message_id, thread_id, ?2 FROM group_messages WHERE message_id = ?1",
        params![message_id, content],
    ).ok();
    Ok(())
}

pub fn create_message(conn: &Connection, m: &Message) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO messages (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![m.message_id, m.thread_id, m.role, m.content, m.tokens_estimate, m.sender_character_id, m.created_at, m.world_day, m.world_time],
    )?;
    // Don't index illustration/video content in FTS — they contain binary data (base64)
    if m.role != "illustration" && m.role != "video" {
        conn.execute(
            "INSERT INTO messages_fts (message_id, thread_id, content) VALUES (?1, ?2, ?3)",
            params![m.message_id, m.thread_id, m.content],
        ).ok();
    }
    Ok(())
}

pub fn list_messages(conn: &Connection, thread_id: &str, limit: i64) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        &format!("SELECT {MSG_COLS} FROM messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT ?2")
    )?;
    let rows = stmt.query_map(params![thread_id, limit], row_to_message)?;
    let mut msgs: Vec<Message> = rows.collect::<Result<Vec<_>, _>>()?;
    msgs.reverse();
    Ok(msgs)
}

pub fn get_all_messages(conn: &Connection, thread_id: &str) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        &format!("SELECT {MSG_COLS} FROM messages WHERE thread_id = ?1 ORDER BY created_at ASC")
    )?;
    let rows = stmt.query_map(params![thread_id], row_to_message)?;
    rows.collect()
}

/// Returns the most recent `limit` messages, skipping the newest `offset`.
/// Result is in chronological order (oldest first).
pub fn list_messages_paginated(conn: &Connection, thread_id: &str, limit: i64, offset: i64) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        &format!("SELECT {MSG_COLS} FROM messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3")
    )?;
    let rows = stmt.query_map(params![thread_id, limit, offset], row_to_message)?;
    let mut msgs: Vec<Message> = rows.collect::<Result<Vec<_>, _>>()?;
    msgs.reverse();
    Ok(msgs)
}

pub fn count_messages(conn: &Connection, thread_id: &str) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT count(*) FROM messages WHERE thread_id = ?1",
        params![thread_id],
        |r| r.get(0),
    )
}

pub fn count_messages_since_maintenance(conn: &Connection, thread_id: &str) -> i64 {
    conn.query_row(
        "SELECT count_since_maintenance FROM message_count_tracker WHERE thread_id = ?1",
        params![thread_id],
        |r| r.get(0),
    ).unwrap_or(0)
}

pub fn increment_message_counter(conn: &Connection, thread_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO message_count_tracker (thread_id, count_since_maintenance) VALUES (?1, 1)
         ON CONFLICT(thread_id) DO UPDATE SET count_since_maintenance = count_since_maintenance + 1",
        params![thread_id],
    )?;
    Ok(())
}

pub fn reset_message_counter(conn: &Connection, thread_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE message_count_tracker SET count_since_maintenance = 0 WHERE thread_id = ?1",
        params![thread_id],
    )?;
    Ok(())
}

pub fn row_to_message(row: &rusqlite::Row) -> Result<Message, rusqlite::Error> {
    Ok(Message {
        message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
        content: row.get(3)?, tokens_estimate: row.get(4)?,
        sender_character_id: row.get(5)?,
        created_at: row.get(6)?,
        world_day: row.get(7).ok(),
        world_time: row.get(8).ok(),
    })
}

pub const MSG_COLS: &str = "message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time";


// ─── FTS Search ─────────────────────────────────────────────────────────────

pub fn search_messages_fts(conn: &Connection, thread_id: &str, query: &str, limit: i64) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT f.message_id, f.thread_id, m.role, f.content, m.tokens_estimate, m.sender_character_id, m.created_at
         FROM messages_fts f
         JOIN messages m ON m.message_id = f.message_id
         WHERE f.thread_id = ?1 AND messages_fts MATCH ?2
         ORDER BY rank LIMIT ?3"
    )?;
    let rows = stmt.query_map(params![thread_id, query, limit], row_to_message)?;
    rows.collect()
}




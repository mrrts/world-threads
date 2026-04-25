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
    // create_thread is solo-only (Thread.character_id is String, never
    // NULL). Group-chat shell threads are inserted directly in group.rs
    // with character_id = NULL and no current_location (the group_chats
    // row carries location for group surfaces). New solo chats default
    // to 'Town Square'; the schema migration backfills any pre-existing
    // NULL solo-thread rows to match.
    conn.execute(
        "INSERT INTO threads (thread_id, character_id, world_id, created_at, current_location) VALUES (?1, ?2, ?3, ?4, 'Town Square')",
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
    /// Who the speaker is addressing. NULL = unknown. "user" = the human.
    /// Otherwise a character_id. Used by group-chat flows to make addressee
    /// explicit in the history rendered to the model.
    #[serde(default)]
    pub address_to: Option<String>,
    /// The emoji chain that seeded this reply's AGENCY section, stored as a
    /// JSON array string. Only populated on assistant-role messages; NULL
    /// for user messages and for anything pre-dating the feature. Feeds the
    /// measurement loop (which chains correlate with positive reactions).
    #[serde(default)]
    pub mood_chain: Option<String>,
    /// Set to true on assistant messages emitted as proactive pings (the
    /// character reaching out first, unprompted). Used by the UI to style
    /// them distinctly and by sidebar to surface unread badges.
    #[serde(default)]
    pub is_proactive: bool,
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
        "INSERT INTO messages (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time, address_to, mood_chain, is_proactive) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![m.message_id, m.thread_id, m.role, m.content, m.tokens_estimate, m.sender_character_id, m.created_at, m.world_day, m.world_time, m.address_to, m.mood_chain, m.is_proactive as i64],
    )?;
    // Don't index illustration/video content in FTS — they contain binary data (base64)
    if m.role != "illustration" && m.role != "video" && m.role != "inventory_update" {
        conn.execute(
            "INSERT INTO messages_fts (message_id, thread_id, content) VALUES (?1, ?2, ?3)",
            params![m.message_id, m.thread_id, m.content],
        ).ok();
    }
    // A user reply "answers" any outstanding proactive ping, so the
    // consecutive counter resets. Done here rather than at every call site
    // so it's impossible to forget.
    if m.role == "user" {
        conn.execute(
            "UPDATE threads SET consecutive_proactive_pings = 0 WHERE thread_id = ?1",
            params![m.thread_id],
        ).ok();
    }
    Ok(())
}

// ─── Proactive pings ────────────────────────────────────────────────────────

/// Per-thread state used to decide whether a proactive ping is allowed right
/// now and to enforce the "max 2 consecutive" rule.
#[derive(Debug, Clone)]
pub struct ProactivePingState {
    pub consecutive: i64,
    pub last_at: Option<String>,
}

pub fn get_proactive_ping_state(conn: &Connection, thread_id: &str) -> ProactivePingState {
    conn.query_row(
        "SELECT consecutive_proactive_pings, last_proactive_ping_at FROM threads WHERE thread_id = ?1",
        params![thread_id],
        |r| Ok(ProactivePingState { consecutive: r.get(0)?, last_at: r.get(1).ok() }),
    ).unwrap_or(ProactivePingState { consecutive: 0, last_at: None })
}

/// Increment the consecutive counter and stamp the last-ping timestamp.
/// Called only after a ping has been successfully persisted.
pub fn record_proactive_ping(conn: &Connection, thread_id: &str, at_iso: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE threads
           SET consecutive_proactive_pings = consecutive_proactive_pings + 1,
               last_proactive_ping_at = ?2
         WHERE thread_id = ?1",
        params![thread_id, at_iso],
    )?;
    Ok(())
}

/// Counts proactive assistant messages in the thread that are newer than the
/// most recent user message. Used to surface an unread badge in the sidebar.
pub fn count_unread_proactive_since_last_user(conn: &Connection, thread_id: &str) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM messages
         WHERE thread_id = ?1
           AND is_proactive = 1
           AND created_at > COALESCE(
             (SELECT MAX(created_at) FROM messages WHERE thread_id = ?1 AND role = 'user'),
             '0000-00-00'
           )",
        params![thread_id],
        |r| r.get(0),
    ).unwrap_or(0)
}

/// Read the per-thread mood-reduction ring buffer (most-recent-first JSON
/// array of reaction emojis). Returns an empty Vec if the column is NULL,
/// unparseable, or the thread doesn't exist.
pub fn get_thread_mood_reduction(conn: &Connection, thread_id: &str) -> Vec<String> {
    let raw: Option<String> = conn.query_row(
        "SELECT mood_reduction FROM threads WHERE thread_id = ?1",
        params![thread_id],
        |r| r.get(0),
    ).ok();
    match raw {
        Some(s) => serde_json::from_str::<Vec<String>>(&s).unwrap_or_default(),
        None => Vec::new(),
    }
}

/// Push an emoji onto the thread's mood reduction. Most-recent-first,
/// deduped within the buffer, capped at `MAX_MOOD_REDUCTION` entries.
pub fn push_mood_reduction(conn: &Connection, thread_id: &str, emoji: &str) -> Result<(), rusqlite::Error> {
    const MAX_MOOD_REDUCTION: usize = 8;
    let mut current = get_thread_mood_reduction(conn, thread_id);
    // Remove any existing occurrence — emoji migrates to the front.
    current.retain(|e| e != emoji);
    current.insert(0, emoji.to_string());
    if current.len() > MAX_MOOD_REDUCTION {
        current.truncate(MAX_MOOD_REDUCTION);
    }
    let json = serde_json::to_string(&current).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "UPDATE threads SET mood_reduction = ?2 WHERE thread_id = ?1",
        params![thread_id, json],
    )?;
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

/// Fetch as many recent messages as will fit in `token_budget` based on
/// their stored `tokens_estimate`. Guarantees at least `min_messages` even
/// if the budget would cut shorter — small threads shouldn't lose context
/// just because of stingy accounting. Caps at `SAFETY_MAX` to avoid
/// degenerate cases where old messages have zero token estimates.
/// Returns chronologically (oldest first) like `list_messages`.
pub fn list_messages_within_budget(
    conn: &Connection,
    thread_id: &str,
    token_budget: i64,
    min_messages: i64,
) -> Result<Vec<Message>, rusqlite::Error> {
    const SAFETY_MAX: i64 = 500;
    let mut stmt = conn.prepare(
        &format!("SELECT {MSG_COLS} FROM messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT ?2")
    )?;
    let mut rows = stmt.query(params![thread_id, SAFETY_MAX])?;
    let mut out: Vec<Message> = Vec::new();
    let mut accumulated: i64 = 0;
    while let Some(row) = rows.next()? {
        let msg = row_to_message(row)?;
        accumulated += msg.tokens_estimate.max(0);
        out.push(msg);
        if (out.len() as i64) >= min_messages && accumulated >= token_budget {
            break;
        }
    }
    out.reverse();
    Ok(out)
}

/// Fetch all messages from a thread tagged with a specific in-world day.
/// Used by dream generation so the dream compresses *that day's* material,
/// not whatever happens to fit in the token budget. Returns chronologically
/// (oldest first). Messages whose `world_day` is NULL are excluded — if
/// the world has no clock, there's no in-world day to scope to and the
/// caller should fall back to the budget-based path.
pub fn list_messages_for_world_day(
    conn: &Connection,
    thread_id: &str,
    world_day: i64,
) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        &format!("SELECT {MSG_COLS} FROM messages WHERE thread_id = ?1 AND world_day = ?2 ORDER BY created_at ASC")
    )?;
    let rows = stmt.query_map(params![thread_id, world_day], row_to_message)?;
    rows.collect()
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
        address_to: row.get(9).ok(),
        mood_chain: row.get(10).ok(),
        is_proactive: row.get::<_, i64>(11).map(|v| v != 0).unwrap_or(false),
    })
}

pub const MSG_COLS: &str = "message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time, address_to, mood_chain, is_proactive";


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




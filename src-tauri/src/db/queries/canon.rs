use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// A persisted canonization event. See schema comments for field semantics.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeptRecord {
    pub kept_id: String,
    pub source_message_id: Option<String>,
    pub source_thread_id: Option<String>,
    pub source_world_day: Option<i64>,
    pub source_created_at: Option<String>,
    pub subject_type: String,
    pub subject_id: String,
    pub record_type: String,
    pub content: String,
    #[serde(default)]
    pub user_note: String,
    pub created_at: String,
}

pub fn create_kept_record(conn: &Connection, e: &KeptRecord) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO kept_records (kept_id, source_message_id, source_thread_id, source_world_day, source_created_at, subject_type, subject_id, record_type, content, user_note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            e.kept_id, e.source_message_id, e.source_thread_id, e.source_world_day, e.source_created_at,
            e.subject_type, e.subject_id, e.record_type, e.content, e.user_note, e.created_at,
        ],
    )?;
    Ok(())
}

/// Fetch the distinct message IDs in a thread that have been canonized at
/// least once. Drives the per-message "this moment is canon" indicator.
///
/// Resolves membership in the thread by joining against the actual
/// messages / group_messages tables rather than trusting
/// kept_records.source_thread_id. The denormalized source_thread_id
/// column is populated at write-time and was occasionally getting left
/// NULL when find_message couldn't resolve the source — which silently
/// hid the highlight on freshly-canonized messages. Joining directly
/// fixes this regardless of the denormalized column's state.
pub fn list_kept_message_ids_for_thread(
    conn: &Connection,
    thread_id: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT kr.source_message_id
         FROM kept_records kr
         WHERE kr.source_message_id IS NOT NULL
           AND (
             EXISTS (SELECT 1 FROM messages       m  WHERE m.message_id  = kr.source_message_id AND m.thread_id  = ?1)
             OR
             EXISTS (SELECT 1 FROM group_messages gm WHERE gm.message_id = kr.source_message_id AND gm.thread_id = ?1)
           )"
    )?;
    let rows = stmt.query_map(params![thread_id], |r| r.get::<_, String>(0))?;
    rows.collect()
}

/// All canon entries tied to a specific source message. Used when hovering
/// a canonized indicator to show what the message was promoted to.
pub fn list_kept_for_message(
    conn: &Connection,
    message_id: &str,
) -> Result<Vec<KeptRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT kept_id, source_message_id, source_thread_id, source_world_day, source_created_at,
                subject_type, subject_id, record_type, content, user_note, created_at
         FROM kept_records WHERE source_message_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![message_id], |r| Ok(KeptRecord {
        kept_id: r.get(0)?,
        source_message_id: r.get(1)?,
        source_thread_id: r.get(2)?,
        source_world_day: r.get(3)?,
        source_created_at: r.get(4)?,
        subject_type: r.get(5)?,
        subject_id: r.get(6)?,
        record_type: r.get(7)?,
        content: r.get(8)?,
        user_note: r.get(9)?,
        created_at: r.get(10)?,
    }))?;
    rows.collect()
}

pub fn delete_kept_record(conn: &Connection, kept_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM kept_records WHERE kept_id = ?1", params![kept_id])?;
    Ok(())
}

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// A user-ratified pursuit in a world — "a promise the world has made
/// to itself that the human has agreed to witness," per the design
/// frame. Intentionally a RECORD, not a gamified objective — no
/// progress tracking, no deadlines, no badges.
///
/// Lifecycle: accepted → (active) → EITHER completed OR abandoned.
/// Both completion and abandonment are explicit user acts (or Backstage-
/// proposed + user-ratified), each with its own ceremony and its own
/// note. Completion and abandonment are BOTH terminal and BOTH visible
/// afterward — they're part of the world's accumulated history, not
/// archived to silence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub quest_id: String,
    pub world_id: String,
    pub title: String,
    pub description: String,
    /// User's running commentary as the quest evolves.
    pub notes: String,
    pub accepted_at: String,
    pub accepted_world_day: Option<i64>,
    pub completed_at: Option<String>,
    pub completed_world_day: Option<i64>,
    pub completion_note: String,
    pub abandoned_at: Option<String>,
    pub abandoned_world_day: Option<i64>,
    pub abandonment_note: String,
    /// Where the quest came from. One of:
    ///   "user_authored"   — user typed it in themselves
    ///   "message"         — promoted from a specific chat message
    ///   "meanwhile"       — promoted from a meanwhile event
    ///   "backstage"       — proposed by Backstage as an action card
    pub origin_kind: String,
    /// The id of the originating artifact (message_id, event_id, etc.),
    /// or None for user_authored.
    pub origin_ref: Option<String>,
}

pub fn create_quest(conn: &Connection, q: &Quest) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO quests
           (quest_id, world_id, title, description, notes,
            accepted_at, accepted_world_day,
            completed_at, completed_world_day, completion_note,
            abandoned_at, abandoned_world_day, abandonment_note,
            origin_kind, origin_ref)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            q.quest_id,
            q.world_id,
            q.title,
            q.description,
            q.notes,
            q.accepted_at,
            q.accepted_world_day,
            q.completed_at,
            q.completed_world_day,
            q.completion_note,
            q.abandoned_at,
            q.abandoned_world_day,
            q.abandonment_note,
            q.origin_kind,
            q.origin_ref,
        ],
    )?;
    Ok(())
}

fn row_to_quest(r: &rusqlite::Row) -> rusqlite::Result<Quest> {
    Ok(Quest {
        quest_id: r.get(0)?,
        world_id: r.get(1)?,
        title: r.get(2)?,
        description: r.get(3)?,
        notes: r.get(4)?,
        accepted_at: r.get(5)?,
        accepted_world_day: r.get(6)?,
        completed_at: r.get(7)?,
        completed_world_day: r.get(8)?,
        completion_note: r.get(9)?,
        abandoned_at: r.get(10)?,
        abandoned_world_day: r.get(11)?,
        abandonment_note: r.get(12)?,
        origin_kind: r.get(13)?,
        origin_ref: r.get(14)?,
    })
}

const QUEST_COLS: &str = "quest_id, world_id, title, description, notes,
    accepted_at, accepted_world_day,
    completed_at, completed_world_day, completion_note,
    abandoned_at, abandoned_world_day, abandonment_note,
    origin_kind, origin_ref";

/// All quests for a world — active first (by accepted_at desc), then
/// completed/abandoned (most recently-terminated first). Read-order
/// matches how the frontend displays them.
pub fn list_quests(conn: &Connection, world_id: &str) -> Result<Vec<Quest>, rusqlite::Error> {
    let sql = format!(
        "SELECT {QUEST_COLS}
         FROM quests
         WHERE world_id = ?1
         ORDER BY
           CASE WHEN completed_at IS NULL AND abandoned_at IS NULL THEN 0 ELSE 1 END,
           CASE WHEN completed_at IS NULL AND abandoned_at IS NULL THEN accepted_at END DESC,
           COALESCE(completed_at, abandoned_at) DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![world_id], row_to_quest)?;
    rows.collect()
}

/// Active quests only — what characters implicitly know about.
/// Excludes both completed and abandoned.
pub fn list_active_quests(
    conn: &Connection,
    world_id: &str,
) -> Result<Vec<Quest>, rusqlite::Error> {
    let sql = format!(
        "SELECT {QUEST_COLS}
         FROM quests
         WHERE world_id = ?1 AND completed_at IS NULL AND abandoned_at IS NULL
         ORDER BY accepted_at ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![world_id], row_to_quest)?;
    rows.collect()
}

pub fn get_quest(conn: &Connection, quest_id: &str) -> Result<Quest, rusqlite::Error> {
    let sql = format!("SELECT {QUEST_COLS} FROM quests WHERE quest_id = ?1");
    conn.query_row(&sql, params![quest_id], row_to_quest)
}

pub fn update_quest_title_description(
    conn: &Connection,
    quest_id: &str,
    title: &str,
    description: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE quests SET title = ?2, description = ?3 WHERE quest_id = ?1",
        params![quest_id, title, description],
    )?;
    Ok(())
}

pub fn update_quest_notes(
    conn: &Connection,
    quest_id: &str,
    notes: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE quests SET notes = ?2 WHERE quest_id = ?1",
        params![quest_id, notes],
    )?;
    Ok(())
}

pub fn mark_quest_complete(
    conn: &Connection,
    quest_id: &str,
    completed_at: &str,
    completed_world_day: Option<i64>,
    completion_note: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE quests
         SET completed_at = ?2,
             completed_world_day = ?3,
             completion_note = ?4,
             abandoned_at = NULL,
             abandoned_world_day = NULL,
             abandonment_note = ''
         WHERE quest_id = ?1",
        params![quest_id, completed_at, completed_world_day, completion_note],
    )?;
    Ok(())
}

pub fn mark_quest_abandoned(
    conn: &Connection,
    quest_id: &str,
    abandoned_at: &str,
    abandoned_world_day: Option<i64>,
    abandonment_note: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE quests
         SET abandoned_at = ?2,
             abandoned_world_day = ?3,
             abandonment_note = ?4,
             completed_at = NULL,
             completed_world_day = NULL,
             completion_note = ''
         WHERE quest_id = ?1",
        params![
            quest_id,
            abandoned_at,
            abandoned_world_day,
            abandonment_note
        ],
    )?;
    Ok(())
}

pub fn reopen_quest(conn: &Connection, quest_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE quests
         SET completed_at = NULL,
             completed_world_day = NULL,
             completion_note = '',
             abandoned_at = NULL,
             abandoned_world_day = NULL,
             abandonment_note = ''
         WHERE quest_id = ?1",
        params![quest_id],
    )?;
    Ok(())
}

pub fn delete_quest(conn: &Connection, quest_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM quests WHERE quest_id = ?1", params![quest_id])?;
    Ok(())
}

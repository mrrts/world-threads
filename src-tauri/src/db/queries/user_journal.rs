use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// The in-world player's own private journal entry — written as the
/// user's character, retrospecting a single closed world-day across
/// EVERY chat they were in that day. Parallel to `character_journals`,
/// but keyed by `world_id` since the player is one-per-world, not a
/// character row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJournalEntry {
    pub journal_id: String,
    pub world_id: String,
    pub world_day: i64,
    pub content: String,
    pub created_at: String,
}

pub fn upsert_user_journal_entry(
    conn: &Connection,
    entry: &UserJournalEntry,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO user_journals (journal_id, world_id, world_day, content, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(world_id, world_day) DO UPDATE SET
           content = excluded.content,
           created_at = excluded.created_at",
        params![entry.journal_id, entry.world_id, entry.world_day, entry.content, entry.created_at],
    )?;
    Ok(())
}

pub fn list_user_journal_entries(
    conn: &Connection,
    world_id: &str,
    limit: usize,
) -> Result<Vec<UserJournalEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT journal_id, world_id, world_day, content, created_at
         FROM user_journals
         WHERE world_id = ?1
         ORDER BY world_day DESC, created_at DESC
         LIMIT ?2"
    )?;
    let rows = stmt.query_map(params![world_id, limit as i64], |r| Ok(UserJournalEntry {
        journal_id: r.get(0)?,
        world_id: r.get(1)?,
        world_day: r.get(2)?,
        content: r.get(3)?,
        created_at: r.get(4)?,
    }))?;
    rows.collect()
}

pub fn list_user_journal_entries_before(
    conn: &Connection,
    world_id: &str,
    before_world_day: i64,
    limit: usize,
) -> Result<Vec<UserJournalEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT journal_id, world_id, world_day, content, created_at
         FROM user_journals
         WHERE world_id = ?1 AND world_day < ?2
         ORDER BY world_day DESC, created_at DESC
         LIMIT ?3"
    )?;
    let rows = stmt.query_map(params![world_id, before_world_day, limit as i64], |r| Ok(UserJournalEntry {
        journal_id: r.get(0)?,
        world_id: r.get(1)?,
        world_day: r.get(2)?,
        content: r.get(3)?,
        created_at: r.get(4)?,
    }))?;
    rows.collect()
}

pub fn get_user_journal_entry_for_day(
    conn: &Connection,
    world_id: &str,
    world_day: i64,
) -> Option<UserJournalEntry> {
    conn.query_row(
        "SELECT journal_id, world_id, world_day, content, created_at
         FROM user_journals
         WHERE world_id = ?1 AND world_day = ?2",
        params![world_id, world_day],
        |r| Ok(UserJournalEntry {
            journal_id: r.get(0)?,
            world_id: r.get(1)?,
            world_day: r.get(2)?,
            content: r.get(3)?,
            created_at: r.get(4)?,
        }),
    ).ok()
}

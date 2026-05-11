use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub journal_id: String,
    pub character_id: String,
    pub world_day: i64,
    pub content: String,
    pub created_at: String,
}

/// Upsert an entry for (character_id, world_day). ON CONFLICT REPLACE
/// semantics mean re-running the generate button for today overwrites
/// today's entry instead of accumulating — one per char per day.
/// Phase 2 thread-through (batch-4): user_id now populated on INSERT.
/// user_id is identity-stable on conflict.
pub fn upsert_journal_entry(
    conn: &Connection,
    entry: &JournalEntry,
    user_id: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO character_journals (journal_id, character_id, world_day, content, created_at, user_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(character_id, world_day) DO UPDATE SET
           content = excluded.content,
           created_at = excluded.created_at",
        params![
            entry.journal_id,
            entry.character_id,
            entry.world_day,
            entry.content,
            entry.created_at,
            user_id,
        ],
    )?;
    Ok(())
}

/// Most-recent N entries for a character (newest first).
pub fn list_journal_entries(
    conn: &Connection,
    character_id: &str,
    limit: usize,
) -> Result<Vec<JournalEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT journal_id, character_id, world_day, content, created_at
         FROM character_journals
         WHERE character_id = ?1
         ORDER BY world_day DESC, created_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![character_id, limit as i64], |r| {
        Ok(JournalEntry {
            journal_id: r.get(0)?,
            character_id: r.get(1)?,
            world_day: r.get(2)?,
            content: r.get(3)?,
            created_at: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// Most-recent N entries BEFORE a given world-day. Used by the journal
/// prompt to fetch "yesterday and the day before" when writing a new
/// entry — explicitly excludes the current day so a regenerate doesn't
/// recycle its own previous version as prior context.
pub fn list_journal_entries_before(
    conn: &Connection,
    character_id: &str,
    before_world_day: i64,
    limit: usize,
) -> Result<Vec<JournalEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT journal_id, character_id, world_day, content, created_at
         FROM character_journals
         WHERE character_id = ?1 AND world_day < ?2
         ORDER BY world_day DESC, created_at DESC
         LIMIT ?3",
    )?;
    let rows = stmt.query_map(params![character_id, before_world_day, limit as i64], |r| {
        Ok(JournalEntry {
            journal_id: r.get(0)?,
            character_id: r.get(1)?,
            world_day: r.get(2)?,
            content: r.get(3)?,
            created_at: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// Fetch the single entry for a specific (character, world_day), if any.
pub fn get_journal_entry_for_day(
    conn: &Connection,
    character_id: &str,
    world_day: i64,
) -> Option<JournalEntry> {
    conn.query_row(
        "SELECT journal_id, character_id, world_day, content, created_at
         FROM character_journals
         WHERE character_id = ?1 AND world_day = ?2",
        params![character_id, world_day],
        |r| {
            Ok(JournalEntry {
                journal_id: r.get(0)?,
                character_id: r.get(1)?,
                world_day: r.get(2)?,
                content: r.get(3)?,
                created_at: r.get(4)?,
            })
        },
    )
    .ok()
}

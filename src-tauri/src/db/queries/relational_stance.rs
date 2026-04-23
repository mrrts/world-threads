use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationalStance {
    pub stance_id: String,
    pub character_id: String,
    pub world_id: String,
    pub stance_text: String,
    pub world_day_at_generation: Option<i64>,
    pub source_kept_record_count: i64,
    pub source_journal_count: i64,
    pub source_message_count: i64,
    pub refresh_trigger: String,
    pub model_used: String,
    pub created_at: String,
}

pub fn insert_relational_stance(
    conn: &Connection,
    s: &RelationalStance,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO relational_stances (
            stance_id, character_id, world_id, stance_text,
            world_day_at_generation, source_kept_record_count,
            source_journal_count, source_message_count,
            refresh_trigger, model_used, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            s.stance_id, s.character_id, s.world_id, s.stance_text,
            s.world_day_at_generation, s.source_kept_record_count,
            s.source_journal_count, s.source_message_count,
            s.refresh_trigger, s.model_used, s.created_at,
        ],
    )?;
    Ok(())
}

/// Most recent stance row for a character, or None if never generated.
pub fn latest_relational_stance(
    conn: &Connection,
    character_id: &str,
) -> Result<Option<RelationalStance>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT stance_id, character_id, world_id, stance_text,
                world_day_at_generation, source_kept_record_count,
                source_journal_count, source_message_count,
                refresh_trigger, model_used, created_at
         FROM relational_stances
         WHERE character_id = ?1
         ORDER BY created_at DESC
         LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![character_id], |r| {
        Ok(RelationalStance {
            stance_id: r.get(0)?,
            character_id: r.get(1)?,
            world_id: r.get(2)?,
            stance_text: r.get(3)?,
            world_day_at_generation: r.get(4)?,
            source_kept_record_count: r.get(5)?,
            source_journal_count: r.get(6)?,
            source_message_count: r.get(7)?,
            refresh_trigger: r.get(8)?,
            model_used: r.get(9)?,
            created_at: r.get(10)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Full history for a character, newest first. Used by debug/inspect
/// surfaces (worldcli) and any future analysis of stance evolution.
pub fn list_relational_stances(
    conn: &Connection,
    character_id: &str,
    limit: i64,
) -> Result<Vec<RelationalStance>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT stance_id, character_id, world_id, stance_text,
                world_day_at_generation, source_kept_record_count,
                source_journal_count, source_message_count,
                refresh_trigger, model_used, created_at
         FROM relational_stances
         WHERE character_id = ?1
         ORDER BY created_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![character_id, limit], |r| {
        Ok(RelationalStance {
            stance_id: r.get(0)?,
            character_id: r.get(1)?,
            world_id: r.get(2)?,
            stance_text: r.get(3)?,
            world_day_at_generation: r.get(4)?,
            source_kept_record_count: r.get(5)?,
            source_journal_count: r.get(6)?,
            source_message_count: r.get(7)?,
            refresh_trigger: r.get(8)?,
            model_used: r.get(9)?,
            created_at: r.get(10)?,
        })
    })?;
    rows.collect()
}

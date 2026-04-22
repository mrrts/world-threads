use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanwhileEvent {
    pub event_id: String,
    pub world_id: String,
    pub character_id: String,
    pub world_day: i64,
    pub time_of_day: String,
    pub summary: String,
    pub created_at: String,
}

pub fn create_meanwhile_event(conn: &Connection, e: &MeanwhileEvent) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO meanwhile_events (event_id, world_id, character_id, world_day, time_of_day, summary, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![e.event_id, e.world_id, e.character_id, e.world_day, e.time_of_day, e.summary, e.created_at],
    )?;
    Ok(())
}

/// Most-recent N events for a world, newest first. Joins character
/// display_name so the frontend doesn't need a second lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanwhileEventWithName {
    pub event_id: String,
    pub character_id: String,
    pub character_name: String,
    pub avatar_color: String,
    pub world_day: i64,
    pub time_of_day: String,
    pub summary: String,
    pub created_at: String,
}

/// Most-recent meanwhile event for a single character, limited to the
/// last `within_hours` hours of wall-clock time. Returns None when the
/// character has no recent off-screen beat. Used by the dialogue
/// orchestrator to give each character a concrete "what you were just
/// doing" context to carry into a new reply.
pub fn latest_meanwhile_for_character(
    conn: &Connection,
    character_id: &str,
    within_hours: i64,
) -> Option<MeanwhileEvent> {
    let sql = "SELECT event_id, world_id, character_id, world_day, time_of_day, summary, created_at
               FROM meanwhile_events
               WHERE character_id = ?1
                 AND created_at >= datetime('now', ?2)
               ORDER BY created_at DESC
               LIMIT 1";
    let delta = format!("-{within_hours} hours");
    conn.query_row(sql, params![character_id, delta], |r| {
        Ok(MeanwhileEvent {
            event_id: r.get(0)?,
            world_id: r.get(1)?,
            character_id: r.get(2)?,
            world_day: r.get(3)?,
            time_of_day: r.get(4)?,
            summary: r.get(5)?,
            created_at: r.get(6)?,
        })
    }).ok()
}

pub fn list_meanwhile_events(
    conn: &Connection,
    world_id: &str,
    limit: usize,
) -> Result<Vec<MeanwhileEventWithName>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT m.event_id, m.character_id,
                COALESCE(c.display_name, ''),
                COALESCE(c.avatar_color, '#c4a882'),
                m.world_day, m.time_of_day, m.summary, m.created_at
         FROM meanwhile_events m
         LEFT JOIN characters c ON c.character_id = m.character_id
         WHERE m.world_id = ?1
         ORDER BY m.created_at DESC
         LIMIT ?2"
    )?;
    let rows = stmt.query_map(params![world_id, limit as i64], |r| Ok(MeanwhileEventWithName {
        event_id: r.get(0)?,
        character_id: r.get(1)?,
        character_name: r.get(2)?,
        avatar_color: r.get(3)?,
        world_day: r.get(4)?,
        time_of_day: r.get(5)?,
        summary: r.get(6)?,
        created_at: r.get(7)?,
    }))?;
    rows.collect()
}

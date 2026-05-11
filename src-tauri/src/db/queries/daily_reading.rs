use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// One reading cell — a mix of quantitative (percent, for the HUD bar)
/// and qualitative (phrase, for texture). Lets the visualization feel
/// like a dashboard without collapsing meaning into just a number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingDomain {
    pub name: String,
    pub percent: i32,
    pub phrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReading {
    pub reading_id: String,
    pub world_id: String,
    pub world_day: i64,
    pub domains: Vec<ReadingDomain>,
    pub complication: String,
    pub created_at: String,
}

/// Phase 2 thread-through (batch-4): user_id now populated on INSERT.
/// user_id is identity-stable on conflict.
pub fn upsert_daily_reading(
    conn: &Connection,
    reading: &DailyReading,
    user_id: &str,
) -> Result<(), rusqlite::Error> {
    let domains_json = serde_json::to_string(&reading.domains).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO daily_readings (reading_id, world_id, world_day, domains, complication, created_at, user_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(world_id, world_day) DO UPDATE SET
           domains = excluded.domains,
           complication = excluded.complication,
           created_at = excluded.created_at",
        params![reading.reading_id, reading.world_id, reading.world_day,
            domains_json, reading.complication, reading.created_at, user_id],
    )?;
    Ok(())
}

pub fn get_daily_reading_for_day(
    conn: &Connection,
    world_id: &str,
    world_day: i64,
) -> Option<DailyReading> {
    conn.query_row(
        "SELECT reading_id, world_id, world_day, domains, complication, created_at
         FROM daily_readings
         WHERE world_id = ?1 AND world_day = ?2",
        params![world_id, world_day],
        |r| {
            let domains_json: String = r.get(3)?;
            Ok(DailyReading {
                reading_id: r.get(0)?,
                world_id: r.get(1)?,
                world_day: r.get(2)?,
                domains: serde_json::from_str(&domains_json).unwrap_or_default(),
                complication: r.get(4)?,
                created_at: r.get(5)?,
            })
        },
    )
    .ok()
}

pub fn list_daily_readings(
    conn: &Connection,
    world_id: &str,
    limit: usize,
) -> Result<Vec<DailyReading>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT reading_id, world_id, world_day, domains, complication, created_at
         FROM daily_readings
         WHERE world_id = ?1
         ORDER BY world_day DESC, created_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![world_id, limit as i64], |r| {
        let domains_json: String = r.get(3)?;
        Ok(DailyReading {
            reading_id: r.get(0)?,
            world_id: r.get(1)?,
            world_day: r.get(2)?,
            domains: serde_json::from_str(&domains_json).unwrap_or_default(),
            complication: r.get(4)?,
            created_at: r.get(5)?,
        })
    })?;
    rows.collect()
}

/// Gather ALL messages from the world for a specific world-day,
/// merged across every solo + group thread in the world, chronological.
/// Speaker labels are resolved from `character_id → display_name`.
/// Used by the daily-reading chain so one pass can see the whole day
/// across every conversation in the world.
pub fn gather_world_messages_for_world_day(
    conn: &Connection,
    world_id: &str,
    user_display_name: &str,
    world_day: i64,
) -> Vec<(String, String, String)> {
    // (speaker, content, created_at)
    let mut names: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if let Ok(mut stmt) =
        conn.prepare("SELECT character_id, display_name FROM characters WHERE world_id = ?1")
    {
        if let Ok(rows) = stmt.query_map(params![world_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                names.insert(row.0, row.1);
            }
        }
    }

    let mut all: Vec<(String, String, String)> = Vec::new();

    // Solo thread messages: threads whose character belongs to this world.
    if let Ok(mut stmt) = conn.prepare(
        "SELECT m.role, m.content, m.created_at, m.sender_character_id, t.character_id
         FROM messages m
         JOIN threads t ON t.thread_id = m.thread_id
         WHERE t.world_id = ?1
           AND m.world_day = ?2
           AND m.role NOT IN ('illustration', 'video', 'system', 'context', 'inventory_update')",
    ) {
        if let Ok(rows) = stmt.query_map(params![world_id, world_day], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        }) {
            for (role, content, created_at, sender_id, thread_char_id) in rows.flatten() {
                let speaker = match role.as_str() {
                    "user" => user_display_name.to_string(),
                    "narrative" => "Narrator".to_string(),
                    "dream" => "Dream".to_string(),
                    _ => {
                        let id = sender_id.or(thread_char_id);
                        id.as_deref()
                            .and_then(|i| names.get(i).cloned())
                            .unwrap_or_else(|| "Character".to_string())
                    }
                };
                all.push((speaker, content, created_at));
            }
        }
    }

    // Group thread messages.
    if let Ok(mut stmt) = conn.prepare(
        "SELECT m.role, m.content, m.created_at, m.sender_character_id
         FROM group_messages m
         JOIN group_chats gc ON gc.thread_id = m.thread_id
         WHERE gc.world_id = ?1
           AND m.world_day = ?2
           AND m.role NOT IN ('illustration', 'video', 'system', 'context', 'inventory_update')",
    ) {
        if let Ok(rows) = stmt.query_map(params![world_id, world_day], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        }) {
            for (role, content, created_at, sender_id) in rows.flatten() {
                let speaker = match role.as_str() {
                    "user" => user_display_name.to_string(),
                    "narrative" => "Narrator".to_string(),
                    _ => sender_id
                        .as_deref()
                        .and_then(|i| names.get(i).cloned())
                        .unwrap_or_else(|| "Character".to_string()),
                };
                all.push((speaker, content, created_at));
            }
        }
    }

    all.sort_by(|a, b| a.2.cmp(&b.2));
    all
}

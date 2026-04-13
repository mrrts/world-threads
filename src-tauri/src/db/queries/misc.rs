use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ─── Token Usage ────────────────────────────────────────────────────────────

pub fn record_token_usage(
    conn: &Connection, call_type: &str, model: &str, prompt_tokens: u32, completion_tokens: u32,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO token_usage (call_type, model, prompt_tokens, completion_tokens) VALUES (?1, ?2, ?3, ?4)",
        params![call_type, model, prompt_tokens, completion_tokens],
    )?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailyUsage {
    pub date: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
}

pub fn get_today_usage(conn: &Connection) -> DailyUsage {
    let result = conn.query_row(
        "SELECT COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0) FROM token_usage WHERE date(created_at) = date('now')",
        [],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
    );
    let (prompt, completion) = result.unwrap_or((0, 0));
    DailyUsage {
        date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        prompt_tokens: prompt,
        completion_tokens: completion,
    }
}


// ─── Reactions ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub reaction_id: String,
    pub message_id: String,
    pub emoji: String,
    pub reactor: String,
    pub created_at: String,
}

pub fn add_reaction(conn: &Connection, r: &Reaction) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO reactions (reaction_id, message_id, emoji, reactor, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![r.reaction_id, r.message_id, r.emoji, r.reactor, r.created_at],
    )?;
    Ok(())
}

pub fn remove_reaction(conn: &Connection, message_id: &str, emoji: &str, reactor: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM reactions WHERE message_id = ?1 AND emoji = ?2 AND reactor = ?3",
        params![message_id, emoji, reactor],
    )?;
    Ok(())
}

pub fn get_reactions_for_messages(conn: &Connection, message_ids: &[String]) -> Result<Vec<Reaction>, rusqlite::Error> {
    if message_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders: Vec<String> = (1..=message_ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT reaction_id, message_id, emoji, reactor, created_at FROM reactions WHERE message_id IN ({}) ORDER BY created_at",
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::types::ToSql> = message_ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(Reaction {
            reaction_id: row.get(0)?, message_id: row.get(1)?, emoji: row.get(2)?,
            reactor: row.get(3)?, created_at: row.get(4)?,
        })
    })?;
    rows.collect()
}


// ─── Character Mood ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterMood {
    pub character_id: String,
    pub valence: f64,
    pub energy: f64,
    pub tension: f64,
    pub history: Value,
    pub updated_at: String,
}

pub fn get_character_mood(conn: &Connection, character_id: &str) -> Option<CharacterMood> {
    conn.query_row(
        "SELECT character_id, valence, energy, tension, history, updated_at FROM character_mood WHERE character_id = ?1",
        params![character_id],
        |row| Ok(CharacterMood {
            character_id: row.get(0)?,
            valence: row.get(1)?,
            energy: row.get(2)?,
            tension: row.get(3)?,
            history: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(Value::Array(vec![])),
            updated_at: row.get(5)?,
        }),
    ).ok()
}

pub fn upsert_character_mood(conn: &Connection, mood: &CharacterMood) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO character_mood (character_id, valence, energy, tension, history, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
         ON CONFLICT(character_id) DO UPDATE SET valence=?2, energy=?3, tension=?4, history=?5, updated_at=datetime('now')",
        params![mood.character_id, mood.valence, mood.energy, mood.tension, mood.history.to_string()],
    )?;
    Ok(())
}


// ─── Settings ───────────────────────────────────────────────────────────────

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    match conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |r| r.get(0)) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )?;
    Ok(())
}




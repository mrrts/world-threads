use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ─── User Profile ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub world_id: String,
    pub display_name: String,
    pub description: String,
    pub facts: Value,
    pub avatar_file: String,
    pub updated_at: String,
}

pub fn get_user_profile(conn: &Connection, world_id: &str) -> Result<UserProfile, rusqlite::Error> {
    conn.query_row(
        "SELECT world_id, display_name, description, facts, avatar_file, updated_at FROM user_profiles WHERE world_id = ?1",
        params![world_id],
        |row| Ok(UserProfile {
            world_id: row.get(0)?,
            display_name: row.get(1)?,
            description: row.get(2)?,
            facts: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            avatar_file: row.get(4)?,
            updated_at: row.get(5)?,
        }),
    )
}

pub fn upsert_user_profile(conn: &Connection, p: &UserProfile) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO user_profiles (world_id, display_name, description, facts, avatar_file, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
         ON CONFLICT(world_id) DO UPDATE SET display_name=?2, description=?3, facts=?4, avatar_file=?5, updated_at=datetime('now')",
        params![p.world_id, p.display_name, p.description, p.facts.to_string(), p.avatar_file],
    )?;
    Ok(())
}

pub fn set_user_avatar_file(conn: &Connection, world_id: &str, avatar_file: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE user_profiles SET avatar_file = ?2, updated_at = datetime('now') WHERE world_id = ?1",
        params![world_id, avatar_file],
    )?;
    Ok(())
}



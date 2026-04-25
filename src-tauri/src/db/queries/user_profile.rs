use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ─── User Profile ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub world_id: String,
    pub display_name: String,
    pub description: String,
    /// Known facts about the user — analogous to Character.backstory_facts
    /// but on the user-character. Field name differs (facts vs.
    /// backstory_facts) for historical reasons and for the registration
    /// register the field reads in: a regular character's biography facts
    /// vs. the user's lived facts the characters know about them.
    pub facts: Value,
    /// Boundaries the user has set for themselves in-world — analogous
    /// to Character.boundaries. Same field name as on Character.
    /// Added 2026-04-25 so the canonizer's "Remember this" flow can
    /// route boundary-shaped proposals here. Characters read these the
    /// same way they read each other's: "this is a thing they've named
    /// for themselves; respect it."
    #[serde(default)]
    pub boundaries: Value,
    pub avatar_file: String,
    pub updated_at: String,
}

pub fn get_user_profile(conn: &Connection, world_id: &str) -> Result<UserProfile, rusqlite::Error> {
    conn.query_row(
        "SELECT world_id, display_name, description, facts, boundaries, avatar_file, updated_at FROM user_profiles WHERE world_id = ?1",
        params![world_id],
        |row| Ok(UserProfile {
            world_id: row.get(0)?,
            display_name: row.get(1)?,
            description: row.get(2)?,
            facts: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            boundaries: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_else(|_| serde_json::json!([])),
            avatar_file: row.get(5)?,
            updated_at: row.get(6)?,
        }),
    )
}

pub fn upsert_user_profile(conn: &Connection, p: &UserProfile) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO user_profiles (world_id, display_name, description, facts, boundaries, avatar_file, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
         ON CONFLICT(world_id) DO UPDATE SET display_name=?2, description=?3, facts=?4, boundaries=?5, avatar_file=?6, updated_at=datetime('now')",
        params![p.world_id, p.display_name, p.description, p.facts.to_string(), p.boundaries.to_string(), p.avatar_file],
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



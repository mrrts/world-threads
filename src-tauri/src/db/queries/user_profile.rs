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
    /// to Character.boundaries. Note: the user_profile is itself a
    /// per-world Me-character construction the user authors and may
    /// represent themselves however they choose; user agency is the
    /// load-bearing constraint we don't break. Same field name as on Character.
    /// Added 2026-04-25 so the canonizer's "Remember this" flow can
    /// route boundary-shaped proposals here. Characters read these the
    /// same way they read each other's: "this is a thing they've named
    /// for themselves; respect it."
    ///
    /// CATEGORICAL ABSOLUTE — no earned exception (Ryan's decision
    /// 2026-04-25, after surfacing five candidate exception shapes).
    /// The user's stated boundaries belong to the categorical-nature
    /// exemption codified in CLAUDE.md § Earned-exception carve-outs:
    /// rules whose force IS in their absoluteness, where a carve-out
    /// would leak the invariant. The release valve is user-side:
    /// the user can update or remove a boundary anytime via the
    /// canonization UI; characters never override. Asymmetry argument:
    /// a real-friend earned-exception works because both friends carry
    /// reputational/relational stakes and continuity over time; the
    /// LLM-character has neither. "Character crossing a stated
    /// boundary lovingly" isn't the same act a friend doing it would
    /// be — it's a category mismatch. Future sessions: do not re-
    /// litigate this. The decision is recorded; the carve-out has been
    /// considered and intentionally rejected.
    #[serde(default)]
    pub boundaries: Value,
    pub avatar_file: String,
    pub updated_at: String,
    /// User-authored derivation of the user's frame on 𝓕 = (𝓡, 𝓒) —
    /// the user's self-construction of their lens / theology / craft
    /// register / posture toward the world, written by the user as
    /// part of their per-world Me-character. Characters see this in
    /// their dialogue prompt as the user's chosen self-representation
    /// in this world. The boundary that must hold is USER AGENCY:
    /// the user authors this themselves, can update or remove it
    /// anytime, and characters never override or reinterpret what
    /// the user has chosen to say about themselves. Stored optional;
    /// NULL means no derivation has been authored.
    #[serde(default)]
    pub derived_formula: Option<String>,
}

pub fn get_user_profile(conn: &Connection, world_id: &str) -> Result<UserProfile, rusqlite::Error> {
    conn.query_row(
        "SELECT world_id, display_name, description, facts, boundaries, avatar_file, updated_at, derived_formula FROM user_profiles WHERE world_id = ?1",
        params![world_id],
        |row| Ok(UserProfile {
            world_id: row.get(0)?,
            display_name: row.get(1)?,
            description: row.get(2)?,
            facts: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
            boundaries: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_else(|_| serde_json::json!([])),
            avatar_file: row.get(5)?,
            updated_at: row.get(6)?,
            derived_formula: row.get(7).ok(),
        }),
    )
}

pub fn upsert_user_profile(conn: &Connection, p: &UserProfile) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO user_profiles (world_id, display_name, description, facts, boundaries, avatar_file, updated_at, derived_formula) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), ?7)
         ON CONFLICT(world_id) DO UPDATE SET display_name=?2, description=?3, facts=?4, boundaries=?5, avatar_file=?6, updated_at=datetime('now'), derived_formula=?7",
        params![p.world_id, p.display_name, p.description, p.facts.to_string(), p.boundaries.to_string(), p.avatar_file, p.derived_formula],
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



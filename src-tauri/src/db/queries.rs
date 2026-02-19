use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ─── World ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct World {
    pub world_id: String,
    pub name: String,
    pub description: String,
    pub tone_tags: Value,
    pub invariants: Value,
    pub state: Value,
    pub created_at: String,
    pub updated_at: String,
}

pub fn create_world(conn: &Connection, world: &World) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO worlds (world_id, name, description, tone_tags, invariants, state, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![world.world_id, world.name, world.description,
            world.tone_tags.to_string(), world.invariants.to_string(),
            world.state.to_string(), world.created_at, world.updated_at],
    )?;
    Ok(())
}

pub fn get_world(conn: &Connection, world_id: &str) -> Result<World, rusqlite::Error> {
    conn.query_row(
        "SELECT world_id, name, description, tone_tags, invariants, state, created_at, updated_at FROM worlds WHERE world_id = ?1",
        params![world_id],
        row_to_world,
    )
}

pub fn list_worlds(conn: &Connection) -> Result<Vec<World>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT world_id, name, description, tone_tags, invariants, state, created_at, updated_at FROM worlds ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], row_to_world)?;
    rows.collect()
}

pub fn update_world(conn: &Connection, world: &World) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE worlds SET name=?2, description=?3, tone_tags=?4, invariants=?5, state=?6, updated_at=datetime('now') WHERE world_id=?1",
        params![world.world_id, world.name, world.description,
            world.tone_tags.to_string(), world.invariants.to_string(),
            world.state.to_string()],
    )?;
    Ok(())
}

pub fn delete_world(conn: &Connection, world_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM worlds WHERE world_id = ?1", params![world_id])?;
    Ok(())
}

fn row_to_world(row: &rusqlite::Row) -> Result<World, rusqlite::Error> {
    Ok(World {
        world_id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        tone_tags: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
        invariants: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
        state: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

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

// ─── Character ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    pub character_id: String,
    pub world_id: String,
    pub display_name: String,
    pub identity: String,
    pub voice_rules: Value,
    pub boundaries: Value,
    pub backstory_facts: Value,
    pub relationships: Value,
    pub state: Value,
    pub avatar_color: String,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub fn create_character(conn: &Connection, ch: &Character) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO characters (character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, is_archived, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![ch.character_id, ch.world_id, ch.display_name, ch.identity,
            ch.voice_rules.to_string(), ch.boundaries.to_string(),
            ch.backstory_facts.to_string(), ch.relationships.to_string(),
            ch.state.to_string(), ch.avatar_color, ch.is_archived, ch.created_at, ch.updated_at],
    )?;
    Ok(())
}

pub fn get_character(conn: &Connection, character_id: &str) -> Result<Character, rusqlite::Error> {
    conn.query_row(
        "SELECT character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, is_archived, created_at, updated_at FROM characters WHERE character_id = ?1",
        params![character_id],
        row_to_character,
    )
}

pub fn list_characters(conn: &Connection, world_id: &str) -> Result<Vec<Character>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, is_archived, created_at, updated_at FROM characters WHERE world_id = ?1 AND is_archived = 0 ORDER BY created_at"
    )?;
    let rows = stmt.query_map(params![world_id], row_to_character)?;
    rows.collect()
}

pub fn list_archived_characters(conn: &Connection, world_id: &str) -> Result<Vec<Character>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, is_archived, created_at, updated_at FROM characters WHERE world_id = ?1 AND is_archived = 1 ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map(params![world_id], row_to_character)?;
    rows.collect()
}

pub fn archive_character(conn: &Connection, character_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET is_archived = 1, updated_at = datetime('now') WHERE character_id = ?1",
        params![character_id],
    )?;
    Ok(())
}

pub fn unarchive_character(conn: &Connection, character_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET is_archived = 0, updated_at = datetime('now') WHERE character_id = ?1",
        params![character_id],
    )?;
    Ok(())
}

pub fn update_character(conn: &Connection, ch: &Character) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET display_name=?2, identity=?3, voice_rules=?4, boundaries=?5, backstory_facts=?6, relationships=?7, state=?8, avatar_color=?9, updated_at=datetime('now') WHERE character_id=?1",
        params![ch.character_id, ch.display_name, ch.identity,
            ch.voice_rules.to_string(), ch.boundaries.to_string(),
            ch.backstory_facts.to_string(), ch.relationships.to_string(),
            ch.state.to_string(), ch.avatar_color],
    )?;
    Ok(())
}

pub fn delete_character(conn: &Connection, character_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM characters WHERE character_id = ?1", params![character_id])?;
    Ok(())
}

fn row_to_character(row: &rusqlite::Row) -> Result<Character, rusqlite::Error> {
    Ok(Character {
        character_id: row.get(0)?,
        world_id: row.get(1)?,
        display_name: row.get(2)?,
        identity: row.get(3)?,
        voice_rules: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
        boundaries: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
        backstory_facts: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
        relationships: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
        state: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
        avatar_color: row.get(9)?,
        is_archived: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

// ─── Thread ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Thread {
    pub thread_id: String,
    pub character_id: String,
    pub world_id: String,
    pub created_at: String,
}

pub fn create_thread(conn: &Connection, t: &Thread) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO threads (thread_id, character_id, world_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![t.thread_id, t.character_id, t.world_id, t.created_at],
    )?;
    Ok(())
}

pub fn get_thread_for_character(conn: &Connection, character_id: &str) -> Result<Thread, rusqlite::Error> {
    conn.query_row(
        "SELECT thread_id, character_id, world_id, created_at FROM threads WHERE character_id = ?1",
        params![character_id],
        |row| Ok(Thread { thread_id: row.get(0)?, character_id: row.get(1)?, world_id: row.get(2)?, created_at: row.get(3)? }),
    )
}

// ─── Message ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub message_id: String,
    pub thread_id: String,
    pub role: String,
    pub content: String,
    pub tokens_estimate: i64,
    pub created_at: String,
}

pub fn create_message(conn: &Connection, m: &Message) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO messages (message_id, thread_id, role, content, tokens_estimate, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![m.message_id, m.thread_id, m.role, m.content, m.tokens_estimate, m.created_at],
    )?;
    conn.execute(
        "INSERT INTO messages_fts (message_id, thread_id, content) VALUES (?1, ?2, ?3)",
        params![m.message_id, m.thread_id, m.content],
    ).ok();
    Ok(())
}

pub fn list_messages(conn: &Connection, thread_id: &str, limit: i64) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT message_id, thread_id, role, content, tokens_estimate, created_at
         FROM messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT ?2"
    )?;
    let rows = stmt.query_map(params![thread_id, limit], row_to_message)?;
    let mut msgs: Vec<Message> = rows.collect::<Result<Vec<_>, _>>()?;
    msgs.reverse();
    Ok(msgs)
}

pub fn get_all_messages(conn: &Connection, thread_id: &str) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT message_id, thread_id, role, content, tokens_estimate, created_at
         FROM messages WHERE thread_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![thread_id], row_to_message)?;
    rows.collect()
}

/// Returns the most recent `limit` messages, skipping the newest `offset`.
/// Result is in chronological order (oldest first).
pub fn list_messages_paginated(conn: &Connection, thread_id: &str, limit: i64, offset: i64) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT message_id, thread_id, role, content, tokens_estimate, created_at
         FROM messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
    )?;
    let rows = stmt.query_map(params![thread_id, limit, offset], row_to_message)?;
    let mut msgs: Vec<Message> = rows.collect::<Result<Vec<_>, _>>()?;
    msgs.reverse();
    Ok(msgs)
}

pub fn count_messages(conn: &Connection, thread_id: &str) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT count(*) FROM messages WHERE thread_id = ?1",
        params![thread_id],
        |r| r.get(0),
    )
}

pub fn count_messages_since_maintenance(conn: &Connection, thread_id: &str) -> i64 {
    conn.query_row(
        "SELECT count_since_maintenance FROM message_count_tracker WHERE thread_id = ?1",
        params![thread_id],
        |r| r.get(0),
    ).unwrap_or(0)
}

pub fn increment_message_counter(conn: &Connection, thread_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO message_count_tracker (thread_id, count_since_maintenance) VALUES (?1, 1)
         ON CONFLICT(thread_id) DO UPDATE SET count_since_maintenance = count_since_maintenance + 1",
        params![thread_id],
    )?;
    Ok(())
}

pub fn reset_message_counter(conn: &Connection, thread_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE message_count_tracker SET count_since_maintenance = 0 WHERE thread_id = ?1",
        params![thread_id],
    )?;
    Ok(())
}

fn row_to_message(row: &rusqlite::Row) -> Result<Message, rusqlite::Error> {
    Ok(Message {
        message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
        content: row.get(3)?, tokens_estimate: row.get(4)?, created_at: row.get(5)?,
    })
}

// ─── World Events ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorldEvent {
    pub event_id: String,
    pub world_id: String,
    pub day_index: i64,
    pub time_of_day: String,
    pub summary: String,
    pub involved_characters: Value,
    pub hooks: Value,
    pub trigger_type: String,
    pub created_at: String,
}

pub fn create_world_event(conn: &Connection, e: &WorldEvent) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO world_events (event_id, world_id, day_index, time_of_day, summary, involved_characters, hooks, trigger_type, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![e.event_id, e.world_id, e.day_index, e.time_of_day, e.summary,
            e.involved_characters.to_string(), e.hooks.to_string(), e.trigger_type, e.created_at],
    )?;
    conn.execute(
        "INSERT INTO world_events_fts (event_id, world_id, summary) VALUES (?1, ?2, ?3)",
        params![e.event_id, e.world_id, e.summary],
    ).ok();
    Ok(())
}

pub fn list_world_events(conn: &Connection, world_id: &str, limit: i64) -> Result<Vec<WorldEvent>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT event_id, world_id, day_index, time_of_day, summary, involved_characters, hooks, trigger_type, created_at
         FROM world_events WHERE world_id = ?1 ORDER BY created_at DESC LIMIT ?2"
    )?;
    let rows = stmt.query_map(params![world_id, limit], row_to_event)?;
    let mut evts: Vec<WorldEvent> = rows.collect::<Result<Vec<_>, _>>()?;
    evts.reverse();
    Ok(evts)
}

pub fn delete_last_world_event(conn: &Connection, world_id: &str) -> Result<Option<WorldEvent>, rusqlite::Error> {
    let evt = conn.query_row(
        "SELECT event_id, world_id, day_index, time_of_day, summary, involved_characters, hooks, trigger_type, created_at
         FROM world_events WHERE world_id = ?1 ORDER BY created_at DESC LIMIT 1",
        params![world_id],
        row_to_event,
    );
    match evt {
        Ok(e) => {
            conn.execute("DELETE FROM world_events WHERE event_id = ?1", params![e.event_id])?;
            conn.execute("DELETE FROM world_events_fts WHERE event_id = ?1", params![e.event_id]).ok();
            Ok(Some(e))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

fn row_to_event(row: &rusqlite::Row) -> Result<WorldEvent, rusqlite::Error> {
    Ok(WorldEvent {
        event_id: row.get(0)?, world_id: row.get(1)?, day_index: row.get(2)?,
        time_of_day: row.get(3)?, summary: row.get(4)?,
        involved_characters: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
        hooks: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
        trigger_type: row.get(7)?, created_at: row.get(8)?,
    })
}

// ─── Memory Artifacts ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryArtifact {
    pub artifact_id: String,
    pub artifact_type: String,
    pub subject_id: String,
    pub world_id: String,
    pub content: String,
    pub sources: Value,
    pub created_at: String,
    pub updated_at: String,
}

pub fn upsert_memory_artifact(conn: &Connection, a: &MemoryArtifact) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO memory_artifacts (artifact_id, artifact_type, subject_id, world_id, content, sources, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(artifact_id) DO UPDATE SET content=excluded.content, sources=excluded.sources, updated_at=datetime('now')",
        params![a.artifact_id, a.artifact_type, a.subject_id, a.world_id,
            a.content, a.sources.to_string(), a.created_at, a.updated_at],
    )?;
    Ok(())
}

pub fn get_memory_artifacts(conn: &Connection, subject_id: &str, artifact_type: &str) -> Result<Vec<MemoryArtifact>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT artifact_id, artifact_type, subject_id, world_id, content, sources, created_at, updated_at
         FROM memory_artifacts WHERE subject_id = ?1 AND artifact_type = ?2 ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map(params![subject_id, artifact_type], |row| {
        Ok(MemoryArtifact {
            artifact_id: row.get(0)?, artifact_type: row.get(1)?, subject_id: row.get(2)?,
            world_id: row.get(3)?, content: row.get(4)?,
            sources: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            created_at: row.get(6)?, updated_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_thread_summary(conn: &Connection, thread_id: &str) -> String {
    get_memory_artifacts(conn, thread_id, "thread_summary")
        .ok()
        .and_then(|v| v.into_iter().next())
        .map(|a| a.content)
        .unwrap_or_default()
}

// ─── Character Portraits ────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Portrait {
    pub portrait_id: String,
    pub character_id: String,
    pub prompt: String,
    pub file_name: String,
    pub is_active: bool,
    pub created_at: String,
}

pub fn create_portrait(conn: &Connection, p: &Portrait) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO character_portraits (portrait_id, character_id, prompt, file_name, is_active, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![p.portrait_id, p.character_id, p.prompt, p.file_name, p.is_active, p.created_at],
    )?;
    Ok(())
}

pub fn list_portraits(conn: &Connection, character_id: &str) -> Result<Vec<Portrait>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT portrait_id, character_id, prompt, file_name, is_active, created_at FROM character_portraits WHERE character_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![character_id], |row| {
        Ok(Portrait {
            portrait_id: row.get(0)?, character_id: row.get(1)?, prompt: row.get(2)?,
            file_name: row.get(3)?, is_active: row.get(4)?, created_at: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn get_active_portrait(conn: &Connection, character_id: &str) -> Option<Portrait> {
    conn.query_row(
        "SELECT portrait_id, character_id, prompt, file_name, is_active, created_at FROM character_portraits WHERE character_id = ?1 AND is_active = 1",
        params![character_id],
        |row| Ok(Portrait {
            portrait_id: row.get(0)?, character_id: row.get(1)?, prompt: row.get(2)?,
            file_name: row.get(3)?, is_active: row.get(4)?, created_at: row.get(5)?,
        }),
    ).ok()
}

pub fn set_active_portrait(conn: &Connection, character_id: &str, portrait_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("UPDATE character_portraits SET is_active = 0 WHERE character_id = ?1", params![character_id])?;
    conn.execute("UPDATE character_portraits SET is_active = 1 WHERE portrait_id = ?1", params![portrait_id])?;
    Ok(())
}

/// All portraits for all characters belonging to a given world.
pub fn list_portraits_for_world(conn: &Connection, world_id: &str) -> Result<Vec<(Portrait, String)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT p.portrait_id, p.character_id, p.prompt, p.file_name, p.is_active, p.created_at, c.display_name
         FROM character_portraits p
         JOIN characters c ON c.character_id = p.character_id
         WHERE c.world_id = ?1
         ORDER BY p.created_at DESC"
    )?;
    let rows = stmt.query_map(params![world_id], |row| {
        Ok((
            Portrait {
                portrait_id: row.get(0)?, character_id: row.get(1)?, prompt: row.get(2)?,
                file_name: row.get(3)?, is_active: row.get(4)?, created_at: row.get(5)?,
            },
            row.get::<_, String>(6)?,
        ))
    })?;
    rows.collect()
}

// ─── World Images ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorldImage {
    pub image_id: String,
    pub world_id: String,
    pub prompt: String,
    pub file_name: String,
    pub is_active: bool,
    pub source: String,
    pub created_at: String,
}

pub fn create_world_image(conn: &Connection, img: &WorldImage) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO world_images (image_id, world_id, prompt, file_name, is_active, source, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![img.image_id, img.world_id, img.prompt, img.file_name, img.is_active, img.source, img.created_at],
    )?;
    Ok(())
}

pub fn list_world_images(conn: &Connection, world_id: &str) -> Result<Vec<WorldImage>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT image_id, world_id, prompt, file_name, is_active, source, created_at FROM world_images WHERE world_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![world_id], |row| {
        Ok(WorldImage {
            image_id: row.get(0)?, world_id: row.get(1)?, prompt: row.get(2)?,
            file_name: row.get(3)?, is_active: row.get(4)?, source: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn get_active_world_image(conn: &Connection, world_id: &str) -> Option<WorldImage> {
    conn.query_row(
        "SELECT image_id, world_id, prompt, file_name, is_active, source, created_at FROM world_images WHERE world_id = ?1 AND is_active = 1",
        params![world_id],
        |row| Ok(WorldImage {
            image_id: row.get(0)?, world_id: row.get(1)?, prompt: row.get(2)?,
            file_name: row.get(3)?, is_active: row.get(4)?, source: row.get(5)?,
            created_at: row.get(6)?,
        }),
    ).ok()
}

pub fn set_active_world_image(conn: &Connection, world_id: &str, image_id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("UPDATE world_images SET is_active = 0 WHERE world_id = ?1", params![world_id])?;
    conn.execute("UPDATE world_images SET is_active = 1 WHERE image_id = ?1", params![image_id])?;
    Ok(())
}

// ─── Chat Backgrounds ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatBackground {
    pub character_id: String,
    pub bg_type: String,
    pub bg_color: String,
    pub bg_image_id: String,
    pub bg_blur: i64,
    pub updated_at: String,
}

pub fn get_chat_background(conn: &Connection, character_id: &str) -> Option<ChatBackground> {
    conn.query_row(
        "SELECT character_id, bg_type, bg_color, bg_image_id, bg_blur, updated_at FROM chat_backgrounds WHERE character_id = ?1",
        params![character_id],
        |row| Ok(ChatBackground {
            character_id: row.get(0)?, bg_type: row.get(1)?, bg_color: row.get(2)?,
            bg_image_id: row.get(3)?, bg_blur: row.get(4)?, updated_at: row.get(5)?,
        }),
    ).ok()
}

pub fn upsert_chat_background(conn: &Connection, bg: &ChatBackground) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO chat_backgrounds (character_id, bg_type, bg_color, bg_image_id, bg_blur, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
         ON CONFLICT(character_id) DO UPDATE SET bg_type=?2, bg_color=?3, bg_image_id=?4, bg_blur=?5, updated_at=datetime('now')",
        params![bg.character_id, bg.bg_type, bg.bg_color, bg.bg_image_id, bg.bg_blur],
    )?;
    Ok(())
}

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

// ─── FTS Search ─────────────────────────────────────────────────────────────

pub fn search_messages_fts(conn: &Connection, thread_id: &str, query: &str, limit: i64) -> Result<Vec<Message>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT f.message_id, f.thread_id, m.role, f.content, m.tokens_estimate, m.created_at
         FROM messages_fts f
         JOIN messages m ON m.message_id = f.message_id
         WHERE f.thread_id = ?1 AND messages_fts MATCH ?2
         ORDER BY rank LIMIT ?3"
    )?;
    let rows = stmt.query_map(params![thread_id, query, limit], row_to_message)?;
    rows.collect()
}

pub fn search_events_fts(conn: &Connection, world_id: &str, query: &str, limit: i64) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT f.summary
         FROM world_events_fts f
         WHERE f.world_id = ?1 AND world_events_fts MATCH ?2
         ORDER BY rank LIMIT ?3"
    )?;
    let rows = stmt.query_map(params![world_id, query, limit], |row| row.get::<_, String>(0))?;
    rows.collect()
}

// ─── Vector Search ──────────────────────────────────────────────────────────

pub fn insert_vector_chunk(conn: &Connection, chunk_id: &str, source_type: &str, source_id: &str, world_id: &str, content: &str, embedding: &[f32]) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO chunk_metadata (chunk_id, source_type, source_id, world_id, content) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![chunk_id, source_type, source_id, world_id, content],
    )?;
    let rowid: i64 = conn.query_row(
        "SELECT rowid FROM chunk_metadata WHERE chunk_id = ?1", params![chunk_id], |r| r.get(0)
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO vec_chunks (rowid, embedding) VALUES (?1, ?2)",
        params![rowid, embedding.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>()],
    )?;
    Ok(())
}

pub fn search_vectors(conn: &Connection, world_id: &str, embedding: &[f32], limit: i64) -> Result<Vec<(String, f64)>, rusqlite::Error> {
    let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
    let mut stmt = conn.prepare(
        "SELECT cm.content, v.distance
         FROM vec_chunks v
         JOIN chunk_metadata cm ON cm.rowid = v.rowid
         WHERE v.embedding MATCH ?1 AND k = ?2 AND cm.world_id = ?3
         ORDER BY v.distance"
    )?;
    let rows = stmt.query_map(params![blob, limit, world_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
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

// ─── Tick Cache ─────────────────────────────────────────────────────────────

pub fn get_tick_cache(conn: &Connection, cache_key: &str) -> Result<Option<String>, rusqlite::Error> {
    match conn.query_row("SELECT result FROM tick_cache WHERE cache_key = ?1", params![cache_key], |r| r.get(0)) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn set_tick_cache(conn: &Connection, cache_key: &str, result: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO tick_cache (cache_key, result) VALUES (?1, ?2) ON CONFLICT(cache_key) DO UPDATE SET result=excluded.result, created_at=datetime('now')",
        params![cache_key, result],
    )?;
    Ok(())
}

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



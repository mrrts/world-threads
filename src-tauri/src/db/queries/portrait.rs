use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

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

pub fn delete_portrait(conn: &Connection, portrait_id: &str) -> Result<String, rusqlite::Error> {
    let file_name: String = conn.query_row(
        "SELECT file_name FROM character_portraits WHERE portrait_id = ?1",
        params![portrait_id],
        |r| r.get(0),
    )?;
    conn.execute("DELETE FROM character_portraits WHERE portrait_id = ?1", params![portrait_id])?;
    Ok(file_name)
}



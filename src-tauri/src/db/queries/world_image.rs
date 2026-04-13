use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

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
    pub aspect_ratio: f64,
}

pub fn create_world_image(conn: &Connection, img: &WorldImage) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO world_images (image_id, world_id, prompt, file_name, is_active, source, created_at, aspect_ratio) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![img.image_id, img.world_id, img.prompt, img.file_name, img.is_active, img.source, img.created_at, img.aspect_ratio],
    )?;
    Ok(())
}

pub fn list_world_images(conn: &Connection, world_id: &str) -> Result<Vec<WorldImage>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT image_id, world_id, prompt, file_name, is_active, source, created_at, aspect_ratio FROM world_images WHERE world_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![world_id], |row| {
        Ok(WorldImage {
            image_id: row.get(0)?, world_id: row.get(1)?, prompt: row.get(2)?,
            file_name: row.get(3)?, is_active: row.get(4)?, source: row.get(5)?,
            created_at: row.get(6)?, aspect_ratio: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn get_active_world_image(conn: &Connection, world_id: &str) -> Option<WorldImage> {
    conn.query_row(
        "SELECT image_id, world_id, prompt, file_name, is_active, source, created_at, aspect_ratio FROM world_images WHERE world_id = ?1 AND is_active = 1",
        params![world_id],
        |row| Ok(WorldImage {
            image_id: row.get(0)?, world_id: row.get(1)?, prompt: row.get(2)?,
            file_name: row.get(3)?, is_active: row.get(4)?, source: row.get(5)?,
            created_at: row.get(6)?, aspect_ratio: row.get(7)?,
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



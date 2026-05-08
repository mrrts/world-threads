use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NovelEntry {
    pub novel_id: String,
    pub thread_id: String,
    pub world_day: i64,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn upsert_novel_entry(conn: &Connection, entry: &NovelEntry) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO novel_entries (novel_id, thread_id, world_day, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(thread_id, world_day) DO UPDATE SET content = ?4, updated_at = ?6",
        params![entry.novel_id, entry.thread_id, entry.world_day, entry.content, entry.created_at, entry.updated_at],
    )?;
    Ok(())
}

pub fn get_novel_entry(conn: &Connection, thread_id: &str, world_day: i64) -> Option<NovelEntry> {
    conn.query_row(
        "SELECT novel_id, thread_id, world_day, content, created_at, updated_at FROM novel_entries WHERE thread_id = ?1 AND world_day = ?2",
        params![thread_id, world_day],
        |row| Ok(NovelEntry {
            novel_id: row.get(0)?,
            thread_id: row.get(1)?,
            world_day: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        }),
    ).ok()
}

pub fn list_novel_entries(
    conn: &Connection,
    thread_id: &str,
) -> Result<Vec<NovelEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT novel_id, thread_id, world_day, content, created_at, updated_at FROM novel_entries WHERE thread_id = ?1 ORDER BY world_day ASC"
    )?;
    let rows = stmt.query_map(params![thread_id], |row| {
        Ok(NovelEntry {
            novel_id: row.get(0)?,
            thread_id: row.get(1)?,
            world_day: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn delete_novel_entry(
    conn: &Connection,
    thread_id: &str,
    world_day: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM novel_entries WHERE thread_id = ?1 AND world_day = ?2",
        params![thread_id, world_day],
    )?;
    Ok(())
}

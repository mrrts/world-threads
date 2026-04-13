use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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


// ─── Vector Search ──────────────────────────────────────────────────────────

pub fn insert_vector_chunk(conn: &Connection, chunk_id: &str, source_type: &str, source_id: &str, world_id: &str, character_id: &str, content: &str, embedding: &[f32]) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO chunk_metadata (chunk_id, source_type, source_id, world_id, character_id, content) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![chunk_id, source_type, source_id, world_id, character_id, content],
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

pub fn search_vectors(conn: &Connection, world_id: &str, character_id: &str, embedding: &[f32], limit: i64) -> Result<Vec<(String, f64)>, rusqlite::Error> {
    let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
    let mut stmt = conn.prepare(
        "SELECT cm.content, v.distance
         FROM vec_chunks v
         JOIN chunk_metadata cm ON cm.rowid = v.rowid
         WHERE v.embedding MATCH ?1 AND k = ?2
           AND cm.world_id = ?3
           AND cm.character_id = ?4
         ORDER BY v.distance"
    )?;
    let rows = stmt.query_map(params![blob, limit, world_id, character_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
    })?;
    rows.collect()
}



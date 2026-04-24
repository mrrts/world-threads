use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Per-character "what does this character load-test?" — the architecture-
/// level spine of their authority, periodically synthesized from their
/// recent corpus. See the architecture-vs-vocabulary experiment
/// (`reports/2026-04-24-0948-architecture-hypothesis-bites.md`).
///
/// `anchor_label` is short (e.g. "DEVOTION", "LANGUAGE"). `anchor_body`
/// is the full second-person prompt-block — self-contained and ready
/// to push into the dialogue system prompt without further formatting.
/// `derivation_summary` is a one-paragraph rationale for inspectability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestAnchor {
    pub anchor_id: String,
    pub character_id: String,
    pub world_id: String,
    pub anchor_label: String,
    pub anchor_body: String,
    pub derivation_summary: String,
    pub world_day_at_generation: Option<i64>,
    pub source_message_count: i64,
    pub refresh_trigger: String,
    pub model_used: String,
    pub created_at: String,
}

pub fn insert_load_test_anchor(
    conn: &Connection,
    a: &LoadTestAnchor,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO character_load_test_anchors (
            anchor_id, character_id, world_id, anchor_label, anchor_body,
            derivation_summary, world_day_at_generation, source_message_count,
            refresh_trigger, model_used, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            a.anchor_id, a.character_id, a.world_id, a.anchor_label, a.anchor_body,
            a.derivation_summary, a.world_day_at_generation, a.source_message_count,
            a.refresh_trigger, a.model_used, a.created_at,
        ],
    )?;
    Ok(())
}

/// Most recent anchor row for a character, or None if never generated.
/// Hot-path read on every dialogue assembly; kept narrow.
pub fn latest_load_test_anchor(
    conn: &Connection,
    character_id: &str,
) -> Result<Option<LoadTestAnchor>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT anchor_id, character_id, world_id, anchor_label, anchor_body,
                derivation_summary, world_day_at_generation, source_message_count,
                refresh_trigger, model_used, created_at
         FROM character_load_test_anchors
         WHERE character_id = ?1
         ORDER BY created_at DESC
         LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![character_id], |r| {
        Ok(LoadTestAnchor {
            anchor_id: r.get(0)?,
            character_id: r.get(1)?,
            world_id: r.get(2)?,
            anchor_label: r.get(3)?,
            anchor_body: r.get(4)?,
            derivation_summary: r.get(5)?,
            world_day_at_generation: r.get(6)?,
            source_message_count: r.get(7)?,
            refresh_trigger: r.get(8)?,
            model_used: r.get(9)?,
            created_at: r.get(10)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Full history for a character, newest first. Used by inspect surfaces
/// (worldcli show-anchor) and any future analysis of anchor evolution.
pub fn list_load_test_anchors(
    conn: &Connection,
    character_id: &str,
    limit: i64,
) -> Result<Vec<LoadTestAnchor>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT anchor_id, character_id, world_id, anchor_label, anchor_body,
                derivation_summary, world_day_at_generation, source_message_count,
                refresh_trigger, model_used, created_at
         FROM character_load_test_anchors
         WHERE character_id = ?1
         ORDER BY created_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![character_id, limit], |r| {
        Ok(LoadTestAnchor {
            anchor_id: r.get(0)?,
            character_id: r.get(1)?,
            world_id: r.get(2)?,
            anchor_label: r.get(3)?,
            anchor_body: r.get(4)?,
            derivation_summary: r.get(5)?,
            world_day_at_generation: r.get(6)?,
            source_message_count: r.get(7)?,
            refresh_trigger: r.get(8)?,
            model_used: r.get(9)?,
            created_at: r.get(10)?,
        })
    })?;
    rows.collect()
}

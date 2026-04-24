use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Per-character register-axis row. Originally added as "load_test
/// anchor" only; expanded to multi-axis on 2026-04-24 (see
/// `axis_kind` column). One row per (character_id, axis_kind, refresh
/// generation). New axes (joy_reception, grief, ...) live in the same
/// table differentiated by `axis_kind`.
///
/// `anchor_label` is short (e.g. "DEVOTION", "LIVEABLE LOAD-BEARING").
/// `anchor_body` is the full second-person prompt-block — self-
/// contained and ready to push into the dialogue system prompt without
/// further formatting. `derivation_summary` is a one-paragraph
/// rationale for inspectability.
///
/// (Type name kept as `LoadTestAnchor` for backwards compatibility with
/// downstream consumers; conceptually it's now a `RegisterAxis`. A
/// rename can come later if the axis_kind expansion lands more
/// firmly.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestAnchor {
    pub anchor_id: String,
    pub character_id: String,
    pub world_id: String,
    /// Which register-axis this row holds. Defaults to "load_test" for
    /// rows created before the multi-axis pivot.
    pub axis_kind: String,
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
            anchor_id, character_id, world_id, axis_kind, anchor_label, anchor_body,
            derivation_summary, world_day_at_generation, source_message_count,
            refresh_trigger, model_used, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            a.anchor_id, a.character_id, a.world_id, a.axis_kind,
            a.anchor_label, a.anchor_body,
            a.derivation_summary, a.world_day_at_generation, a.source_message_count,
            a.refresh_trigger, a.model_used, a.created_at,
        ],
    )?;
    Ok(())
}

/// Most recent row for a character on a specific axis_kind. Hot-path
/// read for prompt assembly when only one axis is needed.
pub fn latest_anchor_for_axis(
    conn: &Connection,
    character_id: &str,
    axis_kind: &str,
) -> Result<Option<LoadTestAnchor>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT anchor_id, character_id, world_id, axis_kind, anchor_label, anchor_body,
                derivation_summary, world_day_at_generation, source_message_count,
                refresh_trigger, model_used, created_at
         FROM character_load_test_anchors
         WHERE character_id = ?1 AND axis_kind = ?2
         ORDER BY created_at DESC
         LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![character_id, axis_kind], |r| row_to_anchor(r))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Backwards-compatible thin wrapper for the original load-test axis.
/// Existing callers continue to work unchanged.
pub fn latest_load_test_anchor(
    conn: &Connection,
    character_id: &str,
) -> Result<Option<LoadTestAnchor>, rusqlite::Error> {
    latest_anchor_for_axis(conn, character_id, "load_test")
}

/// Latest row per axis_kind for a character. The hot-path read for
/// prompt assembly when ALL of the character's known axes should be
/// injected (the multi-axis future state). Returns one row per axis
/// (the most recent for each kind).
pub fn latest_axes_for_character(
    conn: &Connection,
    character_id: &str,
) -> Result<Vec<LoadTestAnchor>, rusqlite::Error> {
    // Per-axis latest: subquery picks the max created_at per axis_kind,
    // then joins back to get the full row.
    let mut stmt = conn.prepare(
        "SELECT a.anchor_id, a.character_id, a.world_id, a.axis_kind, a.anchor_label,
                a.anchor_body, a.derivation_summary, a.world_day_at_generation,
                a.source_message_count, a.refresh_trigger, a.model_used, a.created_at
         FROM character_load_test_anchors a
         INNER JOIN (
             SELECT axis_kind, MAX(created_at) AS max_created
             FROM character_load_test_anchors
             WHERE character_id = ?1
             GROUP BY axis_kind
         ) latest
           ON latest.axis_kind = a.axis_kind
          AND latest.max_created = a.created_at
         WHERE a.character_id = ?1
         ORDER BY a.axis_kind ASC",
    )?;
    let rows = stmt.query_map(params![character_id], |r| row_to_anchor(r))?;
    rows.collect()
}

/// Full history for a character (all axes), newest first. Used by
/// inspect surfaces (worldcli show-anchor) and analysis of evolution.
pub fn list_load_test_anchors(
    conn: &Connection,
    character_id: &str,
    limit: i64,
) -> Result<Vec<LoadTestAnchor>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT anchor_id, character_id, world_id, axis_kind, anchor_label, anchor_body,
                derivation_summary, world_day_at_generation, source_message_count,
                refresh_trigger, model_used, created_at
         FROM character_load_test_anchors
         WHERE character_id = ?1
         ORDER BY created_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![character_id, limit], |r| row_to_anchor(r))?;
    rows.collect()
}

fn row_to_anchor(r: &rusqlite::Row) -> Result<LoadTestAnchor, rusqlite::Error> {
    Ok(LoadTestAnchor {
        anchor_id: r.get(0)?,
        character_id: r.get(1)?,
        world_id: r.get(2)?,
        axis_kind: r.get(3)?,
        anchor_label: r.get(4)?,
        anchor_body: r.get(5)?,
        derivation_summary: r.get(6)?,
        world_day_at_generation: r.get(7)?,
        source_message_count: r.get(8)?,
        refresh_trigger: r.get(9)?,
        model_used: r.get(10)?,
        created_at: r.get(11)?,
    })
}

/// Combine all of a character's latest axes into a single prompt-block
/// string ready to inject. Each axis body is separated by a blank line.
/// Returns None if the character has no axes synthesized yet. This is
/// what call-sites should use to feed `load_test_anchor: Option<&str>`
/// into prompts::build_dialogue_system_prompt.
pub fn combined_axes_block(
    conn: &Connection,
    character_id: &str,
) -> Option<String> {
    let axes = latest_axes_for_character(conn, character_id).ok()?;
    if axes.is_empty() { return None; }
    let combined = axes.iter()
        .map(|a| a.anchor_body.trim().to_string())
        .filter(|b| !b.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if combined.is_empty() { None } else { Some(combined) }
}

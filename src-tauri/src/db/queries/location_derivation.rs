use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

/// Per-(world, location-name) cached derivation. See db::schema.rs for
/// the architectural rationale (feeds the elevation injection in
/// orchestrator::run_dialogue_with_base under CHARACTER_FORMULA_AT_TOP=1
/// + the SCENE LOCATION block in dialogue messages).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationDerivation {
    pub world_id: String,
    pub name: String,
    pub derived_formula: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Lookup by (world_id, name) — case-insensitive on name. Returns None
/// when no derivation has been generated for this pair yet.
pub fn get_location_derivation(
    conn: &Connection,
    world_id: &str,
    name: &str,
) -> Result<Option<LocationDerivation>, rusqlite::Error> {
    conn.query_row(
        "SELECT world_id, name, derived_formula, created_at, updated_at \
         FROM location_derivations \
         WHERE world_id = ?1 AND name = ?2 COLLATE NOCASE",
        params![world_id, name],
        |row| {
            Ok(LocationDerivation {
                world_id: row.get(0)?,
                name: row.get(1)?,
                derived_formula: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
    .optional()
}

/// Insert a new derivation row, or update the formula if the (world, name)
/// pair already exists. Idempotent.
pub fn upsert_location_derivation(
    conn: &Connection,
    world_id: &str,
    name: &str,
    derived_formula: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO location_derivations (world_id, name, derived_formula, created_at, updated_at) \
         VALUES (?1, ?2, ?3, datetime('now'), datetime('now')) \
         ON CONFLICT(world_id, name) DO UPDATE SET \
             derived_formula = excluded.derived_formula, \
             updated_at = datetime('now')",
        params![world_id, name, derived_formula],
    )?;
    Ok(())
}

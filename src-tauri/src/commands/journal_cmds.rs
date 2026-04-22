use crate::ai::orchestrator;
use crate::db::queries::*;
use crate::db::Database;
use tauri::State;

const PRIOR_ENTRIES_FOR_CONTEXT: usize = 2;

fn current_world_day(world: &World) -> i64 {
    world.state.get("time")
        .and_then(|t| t.get("day_index"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

/// Generate (or regenerate) YESTERDAY's journal entry for the given
/// character — i.e. the day that just closed (current world-day minus
/// one). Journals retrospect the day that has ended; the day currently
/// unfolding isn't journalled, because it isn't over yet. Writes to
/// `character_journals` with ON CONFLICT REPLACE, so re-clicking
/// overwrites yesterday's entry rather than stacking. If the current
/// world-day is 0 (no yesterday exists), returns an error.
#[tauri::command]
pub async fn generate_character_journal_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
) -> Result<JournalEntry, String> {
    if api_key.trim().is_empty() {
        return Err("no API key".to_string());
    }

    let (character, target_day, model_config, history, prior_items, prior_entries) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let current = current_world_day(&world);
        // Journal retrospects the day that just ended — the world clock
        // must have crossed into at least day 1 before we can journal
        // anything.
        let target_day = current - 1;
        if target_day < 0 {
            return Err("no previous world-day to journal — the world is still on day 0".to_string());
        }
        let model_config = orchestrator::load_model_config(&conn);
        let user_name = get_user_profile(&conn, &character.world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the human".to_string());
        // Bound the history feed strictly to the target (just-closed)
        // world-day — we want yesterday's actual messages, not "the last
        // N by wall-clock recency" and not today's in-progress beats.
        let history = gather_character_messages_for_world_day(
            &conn, &character.character_id, &user_name, target_day,
        );
        let prior_items: Vec<orchestrator::InventoryItem> = character.inventory.as_array()
            .map(|a| a.iter()
                .filter_map(|v| serde_json::from_value::<orchestrator::InventoryItem>(v.clone()).ok())
                .collect())
            .unwrap_or_default();
        // Prior entries for voice continuity — strictly BEFORE the day
        // we're journalling, so a regenerate doesn't use its own
        // previous version of yesterday's entry as context.
        let prior_entries = list_journal_entries_before(
            &conn, &character_id, target_day, PRIOR_ENTRIES_FOR_CONTEXT,
        ).unwrap_or_default();
        (character, target_day, model_config, history, prior_items, prior_entries)
    };

    let base = model_config.chat_api_base();
    let content = orchestrator::generate_character_journal(
        &base, &api_key, &model_config.memory_model,
        &character.display_name,
        &character.identity,
        &character.signature_emoji,
        &prior_items,
        &prior_entries,
        &history,
        target_day,
    ).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let entry = JournalEntry {
        journal_id: uuid::Uuid::new_v4().to_string(),
        character_id: character.character_id.clone(),
        world_day: target_day,
        content,
        created_at: now,
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        upsert_journal_entry(&conn, &entry).map_err(|e| e.to_string())?;
    }
    log::info!("[Journal] wrote entry for {} on Day {target_day} (current world-day: {})", character.display_name, target_day + 1);
    Ok(entry)
}

/// Auto-variant invoked from the chat focus-refresh hook AND from the
/// post-send path when a character message lands in a new world-day.
/// Target day is `current_world_day - 1` (yesterday). Short-circuits if
/// yesterday's entry already exists for this character (no LLM call, no
/// DB write), or if the world is still on day 0 (no yesterday yet).
/// Returns the existing entry when fresh, the newly-generated entry
/// when it wrote one. Shapes the return as a result envelope so the
/// frontend can tell "refreshed vs cached."
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MaybeJournalResult {
    pub entry: Option<JournalEntry>,
    /// true iff this call actually ran the LLM and wrote a row.
    pub refreshed: bool,
}

#[tauri::command]
pub async fn maybe_generate_character_journal_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
) -> Result<MaybeJournalResult, String> {
    // Short-circuit check up front — no API-key probe needed when fresh
    // or when there's nothing to journal yet (day 0).
    let existing = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let target_day = current_world_day(&world) - 1;
        if target_day < 0 {
            // World is still on day 0 — nothing to retrospect yet.
            return Ok(MaybeJournalResult { entry: None, refreshed: false });
        }
        get_journal_entry_for_day(&conn, &character_id, target_day)
    };
    if let Some(entry) = existing {
        return Ok(MaybeJournalResult { entry: Some(entry), refreshed: false });
    }
    if api_key.trim().is_empty() {
        return Ok(MaybeJournalResult { entry: None, refreshed: false });
    }
    // No entry yet — delegate to the forced path for the actual work.
    match generate_character_journal_cmd(db, api_key, character_id).await {
        Ok(entry) => Ok(MaybeJournalResult { entry: Some(entry), refreshed: true }),
        Err(e) => {
            log::warn!("[Journal] auto-generation failed (non-fatal): {e}");
            Ok(MaybeJournalResult { entry: None, refreshed: false })
        }
    }
}

/// List the most-recent N journal entries for a character.
#[tauri::command]
pub fn list_character_journals_cmd(
    db: State<'_, Database>,
    character_id: String,
    limit: Option<usize>,
) -> Result<Vec<JournalEntry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_journal_entries(&conn, &character_id, limit.unwrap_or(30))
        .map_err(|e| e.to_string())
}

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

/// Generate (or regenerate) YESTERDAY's journal entry for the player
/// in the given world. Writes to `user_journals` with ON CONFLICT
/// REPLACE — re-clicking overwrites yesterday's entry rather than
/// stacking. Errors if the world is still on day 0.
#[tauri::command]
pub async fn generate_user_journal_cmd(
    db: State<'_, Database>,
    api_key: String,
    world_id: String,
) -> Result<UserJournalEntry, String> {
    if api_key.trim().is_empty() {
        return Err("no API key".to_string());
    }

    let (profile, target_day, model_config, history, prior_entries) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let current = current_world_day(&world);
        let target_day = current - 1;
        if target_day < 0 {
            return Err("no previous world-day to journal — the world is still on day 0".to_string());
        }
        let profile = get_user_profile(&conn, &world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let history = gather_world_messages_for_world_day(
            &conn, &world_id, &profile.display_name, target_day,
        );
        let prior_entries = list_user_journal_entries_before(
            &conn, &world_id, target_day, PRIOR_ENTRIES_FOR_CONTEXT,
        ).unwrap_or_default();
        (profile, target_day, model_config, history, prior_entries)
    };

    let facts: Vec<String> = profile.facts.as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let base = model_config.chat_api_base();
    let content = orchestrator::generate_user_journal(
        &base, &api_key, &model_config.memory_model,
        &profile.display_name,
        &profile.description,
        &facts,
        &prior_entries,
        &history,
        target_day,
    ).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let entry = UserJournalEntry {
        journal_id: uuid::Uuid::new_v4().to_string(),
        world_id: world_id.clone(),
        world_day: target_day,
        content,
        created_at: now,
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        upsert_user_journal_entry(&conn, &entry).map_err(|e| e.to_string())?;
    }
    log::info!("[UserJournal] wrote entry for {} on Day {target_day} (current world-day: {})", profile.display_name, target_day + 1);
    Ok(entry)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MaybeUserJournalResult {
    pub entry: Option<UserJournalEntry>,
    pub refreshed: bool,
}

/// Auto-variant invoked from the chat focus-refresh hook. Short-circuits
/// if yesterday's entry already exists (no LLM call, no DB write), or if
/// the world is still on day 0.
#[tauri::command]
pub async fn maybe_generate_user_journal_cmd(
    db: State<'_, Database>,
    api_key: String,
    world_id: String,
) -> Result<MaybeUserJournalResult, String> {
    let existing = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let target_day = current_world_day(&world) - 1;
        if target_day < 0 {
            return Ok(MaybeUserJournalResult { entry: None, refreshed: false });
        }
        get_user_journal_entry_for_day(&conn, &world_id, target_day)
    };
    if let Some(entry) = existing {
        return Ok(MaybeUserJournalResult { entry: Some(entry), refreshed: false });
    }
    if api_key.trim().is_empty() {
        return Ok(MaybeUserJournalResult { entry: None, refreshed: false });
    }
    match generate_user_journal_cmd(db, api_key, world_id).await {
        Ok(entry) => Ok(MaybeUserJournalResult { entry: Some(entry), refreshed: true }),
        Err(e) => {
            log::warn!("[UserJournal] auto-generation failed (non-fatal): {e}");
            Ok(MaybeUserJournalResult { entry: None, refreshed: false })
        }
    }
}

#[tauri::command]
pub fn list_user_journals_cmd(
    db: State<'_, Database>,
    world_id: String,
    limit: Option<usize>,
) -> Result<Vec<UserJournalEntry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_user_journal_entries(&conn, &world_id, limit.unwrap_or(30))
        .map_err(|e| e.to_string())
}

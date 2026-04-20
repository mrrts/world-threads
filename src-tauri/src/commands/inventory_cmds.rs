use crate::ai::orchestrator::{self, InventoryItem};
use crate::db::queries::*;
use crate::db::Database;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

/// How many world-days can pass before an inventory is considered stale
/// and the focus-trigger refreshes it. 1 = refresh on any new day.
/// Adjustable knob — later could be swapped to a finer-grained stamp
/// (day + time_of_day) without changing the call sites.
pub const INVENTORY_STALE_DAYS: i64 = 1;

/// How many recent chronologically-merged messages to feed the seed /
/// refresh LLM as context. Caps token cost and attention dilution.
pub const INVENTORY_HISTORY_LIMIT: usize = 40;

/// Result shape for inventory refresh: the current items plus a flag
/// indicating whether a refresh actually ran this call (vs. no-op).
/// Frontend uses `refreshed` to decide whether to display a subtle
/// "inventory updated" cue.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventoryRefreshResult {
    pub character_id: String,
    pub inventory: Vec<InventoryItem>,
    /// true if this call actually ran seed or refresh; false if the
    /// inventory was still fresh and we returned cached.
    pub refreshed: bool,
    /// "seed" | "refresh" | "noop"
    pub mode: String,
}

fn current_world_day(world: &World) -> i64 {
    world
        .state
        .get("time")
        .and_then(|t| t.get("day_index"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

fn parse_inventory(raw: &Value) -> Vec<InventoryItem> {
    raw.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| serde_json::from_value::<InventoryItem>(v.clone()).ok())
                .filter(|it| !it.name.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Run the check-and-possibly-refresh flow for one character. Returns
/// the current inventory either way. Pure function of state + LLM; no
/// side effects outside DB writes and the LLM call.
///
/// Trigger logic:
///   - last_inventory_day IS NULL → SEED.
///   - current_world_day - last_inventory_day >= INVENTORY_STALE_DAYS → REFRESH.
///   - otherwise → NOOP (return the stored inventory as-is).
pub async fn refresh_one_character_inventory(
    db: &Database,
    api_key: &str,
    character_id: &str,
) -> Result<InventoryRefreshResult, String> {
    let (character, world, model_config, user_name) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let user_name = get_user_profile(&conn, &character.world_id)
            .ok()
            .map(|p| p.display_name)
            .unwrap_or_else(|| "the human".to_string());
        (character, world, model_config, user_name)
    };

    let today = current_world_day(&world);
    let prior_items = parse_inventory(&character.inventory);

    let mode = match character.last_inventory_day {
        None => "seed",
        Some(last) if today - last >= INVENTORY_STALE_DAYS => "refresh",
        _ => "noop",
    };

    if mode == "noop" {
        return Ok(InventoryRefreshResult {
            character_id: character.character_id.clone(),
            inventory: prior_items,
            refreshed: false,
            mode: "noop".to_string(),
        });
    }

    // Skip LLM entirely if there's no API key / local-only mode with no
    // memory model. Leave prior inventory untouched but advance the
    // stamp so we don't spam failed attempts.
    if api_key.trim().is_empty() {
        return Ok(InventoryRefreshResult {
            character_id: character.character_id.clone(),
            inventory: prior_items,
            refreshed: false,
            mode: "noop".to_string(),
        });
    }

    let history = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        gather_character_recent_messages(
            &conn,
            &character.character_id,
            &user_name,
            INVENTORY_HISTORY_LIMIT,
        )
    };

    let base = model_config.chat_api_base();
    let model = &model_config.memory_model;

    let new_items = match mode {
        "seed" => orchestrator::seed_character_inventory(
            &base, api_key, model,
            &character.display_name, &character.identity,
            &history,
        ).await,
        _ => orchestrator::refresh_character_inventory(
            &base, api_key, model,
            &character.display_name, &character.identity,
            &prior_items, &history,
        ).await,
    };

    let new_items = match new_items {
        Ok(items) => items,
        Err(e) => {
            log::warn!("[Inventory] {} {} failed: {e}", mode, character.display_name);
            // Don't advance the stamp on failure — we'll try again on next focus.
            return Ok(InventoryRefreshResult {
                character_id: character.character_id.clone(),
                inventory: prior_items,
                refreshed: false,
                mode: "noop".to_string(),
            });
        }
    };

    // Persist the new inventory and advance the stamp to today.
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let json = serde_json::to_value(&new_items).unwrap_or(Value::Array(vec![]));
        let _ = set_character_inventory(&conn, &character.character_id, &json, Some(today));
    }

    log::info!(
        "[Inventory] {} {} — {} items",
        mode, character.display_name, new_items.len(),
    );

    Ok(InventoryRefreshResult {
        character_id: character.character_id.clone(),
        inventory: new_items,
        refreshed: true,
        mode: mode.to_string(),
    })
}

/// Tauri entry point for a single character's focus-trigger refresh.
/// Called by the frontend when the user interacts with a solo chat.
#[tauri::command]
pub async fn refresh_character_inventory_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
) -> Result<InventoryRefreshResult, String> {
    refresh_one_character_inventory(&db, &api_key, &character_id).await
}

/// Tauri entry point for a group chat's focus-trigger refresh. Kicks
/// off one refresh task per group member in parallel so none of them
/// blocks the others. Returns the (possibly updated) inventory for each.
#[tauri::command]
pub async fn refresh_group_inventories_cmd(
    db: State<'_, Database>,
    api_key: String,
    group_chat_id: String,
) -> Result<Vec<InventoryRefreshResult>, String> {
    let char_ids: Vec<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        gc.character_ids
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    };

    // Fan out concurrently — each inventory call is a memory-tier
    // LLM roundtrip, so they parallelize on the wire. SQLite access
    // serializes on the inner Mutex, which is fine since DB time is
    // tiny compared to the LLM wait. No tokio::spawn (avoids 'static
    // lifetime): join_all polls all futures on the current task.
    let db_ref: &Database = db.inner();
    let futs = char_ids.into_iter().map(|cid| {
        let key = api_key.clone();
        async move { refresh_one_character_inventory(db_ref, &key, &cid).await }
    });
    let results = futures_util::future::join_all(futs).await;
    let out: Vec<InventoryRefreshResult> = results.into_iter().filter_map(|r| r.ok()).collect();
    Ok(out)
}

/// User-edit entry point from the character settings page. Blindly
/// stores whatever the user typed (trimmed + capped to max items).
/// Stamps `last_inventory_day` to the current world-day so the
/// refresh check doesn't blow the user's edit away on next focus.
#[tauri::command]
pub fn set_character_inventory_cmd(
    db: State<'_, Database>,
    character_id: String,
    inventory: Vec<InventoryItem>,
) -> Result<Vec<InventoryItem>, String> {
    // Normalize kind, drop empties, preserve order (physicals first,
    // then interiors) so the strip / prompt render stays predictable.
    // Total capped at INVENTORY_MAX_ITEMS — no per-kind cap; the mix
    // is a soft rule enforced in the LLM prompt, not here.
    let mut phys: Vec<InventoryItem> = Vec::new();
    let mut inter: Vec<InventoryItem> = Vec::new();
    for it in inventory.into_iter() {
        let kind = if it.kind.trim().eq_ignore_ascii_case(orchestrator::INVENTORY_KIND_INTERIOR) {
            orchestrator::INVENTORY_KIND_INTERIOR.to_string()
        } else {
            orchestrator::INVENTORY_KIND_PHYSICAL.to_string()
        };
        let normalized = InventoryItem {
            name: it.name.trim().to_string(),
            description: it.description.trim().to_string(),
            kind,
        };
        if normalized.name.is_empty() { continue; }
        if normalized.kind == orchestrator::INVENTORY_KIND_INTERIOR {
            inter.push(normalized);
        } else {
            phys.push(normalized);
        }
    }
    let mut cleaned: Vec<InventoryItem> = phys;
    cleaned.extend(inter);
    if cleaned.len() > orchestrator::INVENTORY_MAX_ITEMS {
        cleaned.truncate(orchestrator::INVENTORY_MAX_ITEMS);
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
    let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
    let today = current_world_day(&world);
    let json = serde_json::to_value(&cleaned).unwrap_or(Value::Array(vec![]));
    set_character_inventory(&conn, &character_id, &json, Some(today))
        .map_err(|e| e.to_string())?;
    Ok(cleaned)
}

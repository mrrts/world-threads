use crate::ai::orchestrator;
use crate::ai::prompts;
use crate::db::queries::*;
use crate::db::Database;
use tauri::State;

const MEANWHILE_HISTORY_WINDOW: usize = 30;

fn current_world_day_and_time(world: &World) -> (i64, String) {
    let day = world
        .state
        .get("time")
        .and_then(|t| t.get("day_index"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let time = world
        .state
        .get("time")
        .and_then(|t| t.get("time_of_day"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    (day, time)
}

/// Generate one meanwhile event per non-archived character in the
/// world — the "small grain of an ongoing life" feed. Uses the
/// memory_model tier. Fan-out in serial to keep it simple and
/// predictable (there aren't usually many characters in one world).
/// Skips characters on LLM errors — non-fatal per-character.
#[tauri::command]
pub async fn generate_meanwhile_events_cmd(
    db: State<'_, Database>,
    api_key: String,
    world_id: String,
) -> Result<Vec<MeanwhileEventWithName>, String> {
    if api_key.trim().is_empty() {
        return Err("no API key".to_string());
    }

    let (characters, model_config, world_day, time_of_day, weather_label, user_name) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let characters = list_characters(&conn, &world_id).map_err(|e| e.to_string())?;
        let (wd, wt) = current_world_day_and_time(&world);
        let weather_label = world
            .state
            .get("weather")
            .and_then(|v| v.as_str())
            .and_then(|k| prompts::weather_meta(k).map(|(_emoji, label)| label.to_string()));
        let user_name = get_user_profile(&conn, &world_id)
            .ok()
            .map(|p| p.display_name)
            .unwrap_or_else(|| "the human".to_string());
        (characters, model_config, wd, wt, weather_label, user_name)
    };

    let base = model_config.chat_api_base();
    let mut inserted: Vec<MeanwhileEventWithName> = Vec::new();

    for character in &characters {
        // Per-character context — inventory + recent history.
        let (prior_items, history) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let items: Vec<orchestrator::InventoryItem> = character
                .inventory
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| {
                            serde_json::from_value::<orchestrator::InventoryItem>(v.clone()).ok()
                        })
                        .collect()
                })
                .unwrap_or_default();
            let hist = gather_character_recent_messages(
                &conn,
                &character.character_id,
                &user_name,
                MEANWHILE_HISTORY_WINDOW,
            );
            (items, hist)
        };
        let summary = match orchestrator::generate_meanwhile_event(
            &base,
            &api_key,
            &model_config.memory_model,
            &character.display_name,
            &character.identity,
            &prior_items,
            &history,
            world_day,
            if time_of_day.is_empty() {
                "day"
            } else {
                &time_of_day
            },
            weather_label.as_deref(),
        )
        .await
        {
            Ok(s) => s,
            Err(e) => {
                log::warn!("[Meanwhile] failed for {}: {e}", character.display_name);
                continue;
            }
        };
        let event = MeanwhileEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.clone(),
            character_id: character.character_id.clone(),
            world_day,
            time_of_day: time_of_day.clone(),
            summary,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let user_id = crate::auth::context::current_user_id(&conn).map_err(|e| e.to_string())?;
            if let Err(e) = create_meanwhile_event(&conn, &event, user_id) {
                log::warn!(
                    "[Meanwhile] insert failed for {}: {e}",
                    character.display_name
                );
                continue;
            }
        }
        inserted.push(MeanwhileEventWithName {
            event_id: event.event_id,
            character_id: event.character_id,
            character_name: character.display_name.clone(),
            avatar_color: character.avatar_color.clone(),
            world_day: event.world_day,
            time_of_day: event.time_of_day,
            summary: event.summary,
            created_at: event.created_at,
        });
    }
    log::info!(
        "[Meanwhile] wrote {} events for world {world_id}",
        inserted.len()
    );
    Ok(inserted)
}

/// Auto-variant invoked from the chat focus-refresh hook: short-circuits
/// if the world already has meanwhile events for the current world_day.
/// Returns empty vec on no-op; returns the generated batch when it
/// actually ran. Non-fatal on LLM errors.
#[tauri::command]
pub async fn maybe_generate_meanwhile_events_cmd(
    db: State<'_, Database>,
    api_key: String,
    world_id: String,
) -> Result<Vec<MeanwhileEventWithName>, String> {
    // Short-circuit check #1: is there already an event for this world +
    // current world_day? If yes, no-op.
    let (world_day, already_exists, latest_meanwhile_at) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let (wd, _) = current_world_day_and_time(&world);
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM meanwhile_events WHERE world_id = ?1 AND world_day = ?2",
                rusqlite::params![world_id, wd],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let latest_at: Option<String> = conn
            .query_row(
                "SELECT MAX(created_at) FROM meanwhile_events WHERE world_id = ?1",
                rusqlite::params![world_id],
                |r| r.get(0),
            )
            .ok()
            .flatten();
        (wd, count > 0, latest_at)
    };
    if already_exists {
        log::info!("[Meanwhile] world {world_id} already has events for Day {world_day} — skipping auto-gen");
        return Ok(Vec::new());
    }

    // Short-circuit check #2: if the most recent meanwhile event in this
    // world is newer than the most recent dialogue message across all of
    // the world's chats (solo + group), don't generate a fresh one. This
    // prevents back-to-back meanwhile cards when the user re-opens a chat
    // across a day transition without any actual conversation having
    // happened since the last meanwhile.
    if let Some(latest_mw) = latest_meanwhile_at.as_ref() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let latest_msg_at: Option<String> = conn
            .query_row(
                "SELECT MAX(latest) FROM (
                    SELECT MAX(m.created_at) AS latest
                      FROM messages m
                      JOIN threads t ON t.thread_id = m.thread_id
                     WHERE t.world_id = ?1
                    UNION ALL
                    SELECT MAX(gm.created_at) AS latest
                      FROM group_messages gm
                      JOIN threads t ON t.thread_id = gm.thread_id
                     WHERE t.world_id = ?1
                 )",
                rusqlite::params![world_id],
                |r| r.get(0),
            )
            .ok()
            .flatten();
        let no_activity_since_last_meanwhile = match latest_msg_at {
            None => true,
            Some(msg_at) => msg_at.as_str() <= latest_mw.as_str(),
        };
        if no_activity_since_last_meanwhile {
            log::info!("[Meanwhile] world {world_id} has no dialogue activity since last meanwhile ({latest_mw}) — skipping auto-gen");
            return Ok(Vec::new());
        }
    }
    if api_key.trim().is_empty() {
        return Ok(Vec::new());
    }
    match generate_meanwhile_events_cmd(db, api_key, world_id.clone()).await {
        Ok(events) => Ok(events),
        Err(e) => {
            log::warn!("[Meanwhile] auto-generation failed for {world_id} (non-fatal): {e}");
            Ok(Vec::new())
        }
    }
}

/// Fetch the most-recent N meanwhile events for a world.
#[tauri::command]
pub fn list_meanwhile_events_cmd(
    db: State<'_, Database>,
    world_id: String,
    limit: Option<usize>,
) -> Result<Vec<MeanwhileEventWithName>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_meanwhile_events(&conn, &world_id, limit.unwrap_or(30)).map_err(|e| e.to_string())
}

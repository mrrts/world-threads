use crate::ai::orchestrator;
use crate::db::queries::*;
use crate::db::Database;
use tauri::State;

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

/// Compose the two-pass daily reading for the given world (forced —
/// regenerates even if today's reading already exists). Used by the
/// manual "Generate" button.
#[tauri::command]
pub async fn generate_daily_reading_cmd(
    db: State<'_, Database>,
    api_key: String,
    world_id: String,
) -> Result<DailyReading, String> {
    if api_key.trim().is_empty() {
        return Err("no API key".to_string());
    }

    let (world, world_day, user_name, characters, model_config, day_messages, yesterday) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let (wd, _) = current_world_day_and_time(&world);
        let user_name = get_user_profile(&conn, &world_id)
            .ok()
            .map(|p| p.display_name)
            .unwrap_or_else(|| "the human".to_string());
        let characters = list_characters(&conn, &world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let messages = gather_world_messages_for_world_day(&conn, &world_id, &user_name, wd);
        let yesterday = if wd > 0 {
            get_daily_reading_for_day(&conn, &world_id, wd - 1)
        } else {
            None
        };
        (
            world,
            wd,
            user_name,
            characters,
            model_config,
            messages,
            yesterday,
        )
    };
    let _ = user_name;

    // Character summaries — display_name + identity + a condensed
    // inventory line (first two item names of each kind).
    let characters_summary = characters
        .iter()
        .map(|c| {
            let inv_names: Vec<String> = c
                .inventory
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
                        .take(4)
                        .collect()
                })
                .unwrap_or_default();
            let inv_line = if inv_names.is_empty() {
                String::new()
            } else {
                format!(" (carrying: {})", inv_names.join(", "))
            };
            let id = if c.identity.is_empty() {
                String::new()
            } else {
                format!(" — {}", c.identity.chars().take(140).collect::<String>())
            };
            format!("  - {}{}{}", c.display_name, id, inv_line)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let day_messages_rendered = day_messages
        .iter()
        .map(|(speaker, content, _)| {
            let clipped: String = content.chars().take(280).collect();
            format!("{speaker}: {clipped}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let base = model_config.chat_api_base();
    let (domains, complication, draft_usage, crit_usage) =
        orchestrator::generate_daily_reading_with_critique(
            &base,
            &api_key,
            &model_config.memory_model,
            &world,
            world_day,
            &characters_summary,
            &day_messages_rendered,
            yesterday.as_ref(),
        )
        .await?;

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Some(u) = &draft_usage {
            let _ = record_token_usage(
                &conn,
                "daily_reading",
                &model_config.memory_model,
                u.prompt_tokens,
                u.completion_tokens,
            );
        }
        if let Some(u) = &crit_usage {
            let _ = record_token_usage(
                &conn,
                "daily_reading",
                &model_config.memory_model,
                u.prompt_tokens,
                u.completion_tokens,
            );
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let reading = DailyReading {
        reading_id: uuid::Uuid::new_v4().to_string(),
        world_id: world_id.clone(),
        world_day,
        domains,
        complication,
        created_at: now,
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        upsert_daily_reading(&conn, &reading).map_err(|e| e.to_string())?;
    }
    log::info!("[DailyReading] wrote reading for world {world_id} Day {world_day}");
    Ok(reading)
}

/// Auto-variant for focus-refresh. Short-circuits if today's reading
/// already exists. Returns the existing reading when fresh (no LLM
/// spend); generates + returns otherwise.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MaybeDailyReadingResult {
    pub reading: Option<DailyReading>,
    pub refreshed: bool,
}

#[tauri::command]
pub async fn maybe_generate_daily_reading_cmd(
    db: State<'_, Database>,
    api_key: String,
    world_id: String,
) -> Result<MaybeDailyReadingResult, String> {
    let existing = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let (wd, _) = current_world_day_and_time(&world);
        get_daily_reading_for_day(&conn, &world_id, wd)
    };
    if let Some(r) = existing {
        return Ok(MaybeDailyReadingResult {
            reading: Some(r),
            refreshed: false,
        });
    }
    if api_key.trim().is_empty() {
        return Ok(MaybeDailyReadingResult {
            reading: None,
            refreshed: false,
        });
    }
    match generate_daily_reading_cmd(db, api_key, world_id).await {
        Ok(r) => Ok(MaybeDailyReadingResult {
            reading: Some(r),
            refreshed: true,
        }),
        Err(e) => {
            log::warn!("[DailyReading] auto-generation failed (non-fatal): {e}");
            Ok(MaybeDailyReadingResult {
                reading: None,
                refreshed: false,
            })
        }
    }
}

#[tauri::command]
pub fn list_daily_readings_cmd(
    db: State<'_, Database>,
    world_id: String,
    limit: Option<usize>,
) -> Result<Vec<DailyReading>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_daily_readings(&conn, &world_id, limit.unwrap_or(30)).map_err(|e| e.to_string())
}

/// Fetch the most-recent reading (for today if present, else yesterday's).
/// Used as sidebar HUD initial load + dialogue-prompt context source.
#[tauri::command]
pub fn get_latest_daily_reading_cmd(
    db: State<'_, Database>,
    world_id: String,
) -> Result<Option<DailyReading>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(list_daily_readings(&conn, &world_id, 1)
        .unwrap_or_default()
        .into_iter()
        .next())
}

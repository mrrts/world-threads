use crate::ai::openai;
use crate::ai::orchestrator::{self, ModelConfig};
use crate::commands::chat_cmds;
use crate::db::queries::*;
use crate::db::Database;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSettingsChange {
    /// Stable key (e.g. "response_length"). Used by the prompt renderer.
    pub key: String,
    /// Human-readable label shown in chat history (e.g. "Response Length").
    pub label: String,
    /// Previous value, formatted for display (e.g. "Auto", "Short").
    pub from: String,
    /// New value, formatted for display.
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSettingsChangeBody {
    pub changes: Vec<ChatSettingsChange>,
}

/// Insert a `settings_update` message row into the chat (or group_messages)
/// table reflecting one or more chat-settings changes the user just made.
/// Surfaces the change in chat history both for the user (a small inline
/// "You changed Response Length from Auto to Short" card) and for the LLM
/// (the dialogue prompt's history block formats this row as an inline
/// note, so the model knows previous replies were under a different setting
/// and shouldn't pattern-match length to scrollback).
#[tauri::command]
pub fn record_chat_settings_change_cmd(
    db: State<Database>,
    thread_id: String,
    changes: Vec<ChatSettingsChange>,
    is_group: bool,
) -> Result<Message, String> {
    if changes.is_empty() {
        return Err("no changes to record".to_string());
    }
    let body = ChatSettingsChangeBody { changes };
    let content = serde_json::to_string(&body).map_err(|e| e.to_string())?;

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Pull world_day / world_time from the active world for this thread,
    // so the row sits naturally in the current beat.
    let world_id: Option<String> = if is_group {
        conn.query_row(
            "SELECT world_id FROM group_chats WHERE thread_id = ?1",
            rusqlite::params![thread_id], |r| r.get(0),
        ).ok()
    } else {
        conn.query_row(
            "SELECT world_id FROM threads WHERE thread_id = ?1",
            rusqlite::params![thread_id], |r| r.get(0),
        ).ok()
    };
    let (world_day, world_time) = match world_id.and_then(|wid| get_world(&conn, &wid).ok()) {
        Some(w) => chat_cmds::world_time_fields(&w),
        None => (None, None),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread_id.clone(),
        role: "settings_update".to_string(),
        content,
        tokens_estimate: 0,
        sender_character_id: None,
        created_at: now,
        world_day,
        world_time,
        address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
    };

    if is_group {
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    } else {
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    Ok(msg)
}

#[tauri::command]
pub fn get_model_config_cmd(db: State<Database>) -> Result<ModelConfig, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator::load_model_config(&conn))
}

#[tauri::command]
pub fn set_model_config_cmd(db: State<Database>, config: ModelConfig) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    orchestrator::save_model_config(&conn, &config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_setting_cmd(db: State<Database>, key: String) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_setting(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting_cmd(db: State<Database>, key: String, value: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    set_setting(&conn, &key, &value).map_err(|e| e.to_string())?;
    if key == "children_mode" {
        let enabled = value == "true" || value == "1" || value.eq_ignore_ascii_case("on");
        unsafe {
            std::env::set_var(
                "WORLDTHREADS_CHILDREN_MODE",
                if enabled { "1" } else { "0" },
            );
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_budget_mode_cmd(db: State<Database>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let val = get_setting(&conn, "budget_mode").map_err(|e| e.to_string())?;
    Ok(val.map(|v| v == "true").unwrap_or(false))
}

#[tauri::command]
pub fn set_budget_mode_cmd(db: State<Database>, enabled: bool) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    set_setting(&conn, "budget_mode", if enabled { "true" } else { "false" }).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_local_models_cmd(url: String) -> Result<Vec<openai::ModelInfo>, String> {
    let base_url = format!("{}/v1", url.trim_end_matches('/'));
    openai::list_models(&base_url, "").await
}

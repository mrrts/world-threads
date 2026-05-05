use crate::ai::openai;
use crate::ai::orchestrator::{self, ModelConfig};
use crate::commands::chat_cmds;
use crate::db::queries::*;
use crate::db::Database;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;

const CHILDREN_MODE_PASSWORD_HASH_KEY: &str = "children_mode_password_hash";

fn hash_children_mode_password(password: &str) -> Result<String, String> {
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt).map_err(|e| format!("salt generation failed: {e}"))?;
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(password.as_bytes());
    let digest = hasher.finalize();
    Ok(format!("{}:{}", hex::encode(salt), hex::encode(digest)))
}

fn verify_children_mode_password(password: &str, stored: &str) -> bool {
    let Some((salt_hex, digest_hex)) = stored.split_once(':') else {
        return false;
    };
    let Ok(salt) = hex::decode(salt_hex) else {
        return false;
    };
    let mut hasher = Sha256::new();
    hasher.update(&salt);
    hasher.update(password.as_bytes());
    let computed = hex::encode(hasher.finalize());
    // constant-time-ish compare on hex strings
    if computed.len() != digest_hex.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (a, b) in computed.bytes().zip(digest_hex.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

fn set_children_mode_env(enabled: bool) {
    unsafe {
        std::env::set_var(
            "WORLDTHREADS_CHILDREN_MODE",
            if enabled { "1" } else { "0" },
        );
    }
}

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
    if key == "children_mode" {
        return Err(
            "children_mode must be toggled via the password-protected commands".to_string(),
        );
    }
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}

/// Returns true when a password hash is already on file for Children Mode.
/// The frontend uses this to decide whether the enable flow needs to set a
/// new password or whether one is already in place.
#[tauri::command]
pub fn is_children_mode_password_set_cmd(db: State<Database>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let stored = get_setting(&conn, CHILDREN_MODE_PASSWORD_HASH_KEY).map_err(|e| e.to_string())?;
    Ok(stored.map(|s| !s.is_empty()).unwrap_or(false))
}

/// Enables Children Mode under a password the user must remember.
/// If a hash is already on file, the supplied password must verify against
/// it; otherwise this call sets the hash. There is no recovery path —
/// forgetting the password means Children Mode cannot be turned off without
/// editing the underlying database directly.
#[tauri::command]
pub fn enable_children_mode_with_password_cmd(
    db: State<Database>,
    password: String,
) -> Result<(), String> {
    if password.len() < 6 {
        return Err("password must be at least 6 characters".to_string());
    }
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let existing =
        get_setting(&conn, CHILDREN_MODE_PASSWORD_HASH_KEY).map_err(|e| e.to_string())?;
    match existing {
        Some(stored) if !stored.is_empty() => {
            if !verify_children_mode_password(&password, &stored) {
                return Err("incorrect password".to_string());
            }
        }
        _ => {
            let hash = hash_children_mode_password(&password)?;
            set_setting(&conn, CHILDREN_MODE_PASSWORD_HASH_KEY, &hash)
                .map_err(|e| e.to_string())?;
        }
    }
    set_setting(&conn, "children_mode", "true").map_err(|e| e.to_string())?;
    set_children_mode_env(true);
    Ok(())
}

/// Disables Children Mode after verifying the password set when it was
/// enabled. The hash is preserved so that re-enabling can require the same
/// password (the protection is symmetric: turning it off should require the
/// same proof of authority that turning it on did).
#[tauri::command]
pub fn disable_children_mode_with_password_cmd(
    db: State<Database>,
    password: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let stored = get_setting(&conn, CHILDREN_MODE_PASSWORD_HASH_KEY)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no Children Mode password is set".to_string())?;
    if !verify_children_mode_password(&password, &stored) {
        return Err("incorrect password".to_string());
    }
    set_setting(&conn, "children_mode", "false").map_err(|e| e.to_string())?;
    set_children_mode_env(false);
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

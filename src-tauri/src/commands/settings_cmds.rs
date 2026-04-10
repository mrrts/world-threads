use crate::ai::openai;
use crate::ai::orchestrator::{self, ModelConfig};
use crate::db::queries::*;
use crate::db::Database;
use tauri::State;

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
    set_setting(&conn, &key, &value).map_err(|e| e.to_string())
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

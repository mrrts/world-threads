use crate::db::queries::*;
use crate::db::Database;
use tauri::State;

#[tauri::command]
pub fn get_character_mood_cmd(
    db: State<Database>,
    character_id: String,
) -> Result<Option<CharacterMood>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(get_character_mood(&conn, &character_id))
}

#[tauri::command]
pub fn get_mood_settings_cmd(db: State<Database>) -> Result<MoodSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let enabled = get_setting(&conn, "mood_drift_enabled")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(true);
    let drift_rate = get_setting(&conn, "mood_drift_rate")
        .ok()
        .flatten()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.15);
    Ok(MoodSettings {
        enabled,
        drift_rate,
    })
}

#[tauri::command]
pub fn set_mood_settings_cmd(db: State<Database>, settings: MoodSettings) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    set_setting(
        &conn,
        "mood_drift_enabled",
        if settings.enabled { "true" } else { "false" },
    )
    .map_err(|e| e.to_string())?;
    set_setting(&conn, "mood_drift_rate", &settings.drift_rate.to_string())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct MoodSettings {
    pub enabled: bool,
    pub drift_rate: f64,
}

use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub fn list_characters_cmd(db: State<Database>, world_id: String) -> Result<Vec<Character>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_characters(&conn, &world_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_archived_characters_cmd(db: State<Database>, world_id: String) -> Result<Vec<Character>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_archived_characters(&conn, &world_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_character_cmd(db: State<Database>, character_id: String) -> Result<Character, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_character(&conn, &character_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_character_cmd(db: State<Database>, character: Character) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    update_character(&conn, &character).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_character_cmd(db: State<Database>, world_id: String, display_name: String) -> Result<Character, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let char_id = uuid::Uuid::new_v4().to_string();
    let thread_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let ch = Character {
        character_id: char_id.clone(),
        world_id: world_id.clone(),
        display_name,
        identity: String::new(),
        voice_rules: json!([]),
        boundaries: json!([]),
        backstory_facts: json!([]),
        relationships: json!({}),
        state: json!({
            "mood": 0.0,
            "trust_user": 0.5,
            "goals": [],
            "open_loops": [],
            "last_seen": { "day_index": 1, "time_of_day": "MORNING" }
        }),
        avatar_color: "#a8c482".to_string(),
        sex: "male".to_string(),
        is_archived: false,
        created_at: now.clone(),
        updated_at: now.clone(),
        visual_description: String::new(),
        visual_description_portrait_id: None,
        inventory: serde_json::Value::Array(vec![]),
        last_inventory_day: None,
        signature_emoji: String::new(),
    };
    create_character(&conn, &ch).map_err(|e| e.to_string())?;
    create_thread(&conn, &Thread {
        thread_id,
        character_id: char_id.clone(),
        world_id,
        created_at: now,
    }).map_err(|e| e.to_string())?;

    get_character(&conn, &char_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_character_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    audio_dir: State<crate::commands::audio_cmds::AudioDir>,
    character_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Collect portrait file names before deletion
    let portrait_files: Vec<String> = list_portraits(&conn, &character_id)
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.file_name)
        .collect();

    let (illustration_files, message_ids) = delete_character(&conn, &character_id).map_err(|e| e.to_string())?;

    // Remove portrait + illustration files from disk
    for file_name in portrait_files.iter().chain(illustration_files.iter()) {
        let path = portraits_dir.0.join(file_name);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }

    // Remove audio files
    for msg_id in &message_ids {
        crate::commands::audio_cmds::delete_audio_for_message(&audio_dir.0, msg_id);
    }

    Ok(())
}

#[tauri::command]
pub fn clear_chat_history_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    audio_dir: State<crate::commands::audio_cmds::AudioDir>,
    character_id: String,
    keep_media: bool,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let (illustration_files, message_ids) = clear_chat_history(&conn, &character_id, keep_media).map_err(|e| e.to_string())?;
    for f in &illustration_files {
        let path = portraits_dir.0.join(f);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
    for msg_id in &message_ids {
        crate::commands::audio_cmds::delete_audio_for_message(&audio_dir.0, msg_id);
    }
    Ok(())
}

#[tauri::command]
pub fn archive_character_cmd(db: State<Database>, character_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    archive_character(&conn, &character_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unarchive_character_cmd(db: State<Database>, character_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    unarchive_character(&conn, &character_id).map_err(|e| e.to_string())
}

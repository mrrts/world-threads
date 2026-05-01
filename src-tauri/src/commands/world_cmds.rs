use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use serde_json::{json, Value};
use tauri::State;

#[tauri::command]
pub fn create_world_cmd(db: State<Database>, name: String) -> Result<World, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let world_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let default_state = json!({
        "time": { "day_index": 1, "time_of_day": "MORNING" },
        "global_arcs": [],
        "facts": []
    });

    let world = World {
        world_id: world_id.clone(),
        name,
        description: String::new(),
        tone_tags: json!([]),
        invariants: json!([]),
        state: default_state,
        created_at: now.clone(),
        updated_at: now,
        derived_formula: None,
    };
    create_world(&conn, &world).map_err(|e| e.to_string())?;

    let colors = ["#c4a882", "#82a8c4"];
    let default_names = ["Mara", "Ion"];
    for i in 0..2 {
        let char_id = uuid::Uuid::new_v4().to_string();
        let thread_id = uuid::Uuid::new_v4().to_string();
        let ch_now = Utc::now().to_rfc3339();
        let ch = Character {
            character_id: char_id.clone(),
            world_id: world_id.clone(),
            display_name: default_names[i].to_string(),
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
            avatar_color: colors[i].to_string(),
            sex: "male".to_string(),
            is_archived: false,
            created_at: ch_now.clone(),
            updated_at: ch_now.clone(),
            visual_description: String::new(),
            visual_description_portrait_id: None,
            inventory: serde_json::Value::Array(vec![]),
            last_inventory_day: None,
            signature_emoji: String::new(),
            action_beat_density: "normal".to_string(),
            derived_formula: None,
            has_read_empiricon: false,
        };
        create_character(&conn, &ch).map_err(|e| e.to_string())?;
        create_thread(&conn, &Thread {
            thread_id,
            character_id: char_id,
            world_id: world_id.clone(),
            created_at: ch_now,
        }).map_err(|e| e.to_string())?;
    }

    get_world(&conn, &world_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_world_cmd(db: State<Database>, world_id: String) -> Result<World, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_world(&conn, &world_id).map_err(|e| e.to_string())
}

/// Read the documentary `derived_formula` for a world. Same shape
/// and rationale as get_character_derivation_cmd. Read-only for now.
#[tauri::command]
pub fn get_world_derivation_cmd(db: State<Database>, world_id: String) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let derived: Option<String> = conn.query_row(
        "SELECT derived_formula FROM worlds WHERE world_id = ?1",
        rusqlite::params![world_id], |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    Ok(derived)
}

#[tauri::command]
pub fn list_worlds_cmd(db: State<Database>) -> Result<Vec<World>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_worlds(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_world_cmd(db: State<Database>, world: World) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    update_world(&conn, &world).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_world_cmd(db: State<Database>, world_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    delete_world(&conn, &world_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_world_state_cmd(db: State<Database>, world_id: String, state: Value) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
    world.state = state;
    update_world(&conn, &world).map_err(|e| e.to_string())
}

use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use tauri::State;

/// Read the current world-day from the world state JSON. Missing /
/// malformed → None so lifecycle records don't lie about timing.
fn current_world_day(world: &World) -> Option<i64> {
    world
        .state
        .get("time")
        .and_then(|t| t.get("day_index"))
        .and_then(|v| v.as_i64())
}

/// Accept a new quest. This IS the commitment ceremony at the backend
/// level — the call commits the row; the frontend wraps the call in
/// a confirmation dialog so the act feels like a small vow, not a
/// notification dismissal.
/// Accept a new quest. `origin_kind` is one of "user_authored" |
/// "message" | "meanwhile" | "backstage"; `origin_ref` is the
/// originating message_id / event_id / etc. or None for user_authored.
#[tauri::command]
pub fn create_quest_cmd(
    db: State<'_, Database>,
    world_id: String,
    title: String,
    description: String,
    origin_kind: Option<String>,
    origin_ref: Option<String>,
) -> Result<Quest, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
    let day = current_world_day(&world);
    let kind = origin_kind
        .filter(|k| {
            matches!(
                k.as_str(),
                "user_authored" | "message" | "meanwhile" | "backstage"
            )
        })
        .unwrap_or_else(|| "user_authored".to_string());
    let quest = Quest {
        quest_id: uuid::Uuid::new_v4().to_string(),
        world_id,
        title: title.trim().to_string(),
        description: description.trim().to_string(),
        notes: String::new(),
        accepted_at: Utc::now().to_rfc3339(),
        accepted_world_day: day,
        completed_at: None,
        completed_world_day: None,
        completion_note: String::new(),
        abandoned_at: None,
        abandoned_world_day: None,
        abandonment_note: String::new(),
        origin_kind: kind,
        origin_ref,
    };
    create_quest(&conn, &quest).map_err(|e| e.to_string())?;
    Ok(quest)
}

#[tauri::command]
pub fn list_quests_cmd(db: State<'_, Database>, world_id: String) -> Result<Vec<Quest>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_quests(&conn, &world_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_quest_cmd(db: State<'_, Database>, quest_id: String) -> Result<Quest, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_quest(&conn, &quest_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_quest_cmd(
    db: State<'_, Database>,
    quest_id: String,
    title: String,
    description: String,
) -> Result<Quest, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    update_quest_title_description(&conn, &quest_id, title.trim(), description.trim())
        .map_err(|e| e.to_string())?;
    get_quest(&conn, &quest_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_quest_notes_cmd(
    db: State<'_, Database>,
    quest_id: String,
    notes: String,
) -> Result<Quest, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    update_quest_notes(&conn, &quest_id, &notes).map_err(|e| e.to_string())?;
    get_quest(&conn, &quest_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn complete_quest_cmd(
    db: State<'_, Database>,
    quest_id: String,
    completion_note: String,
) -> Result<Quest, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let quest = get_quest(&conn, &quest_id).map_err(|e| e.to_string())?;
    let world = get_world(&conn, &quest.world_id).map_err(|e| e.to_string())?;
    let day = current_world_day(&world);
    let now = Utc::now().to_rfc3339();
    mark_quest_complete(&conn, &quest_id, &now, day, completion_note.trim())
        .map_err(|e| e.to_string())?;
    get_quest(&conn, &quest_id).map_err(|e| e.to_string())
}

/// Abandonment is its own ceremony — "this no longer fits" is a
/// meaningful act, not a silent deletion. The user names WHY it's
/// being abandoned; that becomes part of the accumulated history.
#[tauri::command]
pub fn abandon_quest_cmd(
    db: State<'_, Database>,
    quest_id: String,
    abandonment_note: String,
) -> Result<Quest, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let quest = get_quest(&conn, &quest_id).map_err(|e| e.to_string())?;
    let world = get_world(&conn, &quest.world_id).map_err(|e| e.to_string())?;
    let day = current_world_day(&world);
    let now = Utc::now().to_rfc3339();
    mark_quest_abandoned(&conn, &quest_id, &now, day, abandonment_note.trim())
        .map_err(|e| e.to_string())?;
    get_quest(&conn, &quest_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reopen_quest_cmd(db: State<'_, Database>, quest_id: String) -> Result<Quest, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    reopen_quest(&conn, &quest_id).map_err(|e| e.to_string())?;
    get_quest(&conn, &quest_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_quest_cmd(db: State<'_, Database>, quest_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    delete_quest(&conn, &quest_id).map_err(|e| e.to_string())
}

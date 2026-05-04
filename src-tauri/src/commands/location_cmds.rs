use crate::ai::orchestrator;
use crate::commands::chat_cmds::world_time_fields;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatLocationResponse {
    /// The chat's new current_location after the update.
    pub current_location: Option<String>,
    /// The location_change Message that was inserted into chat history
    /// (None when the new location equals the old, which is a no-op).
    pub message: Option<Message>,
    /// The newly-saved place if `save_to_library` was true and the place
    /// wasn't already in the library; None otherwise.
    pub saved_place: Option<SavedPlace>,
}

/// Read the current location for an individual chat (by character_id) or
/// a group chat (by group_chat_id). Exactly one of the two must be set.
#[tauri::command]
pub fn get_chat_location_cmd(
    db: State<Database>,
    character_id: Option<String>,
    group_chat_id: Option<String>,
) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    if let Some(gc_id) = &group_chat_id {
        get_group_chat_location(&conn, gc_id).map_err(|e| e.to_string())
    } else if let Some(char_id) = &character_id {
        let thread = get_thread_for_character(&conn, char_id).map_err(|e| e.to_string())?;
        get_thread_location(&conn, &thread.thread_id).map_err(|e| e.to_string())
    } else {
        Err("Either character_id or group_chat_id must be provided".to_string())
    }
}

/// Set the current location for an individual or group chat. Inserts a
/// `location_change` message into chat history (so the LLM sees the
/// scene moved on the next call) and updates the chat row's
/// current_location. If `save_to_library` is true and the place is not
/// already in `saved_places`, the place is added.
///
/// A no-op (same as previous location) returns success but does NOT
/// insert a message — keeps the history clean of redundant cards.
#[tauri::command]
pub fn set_chat_location_cmd(
    db: State<Database>,
    character_id: Option<String>,
    group_chat_id: Option<String>,
    location: String,
    save_to_library: bool,
    // Optional API key for the background location-derivation refresh.
    // When omitted (legacy frontend code paths) the location is set as
    // usual but the (world, name) derivation is NOT auto-generated; the
    // prompt path renders gracefully without it. New frontend callers
    // should pass the user's API key to keep the derivation cache
    // populated as locations come into use.
    api_key: Option<String>,
) -> Result<ChatLocationResponse, String> {
    let trimmed = location.trim().to_string();
    if trimmed.is_empty() {
        return Err("Location must not be empty".to_string());
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Resolve thread_id, world_id, and previous location.
    let (thread_id, world_id, previous): (String, String, Option<String>) = if let Some(gc_id) = &group_chat_id {
        let gc = get_group_chat(&conn, gc_id).map_err(|e| e.to_string())?;
        let prev = get_group_chat_location(&conn, gc_id).map_err(|e| e.to_string())?;
        (gc.thread_id, gc.world_id, prev)
    } else if let Some(char_id) = &character_id {
        let thread = get_thread_for_character(&conn, char_id).map_err(|e| e.to_string())?;
        let ch = get_character(&conn, char_id).map_err(|e| e.to_string())?;
        let prev = get_thread_location(&conn, &thread.thread_id).map_err(|e| e.to_string())?;
        (thread.thread_id, ch.world_id, prev)
    } else {
        return Err("Either character_id or group_chat_id must be provided".to_string());
    };

    let now = Utc::now().to_rfc3339();

    // No-op when unchanged. Still honor save_to_library so the user can
    // promote the existing place into the library without churn. Don't
    // bump last_used_at on a no-op — the place wasn't freshly chosen.
    if previous.as_deref() == Some(trimmed.as_str()) {
        let saved_place = if save_to_library {
            maybe_save_place(&conn, &world_id, &trimmed, &now)?
        } else {
            None
        };
        return Ok(ChatLocationResponse {
            current_location: previous,
            message: None,
            saved_place,
        });
    }

    let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
    let (wd, wt) = world_time_fields(&world);

    // Persist the new location on the chat row first so any concurrent
    // reads see the truth even before the message lands.
    if group_chat_id.is_some() {
        set_group_chat_location(&conn, group_chat_id.as_deref().unwrap(), Some(&trimmed))
            .map_err(|e| e.to_string())?;
    } else {
        set_thread_location(&conn, &thread_id, Some(&trimmed))
            .map_err(|e| e.to_string())?;
    }

    // Build the location_change message. content is JSON so the renderer
    // can show "Location changed from X to Y" (with from omitted when
    // first-set), and the LLM-history serializer can format it as a
    // [Location changed to ...] system line.
    let payload = json!({
        "from": previous.clone(),
        "to": trimmed.clone(),
    }).to_string();

    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread_id.clone(),
        role: "location_change".to_string(),
        content: payload,
        tokens_estimate: 0,
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
        world_day: wd,
        world_time: wt,
        address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
    };

    if group_chat_id.is_some() {
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    } else {
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    let saved_place = if save_to_library {
        maybe_save_place(&conn, &world_id, &trimmed, &now)?
    } else {
        None
    };

    // Always touch last_used_at when an existing saved place matches
    // (regardless of save_to_library). If save_to_library just CREATED
    // the row, its last_used_at already equals created_at == now, so
    // the touch is a harmless no-op on the same value.
    touch_saved_place(&conn, &world_id, &trimmed, &now).map_err(|e| e.to_string())?;

    // Snapshot what we need for the background derivation before
    // dropping the lock guard.
    let model_config = orchestrator::load_model_config(&conn);
    drop(conn);

    // Fire-and-forget background derivation for this (world, location)
    // pair. No-ops when api_key is None or empty, when cache already
    // has this pair, or when another task is already inflight. See
    // ai::derivation::maybe_refresh_location for the dedupe + caching
    // contract.
    if let Some(key) = api_key {
        crate::ai::derivation::maybe_refresh_location(
            db.conn.clone(),
            model_config.chat_api_base(),
            key,
            model_config.memory_model.clone(),
            world_id.clone(),
            trimmed.clone(),
        );
    }

    Ok(ChatLocationResponse {
        current_location: Some(trimmed),
        message: Some(msg),
        saved_place,
    })
}

fn maybe_save_place(
    conn: &rusqlite::Connection,
    world_id: &str,
    name: &str,
    when: &str,
) -> Result<Option<SavedPlace>, String> {
    let existing = list_saved_places(conn, world_id).map_err(|e| e.to_string())?;
    if existing.iter().any(|p| p.name.eq_ignore_ascii_case(name)) {
        return Ok(None);
    }
    let place = SavedPlace {
        saved_place_id: uuid::Uuid::new_v4().to_string(),
        world_id: world_id.to_string(),
        name: name.to_string(),
        created_at: when.to_string(),
        last_used_at: when.to_string(),
    };
    create_saved_place(conn, &place).map_err(|e| e.to_string())?;
    Ok(Some(place))
}

#[tauri::command]
pub fn list_saved_places_cmd(
    db: State<Database>,
    world_id: String,
) -> Result<Vec<SavedPlace>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_saved_places(&conn, &world_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_saved_place_cmd(
    db: State<Database>,
    saved_place_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    delete_saved_place(&conn, &saved_place_id).map_err(|e| e.to_string())
}

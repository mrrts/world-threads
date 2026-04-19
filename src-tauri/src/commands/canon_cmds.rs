use crate::ai::orchestrator;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

const CONTEXT_BEFORE: i64 = 3;
const CONTEXT_AFTER: i64 = 3;

/// Find the source message across both message tables.
fn find_message(conn: &rusqlite::Connection, message_id: &str) -> Option<(Message, String)> {
    // tuple: (message, table_name)
    if let Ok(m) = conn.query_row(
        &format!("SELECT {MSG_COLS} FROM messages WHERE message_id = ?1"),
        params![message_id], row_to_message,
    ) {
        return Some((m, "messages".to_string()));
    }
    if let Ok(m) = conn.query_row(
        &format!("SELECT {MSG_COLS} FROM group_messages WHERE message_id = ?1"),
        params![message_id], row_to_message,
    ) {
        return Some((m, "group_messages".to_string()));
    }
    None
}

/// Pull a window of messages surrounding the source, from whichever table
/// it lives in. Used to give the weave prompt enough context that it can
/// tell what kind of moment the source line is.
fn surrounding_messages(
    conn: &rusqlite::Connection,
    table: &str,
    thread_id: &str,
    source_created_at: &str,
) -> Vec<Message> {
    let before_sql = format!(
        "SELECT {MSG_COLS} FROM {table} WHERE thread_id = ?1 AND created_at < ?2 ORDER BY created_at DESC LIMIT ?3"
    );
    let after_sql = format!(
        "SELECT {MSG_COLS} FROM {table} WHERE thread_id = ?1 AND created_at > ?2 ORDER BY created_at ASC LIMIT ?3"
    );
    let source_sql = format!(
        "SELECT {MSG_COLS} FROM {table} WHERE thread_id = ?1 AND created_at = ?2"
    );

    let mut before: Vec<Message> = conn.prepare(&before_sql).ok()
        .and_then(|mut s| {
            s.query_map(params![thread_id, source_created_at, CONTEXT_BEFORE], row_to_message)
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();
    before.reverse();

    let source_rows: Vec<Message> = conn.prepare(&source_sql).ok()
        .and_then(|mut s| {
            s.query_map(params![thread_id, source_created_at], row_to_message)
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

    let after: Vec<Message> = conn.prepare(&after_sql).ok()
        .and_then(|mut s| {
            s.query_map(params![thread_id, source_created_at, CONTEXT_AFTER], row_to_message)
                .ok()
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

    let mut out = Vec::with_capacity(before.len() + source_rows.len() + after.len());
    out.extend(before);
    out.extend(source_rows);
    out.extend(after);
    out
}

/// Resolve the display-name label to use for the speaker of a message.
/// For user messages → the user's display_name (or "User"). For assistant
/// messages in a solo thread → the character. For group → the sender.
fn speaker_label_for(
    conn: &rusqlite::Connection,
    msg: &Message,
    user_display_name: &str,
) -> String {
    if msg.role == "user" {
        return user_display_name.to_string();
    }
    if let Some(sender_id) = msg.sender_character_id.as_deref() {
        if let Ok(ch) = get_character(conn, sender_id) {
            return ch.display_name;
        }
    }
    // Solo chat assistant fallback: look up the thread's character.
    let char_id: Option<String> = conn.query_row(
        "SELECT character_id FROM threads WHERE thread_id = ?1",
        params![msg.thread_id], |r| r.get(0),
    ).ok();
    if let Some(cid) = char_id {
        if let Ok(ch) = get_character(conn, &cid) {
            return ch.display_name;
        }
    }
    "Character".to_string()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeaveRequest {
    pub source_message_id: String,
    pub subject_type: String,
    pub subject_id: String,
}

#[derive(Debug, Serialize)]
pub struct WeaveResponse {
    pub current_description: String,
    pub proposed_description: String,
}

/// Run the LLM weave call. Returns the current description (so the UI can
/// diff) plus the proposed revision. Does NOT persist anything.
#[tauri::command]
pub async fn canonize_weave_description_cmd(
    db: State<'_, Database>,
    api_key: String,
    request: WeaveRequest,
) -> Result<WeaveResponse, String> {
    // Read everything needed up-front (lock released before awaiting).
    let (model_config, subject_label, current_description, context_msgs, source_msg, source_speaker_label, world_id_for_user) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);

        let (source_msg, table) = find_message(&conn, &request.source_message_id)
            .ok_or_else(|| "source message not found".to_string())?;

        // User display name (for labeling user-role messages in context).
        let user_display_name: String = get_world(&conn, &{
            // find world via thread
            let thread_world: Option<String> = conn.query_row(
                "SELECT world_id FROM threads WHERE thread_id = ?1",
                params![source_msg.thread_id], |r| r.get(0),
            ).ok();
            thread_world.unwrap_or_default()
        })
            .ok()
            .and_then(|w| get_user_profile(&conn, &w.world_id).ok())
            .map(|p| p.display_name)
            .unwrap_or_else(|| "The human".to_string());

        // Pull current description + subject label based on subject_type.
        let (subject_label, current_description, world_id_for_user) = match request.subject_type.as_str() {
            "character" => {
                let ch = get_character(&conn, &request.subject_id).map_err(|e| e.to_string())?;
                (ch.display_name, ch.identity, None::<String>)
            }
            "user" => {
                // subject_id is world_id for user
                let profile = get_user_profile(&conn, &request.subject_id).map_err(|e| e.to_string())?;
                (profile.display_name, profile.description, Some(request.subject_id.clone()))
            }
            other => return Err(format!("weave not supported for subject_type={other}")),
        };

        let context_msgs = surrounding_messages(&conn, &table, &source_msg.thread_id, &source_msg.created_at);
        let speaker_label = speaker_label_for(&conn, &source_msg, &user_display_name);

        (model_config, subject_label, current_description, context_msgs, source_msg, speaker_label, world_id_for_user)
    };

    let _ = world_id_for_user; // kept for future use; not needed by weave itself

    let (proposed, usage) = orchestrator::generate_canon_weave_description(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &subject_label,
        &current_description,
        &context_msgs,
        &source_msg,
        &source_speaker_label,
    ).await?;

    if let Some(u) = &usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "canon_weave", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    Ok(WeaveResponse {
        current_description,
        proposed_description: proposed,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCanonRequest {
    pub source_message_id: Option<String>,
    pub subject_type: String,      // "character" | "user" | "world" | "relationship"
    pub subject_id: String,        // character_id | world_id | world_id | "char_a::char_b|user"
    pub canon_type: String,        // "description_weave" | "known_fact" | "relationship_note" | "world_fact"
    pub content: String,
    #[serde(default)]
    pub user_note: String,
}

/// Persist a canon entry AND apply its side effect to the target row.
#[tauri::command]
pub fn save_canon_entry_cmd(
    db: State<Database>,
    request: SaveCanonRequest,
) -> Result<CanonEntry, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Look up source message metadata for provenance (if provided).
    let (source_thread_id, source_world_day, source_created_at) = match request.source_message_id.as_deref() {
        Some(mid) if !mid.is_empty() => {
            match find_message(&conn, mid) {
                Some((m, _)) => (Some(m.thread_id), m.world_day, Some(m.created_at)),
                None => (None, None, None),
            }
        }
        _ => (None, None, None),
    };

    // Apply side effect to the subject row.
    match (request.subject_type.as_str(), request.canon_type.as_str()) {
        ("character", "description_weave") => {
            conn.execute(
                "UPDATE characters SET identity = ?2, updated_at = datetime('now') WHERE character_id = ?1",
                params![request.subject_id, request.content],
            ).map_err(|e| e.to_string())?;
        }
        ("character", "known_fact") => {
            let ch = get_character(&conn, &request.subject_id).map_err(|e| e.to_string())?;
            let mut facts: Vec<Value> = ch.backstory_facts.as_array().cloned().unwrap_or_default();
            facts.push(Value::String(request.content.clone()));
            let new_facts = Value::Array(facts);
            conn.execute(
                "UPDATE characters SET backstory_facts = ?2, updated_at = datetime('now') WHERE character_id = ?1",
                params![request.subject_id, new_facts.to_string()],
            ).map_err(|e| e.to_string())?;
        }
        ("character", "relationship_note") => {
            // subject_id encodes "<char_id>::<other>" where other = character_id or "user"
            let parts: Vec<&str> = request.subject_id.splitn(2, "::").collect();
            if parts.len() != 2 {
                return Err(format!("relationship subject_id must be 'char_a::char_b|user', got {}", request.subject_id));
            }
            let char_a = parts[0];
            let other = parts[1];
            let ch = get_character(&conn, char_a).map_err(|e| e.to_string())?;
            let mut rels = ch.relationships.as_object().cloned().unwrap_or_default();
            let existing = rels.remove(other).unwrap_or_else(|| json!({ "notes": [] }));
            let mut obj = existing.as_object().cloned().unwrap_or_default();
            let mut notes: Vec<Value> = obj.get("notes").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            notes.push(Value::String(request.content.clone()));
            obj.insert("notes".to_string(), Value::Array(notes));
            rels.insert(other.to_string(), Value::Object(obj));
            conn.execute(
                "UPDATE characters SET relationships = ?2, updated_at = datetime('now') WHERE character_id = ?1",
                params![char_a, Value::Object(rels).to_string()],
            ).map_err(|e| e.to_string())?;
        }
        ("user", "description_weave") => {
            conn.execute(
                "UPDATE user_profiles SET description = ?2, updated_at = datetime('now') WHERE world_id = ?1",
                params![request.subject_id, request.content],
            ).map_err(|e| e.to_string())?;
        }
        ("user", "known_fact") => {
            let profile = get_user_profile(&conn, &request.subject_id).map_err(|e| e.to_string())?;
            let mut facts: Vec<Value> = profile.facts.as_array().cloned().unwrap_or_default();
            facts.push(Value::String(request.content.clone()));
            conn.execute(
                "UPDATE user_profiles SET facts = ?2, updated_at = datetime('now') WHERE world_id = ?1",
                params![request.subject_id, Value::Array(facts).to_string()],
            ).map_err(|e| e.to_string())?;
        }
        ("world", "world_fact") => {
            let w = get_world(&conn, &request.subject_id).map_err(|e| e.to_string())?;
            let mut invariants: Vec<Value> = w.invariants.as_array().cloned().unwrap_or_default();
            invariants.push(Value::String(request.content.clone()));
            conn.execute(
                "UPDATE worlds SET invariants = ?2, updated_at = datetime('now') WHERE world_id = ?1",
                params![request.subject_id, Value::Array(invariants).to_string()],
            ).map_err(|e| e.to_string())?;
        }
        (st, ct) => return Err(format!("unsupported (subject_type, canon_type) = ({st}, {ct})")),
    }

    let entry = CanonEntry {
        canon_id: uuid::Uuid::new_v4().to_string(),
        source_message_id: request.source_message_id.clone(),
        source_thread_id,
        source_world_day,
        source_created_at,
        subject_type: request.subject_type,
        subject_id: request.subject_id,
        canon_type: request.canon_type,
        content: request.content,
        user_note: request.user_note,
        created_at: Utc::now().to_rfc3339(),
    };
    create_canon_entry(&conn, &entry).map_err(|e| e.to_string())?;
    Ok(entry)
}

/// Return the distinct set of message IDs (in the current thread) that have
/// been canonized at least once — drives the "this moment is canon"
/// indicator on messages.
#[tauri::command]
pub fn list_canonized_message_ids_cmd(
    db: State<Database>,
    thread_id: String,
) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_canonized_message_ids_for_thread(&conn, &thread_id).map_err(|e| e.to_string())
}

/// All canon entries tied to a given message (for the tooltip + undo list).
#[tauri::command]
pub fn list_canon_for_message_cmd(
    db: State<Database>,
    message_id: String,
) -> Result<Vec<CanonEntry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_canon_for_message(&conn, &message_id).map_err(|e| e.to_string())
}

/// Remove a canon entry. NOTE: this removes the provenance row only; it
/// does NOT attempt to undo the side effect on the subject row (e.g. roll
/// back a character description). Undo of the side effect would need a
/// separate path that snapshots the pre-state.
#[tauri::command]
pub fn delete_canon_entry_cmd(
    db: State<Database>,
    canon_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    delete_canon_entry(&conn, &canon_id).map_err(|e| e.to_string())
}

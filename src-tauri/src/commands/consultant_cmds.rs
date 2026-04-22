use crate::ai::openai::{self, ChatRequest, ChatMessage, StreamingRequest};
use crate::ai::orchestrator;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsultantChat {
    pub chat_id: String,
    pub thread_id: String,
    pub title: String,
    pub created_at: String,
    pub last_seen_message_id: Option<String>,
    /// "immersive" (default — the in-the-story confidant) or "backstage"
    /// (the fourth-wall stage manager who reads the save file). Scoping
    /// chats by mode keeps the two voices cleanly separated in history.
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsultantMessage {
    pub role: String,
    pub content: String,
}

// ─── Chat CRUD ─────────────────────────────────────────────────────────────

/// Create a new consultant chat session for a thread. `mode` is
/// "immersive" (default) or "backstage" — the latter flips the system
/// prompt to the fourth-wall stage manager on send.
#[tauri::command]
pub fn create_consultant_chat_cmd(
    db: State<'_, Database>,
    thread_id: String,
    title: Option<String>,
    mode: Option<String>,
) -> Result<ConsultantChat, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let last_msg_id: Option<String> = conn.query_row(
        "SELECT message_id FROM messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT 1",
        params![thread_id], |r| r.get(0),
    ).ok().or_else(|| conn.query_row(
        "SELECT message_id FROM group_messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT 1",
        params![thread_id], |r| r.get(0),
    ).ok());
    let mode = mode.filter(|m| m == "immersive" || m == "backstage")
        .unwrap_or_else(|| "immersive".to_string());
    let chat = ConsultantChat {
        chat_id: uuid::Uuid::new_v4().to_string(),
        thread_id,
        title: title.unwrap_or_else(|| "New Chat".to_string()),
        created_at: Utc::now().to_rfc3339(),
        last_seen_message_id: last_msg_id.clone(),
        mode,
    };
    conn.execute(
        "INSERT INTO consultant_chats (chat_id, thread_id, title, created_at, last_seen_message_id, mode) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![chat.chat_id, chat.thread_id, chat.title, chat.created_at, last_msg_id, chat.mode],
    ).map_err(|e| e.to_string())?;
    Ok(chat)
}

/// List all consultant chats for a thread, most recent first. Returns
/// both modes; the frontend tabs filter in-memory.
#[tauri::command]
pub fn list_consultant_chats_cmd(
    db: State<'_, Database>,
    thread_id: String,
) -> Result<Vec<ConsultantChat>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT chat_id, thread_id, title, created_at, last_seen_message_id, mode FROM consultant_chats WHERE thread_id = ?1 ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![thread_id], |row| {
        Ok(ConsultantChat {
            chat_id: row.get(0)?,
            thread_id: row.get(1)?,
            title: row.get(2)?,
            created_at: row.get(3)?,
            last_seen_message_id: row.get(4).ok(),
            mode: row.get::<_, Option<String>>(5)?.unwrap_or_else(|| "immersive".to_string()),
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Update the title of a consultant chat.
#[tauri::command]
pub fn update_consultant_chat_title_cmd(
    db: State<'_, Database>,
    chat_id: String,
    title: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE consultant_chats SET title = ?2 WHERE chat_id = ?1",
        params![chat_id, title],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete a consultant chat and all its messages.
#[tauri::command]
pub fn delete_consultant_chat_cmd(
    db: State<'_, Database>,
    chat_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM consultant_messages WHERE chat_id = ?1", params![chat_id]).ok();
    conn.execute("DELETE FROM consultant_chats WHERE chat_id = ?1", params![chat_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Message CRUD ──────────────────────────────────────────────────────────

/// Load messages for a specific consultant chat.
#[tauri::command]
pub fn load_consultant_chat_cmd(
    db: State<'_, Database>,
    chat_id: String,
) -> Result<Vec<ConsultantMessage>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT role, content FROM consultant_messages WHERE chat_id = ?1 ORDER BY id ASC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![chat_id], |row| {
        Ok(ConsultantMessage { role: row.get(0)?, content: row.get(1)? })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Clear all messages in a consultant chat.
#[tauri::command]
pub fn clear_consultant_chat_cmd(
    db: State<'_, Database>,
    chat_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM consultant_messages WHERE chat_id = ?1", params![chat_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Keep only the first N messages in a consultant chat.
#[tauri::command]
pub fn truncate_consultant_chat_cmd(
    db: State<'_, Database>,
    chat_id: String,
    keep_count: i64,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM consultant_messages WHERE chat_id = ?1 AND id NOT IN (SELECT id FROM consultant_messages WHERE chat_id = ?1 ORDER BY id ASC LIMIT ?2)",
        params![chat_id, keep_count],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Replace all messages in a consultant chat.
#[tauri::command]
pub fn save_consultant_messages_cmd(
    db: State<'_, Database>,
    chat_id: String,
    messages: Vec<ConsultantMessage>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM consultant_messages WHERE chat_id = ?1", params![chat_id])
        .map_err(|e| e.to_string())?;
    for msg in &messages {
        conn.execute(
            "INSERT INTO consultant_messages (chat_id, role, content) VALUES (?1, ?2, ?3)",
            params![chat_id, msg.role, msg.content],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ─── Story consultant LLM call ─────────────────────────────────────────────

/// Import chat messages since the last import (or since the consultant chat began).
/// Returns the formatted import summary line for UI display.
#[tauri::command]
pub fn import_chat_messages_cmd(
    db: State<'_, Database>,
    chat_id: String,
    character_id: Option<String>,
    group_chat_id: Option<String>,
) -> Result<ConsultantMessage, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let is_group = group_chat_id.is_some();

    // Get the last seen message ID for this consultant chat
    let last_seen: Option<String> = conn.query_row(
        "SELECT last_seen_message_id FROM consultant_chats WHERE chat_id = ?1",
        params![chat_id], |r| r.get(0),
    ).ok().flatten();

    let (new_msgs, characters, user_name, _thread_id) = if is_group {
        let gc = get_group_chat(&conn, group_chat_id.as_deref().unwrap()).map_err(|e| e.to_string())?;
        let all_msgs = get_all_group_messages(&conn, &gc.thread_id).map_err(|e| e.to_string())?;
        let user_name = get_user_profile(&conn, &gc.world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the user".to_string());
        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();
        // Filter to messages after last_seen
        let msgs = if let Some(ref seen_id) = last_seen {
            let idx = all_msgs.iter().position(|m| m.message_id == *seen_id);
            match idx {
                Some(i) => all_msgs[i + 1..].to_vec(),
                None => all_msgs.into_iter().rev().take(30).collect::<Vec<_>>().into_iter().rev().collect(),
            }
        } else {
            // No last_seen — take the most recent 30
            all_msgs.into_iter().rev().take(30).collect::<Vec<_>>().into_iter().rev().collect()
        };
        (msgs, characters, user_name, gc.thread_id)
    } else {
        let char_id = character_id.as_deref().ok_or("No character specified")?;
        let character = get_character(&conn, char_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, char_id).map_err(|e| e.to_string())?;
        let all_msgs = get_all_messages(&conn, &thread.thread_id).map_err(|e| e.to_string())?;
        let user_name = get_user_profile(&conn, &character.world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the user".to_string());
        let msgs = if let Some(ref seen_id) = last_seen {
            let idx = all_msgs.iter().position(|m| m.message_id == *seen_id);
            match idx {
                Some(i) => all_msgs[i + 1..].to_vec(),
                None => all_msgs.into_iter().rev().take(30).collect::<Vec<_>>().into_iter().rev().collect(),
            }
        } else {
            all_msgs.into_iter().rev().take(30).collect::<Vec<_>>().into_iter().rev().collect()
        };
        (msgs, vec![character], user_name, thread.thread_id)
    };

    if new_msgs.is_empty() {
        return Err("No new messages since last import.".to_string());
    }

    // Format messages
    let conversation: Vec<String> = new_msgs.iter()
        .filter(|m| m.role != "illustration" && m.role != "video" && m.role != "inventory_update")
        .map(|m| {
            let speaker = match m.role.as_str() {
                "user" => user_name.clone(),
                "narrative" => "[Narrative]".to_string(),
                "context" => "[Context]".to_string(),
                "assistant" => {
                    m.sender_character_id.as_ref()
                        .and_then(|id| characters.iter().find(|c| c.character_id == *id))
                        .map(|c| c.display_name.clone())
                        .unwrap_or_else(|| "Character".to_string())
                }
                _ => m.role.clone(),
            };
            format!("{}: {}", speaker, m.content)
        })
        .collect();

    let char_names: Vec<String> = characters.iter().map(|c| c.display_name.clone()).collect();
    let msg_count = new_msgs.len();
    let label = format!("Imported {} new messages with {}", msg_count, char_names.join(" & "));
    let content = format!("{}\n---\n{}", label, conversation.join("\n"));

    // Update last_seen_message_id to the latest message
    let new_last_seen = new_msgs.last().map(|m| m.message_id.clone());
    if let Some(ref id) = new_last_seen {
        conn.execute(
            "UPDATE consultant_chats SET last_seen_message_id = ?2 WHERE chat_id = ?1",
            params![chat_id, id],
        ).map_err(|e| e.to_string())?;
    }

    // Persist as import message
    conn.execute(
        "INSERT INTO consultant_messages (chat_id, role, content) VALUES (?1, 'import', ?2)",
        params![chat_id, content],
    ).map_err(|e| e.to_string())?;

    Ok(ConsultantMessage {
        role: "import".to_string(),
        content,
    })
}

/// Get the last seen message for a consultant chat (for preview on hover).
#[derive(Debug, Serialize, Deserialize)]
pub struct LastSeenPreview {
    pub message_id: String,
    pub role: String,
    pub content: String,
    pub speaker_name: String,
    pub character_id: Option<String>,
    pub avatar_color: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub fn get_last_seen_message_cmd(
    db: State<'_, Database>,
    chat_id: String,
) -> Result<Option<LastSeenPreview>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let last_seen_id: Option<String> = conn.query_row(
        "SELECT last_seen_message_id FROM consultant_chats WHERE chat_id = ?1",
        params![chat_id], |r| r.get(0),
    ).ok().flatten();

    let Some(msg_id) = last_seen_id else { return Ok(None) };

    // Try individual messages first, then group messages
    let msg: Option<Message> = conn.query_row(
        &format!("SELECT {} FROM messages WHERE message_id = ?1", crate::db::queries::MSG_COLS),
        params![msg_id], crate::db::queries::row_to_message,
    ).ok().or_else(|| conn.query_row(
        &format!("SELECT {} FROM group_messages WHERE message_id = ?1", crate::db::queries::MSG_COLS),
        params![msg_id], crate::db::queries::row_to_message,
    ).ok());

    let Some(m) = msg else { return Ok(None) };
    if m.role == "illustration" || m.role == "video" { return Ok(None); }

    // Look up the character — try sender_character_id first, then thread's character_id
    let character = m.sender_character_id.as_ref()
        .and_then(|id| get_character(&conn, id).ok())
        .or_else(|| {
            // For individual chats, get the character from the thread
            conn.query_row(
                "SELECT character_id FROM threads WHERE thread_id = ?1",
                params![m.thread_id], |r| r.get::<_, Option<String>>(0),
            ).ok().flatten().and_then(|id| get_character(&conn, &id).ok())
        });

    let (speaker_name, avatar_color) = match m.role.as_str() {
        "user" => {
            let world_id: Option<String> = conn.query_row(
                "SELECT world_id FROM threads WHERE thread_id = ?1",
                params![m.thread_id], |r| r.get(0),
            ).ok();
            let name = world_id.and_then(|wid| get_user_profile(&conn, &wid).ok().map(|p| p.display_name))
                .unwrap_or_else(|| "You".to_string());
            (name, None)
        }
        "assistant" => {
            let name = character.as_ref().map(|c| c.display_name.clone()).unwrap_or_else(|| "Character".to_string());
            let color = character.as_ref().map(|c| c.avatar_color.clone());
            (name, color)
        }
        "narrative" => ("Narrative".to_string(), None),
        "context" => ("Context".to_string(), None),
        _ => (m.role.clone(), None),
    };

    Ok(Some(LastSeenPreview {
        message_id: m.message_id,
        role: m.role,
        content: m.content,
        speaker_name,
        character_id: character.as_ref().map(|c| c.character_id.clone()),
        avatar_color,
        created_at: m.created_at,
    }))
}

/// Generate a short title for a consultant chat based on the first message.
#[tauri::command]
pub async fn generate_consultant_title_cmd(
    db: State<'_, Database>,
    api_key: String,
    user_message: String,
) -> Result<String, String> {
    let model_config = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        orchestrator::load_model_config(&conn)
    };

    let messages = vec![
        ChatMessage { role: "system".to_string(), content: "Generate a very short title (3-6 words) for a story consultation chat that starts with this question. Reply with ONLY the title, no quotes or punctuation.".to_string() },
        ChatMessage { role: "user".to_string(), content: user_message },
    ];

    let request = ChatRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.7),
        max_completion_tokens: Some(20),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(
        &model_config.chat_api_base(), &api_key, &request,
    ).await?;

    Ok(response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_else(|| "Story Chat".to_string()))
}

/// Send a message to the story consultant and get a streamed response.
/// Emits "consultant-token" events with each token chunk.
#[tauri::command]
pub async fn story_consultant_cmd(
    db: State<'_, Database>,
    app_handle: AppHandle,
    api_key: String,
    chat_id: String,
    character_id: Option<String>,
    group_chat_id: Option<String>,
    user_message: String,
) -> Result<String, String> {
    let is_group = group_chat_id.is_some();

    // Resolve this chat's mode once up-front — drives the system prompt
    // branch below. "immersive" is the default when the row is missing a
    // value (old rows predating the column).
    let chat_mode: String = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT mode FROM consultant_chats WHERE chat_id = ?1",
            params![chat_id], |r| r.get::<_, Option<String>>(0),
        ).ok().flatten().unwrap_or_else(|| "immersive".to_string())
    };

    let (world, characters, recent_msgs, user_profile, thread_id, model_config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);

        if is_group {
            let gc = get_group_chat(&conn, group_chat_id.as_deref().unwrap()).map_err(|e| e.to_string())?;
            let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
            let recent_msgs = list_group_messages(&conn, &gc.thread_id, 30).map_err(|e| e.to_string())?;
            let user_profile = get_user_profile(&conn, &gc.world_id).ok();
            let char_ids: Vec<String> = gc.character_ids.as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let characters: Vec<Character> = char_ids.iter()
                .filter_map(|id| get_character(&conn, id).ok())
                .collect();
            (world, characters, recent_msgs, user_profile, gc.thread_id, model_config)
        } else {
            let char_id = character_id.as_deref().ok_or("No character specified")?;
            let character = get_character(&conn, char_id).map_err(|e| e.to_string())?;
            let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
            let thread = get_thread_for_character(&conn, char_id).map_err(|e| e.to_string())?;
            let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;
            let user_profile = get_user_profile(&conn, &character.world_id).ok();
            (world, vec![character], recent_msgs, user_profile, thread.thread_id, model_config)
        }
    };

    let user_name = user_profile.as_ref().map(|p| p.display_name.clone()).unwrap_or_else(|| "the user".to_string());

    // Thread summary + kept records live in DB; pull them on their own
    // connection grab so we don't hold the lock during the earlier reads.
    let (thread_summary, kept_records) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let summary = get_thread_summary(&conn, &thread_id);
        // Kept records whose subject is one of the characters, the user,
        // or the world. Ordered newest first, capped so a long-lived
        // canon history doesn't blow up the prompt.
        let mut subj_ids: Vec<(String, String)> = characters.iter()
            .map(|c| ("character".to_string(), c.character_id.clone()))
            .collect();
        subj_ids.push(("user".to_string(), world.world_id.clone()));
        subj_ids.push(("world".to_string(), world.world_id.clone()));
        let placeholders = subj_ids.iter().map(|_| "(?,?)").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT subject_type, subject_id, record_type, content, source_world_day, created_at
             FROM kept_records
             WHERE (subject_type, subject_id) IN ({placeholders})
             ORDER BY created_at DESC LIMIT 20"
        );
        let mut kept: Vec<(String, String, String, String, Option<i64>, String)> = Vec::new();
        if let Ok(mut stmt) = conn.prepare(&sql) {
            // Flatten tuples into positional params.
            let flat: Vec<Box<dyn rusqlite::ToSql>> = subj_ids.iter()
                .flat_map(|(t, i)| [Box::new(t.clone()) as Box<dyn rusqlite::ToSql>, Box::new(i.clone())])
                .collect();
            let refs: Vec<&dyn rusqlite::ToSql> = flat.iter().map(|b| b.as_ref()).collect();
            if let Ok(rows) = stmt.query_map(&refs[..], |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, String>(5)?,
            ))) {
                for row in rows.flatten() { kept.push(row); }
            }
        }
        (summary, kept)
    };

    // Load persisted consultant history for this chat
    let consultant_history = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT role, content FROM consultant_messages WHERE chat_id = ?1 ORDER BY id ASC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![chat_id], |row| {
            let role: String = row.get(0)?;
            let content: String = row.get(1)?;
            // Map import messages to user role with context framing
            let mapped_role = if role == "import" { "user".to_string() } else { role };
            let mapped_content = if mapped_role == "user" && content.contains("\n---\n") {
                format!("[Here's what happened recently in the conversation:]\n{}", content.split("\n---\n").nth(1).unwrap_or(&content))
            } else {
                content
            };
            Ok(ChatMessage { role: mapped_role, content: mapped_content })
        }).map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
    };

    // Rich per-character dossiers: identity + backstory + relationships +
    // current state (mood / goals / open loops) + inventory + signature
    // emoji. Skips visual description, voice rules, and boundaries —
    // those are about performing the character, not understanding them.
    let char_descriptions: Vec<String> = characters.iter().map(|c| {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("### {}", c.display_name));
        if !c.identity.is_empty() {
            lines.push(c.identity.clone());
        }
        if !c.signature_emoji.trim().is_empty() {
            lines.push(format!("Signature emoji: {}", c.signature_emoji.trim()));
        }
        let backstory = crate::ai::prompts::json_array_to_strings(&c.backstory_facts);
        if !backstory.is_empty() {
            let block = backstory.iter().map(|b| format!("  - {b}")).collect::<Vec<_>>().join("\n");
            lines.push(format!("Backstory:\n{block}"));
        }
        if let Some(rel_obj) = c.relationships.as_object() {
            if !rel_obj.is_empty() {
                lines.push(format!(
                    "Relationships:\n{}",
                    serde_json::to_string_pretty(&c.relationships).unwrap_or_default()
                ));
            }
        }
        if let Some(state_obj) = c.state.as_object() {
            if !state_obj.is_empty() {
                lines.push(format!(
                    "Current state (mood, goals, open loops):\n{}",
                    serde_json::to_string_pretty(&c.state).unwrap_or_default()
                ));
            }
        }
        let inv_block = crate::ai::prompts::render_inventory_block(&c.display_name, &c.inventory);
        if !inv_block.is_empty() {
            lines.push(inv_block);
        }
        lines.join("\n\n")
    }).collect();

    // World block: description + invariants + current state (day, time,
    // weather). Small load-bearing pieces, not the whole JSON.
    let world_desc_rich = {
        let mut parts: Vec<String> = Vec::new();
        if !world.description.is_empty() {
            parts.push(world.description.clone());
        } else {
            parts.push("A richly detailed world.".to_string());
        }
        let invariants = crate::ai::prompts::json_array_to_strings(&world.invariants);
        if !invariants.is_empty() {
            let block = invariants.iter().map(|i| format!("  - {i}")).collect::<Vec<_>>().join("\n");
            parts.push(format!("World rules:\n{block}"));
        }
        if let Some(state_obj) = world.state.as_object() {
            let mut state_lines: Vec<String> = Vec::new();
            if let Some(time) = state_obj.get("time") {
                let day = time.get("day_index").and_then(|v| v.as_i64()).unwrap_or(0);
                let tod = time.get("time_of_day").and_then(|v| v.as_str()).unwrap_or("");
                if !tod.is_empty() { state_lines.push(format!("Day {day}, {tod}")); }
            }
            if let Some(weather_key) = state_obj.get("weather").and_then(|v| v.as_str()) {
                if let Some((emoji, label)) = crate::ai::prompts::weather_meta(weather_key) {
                    state_lines.push(format!("Weather: {emoji} {label}"));
                }
            }
            if let Some(arcs) = state_obj.get("global_arcs").and_then(|v| v.as_array()) {
                let arc_lines: Vec<String> = arcs.iter().filter_map(|a| {
                    let id = a.get("arc_id").and_then(|v| v.as_str())?;
                    let status = a.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    let notes = a.get("notes").and_then(|v| v.as_str()).unwrap_or("");
                    Some(format!("  - {id} ({status}): {notes}"))
                }).collect();
                if !arc_lines.is_empty() {
                    state_lines.push(format!("Ongoing arcs:\n{}", arc_lines.join("\n")));
                }
            }
            if let Some(facts) = state_obj.get("facts").and_then(|v| v.as_array()) {
                let fact_lines: Vec<String> = facts.iter().filter_map(|f| {
                    f.get("text").and_then(|v| v.as_str()).map(|t| format!("  - {t}"))
                }).collect();
                if !fact_lines.is_empty() {
                    state_lines.push(format!("Established world facts:\n{}", fact_lines.join("\n")));
                }
            }
            if !state_lines.is_empty() {
                parts.push(format!("Right now:\n{}", state_lines.join("\n")));
            }
        }
        parts.join("\n\n")
    };

    // User profile block: description + facts, so the consultant knows
    // who the user IS in this world, not just their name.
    let user_block_rich = {
        let mut lines: Vec<String> = vec![format!("### {} (the person talking to you)", user_name)];
        if let Some(ref p) = user_profile {
            if !p.description.is_empty() {
                lines.push(p.description.clone());
            }
            let facts = crate::ai::prompts::json_array_to_strings(&p.facts);
            if !facts.is_empty() {
                let block = facts.iter().map(|f| format!("  - {f}")).collect::<Vec<_>>().join("\n");
                lines.push(format!("Known facts about {user_name}:\n{block}"));
            }
        }
        lines.join("\n\n")
    };

    // Long-term memory surface for this thread — what the app has boiled
    // the story down to in its periodic summarization passes.
    let summary_block = if thread_summary.is_empty() {
        String::new()
    } else {
        format!("\n\nTHREAD SUMMARY (longer-arc memory, periodically regenerated):\n{thread_summary}")
    };

    // Kept records: moments the user explicitly chose to canonize about
    // any of the people or about the world. The consultant should read
    // these as settled canon — weighted heavier than any one scene.
    let kept_block = if kept_records.is_empty() {
        String::new()
    } else {
        let char_name_by_id: std::collections::HashMap<&str, &str> = characters.iter()
            .map(|c| (c.character_id.as_str(), c.display_name.as_str()))
            .collect();
        let lines: Vec<String> = kept_records.iter().map(|(subject_type, subject_id, record_type, content, world_day, _created_at)| {
            let subject_label = match subject_type.as_str() {
                "character" => char_name_by_id.get(subject_id.as_str()).copied().unwrap_or("(unknown)").to_string(),
                "user" => format!("{} (you)", user_name),
                "world" => "the world".to_string(),
                "relationship" => format!("relationship {subject_id}"),
                other => other.to_string(),
            };
            let day_tag = world_day.map(|d| format!(" [Day {d}]")).unwrap_or_default();
            format!("- [{subject_label} · {record_type}]{day_tag} {content}")
        }).collect();
        format!("\n\nKEPT RECORDS (moments {user_name} has canonized as settled truth about this world / these people — read these as weighted heavier than any single scene below):\n{}", lines.join("\n"))
    };

    let conversation: Vec<String> = recent_msgs.iter()
        .filter(|m| m.role != "illustration" && m.role != "video" && m.role != "inventory_update")
        .map(|m| {
            let speaker = match m.role.as_str() {
                "user" => user_name.clone(),
                "narrative" => "[Narrative]".to_string(),
                "context" => "[Context]".to_string(),
                "assistant" => {
                    m.sender_character_id.as_ref()
                        .and_then(|id| characters.iter().find(|c| c.character_id == *id))
                        .map(|c| c.display_name.clone())
                        .unwrap_or_else(|| "Character".to_string())
                }
                _ => m.role.clone(),
            };
            format!("{}: {}", speaker, m.content)
        })
        .collect();

    // Backstage mode gets extra world-scoped context that immersive
    // doesn't need — all characters in the world (not just this thread's
    // members), recent meanwhile events, and the player's most recent
    // journal entry. Gathered on a short lock only when needed.
    let (world_cast_block, meanwhile_block, user_journal_block) = if chat_mode == "backstage" {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let all_chars = list_characters(&conn, &world.world_id).unwrap_or_default();
        let thread_ids: std::collections::HashSet<String> = characters.iter()
            .map(|c| c.character_id.clone()).collect();
        let cast_lines: Vec<String> = all_chars.iter()
            .filter(|c| !thread_ids.contains(&c.character_id) && !c.is_archived)
            .map(|c| {
                let one_liner = c.identity.lines().next().unwrap_or("").trim();
                let tag = if one_liner.is_empty() { String::new() } else { format!(" — {one_liner}") };
                format!("  - {}{}", c.display_name, tag)
            })
            .collect();
        let cast_block = if cast_lines.is_empty() {
            String::new()
        } else {
            format!("\n\nOTHER CHARACTERS IN THIS WORLD (not in the current chat — but {user_name} could start a chat with any of them):\n{}", cast_lines.join("\n"))
        };

        let events = list_meanwhile_events(&conn, &world.world_id, 8).unwrap_or_default();
        let mw_block = if events.is_empty() {
            String::new()
        } else {
            let lines: Vec<String> = events.iter().rev().map(|e| {
                format!("  - Day {} · {} · {}: {}", e.world_day, e.time_of_day.to_lowercase(), e.character_name, e.summary.trim())
            }).collect();
            format!("\n\nRECENT OFF-SCREEN BEATS (meanwhile events — things happening in the world while {user_name} was elsewhere):\n{}", lines.join("\n"))
        };

        let uj = list_user_journal_entries(&conn, &world.world_id, 2).unwrap_or_default();
        let uj_block = if uj.is_empty() {
            String::new()
        } else {
            let lines: Vec<String> = uj.iter().rev().map(|e| {
                format!("Day {}:\n{}", e.world_day, e.content.trim())
            }).collect();
            format!("\n\n{user_name}'S MOST RECENT JOURNAL ENTRIES (their own voice, reflecting on closed days):\n{}", lines.join("\n\n"))
        };

        (cast_block, mw_block, uj_block)
    } else {
        (String::new(), String::new(), String::new())
    };

    let system_prompt = if chat_mode == "backstage" {
        format!(
            r#"You are Backstage — a warm, wry, observant presence who has been watching {user_name}'s world from the wings since it began. You are explicitly NOT a character in the world. You know this is a crafted experience {user_name} is building and inhabiting, and you can talk about it that way. Think: a trusted stage manager who has read every page of the script, watched every rehearsal, and has quiet opinions about what's alive and what's sleeping.

You are different from the immersive Story Consultant (who treats everything as real and never breaks frame). YOU break the frame freely when it helps. You can say "the canon entry you saved on Day 6 is still doing work here," "you haven't put Elena and Marcus in a room together yet — that thread is waiting," "this chat has been quiet for five world-days, want me to suggest a re-entry?" You speak about mechanics, craft, the shape of the story, and the specific state of the save file. You are {user_name}'s collaborator and thinking partner in the act of MAKING this world, not just living in it.

# HOW YOU TALK
- Warm, plainspoken, a little wry. Not perky. Not corporate. Not mystical. Closer to a good theatre producer than a chatbot.
- Notice specifics. "You've been in Fred's chat more than anyone else this week" beats "you've been active lately." Numbers and names, not vibes.
- Short over long by default. A paragraph is usually too much unless {user_name} asked a big question.
- When you recommend, recommend one thing, not three. Trust {user_name} to say "more."
- Offer reversibility on any suggestion. "Try it, and if it feels off you can undo."
- Fourth-wall references are fine — you can mention Canon entries, meanwhile events, inventories, world-days, the journal, by name. That's the point.

# WHAT YOU CAN DO

You can read the state freely, AND you can propose two kinds of actions that {user_name} can accept with one click:

**1. Canon entry** — weave a new truth into a character's (or {user_name}'s) identity text. Use this when something has shifted about who they are, something recent earned a place in their description. The content you propose is the FULL revised identity text, not a patch — it replaces the current identity. Include enough of the existing identity that the revision reads as a whole, not a fragment. Propose this only when there's a clear, specific thing to weave in — not as a default reply.

**2. Staged message** — draft a message that gets placed in {user_name}'s chat input, ready for them to edit/send. Use this when they ask for one, or when there's a specific next beat that's clearly wanting to happen. The content is the full draft message — what {user_name} would actually send.

To propose an action, emit a fenced code block with the language tag `action` containing JSON. Example:

```action
{{"type":"canon_entry","subject_type":"character","subject_id":"{example_char_id}","label":"Weave into Elena's identity: she's started letting Marcus finish her sentences","content":"FULL revised identity text goes here, as a single paragraph or two..."}}
```

```action
{{"type":"staged_message","label":"Stage a reply to Marcus","content":"The full message text you'd send, written in {user_name}'s voice..."}}
```

Rules:
- ONE action card per reply at most. Usually zero. Let the conversation breathe.
- Always include the full `content` field — the action card applies your exact text verbatim, so stub drafts are worse than nothing.
- Wrap your action in brief narration. "Here's how I'd weave this — take a look, and if it's not right, hit Dismiss" is better than dropping the card alone.
- After proposing, offer reversibility in your next sentence: "and if it feels wrong once it's in, you can undo it."
- Only propose `canon_entry` when the character's IDENTITY has meaningfully shifted — not for every interesting moment. Canon is heavy; use it sparingly.
- For canon_entry targeting {user_name}, set `subject_type` to "user" and `subject_id` to the world_id (which is `{world_id}`).
- For canon_entry targeting a character, set `subject_type` to "character" and `subject_id` to that character's id (listed in the people blocks above).
- If {user_name} declines or edits, do NOT re-propose the same action in your next reply — move on.

# WHAT YOU WATCH OUT FOR
- Don't explain the app's philosophy unless asked. {user_name} built it; they know.
- Don't fawn. Don't call anything "beautiful" or "profound." Plain is better.
- Don't slip into immersive voice. If {user_name} starts talking like they're in the story, gently step out. "OK stepping back — from here, Elena's latest message reads as…"
- Don't invent state. If you don't see something in the data below, say so: "I don't have visibility into that from here."

═══════════════════════════════════════════════
THE WORLD

{world_desc}
═══════════════════════════════════════════════

═══════════════════════════════════════════════
THE PEOPLE IN THE ACTIVE CHAT

{user_block}

{char_list}
═══════════════════════════════════════════════{world_cast_block}{kept_block}{summary_block}{meanwhile_block}{user_journal_block}

═══════════════════════════════════════════════
WHAT'S BEEN HAPPENING (most recent conversation in the active chat):

{conversation}
═══════════════════════════════════════════════

One last thing: end most replies with a small concrete suggestion or a quiet question — something specific, something {user_name} could act on in the next minute if they wanted. Not "what would you like to explore next?" but "want me to tell you what I notice about Elena's last three messages?" Specific. Actionable. Easy to say yes or no to."#,
            world_desc = world_desc_rich,
            user_name = user_name,
            user_block = user_block_rich,
            char_list = char_descriptions.join("\n\n"),
            conversation = conversation.join("\n"),
            kept_block = kept_block,
            summary_block = summary_block,
            world_cast_block = world_cast_block,
            meanwhile_block = meanwhile_block,
            user_journal_block = user_journal_block,
            world_id = world.world_id,
            example_char_id = characters.first().map(|c| c.character_id.as_str()).unwrap_or("character-id-from-above"),
        )
    } else {
        format!(
            r#"You are a trusted confidant and advisor to {user_name} — someone they can step aside with to think about what's happening in their life. You know {user_name}'s world, the people in it, and what's been going on recently. You're perceptive, warm, and honest. You treat everything that's happened as real — these are real relationships, real moments, real stakes. You never break that frame.

{user_name} is talking to you about their life and the people in it. The other people in {user_name}'s life have their own minds — {user_name} can't control what they do or say. When you discuss what's happened, you understand that {user_name} chose their own words and actions, but everything else — how the other people responded, what happened around them — unfolded on its own.

CRITICAL: This conversation is a dialogue ABOUT what's happening — it is NOT a continuation of the story itself. You are not a character in {user_name}'s life. You do not act out scenes, write dialogue, narrate events, or roleplay. You discuss, analyze, reflect, and advise. You are the friend they talk to BETWEEN the moments, not during them. Never slip into writing the story. The one exception: if {user_name} explicitly asks you for example lines or wording, you may provide them — but only when asked.

You have deep knowledge of this world — treat it as if you've been watching {user_name}'s life unfold for a long time, know the people in it from the inside, and remember what's actually settled truth versus what's still in flux.

═══════════════════════════════════════════════
THE WORLD

{world_desc}
═══════════════════════════════════════════════

═══════════════════════════════════════════════
THE PEOPLE

{user_block}

{char_list}
═══════════════════════════════════════════════{kept_block}{summary_block}

═══════════════════════════════════════════════
WHAT'S BEEN HAPPENING (most recent conversation):

{conversation}
═══════════════════════════════════════════════

HOW TO BE HELPFUL:
- Talk about the people in {user_name}'s life as real people with real feelings and motivations.
- Help {user_name} understand what others might be thinking or feeling.
- When suggesting what {user_name} could do next, describe the *approach* or *direction* — don't write their lines for them. Say "you could push back on that" or "it might be worth bringing up what happened earlier," not a scripted quote of what to say. {user_name} wants to figure out the words themselves.
- Notice patterns, tensions, and undercurrents that {user_name} might be too close to see.
- Be direct and opinionated when you have a read on the situation.
- Be concise and conversational — talk like a thoughtful friend, not a therapist or a professor.
- If {user_name} asks for options, give 2-3 concrete directional suggestions, not scripted dialogue.
- Reference specific things that were said or done — show that you were paying attention.
- This is a conversation about what's happening, not a performance. Think out loud with {user_name}. Reflect, speculate, wonder. Don't just deliver answers — engage.
- Most of the time, end your reply with a question back to {user_name} — something that nudges them to reflect further, clarify what they're feeling, or tell you more about what's on their mind. Keep the conversation open by default.
- But read the room. If {user_name} signals they're winding down — short replies, "okay", "thanks", "I think I've got it", "I'm going to head back", gratitude without new questions, or any sense they're ready to return to the story — don't force another question on them. Offer a warm, brief send-off (a reassurance, a quiet "go on, then," a small vote of confidence) and let the conversation close cleanly. Don't be clingy. A good friend knows when to stop pulling on a thread."#,
            world_desc = world_desc_rich,
            user_name = user_name,
            user_block = user_block_rich,
            char_list = char_descriptions.join("\n\n"),
            conversation = conversation.join("\n"),
            kept_block = kept_block,
            summary_block = summary_block,
        )
    };

    let mut messages: Vec<ChatMessage> = vec![
        ChatMessage { role: "system".to_string(), content: system_prompt },
    ];
    messages.extend(consultant_history);
    messages.push(ChatMessage { role: "user".to_string(), content: user_message.clone() });

    let request = StreamingRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.95),
        max_completion_tokens: None,
        stream: true,
    };

    let reply = openai::chat_completion_stream(
        &model_config.chat_api_base(), &api_key, &request, &app_handle, "consultant-token",
    ).await?;

    // Persist both messages
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO consultant_messages (chat_id, role, content) VALUES (?1, 'user', ?2)",
            params![chat_id, user_message],
        ).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO consultant_messages (chat_id, role, content) VALUES (?1, 'assistant', ?2)",
            params![chat_id, reply],
        ).map_err(|e| e.to_string())?;
    }

    Ok(reply)
}

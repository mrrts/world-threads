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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsultantMessage {
    pub role: String,
    pub content: String,
}

// ─── Chat CRUD ─────────────────────────────────────────────────────────────

/// Create a new consultant chat session for a thread.
#[tauri::command]
pub fn create_consultant_chat_cmd(
    db: State<'_, Database>,
    thread_id: String,
    title: Option<String>,
) -> Result<ConsultantChat, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    // Capture the latest message ID from the thread
    let last_msg_id: Option<String> = conn.query_row(
        "SELECT message_id FROM messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT 1",
        params![thread_id], |r| r.get(0),
    ).ok().or_else(|| conn.query_row(
        "SELECT message_id FROM group_messages WHERE thread_id = ?1 ORDER BY created_at DESC LIMIT 1",
        params![thread_id], |r| r.get(0),
    ).ok());
    let chat = ConsultantChat {
        chat_id: uuid::Uuid::new_v4().to_string(),
        thread_id,
        title: title.unwrap_or_else(|| "New Chat".to_string()),
        created_at: Utc::now().to_rfc3339(),
        last_seen_message_id: last_msg_id.clone(),
    };
    conn.execute(
        "INSERT INTO consultant_chats (chat_id, thread_id, title, created_at, last_seen_message_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![chat.chat_id, chat.thread_id, chat.title, chat.created_at, last_msg_id],
    ).map_err(|e| e.to_string())?;
    Ok(chat)
}

/// List all consultant chats for a thread, most recent first.
#[tauri::command]
pub fn list_consultant_chats_cmd(
    db: State<'_, Database>,
    thread_id: String,
) -> Result<Vec<ConsultantChat>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT chat_id, thread_id, title, created_at, last_seen_message_id FROM consultant_chats WHERE thread_id = ?1 ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![thread_id], |row| {
        Ok(ConsultantChat {
            chat_id: row.get(0)?,
            thread_id: row.get(1)?,
            title: row.get(2)?,
            created_at: row.get(3)?,
            last_seen_message_id: row.get(4).ok(),
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

    let (new_msgs, characters, user_name, thread_id) = if is_group {
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
                None => all_msgs,
            }
        } else {
            all_msgs
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
                None => all_msgs,
            }
        } else {
            all_msgs
        };
        (msgs, vec![character], user_name, thread.thread_id)
    };

    if new_msgs.is_empty() {
        return Err("No new messages since last import.".to_string());
    }

    // Format messages
    let conversation: Vec<String> = new_msgs.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
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

    let speaker_name = match m.role.as_str() {
        "user" => {
            let world_id: Option<String> = conn.query_row(
                "SELECT world_id FROM threads WHERE thread_id = ?1",
                params![m.thread_id], |r| r.get(0),
            ).ok();
            world_id.and_then(|wid| get_user_profile(&conn, &wid).ok().map(|p| p.display_name))
                .unwrap_or_else(|| "You".to_string())
        }
        "assistant" => {
            m.sender_character_id.as_ref()
                .and_then(|id| get_character(&conn, id).ok())
                .map(|c| c.display_name)
                .unwrap_or_else(|| "Character".to_string())
        }
        "narrative" => "Narrative".to_string(),
        "context" => "Context".to_string(),
        _ => m.role.clone(),
    };

    Ok(Some(LastSeenPreview {
        message_id: m.message_id,
        role: m.role,
        content: m.content,
        speaker_name,
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

    let (world, characters, recent_msgs, user_name, model_config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);

        if is_group {
            let gc = get_group_chat(&conn, group_chat_id.as_deref().unwrap()).map_err(|e| e.to_string())?;
            let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
            let recent_msgs = list_group_messages(&conn, &gc.thread_id, 30).map_err(|e| e.to_string())?;
            let user_name = get_user_profile(&conn, &gc.world_id)
                .ok().map(|p| p.display_name).unwrap_or_else(|| "the user".to_string());
            let char_ids: Vec<String> = gc.character_ids.as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let characters: Vec<Character> = char_ids.iter()
                .filter_map(|id| get_character(&conn, id).ok())
                .collect();
            (world, characters, recent_msgs, user_name, model_config)
        } else {
            let char_id = character_id.as_deref().ok_or("No character specified")?;
            let character = get_character(&conn, char_id).map_err(|e| e.to_string())?;
            let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
            let thread = get_thread_for_character(&conn, char_id).map_err(|e| e.to_string())?;
            let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;
            let user_name = get_user_profile(&conn, &character.world_id)
                .ok().map(|p| p.display_name).unwrap_or_else(|| "the user".to_string());
            (world, vec![character], recent_msgs, user_name, model_config)
        }
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

    let char_descriptions: Vec<String> = characters.iter().map(|c| {
        let mut desc = format!("- {}", c.display_name);
        if !c.identity.is_empty() {
            desc.push_str(&format!(": {}", c.identity));
        }
        desc
    }).collect();

    let conversation: Vec<String> = recent_msgs.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
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

    let system_prompt = format!(
        r#"You are a trusted confidant and advisor to {user_name} — someone they can step aside with to think about what's happening in their life. You know {user_name}'s world, the people in it, and what's been going on recently. You're perceptive, warm, and honest. You treat everything that's happened as real — these are real relationships, real moments, real stakes. You never break that frame.

{user_name} is talking to you about their life and the people in it. The other people in {user_name}'s life have their own minds — {user_name} can't control what they do or say. When you discuss what's happened, you understand that {user_name} chose their own words and actions, but everything else — how the other people responded, what happened around them — unfolded on its own.

CRITICAL: This conversation is a dialogue ABOUT what's happening — it is NOT a continuation of the story itself. You are not a character in {user_name}'s life. You do not act out scenes, write dialogue, narrate events, or roleplay. You discuss, analyze, reflect, and advise. You are the friend they talk to BETWEEN the moments, not during them. Never slip into writing the story. The one exception: if {user_name} explicitly asks you for example lines or wording, you may provide them — but only when asked.

You have full knowledge of:

WORLD: {world_desc}

PEOPLE:
- {user_name} (the person you're talking to)
{char_list}

WHAT'S BEEN HAPPENING:
{conversation}

HOW TO BE HELPFUL:
- Talk about the people in {user_name}'s life as real people with real feelings and motivations.
- Help {user_name} understand what others might be thinking or feeling.
- When suggesting what {user_name} could do next, describe the *approach* or *direction* — don't write their lines for them. Say "you could push back on that" or "it might be worth bringing up what happened earlier," not a scripted quote of what to say. {user_name} wants to figure out the words themselves.
- Notice patterns, tensions, and undercurrents that {user_name} might be too close to see.
- Be direct and opinionated when you have a read on the situation.
- Be concise and conversational — talk like a thoughtful friend, not a therapist or a professor.
- If {user_name} asks for options, give 2-3 concrete directional suggestions, not scripted dialogue.
- Reference specific things that were said or done — show that you were paying attention.
- This is a conversation about what's happening, not a performance. Think out loud with {user_name}. Reflect, speculate, wonder. Don't just deliver answers — engage."#,
        world_desc = if world.description.is_empty() { "A richly detailed world." } else { &world.description },
        user_name = user_name,
        char_list = char_descriptions.join("\n"),
        conversation = conversation.join("\n"),
    );

    let mut messages: Vec<ChatMessage> = vec![
        ChatMessage { role: "system".to_string(), content: system_prompt },
    ];
    messages.extend(consultant_history);
    messages.push(ChatMessage { role: "user".to_string(), content: user_message.clone() });

    let request = StreamingRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.95),
        max_completion_tokens: Some(800),
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

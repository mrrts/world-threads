use crate::ai::orchestrator;
use crate::ai::prompts::{self, GroupContext, OtherCharacter};
use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendGroupMessageResult {
    pub user_message: Message,
    pub character_responses: Vec<Message>,
}

#[tauri::command]
pub fn create_group_chat_cmd(
    db: State<Database>,
    world_id: String,
    character_ids: Vec<String>,
) -> Result<GroupChat, String> {
    if character_ids.len() < 2 || character_ids.len() > 3 {
        return Err("Group chats require 2-3 characters".to_string());
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if a group with these exact members already exists
    if let Some(existing) = find_group_chat_by_members(&conn, &world_id, &character_ids) {
        return Ok(existing);
    }

    // Sort character IDs for canonical storage
    let mut sorted_ids = character_ids.clone();
    sorted_ids.sort();

    // Build display name from character names
    let names: Vec<String> = sorted_ids.iter().filter_map(|id| {
        get_character(&conn, id).ok().map(|c| c.display_name)
    }).collect();
    let display_name = match names.len() {
        2 => format!("{} and {}", names[0], names[1]),
        3 => format!("{}, {}, and {}", names[0], names[1], names[2]),
        _ => names.join(", "),
    };

    let gc = GroupChat {
        group_chat_id: uuid::Uuid::new_v4().to_string(),
        world_id: world_id.clone(),
        character_ids: serde_json::json!(sorted_ids),
        thread_id: uuid::Uuid::new_v4().to_string(),
        display_name,
        created_at: Utc::now().to_rfc3339(),
    };

    create_group_chat(&conn, &gc).map_err(|e| e.to_string())?;
    Ok(gc)
}

#[tauri::command]
pub fn list_group_chats_cmd(
    db: State<Database>,
    world_id: String,
) -> Result<Vec<GroupChat>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_group_chats(&conn, &world_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_group_chat_cmd(
    db: State<Database>,
    group_chat_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    delete_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_group_messages_cmd(
    db: State<Database>,
    group_chat_id: String,
) -> Result<chat_cmds::PaginatedMessages, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
    let total = count_messages(&conn, &gc.thread_id).map_err(|e| e.to_string())?;
    let messages = get_all_messages(&conn, &gc.thread_id).map_err(|e| e.to_string())?;
    Ok(chat_cmds::PaginatedMessages { messages, total })
}

#[tauri::command]
pub fn save_group_user_message_cmd(
    db: State<Database>,
    group_chat_id: String,
    content: String,
) -> Result<Message, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;

    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: gc.thread_id.clone(),
        role: "user".to_string(),
        content,
        tokens_estimate: 0,
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
    };
    create_message(&conn, &msg).map_err(|e| e.to_string())?;
    Ok(msg)
}

use crate::commands::chat_cmds;
use crate::ai::openai;

/// Send a message in a group chat. The user's message is saved, then each character
/// responds in order. Returns the user message and all character responses.
#[tauri::command]
pub async fn send_group_message_cmd(
    db: State<'_, Database>,
    api_key: String,
    group_chat_id: String,
    content: String,
) -> Result<SendGroupMessageResult, String> {
    // Phase 1: Save user message and load context
    let (gc, world, characters, model_config, user_profile, user_msg) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let user_profile = get_user_profile(&conn, &gc.world_id).ok();

        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();

        // Save user message
        let user_msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: gc.thread_id.clone(),
            role: "user".to_string(),
            content: content.clone(),
            tokens_estimate: (content.len() as i64) / 4,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
        };
        create_message(&conn, &user_msg).map_err(|e| e.to_string())?;

        (gc, world, characters, model_config, user_profile, user_msg)
    };

    // Build character name map for message formatting
    let character_names: HashMap<String, String> = characters.iter()
        .map(|c| (c.character_id.clone(), c.display_name.clone()))
        .collect();

    // Phase 2: Each character responds in order
    let mut responses: Vec<Message> = Vec::new();

    for (i, character) in characters.iter().enumerate() {
        // Build group context (other characters, excluding the one responding)
        let other_chars: Vec<OtherCharacter> = characters.iter()
            .filter(|c| c.character_id != character.character_id)
            .map(|c| OtherCharacter {
                character_id: c.character_id.clone(),
                display_name: c.display_name.clone(),
                identity_summary: c.identity.clone(),
            })
            .collect();
        let group_context = GroupContext { other_characters: other_chars };

        // Load response_length setting for this character
        let response_length = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_setting(&conn, &format!("response_length.{}", character.character_id))
                .ok().flatten()
        };

        // Re-fetch recent messages (includes previous characters' responses)
        let recent_msgs = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_messages(&conn, &gc.thread_id, 30).map_err(|e| e.to_string())?
        };

        // Get thread summary for retrieval context
        let mut retrieved: Vec<String> = Vec::new();
        {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let summary = get_thread_summary(&conn, &gc.thread_id);
            if !summary.is_empty() {
                retrieved.push(format!("[Thread summary] {summary}"));
            }
        }

        // Generate response
        let (reply_text, usage) = orchestrator::run_dialogue_with_base(
            &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
            &world, character, &recent_msgs, &retrieved,
            user_profile.as_ref(),
            None, // no mood directive for group chats (keep it simpler)
            response_length.as_deref(),
            Some(&group_context),
            Some(&character_names),
        ).await?;

        if let Some(u) = &usage {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = record_token_usage(&conn, "group_dialogue", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
        }

        // Save response
        let tokens = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
        let response_msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: gc.thread_id.clone(),
            role: "assistant".to_string(),
            content: reply_text,
            tokens_estimate: tokens as i64,
            sender_character_id: Some(character.character_id.clone()),
            created_at: Utc::now().to_rfc3339(),
        };
        {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            create_message(&conn, &response_msg).map_err(|e| e.to_string())?;
        }

        responses.push(response_msg);
    }

    Ok(SendGroupMessageResult {
        user_message: user_msg,
        character_responses: responses,
    })
}

/// Prompt a specific character to speak in a group chat (Talk to Me).
#[tauri::command]
pub async fn prompt_group_character_cmd(
    db: State<'_, Database>,
    api_key: String,
    group_chat_id: String,
    character_id: String,
) -> Result<Message, String> {
    let (gc, world, character, characters, model_config, user_profile) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let user_profile = get_user_profile(&conn, &gc.world_id).ok();

        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();

        (gc, world, character, characters, model_config, user_profile)
    };

    let character_names: HashMap<String, String> = characters.iter()
        .map(|c| (c.character_id.clone(), c.display_name.clone()))
        .collect();

    let other_chars: Vec<OtherCharacter> = characters.iter()
        .filter(|c| c.character_id != character_id)
        .map(|c| OtherCharacter {
            character_id: c.character_id.clone(),
            display_name: c.display_name.clone(),
            identity_summary: c.identity.clone(),
        })
        .collect();
    let group_context = GroupContext { other_characters: other_chars };

    let recent_msgs = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_messages(&conn, &gc.thread_id, 30).map_err(|e| e.to_string())?
    };

    let mut retrieved: Vec<String> = Vec::new();
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let summary = get_thread_summary(&conn, &gc.thread_id);
        if !summary.is_empty() {
            retrieved.push(format!("[Thread summary] {summary}"));
        }
    }

    // Add a nudge if the last message isn't from the user
    let mut dialogue_msgs = recent_msgs.clone();
    if dialogue_msgs.last().map(|m| m.role != "user").unwrap_or(true) {
        dialogue_msgs.push(Message {
            message_id: String::new(),
            thread_id: String::new(),
            role: "user".to_string(),
            content: "[Everyone looks at you expectantly, waiting for you to say something.]".to_string(),
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
        });
    }

    let response_length = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, &format!("response_length.{}", character_id))
            .ok().flatten()
    };

    let (reply_text, usage) = orchestrator::run_dialogue_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &dialogue_msgs, &retrieved,
        user_profile.as_ref(),
        None,
        response_length.as_deref(),
        Some(&group_context),
        Some(&character_names),
    ).await?;

    if let Some(u) = &usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "group_dialogue", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    let tokens = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: gc.thread_id.clone(),
        role: "assistant".to_string(),
        content: reply_text,
        tokens_estimate: tokens as i64,
        sender_character_id: Some(character_id),
        created_at: Utc::now().to_rfc3339(),
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    Ok(msg)
}

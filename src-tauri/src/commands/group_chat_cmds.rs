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
    if character_ids.len() != 2 {
        return Err("Group chats require exactly 2 characters".to_string());
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
pub fn clear_group_chat_history_cmd(
    db: State<Database>,
    audio_dir: State<crate::commands::audio_cmds::AudioDir>,
    group_chat_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;

    // Collect message IDs for audio cleanup before deletion
    let msg_ids: Vec<String> = conn.prepare("SELECT message_id FROM group_messages WHERE thread_id = ?1")
        .map_err(|e| e.to_string())?
        .query_map(params![gc.thread_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();

    conn.execute("DELETE FROM group_messages_fts WHERE thread_id = ?1", params![gc.thread_id]).ok();
    conn.execute("DELETE FROM group_messages WHERE thread_id = ?1", params![gc.thread_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM memory_artifacts WHERE subject_id = ?1", params![gc.thread_id]).ok();
    conn.execute("DELETE FROM message_count_tracker WHERE thread_id = ?1", params![gc.thread_id]).ok();

    for msg_id in &msg_ids {
        crate::commands::audio_cmds::delete_audio_for_message(&audio_dir.0, msg_id);
    }

    Ok(())
}

#[tauri::command]
pub fn get_group_messages_cmd(
    db: State<Database>,
    group_chat_id: String,
) -> Result<chat_cmds::PaginatedMessages, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
    let total = count_group_messages(&conn, &gc.thread_id).map_err(|e| e.to_string())?;
    let messages = get_all_group_messages(&conn, &gc.thread_id).map_err(|e| e.to_string())?;
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
    create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
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
        create_group_message(&conn, &user_msg).map_err(|e| e.to_string())?;

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

        // Load settings scoped to the group chat
        let (response_length, narration_tone) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let rl = get_setting(&conn, &format!("response_length.{}", gc.group_chat_id)).ok().flatten();
            let nt = get_setting(&conn, &format!("narration_tone.{}", gc.group_chat_id)).ok().flatten();
            (rl, nt)
        };

        // Re-fetch recent messages (includes previous characters' responses)
        let recent_msgs = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_group_messages(&conn, &gc.thread_id, 30).map_err(|e| e.to_string())?
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
        let (raw_reply, usage) = orchestrator::run_dialogue_with_base(
            &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
            &world, character, &recent_msgs, &retrieved,
            user_profile.as_ref(),
            None, // no mood directive for group chats (keep it simpler)
            response_length.as_deref(),
            Some(&group_context),
            Some(&character_names),
            narration_tone.as_deref(),
        ).await?;

        // Strip own prefix and truncate any other-character dialogue
        let other_names: Vec<&str> = characters.iter()
            .filter(|c| c.character_id != character.character_id)
            .map(|c| c.display_name.as_str()).collect();
        let reply_text = strip_character_prefix(&raw_reply, &character.display_name, &other_names);

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
            create_group_message(&conn, &response_msg).map_err(|e| e.to_string())?;
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
        list_group_messages(&conn, &gc.thread_id, 30).map_err(|e| e.to_string())?
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

    let (response_length, narration_tone) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let rl = get_setting(&conn, &format!("response_length.{}", gc.group_chat_id)).ok().flatten();
        let nt = get_setting(&conn, &format!("narration_tone.{}", gc.group_chat_id)).ok().flatten();
        (rl, nt)
    };

    let (raw_reply, usage) = orchestrator::run_dialogue_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &dialogue_msgs, &retrieved,
        user_profile.as_ref(),
        None,
        response_length.as_deref(),
        Some(&group_context),
        Some(&character_names),
        narration_tone.as_deref(),
    ).await?;

    let other_names: Vec<&str> = characters.iter()
        .filter(|c| c.character_id != character.character_id)
        .map(|c| c.display_name.as_str()).collect();
    let reply_text = strip_character_prefix(&raw_reply, &character.display_name, &other_names);

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
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    Ok(msg)
}

/// Generate an illustration for a group chat. Sends all character portraits + user avatar as references.
#[tauri::command]
pub async fn generate_group_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    group_chat_id: String,
    quality_tier: Option<String>,
    custom_instructions: Option<String>,
    previous_illustration_id: Option<String>,
    include_scene_summary: Option<bool>,
) -> Result<chat_cmds::IllustrationResult, String> {
    let (world, characters, gc, recent_msgs, model_config, user_profile) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_group_messages(&conn, &gc.thread_id, 30).map_err(|e| e.to_string())?;
        let user_profile = get_user_profile(&conn, &gc.world_id).ok();

        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();

        (world, characters, gc, recent_msgs, model_config, user_profile)
    };

    let dir = &portraits_dir.0;
    let mut reference_images: Vec<Vec<u8>> = Vec::new();

    // User avatar first
    if let Some(ref profile) = user_profile {
        if !profile.avatar_file.is_empty() {
            let path = dir.join(&profile.avatar_file);
            if path.exists() {
                if let Ok(bytes) = std::fs::read(&path) {
                    reference_images.push(bytes);
                }
            }
        }
    }

    // All character portraits
    for character in &characters {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Some(portrait) = get_active_portrait(&conn, &character.character_id) {
            let path = dir.join(&portrait.file_name);
            if path.exists() {
                if let Ok(bytes) = std::fs::read(&path) {
                    reference_images.push(bytes);
                }
            }
        }
    }

    // Previous illustration
    let has_previous = if let Some(ref prev_id) = previous_illustration_id {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Ok(file_name) = conn.query_row(
            "SELECT file_name FROM world_images WHERE image_id = ?1",
            params![prev_id], |r| r.get::<_, String>(0),
        ) {
            let path = dir.join(&file_name);
            if path.exists() {
                if let Ok(bytes) = std::fs::read(&path) {
                    reference_images.push(bytes);
                    true
                } else { false }
            } else { false }
        } else { false }
    } else { false };

    let tier = quality_tier.as_deref().unwrap_or("high");
    let (img_size, img_quality) = match tier {
        "low" => ("1024x1024", "low"),
        "medium" => ("1024x1024", "medium"),
        _ => ("1536x1024", "medium"),
    };

    // Use first character for the orchestrator (it needs a Character struct)
    let primary_character = characters.first()
        .ok_or_else(|| "No characters in group chat".to_string())?;

    let (scene_description, image_bytes, chat_usage) = orchestrator::generate_illustration_with_base(
        &model_config.chat_api_base(),
        &model_config.openai_api_base(),
        &api_key,
        &model_config.dialogue_model,
        &model_config.image_model,
        img_quality,
        img_size,
        model_config.image_output_format().as_deref(),
        &world, primary_character, &recent_msgs,
        user_profile.as_ref(),
        &reference_images,
        custom_instructions.as_deref(),
        has_previous,
        include_scene_summary.unwrap_or(true),
        Some(&characters.iter().map(|c| c.display_name.clone()).collect::<Vec<_>>()),
    ).await?;

    if let Some(u) = &chat_usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "illustration", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    let aspect = chat_cmds::png_aspect_ratio(&image_bytes);
    let message_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("illustration_{message_id}.png");
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &image_bytes)
        .map_err(|e| format!("Failed to save illustration: {e}"))?;

    let b64 = chat_cmds::base64_encode_bytes(&image_bytes);
    let data_url = format!("data:image/png;base64,{b64}");
    let now = Utc::now().to_rfc3339();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let img = WorldImage {
            image_id: message_id.clone(),
            world_id: world.world_id.clone(),
            prompt: scene_description,
            file_name: file_name.clone(),
            is_active: false,
            source: "illustration".to_string(),
            created_at: now.clone(),
            aspect_ratio: aspect,
        };
        let _ = create_world_image(&conn, &img);

        let msg = Message {
            message_id: message_id.clone(),
            thread_id: gc.thread_id.clone(),
            role: "illustration".to_string(),
            content: data_url,
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: now,
        };
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let illustration_msg = conn.query_row(
        "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at FROM group_messages WHERE message_id = ?1",
        params![message_id], |row| Ok(Message {
            message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
            content: row.get(3)?, tokens_estimate: row.get(4)?,
            sender_character_id: row.get(5)?, created_at: row.get(6)?,
        })
    ).map_err(|e| e.to_string())?;

    Ok(chat_cmds::IllustrationResult {
        illustration_message: illustration_msg,
    })
}

/// Generate a narrative beat for a group chat.
#[tauri::command]
pub async fn generate_group_narrative_cmd(
    db: State<'_, Database>,
    api_key: String,
    group_chat_id: String,
) -> Result<chat_cmds::NarrativeResult, String> {
    let (world, characters, gc, recent_msgs, model_config, user_profile) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_group_messages(&conn, &gc.thread_id, 30).map_err(|e| e.to_string())?;
        let user_profile = get_user_profile(&conn, &gc.world_id).ok();

        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();

        (world, characters, gc, recent_msgs, model_config, user_profile)
    };

    let primary_character = characters.first()
        .ok_or_else(|| "No characters in group chat".to_string())?;

    // Load narration settings scoped to the group chat
    let (narration_tone, narration_instructions) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let tone = get_setting(&conn, &format!("narration_tone.{}", group_chat_id))
            .ok().flatten();
        let instructions = get_setting(&conn, &format!("narration_instructions.{}", group_chat_id))
            .ok().flatten();
        (tone, instructions)
    };

    let (narrative_text, usage) = orchestrator::run_narrative_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, primary_character, &recent_msgs, &[],
        user_profile.as_ref(),
        None,
        narration_tone.as_deref(),
        narration_instructions.as_deref(),
    ).await?;

    if let Some(u) = &usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "narrative", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    let narrative_msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: gc.thread_id.clone(),
        role: "narrative".to_string(),
        content: narrative_text,
        tokens_estimate: usage.as_ref().map(|u| u.total_tokens as i64).unwrap_or(0),
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_group_message(&conn, &narrative_msg).map_err(|e| e.to_string())?;
    }

    Ok(chat_cmds::NarrativeResult {
        narrative_message: narrative_msg,
    })
}

/// Strip any [CharacterName]: or CharacterName: prefix that the LLM may prepend to its response.
fn strip_character_prefix(text: &str, character_name: &str, other_names: &[&str]) -> String {
    let trimmed = text.trim();
    // Strip own name prefix
    let cleaned = if let Some(rest) = trimmed.strip_prefix(&format!("[{}]:", character_name)) {
        rest.trim()
    } else if let Some(rest) = trimmed.strip_prefix(&format!("[{}] :", character_name)) {
        rest.trim()
    } else if let Some(rest) = trimmed.strip_prefix(&format!("{}:", character_name)) {
        rest.trim()
    } else {
        trimmed
    };

    // Truncate at any point where another character's dialogue begins
    let mut result = cleaned.to_string();
    for name in other_names {
        for pattern in [format!("\n[{}]:", name), format!("\n[{}] :", name), format!("\n{}:", name)] {
            if let Some(pos) = result.find(&pattern) {
                result.truncate(pos);
            }
        }
    }
    result.trim().to_string()
}

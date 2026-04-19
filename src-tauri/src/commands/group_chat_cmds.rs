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

/// Return type for prompt_group_character_cmd. Mirrors the solo flow's
/// PromptCharacterResult — carries both the generated assistant message
/// and any reactions the character emitted this turn, so the frontend
/// can merge reactions into state without a separate round-trip.
#[derive(Debug, Serialize, Deserialize)]
pub struct PromptGroupCharacterResult {
    pub assistant_message: Message,
    pub ai_reactions: Vec<Reaction>,
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
    portraits_dir: State<PortraitsDir>,
    group_chat_id: String,
    keep_media: bool,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;

    // Collect deletable (non-illustration if keeping media) message IDs for audio cleanup.
    let deletable_sql = if keep_media {
        "SELECT message_id FROM group_messages WHERE thread_id = ?1 AND role != 'illustration'"
    } else {
        "SELECT message_id FROM group_messages WHERE thread_id = ?1"
    };
    let msg_ids: Vec<String> = conn.prepare(deletable_sql)
        .map_err(|e| e.to_string())?
        .query_map(params![gc.thread_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok()).collect();

    // Illustrations (only clean up when not keeping media)
    let mut illustration_files: Vec<String> = Vec::new();
    if !keep_media {
        let illus_ids: Vec<String> = conn.prepare(
            "SELECT message_id FROM group_messages WHERE thread_id = ?1 AND role = 'illustration'"
        ).map_err(|e| e.to_string())?
            .query_map(params![gc.thread_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok()).collect();
        for illus_id in &illus_ids {
            let file_name: Option<String> = conn.query_row(
                "SELECT file_name FROM world_images WHERE image_id = ?1",
                params![illus_id], |r| r.get(0),
            ).ok();
            conn.execute("DELETE FROM world_images WHERE image_id = ?1", params![illus_id]).ok();
            if let Some(f) = file_name {
                illustration_files.push(f);
            }
        }
    }

    // FTS — group_messages_fts is only populated for text messages, safe to blanket-delete.
    conn.execute("DELETE FROM group_messages_fts WHERE thread_id = ?1", params![gc.thread_id]).ok();

    if keep_media {
        conn.execute(
            "DELETE FROM group_messages WHERE thread_id = ?1 AND role != 'illustration'",
            params![gc.thread_id],
        ).map_err(|e| e.to_string())?;
    } else {
        conn.execute("DELETE FROM group_messages WHERE thread_id = ?1", params![gc.thread_id])
            .map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM novel_entries WHERE thread_id = ?1", params![gc.thread_id]).ok();
    }

    conn.execute("DELETE FROM memory_artifacts WHERE subject_id = ?1", params![gc.thread_id]).ok();
    conn.execute("DELETE FROM message_count_tracker WHERE thread_id = ?1", params![gc.thread_id]).ok();

    for msg_id in &msg_ids {
        crate::commands::audio_cmds::delete_audio_for_message(&audio_dir.0, msg_id);
    }
    for f in &illustration_files {
        let path = portraits_dir.0.join(f);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_group_messages_cmd(
    db: State<Database>,
    group_chat_id: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<chat_cmds::PaginatedMessages, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
    let total = count_group_messages(&conn, &gc.thread_id).map_err(|e| e.to_string())?;
    let messages = match limit {
        Some(lim) => list_group_messages_paginated(&conn, &gc.thread_id, lim, offset.unwrap_or(0))
            .map_err(|e| e.to_string())?,
        None => get_all_group_messages(&conn, &gc.thread_id).map_err(|e| e.to_string())?,
    };
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
    let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
    let (wd, wt) = chat_cmds::world_time_fields(&world);

    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: gc.thread_id.clone(),
        role: "user".to_string(),
        content,
        tokens_estimate: 0,
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
        world_day: wd, world_time: wt,
            address_to: None,
        mood_chain: None,
        };
    create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    Ok(msg)
}

use crate::commands::chat_cmds;

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
        let (wd, wt) = chat_cmds::world_time_fields(&world);
        let user_msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: gc.thread_id.clone(),
            role: "user".to_string(),
            content: content.clone(),
            tokens_estimate: (content.len() as i64) / 4,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
            world_day: wd, world_time: wt.clone(),
            address_to: None,
        mood_chain: None,
        };
        create_group_message(&conn, &user_msg).map_err(|e| e.to_string())?;

        (gc, world, characters, model_config, user_profile, user_msg)
    };

    let (wd, wt) = chat_cmds::world_time_fields(&world);

    // Build character name map for message formatting
    let character_names: HashMap<String, String> = characters.iter()
        .map(|c| (c.character_id.clone(), c.display_name.clone()))
        .collect();

    // Kick off the character-reaction emoji pick NOW, in parallel with the
    // entire character-response loop below. The pick only needs user
    // content + mood_reduction + recent-scene context, none of which
    // depend on replies. We await it at the end — saves N × reaction-
    // latency on group turns.
    let (reduction_snapshot, reaction_context) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let r = get_thread_mood_reduction(&conn, &gc.thread_id);
        // Last 4 messages before the user's brand-new one.
        let all = list_group_messages(&conn, &gc.thread_id, 5).unwrap_or_default();
        let ctx: Vec<Message> = all.into_iter()
            .filter(|m| m.message_id != user_msg.message_id)
            .collect();
        (r, ctx)
    };
    let reaction_base = model_config.chat_api_base();
    let reaction_model = model_config.dialogue_model.clone();
    let reaction_content = content.clone();
    let reaction_reduction = reduction_snapshot.clone();
    let reaction_api_key = api_key.clone();
    let reaction_ctx = reaction_context.clone();
    let reaction_handle = tokio::spawn(async move {
        orchestrator::pick_character_reaction_via_llm(
            &reaction_base, &reaction_api_key, &reaction_model,
            &reaction_content, &reaction_reduction, &reaction_ctx,
        ).await
    });

    // Phase 2: Each character responds in order
    let mut responses: Vec<Message> = Vec::new();

    for (_i, character) in characters.iter().enumerate() {
        // Build group context (other characters, excluding the one responding)
        let other_chars: Vec<OtherCharacter> = characters.iter()
            .filter(|c| c.character_id != character.character_id)
            .map(|c| OtherCharacter {
                character_id: c.character_id.clone(),
                display_name: c.display_name.clone(),
                identity_summary: c.identity.clone(),
                sex: c.sex.clone(),
                voice_rules: crate::ai::prompts::json_array_to_strings(&c.voice_rules),
            })
            .collect();
        let group_context = GroupContext { other_characters: other_chars };

        // Load settings scoped to the group chat
        let (response_length, narration_tone, leader) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let rl = get_setting(&conn, &format!("response_length.{}", gc.group_chat_id)).ok().flatten();
            let nt = get_setting(&conn, &format!("narration_tone.{}", gc.group_chat_id)).ok().flatten();
            let leader = get_setting(&conn, &format!("leader.{}", gc.group_chat_id)).ok().flatten();
            (rl, nt, leader)
        };

        // Re-fetch recent messages (includes previous characters' responses)
        let recent_msgs = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_group_messages_within_budget(&conn, &gc.thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?
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

        // Just-before-turn system hint: re-affirm whose voice this is, now
        // that the conversation history may have drifted a speaker or two.
        // Reinforces the "# THE TURN" section of the system prompt at the
        // moment it matters most — right before generation.
        let user_name = user_profile.as_ref()
            .map(|p| p.display_name.as_str())
            .unwrap_or("the human");
        let mut dialogue_msgs = recent_msgs.clone();
        dialogue_msgs.push(Message {
            message_id: String::new(),
            thread_id: String::new(),
            role: "user".to_string(),
            content: format!(
                "[It is now {name}'s turn to speak. Reply ONLY as {name}, addressing {user_name}. Do not prefix your reply with your name.]",
                name = character.display_name,
            ),
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
            world_day: None,
            world_time: None,
            address_to: None,
        mood_chain: None,
        });

        // Generate response — load mood_reduction + pick chain for AGENCY.
        let mood_reduction = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_thread_mood_reduction(&conn, &gc.thread_id)
        };
        let mood_chain = prompts::pick_mood_chain(Some(&mood_reduction));
        let mood_chain_json = serde_json::to_string(&mood_chain).ok();

        let (raw_reply, usage) = orchestrator::run_dialogue_with_base(
            &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
            &world, character, &dialogue_msgs, &retrieved,
            user_profile.as_ref(),
            None, // no mood directive for group chats (keep it simpler)
            response_length.as_deref(),
            Some(&group_context),
            Some(&character_names),
            narration_tone.as_deref(),
            model_config.is_local(),
            &mood_chain,
            leader.as_deref(),
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

        // Save response — in auto-respond chain triggered by a user message,
        // the character's reply is (by default) addressed to the user.
        let tokens = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
        let response_msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: gc.thread_id.clone(),
            role: "assistant".to_string(),
            content: reply_text,
            tokens_estimate: tokens as i64,
            sender_character_id: Some(character.character_id.clone()),
            created_at: Utc::now().to_rfc3339(),
            world_day: wd, world_time: wt.clone(),
            address_to: Some("user".to_string()),
            mood_chain: mood_chain_json.clone(),
        };
        {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            create_group_message(&conn, &response_msg).map_err(|e| e.to_string())?;
        }

        responses.push(response_msg);
    }

    // Await the parallel reaction pick (launched before the character loop).
    let reaction_emoji = match reaction_handle.await {
        Ok(Ok(e)) => e,
        _ => {
            let chain = prompts::pick_mood_chain(Some(&reduction_snapshot));
            prompts::pick_character_reaction_emoji(&chain)
        }
    };
    let _ = chat_cmds::emit_character_reaction(
        &db,
        &user_msg.message_id,
        &reaction_emoji,
    );

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
    address_to: Option<String>,
) -> Result<PromptGroupCharacterResult, String> {
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
            sex: c.sex.clone(),
            voice_rules: crate::ai::prompts::json_array_to_strings(&c.voice_rules),
        })
        .collect();
    let group_context = GroupContext { other_characters: other_chars };

    let recent_msgs = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_group_messages_within_budget(&conn, &gc.thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?
    };

    let mut retrieved: Vec<String> = Vec::new();
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let summary = get_thread_summary(&conn, &gc.thread_id);
        if !summary.is_empty() {
            retrieved.push(format!("[Thread summary] {summary}"));
        }
    }

    // Add a nudge directing who the character should address
    let mut dialogue_msgs = recent_msgs.clone();
    let user_name = user_profile.as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");
    let nudge = match address_to.as_deref() {
        Some(target) if !target.is_empty() => {
            format!("[Turn to {target} and say something to them directly. Address {target} specifically.]")
        }
        _ => {
            format!("[Turn to {user_name} and say something to them directly. Address {user_name} specifically.]")
        }
    };
    dialogue_msgs.push(Message {
        message_id: String::new(),
        thread_id: String::new(),
        role: "user".to_string(),
        content: nudge,
        tokens_estimate: 0,
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
            world_day: None, world_time: None,
            address_to: None,
        mood_chain: None,
        });

    let (response_length, narration_tone, leader) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let rl = get_setting(&conn, &format!("response_length.{}", gc.group_chat_id)).ok().flatten();
        let nt = get_setting(&conn, &format!("narration_tone.{}", gc.group_chat_id)).ok().flatten();
        let leader = get_setting(&conn, &format!("leader.{}", gc.group_chat_id)).ok().flatten();
        (rl, nt, leader)
    };

    let mood_reduction2 = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_thread_mood_reduction(&conn, &gc.thread_id)
    };
    let mood_chain2 = prompts::pick_mood_chain(Some(&mood_reduction2));
    let mood_chain_json2 = serde_json::to_string(&mood_chain2).ok();

    // Target + content for the per-character reaction emit. We reach for
    // the most recent USER message in this thread — the one this
    // character is felt-responding to. In auto-respond chains where this
    // character is the 2nd or 3rd to go, that message is still the
    // triggering user turn, not the intermediate assistant messages.
    let (reaction_target_id, reaction_user_content): (Option<String>, String) = recent_msgs.iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| (Some(m.message_id.clone()), m.content.clone()))
        .unwrap_or_else(|| (None, String::new()));
    let reaction_context: Vec<Message> = recent_msgs.iter()
        .rev().skip(1).take(4).rev().cloned().collect();

    let base = model_config.chat_api_base();
    let dialogue_fut = orchestrator::run_dialogue_with_base(
        &base, &api_key, &model_config.dialogue_model,
        &world, &character, &dialogue_msgs, &retrieved,
        user_profile.as_ref(),
        None,
        response_length.as_deref(),
        Some(&group_context),
        Some(&character_names),
        narration_tone.as_deref(),
        model_config.is_local(),
        &mood_chain2,
        leader.as_deref(),
    );
    let reaction_fut = orchestrator::pick_character_reaction_via_llm(
        &base, &api_key, &model_config.dialogue_model,
        &reaction_user_content, &mood_reduction2, &reaction_context,
    );
    let (dialogue_res, reaction_res) = tokio::join!(dialogue_fut, reaction_fut);
    let (raw_reply, usage) = dialogue_res?;

    let other_names: Vec<&str> = characters.iter()
        .filter(|c| c.character_id != character.character_id)
        .map(|c| c.display_name.as_str()).collect();
    let reply_text = strip_character_prefix(&raw_reply, &character.display_name, &other_names);

    if let Some(u) = &usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "group_dialogue", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    let tokens = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
    let (wd_p, wt_p) = chat_cmds::world_time_fields(&world);

    // Canonicalize the address_to param for storage: "user" by default,
    // otherwise resolve the display-name target to a character_id if it
    // matches another character in this group.
    let canonical_address: Option<String> = match address_to.as_deref() {
        None | Some("") => Some("user".to_string()),
        Some(name) => {
            if user_profile.as_ref().map(|p| p.display_name.eq_ignore_ascii_case(name)).unwrap_or(false) {
                Some("user".to_string())
            } else {
                characters.iter()
                    .find(|c| c.character_id != character_id && c.display_name.eq_ignore_ascii_case(name))
                    .map(|c| c.character_id.clone())
                    .or_else(|| Some("user".to_string()))
            }
        }
    };

    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: gc.thread_id.clone(),
        role: "assistant".to_string(),
        content: reply_text,
        tokens_estimate: tokens as i64,
        sender_character_id: Some(character_id),
        created_at: Utc::now().to_rfc3339(),
        world_day: wd_p, world_time: wt_p,
        address_to: canonical_address,
        mood_chain: mood_chain_json2.clone(),
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    // Emit the character's reaction on the triggering user message. Each
    // responding character gets their own pick (parallel above), so a
    // group turn accumulates multiple emoji reactions on the user's
    // message — one per responder. Different emojis render as distinct
    // bubbles; same emoji dedupes via the (msg, emoji, 'assistant') key.
    let ai_reactions: Vec<Reaction> = match reaction_target_id {
        Some(target_id) => {
            let reaction_emoji = reaction_res
                .unwrap_or_else(|_| prompts::pick_character_reaction_emoji(&mood_chain2));
            chat_cmds::emit_character_reaction(&db, &target_id, &reaction_emoji)
        }
        None => Vec::new(),
    };

    Ok(PromptGroupCharacterResult {
        assistant_message: msg,
        ai_reactions,
    })
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
        let recent_msgs = list_group_messages_within_budget(&conn, &gc.thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?;
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

    // Use first character as the "primary" for the orchestrator, and pass the
    // rest as additional_cast so the scene director knows the full cast.
    let primary_character = characters.first()
        .ok_or_else(|| "No characters in group chat".to_string())?;
    let additional_cast_vec: Vec<&Character> = characters.iter()
        .filter(|c| c.character_id != primary_character.character_id)
        .collect();
    let additional_cast_opt: Option<&[&Character]> = if additional_cast_vec.is_empty() { None } else { Some(&additional_cast_vec) };
    let names_map: std::collections::HashMap<String, String> = characters.iter()
        .map(|c| (c.character_id.clone(), c.display_name.clone()))
        .collect();

    let (scene_description, image_bytes, chat_usage) = orchestrator::generate_illustration_with_base(
        &model_config.chat_api_base(),
        &model_config.openai_api_base(),
        &api_key,
        &model_config.dialogue_model,
        &model_config.image_model,
        img_quality,
        img_size,
        model_config.image_output_format().as_deref(),
        &world, primary_character, additional_cast_opt, &recent_msgs,
        user_profile.as_ref(),
        &reference_images,
        custom_instructions.as_deref(),
        has_previous,
        include_scene_summary.unwrap_or(true),
        Some(&characters.iter().map(|c| c.display_name.clone()).collect::<Vec<_>>()),
        Some(&names_map),
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

        let (wd, wt) = chat_cmds::world_time_fields(&world);
        let msg = Message {
            message_id: message_id.clone(),
            thread_id: gc.thread_id.clone(),
            role: "illustration".to_string(),
            content: data_url,
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: now,
            world_day: wd, world_time: wt,
            address_to: None,
        mood_chain: None,
        };
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let illustration_msg = conn.query_row(
        "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM group_messages WHERE message_id = ?1",
        params![message_id], |row| Ok(Message {
            message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
            content: row.get(3)?, tokens_estimate: row.get(4)?,
            sender_character_id: row.get(5)?, created_at: row.get(6)?,
            world_day: row.get(7).ok(), world_time: row.get(8).ok(),
            address_to: None,
        mood_chain: None,
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
    custom_instructions: Option<String>,
) -> Result<chat_cmds::NarrativeResult, String> {
    let (world, characters, gc, recent_msgs, model_config, user_profile) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_group_messages_within_budget(&conn, &gc.thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?;
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

    let prev_is_narrative = recent_msgs.last().map(|m| m.role == "narrative").unwrap_or(false);
    let continuation_prefix = if prev_is_narrative {
        Some("IMPORTANT: The previous message in the conversation is also a narrative beat. Do NOT revise or repeat it. Write a CONTINUATION that advances to the NEXT story beat — new action, new moment, new tension. Pick up where the previous narrative left off and move the story forward.".to_string())
    } else {
        None
    };

    let all_instructions: Vec<&str> = [
        continuation_prefix.as_deref(),
        narration_instructions.as_deref().filter(|s| !s.is_empty()),
        custom_instructions.as_deref().filter(|s| !s.is_empty()),
    ].into_iter().flatten().collect();
    let merged_instructions = if all_instructions.is_empty() { None } else { Some(all_instructions.join("\n")) };

    let additional_cast: Vec<&Character> = characters.iter()
        .filter(|c| c.character_id != primary_character.character_id)
        .collect();
    let (narrative_text, usage) = orchestrator::run_narrative_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, primary_character,
        if additional_cast.is_empty() { None } else { Some(&additional_cast) },
        &recent_msgs, &[],
        user_profile.as_ref(),
        None,
        narration_tone.as_deref(),
        merged_instructions.as_deref(),
    ).await?;

    if let Some(u) = &usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "narrative", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    let (wd, wt) = chat_cmds::world_time_fields(&world);
    let narrative_msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: gc.thread_id.clone(),
        role: "narrative".to_string(),
        content: narrative_text,
        tokens_estimate: usage.as_ref().map(|u| u.total_tokens as i64).unwrap_or(0),
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
        world_day: wd, world_time: wt,
            address_to: None,
        mood_chain: None,
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

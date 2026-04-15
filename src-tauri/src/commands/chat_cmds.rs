use crate::ai::mood;
use crate::ai::openai;
use crate::ai::orchestrator;
use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

const MEMORY_MAINTENANCE_INTERVAL: i64 = 10;

/// Compute mood drift from recent messages and persist the updated mood.
/// Returns the mood style directive string, or None if mood is neutral or disabled.
pub fn compute_and_persist_mood(
    db: &Database,
    character_id: &str,
    world: &World,
    character: &Character,
    recent_msgs: &[Message],
    current_mood: Option<&CharacterMood>,
    mood_enabled: bool,
    mood_drift_rate: Option<f64>,
) -> Result<Option<String>, String> {
    if !mood_enabled { return Ok(None); }

    let current = current_mood
        .map(mood::MoodVector::from)
        .unwrap_or_else(mood::MoodVector::neutral);
    let target = mood::compute_mood_target(world, character, recent_msgs);
    let drifted = mood::drift_mood(&current, &target, mood_drift_rate);
    let directive = mood::mood_to_style_directive(&drifted);

    let history = current_mood
        .map(|m| m.history.clone())
        .unwrap_or_else(|| serde_json::Value::Array(vec![]));
    let new_history = mood::append_mood_history(&history, &drifted);

    let updated = CharacterMood {
        character_id: character_id.to_string(),
        valence: drifted.valence,
        energy: drifted.energy,
        tension: drifted.tension,
        history: new_history,
        updated_at: Utc::now().to_rfc3339(),
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = upsert_character_mood(&conn, &updated);
    }

    log::info!("[Mood] {} → v={:.2} e={:.2} t={:.2} | directive: {:.60}",
        character.display_name, drifted.valence, drifted.energy, drifted.tension,
        if directive.is_empty() { "(neutral)" } else { &directive });

    Ok(if directive.is_empty() { None } else { Some(directive) })
}

pub fn world_time_fields(world: &World) -> (Option<i64>, Option<String>) {
    let time = world.state.get("time");
    let day = time.and_then(|t| t.get("day_index")).and_then(|v| v.as_i64());
    let tod = time.and_then(|t| t.get("time_of_day")).and_then(|v| v.as_str()).map(|s| s.to_string());
    (day, tod)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResult {
    pub user_message: Message,
    pub assistant_message: Message,
    pub ai_reactions: Vec<Reaction>,
}

#[tauri::command]
pub fn save_user_message_cmd(
    db: State<'_, Database>,
    character_id: String,
    content: String,
) -> Result<Message, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
    let ch = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
    let w = get_world(&conn, &ch.world_id).map_err(|e| e.to_string())?;
    let (wd_s, wt_s) = world_time_fields(&w);

    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread.thread_id.clone(),
        role: "user".to_string(),
        content: content.clone(),
        tokens_estimate: (content.len() as i64) / 4,
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
        world_day: wd_s, world_time: wt_s,
    };
    create_message(&conn, &msg).map_err(|e| e.to_string())?;
    increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

    Ok(msg)
}

/// Create a cross-chat context message in an individual or group chat.
#[tauri::command]
pub fn create_context_message_cmd(
    db: State<Database>,
    character_id: Option<String>,
    group_chat_id: Option<String>,
    content: String,
) -> Result<Message, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (thread_id, world) = if let Some(gc_id) = &group_chat_id {
        let gc = get_group_chat(&conn, gc_id).map_err(|e| e.to_string())?;
        let w = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
        (gc.thread_id, w)
    } else if let Some(char_id) = &character_id {
        let thread = get_thread_for_character(&conn, char_id).map_err(|e| e.to_string())?;
        let ch = get_character(&conn, char_id).map_err(|e| e.to_string())?;
        let w = get_world(&conn, &ch.world_id).map_err(|e| e.to_string())?;
        (thread.thread_id, w)
    } else {
        return Err("Either character_id or group_chat_id must be provided".to_string());
    };

    let (wd, wt) = world_time_fields(&world);
    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id,
        role: "context".to_string(),
        content,
        tokens_estimate: 0,
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
        world_day: wd,
        world_time: wt,
    };

    if group_chat_id.is_some() {
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    } else {
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    Ok(msg)
}

#[tauri::command]
pub async fn send_message_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
    content: String,
) -> Result<SendMessageResult, String> {
    // Phase 1: Read everything from DB, persist user message, build retrieval context
    let (world, character, thread, recent_msgs, model_config,
         retrieved, should_run_maintenance, user_profile,
         current_mood, mood_enabled, mood_drift_rate, response_length, narration_tone) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);

        let (wd_u, wt_u) = world_time_fields(&world);
        let user_msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread.thread_id.clone(),
            role: "user".to_string(),
            content: content.clone(),
            tokens_estimate: (content.len() as i64) / 4,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
            world_day: wd_u, world_time: wt_u,
        };
        create_message(&conn, &user_msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

        let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;

        // Hybrid retrieval: FTS messages + thread summary
        let mut retrieved: Vec<String> = Vec::new();

        let summary = get_thread_summary(&conn, &thread.thread_id);
        if !summary.is_empty() {
            log::info!("[Memory] Thread summary found ({} chars)", summary.len());
            retrieved.push(format!("[Thread summary] {summary}"));
        } else {
            log::info!("[Memory] No thread summary yet");
        }

        // Sanitize content for FTS5 MATCH: strip operators, quote as a phrase
        let fts_query = sanitize_fts_query(&content);

        if !fts_query.is_empty() {
            match search_messages_fts(&conn, &thread.thread_id, &fts_query, 6) {
                Ok(fts_msgs) => {
                    let fts_msgs: Vec<_> = fts_msgs.into_iter()
                        .filter(|m| m.role != "illustration" && m.role != "video")
                        .collect();
                    log::info!("[Memory] FTS messages: {} results for {:?}", fts_msgs.len(), fts_query);
                    for m in fts_msgs {
                        retrieved.push(format!("[{}] {}: {}", m.created_at, m.role, m.content));
                    }
                }
                Err(e) => log::warn!("[Memory] FTS messages query failed: {e}"),
            }
        }

        let msg_count = count_messages_since_maintenance(&conn, &thread.thread_id);
        let should_run_maintenance = msg_count >= MEMORY_MAINTENANCE_INTERVAL;

        let user_profile = get_user_profile(&conn, &character.world_id).ok();

        let current_mood = get_character_mood(&conn, &character_id);
        let mood_enabled = get_setting(&conn, "mood_drift_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        let mood_drift_rate = get_setting(&conn, "mood_drift_rate")
            .ok().flatten().and_then(|v| v.parse::<f64>().ok());

        let response_length = get_setting(&conn, &format!("response_length.{}", character_id))
            .ok().flatten();
        let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
            .ok().flatten();

        (world, character, thread, recent_msgs, model_config,
         retrieved, should_run_maintenance, user_profile,
         current_mood, mood_enabled, mood_drift_rate, response_length, narration_tone)
    };

    let (wd, wt) = world_time_fields(&world);

    // Phase 2: Vector search (if embeddings exist) — requires OpenAI, skip for LM Studio
    let mut full_retrieved = retrieved;
    let is_local = model_config.ai_provider == "lmstudio";

    if !is_local {
        let has_chunks = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let count: i64 = conn.query_row(
                "SELECT count(*) FROM chunk_metadata WHERE world_id = ?1 AND character_id = ?2",
                params![world.world_id, character_id], |r| r.get(0),
            ).unwrap_or(0);
            log::info!("[Memory] Vector store: {} chunks for character {}", count, character.display_name);
            count > 0
        };

        if has_chunks {
            match orchestrator::generate_embeddings_with_base(
                &model_config.openai_api_base(), &api_key, &model_config.embedding_model, vec![content.clone()]
            ).await {
                Ok((embeddings, embed_tokens)) if !embeddings.is_empty() => {
                    {
                        let conn = db.conn.lock().map_err(|e| e.to_string())?;
                        let _ = record_token_usage(&conn, "embedding", &model_config.embedding_model, embed_tokens, 0);
                    }
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    match search_vectors(&conn, &world.world_id, &character_id, &embeddings[0], 4) {
                        Ok(results) => {
                            log::info!("[Memory] Vector search: {} results", results.len());
                            for (chunk_content, distance) in results {
                                log::info!("[Memory]   - dist={:.3}: {:.80}", distance, chunk_content);
                                full_retrieved.push(format!("[Memory] {chunk_content}"));
                            }
                        }
                        Err(e) => log::warn!("[Memory] Vector search failed: {e}"),
                    }
                }
                Ok((_, _)) => log::warn!("[Memory] Embedding returned empty"),
                Err(e) => log::warn!("[Memory] Query embedding failed: {e}"),
            }
        }
    } else {
        log::info!("[Memory] Skipping vector search (LM Studio mode)");
    }

    log::info!("[Memory] Total retrieval context: {} items passed to dialogue", full_retrieved.len());

    // Phase 3: Mood drift
    let mood_directive = compute_and_persist_mood(
        &db, &character_id, &world, &character, &recent_msgs,
        current_mood.as_ref(), mood_enabled, mood_drift_rate,
    )?;

    // Phase 4: Run dialogue
    let (reply_text, dialogue_usage) = orchestrator::run_dialogue_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &recent_msgs, &full_retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        response_length.as_deref(),
        None, None, narration_tone.as_deref(),
    ).await?;
    let tokens = dialogue_usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);

    // Phase 5: Store assistant message
    let (user_message, assistant_msg) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread.thread_id.clone(),
            role: "assistant".to_string(),
            content: reply_text,
            tokens_estimate: tokens as i64,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
            world_day: wd, world_time: wt.clone(),
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

        let user_message = recent_msgs.last().cloned().unwrap_or_else(|| Message {
            message_id: String::new(), thread_id: thread.thread_id.clone(),
            role: "user".to_string(), content: content.clone(),
            tokens_estimate: 0, created_at: Utc::now().to_rfc3339(),
            world_day: None, world_time: None,
            sender_character_id: None,
        });

        (user_message, msg)
    };

    // Phase 6: AI reaction to user's message (disabled — re-enable by uncommenting)
    let ai_reactions: Vec<Reaction> = Vec::new();
    // match orchestrator::generate_reaction_with_base(
    //     &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
    //     &character, &content, &assistant_msg.content,
    // ).await { ... }

    // Phase 7: Generate embeddings for new messages — requires OpenAI, skip for LM Studio
    if !is_local {
        let embed_texts = vec![
            format!("user: {}", content),
            format!("{}: {}", character.display_name, assistant_msg.content),
        ];
        let embed_ids = vec![user_message.message_id.clone(), assistant_msg.message_id.clone()];

        log::info!("[Memory] Generating embeddings for {} items", embed_texts.len());
        match orchestrator::generate_embeddings_with_base(&model_config.openai_api_base(), &api_key, &model_config.embedding_model, embed_texts.clone()).await {
            Ok((embeddings, embed_tokens)) => {
                log::info!("[Memory] Got {} embeddings, storing in vector DB", embeddings.len());
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                let _ = record_token_usage(&conn, "embedding", &model_config.embedding_model, embed_tokens, 0);
                for (i, emb) in embeddings.iter().enumerate() {
                    if i < embed_ids.len() {
                        match insert_vector_chunk(
                            &conn, &embed_ids[i], "message", &embed_ids[i],
                            &world.world_id, &character_id, &embed_texts[i], emb,
                        ) {
                            Ok(_) => {}
                            Err(e) => log::warn!("[Memory] Failed to store chunk {}: {e}", embed_ids[i]),
                        }
                    }
                }
            }
            Err(e) => log::warn!("[Memory] Embedding generation failed: {e}"),
        }
    } else {
        log::info!("[Memory] Skipping embedding generation (LM Studio mode)");
    }

    // Phase 8: Memory maintenance (every N messages)
    log::info!("[Memory] Maintenance check: should_run={} (interval={})", should_run_maintenance, MEMORY_MAINTENANCE_INTERVAL);
    if should_run_maintenance {
        let summary = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_thread_summary(&conn, &thread.thread_id)
        };
        match orchestrator::run_memory_update_with_base(
            &model_config.chat_api_base(), &api_key, &model_config.memory_model,
            &character, &summary, &recent_msgs,
        ).await {
            Ok((update, usage)) => {
                if let Some(u) = usage {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "memory", &model_config.memory_model, u.prompt_tokens, u.completion_tokens);
                }
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                if let Some(new_summary) = update.get("updated_summary").and_then(|v| v.as_str()) {
                    let artifact = MemoryArtifact {
                        artifact_id: format!("summary_{}", thread.thread_id),
                        artifact_type: "thread_summary".to_string(),
                        subject_id: thread.thread_id.clone(),
                        world_id: world.world_id.clone(),
                        content: new_summary.to_string(),
                        sources: json!([]),
                        created_at: Utc::now().to_rfc3339(),
                        updated_at: Utc::now().to_rfc3339(),
                    };
                    let _ = upsert_memory_artifact(&conn, &artifact);
                }
                let _ = reset_message_counter(&conn, &thread.thread_id);
                log::info!("Memory maintenance completed for thread {}", thread.thread_id);
            }
            Err(e) => log::warn!("Memory maintenance failed (non-fatal): {e}"),
        }
    }

    Ok(SendMessageResult {
        user_message,
        assistant_message: assistant_msg,
        ai_reactions,
    })
}

/// Get the most recent message timestamp across all threads in a world.
#[tauri::command]
pub fn get_last_message_time_cmd(
    db: State<Database>,
    world_id: String,
) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    // Check both individual and group message tables
    let indiv: Option<String> = conn.query_row(
        "SELECT m.created_at FROM messages m
         JOIN threads t ON t.thread_id = m.thread_id
         JOIN characters c ON c.character_id = t.character_id
         WHERE c.world_id = ?1
         ORDER BY m.created_at DESC LIMIT 1",
        params![world_id], |r| r.get(0),
    ).ok();
    let group: Option<String> = conn.query_row(
        "SELECT gm.created_at FROM group_messages gm
         JOIN group_chats gc ON gc.thread_id = gm.thread_id
         WHERE gc.world_id = ?1
         ORDER BY gm.created_at DESC LIMIT 1",
        params![world_id], |r| r.get(0),
    ).ok();
    // Return the more recent of the two
    Ok(match (indiv, group) {
        (Some(a), Some(b)) => Some(if a > b { a } else { b }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    })
}

#[derive(serde::Serialize)]
pub struct PaginatedMessages {
    pub messages: Vec<Message>,
    pub total: i64,
}

#[tauri::command]
pub fn get_messages_cmd(
    db: State<Database>,
    character_id: String,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<PaginatedMessages, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
    let total = count_messages(&conn, &thread.thread_id).map_err(|e| e.to_string())?;
    let messages = match limit {
        Some(lim) => list_messages_paginated(&conn, &thread.thread_id, lim, offset.unwrap_or(0))
            .map_err(|e| e.to_string())?,
        None => get_all_messages(&conn, &thread.thread_id).map_err(|e| e.to_string())?,
    };
    Ok(PaginatedMessages { messages, total })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptCharacterResult {
    pub assistant_message: Message,
    pub ai_reactions: Vec<Reaction>,
}

/// Directly edit a message's text content (no LLM call).
#[tauri::command]
pub fn edit_message_content_cmd(
    db: State<'_, Database>,
    message_id: String,
    content: String,
    is_group: bool,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tokens = (content.len() / 4) as i64;
    if is_group {
        update_group_message_content(&conn, &message_id, &content, tokens).map_err(|e| e.to_string())
    } else {
        update_message_content(&conn, &message_id, &content, tokens).map_err(|e| e.to_string())
    }
}

/// Delete a single message by ID.
#[tauri::command]
pub fn delete_message_cmd(
    db: State<'_, Database>,
    message_id: String,
    is_group: bool,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let table = if is_group { "group_messages" } else { "messages" };
    let fts_table = if is_group { "group_messages_fts" } else { "messages_fts" };
    conn.execute(&format!("DELETE FROM {} WHERE message_id = ?1", fts_table), rusqlite::params![message_id]).ok();
    conn.execute(&format!("DELETE FROM {} WHERE message_id = ?1", table), rusqlite::params![message_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn prompt_character_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
) -> Result<PromptCharacterResult, String> {
    let (world, character, thread, recent_msgs, model_config, retrieved,
         user_profile, current_mood, mood_enabled, response_length, narration_tone) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;

        let mut retrieved: Vec<String> = Vec::new();
        let summary = get_thread_summary(&conn, &thread.thread_id);
        if !summary.is_empty() {
            retrieved.push(format!("[Thread summary] {summary}"));
        }

        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let current_mood = get_character_mood(&conn, &character_id);
        let mood_enabled = get_setting(&conn, "mood_drift_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        let response_length = get_setting(&conn, &format!("response_length.{}", character_id))
            .ok().flatten();
        let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
            .ok().flatten();

        (world, character, thread, recent_msgs, model_config, retrieved,
         user_profile, current_mood, mood_enabled, response_length, narration_tone)
    };

    // Mood
    let mood_directive = compute_and_persist_mood(
        &db, &character_id, &world, &character, &recent_msgs,
        current_mood.as_ref(), mood_enabled, None,
    )?;

    // If the last message is from the assistant, add a nudge so the model knows to speak again
    let mut dialogue_msgs = recent_msgs.clone();
    let needs_nudge = dialogue_msgs.last().map(|m| m.role != "user").unwrap_or(true);
    if needs_nudge {
        dialogue_msgs.push(Message {
            message_id: String::new(),
            thread_id: String::new(),
            role: "user".to_string(),
            content: "[The user looks at you expectantly, waiting for you to say something.]".to_string(),
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
            world_day: None, world_time: None,
        });
    }

    // Dialogue
    let (reply_text, dialogue_usage) = orchestrator::run_dialogue_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &dialogue_msgs, &retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        response_length.as_deref(),
        None, None, narration_tone.as_deref(),
    ).await?;

    let tokens = dialogue_usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
    let (wd, wt) = world_time_fields(&world);
    let assistant_msg = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread.thread_id.clone(),
            role: "assistant".to_string(),
            content: reply_text,
            tokens_estimate: tokens as i64,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
            world_day: wd, world_time: wt.clone(),
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;
        msg
    };

    // Reaction on the last user message (disabled — re-enable by uncommenting)
    let ai_reactions: Vec<Reaction> = Vec::new();

    Ok(PromptCharacterResult {
        assistant_message: assistant_msg,
        ai_reactions,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NarrativeResult {
    pub narrative_message: Message,
}

#[tauri::command]
pub async fn generate_narrative_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
    custom_instructions: Option<String>,
) -> Result<NarrativeResult, String> {
    // Read everything from DB
    let (world, character, thread, recent_msgs, model_config, retrieved,
         user_profile, current_mood, mood_enabled, narration_tone, narration_instructions) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);

        let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;

        let mut retrieved: Vec<String> = Vec::new();
        let summary = get_thread_summary(&conn, &thread.thread_id);
        if !summary.is_empty() {
            retrieved.push(format!("[Thread summary] {summary}"));
        }

        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let current_mood = get_character_mood(&conn, &character_id);
        let mood_enabled = get_setting(&conn, "mood_drift_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true);

        let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
            .ok().flatten();
        let narration_instructions = get_setting(&conn, &format!("narration_instructions.{}", character_id))
            .ok().flatten();

        (world, character, thread, recent_msgs, model_config, retrieved,
         user_profile, current_mood, mood_enabled, narration_tone, narration_instructions)
    };

    // Mood directive
    let mood_directive = if mood_enabled {
        let current = current_mood.as_ref()
            .map(crate::ai::mood::MoodVector::from)
            .unwrap_or_else(crate::ai::mood::MoodVector::neutral);
        let directive = crate::ai::mood::mood_to_style_directive(&current);
        if directive.is_empty() { None } else { Some(directive) }
    } else {
        None
    };

    // Check if the previous message is also a narrative — if so, add continuation guidance
    let prev_is_narrative = recent_msgs.last().map(|m| m.role == "narrative").unwrap_or(false);
    let continuation_prefix = if prev_is_narrative {
        Some("IMPORTANT: The previous message in the conversation is also a narrative beat. Do NOT revise or repeat it. Write a CONTINUATION that advances to the NEXT story beat — new action, new moment, new tension. Pick up where the previous narrative left off and move the story forward.".to_string())
    } else {
        None
    };

    // Merge saved narration instructions with ad-hoc custom instructions and continuation guidance
    let all_instructions: Vec<&str> = [
        continuation_prefix.as_deref(),
        narration_instructions.as_deref().filter(|s| !s.is_empty()),
        custom_instructions.as_deref().filter(|s| !s.is_empty()),
    ].into_iter().flatten().collect();
    let merged_instructions = if all_instructions.is_empty() { None } else { Some(all_instructions.join("\n")) };

    // Generate narrative
    let (narrative_text, usage) = orchestrator::run_narrative_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &recent_msgs, &retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        narration_tone.as_deref(),
        merged_instructions.as_deref(),
    ).await?;

    // Store as a "narrative" role message
    let (wd, wt) = world_time_fields(&world);
    let tokens_est = usage.as_ref().map(|u| u.total_tokens as i64).unwrap_or(0);
    let narrative_msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread.thread_id.clone(),
        role: "narrative".to_string(),
        content: narrative_text,
        tokens_estimate: tokens_est,
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
            world_day: wd, world_time: wt.clone(),
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_message(&conn, &narrative_msg).map_err(|e| e.to_string())?;
    }

    Ok(NarrativeResult {
        narrative_message: narrative_msg,
    })
}

// Illustration types and helpers are in illustration_cmds.rs
pub use crate::commands::illustration_cmds::{IllustrationResult, base64_encode_bytes, png_aspect_ratio};

/// Adjust a character message in-place using LLM with user instructions.
#[tauri::command]
pub async fn adjust_message_cmd(
    db: State<'_, Database>,
    audio_dir: State<'_, crate::commands::audio_cmds::AudioDir>,
    api_key: String,
    message_id: String,
    instructions: String,
    is_group: bool,
) -> Result<Message, String> {
    // Load the original message and context
    let (original_msg, character, model_config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let (msg, table) = if is_group {
            let m: Message = conn.query_row(
                "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM group_messages WHERE message_id = ?1",
                params![message_id], |r| Ok(Message {
                    message_id: r.get(0)?, thread_id: r.get(1)?, role: r.get(2)?,
                    content: r.get(3)?, tokens_estimate: r.get(4)?,
                    sender_character_id: r.get(5)?, created_at: r.get(6)?,
                    world_day: r.get(7).ok(), world_time: r.get(8).ok(),
                }),
            ).map_err(|e| format!("Message not found: {e}"))?;
            (m, "group")
        } else {
            let m: Message = conn.query_row(
                "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM messages WHERE message_id = ?1",
                params![message_id], |r| Ok(Message {
                    message_id: r.get(0)?, thread_id: r.get(1)?, role: r.get(2)?,
                    content: r.get(3)?, tokens_estimate: r.get(4)?,
                    sender_character_id: r.get(5)?, created_at: r.get(6)?,
                    world_day: r.get(7).ok(), world_time: r.get(8).ok(),
                }),
            ).map_err(|e| format!("Message not found: {e}"))?;
            (m, "indiv")
        };

        let char_id = msg.sender_character_id.as_deref()
            .or_else(|| if table == "indiv" {
                // For individual chats, get character from thread
                conn.query_row(
                    "SELECT character_id FROM threads WHERE thread_id = ?1",
                    params![msg.thread_id], |r| r.get::<_, String>(0),
                ).ok().as_deref().map(|_| "") // placeholder
            } else { None });

        // Get character for the adjustment prompt context
        let character = if let Some(cid) = &msg.sender_character_id {
            get_character(&conn, cid).map_err(|e| e.to_string())?
        } else if is_group {
            // Group chat — get first character from the group
            let gc_char_ids: String = conn.query_row(
                "SELECT gc.character_ids FROM group_chats gc WHERE gc.thread_id = ?1",
                params![msg.thread_id], |r| r.get(0),
            ).map_err(|e| format!("Could not find group chat for thread: {e}"))?;
            let first_id = serde_json::from_str::<Vec<String>>(&gc_char_ids)
                .unwrap_or_default().into_iter().next()
                .ok_or_else(|| "No characters in group chat".to_string())?;
            get_character(&conn, &first_id).map_err(|e| e.to_string())?
        } else {
            // Individual chat — get character from thread
            let cid: String = conn.query_row(
                "SELECT character_id FROM threads WHERE thread_id = ?1",
                params![msg.thread_id], |r| r.get(0),
            ).map_err(|e| format!("Could not find character for thread: {e}"))?;
            get_character(&conn, &cid).map_err(|e| e.to_string())?
        };

        let model_config = orchestrator::load_model_config(&conn);
        let _ = char_id; // suppress unused warning
        (msg, character, model_config)
    };

    // Build adjustment prompt
    let system_content = if original_msg.role == "narrative" {
        "You are a narrative voice in a conversation. You wrote the following narrative passage. \
         The user wants you to adjust it according to their instructions. \
         Rewrite the passage with the requested changes while keeping the narrative tone and style intact. \
         Output ONLY the adjusted narrative text — no preamble, no explanation, no quotes.".to_string()
    } else {
        format!(
            "You are {}. You wrote the following message in a conversation. \
             The user wants you to adjust it according to their instructions. \
             Rewrite the message with the requested changes while keeping your voice, personality, and the general meaning intact. \
             Output ONLY the adjusted message text — no preamble, no explanation, no quotes.",
            character.display_name,
        )
    };
    let messages = vec![
        openai::ChatMessage {
            role: "system".to_string(),
            content: system_content,
        },
        openai::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Original message:\n{}\n\nAdjust it according to these instructions:\n{}",
                original_msg.content, instructions,
            ),
        },
    ];

    let request = openai::ChatRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.95),
        max_completion_tokens: Some(1024),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(
        &model_config.chat_api_base(), &api_key, &request,
    ).await?;

    let new_content = response.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from model".to_string())?;

    let tokens = response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);

    // Update in place
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if is_group {
            update_group_message_content(&conn, &message_id, &new_content, tokens as i64)
                .map_err(|e| e.to_string())?;
        } else {
            update_message_content(&conn, &message_id, &new_content, tokens as i64)
                .map_err(|e| e.to_string())?;
        }
        if let Some(u) = &response.usage {
            let _ = record_token_usage(&conn, "adjust_message", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
        }
    }

    // Delete any cached audio for this message (content changed)
    crate::commands::audio_cmds::delete_audio_for_message(&audio_dir.0, &message_id);

    Ok(Message {
        message_id: original_msg.message_id,
        thread_id: original_msg.thread_id,
        role: original_msg.role,
        content: new_content,
        tokens_estimate: tokens as i64,
        sender_character_id: original_msg.sender_character_id,
        created_at: original_msg.created_at,
        world_day: original_msg.world_day,
        world_time: original_msg.world_time,
    })
}

// Illustration commands are in illustration_cmds.rs
// Video commands are in video_cmds.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetToMessageResult {
    pub deleted_count: usize,
    /// If the anchor message was from the user, this contains the new AI response
    pub new_response: Option<SendMessageResult>,
}

#[tauri::command]
pub async fn reset_to_message_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    audio_dir: State<'_, crate::commands::audio_cmds::AudioDir>,
    api_key: String,
    character_id: String,
    message_id: String,
) -> Result<ResetToMessageResult, String> {
    let is_group = character_id.is_empty();

    // Phase 1: Delete messages after the anchor
    let (anchor_role, anchor_content, deleted_count, thread_id, world, character, model_config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let anchor: Message = {
            let table = if is_group { "group_messages" } else { "messages" };
            let sql = format!("SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM {} WHERE message_id = ?1", table);
            let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
            stmt.query_row(params![message_id], |row| {
                Ok(Message {
                    message_id: row.get(0)?,
                    thread_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    tokens_estimate: row.get(4)?,
                    sender_character_id: row.get(5)?,
                    created_at: row.get(6)?,
                    world_day: row.get(7).ok(),
                    world_time: row.get(8).ok(),
                })
            }).map_err(|e| e.to_string())?
        };

        let thread_id = anchor.thread_id.clone();
        let char_id_for_cleanup = if is_group { "" } else { &character_id };

        let (character, world, model_config) = if is_group {
            // For group chats, get world from thread
            let world_id: String = conn.query_row(
                "SELECT world_id FROM threads WHERE thread_id = ?1",
                params![thread_id], |r| r.get(0),
            ).map_err(|e| e.to_string())?;
            let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
            let mc = orchestrator::load_model_config(&conn);
            // Dummy character for group — won't be used for re-generation
            let dummy = Character {
                character_id: String::new(), world_id, display_name: String::new(),
                identity: String::new(), voice_rules: serde_json::json!([]),
                boundaries: serde_json::json!([]), backstory_facts: serde_json::json!([]),
                relationships: serde_json::json!({}), state: serde_json::json!({}),
                avatar_color: String::new(), is_archived: false,
                created_at: String::new(), updated_at: String::new(),
            };
            (dummy, world, mc)
        } else {
            let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
            let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
            let mc = orchestrator::load_model_config(&conn);
            (character, world, mc)
        };

        let (deleted, illustration_files) = if is_group {
            delete_group_messages_after(&conn, &thread_id, &message_id)
        } else {
            delete_messages_after(&conn, &thread_id, char_id_for_cleanup, &message_id)
        }
            .map_err(|e| e.to_string())?;

        for f in &illustration_files {
            let path = portraits_dir.0.join(f);
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }

        // Clean up audio files for deleted messages
        for (msg_id, _) in &deleted {
            crate::commands::audio_cmds::delete_audio_for_message(&audio_dir.0, msg_id);
        }

        (anchor.role, anchor.content, deleted.len(), thread_id, world, character, model_config)
    };

    // Phase 2: Rebuild thread summary from remaining messages so the model has accurate context
    {
        let recent_msgs = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            if is_group {
                list_group_messages(&conn, &thread_id, 30).map_err(|e| e.to_string())?
            } else {
                list_messages(&conn, &thread_id, 30).map_err(|e| e.to_string())?
            }
        };
        if recent_msgs.len() >= 4 {
            match orchestrator::run_memory_update_with_base(
                &model_config.chat_api_base(), &api_key, &model_config.memory_model,
                &character, "", &recent_msgs,
            ).await {
                Ok((update, usage)) => {
                    if let Some(u) = &usage {
                        let conn = db.conn.lock().map_err(|e| e.to_string())?;
                        let _ = record_token_usage(&conn, "memory", &model_config.memory_model, u.prompt_tokens, u.completion_tokens);
                    }
                    if let Some(new_summary) = update.get("updated_summary").and_then(|v| v.as_str()) {
                        let conn = db.conn.lock().map_err(|e| e.to_string())?;
                        let artifact = MemoryArtifact {
                            artifact_id: format!("summary_{}", thread_id),
                            artifact_type: "thread_summary".to_string(),
                            subject_id: thread_id.clone(),
                            world_id: world.world_id.clone(),
                            content: new_summary.to_string(),
                            sources: json!([]),
                            created_at: Utc::now().to_rfc3339(),
                            updated_at: Utc::now().to_rfc3339(),
                        };
                        let _ = upsert_memory_artifact(&conn, &artifact);
                        log::info!("[Reset] Rebuilt thread summary ({} chars)", new_summary.len());
                    }
                }
                Err(e) => log::warn!("[Reset] Summary rebuild failed (non-fatal): {e}"),
            }
        }
    }

    // Phase 3: If the anchor is a user message in a 1-on-1 chat, generate a new character response
    // (Skip for group chats — no automatic re-generation)
    if anchor_role == "user" && !is_group {
        let (recent_msgs, retrieved, user_profile, current_mood, mood_enabled, response_length, narration_tone) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let recent_msgs = list_messages(&conn, &thread_id, 30).map_err(|e| e.to_string())?;

            let mut retrieved: Vec<String> = Vec::new();
            let summary = get_thread_summary(&conn, &thread_id);
            if !summary.is_empty() {
                retrieved.push(format!("[Thread summary] {summary}"));
            }

            let fts_query = sanitize_fts_query(&anchor_content);
            if !fts_query.is_empty() {
                if let Ok(fts_msgs) = search_messages_fts(&conn, &thread_id, &fts_query, 6) {
                    for m in fts_msgs.into_iter().filter(|m| m.role != "illustration" && m.role != "video") {
                        retrieved.push(format!("[{}] {}: {}", m.created_at, m.role, m.content));
                    }
                }
            }

            let user_profile = get_user_profile(&conn, &character.world_id).ok();
            let current_mood = get_character_mood(&conn, &character_id);
            let mood_enabled = get_setting(&conn, "mood_drift_enabled")
                .ok().flatten().map(|v| v == "true").unwrap_or(true);
            let response_length = get_setting(&conn, &format!("response_length.{}", character_id))
                .ok().flatten();
            let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
                .ok().flatten();

            (recent_msgs, retrieved, user_profile, current_mood, mood_enabled, response_length, narration_tone)
        };

        // Mood directive
        let mood_directive = compute_and_persist_mood(
            &db, &character_id, &world, &character, &recent_msgs,
            current_mood.as_ref(), mood_enabled, None,
        )?;

        // Generate dialogue
        let (reply_text, dialogue_usage) = orchestrator::run_dialogue_with_base(
            &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
            &world, &character, &recent_msgs, &retrieved,
            user_profile.as_ref(),
            mood_directive.as_deref(),
            response_length.as_deref(),
            None, None, narration_tone.as_deref(),
        ).await?;

        if let Some(u) = &dialogue_usage {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = record_token_usage(&conn, "dialogue", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
        }

        let tokens = dialogue_usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
        let (wd, wt) = world_time_fields(&world);
        let (user_message, assistant_msg) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let msg = Message {
                message_id: uuid::Uuid::new_v4().to_string(),
                thread_id: thread_id.clone(),
                role: "assistant".to_string(),
                content: reply_text,
                tokens_estimate: tokens as i64,
                sender_character_id: None,
                created_at: Utc::now().to_rfc3339(),
            world_day: wd, world_time: wt.clone(),
            };
            create_message(&conn, &msg).map_err(|e| e.to_string())?;
            increment_message_counter(&conn, &thread_id).map_err(|e| e.to_string())?;

            let user_message = recent_msgs.last().cloned().unwrap_or_else(|| Message {
                message_id: String::new(), thread_id: thread_id.clone(),
                role: "user".to_string(), content: anchor_content.clone(),
                tokens_estimate: 0, created_at: Utc::now().to_rfc3339(),
            world_day: None, world_time: None,
            sender_character_id: None,
            });

            (user_message, msg)
        };

        // Reaction (disabled — re-enable by uncommenting)
        let ai_reactions: Vec<Reaction> = Vec::new();

        return Ok(ResetToMessageResult {
            deleted_count,
            new_response: Some(SendMessageResult {
                user_message,
                assistant_message: assistant_msg,
                ai_reactions,
            }),
        });
    }

    Ok(ResetToMessageResult {
        deleted_count,
        new_response: None,
    })
}

/// Strip FTS5 special characters and extract plain words for safe MATCH queries.
fn sanitize_fts_query(input: &str) -> String {
    let words: Vec<&str> = input
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| w.len() >= 2)
        .collect();
    if words.is_empty() {
        return String::new();
    }
    // Join with spaces — FTS5 implicit AND
    words.join(" ")
}

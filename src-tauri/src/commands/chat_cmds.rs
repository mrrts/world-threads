use crate::ai::google;
use crate::ai::mood;
use crate::ai::openai::{self, ChatRequest};
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

    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread.thread_id.clone(),
        role: "user".to_string(),
        content: content.clone(),
        tokens_estimate: (content.len() as i64) / 4,
            sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
    };
    create_message(&conn, &msg).map_err(|e| e.to_string())?;
    increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

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

        let user_msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread.thread_id.clone(),
            role: "user".to_string(),
            content: content.clone(),
            tokens_estimate: (content.len() as i64) / 4,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
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
    let mood_directive = if mood_enabled {
        let current = current_mood.as_ref()
            .map(mood::MoodVector::from)
            .unwrap_or_else(mood::MoodVector::neutral);
        let target = mood::compute_mood_target(&world, &character, &recent_msgs);
        let drifted = mood::drift_mood(&current, &target, mood_drift_rate);
        let directive = mood::mood_to_style_directive(&drifted);

        let history = current_mood.as_ref()
            .map(|m| m.history.clone())
            .unwrap_or_else(|| serde_json::Value::Array(vec![]));
        let new_history = mood::append_mood_history(&history, &drifted);

        let updated = CharacterMood {
            character_id: character_id.clone(),
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

        if directive.is_empty() { None } else { Some(directive) }
    } else {
        None
    };

    // Phase 4: Run dialogue (async, no DB lock)
    let (reply_text, dialogue_usage) = orchestrator::run_dialogue_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &recent_msgs, &full_retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        response_length.as_deref(),
        None, None, narration_tone.as_deref(),
    ).await?;
    let tokens = dialogue_usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
    if let Some(u) = &dialogue_usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "dialogue", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

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
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

        let user_message = recent_msgs.last().cloned().unwrap_or_else(|| Message {
            message_id: String::new(), thread_id: thread.thread_id.clone(),
            role: "user".to_string(), content: content.clone(),
            tokens_estimate: 0, created_at: Utc::now().to_rfc3339(),
            sender_character_id: None,
        });

        (user_message, msg)
    };

    // Phase 6: AI reaction to user's message
    let mut ai_reactions: Vec<Reaction> = Vec::new();
    match orchestrator::generate_reaction_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &character, &content, &assistant_msg.content,
    ).await {
        Ok((Some(emoji), usage)) => {
            if let Some(u) = usage {
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                let _ = record_token_usage(&conn, "reaction", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
            }
            let reaction = Reaction {
                reaction_id: uuid::Uuid::new_v4().to_string(),
                message_id: user_message.message_id.clone(),
                emoji,
                reactor: "assistant".to_string(),
                created_at: Utc::now().to_rfc3339(),
            };
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = add_reaction(&conn, &reaction);
            ai_reactions.push(reaction);
        }
        Ok((None, usage)) => {
            if let Some(u) = usage {
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                let _ = record_token_usage(&conn, "reaction", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
            }
        }
        Err(e) => log::warn!("Reaction generation failed (non-fatal): {e}"),
    }

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
    let mood_directive = if mood_enabled {
        let current = current_mood.as_ref()
            .map(crate::ai::mood::MoodVector::from)
            .unwrap_or_else(crate::ai::mood::MoodVector::neutral);
        let target = crate::ai::mood::compute_mood_target(&world, &character, &recent_msgs);
        let drifted = crate::ai::mood::drift_mood(&current, &target, None);
        let directive = crate::ai::mood::mood_to_style_directive(&drifted);

        let history = current_mood.as_ref()
            .map(|m| m.history.clone())
            .unwrap_or_else(|| serde_json::Value::Array(vec![]));
        let new_history = crate::ai::mood::append_mood_history(&history, &drifted);

        let updated = CharacterMood {
            character_id: character_id.clone(),
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
        if directive.is_empty() { None } else { Some(directive) }
    } else {
        None
    };

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

    if let Some(u) = &dialogue_usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "dialogue", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    let tokens = dialogue_usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
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
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;
        msg
    };

    // Reaction on the last user message (if any)
    let mut ai_reactions: Vec<Reaction> = Vec::new();
    if let Some(last_user) = recent_msgs.iter().rev().find(|m| m.role == "user") {
        match orchestrator::generate_reaction_with_base(
            &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
            &character, &last_user.content, &assistant_msg.content,
        ).await {
            Ok((Some(emoji), usage)) => {
                if let Some(u) = usage {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "reaction", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
                }
                let reaction = Reaction {
                    reaction_id: uuid::Uuid::new_v4().to_string(),
                    message_id: last_user.message_id.clone(),
                    emoji,
                    reactor: "assistant".to_string(),
                    created_at: Utc::now().to_rfc3339(),
                };
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                let _ = add_reaction(&conn, &reaction);
                ai_reactions.push(reaction);
            }
            Ok((None, usage)) => {
                if let Some(u) = usage {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "reaction", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
                }
            }
            Err(e) => log::warn!("Reaction generation failed (non-fatal): {e}"),
        }
    }

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

    // Generate narrative
    let (narrative_text, usage) = orchestrator::run_narrative_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &recent_msgs, &retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        narration_tone.as_deref(),
        narration_instructions.as_deref(),
    ).await?;

    if let Some(u) = &usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "narrative", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    // Store as a "narrative" role message
    let narrative_msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread.thread_id.clone(),
        role: "narrative".to_string(),
        content: narrative_text,
        tokens_estimate: usage.as_ref().map(|u| u.total_tokens as i64).unwrap_or(0),
        sender_character_id: None,
        created_at: Utc::now().to_rfc3339(),
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_message(&conn, &narrative_msg).map_err(|e| e.to_string())?;
    }

    Ok(NarrativeResult {
        narrative_message: narrative_msg,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IllustrationResult {
    pub illustration_message: Message,
}

#[tauri::command]
pub async fn generate_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
    quality_tier: Option<String>,
    custom_instructions: Option<String>,
    previous_illustration_id: Option<String>,
    include_scene_summary: Option<bool>,
) -> Result<IllustrationResult, String> {
    let (world, character, thread_id, recent_msgs, model_config, user_profile) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;
        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        (world, character, thread.thread_id, recent_msgs, model_config, user_profile)
    };

    // Load reference portraits: user avatar first, then character's active portrait
    let mut reference_images: Vec<Vec<u8>> = Vec::new();
    let dir = &portraits_dir.0;

    // User avatar
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

    // Character active portrait
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Some(portrait) = get_active_portrait(&conn, &character_id) {
            let path = dir.join(&portrait.file_name);
            if path.exists() {
                if let Ok(bytes) = std::fs::read(&path) {
                    reference_images.push(bytes);
                }
            }
        }
    }

    // Previous illustration as reference (if requested)
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

    // Resolve quality tier to image size and quality
    let tier = quality_tier.as_deref().unwrap_or("high");
    let (img_size, img_quality) = match tier {
        "low" => ("1024x1024", "low"),
        "medium" => ("1024x1024", "medium"),
        _ => ("1536x1024", "medium"),  // "high"
    };

    log::info!("[Illustration] Generating for '{}' with {} reference images (tier={}, size={}, quality={})",
        character.display_name, reference_images.len(), tier, img_size, img_quality);

    let (scene_description, image_bytes, chat_usage) = orchestrator::generate_illustration_with_base(
        &model_config.chat_api_base(),
        &model_config.openai_api_base(),
        &api_key,
        &model_config.dialogue_model,
        &model_config.image_model,
        img_quality,
        img_size,
        model_config.image_output_format().as_deref(),
        &world, &character, &recent_msgs,
        user_profile.as_ref(),
        &reference_images,
        custom_instructions.as_deref(),
        has_previous,
        include_scene_summary.unwrap_or(true),
        None,
    ).await?;

    if let Some(u) = &chat_usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "illustration", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    // Use message_id as image_id so they're linked for cleanup
    let message_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("illustration_{message_id}.png");
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &image_bytes)
        .map_err(|e| format!("Failed to save illustration: {e}"))?;

    log::info!("[Illustration] Saved {} ({} bytes)", file_name, image_bytes.len());

    let aspect = png_aspect_ratio(&image_bytes);
    let b64 = base64_encode_bytes(&image_bytes);
    let data_url = format!("data:image/png;base64,{b64}");
    let now = Utc::now().to_rfc3339();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        // Save to world gallery (linked by message_id)
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
            thread_id: thread_id.clone(),
            role: "illustration".to_string(),
            content: data_url,
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: now,
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let illustration_msg = conn.query_row(
        "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at FROM messages WHERE message_id = ?1",
        params![message_id], |row| Ok(Message {
            message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
            content: row.get(3)?, tokens_estimate: row.get(4)?, sender_character_id: row.get(5)?, created_at: row.get(6)?,
        })
    ).map_err(|e| e.to_string())?;

    Ok(IllustrationResult {
        illustration_message: illustration_msg,
    })
}

/// Encode bytes to base64 string.
pub fn base64_encode_bytes(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Get aspect ratio (width/height) from PNG image bytes.
pub fn png_aspect_ratio(bytes: &[u8]) -> f64 {
    if bytes.len() >= 24 && &bytes[0..4] == b"\x89PNG" {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as f64;
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as f64;
        if h > 0.0 { w / h } else { 1.0 }
    } else {
        1.0
    }
}

/// Delete a single illustration: message, gallery entry, and file on disk.
fn delete_illustration_inner(conn: &rusqlite::Connection, portraits_dir: &std::path::Path, message_id: &str) -> Result<(), String> {
    // Delete associated video file if one exists
    let video_file: Option<String> = conn.query_row(
        "SELECT video_file FROM world_images WHERE image_id = ?1",
        params![message_id], |r| r.get(0),
    ).ok();
    if let Some(ref vf) = video_file {
        if !vf.is_empty() {
            let path = portraits_dir.join(vf);
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    // Delete gallery entry (linked by message_id = image_id)
    let file_name: Option<String> = conn.query_row(
        "SELECT file_name FROM world_images WHERE image_id = ?1",
        params![message_id], |r| r.get(0),
    ).ok();
    conn.execute("DELETE FROM world_images WHERE image_id = ?1", params![message_id])
        .map_err(|e| e.to_string())?;
    // Delete FTS entry
    conn.execute("DELETE FROM messages_fts WHERE message_id = ?1", params![message_id]).ok();
    // Delete message
    conn.execute("DELETE FROM messages WHERE message_id = ?1", params![message_id])
        .map_err(|e| e.to_string())?;
    // Delete illustration image file
    if let Some(f) = file_name {
        let path = portraits_dir.join(&f);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    message_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    delete_illustration_inner(&conn, &portraits_dir.0, &message_id)
}

#[tauri::command]
pub async fn regenerate_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
    message_id: String,
) -> Result<IllustrationResult, String> {
    // Delete the old illustration
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        delete_illustration_inner(&conn, &portraits_dir.0, &message_id)?;
    }

    // Generate a new one (reuses the full generate_illustration_cmd logic)
    generate_illustration_cmd(db, portraits_dir, api_key, character_id, Some("high".to_string()), None, None, None).await
}

#[tauri::command]
pub async fn adjust_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
    message_id: String,
    instructions: String,
) -> Result<IllustrationResult, String> {
    // Load the current illustration image, model config, and reference portraits
    let (image_bytes, world, character, thread, model_config, user_profile) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let user_profile = get_user_profile(&conn, &character.world_id).ok();

        // Read the current illustration file
        let file_name: String = conn.query_row(
            "SELECT file_name FROM world_images WHERE image_id = ?1",
            params![message_id], |r| r.get(0),
        ).map_err(|_| "Illustration not found in gallery".to_string())?;

        let path = portraits_dir.0.join(&file_name);
        let bytes = std::fs::read(&path)
            .map_err(|e| format!("Failed to read illustration file: {e}"))?;

        (bytes, world, character, thread, model_config, user_profile)
    };

    let dir = &portraits_dir.0;

    // Build reference images: current illustration first, then user avatar, then character portrait
    let mut reference_images: Vec<Vec<u8>> = vec![image_bytes];

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

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Some(portrait) = get_active_portrait(&conn, &character_id) {
            let path = dir.join(&portrait.file_name);
            if path.exists() {
                if let Ok(bytes) = std::fs::read(&path) {
                    reference_images.push(bytes);
                }
            }
        }
    }

    let user_name = user_profile.as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");

    // Build the adjustment prompt
    let prompt_parts = vec![
        "Hand-painted watercolor illustration in a lush, realistic style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones.".to_string(),
        "Wide cinematic composition.".to_string(),
        "The first reference image is the current illustration to adjust. Preserve its overall composition and scene.".to_string(),
        format!("The other reference images show {} and {}. Keep them recognizable.", user_name, character.display_name),
        format!("ADJUSTMENT INSTRUCTIONS:\n{instructions}"),
        "Apply the requested changes while keeping everything else about the scene intact.".to_string(),
        "CRITICAL: The image must contain absolutely no text, no words, no letters, no numbers, no writing, no labels, no titles, no captions, no watermarks, no signatures, no UI elements, no names.".to_string(),
    ];

    let prompt = prompt_parts.join(" ");

    log::info!("[Illustration Adjust] Adjusting with {} reference images, instructions: {:.100}", reference_images.len(), instructions);

    let response = crate::ai::openai::generate_image_edit_with_base(
        &model_config.openai_api_base(), &api_key, &model_config.image_model,
        &prompt, &reference_images,
        "1536x1024", model_config.image_quality(),
        model_config.image_output_format().as_deref(),
    ).await?;

    let b64 = response.data.first()
        .and_then(|d| d.image_b64())
        .ok_or_else(|| "No image data in response".to_string())?;

    let new_image_bytes = orchestrator::openai_base64_decode_pub(b64)?;

    // Delete the old illustration
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        delete_illustration_inner(&conn, &portraits_dir.0, &message_id)?;
    }

    // Save new image
    let new_message_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("illustration_{new_message_id}.png");
    std::fs::write(dir.join(&file_name), &new_image_bytes)
        .map_err(|e| format!("Failed to save adjusted illustration: {e}"))?;

    let aspect = png_aspect_ratio(&new_image_bytes);
    let b64_out = base64_encode_bytes(&new_image_bytes);
    let data_url = format!("data:image/png;base64,{b64_out}");
    let now = Utc::now().to_rfc3339();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let img = WorldImage {
            image_id: new_message_id.clone(),
            world_id: world.world_id.clone(),
            prompt: instructions.clone(),
            file_name,
            is_active: false,
            source: "illustration".to_string(),
            created_at: now.clone(),
            aspect_ratio: aspect,
        };
        let _ = create_world_image(&conn, &img);

        let msg = Message {
            message_id: new_message_id.clone(),
            thread_id: thread.thread_id.clone(),
            role: "illustration".to_string(),
            content: data_url,
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: now,
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let illustration_msg = conn.query_row(
        "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at FROM messages WHERE message_id = ?1",
        params![new_message_id], |row| Ok(Message {
            message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
            content: row.get(3)?, tokens_estimate: row.get(4)?, sender_character_id: row.get(5)?, created_at: row.get(6)?,
        })
    ).map_err(|e| e.to_string())?;

    Ok(IllustrationResult {
        illustration_message: illustration_msg,
    })
}

/// Generate a video from an existing illustration. Attaches the video file to the illustration's
/// world_images record via the video_file column. Returns the video filename.
#[tauri::command]
pub async fn generate_video_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    google_api_key: String,
    character_id: String,
    illustration_message_id: String,
    custom_prompt: Option<String>,
    duration_seconds: Option<u32>,
    style: Option<String>,
) -> Result<String, String> {
    let is_group = character_id.is_empty();

    // Load context
    let (character, recent_msgs, model_config, user_profile, illustration_file) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);

        // Get the illustration file and timestamp — check both messages and group_messages
        let (file_name, illus_created_at, thread_id): (String, String, String) = conn.query_row(
            "SELECT w.file_name, m.created_at, m.thread_id FROM world_images w JOIN messages m ON m.message_id = w.image_id WHERE w.image_id = ?1",
            params![illustration_message_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ).or_else(|_| conn.query_row(
            "SELECT w.file_name, m.created_at, m.thread_id FROM world_images w JOIN group_messages m ON m.message_id = w.image_id WHERE w.image_id = ?1",
            params![illustration_message_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )).map_err(|_| "Illustration not found".to_string())?;

        // Get world from thread
        let world_id: String = conn.query_row(
            "SELECT world_id FROM threads WHERE thread_id = ?1",
            params![thread_id], |r| r.get(0),
        ).map_err(|e| e.to_string())?;

        let character = if is_group {
            // Dummy character for animation prompt
            let chars_in_world = list_characters(&conn, &world_id).unwrap_or_default();
            chars_in_world.into_iter().next().unwrap_or_else(|| Character {
                character_id: String::new(), world_id: world_id.clone(), display_name: String::new(),
                identity: String::new(), voice_rules: serde_json::json!([]),
                boundaries: serde_json::json!([]), backstory_facts: serde_json::json!([]),
                relationships: serde_json::json!({}), state: serde_json::json!({}),
                avatar_color: String::new(), is_archived: false,
                created_at: String::new(), updated_at: String::new(),
            })
        } else {
            get_character(&conn, &character_id).map_err(|e| e.to_string())?
        };

        let user_profile = get_user_profile(&conn, &world_id).ok();

        // Get messages up to the illustration's creation time
        let msg_table = if is_group { "group_messages" } else { "messages" };
        let sql = format!(
            "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at
             FROM {} WHERE thread_id = ?1 AND created_at <= ?2
             ORDER BY created_at DESC LIMIT 30", msg_table
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let mut recent_msgs: Vec<Message> = stmt.query_map(params![thread_id, illus_created_at], |row| {
            Ok(Message {
                message_id: row.get(0)?,
                thread_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                tokens_estimate: row.get(4)?,
                sender_character_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        recent_msgs.reverse();

        (character, recent_msgs, model_config, user_profile, file_name)
    };

    let dir = &portraits_dir.0;

    // Read the illustration image
    let image_bytes = std::fs::read(dir.join(&illustration_file))
        .map_err(|e| format!("Failed to read illustration: {e}"))?;
    let image_b64 = base64_encode_bytes(&image_bytes);

    // Generate animation prompt, appending custom instructions if provided
    let mut animation_prompt = generate_animation_prompt(&api_key, &model_config, &character, user_profile.as_ref(), &recent_msgs).await?;
    if let Some(ref custom) = custom_prompt {
        if !custom.is_empty() {
            animation_prompt.push_str(&format!(" Additionally: {custom}"));
        }
    }

    // Build the full Veo prompt: style directive + animation direction + character context
    let user_name = user_profile.as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");

    let style_directive = match style.as_deref().unwrap_or("action-no-dialogue") {
        "still" => "Subtle ambient motion only — gentle breathing, wind, light shifts. Characters remain mostly still. No speech or dialogue.",
        "dialogue" => "Characters are talking. Show natural lip movement, facial expressions, and conversational gestures. Include realistic speech motion.",
        "action-no-dialogue" => "Characters in motion with expressive body language and physical action. No speech or lip movement — a silent scene.",
        "action-dialogue" => "Characters in motion with expressive body language AND speaking. Show natural lip movement alongside physical action and gestures.",
        _ => "Characters in motion with expressive body language and physical action. No speech or lip movement — a silent scene.",
    };

    let veo_prompt = format!(
        "Cinematic, realistic animation with natural lighting, lifelike motion, and subtle detail. Maintain the characters and composition from the reference image but render with photorealistic quality. {style_directive} {animation_prompt} The scene shows {user} and {char} together.",
        user = user_name,
        char = character.display_name,
    );

    log::info!("[Video] Veo prompt: {:.300}", veo_prompt);

    // Start Veo generation with the illustration as the first frame
    // Try the full model first, fall back to lite on rate limit
    let dur = Some(duration_seconds.unwrap_or(8));
    let has_dialogue = matches!(style.as_deref(), Some("dialogue") | Some("action-dialogue"));
    let audio = if has_dialogue { Some(true) } else { None };
    let models = [
        "veo-3.1-generate-preview",
        "veo-3.1-lite-generate-preview",
        "veo-3.1-fast-generate-preview",
    ];
    let mut operation = None;
    for (i, model) in models.iter().enumerate() {
        match google::start_veo_generation(
            &google_api_key, model, Some(&image_b64), &veo_prompt, dur, Some("16:9"), audio,
        ).await {
            Ok(op) => { operation = Some(op); break; }
            Err(e) if e == "RATE_LIMITED" => {
                if i < models.len() - 1 {
                    log::info!("[Video] {} rate limited, trying {}", model, models[i + 1]);
                } else {
                    return Err("DAILY_LIMIT_REACHED".to_string());
                }
            }
            Err(e) => return Err(e),
        }
    }
    let operation = operation.ok_or_else(|| "DAILY_LIMIT_REACHED".to_string())?;

    // Poll until done
    let video_uri = google::poll_veo_until_done(&google_api_key, &operation).await?;

    // Download video
    let video_bytes = google::download_video(&video_uri, &google_api_key).await?;

    // Save video file alongside the illustration
    let video_file = format!("video_{illustration_message_id}.mp4");
    std::fs::write(dir.join(&video_file), &video_bytes)
        .map_err(|e| format!("Failed to save video: {e}"))?;

    log::info!("[Video] Saved {} ({} bytes)", video_file, video_bytes.len());

    // Attach video to the illustration's world_images record
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE world_images SET video_file = ?1 WHERE image_id = ?2",
            params![video_file, illustration_message_id],
        ).map_err(|e| e.to_string())?;
    }

    Ok(video_file)
}

/// Get a video file as a base64 data URL for playback.
#[tauri::command]
pub fn get_video_bytes_cmd(
    portraits_dir: State<'_, PortraitsDir>,
    video_file: String,
) -> Result<Vec<u8>, String> {
    let path = portraits_dir.0.join(&video_file);
    if !path.exists() {
        return Err("Video file not found".to_string());
    }
    std::fs::read(&path).map_err(|e| format!("Failed to read video: {e}"))
}

/// Remove the video file attached to an illustration.
#[tauri::command]
pub fn remove_video_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    illustration_message_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let video_file: String = conn.query_row(
        "SELECT COALESCE(video_file, '') FROM world_images WHERE image_id = ?1",
        params![illustration_message_id], |r| r.get(0),
    ).unwrap_or_default();
    if !video_file.is_empty() {
        let path = portraits_dir.0.join(&video_file);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        conn.execute(
            "UPDATE world_images SET video_file = '' WHERE image_id = ?1",
            params![illustration_message_id],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Download an illustration image to ~/Downloads.
#[tauri::command]
pub fn download_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    illustration_message_id: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let file_name: String = conn.query_row(
        "SELECT file_name FROM world_images WHERE image_id = ?1",
        params![illustration_message_id], |r| r.get(0),
    ).map_err(|_| "Illustration not found".to_string())?;

    let src = portraits_dir.0.join(&file_name);
    if !src.exists() {
        return Err("Illustration file not found on disk".to_string());
    }

    let home = std::env::var("HOME").map_err(|_| "Could not find home directory".to_string())?;
    let downloads = std::path::PathBuf::from(home).join("Downloads");
    let dest = downloads.join(&file_name);
    std::fs::copy(&src, &dest).map_err(|e| format!("Failed to copy: {e}"))?;

    Ok(dest.to_string_lossy().to_string())
}

/// Upload a video file and attach it to an illustration.
#[tauri::command]
pub fn upload_video_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    illustration_message_id: String,
    video_data: String,
) -> Result<String, String> {
    // video_data is base64-encoded video bytes (from frontend FileReader)
    let raw = if video_data.contains(',') {
        video_data.split(',').nth(1).unwrap_or(&video_data)
    } else {
        &video_data
    };

    let video_bytes = orchestrator::openai_base64_decode_pub(raw)?;

    let video_file = format!("video_{illustration_message_id}.mp4");
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&video_file), &video_bytes)
        .map_err(|e| format!("Failed to save video: {e}"))?;

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE world_images SET video_file = ?1 WHERE image_id = ?2",
        params![video_file, illustration_message_id],
    ).map_err(|e| e.to_string())?;

    Ok(video_file)
}

/// Get the aspect ratio for an illustration. Returns 0.0 if unknown.
#[tauri::command]
pub fn get_illustration_aspect_ratio_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    illustration_message_id: String,
) -> Result<f64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let (ratio, file_name): (f64, String) = conn.query_row(
        "SELECT COALESCE(aspect_ratio, 0.0), file_name FROM world_images WHERE image_id = ?1",
        params![illustration_message_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).unwrap_or((0.0, String::new()));

    // Backfill if unknown
    if ratio == 0.0 && !file_name.is_empty() {
        let path = portraits_dir.0.join(&file_name);
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                let ar = png_aspect_ratio(&bytes);
                if ar > 0.0 {
                    let _ = conn.execute(
                        "UPDATE world_images SET aspect_ratio = ?1 WHERE image_id = ?2",
                        params![ar, illustration_message_id],
                    );
                    return Ok(ar);
                }
            }
        }
    }

    Ok(ratio)
}

/// Get the video file name for an illustration, if one has been generated.
#[tauri::command]
pub fn get_video_file_cmd(
    db: State<'_, Database>,
    illustration_message_id: String,
) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let video_file: String = conn.query_row(
        "SELECT COALESCE(video_file, '') FROM world_images WHERE image_id = ?1",
        params![illustration_message_id], |r| r.get(0),
    ).unwrap_or_default();
    if video_file.is_empty() {
        Ok(None)
    } else {
        Ok(Some(video_file))
    }
}

/// Helper: generate animation prompt via chat model
async fn generate_animation_prompt(
    api_key: &str,
    model_config: &orchestrator::ModelConfig,
    character: &Character,
    user_profile: Option<&UserProfile>,
    recent_msgs: &[Message],
) -> Result<String, String> {
    use crate::ai::prompts;

    let messages = prompts::build_animation_prompt(character, user_profile, recent_msgs);
    let request = ChatRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.9),
        max_completion_tokens: Some(200),
        response_format: None,
    };
    let response = openai::chat_completion_with_base(
        &model_config.chat_api_base(), api_key, &request,
    ).await?;
    let prompt = response.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No animation prompt from model".to_string())?;
    Ok(prompt)
}

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
            let sql = format!("SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at FROM {} WHERE message_id = ?1", table);
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
        let mood_directive = if mood_enabled {
            let current = current_mood.as_ref()
                .map(crate::ai::mood::MoodVector::from)
                .unwrap_or_else(crate::ai::mood::MoodVector::neutral);
            let target = crate::ai::mood::compute_mood_target(&world, &character, &recent_msgs);
            let drifted = crate::ai::mood::drift_mood(&current, &target, None);
            let directive = crate::ai::mood::mood_to_style_directive(&drifted);

            let history = current_mood.as_ref()
                .map(|m| m.history.clone())
                .unwrap_or_else(|| serde_json::Value::Array(vec![]));
            let new_history = crate::ai::mood::append_mood_history(&history, &drifted);

            let updated = CharacterMood {
                character_id: character_id.clone(),
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
            if directive.is_empty() { None } else { Some(directive) }
        } else {
            None
        };

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
            };
            create_message(&conn, &msg).map_err(|e| e.to_string())?;
            increment_message_counter(&conn, &thread_id).map_err(|e| e.to_string())?;

            let user_message = recent_msgs.last().cloned().unwrap_or_else(|| Message {
                message_id: String::new(), thread_id: thread_id.clone(),
                role: "user".to_string(), content: anchor_content.clone(),
                tokens_estimate: 0, created_at: Utc::now().to_rfc3339(),
            sender_character_id: None,
            });

            (user_message, msg)
        };

        // Reaction
        let mut ai_reactions: Vec<Reaction> = Vec::new();
        match orchestrator::generate_reaction_with_base(
            &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
            &character, &anchor_content, &assistant_msg.content,
        ).await {
            Ok((Some(emoji), usage)) => {
                if let Some(u) = usage {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "reaction", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
                }
                let reaction = Reaction {
                    reaction_id: uuid::Uuid::new_v4().to_string(),
                    message_id: user_message.message_id.clone(),
                    emoji,
                    reactor: "assistant".to_string(),
                    created_at: Utc::now().to_rfc3339(),
                };
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                let _ = add_reaction(&conn, &reaction);
                ai_reactions.push(reaction);
            }
            Ok((None, usage)) => {
                if let Some(u) = usage {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "reaction", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
                }
            }
            Err(e) => log::warn!("Reaction generation failed (non-fatal): {e}"),
        }

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

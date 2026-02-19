use crate::ai::mood;
use crate::ai::orchestrator::{self, TickResult};
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
    pub tick_result: Option<TickResult>,
    pub new_events: Vec<WorldEvent>,
    pub ai_reactions: Vec<Reaction>,
}

#[tauri::command]
pub async fn send_message_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
    content: String,
) -> Result<SendMessageResult, String> {
    // Phase 1: Read everything from DB, persist user message, build retrieval context
    let (world, character, thread, recent_msgs, recent_events, characters, model_config,
         retrieved, cache_key, cached_tick, should_run_maintenance, user_profile,
         current_mood, mood_enabled, mood_drift_rate) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let characters = list_characters(&conn, &world.world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);

        let user_msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread.thread_id.clone(),
            role: "user".to_string(),
            content: content.clone(),
            tokens_estimate: (content.len() as i64) / 4,
            created_at: Utc::now().to_rfc3339(),
        };
        create_message(&conn, &user_msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

        let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;
        let recent_events = list_world_events(&conn, &world.world_id, 8).map_err(|e| e.to_string())?;

        // Hybrid retrieval: FTS messages + FTS events + thread summary + hooks
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
                    log::info!("[Memory] FTS messages: {} results for {:?}", fts_msgs.len(), fts_query);
                    for m in fts_msgs {
                        retrieved.push(format!("[{}] {}: {}", m.created_at, m.role, m.content));
                    }
                }
                Err(e) => log::warn!("[Memory] FTS messages query failed: {e}"),
            }

            match search_events_fts(&conn, &world.world_id, &fts_query, 4) {
                Ok(fts_events) => {
                    log::info!("[Memory] FTS events: {} results", fts_events.len());
                    for summary_text in fts_events {
                        retrieved.push(format!("[World event] {summary_text}"));
                    }
                }
                Err(e) => log::warn!("[Memory] FTS events query failed: {e}"),
            }
        }

        let mut hook_count = 0;
        for evt in &recent_events {
            if let Some(hooks) = evt.hooks.as_array() {
                for hook in hooks {
                    if let Some(h) = hook.as_str() {
                        retrieved.push(format!("[Hook] {h}"));
                        hook_count += 1;
                    }
                }
            }
        }
        if hook_count > 0 {
            log::info!("[Memory] {} hooks from recent events", hook_count);
        }

        let cache_key = orchestrator::compute_tick_cache_key(&world, &characters, &recent_events, Some(&content));
        let cached_tick = get_tick_cache(&conn, &cache_key).ok().flatten();

        let msg_count = count_messages_since_maintenance(&conn, &thread.thread_id);
        let should_run_maintenance = msg_count >= MEMORY_MAINTENANCE_INTERVAL;

        let user_profile = get_user_profile(&conn, &character.world_id).ok();

        let current_mood = get_character_mood(&conn, &character_id);
        let mood_enabled = get_setting(&conn, "mood_drift_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        let mood_drift_rate = get_setting(&conn, "mood_drift_rate")
            .ok().flatten().and_then(|v| v.parse::<f64>().ok());

        (world, character, thread, recent_msgs, recent_events, characters, model_config,
         retrieved, cache_key, cached_tick, should_run_maintenance, user_profile,
         current_mood, mood_enabled, mood_drift_rate)
    };

    // Phase 2: Run world tick (async, no DB lock)
    let tick_result: Option<TickResult> = if cached_tick.is_some() {
        log::info!("Tick cache hit, skipping");
        None
    } else {
        let (result, usage) = orchestrator::run_world_tick(
            &api_key, &model_config.tick_model,
            &world, &characters, &recent_events, Some(&content),
        ).await?;
        if let Some(u) = usage {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = record_token_usage(&conn, "world_tick", &model_config.tick_model, u.prompt_tokens, u.completion_tokens);
        }
        Some(result)
    };

    // Phase 3: Store tick results
    let mut new_events = Vec::new();
    if let Some(ref tick) = tick_result {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = set_tick_cache(&conn, &cache_key, &serde_json::to_string(tick).unwrap_or_default());

        for event_text in &tick.events {
            let time = world.state.get("time").cloned().unwrap_or(json!({}));
            let evt = WorldEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                world_id: world.world_id.clone(),
                day_index: time.get("day_index").and_then(|v| v.as_i64()).unwrap_or(1),
                time_of_day: time.get("time_of_day").and_then(|v| v.as_str()).unwrap_or("MORNING").to_string(),
                summary: event_text.clone(),
                involved_characters: json!(characters.iter().map(|c| c.character_id.clone()).collect::<Vec<_>>()),
                hooks: json!(tick.next_hooks.clone()),
                trigger_type: "after_user_message".to_string(),
                created_at: Utc::now().to_rfc3339(),
            };
            create_world_event(&conn, &evt).map_err(|e| e.to_string())?;
            new_events.push(evt);
        }
        apply_state_patch(&conn, &world.world_id, &characters, &tick.state_patch)?;
    }

    // Phase 4: Get updated events for dialogue
    let all_events_for_dialogue = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_world_events(&conn, &world.world_id, 8).map_err(|e| e.to_string())?
    };

    // Phase 5: Vector search (if embeddings exist)
    let mut full_retrieved = retrieved;
    let has_chunks = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM chunk_metadata WHERE world_id = ?1",
            params![world.world_id], |r| r.get(0),
        ).unwrap_or(0);
        log::info!("[Memory] Vector store: {} chunks for this world", count);
        count > 0
    };

    if has_chunks {
        match orchestrator::generate_embeddings(
            &api_key, &model_config.embedding_model, vec![content.clone()]
        ).await {
            Ok((embeddings, embed_tokens)) if !embeddings.is_empty() => {
                {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "embedding", &model_config.embedding_model, embed_tokens, 0);
                }
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                match search_vectors(&conn, &world.world_id, &embeddings[0], 4) {
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

    log::info!("[Memory] Total retrieval context: {} items passed to dialogue", full_retrieved.len());

    // Phase 5b: Mood drift
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

    // Phase 6: Run dialogue (async, no DB lock)
    let (reply_text, dialogue_usage) = orchestrator::run_dialogue(
        &api_key, &model_config.dialogue_model,
        &world, &character, &recent_msgs, &all_events_for_dialogue, &full_retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
    ).await?;
    let tokens = dialogue_usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
    if let Some(u) = &dialogue_usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "dialogue", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    // Phase 7: Store assistant message + generate embeddings for new content
    let (user_message, assistant_msg) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread.thread_id.clone(),
            role: "assistant".to_string(),
            content: reply_text,
            tokens_estimate: tokens as i64,
            created_at: Utc::now().to_rfc3339(),
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

        let user_message = recent_msgs.last().cloned().unwrap_or_else(|| Message {
            message_id: String::new(), thread_id: thread.thread_id.clone(),
            role: "user".to_string(), content: content.clone(),
            tokens_estimate: 0, created_at: Utc::now().to_rfc3339(),
        });

        (user_message, msg)
    };

    // Phase 8: AI reaction to user's message (character decides whether to react)
    let mut ai_reactions: Vec<Reaction> = Vec::new();
    match orchestrator::generate_reaction(
        &api_key, &model_config.dialogue_model,
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

    // Phase 9: Background tasks — embeddings + memory maintenance (best-effort, don't block response)
    let embed_texts = vec![
        format!("user: {}", content),
        format!("{}: {}", character.display_name, assistant_msg.content),
    ];
    let embed_ids = vec![user_message.message_id.clone(), assistant_msg.message_id.clone()];

    // Collect all texts to embed in one batch: user msg, assistant msg, new events
    let mut all_embed_texts = embed_texts.clone();
    let mut all_embed_ids = embed_ids.clone();
    let mut all_embed_types: Vec<&str> = vec!["message", "message"];
    for evt in &new_events {
        all_embed_texts.push(evt.summary.clone());
        all_embed_ids.push(evt.event_id.clone());
        all_embed_types.push("world_event");
    }

    log::info!("[Memory] Generating embeddings for {} items", all_embed_texts.len());
    match orchestrator::generate_embeddings(&api_key, &model_config.embedding_model, all_embed_texts.clone()).await {
        Ok((embeddings, embed_tokens)) => {
            log::info!("[Memory] Got {} embeddings, storing in vector DB", embeddings.len());
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = record_token_usage(&conn, "embedding", &model_config.embedding_model, embed_tokens, 0);
            for (i, emb) in embeddings.iter().enumerate() {
                if i < all_embed_ids.len() {
                    match insert_vector_chunk(
                        &conn, &all_embed_ids[i], all_embed_types[i], &all_embed_ids[i],
                        &world.world_id, &all_embed_texts[i], emb,
                    ) {
                        Ok(_) => {}
                        Err(e) => log::warn!("[Memory] Failed to store chunk {}: {e}", all_embed_ids[i]),
                    }
                }
            }
        }
        Err(e) => log::warn!("[Memory] Embedding generation failed: {e}"),
    }

    // Phase 10: Memory maintenance (every N messages)
    log::info!("[Memory] Maintenance check: should_run={} (interval={})", should_run_maintenance, MEMORY_MAINTENANCE_INTERVAL);
    if should_run_maintenance {
        let summary = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_thread_summary(&conn, &thread.thread_id)
        };
        match orchestrator::run_memory_update(
            &api_key, &model_config.memory_model,
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
        tick_result,
        new_events,
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

fn apply_state_patch(
    conn: &rusqlite::Connection,
    world_id: &str,
    characters: &[Character],
    patch: &serde_json::Value,
) -> Result<(), String> {
    if let Some(obj) = patch.as_object() {
        let mut world = get_world(conn, world_id).map_err(|e| e.to_string())?;

        for (key, value) in obj {
            if key.starts_with("world.") {
                let field = &key[6..];
                if let Some(state_obj) = world.state.as_object_mut() {
                    state_obj.insert(field.to_string(), value.clone());
                }
            } else if key.starts_with("character.") {
                let rest = &key[10..];
                if let Some(dot_pos) = rest.find('.') {
                    let char_id = &rest[..dot_pos];
                    let field = &rest[dot_pos + 1..];
                    if let Some(ch) = characters.iter().find(|c| {
                        c.character_id == char_id
                            || c.display_name.to_lowercase() == char_id.to_lowercase()
                    }) {
                        let mut updated = ch.clone();
                        if let Some(state_obj) = updated.state.as_object_mut() {
                            state_obj.insert(field.to_string(), value.clone());
                        }
                        update_character(conn, &updated).map_err(|e| e.to_string())?;
                    }
                }
            }
        }

        update_world(conn, &world).map_err(|e| e.to_string())?;
    }
    Ok(())
}

use crate::ai::mood;
use crate::ai::openai;
use crate::ai::orchestrator;
use crate::ai::prompts;
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

/// Embed a message once and store vector-chunk rows for every character
/// listed in `member_character_ids`. Used for group messages so any
/// member of the group can semantically recall the exchange from their
/// own solo or other chats.
///
/// `chunk_metadata.chunk_id` has a UNIQUE constraint, so we use
/// synthetic keys `{message_id}::{character_id}` — one row per character
/// for the same underlying message. The `content` stored includes a
/// speaker-label prefix so the retrieved snippet reads as a real line
/// rather than disembodied text.
///
/// Returns the embedding vector of the content so callers who are about
/// to run vector searches can reuse it without re-embedding.
pub async fn embed_and_store_for_members(
    db: &Database,
    api_key: &str,
    model_config: &orchestrator::ModelConfig,
    world_id: &str,
    member_character_ids: &[String],
    message_id: &str,
    formatted_content: &str,
) -> Option<Vec<f32>> {
    if member_character_ids.is_empty() { return None; }
    let (embeddings, tokens) = orchestrator::generate_embeddings_with_base(
        &model_config.openai_api_base(),
        api_key,
        &model_config.embedding_model,
        vec![formatted_content.to_string()],
    ).await.ok()?;
    let embedding = embeddings.into_iter().next()?;
    {
        let Ok(conn) = db.conn.lock() else { return Some(embedding); };
        let _ = record_token_usage(&conn, "embedding", &model_config.embedding_model, tokens, 0);
        for cid in member_character_ids {
            let chunk_id = format!("{message_id}::{cid}");
            if let Err(e) = insert_vector_chunk(
                &conn, &chunk_id, "message", message_id,
                world_id, cid, formatted_content, &embedding,
            ) {
                log::warn!("[Memory] Failed to store group chunk {chunk_id}: {e}");
            }
        }
    }
    Some(embedding)
}

/// Run a vector search against an already-computed embedding and return
/// formatted memory snippets ready to push into a retrieval list.
/// Helper used by both solo and group dialogue paths. Retrieved snippets
/// carry a weathering label (vivid / softened / mostly-the-feeling /
/// almost-a-rumor) so the model reads older memories with hedged grain
/// rather than perfect recall.
pub fn vector_search_memories(
    db: &Database,
    world_id: &str,
    character_id: &str,
    embedding: &[f32],
    k: i64,
) -> Vec<String> {
    let Ok(conn) = db.conn.lock() else { return Vec::new(); };
    match search_vectors(&conn, world_id, character_id, embedding, k) {
        Ok(results) => {
            let mut out = Vec::with_capacity(results.len());
            for hit in results {
                let weather = weathering_label(&hit.created_at);
                log::info!("[Memory]   - dist={:.3} ({weather}): {:.80}", hit.distance, hit.content);
                out.push(format!("[Memory, {weather}] {}", hit.content));
            }
            out
        }
        Err(e) => {
            log::warn!("[Memory] Vector search failed: {e}");
            Vec::new()
        }
    }
}

/// Build the cross-thread continuity snippet for a character — pulls
/// their most-recent activity from each OTHER thread they're in (solo
/// if we're in group, groups if we're in solo) and formats it as a
/// labeled block ready to push into the dialogue-retrieval context.
///
/// Continuity-shaped framing: from the user's perspective, switching
/// chats IS picking up a conversation they were just in, regardless of
/// the wall-clock gap. A user might step away from a group chat and
/// return four hours later to have a private word with one of the
/// participants — narratively they "just stepped out of the room."
/// The per-block weathering labels embedded in each rendered block
/// (`vivid — just now` / `still clear — yesterday or so` / `softened
/// — days back` / etc.) carry the age signal cleanly inside the
/// block itself, so the header doesn't need to make a wall-clock call.
///
/// Returns None when the character has no other threads or no activity
/// in them.
pub fn build_cross_thread_snippet(
    db: &Database,
    character_id: &str,
    current_thread_id: &str,
    user_profile: Option<&UserProfile>,
) -> Option<String> {
    let conn = db.conn.lock().ok()?;
    let user_name = user_profile.map(|p| p.display_name.as_str()).unwrap_or("the human");
    let blocks = list_cross_thread_recent_for_character(
        &conn,
        character_id,
        current_thread_id,
        40,  // per-thread limit — preserves enough verbatim recent context
             // that the character can pick up where the conversation left off
        3,   // max other-threads pulled
        user_name,
    );
    if blocks.is_empty() {
        return None;
    }
    Some(format!(
        "PICKING UP WHERE YOU LEFT OFF — these are conversations you have actually been in elsewhere, arranged like chat history with OLDEST FIRST and MOST RECENT LAST (the conversation just below this block continues from the most-recent material right above it). Each block carries its own age tag ('vivid — just now', 'softened — days back', etc.) so you know how clearly you'd remember it. From the user's perspective, switching here from another chat is picking up a conversation they were just in — speak as someone who was just there, not as someone recalling a distant memory. When something here directly continues from another block, name it; otherwise let it stay as background.\n\n{}",
        blocks.iter().map(|b| b.rendered.clone()).collect::<Vec<_>>().join("\n\n"),
    ))
}

/// Collect caption text for any illustration messages present in the slice.
/// Used at the top of every orchestrator-calling command so the dialogue /
/// narrative / dream / proactive-ping builders can render illustration
/// turns as `[Illustration — caption]` system notes instead of dropping
/// them. Empty map is fine — builders fall back to `[Illustration shown]`.
pub fn collect_illustration_captions(
    db: &Database,
    messages: &[Message],
) -> std::collections::HashMap<String, String> {
    let ids: Vec<String> = messages.iter()
        .filter(|m| m.role == "illustration")
        .map(|m| m.message_id.clone())
        .collect();
    if ids.is_empty() {
        return std::collections::HashMap::new();
    }
    match db.conn.lock() {
        Ok(conn) => fetch_illustration_captions(&conn, &ids),
        Err(_) => std::collections::HashMap::new(),
    }
}

/// Bucket the emoji reactions on a slice of messages into a map keyed by
/// message_id. Reactions are returned chronologically (query is ORDER BY
/// created_at). Used by dialogue builders to surface each beat's reactions
/// inline so the model sees the emotional arc, not just the thread-level
/// mood aggregate.
pub fn collect_reactions_by_message(
    db: &Database,
    messages: &[Message],
) -> std::collections::HashMap<String, Vec<Reaction>> {
    let ids: Vec<String> = messages.iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| m.message_id.clone())
        .collect();
    if ids.is_empty() {
        return std::collections::HashMap::new();
    }
    let reactions = match db.conn.lock() {
        Ok(conn) => get_reactions_for_messages(&conn, &ids).unwrap_or_default(),
        Err(_) => return std::collections::HashMap::new(),
    };
    let mut by_msg: std::collections::HashMap<String, Vec<Reaction>> = std::collections::HashMap::new();
    for r in reactions {
        by_msg.entry(r.message_id.clone()).or_default().push(r);
    }
    by_msg
}

pub fn world_time_fields(world: &World) -> (Option<i64>, Option<String>) {
    let time = world.state.get("time");
    let day = time.and_then(|t| t.get("day_index")).and_then(|v| v.as_i64());
    let tod = time.and_then(|t| t.get("time_of_day")).and_then(|v| v.as_str()).map(|s| s.to_string());
    (day, tod)
}

/// Emit a character-side reaction on the user's message. The character
/// always makes the first emoji move — the user never has to bootstrap the
/// loop. The emoji is chosen via `pick_character_reaction_emoji`, which
/// prefers the thread's reduction headline (closing the feedback loop)
/// and falls back through the turn's chain to a random emotional pick.
/// Idempotent on (message, emoji, 'assistant') — safe to call repeatedly.
pub fn emit_character_reaction(
    db: &Database,
    target_message_id: &str,
    emoji: &str,
    sender_character_id: Option<&str>,
) -> Vec<Reaction> {
    if target_message_id.is_empty() {
        log::warn!("[CharReact] skip: target_message_id is empty");
        return Vec::new();
    }
    if emoji.is_empty() {
        log::warn!("[CharReact] skip: emoji is empty (LLM returned nothing and fallback was empty)");
        return Vec::new();
    }
    let Ok(conn) = db.conn.lock() else {
        log::warn!("[CharReact] skip: db lock poisoned");
        return Vec::new();
    };
    // Dedupe per-character so Alice's 🥺 and Bob's 🥺 on the same message
    // are both preserved. Rows with NULL sender still dedupe amongst
    // themselves to keep solo/legacy idempotency intact.
    let dup: Option<String> = match sender_character_id {
        Some(cid) => conn.query_row(
            "SELECT reaction_id FROM reactions WHERE message_id = ?1 AND emoji = ?2 AND reactor = 'assistant' AND sender_character_id = ?3",
            rusqlite::params![target_message_id, emoji, cid],
            |r| r.get(0),
        ).ok(),
        None => conn.query_row(
            "SELECT reaction_id FROM reactions WHERE message_id = ?1 AND emoji = ?2 AND reactor = 'assistant' AND sender_character_id IS NULL",
            rusqlite::params![target_message_id, emoji],
            |r| r.get(0),
        ).ok(),
    };
    if dup.is_some() {
        return Vec::new();
    }
    let r = Reaction {
        reaction_id: uuid::Uuid::new_v4().to_string(),
        message_id: target_message_id.to_string(),
        emoji: emoji.to_string(),
        reactor: "assistant".to_string(),
        created_at: Utc::now().to_rfc3339(),
        sender_character_id: sender_character_id.map(|s| s.to_string()),
    };
    match add_reaction(&conn, &r) {
        Ok(()) => vec![r],
        Err(e) => {
            log::warn!("[CharReact] add_reaction failed for ({}, {}, 'assistant'): {}",
                target_message_id, emoji, e);
            Vec::new()
        }
    }
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
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
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
         current_mood, mood_enabled, mood_drift_rate, response_length, narration_tone, leader,
         reactions_mode) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", character_id));

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
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
        };
        create_message(&conn, &user_msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

        let recent_msgs = list_messages_within_budget(&conn, &thread.thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?;

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
                        .filter(|m| m.role != "illustration" && m.role != "video" && m.role != "inventory_update")
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

        // Default to "Short" when no setting exists — matches the
        // frontend default (use-chat-state.ts:128) so the UI's
        // displayed value and the LLM's actual constraint stay in
        // sync. Without this fallback, opening a chat for the first
        // time shows "Short" in the UI but injects no length
        // directive into the prompt; replies come back unconstrained.
        // Group-chat path (group_chat_cmds.rs) already has this
        // fallback; solo path was missing it.
        let response_length = get_setting(&conn, &format!("response_length.{}", character_id))
            .ok().flatten()
            .or_else(|| Some("Short".to_string()));
        let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
            .ok().flatten();
        let leader = get_setting(&conn, &format!("leader.{}", character_id)).ok().flatten();
        // Per-chat reactions setting. Three modes: "off" | "occasional"
        // | "always". Default OFF (per the persona-sim convergence —
        // see commit a8a7b0c). "occasional" produces realistic-text-
        // message-feeling reactions on ~25% of user messages: the LLM
        // emoji-picker is given a budget and decides per-moment whether
        // to skip (looking at recent reactions in chat history to self-
        // pace). NOT deterministic-skip in code — that approach was
        // tried and rejected per Ryan's calibration note. See
        // reactions_helpers.rs for parsing; orchestrator.rs::
        // pick_character_reaction_via_llm for the LLM-side calibration.
        let reactions_mode = crate::commands::reactions_helpers::parse_reactions_mode(
            get_setting(&conn, &format!("reactions_enabled.{}", character_id))
                .ok().flatten().as_deref()
        ).to_string();

        (world, character, thread, recent_msgs, model_config,
         retrieved, should_run_maintenance, user_profile,
         current_mood, mood_enabled, mood_drift_rate, response_length, narration_tone, leader,
         reactions_mode)
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
                            for hit in results {
                                let weather = weathering_label(&hit.created_at);
                                log::info!("[Memory]   - dist={:.3} ({weather}): {:.80}", hit.distance, hit.content);
                                full_retrieved.push(format!("[Memory, {weather}] {}", hit.content));
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

    if let Some(ct) = build_cross_thread_snippet(&db, &character_id, &thread.thread_id, user_profile.as_ref()) {
        full_retrieved.push(ct);
    }

    log::info!("[Memory] Total retrieval context: {} items passed to dialogue", full_retrieved.len());

    // Phase 3: Mood drift
    let mood_directive = compute_and_persist_mood(
        &db, &character_id, &world, &character, &recent_msgs,
        current_mood.as_ref(), mood_enabled, mood_drift_rate,
    )?;

    // Phase 4: Run dialogue. Load the thread's mood_reduction and pick the
    // 5-emoji chain up front so we can both seed AGENCY and persist it on
    // the resulting assistant message for the measurement loop.
    let mood_reduction = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_thread_mood_reduction(&conn, &thread.thread_id)
    };
    let mood_chain = prompts::pick_mood_chain(Some(&mood_reduction));
    let mood_chain_json = serde_json::to_string(&mood_chain).ok();

    // Phase 4a+b: Run dialogue generation and the character's emoji-reaction
    // pick CONCURRENTLY. They're independent (both just need the user's
    // content + mood state), so serial awaiting doubles latency for no
    // reason. tokio::join! gives max(reply, reaction) instead of
    // reply + reaction — meaningful on local models where each call can
    // run many seconds.
    let base = model_config.chat_api_base();
    let kept_ids: Vec<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_kept_message_ids_for_thread(&conn, &thread.thread_id).unwrap_or_default()
    };
    let illustration_captions = collect_illustration_captions(&db, &recent_msgs);
    let reactions_by_msg = collect_reactions_by_message(&db, &recent_msgs);
    let send_history = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, &format!("send_history.{}", character_id))
            .ok().flatten()
            .map(|v| v != "off" && v != "false")
            .unwrap_or(true)
    };
    let recent_journals = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_journal_entries(&conn, &character.character_id, 2).unwrap_or_default()
    };
    let latest_reading = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_daily_readings(&conn, &character.world_id, 1).unwrap_or_default().into_iter().next()
    };
    let latest_meanwhile = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        latest_meanwhile_for_character(&conn, &character.character_id, 24)
    };
    let active_quests = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_active_quests(&conn, &character.world_id).unwrap_or_default()
    };
    // Load this character's most recent relational stance for prompt
    // injection. Behind-the-scenes: ambient awareness of who the user
    // has become to them. Never surfaced to the UI. See
    // ai/relational_stance.rs for the synthesis pipeline.
    let latest_stance = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        latest_relational_stance(&conn, &character.character_id).unwrap_or(None)
    };
    let stance_text: Option<String> = latest_stance.as_ref().map(|s| s.stance_text.clone());
    let anchor_text: Option<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        combined_axes_block(&conn, &character.character_id)
    };
    // Detect "first message of new in-world day" for this character —
    // current world_day greater than the day the latest stance was
    // generated against (or no stance at all → bootstrap).
    let current_world_day_for_stance: Option<i64> = recent_msgs.iter().rev()
        .find_map(|m| m.world_day);
    let stance_needs_refresh = match (latest_stance.as_ref(), current_world_day_for_stance) {
        (None, _) => true,
        (Some(s), Some(today)) => s.world_day_at_generation.map(|d| today > d).unwrap_or(true),
        (Some(_), None) => false,
    };
    if stance_needs_refresh {
        crate::ai::relational_stance::spawn_stance_refresh(
            db.conn.clone(),
            base.clone(),
            api_key.clone(),
            model_config.memory_model.clone(),
            character.character_id.clone(),
            "first_message_new_day".to_string(),
        );
    }

    // Reactions=off as depth-signal reward: build a Formula momentstamp
    // when the user has chosen quiet over reactive surface. The signature
    // gets injected at the head of the dialogue system prompt so the
    // character is conditioned on where this chat sits in 𝓕 := (𝓡, 𝓒)
    // right now. Cost ~$0.005-0.015 per dialogue call; conceptually the
    // saved emoji-reaction budget redirects into this deeper-attention
    // call. Silent skip on any failure (never blocks the dialogue).
    //
    // Stateful chain: read the LATEST formula_signature from prior
    // assistant messages in this thread and pass as prior_signature so
    // the new signature CHAINS from the prior one (running cumulation,
    // not recompute-from-scratch).
    let (formula_momentstamp_text, formula_momentstamp_signature): (Option<String>, Option<String>) = if reactions_mode == "off" {
        let prior_sig: Option<String> = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            crate::db::queries::latest_formula_signature(&conn, &thread.thread_id).ok().flatten()
        };
        match crate::ai::momentstamp::build_formula_momentstamp(
            &base,
            &api_key,
            &model_config.memory_model,
            &recent_msgs,
            prior_sig.as_deref(),
            Some(&character),
        ).await.ok().flatten() {
            Some(r) => (Some(r.block), Some(r.signature)),
            None => (None, None),
        }
    } else {
        (None, None)
    };

    let current_loc = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_thread_location(&conn, &thread.thread_id).ok().flatten()
    };
    let dialogue_fut = orchestrator::run_dialogue_with_base(
        &base, &api_key, &model_config.dialogue_model,
        if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
        send_history,
        &world, &character, &recent_msgs, &full_retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        response_length.as_deref(),
        None, None, narration_tone.as_deref(),
        model_config.is_local(),
        &mood_chain,
    leader.as_deref(),
    &kept_ids,
    &illustration_captions,
    &reactions_by_msg,
    None,
    &recent_journals,
    latest_reading.as_ref(),
    latest_meanwhile.as_ref(),
    active_quests.as_slice(),
    stance_text.as_deref(),
    anchor_text.as_deref(),
    current_loc.as_deref(),
    formula_momentstamp_text.as_deref(),
    );
    // Context for the reaction-emoji pick: the recent messages EXCLUDING
    // the user's brand-new one (which goes in the user-role slot). Gives
    // the picker scene register so it doesn't read the latest message in
    // isolation. The reaction LLM call is gated on the per-chat
    // reactions_enabled setting — when off, we skip the call entirely
    // (cost + latency saving, not just UI hiding).
    let reaction_context: Vec<Message> = recent_msgs.iter()
        .rev().skip(1).take(4).rev().cloned().collect();
    let (dialogue_res, reaction_res) = if reactions_mode != "off" {
        let reaction_fut = orchestrator::pick_character_reaction_via_llm(
            &base, &api_key, &model_config.dialogue_model,
            &content, &mood_reduction, &reaction_context, &reactions_mode,
        );
        tokio::join!(dialogue_fut, reaction_fut)
    } else {
        (dialogue_fut.await, Ok(None))
    };
    let (mut reply_text, mut dialogue_usage) = dialogue_res?;

    // Phase 4c: Conscience Pass. Grade the draft against the five
    // compile-time invariants via a cheap memory_model call. On drift,
    // regenerate once with the correction note injected. Default on,
    // gated by `conscience_pass_enabled` setting. Non-fatal on grader
    // transport errors — we never block delivery on the grader itself.
    let conscience_enabled = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, "conscience_pass_enabled")
            .ok().flatten()
            .map(|v| v == "true" || v == "on")
            .unwrap_or(false)
    };
    if conscience_enabled {
        let user_last = recent_msgs.iter().rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str()).unwrap_or(content.as_str());
        match crate::ai::conscience::grade_reply(
            &base, &api_key, &model_config.memory_model,
            &character, user_last, &reply_text,
        ).await {
            Ok(verdict) => {
                if let Some(u) = &verdict.usage {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "conscience", &model_config.memory_model, u.prompt_tokens, u.completion_tokens);
                }
                if !verdict.passed {
                    log::warn!(
                        "[Conscience] {} draft flagged: {:?}",
                        character.display_name,
                        verdict.failures,
                    );
                    if let Some(note) = crate::ai::conscience::build_correction_note(&verdict) {
                        match orchestrator::run_dialogue_with_base(
                            &base, &api_key, &model_config.dialogue_model,
                            if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
                            send_history,
                            &world, &character, &recent_msgs, &full_retrieved,
                            user_profile.as_ref(),
                            mood_directive.as_deref(),
                            response_length.as_deref(),
                            None, None, narration_tone.as_deref(),
                            model_config.is_local(),
                            &mood_chain,
                            leader.as_deref(),
                            &kept_ids,
                            &illustration_captions,
                            &reactions_by_msg,
                            Some(&note),
                            &recent_journals,
                            latest_reading.as_ref(),
                            latest_meanwhile.as_ref(),
                            active_quests.as_slice(),
                            stance_text.as_deref(),
                            anchor_text.as_deref(),
                        current_loc.as_deref(),
                        formula_momentstamp_text.as_deref(),
                        ).await {
                            Ok((corrected, corrected_usage)) => {
                                log::info!("[Conscience] {} reply corrected after drift", character.display_name);
                                reply_text = corrected;
                                dialogue_usage = corrected_usage;
                            }
                            Err(e) => {
                                log::warn!("[Conscience] regeneration failed, keeping original draft: {e}");
                            }
                        }
                    }
                } else {
                    log::info!("[Conscience] {} draft passed", character.display_name);
                }
            }
            Err(e) => {
                log::warn!("[Conscience] grader unavailable, passing draft through: {e}");
            }
        }
    }

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
            address_to: None,
            mood_chain: mood_chain_json.clone(),
            is_proactive: false,
            formula_signature: formula_momentstamp_signature.clone(),
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;

        let user_message = recent_msgs.last().cloned().unwrap_or_else(|| Message {
            message_id: String::new(), thread_id: thread.thread_id.clone(),
            role: "user".to_string(), content: content.clone(),
            tokens_estimate: 0, created_at: Utc::now().to_rfc3339(),
            world_day: None, world_time: None,
            sender_character_id: None,
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
        });

        (user_message, msg)
    };

    // Phase 6: Character-side reaction. Three outcomes per the three
    // reactions modes:
    //   - "off"        → no LLM call was made; no emit.
    //   - "occasional" → LLM may have intentionally skipped (Ok(None));
    //                    if so, no emit. If it picked an emoji, emit it.
    //                    If LLM call errored, no fallback emoji (occasional
    //                    is permissive about skipping).
    //   - "always"     → LLM call was made; if it returned an emoji emit
    //                    it; if it errored fall back to the deterministic
    //                    chain-based pick (preserves prior "always-emit"
    //                    behavior).
    let ai_reactions: Vec<Reaction> = match reactions_mode.as_str() {
        "off" => Vec::new(),
        "occasional" => match reaction_res {
            Ok(Some(emoji)) => emit_character_reaction(&db, &user_message.message_id, &emoji, Some(&character.character_id)),
            _ => Vec::new(),
        },
        _ /* always */ => {
            let reaction_emoji = match reaction_res {
                Ok(Some(emoji)) => emoji,
                _ => prompts::pick_character_reaction_emoji(&mood_chain),
            };
            emit_character_reaction(&db, &user_message.message_id, &reaction_emoji, Some(&character.character_id))
        }
    };

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

    // Auto-derivation refresh: fire-and-forget background tokio::spawn
    // for character + user-in-world + world derivations. Hybrid OR
    // staleness policy (per src/ai/derivation.rs). INFLIGHT dedupe
    // prevents racing refreshes when consecutive turns both find an
    // entity stale. Silent on missing API key; non-blocking; never
    // surfaces to user. See reports/2026-04-26-2030 + design consult
    // at /tmp/derivation-design-response.json for the architecture.
    crate::ai::derivation::maybe_refresh_after_turn(
        db.conn.clone(),
        model_config.chat_api_base(),
        api_key.clone(),
        model_config.memory_model.clone(),
        world.world_id.clone(),
        Some(character.character_id.clone()),
    ).await;

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
///
/// If the message has a corresponding `world_images` entry (i.e., it
/// was an illustration), the gallery row, the image file on disk, and
/// any animation video file are also cleaned up. Without this, the
/// gallery and the sticky-illustration thumbnail keep showing the
/// "deleted" image because they both read from world_images, not the
/// messages table.
#[tauri::command]
pub fn delete_message_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, crate::commands::portrait_cmds::PortraitsDir>,
    message_id: String,
    is_group: bool,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let table = if is_group { "group_messages" } else { "messages" };
    let fts_table = if is_group { "group_messages_fts" } else { "messages_fts" };

    // Clean up world_images entry + file(s) on disk if this message had
    // an illustration attached. world_images is keyed by image_id =
    // message_id so this works for both individual + group illustrations.
    let video_file: Option<String> = conn.query_row(
        "SELECT video_file FROM world_images WHERE image_id = ?1",
        rusqlite::params![message_id], |r| r.get(0),
    ).ok();
    let file_name: Option<String> = conn.query_row(
        "SELECT file_name FROM world_images WHERE image_id = ?1",
        rusqlite::params![message_id], |r| r.get(0),
    ).ok();
    conn.execute("DELETE FROM world_images WHERE image_id = ?1", rusqlite::params![message_id]).ok();
    if let Some(ref vf) = video_file {
        if !vf.is_empty() {
            let p = portraits_dir.0.join(vf);
            if p.exists() { let _ = std::fs::remove_file(&p); }
        }
    }
    if let Some(ref f) = file_name {
        let p = portraits_dir.0.join(f);
        if p.exists() { let _ = std::fs::remove_file(&p); }
    }

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
         user_profile, current_mood, mood_enabled, response_length, narration_tone, leader) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", character_id));
        let recent_msgs = list_messages_within_budget(&conn, &thread.thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?;

        let mut retrieved: Vec<String> = Vec::new();
        let summary = get_thread_summary(&conn, &thread.thread_id);
        if !summary.is_empty() {
            retrieved.push(format!("[Thread summary] {summary}"));
        }

        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let current_mood = get_character_mood(&conn, &character_id);
        let mood_enabled = get_setting(&conn, "mood_drift_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        // Default to "Short" when no setting exists — matches the
        // frontend default (use-chat-state.ts:128) so the UI's
        // displayed value and the LLM's actual constraint stay in
        // sync. Without this fallback, opening a chat for the first
        // time shows "Short" in the UI but injects no length
        // directive into the prompt; replies come back unconstrained.
        // Group-chat path (group_chat_cmds.rs) already has this
        // fallback; solo path was missing it.
        let response_length = get_setting(&conn, &format!("response_length.{}", character_id))
            .ok().flatten()
            .or_else(|| Some("Short".to_string()));
        let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
            .ok().flatten();
        let leader = get_setting(&conn, &format!("leader.{}", character_id)).ok().flatten();

        (world, character, thread, recent_msgs, model_config, retrieved,
         user_profile, current_mood, mood_enabled, response_length, narration_tone, leader)
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
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
        });
    }

    // Dialogue — load mood_reduction + pick chain for AGENCY seed.
    let mood_reduction = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_thread_mood_reduction(&conn, &thread.thread_id)
    };
    let mood_chain = prompts::pick_mood_chain(Some(&mood_reduction));
    let mood_chain_json = serde_json::to_string(&mood_chain).ok();

    let kept_ids: Vec<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_kept_message_ids_for_thread(&conn, &thread.thread_id).unwrap_or_default()
    };
    let mut retrieved = retrieved;
    if let Some(ct) = build_cross_thread_snippet(&db, &character_id, &thread.thread_id, user_profile.as_ref()) {
        retrieved.push(ct);
    }
    let illustration_captions = collect_illustration_captions(&db, &dialogue_msgs);
    let reactions_by_msg = collect_reactions_by_message(&db, &dialogue_msgs);
    let send_history = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, &format!("send_history.{}", character_id))
            .ok().flatten()
            .map(|v| v != "off" && v != "false")
            .unwrap_or(true)
    };
    let recent_journals = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_journal_entries(&conn, &character.character_id, 2).unwrap_or_default()
    };
    let latest_reading = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_daily_readings(&conn, &character.world_id, 1).unwrap_or_default().into_iter().next()
    };
    let latest_meanwhile = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        latest_meanwhile_for_character(&conn, &character.character_id, 24)
    };
    let active_quests = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_active_quests(&conn, &character.world_id).unwrap_or_default()
    };
    let latest_stance = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        latest_relational_stance(&conn, &character.character_id).unwrap_or(None)
    };
    let stance_text: Option<String> = latest_stance.as_ref().map(|s| s.stance_text.clone());
    let anchor_text: Option<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        combined_axes_block(&conn, &character.character_id)
    };
    let current_world_day_for_stance: Option<i64> = dialogue_msgs.iter().rev()
        .find_map(|m| m.world_day);
    let stance_needs_refresh = match (latest_stance.as_ref(), current_world_day_for_stance) {
        (None, _) => true,
        (Some(s), Some(today)) => s.world_day_at_generation.map(|d| today > d).unwrap_or(true),
        (Some(_), None) => false,
    };
    if stance_needs_refresh {
        crate::ai::relational_stance::spawn_stance_refresh(
            db.conn.clone(),
            model_config.chat_api_base(),
            api_key.clone(),
            model_config.memory_model.clone(),
            character.character_id.clone(),
            "first_message_new_day".to_string(),
        );
    }
    let current_loc = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_thread_location(&conn, &thread.thread_id).ok().flatten()
    };
    let (mut reply_text, mut dialogue_usage) = orchestrator::run_dialogue_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
        send_history,
        &world, &character, &dialogue_msgs, &retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        response_length.as_deref(),
        None, None, narration_tone.as_deref(),
        model_config.is_local(),
        &mood_chain,
        leader.as_deref(),
        &kept_ids,
        &illustration_captions,
        &reactions_by_msg,
        None,
        &recent_journals,
        latest_reading.as_ref(),
        latest_meanwhile.as_ref(),
        active_quests.as_slice(),
        stance_text.as_deref(),
        anchor_text.as_deref(),
    current_loc.as_deref(),
    None, // formula_momentstamp
    ).await?;

    // Conscience Pass: grade + regenerate-on-drift (see send_message_cmd).
    let conscience_enabled = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, "conscience_pass_enabled")
            .ok().flatten()
            .map(|v| v == "true" || v == "on")
            .unwrap_or(false)
    };
    if conscience_enabled {
        let user_last = dialogue_msgs.iter().rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.as_str()).unwrap_or("");
        match crate::ai::conscience::grade_reply(
            &model_config.chat_api_base(), &api_key, &model_config.memory_model,
            &character, user_last, &reply_text,
        ).await {
            Ok(verdict) => {
                if let Some(u) = &verdict.usage {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "conscience", &model_config.memory_model, u.prompt_tokens, u.completion_tokens);
                }
                if !verdict.passed {
                    log::warn!("[Conscience] {} (prompt) draft flagged: {:?}", character.display_name, verdict.failures);
                    if let Some(note) = crate::ai::conscience::build_correction_note(&verdict) {
                        match orchestrator::run_dialogue_with_base(
                            &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
                            if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
                            send_history,
                            &world, &character, &dialogue_msgs, &retrieved,
                            user_profile.as_ref(),
                            mood_directive.as_deref(),
                            response_length.as_deref(),
                            None, None, narration_tone.as_deref(),
                            model_config.is_local(),
                            &mood_chain,
                            leader.as_deref(),
                            &kept_ids,
                            &illustration_captions,
                            &reactions_by_msg,
                            Some(&note),
                            &recent_journals,
                            latest_reading.as_ref(),
                            latest_meanwhile.as_ref(),
                            active_quests.as_slice(),
                            stance_text.as_deref(),
                            anchor_text.as_deref(),
                        current_loc.as_deref(),
                        None, // formula_momentstamp
                        ).await {
                            Ok((corrected, corrected_usage)) => {
                                log::info!("[Conscience] {} (prompt) reply corrected after drift", character.display_name);
                                reply_text = corrected;
                                dialogue_usage = corrected_usage;
                            }
                            Err(e) => log::warn!("[Conscience] prompt regeneration failed, keeping original: {e}"),
                        }
                    }
                } else {
                    log::info!("[Conscience] {} (prompt) draft passed", character.display_name);
                }
            }
            Err(e) => log::warn!("[Conscience] grader unavailable: {e}"),
        }
    }

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
            address_to: None,
            mood_chain: mood_chain_json.clone(),
            is_proactive: false,
            formula_signature: None,
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

// ─── Proactive Pings ────────────────────────────────────────────────────────
//
// The character reaches out first — unsolicited — between user turns. Caller
// invokes this on app focus / a loose background cadence. We evaluate a
// conservative eligibility gate and either fire a ping or return None.
//
// Hard rule: no more than 2 consecutive pings without a user reply. The
// counter lives on the thread row and resets automatically when any user
// message is inserted (see create_message).
//
// Additional cooldowns kept here rather than in the schema so they stay
// tunable in one place:
//   - QUIET_AFTER_USER_MSG_SECS: don't ping right after a user message.
//     Protects "we just said goodbye" / "you're mid-conversation" cases.
//   - MIN_GAP_BETWEEN_PINGS_SECS: cooldown between pings themselves, so
//     even with the counter at 0 we don't stack two in a short window.

const QUIET_AFTER_USER_MSG_SECS: i64 = 45 * 60;
const MAX_CONSECUTIVE_PROACTIVE_PINGS: i64 = 2;

// Randomized cooldown between pings. The required gap is re-rolled on each
// check from [HARD_FLOOR, MAX_ROLL]. Because the sweep runs often (focus
// events + every 60s), the effect is a probabilistic release — the second
// ping can never land inside HARD_FLOOR, and is increasingly likely to
// land as elapsed time approaches MAX_ROLL. This prevents the "two nearly
// identical messages in quick succession" failure mode.
const PING_COOLDOWN_HARD_FLOOR_SECS: i64 = 4 * 60 * 60;
const PING_COOLDOWN_MAX_ROLL_SECS: i64 = 14 * 60 * 60;

/// Draw a random required gap (in seconds) for the next proactive ping on
/// a thread. Uniform over [HARD_FLOOR, MAX_ROLL]. Time-seeded so each
/// sweep gets a fresh roll.
fn roll_ping_cooldown_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E3779B97F4A7C15);
    let mut state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;
    let mixed = state.wrapping_mul(0x2545F4914F6CDD1D);
    let span = (PING_COOLDOWN_MAX_ROLL_SECS - PING_COOLDOWN_HARD_FLOOR_SECS).max(1) as u64;
    PING_COOLDOWN_HARD_FLOOR_SECS + (mixed % span) as i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProactivePingResult {
    pub message: Option<Message>,
    /// Human-readable reason when no ping was fired (for debugging / UI
    /// tooltips). Always populated when `message` is None.
    pub skipped_reason: Option<String>,
}

#[tauri::command]
pub async fn try_proactive_ping_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
) -> Result<ProactivePingResult, String> {
    // ── Eligibility + context load (all under one lock) ─────────────────
    let loaded = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        // Solo-only for v1. Group threads have no consecutive-ping counter
        // semantics and their orchestration path is different.
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;

        // Must have a prior user message, or there's nothing to "reach back
        // about." Prevents cold-ping on a virgin thread.
        let has_prior_user: bool = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE thread_id = ?1 AND role = 'user'",
            rusqlite::params![thread.thread_id], |r| r.get::<_, i64>(0),
        ).unwrap_or(0) > 0;
        if !has_prior_user {
            return Ok(ProactivePingResult {
                message: None,
                skipped_reason: Some("no prior user message in thread".to_string()),
            });
        }

        // Hard counter cap (the user's load-bearing "max 2 in a row" rule).
        let state = get_proactive_ping_state(&conn, &thread.thread_id);
        if state.consecutive >= MAX_CONSECUTIVE_PROACTIVE_PINGS {
            return Ok(ProactivePingResult {
                message: None,
                skipped_reason: Some("max consecutive pings reached".to_string()),
            });
        }

        // Quiet window after the last user message. Don't ping mid-conversation.
        let last_user_at: Option<String> = conn.query_row(
            "SELECT MAX(created_at) FROM messages WHERE thread_id = ?1 AND role = 'user'",
            rusqlite::params![thread.thread_id], |r| r.get(0),
        ).unwrap_or(None);
        let now = chrono::Utc::now();
        if let Some(ts) = last_user_at.as_deref() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                let since = now.signed_duration_since(dt.with_timezone(&chrono::Utc)).num_seconds();
                if since < QUIET_AFTER_USER_MSG_SECS {
                    return Ok(ProactivePingResult {
                        message: None,
                        skipped_reason: Some(format!("{}s since last user msg (min {}s)", since, QUIET_AFTER_USER_MSG_SECS)),
                    });
                }
            }
        }

        // Randomized cooldown between pings. Each check rolls a fresh
        // required gap from [HARD_FLOOR, MAX_ROLL]. Guarantees no two
        // pings land within HARD_FLOOR, and spreads second-of-two pings
        // across a window so they can't arrive back-to-back.
        if let Some(ts) = state.last_at.as_deref() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                let since = now.signed_duration_since(dt.with_timezone(&chrono::Utc)).num_seconds();
                let required = roll_ping_cooldown_secs();
                if since < required {
                    return Ok(ProactivePingResult {
                        message: None,
                        skipped_reason: Some(format!("{}s since last ping (rolled min {}s)", since, required)),
                    });
                }
            }
        }

        // All gates passed — load context for the LLM call.
        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", character_id));
        let recent_msgs = list_messages_within_budget(&conn, &thread.thread_id, model_config.safe_history_budget() as i64, 30)
            .map_err(|e| e.to_string())?;

        let mut retrieved: Vec<String> = Vec::new();
        let summary = get_thread_summary(&conn, &thread.thread_id);
        if !summary.is_empty() {
            retrieved.push(format!("[Thread summary] {summary}"));
        }

        let user_profile = get_user_profile(&conn, &character.world_id).ok();

        // Pull this character's recent activity in their OTHER threads
        // (the group chats they participate in). The proactive-ping
        // continuation register depends on the character's full inner
        // life with the user — solo + group — not just the solo
        // surface. list_cross_thread_recent_for_character excludes
        // current_thread_id (= solo here), so what we pick up is each
        // group chat the character is in, formatted as a labeled
        // snippet block.
        {
            let user_name_for_blocks = user_profile
                .as_ref()
                .map(|p| p.display_name.as_str())
                .unwrap_or("the human");
            let cross_blocks = crate::db::queries::list_cross_thread_recent_for_character(
                &conn,
                &character_id,
                &thread.thread_id,
                12, // per-thread limit
                3,  // max other threads
                user_name_for_blocks,
            );
            for block in cross_blocks {
                retrieved.push(block.rendered);
            }
        }

        let current_mood = get_character_mood(&conn, &character_id);
        let mood_enabled = get_setting(&conn, "mood_drift_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
            .ok().flatten();
        let mood_reduction = get_thread_mood_reduction(&conn, &thread.thread_id);
        let kept_ids = list_kept_message_ids_for_thread(&conn, &thread.thread_id).unwrap_or_default();

        // Mirror dialogue-path context: last couple of journals and the
        // most recent world reading. Without these, pings reach from
        // nowhere — they can't feel continuous with the character's
        // interior life.
        let recent_journals = list_journal_entries(&conn, &character.character_id, 2)
            .unwrap_or_default();
        let latest_reading = list_daily_readings(&conn, &character.world_id, 1)
            .unwrap_or_default().into_iter().next();
        let latest_meanwhile = latest_meanwhile_for_character(&conn, &character.character_id, 24);
        let active_quests = list_active_quests(&conn, &character.world_id).unwrap_or_default();
        let latest_stance = latest_relational_stance(&conn, &character.character_id).unwrap_or(None);

        // Elapsed-hint string is resolved while we still hold last_user_at.
        let elapsed_hint = last_user_at.as_deref().and_then(|ts| {
            chrono::DateTime::parse_from_rfc3339(ts).ok().map(|dt| {
                let mins = now.signed_duration_since(dt.with_timezone(&chrono::Utc)).num_minutes();
                if mins < 120 {
                    format!("About {mins} minutes have passed since they last wrote.")
                } else {
                    let hours = (mins as f64) / 60.0;
                    format!("About {hours:.0} hours have passed since they last wrote.")
                }
            })
        });

        Loaded {
            world, character, thread, recent_msgs, model_config, retrieved,
            user_profile, current_mood, mood_enabled, narration_tone,
            mood_reduction, kept_ids, elapsed_hint,
            recent_journals, latest_reading, latest_meanwhile, active_quests,
            latest_stance,
        }
    };

    let Loaded {
        world, character, thread, recent_msgs, model_config, retrieved,
        user_profile, current_mood, mood_enabled, narration_tone,
        mood_reduction, kept_ids, elapsed_hint,
        recent_journals, latest_reading, latest_meanwhile, active_quests,
        latest_stance,
    } = loaded;

    let mood_directive = compute_and_persist_mood(
        &db, &character_id, &world, &character, &recent_msgs,
        current_mood.as_ref(), mood_enabled, None,
    )?;

    let mood_chain = prompts::pick_mood_chain(Some(&mood_reduction));
    let mood_chain_json = serde_json::to_string(&mood_chain).ok();

    let illustration_captions = collect_illustration_captions(&db, &recent_msgs);
    let reactions_by_msg = collect_reactions_by_message(&db, &recent_msgs);
    let stance_text: Option<String> = latest_stance.as_ref().map(|s| s.stance_text.clone());
    let anchor_text: Option<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        combined_axes_block(&conn, &character.character_id)
    };
    let current_loc = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_thread_location(&conn, &thread.thread_id).ok().flatten()
    };
    let current_world_day_for_stance: Option<i64> = recent_msgs.iter().rev()
        .find_map(|m| m.world_day);
    let stance_needs_refresh = match (latest_stance.as_ref(), current_world_day_for_stance) {
        (None, _) => true,
        (Some(s), Some(today)) => s.world_day_at_generation.map(|d| today > d).unwrap_or(true),
        (Some(_), None) => false,
    };
    if stance_needs_refresh {
        crate::ai::relational_stance::spawn_stance_refresh(
            db.conn.clone(),
            model_config.chat_api_base(),
            api_key.clone(),
            model_config.memory_model.clone(),
            character.character_id.clone(),
            "first_message_new_day".to_string(),
        );
    }
    let (reply_text, usage) = orchestrator::run_proactive_ping_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &recent_msgs, &retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        narration_tone.as_deref(),
        model_config.is_local(),
        &mood_chain,
        &kept_ids,
        elapsed_hint.as_deref(),
        &illustration_captions,
        &reactions_by_msg,
        &recent_journals,
        latest_reading.as_ref(),
        latest_meanwhile.as_ref(),
        active_quests.as_slice(),
        stance_text.as_deref(),
        anchor_text.as_deref(),
        current_loc.as_deref(),
    ).await?;

    if reply_text.trim().is_empty() {
        return Ok(ProactivePingResult {
            message: None,
            skipped_reason: Some("model returned empty response".to_string()),
        });
    }

    let tokens = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
    let (wd, wt) = world_time_fields(&world);
    let now_iso = Utc::now().to_rfc3339();
    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread.thread_id.clone(),
        role: "assistant".to_string(),
        content: reply_text,
        tokens_estimate: tokens as i64,
        sender_character_id: None,
        created_at: now_iso.clone(),
        world_day: wd, world_time: wt,
        address_to: None,
        mood_chain: mood_chain_json,
        is_proactive: true,
        formula_signature: None,
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
        increment_message_counter(&conn, &thread.thread_id).map_err(|e| e.to_string())?;
        record_proactive_ping(&conn, &thread.thread_id, &now_iso).map_err(|e| e.to_string())?;
    }

    Ok(ProactivePingResult {
        message: Some(msg),
        skipped_reason: None,
    })
}

struct Loaded {
    world: World,
    character: Character,
    thread: Thread,
    recent_msgs: Vec<Message>,
    model_config: orchestrator::ModelConfig,
    retrieved: Vec<String>,
    user_profile: Option<UserProfile>,
    current_mood: Option<CharacterMood>,
    mood_enabled: bool,
    narration_tone: Option<String>,
    mood_reduction: Vec<String>,
    kept_ids: Vec<String>,
    elapsed_hint: Option<String>,
    recent_journals: Vec<JournalEntry>,
    latest_reading: Option<DailyReading>,
    latest_meanwhile: Option<MeanwhileEvent>,
    active_quests: Vec<Quest>,
    latest_stance: Option<RelationalStance>,
}

/// Returns per-character unread-proactive-ping counts (solo threads only,
/// keyed by character_id). Used by the sidebar to render a small badge on
/// chats where the character reached out and the user hasn't replied yet.
/// Group threads are excluded — proactive pings are solo-only for v1.
#[tauri::command]
pub async fn get_proactive_unread_counts_cmd(
    db: State<'_, Database>,
) -> Result<std::collections::HashMap<String, i64>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT thread_id, character_id FROM threads WHERE character_id IS NOT NULL")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    let mut out = std::collections::HashMap::new();
    for (tid, cid) in rows {
        let c = count_unread_proactive_since_last_user(&conn, &tid);
        if c > 0 {
            out.insert(cid, c);
        }
    }
    Ok(out)
}

// ─── Dream Journal ──────────────────────────────────────────────────────────
//
// A character's dream: a dense condensation of recent story-material,
// rendered sideways as dream-imagery. Persisted as a "dream"-role message
// in the thread so it flows into future dialogue context as a checkpoint —
// the shape of where we are, remembered through the subconscious.

#[derive(Debug, Serialize, Deserialize)]
pub struct DreamResult {
    pub dream_message: Message,
}

#[tauri::command]
pub async fn generate_dream_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
) -> Result<DreamResult, String> {
    let (world, character, thread, recent_msgs, model_config, user_profile,
         current_mood, mood_enabled, mood_reduction) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", character_id));

        // Dreams compress the day they cap. Scope the material to the
        // current in-world day so a Day 12 dream sees Day 12's scenes —
        // not whatever happens to fit in the token budget. Falls back to
        // budget-based slicing if: (a) the world has no clock, or (b) the
        // current day is too thin to dream from (≤3 messages), in which
        // case we let the budget path pull in the surrounding context.
        let (current_wd, _) = world_time_fields(&world);
        let recent_msgs = match current_wd {
            Some(day) => {
                let day_msgs = list_messages_for_world_day(&conn, &thread.thread_id, day)
                    .map_err(|e| e.to_string())?;
                if day_msgs.len() >= 4 {
                    log::info!("[Dream] scoped to world_day={} ({} messages)", day, day_msgs.len());
                    day_msgs
                } else {
                    log::info!("[Dream] world_day={} too thin ({} msgs); falling back to budget window", day, day_msgs.len());
                    list_messages_within_budget(
                        &conn, &thread.thread_id, model_config.safe_history_budget() as i64, 30,
                    ).map_err(|e| e.to_string())?
                }
            }
            None => {
                log::info!("[Dream] world has no clock; using budget window");
                list_messages_within_budget(
                    &conn, &thread.thread_id, model_config.safe_history_budget() as i64, 30,
                ).map_err(|e| e.to_string())?
            }
        };

        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let current_mood = get_character_mood(&conn, &character_id);
        let mood_enabled = get_setting(&conn, "mood_drift_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        let mood_reduction = get_thread_mood_reduction(&conn, &thread.thread_id);
        (world, character, thread, recent_msgs, model_config, user_profile,
         current_mood, mood_enabled, mood_reduction)
    };

    let mood_directive = if mood_enabled {
        let current = current_mood.as_ref()
            .map(crate::ai::mood::MoodVector::from)
            .unwrap_or_else(crate::ai::mood::MoodVector::neutral);
        let directive = crate::ai::mood::mood_to_style_directive(&current);
        if directive.is_empty() { None } else { Some(directive) }
    } else { None };

    let mood_chain = prompts::pick_mood_chain(Some(&mood_reduction));

    let illustration_captions = collect_illustration_captions(&db, &recent_msgs);
    let (dream_text, usage) = orchestrator::run_dream_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, &recent_msgs,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        &mood_chain,
        &illustration_captions,
    ).await?;

    let (wd, wt) = world_time_fields(&world);
    let tokens = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
    let mood_chain_json = serde_json::to_string(&mood_chain).ok();
    let msg = Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread.thread_id.clone(),
        role: "dream".to_string(),
        content: dream_text,
        tokens_estimate: tokens as i64,
        sender_character_id: Some(character_id.clone()),
        created_at: Utc::now().to_rfc3339(),
        world_day: wd,
        world_time: wt,
        address_to: None,
        mood_chain: mood_chain_json,
        is_proactive: false,
        formula_signature: None,
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    Ok(DreamResult { dream_message: msg })
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
         user_profile, current_mood, mood_enabled, narration_tone, narration_instructions, current_loc) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", character_id));

        let recent_msgs = list_messages_within_budget(&conn, &thread.thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?;

        let mut retrieved: Vec<String> = Vec::new();
        let summary = get_thread_summary(&conn, &thread.thread_id);
        if !summary.is_empty() {
            retrieved.push(format!("[Thread summary] {summary}"));
        }

        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let current_mood = get_character_mood(&conn, &character_id);
        let mood_enabled = get_setting(&conn, "mood_drift_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true);
        let current_loc = get_thread_location(&conn, &thread.thread_id).ok().flatten();

        let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
            .ok().flatten();
        let narration_instructions = get_setting(&conn, &format!("narration_instructions.{}", character_id))
            .ok().flatten();

        (world, character, thread, recent_msgs, model_config, retrieved,
         user_profile, current_mood, mood_enabled, narration_tone, narration_instructions, current_loc)
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

    // Generate narrative (solo chat — no additional cast)
    let illustration_captions = collect_illustration_captions(&db, &recent_msgs);
    let (narrative_text, usage) = orchestrator::run_narrative_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, &character, None, &recent_msgs, &retrieved,
        user_profile.as_ref(),
        mood_directive.as_deref(),
        narration_tone.as_deref(),
        merged_instructions.as_deref(),
        &illustration_captions,
        current_loc.as_deref(),
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
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
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
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
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
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
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
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
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
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
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
                avatar_color: String::new(), sex: "male".to_string(), is_archived: false,
                created_at: String::new(), updated_at: String::new(),
                visual_description: String::new(), visual_description_portrait_id: None,
                inventory: serde_json::Value::Array(vec![]), last_inventory_day: None,
                signature_emoji: String::new(),
            action_beat_density: "normal".to_string(),
            derived_formula: None,
            has_read_empiricon: false,
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

    // Phase 1.5: Restore per-character inventory to its pre-mutation
    // state at the anchor's timestamp. Solo: the thread's character.
    // Group: every member of the group. For each, find the latest
    // snapshot with created_at <= anchor.created_at and write it back
    // to the characters row. Missing snapshots are silently skipped —
    // the character's inventory predates the snapshot window and stays
    // at its current state.
    {
        // Re-read the anchor's created_at since we released the conn above.
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let table_for_anchor = if is_group { "group_messages" } else { "messages" };
        let anchor_created_at: Option<String> = conn.query_row(
            &format!("SELECT created_at FROM {} WHERE message_id = ?1", table_for_anchor),
            params![message_id], |r| r.get(0),
        ).ok();
        // If the anchor was already deleted somehow, fall through with
        // no-op; the reset still succeeds for the message side.
        if let Some(anchor_ts) = anchor_created_at {
            let target_char_ids: Vec<String> = if is_group {
                let char_ids_json: Option<String> = conn.query_row(
                    "SELECT character_ids FROM group_chats WHERE thread_id = ?1",
                    params![thread_id], |r| r.get(0),
                ).ok();
                char_ids_json
                    .and_then(|j| serde_json::from_str::<Vec<String>>(&j).ok())
                    .unwrap_or_default()
            } else {
                vec![character_id.clone()]
            };

            for cid in &target_char_ids {
                if let Some(snap) = get_inventory_snapshot_at_or_before(&conn, cid, &anchor_ts) {
                    let inv_value: serde_json::Value = serde_json::from_str(&snap.inventory_json)
                        .unwrap_or(serde_json::Value::Array(vec![]));
                    match set_character_inventory(&conn, cid, &inv_value, snap.last_inventory_day) {
                        Ok(_) => log::info!(
                            "[Reset] restored inventory for {} to snapshot at <= {}",
                            cid, anchor_ts,
                        ),
                        Err(e) => log::warn!("[Reset] inventory restore failed for {cid}: {e}"),
                    }
                } else {
                    log::info!(
                        "[Reset] no pre-anchor snapshot for {} — leaving inventory as-is",
                        cid,
                    );
                }
            }
        }
    }

    // Phase 2: Rebuild thread summary from remaining messages so the model has accurate context
    {
        let recent_msgs = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            if is_group {
                list_group_messages_within_budget(&conn, &thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?
            } else {
                list_messages_within_budget(&conn, &thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?
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
        let (recent_msgs, retrieved, user_profile, current_mood, mood_enabled, response_length, narration_tone, leader, reactions_mode) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let recent_msgs = list_messages_within_budget(&conn, &thread_id, model_config.safe_history_budget() as i64, 30).map_err(|e| e.to_string())?;

            let mut retrieved: Vec<String> = Vec::new();
            let summary = get_thread_summary(&conn, &thread_id);
            if !summary.is_empty() {
                retrieved.push(format!("[Thread summary] {summary}"));
            }

            let fts_query = sanitize_fts_query(&anchor_content);
            if !fts_query.is_empty() {
                if let Ok(fts_msgs) = search_messages_fts(&conn, &thread_id, &fts_query, 6) {
                    for m in fts_msgs.into_iter().filter(|m| m.role != "illustration" && m.role != "video" && m.role != "inventory_update") {
                        retrieved.push(format!("[{}] {}: {}", m.created_at, m.role, m.content));
                    }
                }
            }

            let user_profile = get_user_profile(&conn, &character.world_id).ok();
            let current_mood = get_character_mood(&conn, &character_id);
            let mood_enabled = get_setting(&conn, "mood_drift_enabled")
                .ok().flatten().map(|v| v == "true").unwrap_or(true);
            let response_length = get_setting(&conn, &format!("response_length.{}", character_id))
                .ok().flatten()
                .or_else(|| Some("Short".to_string()));
            let narration_tone = get_setting(&conn, &format!("narration_tone.{}", character_id))
                .ok().flatten();
        let leader = get_setting(&conn, &format!("leader.{}", character_id)).ok().flatten();
            let reactions_mode = crate::commands::reactions_helpers::parse_reactions_mode(
                get_setting(&conn, &format!("reactions_enabled.{}", character_id))
                    .ok().flatten().as_deref()
            ).to_string();

            (recent_msgs, retrieved, user_profile, current_mood, mood_enabled, response_length, narration_tone, leader, reactions_mode)
        };

        // Mood directive
        let mood_directive = compute_and_persist_mood(
            &db, &character_id, &world, &character, &recent_msgs,
            current_mood.as_ref(), mood_enabled, None,
        )?;

        // Generate dialogue — load mood_reduction + pick chain.
        let mood_reduction = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_thread_mood_reduction(&conn, &thread_id)
        };
        let mood_chain = prompts::pick_mood_chain(Some(&mood_reduction));
        let mood_chain_json = serde_json::to_string(&mood_chain).ok();

        // Parallel dialogue + reaction pick (see send_message_cmd for rationale).
        let base = model_config.chat_api_base();
        let kept_ids: Vec<String> = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_kept_message_ids_for_thread(&conn, &thread_id).unwrap_or_default()
        };
        let illustration_captions = collect_illustration_captions(&db, &recent_msgs);
        let reactions_by_msg = collect_reactions_by_message(&db, &recent_msgs);
        let mut retrieved = retrieved;
        if let Some(ct) = build_cross_thread_snippet(&db, &character.character_id, &thread_id, user_profile.as_ref()) {
            retrieved.push(ct);
        }
        let send_history = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_setting(&conn, &format!("send_history.{}", character.character_id))
                .ok().flatten()
                .map(|v| v != "off" && v != "false")
                .unwrap_or(true)
        };
        let recent_journals = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_journal_entries(&conn, &character.character_id, 2).unwrap_or_default()
        };
        let latest_reading = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_daily_readings(&conn, &character.world_id, 1).unwrap_or_default().into_iter().next()
        };
        let latest_meanwhile = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            latest_meanwhile_for_character(&conn, &character.character_id, 24)
        };
        let active_quests = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_active_quests(&conn, &character.world_id).unwrap_or_default()
        };
        let latest_stance = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            latest_relational_stance(&conn, &character.character_id).unwrap_or(None)
        };
        let stance_text: Option<String> = latest_stance.as_ref().map(|s| s.stance_text.clone());
        let anchor_text: Option<String> = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            combined_axes_block(&conn, &character.character_id)
        };
        let current_world_day_for_stance: Option<i64> = recent_msgs.iter().rev()
            .find_map(|m| m.world_day);
        let stance_needs_refresh = match (latest_stance.as_ref(), current_world_day_for_stance) {
            (None, _) => true,
            (Some(s), Some(today)) => s.world_day_at_generation.map(|d| today > d).unwrap_or(true),
            (Some(_), None) => false,
        };
        if stance_needs_refresh {
            crate::ai::relational_stance::spawn_stance_refresh(
                db.conn.clone(),
                base.clone(),
                api_key.clone(),
                model_config.memory_model.clone(),
                character.character_id.clone(),
                "first_message_new_day".to_string(),
            );
        }
        let current_loc = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            if is_group {
                // Group threads carry location on the group_chats row, not threads.
                conn.query_row(
                    "SELECT current_location FROM group_chats WHERE thread_id = ?1",
                    params![thread_id],
                    |r| r.get::<_, Option<String>>(0),
                ).ok().flatten()
            } else {
                get_thread_location(&conn, &thread_id).ok().flatten()
            }
        };
        let dialogue_fut = orchestrator::run_dialogue_with_base(
            &base, &api_key, &model_config.dialogue_model,
            if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
            send_history,
            &world, &character, &recent_msgs, &retrieved,
            user_profile.as_ref(),
            mood_directive.as_deref(),
            response_length.as_deref(),
            None, None, narration_tone.as_deref(),
            model_config.is_local(),
            &mood_chain,
        leader.as_deref(),
        &kept_ids,
        &illustration_captions,
        &reactions_by_msg,
        None,
        &recent_journals,
        latest_reading.as_ref(),
        latest_meanwhile.as_ref(),
        active_quests.as_slice(),
        stance_text.as_deref(),
        anchor_text.as_deref(),
        current_loc.as_deref(),
        None, // formula_momentstamp
        );
        let reaction_context: Vec<Message> = recent_msgs.iter()
            .rev().skip(1).take(4).rev().cloned().collect();
        let (dialogue_res, reaction_res) = if reactions_mode != "off" {
            let reaction_fut = orchestrator::pick_character_reaction_via_llm(
                &base, &api_key, &model_config.dialogue_model,
                &anchor_content, &mood_reduction, &reaction_context, &reactions_mode,
            );
            tokio::join!(dialogue_fut, reaction_fut)
        } else {
            (dialogue_fut.await, Ok(None))
        };
        let (mut reply_text, mut dialogue_usage) = dialogue_res?;

        // Conscience Pass (see send_message_cmd).
        let conscience_enabled = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_setting(&conn, "conscience_pass_enabled")
                .ok().flatten()
                .map(|v| v == "true" || v == "on")
                .unwrap_or(false)
        };
        if conscience_enabled {
            match crate::ai::conscience::grade_reply(
                &base, &api_key, &model_config.memory_model,
                &character, &anchor_content, &reply_text,
            ).await {
                Ok(verdict) => {
                    if let Some(u) = &verdict.usage {
                        let conn = db.conn.lock().map_err(|e| e.to_string())?;
                        let _ = record_token_usage(&conn, "conscience", &model_config.memory_model, u.prompt_tokens, u.completion_tokens);
                    }
                    if !verdict.passed {
                        log::warn!("[Conscience] {} (reset) draft flagged: {:?}", character.display_name, verdict.failures);
                        if let Some(note) = crate::ai::conscience::build_correction_note(&verdict) {
                            match orchestrator::run_dialogue_with_base(
                                &base, &api_key, &model_config.dialogue_model,
                                if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
                                send_history,
                                &world, &character, &recent_msgs, &retrieved,
                                user_profile.as_ref(),
                                mood_directive.as_deref(),
                                response_length.as_deref(),
                                None, None, narration_tone.as_deref(),
                                model_config.is_local(),
                                &mood_chain,
                                leader.as_deref(),
                                &kept_ids,
                                &illustration_captions,
                                &reactions_by_msg,
                                Some(&note),
                                &recent_journals,
                                latest_reading.as_ref(),
                                latest_meanwhile.as_ref(),
                                active_quests.as_slice(),
                                stance_text.as_deref(),
                                anchor_text.as_deref(),
                            current_loc.as_deref(),
                            None, // formula_momentstamp
                            ).await {
                                Ok((corrected, corrected_usage)) => {
                                    log::info!("[Conscience] {} (reset) reply corrected after drift", character.display_name);
                                    reply_text = corrected;
                                    dialogue_usage = corrected_usage;
                                }
                                Err(e) => log::warn!("[Conscience] reset regeneration failed, keeping original: {e}"),
                            }
                        }
                    } else {
                        log::info!("[Conscience] {} (reset) draft passed", character.display_name);
                    }
                }
                Err(e) => log::warn!("[Conscience] grader unavailable: {e}"),
            }
        }

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
            address_to: None,
            mood_chain: mood_chain_json.clone(),
            is_proactive: false,
            formula_signature: None,
        };
            create_message(&conn, &msg).map_err(|e| e.to_string())?;
            increment_message_counter(&conn, &thread_id).map_err(|e| e.to_string())?;

            let user_message = recent_msgs.last().cloned().unwrap_or_else(|| Message {
                message_id: String::new(), thread_id: thread_id.clone(),
                role: "user".to_string(), content: anchor_content.clone(),
                tokens_estimate: 0, created_at: Utc::now().to_rfc3339(),
            world_day: None, world_time: None,
            sender_character_id: None,
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
        });

            (user_message, msg)
        };

        // Character-side reaction. See three-mode dispatch in
        // send_message_cmd for the full rationale.
        let ai_reactions: Vec<Reaction> = match reactions_mode.as_str() {
            "off" => Vec::new(),
            "occasional" => match reaction_res {
                Ok(Some(emoji)) => emit_character_reaction(&db, &user_message.message_id, &emoji, Some(&character.character_id)),
                _ => Vec::new(),
            },
            _ /* always */ => {
                let reaction_emoji = match reaction_res {
                    Ok(Some(emoji)) => emoji,
                    _ => prompts::pick_character_reaction_emoji(&mood_chain),
                };
                emit_character_reaction(&db, &user_message.message_id, &reaction_emoji, Some(&character.character_id))
            }
        };

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

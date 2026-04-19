use crate::ai::{openai::{self, StreamingRequest}, orchestrator};
use crate::db::queries::*;
use crate::db::Database;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_memory_artifacts_cmd(
    db: State<Database>,
    subject_id: String,
    artifact_type: String,
) -> Result<Vec<MemoryArtifact>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_memory_artifacts(&conn, &subject_id, &artifact_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_thread_summary_cmd(
    db: State<Database>,
    character_id: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
    Ok(get_thread_summary(&conn, &thread.thread_id))
}

/// Summary returned from the backfill sweep.
#[derive(Debug, Serialize)]
pub struct BackfillSummary {
    /// Chunks successfully embedded and stored (counts each per-character row).
    pub embedded: usize,
    /// Messages already covered (no work needed).
    pub skipped: usize,
    /// Chunks that failed to embed or store.
    pub errors: usize,
}

/// One unit of backfill work — one (message, character_id) pair needing
/// a vector chunk. Synthesized so solo and group paths produce uniform
/// items the batcher can consume.
struct BackfillItem {
    chunk_id: String,
    source_id: String,
    character_id: String,
    world_id: String,
    formatted_content: String,
}

/// Retroactively embed all messages (solo + group) that don't yet have
/// vector-chunk coverage for every character who should remember them.
/// Solo threads: one chunk per message, tagged with the thread's
/// character_id. Group threads: one chunk per message per group member.
/// Embeddings are deduped by content so group messages (same text across
/// N members) hit the API once, not N times.
///
/// Skipped entirely in LM Studio mode — no embedding endpoint. Safe to
/// run repeatedly; the `(source_id, character_id)` existence check
/// prevents re-embedding anything already covered.
#[tauri::command]
pub async fn backfill_embeddings_cmd(
    db: State<'_, Database>,
    api_key: String,
) -> Result<BackfillSummary, String> {
    let model_config = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        orchestrator::load_model_config(&conn)
    };
    if model_config.is_local() {
        log::info!("[Backfill] Skipping — LM Studio mode has no embedding endpoint");
        return Ok(BackfillSummary { embedded: 0, skipped: 0, errors: 0 });
    }

    // ── Gather existing coverage so we can skip already-embedded rows.
    // Collect to Vec inside the lock scope (keeps stmt alive), then
    // transform into HashSet/HashMap after the lock drops.
    let existing_vec: Vec<(String, String)> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT source_id, character_id FROM chunk_metadata WHERE source_type = 'message'")
            .map_err(|e| e.to_string())?;
        let iter = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        iter.filter_map(|r| r.ok()).collect()
    };
    let existing: HashSet<(String, String)> = existing_vec.into_iter().collect();

    // Per-world user name cache (for "{user}: {content}" formatting).
    let user_name_vec: Vec<(String, String)> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT world_id, display_name FROM user_profiles")
            .map_err(|e| e.to_string())?;
        let iter = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        iter.filter_map(|r| r.ok()).collect()
    };
    let user_name_by_world: HashMap<String, String> = user_name_vec.into_iter().collect();
    let user_name_for = |world_id: &str| -> String {
        user_name_by_world.get(world_id).cloned().unwrap_or_else(|| "the human".to_string())
    };

    // Character display-names by id (for formatting speaker prefixes).
    let character_names_vec: Vec<(String, String)> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT character_id, display_name FROM characters")
            .map_err(|e| e.to_string())?;
        let iter = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        iter.filter_map(|r| r.ok()).collect()
    };
    let character_names: HashMap<String, String> = character_names_vec.into_iter().collect();

    let format_content = |role: &str, content: &str, sender_id: Option<&str>, world_id: &str, thread_character_id: Option<&str>| -> String {
        let speaker = match role {
            "user" => user_name_for(world_id),
            "assistant" => sender_id
                .and_then(|id| character_names.get(id).cloned())
                .or_else(|| thread_character_id.and_then(|id| character_names.get(id).cloned()))
                .unwrap_or_else(|| "Character".to_string()),
            "narrative" => "Narrator".to_string(),
            "dream" => sender_id
                .and_then(|id| character_names.get(id).cloned())
                .map(|n| format!("{n} (dream)"))
                .unwrap_or_else(|| "Dream".to_string()),
            _ => "Someone".to_string(),
        };
        format!("{speaker}: {content}")
    };

    // ── Collect backfill work items.
    let mut work: Vec<BackfillItem> = Vec::new();
    let mut skipped: usize = 0;

    // Solo messages — one chunk per message, character_id = thread's character.
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT m.message_id, m.role, m.content, m.sender_character_id, t.character_id, t.world_id
             FROM messages m
             JOIN threads t ON t.thread_id = m.thread_id
             WHERE t.character_id IS NOT NULL
               AND m.role IN ('user', 'assistant', 'narrative', 'dream')
               AND m.content != ''"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
        ))).map_err(|e| e.to_string())?;
        for row in rows.filter_map(|r| r.ok()) {
            let (message_id, role, content, sender_id, character_id, world_id) = row;
            if existing.contains(&(message_id.clone(), character_id.clone())) {
                skipped += 1;
                continue;
            }
            let formatted = format_content(&role, &content, sender_id.as_deref(), &world_id, Some(&character_id));
            work.push(BackfillItem {
                chunk_id: message_id.clone(),
                source_id: message_id,
                character_id,
                world_id,
                formatted_content: formatted,
            });
        }
    }

    // Group messages — one chunk per message per group member.
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT gm.message_id, gm.role, gm.content, gm.sender_character_id, gc.character_ids, gc.world_id
             FROM group_messages gm
             JOIN group_chats gc ON gc.thread_id = gm.thread_id
             WHERE gm.role IN ('user', 'assistant', 'narrative', 'dream')
               AND gm.content != ''"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
        ))).map_err(|e| e.to_string())?;
        for row in rows.filter_map(|r| r.ok()) {
            let (message_id, role, content, sender_id, ids_json, world_id) = row;
            let members: Vec<String> = serde_json::from_str(&ids_json).unwrap_or_default();
            if members.is_empty() { continue; }
            let formatted = format_content(&role, &content, sender_id.as_deref(), &world_id, None);
            for member_id in &members {
                if existing.contains(&(message_id.clone(), member_id.clone())) {
                    skipped += 1;
                    continue;
                }
                work.push(BackfillItem {
                    chunk_id: format!("{message_id}::{member_id}"),
                    source_id: message_id.clone(),
                    character_id: member_id.clone(),
                    world_id: world_id.clone(),
                    formatted_content: formatted.clone(),
                });
            }
        }
    }

    if work.is_empty() {
        log::info!("[Backfill] Nothing to do — {skipped} already covered");
        return Ok(BackfillSummary { embedded: 0, skipped, errors: 0 });
    }

    log::info!("[Backfill] {} items to embed ({skipped} already covered)", work.len());

    // ── Dedupe by content so identical text (e.g. the user's line across
    // all group members) only costs one embedding API call.
    let mut by_content: HashMap<String, Vec<BackfillItem>> = HashMap::new();
    for item in work {
        by_content.entry(item.formatted_content.clone()).or_default().push(item);
    }
    let unique_contents: Vec<String> = by_content.keys().cloned().collect();
    log::info!("[Backfill] {} unique texts after dedupe", unique_contents.len());

    // ── Batch-embed + store.
    let mut embedded: usize = 0;
    let mut errors: usize = 0;
    const BATCH: usize = 20;

    for batch in unique_contents.chunks(BATCH) {
        let contents: Vec<String> = batch.to_vec();
        match orchestrator::generate_embeddings_with_base(
            &model_config.openai_api_base(),
            &api_key,
            &model_config.embedding_model,
            contents.clone(),
        ).await {
            Ok((embeddings, tokens)) => {
                if embeddings.len() != contents.len() {
                    log::warn!("[Backfill] Embedding count mismatch: got {}, expected {}", embeddings.len(), contents.len());
                }
                // Record token usage once per batch.
                {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "embedding", &model_config.embedding_model, tokens, 0);
                }
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                for (content, embedding) in contents.iter().zip(embeddings.iter()) {
                    let items = by_content.remove(content).unwrap_or_default();
                    for item in items {
                        match insert_vector_chunk(
                            &conn,
                            &item.chunk_id,
                            "message",
                            &item.source_id,
                            &item.world_id,
                            &item.character_id,
                            &item.formatted_content,
                            embedding,
                        ) {
                            Ok(_) => embedded += 1,
                            Err(e) => {
                                log::warn!("[Backfill] Failed to store chunk {}: {e}", item.chunk_id);
                                errors += 1;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("[Backfill] Batch embed failed: {e}");
                // Count the items in this batch as errors.
                for content in &contents {
                    errors += by_content.get(content).map(|v| v.len()).unwrap_or(0);
                    by_content.remove(content);
                }
            }
        }
    }

    log::warn!("[Backfill] Done — embedded={embedded} skipped={skipped} errors={errors}");
    Ok(BackfillSummary { embedded, skipped, errors })
}

/// Generate a fresh on-demand summary for a character's chat thread.
#[tauri::command]
pub async fn generate_chat_summary_cmd(
    db: State<'_, Database>,
    app_handle: AppHandle,
    api_key: String,
    character_id: String,
) -> Result<String, String> {
    let (character, recent_msgs, model_config, user_name) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_messages(&conn, &thread.thread_id, 50).map_err(|e| e.to_string())?;
        let user_name = get_user_profile(&conn, &character.world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the protagonist".to_string());
        (character, recent_msgs, model_config, user_name)
    };

    if recent_msgs.is_empty() {
        return Ok("No conversation yet.".to_string());
    }

    let conversation: Vec<String> = recent_msgs.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
        .map(|m| format!("[{}] {}", m.role, m.content))
        .collect();

    let messages = vec![
        openai::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Summarize the recent conversation between {user} and {char}. \
                 Write a substantial narrative summary (12-24 sentences) covering the key events, \
                 emotional beats, and where things currently stand. Include a few key specific details — \
                 names, places, actions, or things said that capture the texture of the conversation. \
                 Write in third person. Refer to the human as \"{user}\", never as \"the user\" or \"you\". \
                 Refer to {char} by name.",
                user = user_name, char = character.display_name,
            ),
        },
        openai::ChatMessage {
            role: "user".to_string(),
            content: format!("Recent messages:\n{}", conversation.join("\n")),
        },
    ];

    let request = StreamingRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.5),
        max_completion_tokens: Some(3200),
        stream: true,
    };

    openai::chat_completion_stream(
        &model_config.chat_api_base(), &api_key, &request, &app_handle, "summary-token",
    ).await
}

/// Generate a fresh on-demand summary for a group chat thread.
#[tauri::command]
pub async fn generate_group_chat_summary_cmd(
    db: State<'_, Database>,
    app_handle: AppHandle,
    api_key: String,
    group_chat_id: String,
) -> Result<String, String> {
    let (characters, recent_msgs, model_config, user_name) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_group_messages(&conn, &gc.thread_id, 50).map_err(|e| e.to_string())?;
        let user_name = get_user_profile(&conn, &gc.world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the protagonist".to_string());

        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();

        (characters, recent_msgs, model_config, user_name)
    };

    if recent_msgs.is_empty() {
        return Ok("No conversation yet.".to_string());
    }

    let char_names: Vec<String> = characters.iter().map(|c| c.display_name.clone()).collect();

    let conversation: Vec<String> = recent_msgs.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
        .map(|m| {
            let speaker = if m.role == "user" { "User".to_string() }
                else if let Some(sid) = &m.sender_character_id {
                    characters.iter().find(|c| &c.character_id == sid)
                        .map(|c| c.display_name.clone()).unwrap_or_else(|| m.role.clone())
                } else { m.role.clone() };
            format!("[{}] {}", speaker, m.content)
        })
        .collect();

    let messages = vec![
        openai::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Summarize the recent group conversation involving {user} and {chars}. \
                 Write a substantial narrative summary (12-24 sentences) covering the key events, \
                 emotional beats, and where things currently stand. Include a few key specific details — \
                 names, places, actions, or things said that capture the texture of the conversation. \
                 Write in third person. Refer to the human as \"{user}\", never as \"the user\" or \"you\". \
                 Refer to each character by name.",
                user = user_name, chars = char_names.join(" and "),
            ),
        },
        openai::ChatMessage {
            role: "user".to_string(),
            content: format!("Recent messages:\n{}", conversation.join("\n")),
        },
    ];

    let request = StreamingRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.5),
        max_completion_tokens: Some(3200),
        stream: true,
    };

    openai::chat_completion_stream(
        &model_config.chat_api_base(), &api_key, &request, &app_handle, "summary-token",
    ).await
}

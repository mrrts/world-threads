use crate::ai::orchestrator;
use crate::ai::prompts::{self, GroupContext, OtherCharacter};
use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

/// Word-boundary substring check (case-sensitive; callers lowercase both sides).
/// A word boundary is any non-alphanumeric byte or the start/end of the string.
/// Used by name-mention detection so "Darren" matches "Hey Darren, ..." but
/// not "Darrening" or a stray substring.
fn contains_word(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || n.len() > h.len() { return false; }
    let mut i = 0;
    while i + n.len() <= h.len() {
        if &h[i..i + n.len()] == n {
            let before_ok = i == 0 || !h[i - 1].is_ascii_alphanumeric();
            let after_ok = i + n.len() == h.len() || !h[i + n.len()].is_ascii_alphanumeric();
            if before_ok && after_ok { return true; }
        }
        i += 1;
    }
    false
}

/// Return the character_ids whose display_name appears as a word in `content`.
/// Case-insensitive; order preserved per the characters slice.
fn detect_named_characters(content: &str, characters: &[Character]) -> Vec<String> {
    let lower = content.to_lowercase();
    characters.iter()
        .filter(|ch| {
            let name = ch.display_name.to_lowercase();
            !name.is_empty() && contains_word(&lower, &name)
        })
        .map(|ch| ch.character_id.clone())
        .collect()
}

/// Ask the model which characters should respond to the user's latest
/// message, in what order. Returns a JSON-array of character_ids.
/// Empty array means nobody speaks (silence makes sense). Unknown ids are
/// filtered out by the caller.
///
/// Uses the memory_model (cheaper tier) — this is classification, not
/// performance. Short context: identity summaries + last ~4 messages +
/// the user's new line.
async fn llm_pick_responders(
    api_key: &str,
    model_config: &orchestrator::ModelConfig,
    content: &str,
    characters: &[Character],
    recent_context: &[Message],
    user_name: &str,
) -> Result<Vec<String>, String> {
    let cast: String = characters.iter()
        .map(|c| {
            let id_line = format!("- id=\"{}\" name=\"{}\"", c.character_id, c.display_name);
            let ident = if c.identity.is_empty() {
                String::new()
            } else {
                let clipped: String = c.identity.chars().take(160).collect();
                format!("\n  identity: {clipped}")
            };
            format!("{id_line}{ident}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let scene: String = recent_context.iter()
        .rev().take(4).rev()
        .filter(|m| m.role == "user" || m.role == "assistant" || m.role == "narrative")
        .map(|m| {
            let speaker = match m.role.as_str() {
                "user" => user_name.to_string(),
                "assistant" => m.sender_character_id.as_ref()
                    .and_then(|id| characters.iter().find(|c| &c.character_id == id).map(|c| c.display_name.clone()))
                    .unwrap_or_else(|| "Character".to_string()),
                "narrative" => "Narrator".to_string(),
                _ => "Someone".to_string(),
            };
            let clipped: String = m.content.chars().take(240).collect();
            format!("{speaker}: {clipped}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let system = r#"You decide which characters in a group chat should respond to the user's latest message.

STRONG DEFAULT: return exactly ONE character. In real group conversation, one person typically picks up a thread — the rest listen, nod, keep doing what they were doing. Multiple-character responses should feel like the exception, not the norm.

Return exactly ONE when ANY of these is true (most cases):
- The user addresses someone by name.
- The user's message continues a thread one character was already carrying.
- The message is a question or comment that doesn't obviously belong to everyone.
- The character who most recently spoke would naturally keep going.
- The register is quiet, intimate, or one-on-one.

Return MULTIPLE only when ALL of these hold:
- The message genuinely invites more than one voice (disagreement surfacing, a question everyone has a stake in, a shift where each character would have a visible reaction they can't hide).
- A single responder would feel like someone visibly holding back an answer they clearly have.
- The multiple voices would say different things, not echo each other.

Return an empty array [] only if silence is genuinely the right move — the user's line doesn't call for a response at all (rare).

When you do return multiple, order them by who'd actually speak first — not by who's listed first in the cast.

Output: raw JSON array of character ids, e.g. ["char_abc"] or ["char_abc","char_def"] or []. No commentary, no markdown, no code fences."#.to_string();

    let user = format!(
        "Group cast:\n{cast}\n\nRecent conversation (last ~4 lines):\n{scene}\n\nUser just said: \"{content}\"\n\nWhich character ids should respond, in order?"
    );

    let request = crate::ai::openai::ChatRequest {
        model: model_config.memory_model.clone(),
        messages: vec![
            crate::ai::openai::ChatMessage { role: "system".to_string(), content: system },
            crate::ai::openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.3),
        max_completion_tokens: Some(80),
        response_format: None,
    };

    let response = crate::ai::openai::chat_completion_with_base(
        &model_config.chat_api_base(), api_key, &request
    ).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    // Tolerate markdown code fences / surrounding text by extracting the
    // first `[...]` substring.
    let body = if let (Some(start), Some(end)) = (raw.find('['), raw.rfind(']')) {
        if end > start { &raw[start..=end] } else { raw.as_str() }
    } else {
        raw.as_str()
    };
    serde_json::from_str::<Vec<String>>(body)
        .map_err(|e| format!("llm_pick_responders parse error: {e} — raw: {raw:?}"))
}

/// Render a list of names as "A", "A and B", or "A, B, and C".
fn join_names_english(names: &[String]) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => {
            let head = &names[..names.len() - 1];
            let tail = &names[names.len() - 1];
            format!("{}, and {tail}", head.join(", "))
        }
    }
}

/// Append a length reminder to just-before-turn system hints in group
/// chats. Group prompts are long (many characters, scene context, craft
/// notes, invariants), and the mid-prompt response-length block loses
/// attention against that weight. Repeating the length directive at
/// late position — inside the turn hint right before generation —
/// keeps short replies actually short. Empty string for Auto / unknown.
///
/// Phrased firmly because group chats specifically tend to drift long:
/// more context, more characters, more pressure to perform.
fn length_reminder_for_turn(response_length: Option<&str>) -> &'static str {
    match response_length {
        Some("Short") => " HARD LENGTH LIMIT: 1–2 sentences. Never 3. Stop before you'd need a third. A two-sentence reply is the target; one sentence is often better. If you feel the urge to keep going, that urge is the mistake.",
        Some("Medium") => " LENGTH LIMIT: 3–4 sentences. Never more than 5. Stop before you'd need a sixth.",
        Some("Long") => " LENGTH: 5–8 sentences, 10 maximum. Stop before you'd need more.",
        _ => "",
    }
}
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
        is_proactive: false,
        };
    create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    Ok(msg)
}

use crate::commands::chat_cmds;

/// Send a message in a group chat. The user's message is saved, then each character
/// responds in order. Returns the user message and all character responses.
/// Pick which group characters should respond to a just-saved user message,
/// and in what order. Deterministic hybrid policy:
///
///   1. Name-mention. If the user's message calls out exactly one
///      character by name, only that character responds. Covers `*To
///      Darren* hey man` and similar direct-address patterns.
///   2. Locked-in exchange. If the recent assistant messages have all
///      been from one specific character, the user is visibly in a
///      two-way thread with them — default to that character. Most of
///      the time reply solo; an occasional (~20%) interjection adds the
///      other members so the room doesn't feel like it disappeared.
///   3. No signals. Everyone responds, first_speaker-reordered — this
///      is the "all pile on" default for topics that belong to the
///      whole room.
///
/// No LLM call. The previous LLM-pick defaulted to "one responder" and
/// routinely dropped members the user wanted included; the locked-in
/// heuristic captures the legitimate "just us two" case without needing
/// the model to make that judgment. The frontend iterates the returned
/// ids through `prompt_group_character_cmd` so the UI streams one
/// character reply at a time.
#[tauri::command]
pub async fn pick_group_responders_cmd(
    db: State<'_, Database>,
    _api_key: String,
    group_chat_id: String,
    content: String,
) -> Result<Vec<String>, String> {
    let (gc, characters, recent_msgs) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;

        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();

        // Window for the locked-in check. 12 is enough to catch a few
        // back-and-forth exchanges without letting ancient history
        // outvote the recent pattern.
        let recent = list_group_messages(&conn, &gc.thread_id, 12).unwrap_or_default();

        (gc, characters, recent)
    };

    // Respect the per-group "first speaker" setting. Reorder the cast up
    // front so every downstream branch that returns "all respond" gets
    // the right order.
    let first_speaker: Option<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, &format!("first_speaker.{}", gc.group_chat_id)).ok().flatten()
    };
    let mut characters = characters;
    if let Some(fs_id) = first_speaker.as_deref().filter(|s| !s.is_empty()) {
        if let Some(pos) = characters.iter().position(|c| c.character_id == fs_id) {
            if pos != 0 {
                let picked = characters.remove(pos);
                characters.insert(0, picked);
                log::info!("[GroupPick] first_speaker={} — reordered cast", fs_id);
            }
        }
    }

    // 1. Name-mention short-circuit. Exactly-one match wins.
    let named = detect_named_characters(&content, &characters);
    if named.len() == 1 {
        log::info!("[GroupPick] name-mention matched: {:?}", named);
        return Ok(named);
    }

    // 2. Locked-in exchange detection. Two signals qualify:
    //
    //    (a) The last two assistant messages (ignoring user interleaving)
    //        are from the same character — a sustained two-way thread.
    //    (b) The single most recent assistant message came from character
    //        X and the message before IT was a user message — i.e. the
    //        user and X just had a one-round exchange and this is the
    //        next user turn. Catches "just started talking to Darren"
    //        which (a) alone needs two rounds to confirm.
    //
    //    Either signal locks in. The 20% interjection downstream keeps
    //    the other characters occasionally present so locking in doesn't
    //    mean the room disappears.
    let locked_in: Option<String> = {
        // Signal (a): last two assistants same char.
        let two_same: Option<String> = {
            let recent_assistants: Vec<&Message> = recent_msgs.iter()
                .rev()
                .filter(|m| m.role == "assistant" && m.sender_character_id.is_some())
                .take(2)
                .collect();
            if recent_assistants.len() >= 2 {
                let first_sender = recent_assistants[0].sender_character_id.as_deref();
                let all_same = recent_assistants.iter()
                    .all(|m| m.sender_character_id.as_deref() == first_sender);
                if all_same {
                    first_sender
                        .filter(|id| characters.iter().any(|c| c.character_id == *id))
                        .map(String::from)
                } else {
                    None
                }
            } else {
                None
            }
        };
        // Signal (b): one-round exchange just started — assistant-from-X
        // immediately preceded by a user message (itself preceded by the
        // just-saved user we skip past at the top of the scan).
        let one_round: Option<String> = {
            let msgs: Vec<&Message> = recent_msgs.iter().rev().collect();
            // Skip any trailing user messages (the just-saved one, plus
            // defensively any others) to land on the most recent
            // non-user message.
            let mut idx = 0;
            while idx < msgs.len() && msgs[idx].role == "user" { idx += 1; }
            let recent = msgs.get(idx);
            let prev = msgs.get(idx + 1);
            match (recent, prev) {
                (Some(r), Some(p)) if r.role == "assistant" && p.role == "user" => {
                    r.sender_character_id.as_deref()
                        .filter(|id| characters.iter().any(|c| c.character_id == *id))
                        .map(String::from)
                }
                _ => None,
            }
        };
        two_same.or(one_round)
    };

    if let Some(partner_id) = locked_in {
        // ~10% interjection chance. Hash-based entropy rather than raw
        // clock nanos: macOS often quantizes SystemTime to microsecond
        // or millisecond resolution, and 1000 / 1_000_000 are both
        // multiples of common denominators — so clock-based mod was
        // always triggering. Hashing the content + group id + timestamp
        // gives a well-mixed value that mod 10 hits ~1-in-10 as intended.
        let roll: u64 = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            content.hash(&mut h);
            gc.group_chat_id.hash(&mut h);
            partner_id.hash(&mut h);
            if let Ok(d) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                d.as_nanos().hash(&mut h);
            }
            h.finish()
        };
        let interject = roll % 10 == 0; // ~10%
        if interject {
            let mut ordered: Vec<String> = vec![partner_id.clone()];
            ordered.extend(
                characters.iter()
                    .map(|c| c.character_id.clone())
                    .filter(|id| id != &partner_id)
            );
            log::info!("[GroupPick] locked-in with {} + interjection: {:?}", partner_id, ordered);
            Ok(ordered)
        } else {
            log::info!("[GroupPick] locked-in with {} — solo reply", partner_id);
            Ok(vec![partner_id])
        }
    } else {
        // 3. No signals. Everyone responds, in first_speaker order.
        let all: Vec<String> = characters.iter().map(|c| c.character_id.clone()).collect();
        log::info!("[GroupPick] no signals — all respond: {:?}", all);
        Ok(all)
    }
}

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
        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", group_chat_id));
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
        is_proactive: false,
        };
        create_group_message(&conn, &user_msg).map_err(|e| e.to_string())?;

        (gc, world, characters, model_config, user_profile, user_msg)
    };

    let (wd, wt) = chat_cmds::world_time_fields(&world);

    // Build character name map for message formatting
    let character_names: HashMap<String, String> = characters.iter()
        .map(|c| (c.character_id.clone(), c.display_name.clone()))
        .collect();

    // Phase 1b: Embed the user message once and store under every group
    // member's character_id so each character can later recall this
    // exchange via semantic search — including from their solo chats.
    // Skipped in local-provider mode (no embedding endpoint). Returns
    // the vector so we can reuse it as the query in per-character
    // retrieval below without re-embedding.
    let member_ids: Vec<String> = characters.iter().map(|c| c.character_id.clone()).collect();
    let user_name = user_profile.as_ref().map(|p| p.display_name.as_str()).unwrap_or("the human");
    let user_chunk_text = format!("{user_name}: {}", content);
    let query_embedding: Option<Vec<f32>> = if !model_config.is_local() {
        chat_cmds::embed_and_store_for_members(
            &db, &api_key, &model_config,
            &world.world_id, &member_ids,
            &user_msg.message_id, &user_chunk_text,
        ).await
    } else {
        None
    };

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

    // Read the per-group "first speaker" setting and reorder the
    // characters list so the chosen character leads. Others follow in
    // their original relative order. Default (no setting) preserves the
    // natural character_ids order.
    let first_speaker: Option<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, &format!("first_speaker.{}", gc.group_chat_id)).ok().flatten()
    };
    let mut characters = characters;
    if let Some(fs_id) = first_speaker.as_deref().filter(|s| !s.is_empty()) {
        if let Some(pos) = characters.iter().position(|c| c.character_id == fs_id) {
            if pos != 0 {
                let picked = characters.remove(pos);
                characters.insert(0, picked);
                log::info!("[GroupTurn] first_speaker={} — reordered character loop", fs_id);
            }
        }
    }

    // Decide who responds. Hybrid policy:
    // 1. Name-mention heuristic — free, deterministic. If the user's
    //    message mentions exactly ONE group character by name (word
    //    boundary, case-insensitive), only that character speaks.
    // 2. LLM-pick fallback — when zero or multiple names matched, ask
    //    the memory model which characters should respond in what
    //    order. Real group conversation rarely has everyone pile onto
    //    every utterance.
    // 3. All-respond fallback — if the LLM call fails or returns no
    //    valid ids, use the full first_speaker-ordered list.
    let user_name_for_pick = user_profile.as_ref().map(|p| p.display_name.as_str()).unwrap_or("the human");
    let named = detect_named_characters(&content, &characters);
    let responder_ids: Vec<String> = if named.len() == 1 {
        log::info!("[GroupTurn] name-mention matched: {:?}", named);
        named
    } else {
        // Gather recent context for the picker.
        let ctx_for_pick: Vec<Message> = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let mut all = list_group_messages(&conn, &gc.thread_id, 6).unwrap_or_default();
            all.retain(|m| m.message_id != user_msg.message_id);
            all
        };
        match llm_pick_responders(&api_key, &model_config, &content, &characters, &ctx_for_pick, user_name_for_pick).await {
            Ok(picks) => {
                let mut valid: Vec<String> = picks.into_iter()
                    .filter(|id| characters.iter().any(|c| &c.character_id == id))
                    .collect();
                if valid.is_empty() {
                    log::info!("[GroupTurn] LLM pick returned no valid ids — all respond");
                    characters.iter().map(|c| c.character_id.clone()).collect()
                } else {
                    // Respect the user's "who speaks first" preference: if
                    // the designated first_speaker is among the LLM-picked
                    // responders, move them to the front. Scope is ORDER —
                    // never force someone to speak the LLM left out (that
                    // would override the silence-is-right-here judgment).
                    if let Some(fs_id) = first_speaker.as_deref().filter(|s| !s.is_empty()) {
                        if let Some(pos) = valid.iter().position(|id| id == fs_id) {
                            if pos != 0 {
                                let picked = valid.remove(pos);
                                valid.insert(0, picked);
                                log::info!("[GroupTurn] first_speaker={} — promoted within LLM picks", fs_id);
                            }
                        }
                    }
                    log::info!("[GroupTurn] LLM-picked responders: {:?}", valid);
                    valid
                }
            }
            Err(e) => {
                log::warn!("[GroupTurn] LLM-pick failed ({e}) — all respond");
                characters.iter().map(|c| c.character_id.clone()).collect()
            }
        }
    };
    let responders: Vec<Character> = responder_ids.iter()
        .filter_map(|id| characters.iter().find(|c| &c.character_id == id).cloned())
        .collect();

    // Phase 2: Each chosen character responds in order. Track who has
    // already spoken in this turn-cycle so second-and-later characters
    // can be explicitly told they may respond to what another character
    // just said (not only to the user).
    let mut responses: Vec<Message> = Vec::new();
    let mut prior_speakers_this_turn: Vec<String> = Vec::new();

    for (_i, character) in responders.iter().enumerate() {
        // Build group context (other characters, excluding the one responding)
        let other_chars: Vec<OtherCharacter> = characters.iter()
            .filter(|c| c.character_id != character.character_id)
            .map(|c| OtherCharacter {
                character_id: c.character_id.clone(),
                display_name: c.display_name.clone(),
                identity_summary: c.identity.clone(),
                sex: c.sex.clone(),
                voice_rules: crate::ai::prompts::json_array_to_strings(&c.voice_rules),
                visual_description: c.visual_description.clone(),
            })
            .collect();
        let group_context = GroupContext { other_characters: other_chars };

        // Load settings scoped to the group chat
        let (response_length, narration_tone, leader) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            // Group chats default to Short when the user hasn't explicitly
            // set anything. The frontend already shows Short as the default,
            // but the setting is only persisted when dirtied — so for fresh
            // groups the DB returns None. Without this fallback, None →
            // no token cap in the orchestrator, which is exactly the "group
            // chats drift long" failure mode users report.
            let rl = get_setting(&conn, &format!("response_length.{}", gc.group_chat_id))
                .ok().flatten()
                .or_else(|| Some("Short".to_string()));
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
        let length_tail = length_reminder_for_turn(response_length.as_deref());
        let hint_content = if prior_speakers_this_turn.is_empty() {
            // First speaker of the turn: default direction toward the
            // user, but free to pivot to anyone they want (other
            // characters in the room, a third party) as long as they
            // mark it with an action beat — `*Looks at Aaron.*` or
            // `*To Aaron:*`.
            format!(
                "[It is now {name}'s turn to speak. Default addressee is {user_name} — speak to them. If you pivot to someone else in the room (another character, a passerby), mark it visibly with an action beat like `*Looks at Aaron.*` or `*To Aaron:*` so the reader knows who you're speaking to. Do NOT open your line with any name at the top (no \"{user_name},\" or \"Aaron.\"). Do not prefix your reply with your own name either. Reply ONLY as {name}.{length_tail}]",
                name = character.display_name,
            )
        } else {
            // Second-and-later: explicitly unlock responding to whoever
            // just spoke in this same turn. The prior speaker often gave
            // the more interesting opening to respond to, and if the
            // model only ever replies to the user in parallel it makes
            // the group feel like two private conversations stapled
            // together rather than a real three-way exchange.
            let prior_list = join_names_english(&prior_speakers_this_turn);
            format!(
                "[It is now {name}'s turn to speak. {prior_list} just spoke in this same turn — you are free to respond to what they said, or to {user_name}, or both, whichever the moment actually asks for. In group conversation, real people often pick up a thread from whoever just spoke rather than answering the original question in parallel. Mark who you're addressing with an action beat — `*Looks at {first_prior}.*` or `*To {first_prior}:*` — so it's clear. Do NOT open your line with any name at the top. Do not prefix your reply with your own name. Reply ONLY as {name}.{length_tail}]",
                name = character.display_name,
                first_prior = prior_speakers_this_turn[0],
            )
        };
        dialogue_msgs.push(Message {
            message_id: String::new(),
            thread_id: String::new(),
            role: "user".to_string(),
            content: hint_content,
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: Utc::now().to_rfc3339(),
            world_day: None,
            world_time: None,
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        });

        // Generate response — load mood_reduction + pick chain for AGENCY.
        let mood_reduction = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_thread_mood_reduction(&conn, &gc.thread_id)
        };
        let mood_chain = prompts::pick_mood_chain(Some(&mood_reduction));
        let mood_chain_json = serde_json::to_string(&mood_chain).ok();

        let kept_ids: Vec<String> = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_kept_message_ids_for_thread(&conn, &gc.thread_id).unwrap_or_default()
        };
        let illustration_captions = crate::commands::chat_cmds::collect_illustration_captions(&db, &dialogue_msgs);
        let reactions_by_msg = crate::commands::chat_cmds::collect_reactions_by_message(&db, &dialogue_msgs);
        let mut retrieved = retrieved.clone();
        if let Some(ct) = crate::commands::chat_cmds::build_cross_thread_snippet(
            &db, &character.character_id, &gc.thread_id, user_profile.as_ref(),
        ) {
            retrieved.push(ct);
        }
        // Semantic memory: search this character's embeddings (spanning
        // their solo + all groups they're in) using the query vector we
        // computed from the user message. Matches [Memory] snippets to
        // surface long-tail recall the recent window can't cover.
        if let Some(emb) = query_embedding.as_ref() {
            let mems = crate::commands::chat_cmds::vector_search_memories(
                &db, &world.world_id, &character.character_id, emb, 4,
            );
            retrieved.extend(mems);
        }
        let send_history = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_setting(&conn, &format!("send_history.{}", gc.group_chat_id))
                .ok().flatten()
                .map(|v| v != "off" && v != "false")
                .unwrap_or(true)
        };
        let (raw_reply, usage) = orchestrator::run_dialogue_with_base(
            &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
            if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
            send_history,
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
            &kept_ids,
            &illustration_captions,
            &reactions_by_msg,
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
            is_proactive: false,
        };
        {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            create_group_message(&conn, &response_msg).map_err(|e| e.to_string())?;
        }

        // Embed the reply for every group member so it lands in their
        // semantic memory — letting any of them recall this exchange
        // from their solo or other chats later.
        if !model_config.is_local() {
            let reply_chunk_text = format!("{}: {}", character.display_name, response_msg.content);
            let _ = crate::commands::chat_cmds::embed_and_store_for_members(
                &db, &api_key, &model_config,
                &world.world_id, &member_ids,
                &response_msg.message_id, &reply_chunk_text,
            ).await;
        }

        prior_speakers_this_turn.push(character.display_name.clone());
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
    // Batch flow (unused by current frontend but kept for completeness) —
    // attribute to None since we don't know which specific character
    // produced the single reaction here.
    let _ = chat_cmds::emit_character_reaction(
        &db,
        &user_msg.message_id,
        &reaction_emoji,
        None,
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
        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", group_chat_id));
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
            visual_description: c.visual_description.clone(),
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

    // Load settings first so the just-before-turn nudge can include the
    // length reminder as a late-position reaffirmation.
    let (response_length, narration_tone, leader) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let rl = get_setting(&conn, &format!("response_length.{}", gc.group_chat_id))
            .ok().flatten()
            .or_else(|| Some("Short".to_string()));
        let nt = get_setting(&conn, &format!("narration_tone.{}", gc.group_chat_id)).ok().flatten();
        let leader = get_setting(&conn, &format!("leader.{}", gc.group_chat_id)).ok().flatten();
        (rl, nt, leader)
    };

    // Add a nudge directing who the character should address
    let mut dialogue_msgs = recent_msgs.clone();
    let user_name = user_profile.as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");
    let length_tail = length_reminder_for_turn(response_length.as_deref());
    let nudge = match address_to.as_deref() {
        Some(target) if !target.is_empty() => {
            format!("[Turn toward {target} and speak directly to them — but do NOT open your line with their name. No \"{target},\" or \"{target}.\" at the top. Just speak.{length_tail}]")
        }
        _ => {
            format!("[Turn toward {user_name} and speak directly to them — but do NOT open your line with their name. No \"{user_name},\" or \"{user_name}.\" at the top. Just speak.{length_tail}]")
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
        is_proactive: false,
        });

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
    let kept_ids: Vec<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_kept_message_ids_for_thread(&conn, &gc.thread_id).unwrap_or_default()
    };
    let illustration_captions = crate::commands::chat_cmds::collect_illustration_captions(&db, &dialogue_msgs);
    let reactions_by_msg = crate::commands::chat_cmds::collect_reactions_by_message(&db, &dialogue_msgs);
    let mut retrieved = retrieved;
    if let Some(ct) = crate::commands::chat_cmds::build_cross_thread_snippet(
        &db, &character.character_id, &gc.thread_id, user_profile.as_ref(),
    ) {
        retrieved.push(ct);
    }
    // Semantic memory: embed the last user message (what this character
    // is responding to) and search this character's vectors. Spans
    // their solo chat + all groups they're in because we store group
    // chunks under each member's character_id.
    let member_ids: Vec<String> = characters.iter().map(|c| c.character_id.clone()).collect();
    let query_embedding: Option<Vec<f32>> = if !model_config.is_local() && !reaction_user_content.is_empty() {
        orchestrator::generate_embeddings_with_base(
            &model_config.openai_api_base(), &api_key,
            &model_config.embedding_model, vec![reaction_user_content.clone()],
        ).await.ok().and_then(|(v, tokens)| {
            let _ = db.conn.lock().ok().map(|conn| record_token_usage(&conn, "embedding", &model_config.embedding_model, tokens, 0));
            v.into_iter().next()
        })
    } else { None };
    if let Some(emb) = query_embedding.as_ref() {
        let mems = crate::commands::chat_cmds::vector_search_memories(
            &db, &world.world_id, &character.character_id, emb, 4,
        );
        retrieved.extend(mems);
    }
    let send_history = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, &format!("send_history.{}", gc.group_chat_id))
            .ok().flatten()
            .map(|v| v != "off" && v != "false")
            .unwrap_or(true)
    };
    let dialogue_fut = orchestrator::run_dialogue_with_base(
        &base, &api_key, &model_config.dialogue_model,
        if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
        send_history,
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
        &kept_ids,
        &illustration_captions,
        &reactions_by_msg,
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
        is_proactive: false,
    };
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    // Embed the reply for every group member so any of them can recall
    // this exchange later from their solo or other chats.
    if !model_config.is_local() {
        let reply_chunk_text = format!("{}: {}", character.display_name, msg.content);
        let _ = crate::commands::chat_cmds::embed_and_store_for_members(
            &db, &api_key, &model_config,
            &world.world_id, &member_ids,
            &msg.message_id, &reply_chunk_text,
        ).await;
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
            chat_cmds::emit_character_reaction(&db, &target_id, &reaction_emoji, Some(&character.character_id))
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
        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", group_chat_id));
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

    // Resolve instructions: if the user left them blank, ask the model to
    // pick a memorable moment from recent messages. That sentence then
    // becomes both the illustration directive and the stored caption/alt.
    let user_display_name = user_profile.as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("The human");
    let resolved_instructions: Option<String> = match custom_instructions.as_deref() {
        Some(s) if !s.trim().is_empty() => Some(s.to_string()),
        _ => {
            match orchestrator::pick_memorable_moment_caption(
                &model_config.chat_api_base(),
                &api_key,
                &model_config.dialogue_model,
                &recent_msgs,
                user_display_name,
            ).await {
                Ok(moment) => Some(moment),
                Err(e) => {
                    log::warn!("[GroupIllustration] memorable-moment pick failed: {e}; proceeding without");
                    None
                }
            }
        }
    };

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
        resolved_instructions.as_deref(),
        has_previous,
        include_scene_summary.unwrap_or(true),
        Some(&characters.iter().map(|c| c.display_name.clone()).collect::<Vec<_>>()),
        Some(&names_map),
    ).await?;
    // Caption: user's instructions verbatim when provided; otherwise
    // derive from scene_description so the caption reflects what was
    // actually painted. See illustration_cmds.rs for full rationale.
    let caption = match custom_instructions.as_deref() {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => {
            match orchestrator::derive_caption_from_scene(
                &model_config.chat_api_base(),
                &api_key,
                &model_config.dialogue_model,
                &scene_description,
            ).await {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("[Illustration] caption derivation failed: {e}; falling back to memorable-moment");
                    resolved_instructions.clone().unwrap_or_default()
                }
            }
        }
    };

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
            caption: caption.clone(),
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
        is_proactive: false,
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
        is_proactive: false,
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
    let illustration_captions = crate::commands::chat_cmds::collect_illustration_captions(&db, &recent_msgs);
    let (narrative_text, usage) = orchestrator::run_narrative_with_base(
        &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
        &world, primary_character,
        if additional_cast.is_empty() { None } else { Some(&additional_cast) },
        &recent_msgs, &[],
        user_profile.as_ref(),
        None,
        narration_tone.as_deref(),
        merged_instructions.as_deref(),
        &illustration_captions,
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
        is_proactive: false,
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

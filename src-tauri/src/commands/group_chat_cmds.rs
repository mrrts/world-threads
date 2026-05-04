use crate::ai::orchestrator;
use crate::ai::prompts::{self, GroupContext, OtherCharacter};
use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};

/// Detect UNAMBIGUOUS direct-address patterns — name appearances that
/// clearly mark someone as the addressee, not a third-person reference.
/// Used as the deterministic fast-path in responder selection; naked
/// name mentions (e.g. "I was thinking about Aaron last time") don't
/// qualify here and get routed to the LLM picker instead, where context
/// can resolve to-X vs about-X.
///
/// Counted as direct address:
///   - Action-beat address: `*To Aaron*`, `*To Aaron:*`, `*Looks at
///     Aaron*`, `*Turns to Aaron*`, `*Glances at Aaron*`.
///   - Vocative opening: the message's very first non-whitespace,
///     non-asterisk token IS the character's name, followed by vocative
///     punctuation (comma, period, ellipsis, em-dash, question mark,
///     exclamation, colon, or a space then another word).
///
/// Everything else — names mid-sentence, possessives like "Aaron's",
/// subject-of-sentence usage — is deliberately NOT caught here.
pub fn detect_direct_address(content: &str, characters: &[Character]) -> Vec<String> {
    let lower = content.to_lowercase();
    let mut matched: Vec<String> = Vec::new();
    for ch in characters {
        let name = ch.display_name.to_lowercase();
        if name.is_empty() { continue; }

        // Action-beat address: look for the patterns inside asterisk
        // pairs. We scan substring presence rather than parsing — close
        // enough, and the false-positive risk is low (these phrases
        // don't arise naturally outside of address markers).
        let ab_patterns = [
            format!("*to {name}*"),
            format!("*to {name}:*"),
            format!("*looks at {name}*"),
            format!("*looks at {name}.*"),
            format!("*turns to {name}*"),
            format!("*turns to {name}.*"),
            format!("*turns toward {name}*"),
            format!("*glances at {name}*"),
            format!("*glances at {name}.*"),
            format!("*to {name},*"),
        ];
        let action_beat_hit = ab_patterns.iter().any(|p| lower.contains(p.as_str()));

        // Vocative opening: strip leading whitespace and an optional
        // leading action beat (text inside *...*), then check whether
        // the next token is the character name followed by vocative
        // punctuation.
        let vocative_hit = {
            let trimmed = lower.trim_start();
            // Skip a leading *...* action beat if present.
            let after_beat: &str = if trimmed.starts_with('*') {
                if let Some(end) = trimmed[1..].find('*') {
                    trimmed[(end + 2)..].trim_start()
                } else { trimmed }
            } else {
                trimmed
            };
            if after_beat.starts_with(&name) {
                let rest = &after_beat[name.len()..];
                // Next char must be vocative punctuation or end-of-string.
                let next = rest.chars().next();
                matches!(next, None | Some(',') | Some('.') | Some('?') | Some('!') | Some(':') | Some('—') | Some('–') | Some(';'))
                    // Or "Aaron " but only if followed by NOTHING else
                    // that turns it into a subject. We skip this: naked
                    // "Aaron look at this" doesn't count as vocative
                    // without a comma, because it's ambiguous with
                    // subject usage.
            } else {
                false
            }
        };

        if action_beat_hit || vocative_hit {
            matched.push(ch.character_id.clone());
        }
    }
    matched
}

/// Ask the model which characters should respond to the user's latest
/// message, in what order. Returns a JSON-array of character_ids.
/// Empty array means nobody speaks (silence makes sense). Unknown ids are
/// filtered out by the caller.
///
/// Uses the memory_model (cheaper tier) — this is classification, not
/// performance. Short context: identity summaries + last ~4 messages +
/// the user's new line.
/// Count how many consecutive recent assistant messages were by the
/// most-recent speaker, ignoring user/narrative messages between them.
/// Returns (display_name, run_length). Used by `llm_pick_responders`
/// to surface speaker-rotation pressure: when one character has been
/// carrying many turns in a row, the picker gets that as data and can
/// weigh whether the silent peer's absence has begun to be felt.
///
/// Doctrine: this is signal-injection only — no system-prompt change.
/// The picker's existing criteria already balance "continues a thread
/// one character was already carrying" (push toward same speaker) with
/// "single responder would feel like someone visibly holding back"
/// (push toward MULTIPLE). Adding the run-length count gives the picker
/// the data it needs to weigh those criteria honestly without us
/// telling it what conclusion to reach.
pub fn consecutive_run_by_recent_speaker(
    recent_context: &[Message],
    characters: &[Character],
) -> Option<(String, usize)> {
    let assistant_speakers: Vec<&str> = recent_context.iter()
        .rev()
        .filter(|m| m.role == "assistant")
        .filter_map(|m| m.sender_character_id.as_deref())
        .collect();
    let most_recent = assistant_speakers.first()?;
    let mut count = 0;
    for s in &assistant_speakers {
        if s == most_recent { count += 1; } else { break; }
    }
    let name = characters.iter()
        .find(|c| &c.character_id == most_recent)
        .map(|c| c.display_name.clone())
        .unwrap_or_else(|| "Character".to_string());
    Some((name, count))
}

async fn llm_pick_responders(
    api_key: &str,
    model_config: &orchestrator::ModelConfig,
    content: &str,
    characters: &[Character],
    recent_context: &[Message],
    user_name: &str,
) -> Result<Vec<String>, String> {
    llm_pick_responders_with_overrides(
        api_key, model_config, content, characters, recent_context, user_name,
        /* omit_continuity_note = */ false,
    ).await
}

/// Same as `llm_pick_responders` but with a flag to suppress the
/// consecutive-run continuity-note injection. Used for bite-tests that
/// want to characterize the note's effect on picker behavior; production
/// callers go through `llm_pick_responders` (which always injects when
/// the run length is >= 2).
pub async fn llm_pick_responders_with_overrides(
    api_key: &str,
    model_config: &orchestrator::ModelConfig,
    content: &str,
    characters: &[Character],
    recent_context: &[Message],
    user_name: &str,
    omit_continuity_note: bool,
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

    // Speaker-rotation pressure signal: when one character has been
    // carrying multiple consecutive turns, surface that count as data
    // for the picker to weigh against its existing criteria. Phrased
    // neutrally — we give the LLM the fact, not the conclusion. Only
    // injected when run length >= 2 (a single recent speaker isn't a
    // run yet) and not suppressed by bite-test override.
    let continuity_note = if omit_continuity_note {
        String::new()
    } else {
        match consecutive_run_by_recent_speaker(recent_context, characters) {
            Some((name, n)) if n >= 2 => format!(
                "\n\nThread continuity: {name} has carried the last {n} character turns alone (the silent peer hasn't spoken in that stretch). Apply the criteria above to this specific message."
            ),
            _ => String::new(),
        }
    };

    let user = format!(
        "Group cast:\n{cast}\n\nRecent conversation (last ~4 lines):\n{scene}{continuity_note}\n\nUser just said: \"{content}\"\n\nWhich character ids should respond, in order?"
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

/// Ask a cheap memory-tier LLM: given the last few messages, which
/// character (if any) is the user most likely addressing right now?
///
/// Returns:
///   - `Some(character_id)` when the model picks one specific member.
///   - `None` when the model returns "none" / "both" / "all" / anything
///     ambiguous, or when the call fails outright. Caller should fall
///     back to all-respond in that case.
///
/// Input window: last `context_limit` messages (excluding the just-saved
/// user message passed separately as `user_content`). Model cost is
/// ~150 input tokens + 20 output tokens. Uses temperature=0.0 so the
/// same scene reliably picks the same addressee; this is not a place
/// we want creative variation.
/// Result of the addressee-picker LLM call.
///
/// Three meaningfully-different cases that callers may want to handle
/// differently:
/// - `Solo(id)`: the user is addressing one specific character; that
///   character should respond.
/// - `Collective`: the user is addressing the group as a whole
///   ("you guys", "both of you", "all of you", "everyone"). Caller
///   chooses how to dispatch — group_chat picks one character at random
///   for auto-respond mode; inventory falls back to most-recent speaker.
/// - `Ambiguous`: the LLM said NONE — message is genuinely unclear or
///   for-the-room-but-not-collectively. Caller falls back to its
///   default (typically the most-recently-active character).
#[derive(Debug, Clone)]
pub enum AddresseePick {
    Solo(String),
    Collective,
    Ambiguous,
}

impl AddresseePick {
    /// For callers that want the simpler `Option<String>` semantics —
    /// either a specific character id or "fall back to your default."
    /// Both `Collective` and `Ambiguous` map to `None` here. The
    /// group_chat caller does NOT use this; it pattern-matches the
    /// enum directly to dispatch the random-pick on `Collective`.
    pub fn into_option(self) -> Option<String> {
        match self {
            AddresseePick::Solo(id) => Some(id),
            AddresseePick::Collective | AddresseePick::Ambiguous => None,
        }
    }
}

pub async fn llm_pick_addressee(
    api_key: &str,
    model_config: &orchestrator::ModelConfig,
    user_content: &str,
    recent_context: &[Message],
    characters: &[Character],
    user_name: &str,
    context_limit: usize,
) -> AddresseePick {
    if characters.is_empty() { return AddresseePick::Ambiguous; }

    // Render the last few non-system messages as speaker-labeled lines.
    let scene: Vec<String> = recent_context.iter()
        .rev().take(context_limit).rev()
        .filter(|m| m.role == "user" || m.role == "assistant" || m.role == "narrative")
        .map(|m| {
            let speaker = match m.role.as_str() {
                "user" => user_name.to_string(),
                "assistant" => m.sender_character_id.as_deref()
                    .and_then(|id| characters.iter().find(|c| &c.character_id == id).map(|c| c.display_name.clone()))
                    .unwrap_or_else(|| "Character".to_string()),
                "narrative" => "Narrator".to_string(),
                _ => "Someone".to_string(),
            };
            let clipped: String = m.content.chars().take(220).collect();
            format!("{speaker}: {clipped}")
        })
        .collect();
    let scene_block = if scene.is_empty() { "(no prior messages)".to_string() } else { scene.join("\n") };
    let names_list = characters.iter().map(|c| c.display_name.as_str()).collect::<Vec<_>>().join(" | ");

    let system = r#"This is an easy question — answer it quickly.

Who is the user speaking WITH, not ABOUT? OR who are they directly addressing?

═══════════════════════════════════════════════════════════════
CRITICAL RULE — READ TWICE:

A CHARACTER BEING TALKED ABOUT, REFERRED TO, OR GESTURED AT IS NOT THE ADDRESSEE. THEY ARE A REFERENCE POINT. DO NOT PICK THEM.

The person the user is talking TO is the person whose reply the user is setting up. NOT the person the user is pointing at, naming in the third person, asking about, looking at, nodding toward, or referencing in any way.

  - "I was thinking about Aaron earlier." → Aaron is REFERRED TO. Not addressed. Do NOT pick Aaron.
  - "*Looks at Aaron.* What do you think, Darren?" → Aaron is GESTURED AT. Darren is addressed. Pick Darren.
  - "What about Aaron, though?" → Aaron is MENTIONED POINTEDLY. Not addressed. Do NOT pick Aaron; pick whoever the user is actually talking to.
  - "Did you hear what Aaron said?" → Aaron is the SUBJECT. The addressee is whoever "you" refers to — probably whoever was just in dialogue with the user. Do NOT pick Aaron.

Direction-of-SPEECH is what matters. Not direction-of-eyes, gesture, name-mention, or topic. The addressee is the person the user's speech is FOR — the one whose response the user is waiting for.
═══════════════════════════════════════════════════════════════

Read the recent scene and the user's latest message, then pick the one character the user is actually talking TO — whether by directly addressing them or by continuing an ongoing exchange with them. A character the user has been going back-and-forth with is being spoken with even if the current message doesn't name them.

If the user pivots addressees mid-message, the FINAL addressee wins. Example: "Yeah, makes sense, Darren. ...Actually — Aaron, what did you think?" → pick Aaron. The earlier address is abandoned once the user turns to someone new.

Output RULES (strict):
- Output exactly one name from the provided list, OR the word ALL, OR the word NONE.
- ALL (also acceptable: BOTH, EVERYONE) means the user is EXPLICITLY addressing the group collectively — phrases like "you guys," "both of you," "all of you," "y'all," "everyone." A clear collective vocative. Use this only when the user is plainly speaking to the whole room as a group, not when the message merely happens to be relevant to multiple people.
- NONE means it's genuinely ambiguous, or the message is for-the-room without an explicit collective vocative, or you can't tell. Use this when no character is being singled out AND the user didn't use a collective address.
- One specific character name means the user is talking to that one character (whether by direct address or by continuing an ongoing exchange).
- No commentary, no punctuation, no explanation. Just the name, or ALL, or NONE."#.to_string();

    let user = format!(
        "Characters in this group (pick one name exactly): {names}\n\nRecent scene (last few messages):\n{scene}\n\n{user_name}'s latest message:\n{content}\n\nWho is {user_name} most likely addressing? Output: one name from the list above, or NONE.",
        names = names_list,
        scene = scene_block,
        user_name = user_name,
        content = user_content,
    );

    let request = crate::ai::openai::ChatRequest {
        model: model_config.memory_model.clone(),
        messages: vec![
            crate::ai::openai::ChatMessage { role: "system".to_string(), content: system },
            crate::ai::openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.0),
        max_completion_tokens: Some(30),
        response_format: None,
    };

    let response = match crate::ai::openai::chat_completion_with_base(
        &model_config.chat_api_base(), api_key, &request,
    ).await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("[GroupPick/Addressee] LLM call failed: {e}");
            return AddresseePick::Ambiguous;
        }
    };
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    if raw.is_empty() {
        log::warn!("[GroupPick/Addressee] empty response from LLM");
        return AddresseePick::Ambiguous;
    }
    log::info!("[GroupPick/Addressee] raw LLM response: {raw:?}");

    // Normalize: strip quotes / punctuation, compare case-insensitively.
    let cleaned = raw.trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .to_string();
    let cleaned_lower = cleaned.to_lowercase();

    // COLLECTIVE: explicit "you guys" / "both of you" / "all of you" /
    // "everyone." The system prompt specifically asks for ALL/BOTH/EVERYONE
    // in this case. Caller decides how to dispatch (group_chat picks
    // randomly; inventory falls back to most-recent).
    if matches!(cleaned_lower.as_str(), "all" | "both" | "everyone" | "y'all" | "yall") {
        log::info!("[GroupPick/Addressee] LLM said collective ({cleaned_lower:?}) — Collective");
        return AddresseePick::Collective;
    }

    // AMBIGUOUS: NONE or empty. Caller falls back to its default
    // (typically most-recently-active speaker).
    if cleaned_lower == "none" || cleaned_lower.is_empty() {
        log::info!("[GroupPick/Addressee] LLM said NONE — Ambiguous");
        return AddresseePick::Ambiguous;
    }

    // SOLO: match against the character list. Prefer exact (case-insensitive)
    // match; fall back to a character whose display name appears as a
    // substring of the model's output (handles "Darren." / "Darren ")
    // and vice versa (handles models that over-qualify with a title).
    let by_exact = characters.iter()
        .find(|c| c.display_name.to_lowercase() == cleaned_lower);
    if let Some(c) = by_exact { return AddresseePick::Solo(c.character_id.clone()); }
    let by_contains = characters.iter()
        .find(|c| cleaned_lower.contains(&c.display_name.to_lowercase())
            || c.display_name.to_lowercase().contains(&cleaned_lower));
    match by_contains {
        Some(c) => AddresseePick::Solo(c.character_id.clone()),
        None => {
            log::warn!("[GroupPick/Addressee] could not match LLM response {raw:?} to any character name ({:?}) — treating as Ambiguous",
                characters.iter().map(|c| c.display_name.as_str()).collect::<Vec<_>>());
            AddresseePick::Ambiguous
        }
    }
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
/// Keep this late-turn reminder in the same register as the prompt
/// stack's response-length doctrine: active contract for fixed modes,
/// soft brevity compass for Auto. Group chats need the late reminder,
/// but not older barked-limit rhetoric.
fn length_reminder_for_turn(response_length: Option<&str>) -> &'static str {
    match response_length {
        Some("Short") => " ACTIVE LENGTH CONTRACT: 1–2 sentences total. One sentence is often enough. Ignore longer scrollback if the setting changed; the current setting governs this reply.",
        Some("Medium") => " ACTIVE LENGTH CONTRACT: 3–4 sentences total. Hold that shape even if earlier turns ran shorter or longer; the current setting governs this reply.",
        Some("Long") => " ACTIVE LENGTH CONTRACT: about 5–10 sentences total. Let the reply breathe when the moment supports it, then land cleanly.",
        Some("Auto") => " DEFAULT LEAN: often 2–3 short sentences total is enough. Keep the substance; cut ornament first. If this moment genuinely needs more air, give it the air it needs.",
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
        formula_signature: None,
        };
    create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    Ok(msg)
}

use crate::commands::chat_cmds;

/// Send a message in a group chat. The user's message is saved, then each character
/// responds in order. Returns the user message and all character responses.
/// Pick which group characters should respond to a just-saved user message,
/// and in what order. Three-step policy:
///
///   1. Name-mention short-circuit. If the user's message calls out
///      exactly one character by name (word-boundary match), only
///      that character responds. Fast, free, deterministic — covers
///      "*Looks at Darren* hey man" and similar direct addresses.
///   2. LLM addressee pick. A single cheap memory-tier call that
///      takes the last 4 messages + latest user content + the
///      character list, and asks "who is the user most likely
///      addressing RIGHT NOW?" Returns one name or NONE.
///   3. Fallback. If the LLM returns NONE or the call fails,
///      everyone responds in first_speaker-reordered order — the
///      "message belongs to the room" default.
///
/// When step 2 returns one character, they respond solo — no
/// trailing interjections from other members. Only step 3 (the
/// NONE / failure fallback) ever returns multiple responders.
/// Frontend iterates the returned ids through
/// `prompt_group_character_cmd` so the UI streams one reply at a time.
#[tauri::command]
pub async fn pick_group_responders_cmd(
    db: State<'_, Database>,
    api_key: String,
    group_chat_id: String,
    content: String,
) -> Result<Vec<String>, String> {
    let (_gc, characters, recent_msgs, model_config, user_name) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;

        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();

        // Window fed to the addressee LLM. 6 is generous since we'll
        // filter to the last 4 textual messages inside the helper.
        let recent = list_group_messages(&conn, &gc.thread_id, 6).unwrap_or_default();

        let mut model_config = orchestrator::load_model_config(&conn);
        model_config.apply_provider_override(&conn, &format!("provider_override.{}", group_chat_id));
        let user_name = get_user_profile(&conn, &gc.world_id)
            .ok()
            .map(|p| p.display_name)
            .unwrap_or_else(|| "the human".to_string());

        (gc, characters, recent, model_config, user_name)
    };

    // 1. Direct-address short-circuit. Only fires on unambiguous patterns
    //    — action-beat address (`*To Darren*`) or vocative opening
    //    (`Darren, ...`). Naked mentions mid-sentence (e.g. "I was
    //    thinking about Aaron last time") fall through to the LLM
    //    picker, which can distinguish "to X" from "about X" with
    //    conversational context.
    let addressed = detect_direct_address(&content, &characters);
    if addressed.len() == 1 {
        log::info!("[GroupPick] direct-address matched: {:?}", addressed);
        return Ok(addressed);
    }

    // Scene window for the LLM call — drop the just-saved user message
    // (we pass `content` separately so the model sees it once).
    let ctx_for_pick: Vec<Message> = {
        let mut out = recent_msgs.clone();
        if let Some(pos) = out.iter().rposition(|m| m.role == "user") {
            out.remove(pos);
        }
        out
    };

    // 2. LLM addressee pick. Picked character responds solo — no
    // interjection, no trailing other responders. Three possible
    // outcomes:
    //   - Solo(id): specific character is addressed → that character
    //     responds.
    //   - Collective: user said "you guys" / "both of you" / "everyone"
    //     etc. — TRULY RANDOM pick from the group's characters in
    //     auto-respond mode. (Each character has equal odds; the user
    //     gets the whoever-speaks-up-first feel of an actual group.)
    //   - Ambiguous: LLM said NONE or call failed — fallback to
    //     most-recently-active speaker (the user was probably still
    //     continuing that thread even if it didn't read that way).
    let pick = llm_pick_addressee(
        &api_key, &model_config, &content, &ctx_for_pick,
        &characters, &user_name, 4,
    ).await;

    match pick {
        AddresseePick::Solo(addressee_id) => {
            log::info!("[GroupPick] LLM addressee={} — solo reply", addressee_id);
            Ok(vec![addressee_id])
        }
        AddresseePick::Collective => {
            // Truly random pick from the group. Uses uuid v4's RNG
            // entropy (already a project dependency); modulo-by-len
            // is fine for 2-4 character groups (the bias from non-
            // power-of-2 lens is microscopic at these sizes).
            let idx = (uuid::Uuid::new_v4().as_u128() as usize) % characters.len();
            let random_id = characters[idx].character_id.clone();
            log::info!(
                "[GroupPick] LLM said collective — random pick: {} ({}/{} chars)",
                random_id, idx + 1, characters.len(),
            );
            Ok(vec![random_id])
        }
        AddresseePick::Ambiguous => {
            // Fallback. The message is ambiguous / for-the-room. Still
            // exactly ONE character speaks: pick the most-recently-active
            // character in the thread. If nobody has spoken yet, default
            // to the first group member.
            let fallback_id: Option<String> = recent_msgs.iter()
                .rev()
                .filter_map(|m| m.sender_character_id.as_deref())
                .find(|id| characters.iter().any(|c| c.character_id == *id))
                .map(String::from)
                .or_else(|| characters.first().map(|c| c.character_id.clone()));

            match fallback_id {
                Some(id) => {
                    log::info!("[GroupPick] LLM returned NONE — fallback to most-recent speaker {}", id);
                    Ok(vec![id])
                }
                None => {
                    log::warn!("[GroupPick] no characters available for fallback");
                    Ok(Vec::new())
                }
            }
        }
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
        formula_signature: None,
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
    let (reduction_snapshot, reaction_context, reactions_mode) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let r = get_thread_mood_reduction(&conn, &gc.thread_id);
        // Last 4 messages before the user's brand-new one.
        let all = list_group_messages(&conn, &gc.thread_id, 5).unwrap_or_default();
        let ctx: Vec<Message> = all.into_iter()
            .filter(|m| m.message_id != user_msg.message_id)
            .collect();
        // Per-chat reactions setting (group-scoped). Three modes:
        // "off" | "occasional" | "always". See solo chat_cmds.rs for
        // the full rationale; same parsing helper used here.
        let reactions_mode = crate::commands::reactions_helpers::parse_reactions_mode(
            get_setting(&conn, &format!("reactions_enabled.{}", gc.group_chat_id))
                .ok().flatten().as_deref()
        ).to_string();
        (r, ctx, reactions_mode)
    };
    // Skip launching the reaction LLM call entirely when reactions are
    // off — cost + latency saving. For "occasional" and "always" we
    // launch; the LLM self-paces in occasional mode (returning Ok(None)
    // when the moment doesn't fit a reaction).
    let reaction_handle = if reactions_mode != "off" {
        let reaction_base = model_config.chat_api_base();
        let reaction_model = model_config.dialogue_model.clone();
        let reaction_content = content.clone();
        let reaction_reduction = reduction_snapshot.clone();
        let reaction_api_key = api_key.clone();
        let reaction_ctx = reaction_context.clone();
        let reaction_mode_clone = reactions_mode.clone();
        Some(tokio::spawn(async move {
            orchestrator::pick_character_reaction_via_llm(
                &reaction_base, &reaction_api_key, &reaction_model,
                &reaction_content, &reaction_reduction, &reaction_ctx, &reaction_mode_clone,
            ).await
        }))
    } else {
        None
    };

    // Read the per-group "first speaker" setting and reorder the
    // characters list so the chosen character leads. Others follow in
    // their original relative order. Default (no setting) preserves the
    // natural character_ids order.
    // Decide who responds. Hybrid policy:
    // 1. Direct-address short-circuit — free, deterministic. If the
    //    user's message unambiguously addresses one character (vocative
    //    opening, action-beat `*To X*`), only that character speaks.
    // 2. LLM-pick — ask the memory model which characters should
    //    respond in what order.
    // 3. All-respond fallback — if the LLM call fails or returns no
    //    valid ids, everyone speaks in stored character_ids order.
    let user_name_for_pick = user_profile.as_ref().map(|p| p.display_name.as_str()).unwrap_or("the human");
    let addressed = detect_direct_address(&content, &characters);
    let responder_ids: Vec<String> = if addressed.len() == 1 {
        log::info!("[GroupTurn] direct-address matched: {:?}", addressed);
        addressed
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
                let valid: Vec<String> = picks.into_iter()
                    .filter(|id| characters.iter().any(|c| &c.character_id == id))
                    .collect();
                if valid.is_empty() {
                    log::info!("[GroupTurn] LLM pick returned no valid ids — all respond");
                    characters.iter().map(|c| c.character_id.clone()).collect()
                } else {
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
                inventory_block: crate::ai::prompts::render_inventory_block(&c.display_name, &c.inventory),
                derived_formula: c.derived_formula.clone(),
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
        formula_signature: None,
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
        let recent_journals = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_journal_entries(&conn, &character.character_id, 2).unwrap_or_default()
        };
        let latest_reading = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_daily_readings(&conn, &world.world_id, 1).unwrap_or_default().into_iter().next()
        };
        let latest_meanwhile = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            latest_meanwhile_for_character(&conn, &character.character_id, 24)
        };
        let active_quests = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            list_active_quests(&conn, &world.world_id).unwrap_or_default()
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

        // Reactions=off depth-signal reward (group surface, flow 1).
        // Build a Formula momentstamp when reactions are off, with
        // stateful chain (read prior signature from latest assistant
        // message in this group chat).
        let (formula_momentstamp_text, formula_momentstamp_signature): (Option<String>, Option<String>) = if reactions_mode == "off" {
            let prior_sig: Option<String> = {
                let conn = db.conn.lock().map_err(|e| e.to_string())?;
                crate::db::queries::latest_formula_signature_group(&conn, &gc.thread_id).ok().flatten()
            };
            match crate::ai::momentstamp::build_formula_momentstamp(
                &model_config.chat_api_base(),
                &api_key,
                &model_config.memory_model,
                &dialogue_msgs,
                prior_sig.as_deref(),
                Some(character),
            ).await.ok().flatten() {
                Some(r) => (Some(r.block), Some(r.signature)),
                None => (None, None),
            }
        } else {
            (None, None)
        };

        let (current_loc, location_pair) = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let cl = get_group_chat_location(&conn, &gc.group_chat_id).ok().flatten();
            let lp = orchestrator::resolve_location_derivation_pair(&conn, &world.world_id, cl.as_deref(), &dialogue_msgs);
            (cl, lp)
        };
        let location_arg: Option<(&str, &str)> = location_pair.as_ref().map(|(n, d)| (n.as_str(), d.as_str()));
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
            None,
            &recent_journals,
            latest_reading.as_ref(),
            latest_meanwhile.as_ref(),
            active_quests.as_slice(),
            stance_text.as_deref(),
            anchor_text.as_deref(),
        current_loc.as_deref(),
        formula_momentstamp_text.as_deref(),
        location_arg,
        ).await?;

        // Strip own prefix and truncate any other-character dialogue
        let other_names: Vec<&str> = characters.iter()
            .filter(|c| c.character_id != character.character_id)
            .map(|c| c.display_name.as_str()).collect();
        let mut reply_text = strip_character_prefix(&raw_reply, &character.display_name, &other_names);
        let mut usage = usage;

        // Conscience Pass: grade against the five invariants and
        // regenerate once on drift. Gated by `conscience_pass_enabled`
        // setting (default on). Non-fatal on grader errors.
        let conscience_enabled = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            get_setting(&conn, "conscience_pass_enabled")
                .ok().flatten()
                .map(|v| v != "off" && v != "false")
                .unwrap_or(true)
        };
        if conscience_enabled {
            match crate::ai::conscience::grade_reply(
                &model_config.chat_api_base(), &api_key, &model_config.memory_model,
                character, &content, &reply_text,
            ).await {
                Ok(verdict) => {
                    if let Some(u) = &verdict.usage {
                        let conn = db.conn.lock().map_err(|e| e.to_string())?;
                        let _ = record_token_usage(&conn, "conscience", &model_config.memory_model, u.prompt_tokens, u.completion_tokens);
                    }
                    if !verdict.passed {
                        log::warn!(
                            "[Conscience] {} (group) draft flagged: {:?}",
                            character.display_name,
                            verdict.failures,
                        );
                        if let Some(note) = crate::ai::conscience::build_correction_note(&verdict) {
                            match orchestrator::run_dialogue_with_base(
                                &model_config.chat_api_base(), &api_key, &model_config.dialogue_model,
                                if !model_config.is_local() { Some(&model_config.memory_model) } else { None },
                                send_history,
                                &world, character, &dialogue_msgs, &retrieved,
                                user_profile.as_ref(),
                                None,
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
                                Some(&note),
                                &recent_journals,
                                latest_reading.as_ref(),
                                latest_meanwhile.as_ref(),
                                active_quests.as_slice(),
                                stance_text.as_deref(),
                                anchor_text.as_deref(),
                            current_loc.as_deref(),
                            formula_momentstamp_text.as_deref(),
                            location_arg,
                            ).await {
                                Ok((corrected_raw, corrected_usage)) => {
                                    log::info!("[Conscience] {} (group) reply corrected after drift", character.display_name);
                                    reply_text = strip_character_prefix(&corrected_raw, &character.display_name, &other_names);
                                    usage = corrected_usage;
                                }
                                Err(e) => {
                                    log::warn!("[Conscience] group regeneration failed, keeping original draft: {e}");
                                }
                            }
                        }
                    } else {
                        log::info!("[Conscience] {} (group) draft passed", character.display_name);
                    }
                }
                Err(e) => {
                    log::warn!("[Conscience] grader unavailable, passing draft through: {e}");
                }
            }
        }

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
            formula_signature: formula_momentstamp_signature.clone(),
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
    // Three-mode dispatch parallels solo chat:
    //   - "off"        → no handle was created; skip.
    //   - "occasional" → handle returned; if Ok(Some(emoji)) emit, else skip.
    //   - "always"     → handle returned; emit emoji or fallback.
    if let Some(handle) = reaction_handle {
        let pick_result = handle.await;
        let to_emit: Option<String> = match (reactions_mode.as_str(), pick_result) {
            ("occasional", Ok(Ok(Some(e)))) => Some(e),
            ("occasional", _) => None,
            (_, Ok(Ok(Some(e)))) => Some(e),
            (_, _) => {
                // "always" fallback path
                let chain = prompts::pick_mood_chain(Some(&reduction_snapshot));
                Some(prompts::pick_character_reaction_emoji(&chain))
            }
        };
        if let Some(emoji) = to_emit {
            // Batch flow (unused by current frontend but kept for completeness) —
            // attribute to None since we don't know which specific character
            // produced the single reaction here.
            let _ = chat_cmds::emit_character_reaction(
                &db,
                &user_msg.message_id,
                &emoji,
                None,
            );
        }
    }

    // Auto-derivation refresh: same shape as chat_cmds.rs solo path.
    // For group, fan out to refresh EACH responding character + the
    // user-in-world + the world. INFLIGHT dedupe in derivation::
    // maybe_refresh_after_turn handles duplicate user/world calls.
    let group_loc_for_refresh = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_group_chat_location(&conn, &gc.group_chat_id).ok().flatten()
    };
    for resp in &responses {
        crate::ai::derivation::maybe_refresh_after_turn(
            db.conn.clone(),
            model_config.chat_api_base(),
            api_key.clone(),
            model_config.memory_model.clone(),
            world.world_id.clone(),
            resp.sender_character_id.clone(),
            group_loc_for_refresh.clone(),
        ).await;
    }
    // If no responders (silent turn), still refresh user + world via
    // a single character_id=None call.
    if responses.is_empty() {
        crate::ai::derivation::maybe_refresh_after_turn(
            db.conn.clone(),
            model_config.chat_api_base(),
            api_key.clone(),
            model_config.memory_model.clone(),
            world.world_id.clone(),
            None,
            group_loc_for_refresh.clone(),
        ).await;
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
            inventory_block: crate::ai::prompts::render_inventory_block(&c.display_name, &c.inventory),
            derived_formula: c.derived_formula.clone(),
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
    let (response_length, narration_tone, leader, reactions_mode) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let rl = get_setting(&conn, &format!("response_length.{}", gc.group_chat_id))
            .ok().flatten()
            .or_else(|| Some("Short".to_string()));
        let nt = get_setting(&conn, &format!("narration_tone.{}", gc.group_chat_id)).ok().flatten();
        let leader = get_setting(&conn, &format!("leader.{}", gc.group_chat_id)).ok().flatten();
        let reactions_mode = crate::commands::reactions_helpers::parse_reactions_mode(
            get_setting(&conn, &format!("reactions_enabled.{}", gc.group_chat_id))
                .ok().flatten().as_deref()
        ).to_string();
        (rl, nt, leader, reactions_mode)
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
        formula_signature: None,
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
    let recent_journals = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_journal_entries(&conn, &character.character_id, 2).unwrap_or_default()
    };
    let latest_reading = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_daily_readings(&conn, &world.world_id, 1).unwrap_or_default().into_iter().next()
    };
    let latest_meanwhile = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        latest_meanwhile_for_character(&conn, &character.character_id, 24)
    };
    let active_quests = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        list_active_quests(&conn, &world.world_id).unwrap_or_default()
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
            base.clone(),
            api_key.clone(),
            model_config.memory_model.clone(),
            character.character_id.clone(),
            "first_message_new_day".to_string(),
        );
    }

    // Reactions=off depth-signal reward (group surface, flow 2).
    // Stateful chain via prior signature.
    let (formula_momentstamp_text2, formula_momentstamp_signature2): (Option<String>, Option<String>) = if reactions_mode == "off" {
        let prior_sig: Option<String> = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            crate::db::queries::latest_formula_signature_group(&conn, &gc.thread_id).ok().flatten()
        };
        match crate::ai::momentstamp::build_formula_momentstamp(
            &base,
            &api_key,
            &model_config.memory_model,
            &dialogue_msgs,
            prior_sig.as_deref(),
            Some(&character),
        ).await.ok().flatten() {
            Some(r) => (Some(r.block), Some(r.signature)),
            None => (None, None),
        }
    } else {
        (None, None)
    };

    let (current_loc, location_pair) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let cl = get_group_chat_location(&conn, &group_chat_id).ok().flatten();
        let lp = orchestrator::resolve_location_derivation_pair(&conn, &world.world_id, cl.as_deref(), &dialogue_msgs);
        (cl, lp)
    };
    let location_arg: Option<(&str, &str)> = location_pair.as_ref().map(|(n, d)| (n.as_str(), d.as_str()));
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
        None,
        &recent_journals,
        latest_reading.as_ref(),
        latest_meanwhile.as_ref(),
        active_quests.as_slice(),
        stance_text.as_deref(),
        anchor_text.as_deref(),
    current_loc.as_deref(),
    formula_momentstamp_text2.as_deref(),
    location_arg,
    );
    let (dialogue_res, reaction_res) = if reactions_mode != "off" {
        let reaction_fut = orchestrator::pick_character_reaction_via_llm(
            &base, &api_key, &model_config.dialogue_model,
            &reaction_user_content, &mood_reduction2, &reaction_context, &reactions_mode,
        );
        tokio::join!(dialogue_fut, reaction_fut)
    } else {
        (dialogue_fut.await, Ok(None))
    };
    let (raw_reply, usage) = dialogue_res?;

    let other_names: Vec<&str> = characters.iter()
        .filter(|c| c.character_id != character.character_id)
        .map(|c| c.display_name.as_str()).collect();
    let mut reply_text = strip_character_prefix(&raw_reply, &character.display_name, &other_names);
    let mut usage = usage;

    // Conscience Pass (see send_group_message_cmd).
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
            &character, &reaction_user_content, &reply_text,
        ).await {
            Ok(verdict) => {
                if let Some(u) = &verdict.usage {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    let _ = record_token_usage(&conn, "conscience", &model_config.memory_model, u.prompt_tokens, u.completion_tokens);
                }
                if !verdict.passed {
                    log::warn!("[Conscience] {} (group-prompt) draft flagged: {:?}", character.display_name, verdict.failures);
                    if let Some(note) = crate::ai::conscience::build_correction_note(&verdict) {
                        match orchestrator::run_dialogue_with_base(
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
                            Some(&note),
                            &recent_journals,
                            latest_reading.as_ref(),
                            latest_meanwhile.as_ref(),
                            active_quests.as_slice(),
                            stance_text.as_deref(),
                            anchor_text.as_deref(),
                        current_loc.as_deref(),
                        formula_momentstamp_text2.as_deref(),
                        location_arg,
                        ).await {
                            Ok((corrected_raw, corrected_usage)) => {
                                log::info!("[Conscience] {} (group-prompt) reply corrected after drift", character.display_name);
                                reply_text = strip_character_prefix(&corrected_raw, &character.display_name, &other_names);
                                usage = corrected_usage;
                            }
                            Err(e) => log::warn!("[Conscience] group-prompt regeneration failed, keeping original: {e}"),
                        }
                    }
                } else {
                    log::info!("[Conscience] {} (group-prompt) draft passed", character.display_name);
                }
            }
            Err(e) => log::warn!("[Conscience] grader unavailable: {e}"),
        }
    }

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
        formula_signature: formula_momentstamp_signature2.clone(),
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

    // Emit the character's reaction on the triggering user message.
    // Three-mode dispatch parallels solo:
    //   - "off"        → no emit.
    //   - "occasional" → emit only if LLM picked a real emoji (Ok(Some)).
    //   - "always"     → emit emoji or fallback.
    let ai_reactions: Vec<Reaction> = match (reactions_mode.as_str(), reaction_target_id) {
        ("off", _) | (_, None) => Vec::new(),
        ("occasional", Some(target_id)) => match reaction_res {
            Ok(Some(emoji)) => chat_cmds::emit_character_reaction(&db, &target_id, &emoji, Some(&character.character_id)),
            _ => Vec::new(),
        },
        (_ /* always */, Some(target_id)) => {
            let reaction_emoji = match reaction_res {
                Ok(Some(emoji)) => emoji,
                _ => prompts::pick_character_reaction_emoji(&mood_chain2),
            };
            chat_cmds::emit_character_reaction(&db, &target_id, &reaction_emoji, Some(&character.character_id))
        }
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
    let (world, characters, gc, recent_msgs, model_config, user_profile, current_loc) = {
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

        let current_loc = get_group_chat_location(&conn, &group_chat_id).ok().flatten();

        (world, characters, gc, recent_msgs, model_config, user_profile, current_loc)
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
        current_loc.as_deref(),
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
        formula_signature: None,
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
        formula_signature: None,
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
    let current_loc = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_group_chat_location(&conn, &group_chat_id).ok().flatten()
    };
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
        current_loc.as_deref(),
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
        formula_signature: None,
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

use crate::ai::openai::{self, ChatRequest, StreamingRequest};
use crate::ai::orchestrator;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use tauri::{AppHandle, Emitter, State};

/// Approximate token count. English text is typically ~4 chars/token; we
/// slightly pessimize (3.5) to leave headroom against the user's declared
/// context window.
fn approx_tokens(s: &str) -> usize {
    (s.chars().count() as f64 / 3.5) as usize
}

/// Generate a novel chapter from a day's messages via LLM.
#[tauri::command]
pub async fn generate_novel_entry_cmd(
    db: State<'_, Database>,
    app_handle: AppHandle,
    api_key: String,
    thread_id: String,
    world_day: i64,
    is_group: bool,
) -> Result<String, String> {
    let (messages, world, characters, character_names, user_name, user_profile, model_config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);

        // Get all messages for this thread and day
        let all_msgs = if is_group {
            get_all_group_messages(&conn, &thread_id).map_err(|e| e.to_string())?
        } else {
            get_all_messages(&conn, &thread_id).map_err(|e| e.to_string())?
        };

        let day_msgs: Vec<Message> = all_msgs.into_iter()
            .filter(|m| m.world_day == Some(world_day) && m.role != "illustration" && m.role != "video")
            .collect();

        if day_msgs.is_empty() {
            return Err("No messages found for this day.".to_string());
        }

        // Get world from thread
        let world_id: String = conn.query_row(
            "SELECT world_id FROM threads WHERE thread_id = ?1",
            rusqlite::params![thread_id], |r| r.get(0),
        ).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;

        let user_name = get_user_profile(&conn, &world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the protagonist".to_string());

        let characters = list_characters(&conn, &world_id).unwrap_or_default();
        let char_names: std::collections::HashMap<String, String> = characters.iter()
            .map(|c| (c.character_id.clone(), c.display_name.clone()))
            .collect();

        let user_profile = get_user_profile(&conn, &world_id).ok();

        (day_msgs, world, characters, char_names, user_name, user_profile, model_config)
    };

    // Build conversation text with time-of-day transition markers inserted
    // whenever world_time changes. Without these, the beats extractor (and
    // the chapter synthesizer) has no idea when morning became afternoon
    // became evening — the novelization ends up lumping everything under
    // one lighting / time mood.
    let mut conversation: Vec<String> = Vec::new();
    let mut last_time: Option<String> = None;
    for m in &messages {
        if let Some(wt) = &m.world_time {
            if last_time.as_deref() != Some(wt.as_str()) {
                let formatted = wt.split(' ').map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        Some(first) => first.to_uppercase().to_string() + &c.as_str().to_lowercase(),
                        None => String::new(),
                    }
                }).collect::<Vec<_>>().join(" ");
                conversation.push(format!("[It is now {formatted}.]"));
                last_time = Some(wt.clone());
            }
        }
        let speaker = match m.role.as_str() {
            "user" => user_name.clone(),
            "narrative" => "[Narrative]".to_string(),
            "context" => "[Context]".to_string(),
            "assistant" => {
                m.sender_character_id.as_ref()
                    .and_then(|id| character_names.get(id))
                    .cloned()
                    .unwrap_or_else(|| "Character".to_string())
            }
            _ => m.role.clone(),
        };
        conversation.push(format!("{}: {}", speaker, m.content));
    }

    // Build rich character descriptions
    let char_descriptions: Vec<String> = characters.iter().map(|c| {
        let mut desc = format!("- {}", c.display_name);
        if !c.identity.is_empty() {
            desc.push_str(&format!(": {}", c.identity));
        }
        let voice_rules = crate::ai::prompts::json_array_to_strings(&c.voice_rules);
        if !voice_rules.is_empty() {
            desc.push_str(&format!("\n  Voice: {}", voice_rules.join("; ")));
        }
        desc
    }).collect();

    let user_desc = user_profile.as_ref().map(|p| {
        let mut d = format!("- {} (the protagonist, written in second person — \"you\")", p.display_name);
        if !p.description.is_empty() {
            d.push_str(&format!(": {}", p.description));
        }
        d
    }).unwrap_or_else(|| format!("- {} (the protagonist, written in second person — \"you\")", user_name));

    let system_prompt = format!(
        r#"You are a gifted literary novelist. Your task is to transform a day's conversation and narrative beats into a vivid, immersive chapter of a novel.

SETTING: {world_desc}

CHARACTERS:
{user_desc}
{char_list}

INSTRUCTIONS:
- A chapter has shape: it opens on a specific image, builds through its middle, and lands on a moment of resonance — an image, a line, a small revelation. Find that shape in the day's events.
- Transform the conversation into rich, flowing prose — a full chapter of a novel.
- Write in SECOND PERSON present tense. {user_name} is always "you."
- Other characters are referred to by name in third person.
- Weave dialogue, action, internal thought, and sensory detail together seamlessly.
- Invent freely, but with restraint. The best literary prose chooses one or two precise sensory details per beat rather than cataloguing everything. A single specific image — the way light catches a glass, the particular way someone holds their hands — does more work than a paragraph of atmosphere. Trust the reader to fill in the rest.
- Expand brief exchanges into full scenes with atmosphere and pacing.
- Include all the key beats from the conversation but enhance them with novelistic craft.
- Lines tagged [Narrative] are existing narration from the source — expand and enrich them, don't just copy. Lines tagged [Context] are background information the characters share — weave them in as known truths, not as exposition.
- Make it feel like one vivid, cohesive chapter — not a transcript.
- Use literary techniques: metaphor, subtext, tension, rhythm.
- Vary sentence length aggressively to keep the second-person present from feeling monotonous. Use sentence fragments. Let some paragraphs breathe.
- The chapter should be substantial — aim for 1500-3000 words.
- Do NOT include chapter titles, headers, or meta-commentary. Just the prose."#,
        world_desc = if world.description.is_empty() { "A richly detailed world." } else { &world.description },
        user_desc = user_desc,
        char_list = char_descriptions.join("\n"),
    );

    let conversation_text = conversation.join("\n");

    // If the local model's context window can't comfortably hold the whole
    // day in one shot, fall back to a two-phase "beats → chapter" novelization
    // (see the "phased novelization" plan). Otherwise keep the original
    // single-shot streaming path.
    let is_local = model_config.is_local();
    let est_prompt_tokens = approx_tokens(&system_prompt) + approx_tokens(&conversation_text) + 200;
    let budget = model_config.safe_local_prompt_budget() as usize;
    let needs_chunking = is_local && est_prompt_tokens > budget;

    if !needs_chunking {
        let api_messages = vec![
            openai::ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            openai::ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Here is the full conversation for Day {}:\n\n{}\n\nTransform this into a vivid novel chapter.",
                    world_day,
                    conversation_text,
                ),
            },
        ];

        let request = StreamingRequest {
            model: model_config.dialogue_model.clone(),
            messages: api_messages,
            temperature: Some(0.95),
            max_completion_tokens: Some(4096),
            stream: true,
        };

        return openai::chat_completion_stream(
            &model_config.chat_api_base(), &api_key, &request, &app_handle, "novel-token",
        ).await;
    }

    // ── Phase 1: beats extraction ────────────────────────────────────────
    // Slice the day's conversation lines into chunks that each fit in the
    // safe prompt budget, and ask the model to produce a compact list of
    // concrete story beats for each chunk. These are tiny — the final
    // chapter call will receive ALL of them together with full context.
    let beats_system = r#"You are a story editor. Read the conversation excerpt and extract a thorough, in-order list of narrative BEATS — every concrete moment that would belong in a novel chapter of this day.

What counts as a beat:
- Intense emotional moments or major decisions.
- A realization, a decision, a confession, a refusal.
- A shift in mood or power between characters.
- A new piece of information learned, or withheld.
- An action taken, a gesture, a significant movement.
- A line of dialogue that lands — include it verbatim in quotation marks.
- A silence that lingers, a pause that means something.
- A change in setting or the time of day ("the light shifts to late afternoon").

Rules:
- Output ONLY a list, one beat per line, prefixed with "- ".
- Each beat is a crisp, specific sentence in the present tense.
- Preserve [It is now X.] time markers when you cross one — emit a beat like "- The time turns to afternoon." so the chapter can honor it.
- Include direct quotes verbatim in "…" whenever a specific line carries weight.
- BE THOROUGH. Aim for 8 to 20 beats per excerpt — err high rather than low. Readers should get the significant moments of what happened, not a vague summary.
- Skip only pure filler — "they keep talking about X" with no change.
- Do NOT write prose. Do NOT write a summary paragraph. Just the beat list."#;

    // Chunk budget accounts for the beats system prompt + completion space.
    // Reserve room for up to ~1500 tokens of beat output per chunk so the
    // model isn't forced to compress; a rich chunk can easily produce 20
    // beats at ~50-80 tokens each.
    let chunk_budget = budget.saturating_sub(approx_tokens(beats_system) + 1_600);
    let chunk_budget = chunk_budget.max(2_000); // don't go absurdly small

    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut current_tokens: usize = 0;
    for line in &conversation {
        let t = approx_tokens(line) + 1;
        if current_tokens + t > chunk_budget && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
            current_tokens = 0;
        }
        current.push(line.clone());
        current_tokens += t;
    }
    if !current.is_empty() {
        chunks.push(current);
    }

    let _ = app_handle.emit("novel-phase", serde_json::json!({
        "phase": "beats",
        "chunks_total": chunks.len(),
        "chunk_index": 0,
    }));

    let mut all_beats: Vec<String> = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let beats_request = ChatRequest {
            model: model_config.dialogue_model.clone(),
            messages: vec![
                openai::ChatMessage {
                    role: "system".to_string(),
                    content: beats_system.to_string(),
                },
                openai::ChatMessage {
                    role: "user".to_string(),
                    content: format!(
                        "Conversation excerpt (part {} of {} from Day {}):\n\n{}\n\nReturn the beat list.",
                        i + 1, chunks.len(), world_day,
                        chunk.join("\n"),
                    ),
                },
            ],
            temperature: Some(0.5),
            max_completion_tokens: Some(1_500),
            response_format: None,
        };
        let beats_response = openai::chat_completion_with_base(
            &model_config.chat_api_base(), &api_key, &beats_request,
        ).await?;
        let beats_text = beats_response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        // Collect bullet lines, tolerant of the model using "* " or no prefix.
        for line in beats_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            let cleaned = trimmed
                .trim_start_matches("- ")
                .trim_start_matches("* ")
                .trim_start_matches("• ")
                .to_string();
            if !cleaned.is_empty() {
                all_beats.push(cleaned);
            }
        }
        let _ = app_handle.emit("novel-phase", serde_json::json!({
            "phase": "beats",
            "chunks_total": chunks.len(),
            "chunk_index": i + 1,
        }));
    }

    // ── Phase 2: chapter synthesis ───────────────────────────────────────
    // Feed all collected beats plus the original character / world context
    // to the model and stream a full chapter.
    let _ = app_handle.emit("novel-phase", serde_json::json!({
        "phase": "chapter",
    }));

    let beats_joined = all_beats.iter()
        .map(|b| format!("- {b}"))
        .collect::<Vec<_>>()
        .join("\n");

    let api_messages = vec![
        openai::ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        openai::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Here are the narrative beats for Day {}, extracted in order from the full conversation:\n\n{}\n\nTransform these beats into a vivid novel chapter. \n\
                 - Preserve the direct quotes verbatim when they appear.\n\
                 - Honor every time-of-day marker (morning, afternoon, evening, night, etc.). When the beats show a transition, let the chapter reflect it in lighting, atmosphere, and pacing.\n\
                 - Every significant beat should land in the chapter — a realization, a decision, a line that mattered, a shift between characters. Do not smooth them into vague summary.\n\
                 - Expand the rest into rich prose with a coherent arc from opening image to resonant closing moment.",
                world_day,
                beats_joined,
            ),
        },
    ];

    let request = StreamingRequest {
        model: model_config.dialogue_model.clone(),
        messages: api_messages,
        temperature: Some(0.95),
        max_completion_tokens: Some(4096),
        stream: true,
    };

    openai::chat_completion_stream(
        &model_config.chat_api_base(), &api_key, &request, &app_handle, "novel-token",
    ).await
}

/// Save (or update) a novel entry.
#[tauri::command]
pub fn save_novel_entry_cmd(
    db: State<'_, Database>,
    thread_id: String,
    world_day: i64,
    content: String,
) -> Result<NovelEntry, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();

    // Check if one exists already
    let existing = get_novel_entry(&conn, &thread_id, world_day);
    let novel_id = existing.map(|e| e.novel_id)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let entry = NovelEntry {
        novel_id: novel_id.clone(),
        thread_id: thread_id.clone(),
        world_day,
        content,
        created_at: now.clone(),
        updated_at: now,
    };
    upsert_novel_entry(&conn, &entry).map_err(|e| e.to_string())?;

    Ok(entry)
}

/// Get a novel entry for a specific thread and day.
#[tauri::command]
pub fn get_novel_entry_cmd(
    db: State<'_, Database>,
    thread_id: String,
    world_day: i64,
) -> Result<Option<NovelEntry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(get_novel_entry(&conn, &thread_id, world_day))
}

/// List all novel entries for a thread.
#[tauri::command]
pub fn list_novel_entries_cmd(
    db: State<'_, Database>,
    thread_id: String,
) -> Result<Vec<NovelEntry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_novel_entries(&conn, &thread_id).map_err(|e| e.to_string())
}

/// Delete a novel entry.
#[tauri::command]
pub fn delete_novel_entry_cmd(
    db: State<'_, Database>,
    thread_id: String,
    world_day: i64,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    delete_novel_entry(&conn, &thread_id, world_day).map_err(|e| e.to_string())
}

use crate::ai::openai::{self, StreamingRequest};
use crate::ai::orchestrator::{self, ModelConfig};
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinHandle;

/// Holds the JoinHandle for the currently-running background novelization
/// task, if any. Used so we can abort() it on window focus — aborting the
/// task drops every in-flight future, which closes the HTTP connection and
/// halts the local model's generation within ~a token batch.
pub struct BgNovelHandle(pub Arc<AsyncMutex<Option<JoinHandle<()>>>>);

impl Default for BgNovelHandle {
    fn default() -> Self {
        Self(Arc::new(AsyncMutex::new(None)))
    }
}

/// Emitted between rendered sections as a literal novel-token event. Renders
/// to a horizontal rule in markdown; the UI can decorate `<hr>` if desired.
const SECTION_DIVIDER: &str = "\n\n* * *\n\n";

/// Approximate token count. English text is typically ~4 chars/token; we
/// slightly pessimize (3.5) to leave headroom against the user's declared
/// context window.
fn approx_tokens(s: &str) -> usize {
    (s.chars().count() as f64 / 3.5) as usize
}

/// Capitalize each word of a world_time label ("MORNING" → "Morning").
fn format_time_of_day(wt: &str) -> String {
    wt.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(first) => first.to_uppercase().to_string() + &c.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// One daypart-scoped slice of a day's conversation. If a day runs through
/// MORNING → AFTERNOON → EVENING, it yields three sections; a day that
/// never changes time yields one.
struct Section {
    /// Capitalized time-of-day label ("Morning") or empty if messages carry
    /// no world_time.
    label: String,
    /// Formatted conversation lines for this section (`speaker: content`).
    lines: Vec<String>,
}

/// Format a single Message into a conversation line. Separated out so the
/// sectioning pass and any single-shot path can produce the same output.
fn format_line(m: &Message, user_name: &str, character_names: &HashMap<String, String>) -> String {
    let speaker = match m.role.as_str() {
        "user" => user_name.to_string(),
        "narrative" => "[Narrative]".to_string(),
        "context" => "[Context]".to_string(),
        "assistant" => m.sender_character_id.as_ref()
            .and_then(|id| character_names.get(id))
            .cloned()
            .unwrap_or_else(|| "Character".to_string()),
        other => other.to_string(),
    };
    format!("{}: {}", speaker, m.content)
}

/// Group the day's messages into sections by world_time transitions. A new
/// section starts every time world_time changes value; messages without
/// world_time are appended to whatever section is currently open.
fn group_into_sections(
    messages: &[Message],
    user_name: &str,
    character_names: &HashMap<String, String>,
) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut cur_label = String::new();
    let mut cur_lines: Vec<String> = Vec::new();

    for m in messages {
        let formatted = m.world_time.as_deref().map(format_time_of_day).unwrap_or_default();
        if !formatted.is_empty() && !cur_label.is_empty() && formatted != cur_label {
            // Time rolled over — close off the current section.
            sections.push(Section { label: std::mem::take(&mut cur_label), lines: std::mem::take(&mut cur_lines) });
        }
        if cur_label.is_empty() && !formatted.is_empty() {
            cur_label = formatted;
        }
        cur_lines.push(format_line(m, user_name, character_names));
    }
    if !cur_lines.is_empty() {
        sections.push(Section { label: cur_label, lines: cur_lines });
    }
    sections
}

/// System prompt for extracting narrative beats from a chunk of conversation.
/// Shared between all beat-extraction calls.
const BEATS_SYSTEM: &str = r#"You are a story editor. Read the conversation excerpt and extract a thorough, in-order list of narrative BEATS — every concrete moment that would belong in a novel chapter of this day.

What counts as a beat:
- Intense emotional moments or major decisions.
- A realization, a decision, a confession, a refusal.
- A shift in mood or power between characters.
- A new piece of information learned, or withheld.
- An action taken, a gesture, a significant movement.
- A line of dialogue that lands — capture its essence. You may quote it verbatim or paraphrase for rhythm and clarity; what matters is that the emotional content comes through.
- A silence that lingers, a pause that means something.

Rules:
- Output ONLY a list, one beat per line, prefixed with "- ".
- Each beat is a crisp, specific sentence in the present tense.
- Include memorable lines in "…" when they land — you may quote verbatim OR lightly paraphrase to capture the essence. Creative liberty with the exact wording is fine; fidelity to the emotional beat is what matters.
- BE THOROUGH. Aim for 8 to 20 beats per excerpt — err high rather than low. Readers should get the significant moments of what happened, not a vague summary.
- Skip only pure filler — "they keep talking about X" with no change.
- Do NOT write prose. Do NOT write a summary paragraph. Just the beat list."#;

/// Extract numbered beats from `lines`, chunking as needed so each chunk
/// fits the local model's safe prompt budget. Emits `novel-phase` progress
/// events between chunks (skipped when silent). Uses streaming for the
/// request itself so that dropping this future cancels generation cleanly.
#[allow(clippy::too_many_arguments)]
async fn extract_beats(
    app_handle: &AppHandle,
    model_config: &ModelConfig,
    api_key: &str,
    lines: &[String],
    label: &str,
    world_day: i64,
    silent: bool,
) -> Result<Vec<String>, String> {
    let budget = model_config.safe_local_prompt_budget() as usize;
    // Reserve space for system prompt + up to 1500 tokens of beat output.
    let chunk_budget = budget.saturating_sub(approx_tokens(BEATS_SYSTEM) + 1_600).max(2_000);

    let mut chunks: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut current_tokens: usize = 0;
    for line in lines {
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

    let mut all_beats: Vec<String> = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let user_msg = openai::ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Conversation excerpt (part {} of {}) from Day {} — {} section:\n\n{}\n\nReturn the beat list.",
                i + 1, chunks.len(), world_day,
                if label.is_empty() { "untagged".to_string() } else { label.to_string() },
                chunk.join("\n"),
            ),
        };
        // Always use streaming for the beats request so that dropping the
        // task (on focus / cancel) closes the HTTP connection and the local
        // model halts generation promptly. For the foreground flow we use
        // the silent variant too — the beats phase has never emitted
        // per-token events.
        let streaming_req = StreamingRequest {
            model: model_config.dialogue_model.clone(),
            messages: vec![
                openai::ChatMessage { role: "system".to_string(), content: BEATS_SYSTEM.to_string() },
                user_msg,
            ],
            temperature: Some(0.5),
            max_completion_tokens: Some(1_500),
            stream: true,
        };
        let beats_text = openai::chat_completion_stream_silent(
            &model_config.chat_api_base(), api_key, &streaming_req,
        ).await?;
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
        if !silent {
            let _ = app_handle.emit("novel-phase", serde_json::json!({
                "phase": "beats",
                "section": label,
                "chunks_total": chunks.len(),
                "chunk_index": i + 1,
            }));
        }
    }
    Ok(all_beats)
}

/// Produce a section of the novel chapter and stream it via `novel-token`.
/// If the section's raw messages fit the local budget, we send them directly
/// to the chapter writer (skipping beat extraction for quality). Otherwise
/// we extract beats first and write the section from those.
// `target_words`: rough target length in words, proportional to this
// section's share of the day's content. The model is told to treat this as
// a guide to pacing, not a strict limit. No corresponding
// max_completion_tokens clamp — we prefer natural endings over hard cutoffs.
// `silent`: when true, bypass novel-token / novel-phase events and use the
// silent streaming variant so no UI updates fire (used by the background
// idle-time novelization path).
#[allow(clippy::too_many_arguments)]
async fn stream_section(
    app_handle: &AppHandle,
    model_config: &ModelConfig,
    api_key: &str,
    base_system: &str,
    section: &Section,
    section_index: usize,
    total_sections: usize,
    world_day: i64,
    target_words: usize,
    silent: bool,
) -> Result<String, String> {
    let section_text = section.lines.join("\n");
    let section_tokens = approx_tokens(&section_text);
    let budget = model_config.safe_local_prompt_budget() as usize;
    // Reserve ~3000 tokens for the chapter completion + per-section framing.
    let fits_directly = section_tokens + 3_000 <= budget;

    let shape_hint = section_shape(section_index, total_sections);
    let section_name = if section.label.is_empty() {
        format!("Section {} of {}", section_index + 1, total_sections)
    } else {
        format!("{} section ({} of {})", section.label, section_index + 1, total_sections)
    };
    let weight_line = format!(
        "- This section should run roughly {target_words} words — give or take. Use that as a guide to pacing and how much weight this part of the day deserves, not a strict limit."
    );

    let user_content = if fits_directly {
        format!(
            "{shape}\n\n\
             You are writing the {name} of Day {day}'s chapter.\n\n\
             Here are the messages for this portion of the day:\n\n{lines}\n\n\
             WRITING INSTRUCTIONS:\n\
             - Transform these messages into vivid literary prose in second person, present tense.\n\
             - Every significant moment must land in the prose — a realization, a decision, a line that mattered, an emotional shift. Do not smooth them into summary.\n\
             - Capture the essence of memorable lines. Quote verbatim if a line is perfect as-is, or lightly paraphrase for rhythm and clarity — creative liberty with the exact wording is welcome.\n\
             {weight}\n\
             - Do NOT include a section heading or title. Write only the prose.\n\
             - Do NOT write \"The End.\" or similar closings unless this is the final section.",
            shape = shape_hint,
            name = section_name,
            day = world_day,
            lines = section_text,
            weight = weight_line,
        )
    } else {
        if !silent {
            let _ = app_handle.emit("novel-phase", serde_json::json!({
                "phase": "beats",
                "section": section.label,
            }));
        }
        let beats = extract_beats(app_handle, model_config, api_key, &section.lines, &section.label, world_day, silent).await?;
        let beat_count = beats.len();
        let beats_joined = beats.iter()
            .enumerate()
            .map(|(i, b)| format!("{}. {}", i + 1, b))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{shape}\n\n\
             You are writing the {name} of Day {day}'s chapter.\n\n\
             NARRATIVE BEATS for this section (in chronological order, {count} total):\n\n{beats}\n\n\
             WRITING INSTRUCTIONS:\n\
             - Walk through all {count} beats in order, expanding each into rich literary prose.\n\
             - Every beat must land in the prose — no skipping, no vague summarizing.\n\
             - Capture the essence of memorable lines. Quote verbatim if a line is perfect as-is, or lightly paraphrase for rhythm and clarity — creative liberty with the exact wording is welcome.\n\
             {weight}\n\
             - Write in second person, present tense.\n\
             - Do NOT include a section heading or title. Write only the prose.\n\
             - Do NOT write \"The End.\" or similar closings unless this is the final section.",
            shape = shape_hint,
            name = section_name,
            day = world_day,
            count = beat_count,
            beats = beats_joined,
            weight = weight_line,
        )
    };

    if !silent {
        let _ = app_handle.emit("novel-phase", serde_json::json!({
            "phase": "section",
            "section": section.label,
            "section_index": section_index,
            "total_sections": total_sections,
        }));
    }

    let request = StreamingRequest {
        model: model_config.dialogue_model.clone(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: base_system.to_string() },
            openai::ChatMessage { role: "user".to_string(), content: user_content },
        ],
        temperature: Some(0.95),
        max_completion_tokens: Some(4_096),
        stream: true,
    };
    let raw = if silent {
        openai::chat_completion_stream_silent(&model_config.chat_api_base(), api_key, &request).await?
    } else {
        openai::chat_completion_stream(
            &model_config.chat_api_base(), api_key, &request, app_handle, "novel-token",
        ).await?
    };
    // Same tail cleanup we use for chat replies — a section that runs out of
    // completion tokens mid-sentence or leaves a dangling quote/paren/asterisk
    // would be jarring at a section boundary right before the divider. The
    // frontend still shows the raw streamed text during generation, but the
    // returned content (which becomes novel_entries.content after save) is
    // clean. When the saved entry reloads after onNovelChange(), the view
    // updates to the cleaned version automatically.
    let trimmed = orchestrator::trim_to_last_complete_sentence(&raw);
    let base = if trimmed.is_empty() { raw.as_str() } else { trimmed.as_str() };
    Ok(orchestrator::balance_trailing_openers(base))
}

/// Per-section shape hint — told to the chapter writer so opening /
/// middle / closing sections each know what they are.
fn section_shape(index: usize, total: usize) -> &'static str {
    if total == 1 {
        "This is a complete, single-section chapter. Open on a specific image, build through the middle, and land on a moment of resonance."
    } else if index == 0 {
        "This is the OPENING section of a multi-section chapter. Open on a specific, evocative image drawn from the first beat or first message of this section. Establish atmosphere. Do NOT conclude the chapter — a later section will close it."
    } else if index + 1 == total {
        "This is the FINAL section of a multi-section chapter. Assume earlier sections have already established the day. Close the chapter on a resonant final image or line."
    } else {
        "This is a MIDDLE section of a multi-section chapter. Assume an earlier section opened the day and a later section will close it. Continue the narrative arc — advance, escalate, or shift. Do NOT open with a fresh scene-setting paragraph or close with a farewell."
    }
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
            .filter(|m| m.world_day == Some(world_day) && m.role != "illustration" && m.role != "video" && m.role != "inventory_update")
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

    // Group messages into sections by world_time (Morning, Afternoon,
    // Evening, ...). Each section will be written independently and stitched
    // together with dividers in the final chapter. Days without time-of-day
    // info collapse to a single untagged section.
    let sections = group_into_sections(&messages, &user_name, &character_names);

    // Flat conversation text, used only by the single-shot (non-local or
    // tiny-day) fallback path below and for estimating total token size.
    let mut conversation: Vec<String> = Vec::new();
    let mut last_label = String::new();
    for s in &sections {
        if !s.label.is_empty() && s.label != last_label {
            conversation.push(format!("[It is now {}.]", s.label));
            last_label = s.label.clone();
        }
        for line in &s.lines { conversation.push(line.clone()); }
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
        if let Some(block) = crate::ai::prompts::empiricon_reader_substrate(c) {
            desc.push_str("\n\n");
            desc.push_str(&block);
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

    // Phased "beats → chapter" is the default for local models regardless of
    // day size. Local models produce more coherent chapter shape when given
    // a numbered-beats skeleton than when handed a raw transcript — even on
    // shorter days that would fit in one shot. OpenAI keeps single-shot: it
    // handles long transcripts cleanly and we'd rather save the round-trips.
    //
    // Exception: for absurdly small days (a handful of lines) beat extraction
    // adds no signal, so fall through to single-shot even on local.
    let is_local = model_config.is_local();
    let est_prompt_tokens = approx_tokens(&system_prompt) + approx_tokens(&conversation_text) + 200;
    let needs_chunking = is_local && conversation.len() >= 4 && est_prompt_tokens >= 800;

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

    // ── Sectioned generation ─────────────────────────────────────────────
    // Each section streams into the same `novel-token` channel. Between
    // sections we emit a divider token directly so the final stored text
    // contains the section break inline.
    //
    // Per-section length target: a section with ~3% of the day's content
    // shouldn't come out the same length as a section with ~90%. We compute
    // each section's token share of the day and tell the model, in words,
    // roughly how much prose that deserves. Total chapter target scales a
    // bit with day length but is clamped to [1500, 5000] words.
    let section_content_tokens: Vec<usize> = sections.iter()
        .map(|s| approx_tokens(&s.lines.join("\n")))
        .collect();
    let total_content_tokens: usize = section_content_tokens.iter().sum();
    let total_chapter_words: usize = (total_content_tokens * 3 / 4).clamp(1_500, 5_000);
    let total = sections.len();
    let mut full_chapter = String::new();
    for (i, section) in sections.iter().enumerate() {
        let ratio = if total_content_tokens > 0 {
            section_content_tokens[i] as f64 / total_content_tokens as f64
        } else {
            1.0 / (total as f64).max(1.0)
        };
        let target_words = ((ratio * total_chapter_words as f64).round() as usize).max(150);

        let section_text = stream_section(
            &app_handle,
            &model_config,
            &api_key,
            &system_prompt,
            section,
            i,
            total,
            world_day,
            target_words,
            false, // silent = false — foreground, emit events to UI
        ).await?;
        full_chapter.push_str(&section_text);

        if i + 1 < total {
            let _ = app_handle.emit("novel-token", SECTION_DIVIDER);
            full_chapter.push_str(SECTION_DIVIDER);
        }
    }
    Ok(full_chapter)
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

// ─── Background novelization ─────────────────────────────────────────────

/// Return every (thread_id, world_day, is_group) triple that has at least
/// one message but no novel_entries row yet. Deterministic order (by
/// thread, then day).
fn list_unnovelized_days(conn: &rusqlite::Connection) -> Result<Vec<(String, i64, bool)>, rusqlite::Error> {
    let mut out = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT DISTINCT m.thread_id, m.world_day
         FROM messages m
         LEFT JOIN novel_entries n ON n.thread_id = m.thread_id AND n.world_day = m.world_day
         WHERE m.world_day IS NOT NULL AND n.novel_id IS NULL
         ORDER BY m.thread_id, m.world_day"
    )?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, false)))?;
    for r in rows.flatten() { out.push(r); }

    let mut stmt = conn.prepare(
        "SELECT DISTINCT gm.thread_id, gm.world_day
         FROM group_messages gm
         LEFT JOIN novel_entries n ON n.thread_id = gm.thread_id AND n.world_day = gm.world_day
         WHERE gm.world_day IS NOT NULL AND n.novel_id IS NULL
         ORDER BY gm.thread_id, gm.world_day"
    )?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, true)))?;
    for r in rows.flatten() { out.push(r); }

    Ok(out)
}

/// Silent novelization for a single day — mirrors generate_novel_entry_cmd's
/// setup but uses stream_section in silent mode so no Tauri events fire.
/// Returns the full chapter text. Safe to call from a spawned background
/// task; dropping the future (via task abort) closes the HTTP streams and
/// stops local-model generation within ~a token batch.
async fn run_day_novel_silent(
    app_handle: &AppHandle,
    api_key: &str,
    thread_id: &str,
    world_day: i64,
    is_group: bool,
) -> Result<String, String> {
    let db = app_handle.state::<Database>();
    let (messages, world, characters, character_names, user_name, user_profile, model_config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let all_msgs = if is_group {
            get_all_group_messages(&conn, thread_id).map_err(|e| e.to_string())?
        } else {
            get_all_messages(&conn, thread_id).map_err(|e| e.to_string())?
        };
        let day_msgs: Vec<Message> = all_msgs.into_iter()
            .filter(|m| m.world_day == Some(world_day) && m.role != "illustration" && m.role != "video" && m.role != "inventory_update")
            .collect();
        if day_msgs.is_empty() { return Err("No messages for this day.".to_string()); }
        let world_id: String = conn.query_row(
            "SELECT world_id FROM threads WHERE thread_id = ?1",
            rusqlite::params![thread_id], |r| r.get(0),
        ).or_else(|_| conn.query_row(
            "SELECT world_id FROM group_chats WHERE thread_id = ?1",
            rusqlite::params![thread_id], |r| r.get(0),
        )).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let user_name = get_user_profile(&conn, &world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the protagonist".to_string());
        let characters = list_characters(&conn, &world_id).unwrap_or_default();
        let char_names: HashMap<String, String> = characters.iter()
            .map(|c| (c.character_id.clone(), c.display_name.clone()))
            .collect();
        let user_profile = get_user_profile(&conn, &world_id).ok();
        (day_msgs, world, characters, char_names, user_name, user_profile, model_config)
    };

    let sections = group_into_sections(&messages, &user_name, &character_names);
    if sections.is_empty() { return Err("No content to novelize.".to_string()); }

    let char_descriptions: Vec<String> = characters.iter().map(|c| {
        let mut desc = format!("- {}", c.display_name);
        if !c.identity.is_empty() {
            desc.push_str(&format!(": {}", c.identity));
        }
        let voice_rules = crate::ai::prompts::json_array_to_strings(&c.voice_rules);
        if !voice_rules.is_empty() {
            desc.push_str(&format!("\n  Voice: {}", voice_rules.join("; ")));
        }
        if let Some(block) = crate::ai::prompts::empiricon_reader_substrate(c) {
            desc.push_str("\n\n");
            desc.push_str(&block);
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
- Invent freely, but with restraint. Choose one or two precise sensory details per beat rather than cataloguing everything.
- Expand brief exchanges into full scenes with atmosphere and pacing.
- Include all the key beats from the conversation but enhance them with novelistic craft.
- Make it feel like one vivid, cohesive chapter — not a transcript.
- Do NOT include chapter titles, headers, or meta-commentary. Just the prose."#,
        world_desc = if world.description.is_empty() { "A richly detailed world." } else { &world.description },
        user_desc = user_desc,
        char_list = char_descriptions.join("\n"),
    );

    let section_content_tokens: Vec<usize> = sections.iter()
        .map(|s| approx_tokens(&s.lines.join("\n")))
        .collect();
    let total_content_tokens: usize = section_content_tokens.iter().sum();
    let total_chapter_words: usize = (total_content_tokens * 3 / 4).clamp(1_500, 5_000);
    let total = sections.len();
    let mut full_chapter = String::new();
    for (i, section) in sections.iter().enumerate() {
        let ratio = if total_content_tokens > 0 {
            section_content_tokens[i] as f64 / total_content_tokens as f64
        } else {
            1.0 / (total as f64).max(1.0)
        };
        let target_words = ((ratio * total_chapter_words as f64).round() as usize).max(150);
        let section_text = stream_section(
            app_handle, &model_config, api_key, &system_prompt,
            section, i, total, world_day, target_words,
            true, // silent
        ).await?;
        full_chapter.push_str(&section_text);
        if i + 1 < total {
            full_chapter.push_str(SECTION_DIVIDER);
        }
    }

    Ok(full_chapter)
}

/// Kick off a background sweep that novelizes every un-novelized day in
/// order. No-op when the provider isn't local (OpenAI cost guard) or no
/// days need work. Aborts any prior sweep still running.
#[tauri::command]
pub async fn run_background_novelization_cmd(
    app_handle: AppHandle,
    bg: State<'_, BgNovelHandle>,
    api_key: String,
) -> Result<(), String> {
    // Abort any prior handle before launching a new sweep.
    {
        let mut guard = bg.0.lock().await;
        if let Some(h) = guard.take() { h.abort(); }
    }

    let db_state = app_handle.state::<Database>();
    let is_local = {
        let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
        orchestrator::load_model_config(&conn).is_local()
    };
    if !is_local {
        return Ok(());
    }

    let todo = {
        let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
        list_unnovelized_days(&conn).map_err(|e| e.to_string())?
    };
    if todo.is_empty() { return Ok(()); }

    let ah = app_handle.clone();
    let _ = ah.emit("bg-novelize", serde_json::json!({ "status": "started", "pending": todo.len() }));
    let handle = tokio::spawn(async move {
        for (thread_id, world_day, is_group) in todo {
            let _ = ah.emit("bg-novelize", serde_json::json!({
                "status": "working",
                "thread_id": thread_id,
                "world_day": world_day,
            }));
            match run_day_novel_silent(&ah, &api_key, &thread_id, world_day, is_group).await {
                Ok(content) => {
                    let db = ah.state::<Database>();
                    let now = Utc::now().to_rfc3339();
                    let entry = NovelEntry {
                        novel_id: uuid::Uuid::new_v4().to_string(),
                        thread_id: thread_id.clone(),
                        world_day,
                        content,
                        created_at: now.clone(),
                        updated_at: now,
                    };
                    if let Ok(conn) = db.conn.lock() {
                        let _ = upsert_novel_entry(&conn, &entry);
                    }
                    let _ = ah.emit("bg-novelize", serde_json::json!({
                        "status": "saved",
                        "thread_id": thread_id,
                        "world_day": world_day,
                    }));
                }
                Err(e) => {
                    let _ = ah.emit("bg-novelize", serde_json::json!({
                        "status": "error",
                        "thread_id": thread_id,
                        "world_day": world_day,
                        "error": e,
                    }));
                }
            }
        }
        let _ = ah.emit("bg-novelize", serde_json::json!({ "status": "done" }));
    });

    let mut guard = bg.0.lock().await;
    *guard = Some(handle);
    Ok(())
}

/// Abort the currently-running background novelization sweep, if any.
/// Dropping the task future closes all in-flight HTTP streams, which the
/// local model detects and halts generation for promptly.
#[tauri::command]
pub async fn cancel_background_novelization_cmd(
    app_handle: AppHandle,
    bg: State<'_, BgNovelHandle>,
) -> Result<(), String> {
    let mut guard = bg.0.lock().await;
    if let Some(h) = guard.take() {
        h.abort();
        let _ = app_handle.emit("bg-novelize", serde_json::json!({ "status": "canceled" }));
    }
    Ok(())
}

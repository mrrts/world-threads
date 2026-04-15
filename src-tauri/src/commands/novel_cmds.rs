use crate::ai::openai::{self, StreamingRequest};
use crate::ai::orchestrator;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use tauri::{AppHandle, State};

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

    // Build conversation text
    let conversation: Vec<String> = messages.iter()
        .map(|m| {
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
            format!("{}: {}", speaker, m.content)
        })
        .collect();

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
                conversation.join("\n"),
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

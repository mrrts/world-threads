use crate::ai::{openai, orchestrator, prompts};
use crate::commands::illustration_cmds::png_aspect_ratio;
use crate::db::queries::*;
use crate::db::Database;
use crate::PortraitsDir;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

// ─── Imagined Chapter — three-stage telephone pipeline ──────────────────────
//
// 1. Invent a specific visual moment for this chat's cast (LLM, JSON output).
// 2. Render the invented moment as an illustration with character + user
//    portraits attached as reference images.
// 3. Feed ONLY the image + labeled portraits into a vision-aware model and
//    stream a chapter that ANSWERS the image. The step-1 prose is never
//    shown to step 3 — image-first inversion is the feature.
//
// Streaming events (emitted to the frontend during generation):
//   - "imagined-chapter-stage" : { phase, chapter_id, ... }
//   - "imagined-chapter-image" : { chapter_id, data_url }
//   - "imagined-chapter-token" : <text chunk>
//   - "imagined-chapter-done"  : { chapter_id, title, content }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateImaginedChapterRequest {
    /// Either thread_id (for solo) or thread_id resolved from group_chat — both paths converge on a thread_id.
    pub thread_id: String,
    /// Optional user-provided hint for what they want to read.
    pub seed_hint: Option<String>,
    /// Continue from the most-recent prior chapter for this thread.
    pub continue_from_previous: bool,
    /// Image quality tier ("low" / "medium" / "high"). Defaults to "medium".
    pub image_tier: Option<String>,
    /// Profundity dial: "Glimpse" / "Opening" / "Deep" / "Sacred".
    /// None / unrecognized → no depth directive (model picks). Default
    /// in the UI is "Opening" — the natural register for chapters that
    /// want to mean something without being seismic.
    pub depth: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateImaginedChapterResponse {
    pub chapter_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChapterStageEvent {
    chapter_id: String,
    phase: &'static str,
    title: Option<String>,
    tone_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChapterImageEvent {
    chapter_id: String,
    data_url: String,
    image_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChapterDoneEvent {
    chapter_id: String,
    title: String,
    content: String,
}

/// Resolve a thread_id to (world, cast: Vec<Character>) — works for both
/// solo threads (1 character) and group threads (N characters via group_chats).
fn resolve_thread_cast(
    conn: &rusqlite::Connection,
    thread_id: &str,
) -> Result<(World, Vec<Character>), String> {
    // First try solo: threads.character_id
    let solo_result: rusqlite::Result<(Option<String>, String)> = conn.query_row(
        "SELECT character_id, world_id FROM threads WHERE thread_id = ?1",
        params![thread_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    );

    let (world, characters) = match solo_result {
        Ok((Some(char_id), world_id)) => {
            let world = get_world(conn, &world_id).map_err(|e| e.to_string())?;
            let ch = get_character(conn, &char_id).map_err(|e| e.to_string())?;
            (world, vec![ch])
        }
        _ => {
            // Group: look up group_chats by thread_id
            let (world_id, character_ids_json): (String, String) = conn.query_row(
                "SELECT world_id, character_ids FROM group_chats WHERE thread_id = ?1",
                params![thread_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            ).map_err(|_| format!("thread {thread_id} is neither solo nor a group chat"))?;
            let world = get_world(conn, &world_id).map_err(|e| e.to_string())?;
            let ids: Vec<String> = serde_json::from_str(&character_ids_json).unwrap_or_default();
            let mut chars: Vec<Character> = Vec::new();
            for cid in &ids {
                if let Ok(c) = get_character(conn, cid) {
                    chars.push(c);
                }
            }
            (world, chars)
        }
    };
    if characters.is_empty() {
        return Err("no characters in thread".to_string());
    }
    Ok((world, characters))
}

#[tauri::command]
pub async fn generate_imagined_chapter_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    app_handle: AppHandle,
    api_key: String,
    request: GenerateImaginedChapterRequest,
) -> Result<GenerateImaginedChapterResponse, String> {
    if api_key.trim().is_empty() {
        return Err("no API key".to_string());
    }

    // ─── Load everything from the DB up front ───────────────────────────
    let (
        world,
        cast_owned,
        user_profile_owned,
        recent_kept_facts,
        cast_journals_owned,
        recent_history,
        portrait_files_by_name,
        user_portrait_file,
        previous_chapter_content,
        narration_tone,
        model_config,
    ) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let (world, cast) = resolve_thread_cast(&conn, &request.thread_id)?;
        let user_profile = get_user_profile(&conn, &world.world_id).ok();

        // Recent canonized facts about anyone in the cast (a few, capped).
        let mut kept_facts: Vec<String> = Vec::new();
        for c in &cast {
            if let Ok(rows) = conn.prepare(
                "SELECT content FROM kept_records
                 WHERE subject_type = 'character' AND subject_id = ?1
                 ORDER BY created_at DESC LIMIT 2"
            ).and_then(|mut s| {
                s.query_map(params![c.character_id], |r| r.get::<_, String>(0))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect::<Vec<_>>())
            }) {
                for r in rows { kept_facts.push(r); }
            }
        }

        // One most-recent journal per character.
        let mut journals: Vec<(String, JournalEntry)> = Vec::new();
        for c in &cast {
            if let Ok(entries) = list_journal_entries(&conn, &c.character_id, 1) {
                if let Some(e) = entries.into_iter().next() {
                    journals.push((c.display_name.clone(), e));
                }
            }
        }

        // Merged recent history across the cast's threads. We pull per
        // primary character (first in cast), which covers their solo +
        // every group they're in. For group chats, all members share the
        // group threads, so the primary's merged view captures everything
        // the group has been through. Capped to 40 lines.
        let user_display = user_profile.as_ref()
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| "the user".to_string());
        let recent_history = if let Some(primary) = cast.first() {
            gather_character_recent_messages(&conn, &primary.character_id, &user_display, 40)
        } else {
            Vec::new()
        };

        // Active portrait file per character.
        let mut portrait_files: Vec<(String, String)> = Vec::new();
        for c in &cast {
            if let Some(p) = get_active_portrait(&conn, &c.character_id) {
                portrait_files.push((c.display_name.clone(), p.file_name));
            }
        }
        let user_portrait = user_profile.as_ref()
            .filter(|p| !p.avatar_file.is_empty())
            .map(|p| p.avatar_file.clone());

        // Previous chapter content if requested + available.
        let prev = if request.continue_from_previous {
            let chapters = list_imagined_chapters_for_thread(&conn, &request.thread_id)
                .unwrap_or_default();
            chapters.into_iter().next().map(|c| c.content)
        } else { None };

        // CONTEXT SCOPING — two distinct modes, no middle ground:
        //
        //   continue_from_previous = false (default): the new chapter is
        //     a FRESH invention. Send no previous-chapter content. Send
        //     world, cast, history, journals, kept facts — the model
        //     reads all of these to know who these people are and where
        //     they stand right now.
        //
        //   continue_from_previous = true: the new chapter is the next
        //     beat AFTER the prior chapter. Send ONLY the previous
        //     chapter (plus the structural minimums: world + cast +
        //     user + tone, which the prompt physically needs). NO
        //     recent_history, NO journals, NO kept_facts — those
        //     feeds would pull the continuation off-course toward
        //     whatever the chats have been about lately. The continuation
        //     should anchor on the prior chapter, full stop.
        //
        // The user wanted these two modes to be cleanly separate so
        // back-to-back fresh chapters don't all converge on the same
        // theme (knots/docks/kayaks etc.) and continuations don't drift
        // away from the prior chapter's thread.
        let (kept_facts, journals, recent_history) = if prev.is_some() {
            (Vec::new(), Vec::new(), Vec::new())
        } else {
            (kept_facts, journals, recent_history)
        };

        // Per-chat narration tone — keyed identically to chat_cmds /
        // group_chat_cmds. Empty string + "Auto" both mean "no tone";
        // tone_directive() filters those out.
        let tone_setting_key = if cast.len() == 1 {
            format!("narration_tone.{}", cast[0].character_id)
        } else {
            // Group: keyed on group_chat_id resolved from thread_id.
            conn.query_row(
                "SELECT group_chat_id FROM group_chats WHERE thread_id = ?1",
                params![request.thread_id], |r| r.get::<_, String>(0),
            ).map(|gid| format!("narration_tone.{}", gid)).unwrap_or_default()
        };
        let narration_tone: Option<String> = if tone_setting_key.is_empty() { None }
            else { get_setting(&conn, &tone_setting_key).ok().flatten()
                .filter(|s| !s.trim().is_empty() && s != "Auto") };

        let mut model_config = orchestrator::load_model_config(&conn);
        // Honor the per-chat provider override that lives at
        // `provider_override.<character_id>` for solo threads and
        // `provider_override.<group_chat_id>` for group threads. This is
        // the same key chat_cmds + group_chat_cmds use, so a chapter
        // generated from a chat will use whatever provider that chat is
        // configured to use.
        let override_key: Option<String> = if cast.len() == 1 {
            // Solo: keyed on character_id.
            Some(format!("provider_override.{}", cast[0].character_id))
        } else {
            // Group: keyed on group_chat_id resolved from thread_id.
            conn.query_row(
                "SELECT group_chat_id FROM group_chats WHERE thread_id = ?1",
                params![request.thread_id],
                |r| r.get::<_, String>(0),
            ).ok().map(|gid| format!("provider_override.{}", gid))
        };
        if let Some(key) = override_key.as_deref() {
            model_config.apply_provider_override(&conn, key);
        }
        (
            world,
            cast,
            user_profile,
            kept_facts,
            journals,
            recent_history,
            portrait_files,
            user_portrait,
            prev,
            narration_tone,
            model_config,
        )
    };

    // Borrow-form for prompt builders.
    let cast_refs: Vec<&Character> = cast_owned.iter().collect();
    let user_profile_ref = user_profile_owned.as_ref();

    // Create the chapter row up-front so the frontend has an id to anchor on.
    let chapter_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let world_day = world.state.get("time")
        .and_then(|t| t.get("day_index"))
        .and_then(|v| v.as_i64());
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let row = ImaginedChapter {
            chapter_id: chapter_id.clone(),
            thread_id: request.thread_id.clone(),
            world_day,
            title: String::new(),
            seed_hint: request.seed_hint.clone().unwrap_or_default(),
            scene_description: String::new(),
            image_id: None,
            content: String::new(),
            created_at: now.clone(),
            breadcrumb_message_id: None,
            canonized: false,
        };
        create_imagined_chapter(&conn, &row).map_err(|e| e.to_string())?;
    }

    // ─── Stage 1: invent the scene ──────────────────────────────────────
    let _ = app_handle.emit("imagined-chapter-stage", ChapterStageEvent {
        chapter_id: chapter_id.clone(),
        phase: "inventing",
        title: None,
        tone_hint: None,
    });

    let (invented, invent_usage) = orchestrator::invent_scene_for_chapter(
        &model_config.chat_api_base(),
        &api_key,
        &model_config.dialogue_model,
        &world,
        &cast_refs,
        user_profile_ref,
        &recent_kept_facts,
        &cast_journals_owned,
        &recent_history,
        request.seed_hint.as_deref(),
        narration_tone.as_deref(),
        previous_chapter_content.as_deref(),
        request.depth.as_deref(),
    ).await?;

    if let Some(u) = &invent_usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "imagined_chapter_scene", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    // Persist title + scene description on the row.
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = conn.execute(
            "UPDATE imagined_chapters SET title = ?2, scene_description = ?3 WHERE chapter_id = ?1",
            params![chapter_id, invented.title, invented.image_prompt],
        );
    }

    let _ = app_handle.emit("imagined-chapter-stage", ChapterStageEvent {
        chapter_id: chapter_id.clone(),
        phase: "rendering",
        title: Some(invented.title.clone()),
        tone_hint: Some(invented.tone_hint.clone()),
    });

    // ─── Stage 2: render the image ──────────────────────────────────────
    let dir = &portraits_dir.0;
    let mut reference_images: Vec<Vec<u8>> = Vec::new();
    let mut reference_labels: Vec<String> = Vec::new();

    // User avatar first (if present), then each character's active portrait.
    if let Some(file) = &user_portrait_file {
        if let Ok(bytes) = std::fs::read(dir.join(file)) {
            reference_images.push(bytes);
            reference_labels.push(
                user_profile_owned.as_ref().map(|p| p.display_name.clone()).unwrap_or_else(|| "the user".to_string()),
            );
        }
    }
    for (name, file) in &portrait_files_by_name {
        if let Ok(bytes) = std::fs::read(dir.join(file)) {
            reference_images.push(bytes);
            reference_labels.push(name.clone());
        }
    }

    let tier = request.image_tier.as_deref().unwrap_or("medium");
    let (img_size, img_quality) = match tier {
        "low" => ("1024x1024", "low"),
        "high" => ("1536x1024", "high"),
        _ => ("1536x1024", "medium"),
    };

    // Names map for the existing illustration helper. We pass cast names so
    // the prompt's "Reference image N is X" labels line up.
    let all_names: Vec<String> = portrait_files_by_name.iter().map(|(n, _)| n.clone()).collect();

    // Tone-shape the image prompt so the painted scene's mood matches the
    // chat's tone setting. Brief — we trust the scene-invention pass to
    // already carry the tone in its visual specifics; this is reinforcement.
    let image_prompt_with_tone: String = match narration_tone.as_deref() {
        Some(t) => format!(
            "{}\n\nVisual tone: {}. Let the light, color palette, posture, and overall atmosphere carry this register.",
            invented.image_prompt, t,
        ),
        None => invented.image_prompt.clone(),
    };

    // Use the existing illustration pipeline with include_scene_summary=false
    // — the scene description we just invented is passed as custom_instructions.
    // This is intentionally a chapter-owned scene, not a live chat scene-state
    // repair surface; if imagined chapters ever need authoritative location,
    // that should be stored on the chapter itself rather than borrowed from
    // thread current_location.
    let primary_char = cast_refs[0];
    let additional: Vec<&Character> = cast_refs.iter().skip(1).copied().collect();
    let (_used_scene_desc, image_bytes, image_chat_usage) = orchestrator::generate_illustration_with_base(
        &model_config.chat_api_base(),
        &model_config.openai_api_base(),
        &api_key,
        &model_config.dialogue_model,
        &model_config.image_model,
        img_quality,
        img_size,
        model_config.image_output_format().as_deref(),
        &world,
        primary_char,
        if additional.is_empty() { None } else { Some(&additional[..]) },
        &[], // recent_messages — unused when include_scene_summary=false
        user_profile_ref,
        &reference_images,
        Some(&image_prompt_with_tone),
        false, // has_previous_scene
        false, // include_scene_summary — we already have the description
        if all_names.is_empty() { None } else { Some(&all_names[..]) },
        None,
    None,
    ).await?;

    if let Some(u) = &image_chat_usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "imagined_chapter_image", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    // Save image to disk + world_images.
    let image_id = uuid::Uuid::new_v4().to_string();
    let image_file = format!("imagined_chapter_{image_id}.png");
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&image_file), &image_bytes)
        .map_err(|e| format!("Failed to save image: {e}"))?;

    let aspect = png_aspect_ratio(&image_bytes);
    let b64 = orchestrator::base64_encode_bytes(&image_bytes);
    let data_url = format!("data:image/png;base64,{b64}");

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let img = WorldImage {
            image_id: image_id.clone(),
            world_id: world.world_id.clone(),
            prompt: invented.image_prompt.clone(),
            file_name: image_file.clone(),
            is_active: false,
            source: "imagined_chapter".to_string(),
            created_at: Utc::now().to_rfc3339(),
            aspect_ratio: aspect,
            caption: invented.title.clone(),
        };
        let _ = create_world_image(&conn, &img);
        let _ = set_imagined_chapter_image(&conn, &chapter_id, &image_id);
    }

    let _ = app_handle.emit("imagined-chapter-image", ChapterImageEvent {
        chapter_id: chapter_id.clone(),
        data_url: data_url.clone(),
        image_id: image_id.clone(),
    });

    let _ = app_handle.emit("imagined-chapter-stage", ChapterStageEvent {
        chapter_id: chapter_id.clone(),
        phase: "writing",
        title: Some(invented.title.clone()),
        tone_hint: Some(invented.tone_hint.clone()),
    });

    // ─── Stage 3: stream the chapter from the image ─────────────────────
    let system_prompt = prompts::build_chapter_from_image_system_prompt(
        &world,
        &cast_refs,
        user_profile_ref,
        &cast_journals_owned,
        &recent_history,
        narration_tone.as_deref(),
        previous_chapter_content.as_deref(),
        request.depth.as_deref(),
    );

    // Build vision content: a brief framing line, the scene image, then
    // each character's portrait labeled with their name.
    let mut vision_content: Vec<openai::VisionContent> = Vec::new();
    vision_content.push(openai::VisionContent {
        content_type: "text".to_string(),
        text: Some(format!(
            "The image below is the scene this chapter is about. The portraits that follow are LABELED with the names of the people in this world — match faces in the scene to the labeled portraits, and name those people in the prose by name. Then write the chapter.\n\nScene:"
        )),
        image_url: None,
    });
    vision_content.push(openai::VisionContent {
        content_type: "image_url".to_string(),
        text: None,
        image_url: Some(openai::VisionImageUrl {
            url: data_url.clone(),
            detail: Some("high".to_string()),
        }),
    });
    for (label, file) in portrait_files_by_name.iter() {
        if let Ok(bytes) = std::fs::read(dir.join(file)) {
            let p_b64 = orchestrator::base64_encode_bytes(&bytes);
            vision_content.push(openai::VisionContent {
                content_type: "text".to_string(),
                text: Some(format!("Portrait of {label}:")),
                image_url: None,
            });
            vision_content.push(openai::VisionContent {
                content_type: "image_url".to_string(),
                text: None,
                image_url: Some(openai::VisionImageUrl {
                    url: format!("data:image/png;base64,{p_b64}"),
                    detail: Some("low".to_string()),
                }),
            });
        }
    }
    if let (Some(file), Some(profile)) = (&user_portrait_file, user_profile_owned.as_ref()) {
        if let Ok(bytes) = std::fs::read(dir.join(file)) {
            let p_b64 = orchestrator::base64_encode_bytes(&bytes);
            vision_content.push(openai::VisionContent {
                content_type: "text".to_string(),
                text: Some(format!("Portrait of {}:", profile.display_name)),
                image_url: None,
            });
            vision_content.push(openai::VisionContent {
                content_type: "image_url".to_string(),
                text: None,
                image_url: Some(openai::VisionImageUrl {
                    url: format!("data:image/png;base64,{p_b64}"),
                    detail: Some("low".to_string()),
                }),
            });
        }
    }
    vision_content.push(openai::VisionContent {
        content_type: "text".to_string(),
        text: Some("Now write the chapter.".to_string()),
        image_url: None,
    });

    let stream_request = openai::VisionStreamingRequest {
        // Honors the per-chat provider override applied above; falls back
        // to the global dialogue_model when no override is set.
        model: model_config.dialogue_model.clone(),
        messages: vec![
            openai::VisionMessage {
                role: "system".to_string(),
                content: vec![openai::VisionContent {
                    content_type: "text".to_string(),
                    text: Some(system_prompt),
                    image_url: None,
                }],
            },
            openai::VisionMessage {
                role: "user".to_string(),
                content: vision_content,
            },
        ],
        temperature: Some(0.85),
        max_completion_tokens: Some(2200),
        stream: true,
    };

    let chapter_text = openai::vision_completion_stream(
        &model_config.chat_api_base(),
        &api_key,
        &stream_request,
        &app_handle,
        "imagined-chapter-token",
    ).await?;

    // Save the final content. Note: NO breadcrumb is inserted here — the
    // chapter starts in the pre-canon state. The breadcrumb (and the
    // chat-history footprint that goes with it) only gets written when
    // the user explicitly canonizes via canonize_imagined_chapter_cmd.
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = update_imagined_chapter(&conn, &chapter_id, &invented.title, &chapter_text);
    }
    let _ = world_day; // silence unused warning when breadcrumb is moved out

    let _ = app_handle.emit("imagined-chapter-done", ChapterDoneEvent {
        chapter_id: chapter_id.clone(),
        title: invented.title.clone(),
        content: chapter_text,
    });

    Ok(GenerateImaginedChapterResponse { chapter_id })
}

#[tauri::command]
pub fn list_imagined_chapters_for_thread_cmd(
    db: State<Database>,
    thread_id: String,
) -> Result<Vec<ImaginedChapter>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    list_imagined_chapters_for_thread(&conn, &thread_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_imagined_chapter_cmd(
    db: State<Database>,
    chapter_id: String,
) -> Result<ImaginedChapter, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_imagined_chapter(&conn, &chapter_id).map_err(|e| e.to_string())
}

/// Delete a chapter and cascade to every artifact it owns:
///   • the breadcrumb message row in messages/group_messages (if canonized)
///   • the world_images row for the chapter illustration
///   • the image file on disk
///   • finally the imagined_chapters row itself
/// Pre-canon chapters skip the breadcrumb step (they never had one).
#[tauri::command]
pub fn delete_imagined_chapter_cmd(
    db: State<Database>,
    portraits_dir: State<'_, PortraitsDir>,
    chapter_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let chapter = get_imagined_chapter(&conn, &chapter_id).map_err(|e| e.to_string())?;

    if let Some(bc_id) = chapter.breadcrumb_message_id.as_deref() {
        let _ = conn.execute("DELETE FROM messages WHERE message_id = ?1", params![bc_id]);
        let _ = conn.execute("DELETE FROM group_messages WHERE message_id = ?1", params![bc_id]);
    }

    if let Some(img_id) = chapter.image_id.as_deref() {
        let file_name: Option<String> = conn.query_row(
            "SELECT file_name FROM world_images WHERE image_id = ?1",
            params![img_id], |r| r.get::<_, String>(0),
        ).ok();
        if let Some(file_name) = file_name {
            let _ = std::fs::remove_file(portraits_dir.0.join(&file_name));
        }
        let _ = conn.execute("DELETE FROM world_images WHERE image_id = ?1", params![img_id]);
    }

    delete_imagined_chapter(&conn, &chapter_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_imagined_chapter_cmd(
    db: State<Database>,
    chapter_id: String,
    title: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    rename_imagined_chapter(&conn, &chapter_id, &title).map_err(|e| e.to_string())
}

/// Read the saved illustration bytes for a chapter and return as a
/// data URL. Empty string if the chapter has no image (still rendering
/// or the file is missing). Used by the modal to display past chapters.
#[tauri::command]
pub fn get_imagined_chapter_image_url_cmd(
    db: State<Database>,
    portraits_dir: State<'_, PortraitsDir>,
    chapter_id: String,
) -> Result<String, String> {
    let file_name: Option<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let chapter = get_imagined_chapter(&conn, &chapter_id).map_err(|e| e.to_string())?;
        match chapter.image_id {
            Some(img_id) => conn.query_row(
                "SELECT file_name FROM world_images WHERE image_id = ?1",
                params![img_id], |r| r.get::<_, String>(0),
            ).ok(),
            None => None,
        }
    };
    let Some(file_name) = file_name else { return Ok(String::new()); };
    let path = portraits_dir.0.join(&file_name);
    if !path.exists() { return Ok(String::new()); }
    let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read image: {e}"))?;
    Ok(format!("data:image/png;base64,{}", orchestrator::base64_encode_bytes(&bytes)))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonizeImaginedChapterResponse {
    /// The newly-inserted breadcrumb's message_id, so the frontend can
    /// hand it directly to the existing canon-classifier flow if it
    /// wants to chain the chapter into character-data evolution.
    pub breadcrumb_message_id: String,
}

/// Bless a chapter into canon: insert the chat-history breadcrumb
/// row AND set canonized=true on the chapter record. Idempotent on
/// canonized chapters (returns the existing breadcrumb_message_id
/// without inserting a second one).
#[tauri::command]
pub fn canonize_imagined_chapter_cmd(
    db: State<Database>,
    chapter_id: String,
) -> Result<CanonizeImaginedChapterResponse, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let chapter = get_imagined_chapter(&conn, &chapter_id).map_err(|e| e.to_string())?;

    // Idempotent: if already canonized AND the breadcrumb still exists,
    // return its id without writing again.
    if chapter.canonized {
        if let Some(existing_id) = chapter.breadcrumb_message_id.as_deref() {
            return Ok(CanonizeImaginedChapterResponse {
                breadcrumb_message_id: existing_id.to_string(),
            });
        }
    }

    // Insert the breadcrumb row into the right messages table.
    let is_group: bool = conn.query_row(
        "SELECT 1 FROM group_chats WHERE thread_id = ?1",
        params![chapter.thread_id], |_| Ok(true),
    ).unwrap_or(false);
    let table = if is_group { "group_messages" } else { "messages" };
    let breadcrumb_id = uuid::Uuid::new_v4().to_string();
    let first_line: String = chapter.content.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .chars().take(200).collect();
    let content = serde_json::json!({
        "chapter_id": chapter.chapter_id,
        "title": chapter.title,
        "image_id": chapter.image_id,
        "first_line": first_line,
    }).to_string();
    let now = Utc::now().to_rfc3339();
    // World-day/time at canon time (which is when the chapter "enters" the chat).
    let (world_day, world_time): (Option<i64>, String) = conn.query_row(
        "SELECT w.state FROM threads t JOIN worlds w ON w.world_id = t.world_id WHERE t.thread_id = ?1
         UNION ALL
         SELECT w.state FROM group_chats gc JOIN worlds w ON w.world_id = gc.world_id WHERE gc.thread_id = ?1
         LIMIT 1",
        params![chapter.thread_id], |r| {
            let s: String = r.get(0)?;
            let v: serde_json::Value = serde_json::from_str(&s).unwrap_or_default();
            let day = v.get("time").and_then(|t| t.get("day_index")).and_then(|x| x.as_i64());
            let time = v.get("time").and_then(|t| t.get("time_of_day"))
                .and_then(|x| x.as_str()).unwrap_or("").to_string();
            Ok((day, time))
        },
    ).unwrap_or((chapter.world_day, String::new()));

    conn.execute(
        &format!(
            "INSERT INTO {table} (message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time)
             VALUES (?1, ?2, 'imagined_chapter', ?3, 0, NULL, ?4, ?5, ?6)"
        ),
        params![breadcrumb_id, chapter.thread_id, content, now, world_day, world_time],
    ).map_err(|e| format!("Failed to insert breadcrumb: {e}"))?;

    set_imagined_chapter_breadcrumb(&conn, &chapter_id, &breadcrumb_id).map_err(|e| e.to_string())?;
    set_imagined_chapter_canonized(&conn, &chapter_id, true).map_err(|e| e.to_string())?;

    Ok(CanonizeImaginedChapterResponse { breadcrumb_message_id: breadcrumb_id })
}

/// Reverse a chapter's canonization. Flips canonized=false, deletes
/// the breadcrumb message row from messages/group_messages, and
/// clears breadcrumb_message_id on the chapter. The chapter content
/// itself is preserved — only the chat-history footprint is removed,
/// and the chapter goes back to pre-canon state in the modal.
#[tauri::command]
pub fn decanonize_imagined_chapter_cmd(
    db: State<Database>,
    chapter_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let chapter = get_imagined_chapter(&conn, &chapter_id).map_err(|e| e.to_string())?;
    if let Some(bc_id) = chapter.breadcrumb_message_id.as_deref() {
        // Try both tables; whichever has it deletes one row.
        let _ = conn.execute("DELETE FROM messages WHERE message_id = ?1", params![bc_id]);
        let _ = conn.execute("DELETE FROM group_messages WHERE message_id = ?1", params![bc_id]);
    }
    conn.execute(
        "UPDATE imagined_chapters SET canonized = 0, breadcrumb_message_id = NULL WHERE chapter_id = ?1",
        params![chapter_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkDecanonizeResponse {
    pub decanonized_count: usize,
}

/// Bulk reset: decanonize every chapter in a thread. Useful as a
/// one-shot cleanup after the migration auto-canonized prior chapters
/// based on their existing breadcrumbs. Same per-chapter mechanics:
/// flips canonized=false, deletes the breadcrumb row, clears the FK.
#[tauri::command]
pub fn bulk_decanonize_imagined_chapters_for_thread_cmd(
    db: State<Database>,
    thread_id: String,
) -> Result<BulkDecanonizeResponse, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let chapters = list_imagined_chapters_for_thread(&conn, &thread_id)
        .map_err(|e| e.to_string())?;
    let mut count = 0usize;
    for chapter in chapters {
        if !chapter.canonized && chapter.breadcrumb_message_id.is_none() { continue; }
        if let Some(bc_id) = chapter.breadcrumb_message_id.as_deref() {
            let _ = conn.execute("DELETE FROM messages WHERE message_id = ?1", params![bc_id]);
            let _ = conn.execute("DELETE FROM group_messages WHERE message_id = ?1", params![bc_id]);
        }
        let _ = conn.execute(
            "UPDATE imagined_chapters SET canonized = 0, breadcrumb_message_id = NULL WHERE chapter_id = ?1",
            params![chapter.chapter_id],
        );
        count += 1;
    }
    Ok(BulkDecanonizeResponse { decanonized_count: count })
}

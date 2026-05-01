use crate::ai::google;
use crate::ai::openai::{self, ChatRequest};
use crate::ai::orchestrator;
use crate::commands::illustration_cmds::base64_encode_bytes;
use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use rusqlite::params;
use tauri::State;

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
    include_context: Option<bool>,
) -> Result<String, String> {
    let is_group = character_id.is_empty();

    // Load context
    let (world, character, recent_msgs, model_config, user_profile, illustration_file, current_loc) = {
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
        let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;

        let character = if is_group {
            // Dummy character for animation prompt
            let chars_in_world = list_characters(&conn, &world_id).unwrap_or_default();
            chars_in_world.into_iter().next().unwrap_or_else(|| Character {
                character_id: String::new(), world_id: world_id.clone(), display_name: String::new(),
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
            })
        } else {
            get_character(&conn, &character_id).map_err(|e| e.to_string())?
        };

        let user_profile = get_user_profile(&conn, &world_id).ok();

        // Get messages up to the illustration's creation time
        let msg_table = if is_group { "group_messages" } else { "messages" };
        let sql = format!(
            "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM {} WHERE thread_id = ?1 AND created_at <= ?2
             ORDER BY created_at DESC LIMIT 30", msg_table
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let mut recent_msgs: Vec<Message> = stmt.query_map(params![thread_id, illus_created_at], |row| {
            Ok(Message {
                message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
                content: row.get(3)?, tokens_estimate: row.get(4)?,
                sender_character_id: row.get(5)?, created_at: row.get(6)?,
                world_day: row.get(7).ok(), world_time: row.get(8).ok(),
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        formula_signature: None,
        })
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
        recent_msgs.reverse();

        let current_loc = if is_group {
            None
        } else {
            get_thread_location(&conn, &thread_id).ok().flatten()
        };

        (world, character, recent_msgs, model_config, user_profile, file_name, current_loc)
    };

    let dir = &portraits_dir.0;

    // Read the illustration image
    let image_bytes = std::fs::read(dir.join(&illustration_file))
        .map_err(|e| format!("Failed to read illustration: {e}"))?;
    let image_b64 = base64_encode_bytes(&image_bytes);

    // Generate animation prompt — optionally include conversation context
    let mut animation_prompt = if include_context.unwrap_or(false) {
        generate_animation_prompt(
            &api_key,
            &model_config,
            &world,
            &character,
            user_profile.as_ref(),
            &recent_msgs,
            current_loc.as_deref(),
        ).await?
    } else {
        "Bring this illustration to life with natural, subtle motion.".to_string()
    };
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
    world: &World,
    character: &Character,
    user_profile: Option<&UserProfile>,
    recent_msgs: &[Message],
    current_location_override: Option<&str>,
) -> Result<String, String> {
    use crate::ai::prompts;

    // Videos are only generated from individual chats today (the frontend
    // guards on activeCharacter), so no additional cast or names map.
    let messages = prompts::build_animation_prompt(
        world,
        character,
        None,
        user_profile,
        recent_msgs,
        None,
        current_location_override,
    );
    let request = ChatRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.95),
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

use crate::ai::orchestrator;
use crate::commands::chat_cmds::world_time_fields;
use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct IllustrationResult {
    pub illustration_message: Message,
}

/// Encode bytes to base64 string.
pub fn base64_encode_bytes(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Get aspect ratio (width/height) from PNG image bytes.
pub fn png_aspect_ratio(bytes: &[u8]) -> f64 {
    if bytes.len() >= 24 && &bytes[0..4] == b"\x89PNG" {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as f64;
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as f64;
        if h > 0.0 { w / h } else { 1.0 }
    } else {
        1.0
    }
}

/// Delete a single illustration: message, gallery entry, and file on disk.
pub(crate) fn delete_illustration_inner(conn: &rusqlite::Connection, portraits_dir: &std::path::Path, message_id: &str) -> Result<(), String> {
    // Delete associated video file if one exists
    let video_file: Option<String> = conn.query_row(
        "SELECT video_file FROM world_images WHERE image_id = ?1",
        params![message_id], |r| r.get(0),
    ).ok();
    if let Some(ref vf) = video_file {
        if !vf.is_empty() {
            let path = portraits_dir.join(vf);
            if path.exists() {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    // Delete gallery entry (linked by message_id = image_id)
    let file_name: Option<String> = conn.query_row(
        "SELECT file_name FROM world_images WHERE image_id = ?1",
        params![message_id], |r| r.get(0),
    ).ok();
    conn.execute("DELETE FROM world_images WHERE image_id = ?1", params![message_id])
        .map_err(|e| e.to_string())?;
    // Delete FTS entry
    conn.execute("DELETE FROM messages_fts WHERE message_id = ?1", params![message_id]).ok();
    // Delete message
    conn.execute("DELETE FROM messages WHERE message_id = ?1", params![message_id])
        .map_err(|e| e.to_string())?;
    // Delete illustration image file
    if let Some(f) = file_name {
        let path = portraits_dir.join(&f);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_illustration_captions_cmd(
    db: State<Database>,
    message_ids: Vec<String>,
) -> Result<std::collections::HashMap<String, String>, String> {
    if message_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let placeholders: Vec<String> = (1..=message_ids.len()).map(|i| format!("?{i}")).collect();
    let sql = format!(
        "SELECT image_id, caption FROM world_images WHERE image_id IN ({}) AND caption != ''",
        placeholders.join(", ")
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params_vec: Vec<&dyn rusqlite::types::ToSql> = message_ids.iter()
        .map(|id| id as &dyn rusqlite::types::ToSql)
        .collect();
    let rows = stmt.query_map(params_vec.as_slice(), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| e.to_string())?;
    let mut out = std::collections::HashMap::new();
    for r in rows.flatten() {
        out.insert(r.0, r.1);
    }
    Ok(out)
}

/// Persist a user-edited caption for an illustration. The caption is both
/// the alt-text shown in chat AND the text fed into future dialogue /
/// narrative / dream history as `[Illustration — {caption}]`, so editing
/// it updates what the LLM sees about that visual beat going forward.
#[tauri::command]
pub fn update_illustration_caption_cmd(
    db: State<Database>,
    message_id: String,
    caption: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE world_images SET caption = ?2 WHERE image_id = ?1",
        rusqlite::params![message_id, caption],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn generate_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
    quality_tier: Option<String>,
    custom_instructions: Option<String>,
    previous_illustration_id: Option<String>,
    include_scene_summary: Option<bool>,
) -> Result<IllustrationResult, String> {
    let (world, character, thread_id, recent_msgs, model_config, user_profile) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;
        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        (world, character, thread.thread_id, recent_msgs, model_config, user_profile)
    };

    // Load reference portraits: user avatar first, then character's active portrait
    let mut reference_images: Vec<Vec<u8>> = Vec::new();
    let dir = &portraits_dir.0;

    // User avatar
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

    // Character active portrait
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Some(portrait) = get_active_portrait(&conn, &character_id) {
            let path = dir.join(&portrait.file_name);
            if path.exists() {
                if let Ok(bytes) = std::fs::read(&path) {
                    reference_images.push(bytes);
                }
            }
        }
    }

    // Previous illustration as reference (if requested)
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

    // Resolve quality tier to image size and quality
    let tier = quality_tier.as_deref().unwrap_or("high");
    let (img_size, img_quality) = match tier {
        "low" => ("1024x1024", "low"),
        "medium" => ("1024x1024", "medium"),
        _ => ("1536x1024", "medium"),  // "high"
    };

    log::info!("[Illustration] Generating for '{}' with {} reference images (tier={}, size={}, quality={})",
        character.display_name, reference_images.len(), tier, img_size, img_quality);

    // When no user instructions are provided, do a quick LLM call to pick
    // a memorable moment from recent scene messages. The picked sentence
    // serves double duty: guides the image generation AND becomes the
    // stored caption/alt-text for the illustration.
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
                    log::warn!("[Illustration] memorable-moment pick failed: {e}; proceeding without");
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
        &world, &character, None, &recent_msgs,
        user_profile.as_ref(),
        &reference_images,
        resolved_instructions.as_deref(),
        has_previous,
        include_scene_summary.unwrap_or(true),
        None,
        None,
    ).await?;

    // Caption: user's instructions verbatim when provided; otherwise
    // derive from scene_description so it describes what was actually
    // painted. The pre-image "memorable moment" pick used to supply the
    // caption, but it could anchor on a different beat than the scene
    // director ended up painting — making the caption look like it
    // belonged to the previous illustration. Fall back to the memorable
    // moment if the compression call fails; empty string as last resort.
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

    // Use message_id as image_id so they're linked for cleanup
    let message_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("illustration_{message_id}.png");
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &image_bytes)
        .map_err(|e| format!("Failed to save illustration: {e}"))?;

    log::info!("[Illustration] Saved {} ({} bytes)", file_name, image_bytes.len());

    let aspect = png_aspect_ratio(&image_bytes);
    let b64 = base64_encode_bytes(&image_bytes);
    let data_url = format!("data:image/png;base64,{b64}");
    let now = Utc::now().to_rfc3339();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        // Save to world gallery (linked by message_id)
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

        let (wd_ill, wt_ill) = world_time_fields(&world);
        let msg = Message {
            message_id: message_id.clone(),
            thread_id: thread_id.clone(),
            role: "illustration".to_string(),
            content: data_url,
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: now,
            world_day: wd_ill, world_time: wt_ill,
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let illustration_msg = conn.query_row(
        "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM messages WHERE message_id = ?1",
        params![message_id], |row| Ok(Message {
            message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
            content: row.get(3)?, tokens_estimate: row.get(4)?, sender_character_id: row.get(5)?, created_at: row.get(6)?, world_day: row.get(7).ok(), world_time: row.get(8).ok(),
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        })
    ).map_err(|e| e.to_string())?;

    Ok(IllustrationResult {
        illustration_message: illustration_msg,
    })
}

#[tauri::command]
pub async fn delete_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    message_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    delete_illustration_inner(&conn, &portraits_dir.0, &message_id)
}

/// Get a single illustration's data URL by message ID.
#[tauri::command]
pub fn get_illustration_data_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    message_id: String,
) -> Result<Option<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let file_name: Option<String> = conn.query_row(
        "SELECT file_name FROM world_images WHERE image_id = ?1",
        params![message_id], |r| r.get(0),
    ).ok();
    if let Some(f) = file_name {
        let path = portraits_dir.0.join(&f);
        if path.exists() {
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            let b64 = base64_encode_bytes(&bytes);
            Ok(Some(format!("data:image/png;base64,{b64}")))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn regenerate_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
    message_id: String,
) -> Result<IllustrationResult, String> {
    // Delete the old illustration
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        delete_illustration_inner(&conn, &portraits_dir.0, &message_id)?;
    }

    // Generate a new one (reuses the full generate_illustration_cmd logic)
    generate_illustration_cmd(db, portraits_dir, api_key, character_id, Some("high".to_string()), None, None, None).await
}

#[tauri::command]
pub async fn adjust_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
    message_id: String,
    instructions: String,
) -> Result<IllustrationResult, String> {
    // Load the current illustration image, model config, and reference portraits
    let (image_bytes, world, character, thread, model_config, user_profile) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let user_profile = get_user_profile(&conn, &character.world_id).ok();

        // Read the current illustration file
        let file_name: String = conn.query_row(
            "SELECT file_name FROM world_images WHERE image_id = ?1",
            params![message_id], |r| r.get(0),
        ).map_err(|_| "Illustration not found in gallery".to_string())?;

        let path = portraits_dir.0.join(&file_name);
        let bytes = std::fs::read(&path)
            .map_err(|e| format!("Failed to read illustration file: {e}"))?;

        (bytes, world, character, thread, model_config, user_profile)
    };

    let dir = &portraits_dir.0;

    // Build reference images: current illustration first, then user avatar, then character portrait
    let mut reference_images: Vec<Vec<u8>> = vec![image_bytes];

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

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Some(portrait) = get_active_portrait(&conn, &character_id) {
            let path = dir.join(&portrait.file_name);
            if path.exists() {
                if let Ok(bytes) = std::fs::read(&path) {
                    reference_images.push(bytes);
                }
            }
        }
    }

    let user_name = user_profile.as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");

    // Build the adjustment prompt
    let time_lighting = world.state.get("time")
        .and_then(|t| t.get("time_of_day"))
        .and_then(|v| v.as_str())
        .map(|tod| match tod.to_uppercase().as_str() {
            "DAWN" => "Early dawn light, sky shifting from deep blue to warm gold at the horizon.",
            "MORNING" => "Bright warm morning light, clear and inviting.",
            "MIDDAY" => "High midday sun, strong overhead light with short crisp shadows.",
            "AFTERNOON" => "Warm golden afternoon light with long gentle rays.",
            "EVENING" | "DUSK" => "Dusky evening light, warm oranges and purples painting the sky, long dramatic shadows.",
            "NIGHT" => "Nighttime scene, moonlight and ambient glow, deep blues and soft shadows.",
            "LATE NIGHT" => "Deep night, very dark atmosphere, only dim moonlight or artificial light sources.",
            _ => "Gentle diffused natural lighting.",
        })
        .unwrap_or("Gentle diffused natural lighting.");

    let prompt_parts = vec![
        "Hand-painted watercolor illustration in a lush, realistic style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones.".to_string(),
        time_lighting.to_string(),
        "Wide cinematic composition.".to_string(),
        "The first reference image is the current illustration to adjust. Preserve its overall composition and scene.".to_string(),
        format!("The other reference images show {} and {}. Keep them recognizable.", user_name, character.display_name),
        format!("ADJUSTMENT INSTRUCTIONS:\n{instructions}"),
        "Apply the requested changes while keeping everything else about the scene intact.".to_string(),
        "CRITICAL: The image must contain absolutely no text, no words, no letters, no numbers, no writing, no labels, no titles, no captions, no watermarks, no signatures, no UI elements, no names.".to_string(),
    ];

    let prompt = prompt_parts.join(" ");

    log::info!("[Illustration Adjust] Adjusting with {} reference images, instructions: {:.100}", reference_images.len(), instructions);

    let response = crate::ai::openai::generate_image_edit_with_base(
        &model_config.openai_api_base(), &api_key, &model_config.image_model,
        &prompt, &reference_images,
        "1536x1024", model_config.image_quality(),
        model_config.image_output_format().as_deref(),
    ).await?;

    let b64 = response.data.first()
        .and_then(|d| d.image_b64())
        .ok_or_else(|| "No image data in response".to_string())?;

    let new_image_bytes = orchestrator::openai_base64_decode_pub(b64)?;

    // Delete the old illustration
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        delete_illustration_inner(&conn, &portraits_dir.0, &message_id)?;
    }

    // Save new image
    let new_message_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("illustration_{new_message_id}.png");
    std::fs::write(dir.join(&file_name), &new_image_bytes)
        .map_err(|e| format!("Failed to save adjusted illustration: {e}"))?;

    let aspect = png_aspect_ratio(&new_image_bytes);
    let b64_out = base64_encode_bytes(&new_image_bytes);
    let data_url = format!("data:image/png;base64,{b64_out}");
    let now = Utc::now().to_rfc3339();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let img = WorldImage {
            image_id: new_message_id.clone(),
            world_id: world.world_id.clone(),
            prompt: instructions.clone(),
            file_name,
            is_active: false,
            source: "illustration".to_string(),
            created_at: now.clone(),
            aspect_ratio: aspect,
            caption: String::new(),
        };
        let _ = create_world_image(&conn, &img);

        let (wd_adj, wt_adj) = world_time_fields(&world);
        let msg = Message {
            message_id: new_message_id.clone(),
            thread_id: thread.thread_id.clone(),
            role: "illustration".to_string(),
            content: data_url,
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: now,
            world_day: wd_adj, world_time: wt_adj,
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        };
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let illustration_msg = conn.query_row(
        "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id, created_at, world_day, world_time FROM messages WHERE message_id = ?1",
        params![new_message_id], |row| Ok(Message {
            message_id: row.get(0)?, thread_id: row.get(1)?, role: row.get(2)?,
            content: row.get(3)?, tokens_estimate: row.get(4)?, sender_character_id: row.get(5)?, created_at: row.get(6)?, world_day: row.get(7).ok(), world_time: row.get(8).ok(),
            address_to: None,
        mood_chain: None,
        is_proactive: false,
        })
    ).map_err(|e| e.to_string())?;

    Ok(IllustrationResult {
        illustration_message: illustration_msg,
    })
}

/// Download an illustration image to ~/Downloads.
#[tauri::command]
pub fn download_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    illustration_message_id: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let file_name: String = conn.query_row(
        "SELECT file_name FROM world_images WHERE image_id = ?1",
        params![illustration_message_id], |r| r.get(0),
    ).map_err(|_| "Illustration not found".to_string())?;

    let src = portraits_dir.0.join(&file_name);
    if !src.exists() {
        return Err("Illustration file not found on disk".to_string());
    }

    let home = std::env::var("HOME").map_err(|_| "Could not find home directory".to_string())?;
    let downloads = std::path::PathBuf::from(home).join("Downloads");
    let dest = downloads.join(&file_name);
    std::fs::copy(&src, &dest).map_err(|e| format!("Failed to copy: {e}"))?;

    Ok(dest.to_string_lossy().to_string())
}

/// Get the aspect ratio for an illustration. Returns 0.0 if unknown.
#[tauri::command]
pub fn get_illustration_aspect_ratio_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    illustration_message_id: String,
) -> Result<f64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let (ratio, file_name): (f64, String) = conn.query_row(
        "SELECT COALESCE(aspect_ratio, 0.0), file_name FROM world_images WHERE image_id = ?1",
        params![illustration_message_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).unwrap_or((0.0, String::new()));

    // Backfill if unknown
    if ratio == 0.0 && !file_name.is_empty() {
        let path = portraits_dir.0.join(&file_name);
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                let ar = png_aspect_ratio(&bytes);
                if ar > 0.0 {
                    let _ = conn.execute(
                        "UPDATE world_images SET aspect_ratio = ?1 WHERE image_id = ?2",
                        params![ar, illustration_message_id],
                    );
                    return Ok(ar);
                }
            }
        }
    }

    Ok(ratio)
}

/// Compact summary of an illustration message — enough for a thumbnail
/// view (message_id, the image content URL/path, timestamp, and the
/// world time metadata). Returned by list_thread_illustrations_cmd
/// when the caller needs every illustration in a thread regardless of
/// which messages are currently paginated into memory.
#[derive(Debug, Serialize)]
pub struct IllustrationSummary {
    pub message_id: String,
    pub content: String,
    pub created_at: String,
    pub world_day: Option<i64>,
    pub world_time: Option<String>,
    pub thread_id: String,
}

/// List every illustration message for a thread, across both
/// `messages` (solo) and `group_messages` (group chat) tables, ordered
/// ASC by created_at. Used by the sticky-illustration feature so the
/// UI has access to the full illustration timeline even when older
/// history hasn't been paginated into `store.messages`.
#[tauri::command]
pub fn list_thread_illustrations_cmd(
    db: State<'_, Database>,
    thread_id: String,
) -> Result<Vec<IllustrationSummary>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut out: Vec<IllustrationSummary> = Vec::new();

    // Check both tables. A thread_id will normally only appear in one,
    // but the query is cheap and avoids needing to know which.
    for table in &["messages", "group_messages"] {
        let sql = format!(
            "SELECT message_id, content, created_at, world_day, world_time, thread_id \
             FROM {table} \
             WHERE thread_id = ?1 AND role = 'illustration' \
             ORDER BY created_at ASC"
        );
        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let rows = stmt.query_map(params![thread_id], |r| {
            Ok(IllustrationSummary {
                message_id: r.get(0)?,
                content: r.get(1)?,
                created_at: r.get(2)?,
                world_day: r.get::<_, Option<i64>>(3)?,
                world_time: r.get::<_, Option<String>>(4)?,
                thread_id: r.get(5)?,
            })
        });
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                out.push(row);
            }
        }
    }

    // Belt-and-suspenders: sort merged results by created_at.
    out.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(out)
}

// ─── Backstage two-step illustration flow ───────────────────────────────
//
// Backstage proposes an illustration as an action card. The user should be
// able to PREVIEW the rendered image inside the card before committing it
// to the chat — and reject it if it doesn't land. Three commands wire this:
//
//   1. preview_backstage_illustration_cmd — generate + save bytes + write
//      to world_images, but do NOT insert a chat message yet. Returns the
//      image_id + data URL + aspect ratio + caption so the card can render.
//   2. attach_previewed_illustration_cmd — given an existing image_id and
//      the target thread (solo or group), insert the message row into the
//      correct table. World_day is read from the active world state.
//   3. discard_previewed_illustration_cmd — clean up the file + world_images
//      row when the user rejects the preview.
//
// Routing fix vs the prior bug: the previous BackstageActionCard always
// called the SOLO command, so an illustration generated from a group chat
// went to the wrong thread. This split lets the card pass the active
// chat's own thread context (group_chat_id when in a group) at preview
// time AND at attach time.

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewedIllustration {
    /// UUID assigned at preview time. Stable across attach/discard.
    pub image_id: String,
    /// data:image/png;base64,... — render directly in the action card.
    pub data_url: String,
    pub aspect_ratio: f64,
    pub caption: String,
}

/// Generate an illustration for Backstage WITHOUT inserting a chat message.
/// Saves the image to disk + world_images so the attach step can reference
/// it by id. When `group_chat_id` is set, paints the scene with the group's
/// full cast as references; otherwise paints as a solo scene for the
/// `character_id`'s chat.
#[tauri::command]
pub async fn preview_backstage_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
    group_chat_id: Option<String>,
    custom_instructions: Option<String>,
) -> Result<PreviewedIllustration, String> {
    let dir = &portraits_dir.0;

    // ── Resolve context: solo vs group ──────────────────────────────────
    let is_group = group_chat_id.is_some();
    let (
        world,
        primary_character,
        additional_cast_owned,
        recent_msgs,
        model_config,
        user_profile,
        all_names,
        names_map,
        list_for_thread_msg_id_set,
    ): (
        World,
        Character,
        Vec<Character>,
        Vec<Message>,
        orchestrator::ModelConfig,
        Option<UserProfile>,
        Vec<String>,
        Option<std::collections::HashMap<String, String>>,
        std::collections::HashSet<String>,
    ) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Some(gc_id) = group_chat_id.as_deref() {
            let gc = get_group_chat(&conn, gc_id).map_err(|e| e.to_string())?;
            let world = get_world(&conn, &gc.world_id).map_err(|e| e.to_string())?;
            let mut model_config = orchestrator::load_model_config(&conn);
            model_config.apply_provider_override(&conn, &format!("provider_override.{}", gc_id));
            let recent_msgs = list_group_messages_within_budget(
                &conn, &gc.thread_id, model_config.safe_history_budget() as i64, 30,
            ).map_err(|e| e.to_string())?;
            let user_profile = get_user_profile(&conn, &gc.world_id).ok();
            let char_ids: Vec<String> = gc.character_ids.as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let characters: Vec<Character> = char_ids.iter()
                .filter_map(|id| get_character(&conn, id).ok())
                .collect();
            let primary = characters.iter().find(|c| c.character_id == character_id)
                .cloned()
                .or_else(|| characters.first().cloned())
                .ok_or_else(|| "No characters in group chat".to_string())?;
            let additional: Vec<Character> = characters.iter()
                .filter(|c| c.character_id != primary.character_id)
                .cloned()
                .collect();
            let all_names: Vec<String> = characters.iter().map(|c| c.display_name.clone()).collect();
            let names_map: std::collections::HashMap<String, String> = characters.iter()
                .map(|c| (c.character_id.clone(), c.display_name.clone()))
                .collect();
            (world, primary, additional, recent_msgs, model_config, user_profile,
             all_names, Some(names_map), std::collections::HashSet::new())
        } else {
            let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
            let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
            let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
            let model_config = orchestrator::load_model_config(&conn);
            let recent_msgs = list_messages(&conn, &thread.thread_id, 30).map_err(|e| e.to_string())?;
            let user_profile = get_user_profile(&conn, &character.world_id).ok();
            (world, character, Vec::new(), recent_msgs, model_config, user_profile,
             Vec::new(), None, std::collections::HashSet::new())
        }
    };
    let _ = list_for_thread_msg_id_set;

    // ── Reference images: user avatar + character portrait(s) ───────────
    let mut reference_images: Vec<Vec<u8>> = Vec::new();
    if let Some(ref profile) = user_profile {
        if !profile.avatar_file.is_empty() {
            let path = dir.join(&profile.avatar_file);
            if let Ok(bytes) = std::fs::read(&path) { reference_images.push(bytes); }
        }
    }
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        if let Some(p) = get_active_portrait(&conn, &primary_character.character_id) {
            let path = dir.join(&p.file_name);
            if let Ok(bytes) = std::fs::read(&path) { reference_images.push(bytes); }
        }
        for c in &additional_cast_owned {
            if let Some(p) = get_active_portrait(&conn, &c.character_id) {
                let path = dir.join(&p.file_name);
                if let Ok(bytes) = std::fs::read(&path) { reference_images.push(bytes); }
            }
        }
    }

    // High tier by default for Backstage previews — the user is choosing
    // to commit them deliberately and shouldn't get the rough draft.
    let (img_size, img_quality) = ("1536x1024", "medium");

    let user_display_name = user_profile.as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("The human");
    let resolved_instructions: Option<String> = match custom_instructions.as_deref() {
        Some(s) if !s.trim().is_empty() => Some(s.to_string()),
        _ => orchestrator::pick_memorable_moment_caption(
                &model_config.chat_api_base(),
                &api_key,
                &model_config.dialogue_model,
                &recent_msgs,
                user_display_name,
            ).await.ok(),
    };

    let additional_refs: Vec<&Character> = additional_cast_owned.iter().collect();
    let additional_opt: Option<&[&Character]> = if additional_refs.is_empty() { None } else { Some(&additional_refs) };
    let names_map_ref = names_map.as_ref();

    let (scene_description, image_bytes, chat_usage) = orchestrator::generate_illustration_with_base(
        &model_config.chat_api_base(),
        &model_config.openai_api_base(),
        &api_key,
        &model_config.dialogue_model,
        &model_config.image_model,
        img_quality,
        img_size,
        model_config.image_output_format().as_deref(),
        &world,
        &primary_character,
        additional_opt,
        &recent_msgs,
        user_profile.as_ref(),
        &reference_images,
        resolved_instructions.as_deref(),
        false, // has_previous
        true,  // include_scene_summary
        if all_names.is_empty() { None } else { Some(&all_names) },
        names_map_ref,
    ).await?;

    let caption = match custom_instructions.as_deref() {
        Some(s) if !s.trim().is_empty() => s.to_string(),
        _ => orchestrator::derive_caption_from_scene(
                &model_config.chat_api_base(),
                &api_key,
                &model_config.dialogue_model,
                &scene_description,
            ).await.unwrap_or_else(|_| resolved_instructions.clone().unwrap_or_default()),
    };

    if let Some(u) = &chat_usage {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = record_token_usage(&conn, "illustration", &model_config.dialogue_model, u.prompt_tokens, u.completion_tokens);
    }

    let image_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("illustration_{image_id}.png");
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &image_bytes)
        .map_err(|e| format!("Failed to save illustration: {e}"))?;
    let aspect = png_aspect_ratio(&image_bytes);
    let b64 = base64_encode_bytes(&image_bytes);
    let data_url = format!("data:image/png;base64,{b64}");
    let now = Utc::now().to_rfc3339();

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let img = WorldImage {
            image_id: image_id.clone(),
            world_id: world.world_id.clone(),
            prompt: scene_description,
            file_name: file_name.clone(),
            is_active: false,
            // Mark previewed (not yet attached) so cleanup tooling can
            // see the difference. The attach step rewrites this to
            // "illustration".
            source: "illustration_preview".to_string(),
            created_at: now,
            aspect_ratio: aspect,
            caption: caption.clone(),
        };
        let _ = create_world_image(&conn, &img);
    }
    let _ = is_group;

    Ok(PreviewedIllustration { image_id, data_url, aspect_ratio: aspect, caption })
}

/// Commit a previewed illustration into the active chat. Routes to the
/// correct table based on whether the target is a group or solo thread.
/// World_day / world_time on the message come from the active world state
/// at attach-time so the illustration gets the right time-of-day badge.
#[tauri::command]
pub fn attach_previewed_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    image_id: String,
    target_thread_id: String,
    is_group_thread: bool,
) -> Result<Message, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (file_name, world_id): (String, String) = conn.query_row(
        "SELECT file_name, world_id FROM world_images WHERE image_id = ?1",
        params![image_id], |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|_| "Previewed image not found — may have been discarded".to_string())?;

    let world = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
    let path = portraits_dir.0.join(&file_name);
    let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read previewed image: {e}"))?;
    let b64 = base64_encode_bytes(&bytes);
    let data_url = format!("data:image/png;base64,{b64}");

    // Promote source from "illustration_preview" to "illustration" so it's
    // indexed alongside the regular illustrations in the world gallery.
    conn.execute(
        "UPDATE world_images SET source = 'illustration' WHERE image_id = ?1",
        params![image_id],
    ).map_err(|e| e.to_string())?;

    let (wd, wt) = world_time_fields(&world);
    let now = Utc::now().to_rfc3339();
    let msg = Message {
        message_id: image_id.clone(),
        thread_id: target_thread_id.clone(),
        role: "illustration".to_string(),
        content: data_url,
        tokens_estimate: 0,
        sender_character_id: None,
        created_at: now,
        world_day: wd,
        world_time: wt,
        address_to: None,
        mood_chain: None,
        is_proactive: false,
    };

    if is_group_thread {
        create_group_message(&conn, &msg).map_err(|e| e.to_string())?;
    } else {
        create_message(&conn, &msg).map_err(|e| e.to_string())?;
    }

    Ok(msg)
}

/// Discard a previewed illustration: delete the file from disk and remove
/// the world_images row.
#[tauri::command]
pub fn discard_previewed_illustration_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    image_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let file_name: Option<String> = conn.query_row(
        "SELECT file_name FROM world_images WHERE image_id = ?1",
        params![image_id], |r| r.get(0),
    ).ok();
    if let Some(f) = file_name {
        let path = portraits_dir.0.join(&f);
        if path.exists() { let _ = std::fs::remove_file(&path); }
    }
    let _ = conn.execute("DELETE FROM world_images WHERE image_id = ?1", params![image_id]);
    Ok(())
}

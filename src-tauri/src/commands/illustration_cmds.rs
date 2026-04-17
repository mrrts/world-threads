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

    let (scene_description, image_bytes, chat_usage) = orchestrator::generate_illustration_with_base(
        &model_config.chat_api_base(),
        &model_config.openai_api_base(),
        &api_key,
        &model_config.dialogue_model,
        &model_config.image_model,
        img_quality,
        img_size,
        model_config.image_output_format().as_deref(),
        &world, &character, &recent_msgs,
        user_profile.as_ref(),
        &reference_images,
        custom_instructions.as_deref(),
        has_previous,
        include_scene_summary.unwrap_or(true),
        None,
    ).await?;

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

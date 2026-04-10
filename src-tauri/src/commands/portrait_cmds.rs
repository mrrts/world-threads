use crate::ai::openai::{self, ImageRequest};
use crate::ai::orchestrator;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Clone)]
pub struct PortraitsDir(pub PathBuf);

fn json_array_to_strings(val: &serde_json::Value) -> Vec<String> {
    match val.as_array() {
        Some(arr) => arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
        None => Vec::new(),
    }
}

fn build_portrait_prompt(character: &Character, world: &World) -> String {
    let mut parts = vec![
        "Hand-painted watercolor portrait of a character in a lush, realistic illustration style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones with pops of verdant green and sky blue.".to_string(),
        "Gentle diffused natural lighting, nostalgic and contemplative mood, as if lifted from an illustrated fairy tale.".to_string(),
        "Close-up face and bust portrait only, facing straight ahead toward the viewer, expressive eyes making direct eye contact. Framing ends at the upper chest.".to_string(),
    ];

    parts.push(format!("Character name: {}", character.display_name));

    if !character.identity.is_empty() {
        let identity = if character.identity.len() > 300 {
            format!("{}...", &character.identity[..300])
        } else {
            character.identity.clone()
        };
        parts.push(format!("Personality and appearance: {identity}"));
    }

    let backstory = json_array_to_strings(&character.backstory_facts);
    if !backstory.is_empty() {
        let facts = backstory.iter().take(3).cloned().collect::<Vec<_>>().join("; ");
        parts.push(format!("Key traits: {facts}"));
    }

    if !world.description.is_empty() {
        let desc = if world.description.len() > 150 {
            format!("{}...", &world.description[..150])
        } else {
            world.description.clone()
        };
        parts.push(format!("World setting: {desc}"));
    }

    parts.push("CRITICAL: The image must contain absolutely no text, no words, no letters, no numbers, no writing, no labels, no titles, no captions, no watermarks, no signatures, no UI elements, no names.".to_string());

    parts.join(" ")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortraitInfo {
    pub portrait_id: String,
    pub character_id: String,
    pub prompt: String,
    pub is_active: bool,
    pub created_at: String,
    pub data_url: String,
}

fn portrait_to_info(p: &Portrait, portraits_dir: &std::path::Path) -> PortraitInfo {
    let file_path = portraits_dir.join(&p.file_name);
    let data_url = if file_path.exists() {
        let bytes = std::fs::read(&file_path).unwrap_or_default();
        format!("data:image/png;base64,{}", base64_encode(&bytes))
    } else {
        String::new()
    };
    PortraitInfo {
        portrait_id: p.portrait_id.clone(),
        character_id: p.character_id.clone(),
        prompt: p.prompt.clone(),
        is_active: p.is_active,
        created_at: p.created_at.clone(),
        data_url,
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[derive(Debug, Deserialize, Default)]
pub struct CharacterFormHint {
    pub display_name: Option<String>,
    pub identity: Option<String>,
    pub backstory_facts: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn generate_portrait_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
    form_hint: Option<CharacterFormHint>,
) -> Result<PortraitInfo, String> {
    let (mut character, world) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let ch = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let w = get_world(&conn, &ch.world_id).map_err(|e| e.to_string())?;
        (ch, w)
    };

    if let Some(hint) = form_hint {
        if let Some(name) = hint.display_name { character.display_name = name; }
        if let Some(id) = hint.identity { character.identity = id; }
        if let Some(facts) = hint.backstory_facts { character.backstory_facts = facts; }
    }

    let model_config = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        orchestrator::load_model_config(&conn)
    };

    let prompt = build_portrait_prompt(&character, &world);
    log::info!("[Portrait] Generating for '{}': {:.120}...", character.display_name, prompt);

    let request = ImageRequest {
        model: model_config.image_model.clone(),
        prompt: prompt.clone(),
        n: 1,
        size: "1024x1024".to_string(),
        quality: model_config.image_quality().to_string(),
        response_format: model_config.image_response_format(),
        output_format: model_config.image_output_format(),
    };

    let response = openai::generate_image_with_base(&model_config.openai_api_base(), &api_key, &request).await?;
    let b64 = response.data.first()
        .and_then(|d| d.image_b64())
        .ok_or_else(|| "No image data in response".to_string())?;

    let image_bytes = base64_decode(b64)
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let portrait_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("{portrait_id}.png");
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create portraits dir: {e}"))?;
    let file_path = dir.join(&file_name);
    std::fs::write(&file_path, &image_bytes).map_err(|e| format!("Failed to save image: {e}"))?;

    log::info!("[Portrait] Saved {} ({} bytes)", file_name, image_bytes.len());

    let portrait = Portrait {
        portrait_id: portrait_id.clone(),
        character_id: character_id.clone(),
        prompt,
        file_name: file_name.clone(),
        is_active: true,
        created_at: Utc::now().to_rfc3339(),
    };

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        // Deactivate all existing, then insert new as active
        let _ = conn.execute("UPDATE character_portraits SET is_active = 0 WHERE character_id = ?1", rusqlite::params![character_id]);
        create_portrait(&conn, &portrait).map_err(|e| e.to_string())?;
    }

    Ok(portrait_to_info(&portrait, dir))
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const DECODE: [u8; 128] = {
        let mut table = [255u8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 {
            table[chars[i] as usize] = i as u8;
            i += 1;
        }
        table
    };

    let input = input.as_bytes();
    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0;

    for &b in input {
        if b == b'=' || b == b'\n' || b == b'\r' { continue; }
        if b >= 128 { return Err("Invalid base64 character".to_string()); }
        let val = DECODE[b as usize];
        if val == 255 { return Err(format!("Invalid base64 character: {}", b as char)); }
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn list_portraits_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    character_id: String,
) -> Result<Vec<PortraitInfo>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let portraits = list_portraits(&conn, &character_id).map_err(|e| e.to_string())?;
    Ok(portraits.iter().map(|p| portrait_to_info(p, &portraits_dir.0)).collect())
}

#[tauri::command]
pub fn delete_portrait_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    portrait_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let file_name = delete_portrait(&conn, &portrait_id).map_err(|e| e.to_string())?;
    let file_path = portraits_dir.0.join(&file_name);
    if file_path.exists() {
        let _ = std::fs::remove_file(&file_path);
    }
    Ok(())
}

#[tauri::command]
pub fn set_active_portrait_cmd(
    db: State<Database>,
    character_id: String,
    portrait_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    set_active_portrait(&conn, &character_id, &portrait_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_active_portrait_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    character_id: String,
) -> Result<Option<PortraitInfo>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(get_active_portrait(&conn, &character_id).map(|p| portrait_to_info(&p, &portraits_dir.0)))
}

/// Generate a variation of the active portrait by sending the reference image directly to the image model.
#[tauri::command]
pub async fn generate_portrait_variation_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    character_id: String,
) -> Result<PortraitInfo, String> {
    let (character, world, active_portrait, mc) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let ch = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let w = get_world(&conn, &ch.world_id).map_err(|e| e.to_string())?;
        let ap = get_active_portrait(&conn, &character_id)
            .ok_or_else(|| "No active portrait to create a variation from".to_string())?;
        let mc = orchestrator::load_model_config(&conn);
        (ch, w, ap, mc)
    };

    let file_path = portraits_dir.0.join(&active_portrait.file_name);
    if !file_path.exists() {
        return Err("Active portrait file not found on disk".to_string());
    }
    let image_bytes = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read portrait file: {e}"))?;

    let mut prompt_parts = vec![
        "Hand-painted watercolor portrait in a lush, realistic illustration style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones with pops of verdant green and sky blue.".to_string(),
        "Gentle diffused natural lighting, nostalgic and contemplative mood.".to_string(),
        format!("Generate a new portrait of the SAME person shown in the reference image, but in a DIFFERENT pose, angle, or situation. The character must be clearly recognizable as the same person."),
        format!("Character name: {}", character.display_name),
    ];

    if !character.identity.is_empty() {
        let identity = if character.identity.len() > 200 {
            format!("{}...", &character.identity[..200])
        } else {
            character.identity.clone()
        };
        prompt_parts.push(format!("Character identity: {identity}"));
    }

    if !world.description.is_empty() {
        let desc = if world.description.len() > 150 {
            format!("{}...", &world.description[..150])
        } else {
            world.description.clone()
        };
        prompt_parts.push(format!("World setting: {desc}"));
    }

    prompt_parts.push("Show the character in a different pose, angle, expression, or situation than the reference image. Vary the composition while keeping them unmistakably the same person.".to_string());
    prompt_parts.push("CRITICAL: The image must contain absolutely no text, no words, no letters, no numbers, no writing, no labels, no titles, no captions, no watermarks, no signatures, no UI elements, no names.".to_string());

    let prompt = prompt_parts.join(" ");

    log::info!("[PortraitVariation] Generating variation for '{}' with reference image ({} bytes)", character.display_name, image_bytes.len());

    let response = openai::generate_image_edit_with_base(
        &mc.openai_api_base(), &api_key, &mc.image_model,
        &prompt, &image_bytes,
        "1024x1024", mc.image_quality(),
        mc.image_output_format().as_deref(),
    ).await?;
    let b64 = response.data.first()
        .and_then(|d| d.image_b64())
        .ok_or_else(|| "No image data in response".to_string())?;

    let new_bytes = base64_decode(b64)
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let portrait_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("{portrait_id}.png");
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create portraits dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &new_bytes)
        .map_err(|e| format!("Failed to save image: {e}"))?;

    log::info!("[PortraitVariation] Saved variation {} ({} bytes)", file_name, new_bytes.len());

    let portrait = Portrait {
        portrait_id,
        character_id: character_id.clone(),
        prompt,
        file_name,
        is_active: false,
        created_at: Utc::now().to_rfc3339(),
    };

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_portrait(&conn, &portrait).map_err(|e| e.to_string())?;
    }

    Ok(portrait_to_info(&portrait, dir))
}

/// Set a character's active portrait from any existing image file in the portraits directory.
#[tauri::command]
pub fn set_portrait_from_gallery_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    character_id: String,
    source_file: String,
) -> Result<PortraitInfo, String> {
    let dir = &portraits_dir.0;
    let src_path = dir.join(&source_file);
    if !src_path.exists() {
        return Err(format!("Source file not found: {source_file}"));
    }

    let portrait_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("{portrait_id}.png");
    std::fs::copy(&src_path, dir.join(&file_name))
        .map_err(|e| format!("Failed to copy image: {e}"))?;

    let portrait = Portrait {
        portrait_id: portrait_id.clone(),
        character_id: character_id.clone(),
        prompt: String::new(),
        file_name: file_name.clone(),
        is_active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let _ = conn.execute(
        "UPDATE character_portraits SET is_active = 0 WHERE character_id = ?1",
        rusqlite::params![character_id],
    );
    create_portrait(&conn, &portrait).map_err(|e| e.to_string())?;

    Ok(portrait_to_info(&portrait, dir))
}

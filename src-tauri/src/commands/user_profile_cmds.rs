use crate::ai::openai::{self, ImageRequest};
use crate::ai::orchestrator;
use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use tauri::State;

#[tauri::command]
pub fn get_user_profile_cmd(
    db: State<Database>,
    world_id: String,
) -> Result<Option<UserProfile>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    match get_user_profile(&conn, &world_id) {
        Ok(p) => Ok(Some(p)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn update_user_profile_cmd(
    db: State<Database>,
    profile: UserProfile,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    upsert_user_profile(&conn, &profile).map_err(|e| e.to_string())
}

#[derive(Debug, serde::Deserialize, Default)]
pub struct UserFormHint {
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn generate_user_avatar_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    world_id: String,
    form_hint: Option<UserFormHint>,
) -> Result<String, String> {
    let (profile, mc) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let p = get_user_profile(&conn, &world_id).map_err(|e| e.to_string())?;
        let mc = orchestrator::load_model_config(&conn);
        (p, mc)
    };

    let display_name = form_hint.as_ref()
        .and_then(|h| h.display_name.clone())
        .unwrap_or(profile.display_name);
    let description = form_hint.as_ref()
        .and_then(|h| h.description.clone())
        .unwrap_or(profile.description);

    let mut prompt_parts = vec![
        "Hand-painted watercolor portrait of a person in a lush, realistic illustration style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones with pops of verdant green and sky blue.".to_string(),
        "Close-up face and bust portrait only, facing straight ahead toward the viewer, expressive eyes making direct eye contact. Framing ends at the upper chest.".to_string(),
    ];

    if !display_name.is_empty() && display_name != "Me" {
        prompt_parts.push(format!("Name: {}", display_name));
    }
    if !description.is_empty() {
        let desc = if description.len() > 300 {
            format!("{}...", &description[..300])
        } else {
            description
        };
        prompt_parts.push(format!("Appearance and personality: {desc}"));
    }
    prompt_parts.push("CRITICAL: The image must contain absolutely no text, no words, no letters, no numbers, no writing, no labels, no titles, no captions, no watermarks, no signatures, no UI elements, no names.".to_string());

    let prompt = prompt_parts.join(" ");
    log::info!("[UserAvatar] Generating: {:.120}...", prompt);

    let request = ImageRequest {
        model: mc.image_model.clone(),
        prompt,
        n: 1,
        size: "1024x1024".to_string(),
        quality: mc.image_quality().to_string(),
        response_format: mc.image_response_format(),
        output_format: mc.image_output_format(),
    };

    let prompt = request.prompt.clone();
    let response = openai::generate_image_with_base(&mc.openai_api_base(), &api_key, &request).await?;
    let b64 = response.data.first()
        .and_then(|d| d.image_b64())
        .ok_or_else(|| "No image data in response".to_string())?;

    let image_bytes = base64_decode(b64)
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let file_name = format!("user_{}.png", uuid::Uuid::new_v4());
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &image_bytes)
        .map_err(|e| format!("Failed to save image: {e}"))?;

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        set_user_avatar_file(&conn, &world_id, &file_name).map_err(|e| e.to_string())?;

        let img = WorldImage {
            image_id: uuid::Uuid::new_v4().to_string(),
            world_id: world_id.clone(),
            prompt,
            file_name: file_name.clone(),
            is_active: false,
            source: "user_avatar".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            aspect_ratio: 1.0,
            caption: String::new(),
        };
        let _ = create_world_image(&conn, &img);
    }

    let data_url = format!("data:image/png;base64,{}", base64_encode(&image_bytes));
    Ok(data_url)
}

#[tauri::command]
pub fn upload_user_avatar_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    world_id: String,
    image_data: String,
) -> Result<String, String> {
    let raw = if image_data.contains(',') {
        image_data.split(',').nth(1).unwrap_or(&image_data)
    } else {
        &image_data
    };

    let image_bytes = base64_decode(raw)
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let file_name = format!("user_{}.png", uuid::Uuid::new_v4());
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &image_bytes)
        .map_err(|e| format!("Failed to save image: {e}"))?;

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        set_user_avatar_file(&conn, &world_id, &file_name).map_err(|e| e.to_string())?;
    }

    let data_url = format!("data:image/png;base64,{}", base64_encode(&image_bytes));
    Ok(data_url)
}

#[tauri::command]
pub fn get_user_avatar_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    world_id: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let profile = match get_user_profile(&conn, &world_id) {
        Ok(p) => p,
        Err(_) => return Ok(String::new()),
    };
    if profile.avatar_file.is_empty() {
        return Ok(String::new());
    }
    let file_path = portraits_dir.0.join(&profile.avatar_file);
    if !file_path.exists() {
        return Ok(String::new());
    }
    let bytes = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    Ok(format!("data:image/png;base64,{}", base64_encode(&bytes)))
}

/// List all user avatars across all worlds (for cross-world avatar copying).
#[tauri::command]
pub fn list_all_user_avatars_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT up.world_id, up.avatar_file, w.name FROM user_profiles up JOIN worlds w ON w.world_id = up.world_id WHERE up.avatar_file != ''"
    ).map_err(|e| e.to_string())?;
    let rows: Vec<(String, String, String)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok()).collect();

    let mut results = Vec::new();
    for (world_id, avatar_file, world_name) in rows {
        let path = portraits_dir.0.join(&avatar_file);
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                results.push(serde_json::json!({
                    "world_id": world_id,
                    "world_name": world_name,
                    "avatar_file": avatar_file,
                    "data_url": format!("data:image/png;base64,{}", base64_encode(&bytes)),
                }));
            }
        }
    }
    Ok(results)
}

/// Set user avatar from any existing image file in the portraits directory.
#[tauri::command]
pub fn set_user_avatar_from_gallery_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    world_id: String,
    source_file: String,
) -> Result<String, String> {
    let dir = &portraits_dir.0;
    let src_path = dir.join(&source_file);
    if !src_path.exists() {
        return Err(format!("Source file not found: {source_file}"));
    }

    let file_name = format!("user_{}.png", uuid::Uuid::new_v4());
    std::fs::copy(&src_path, dir.join(&file_name))
        .map_err(|e| format!("Failed to copy image: {e}"))?;

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    set_user_avatar_file(&conn, &world_id, &file_name).map_err(|e| e.to_string())?;

    let bytes = std::fs::read(dir.join(&file_name)).map_err(|e| e.to_string())?;
    Ok(format!("data:image/png;base64,{}", base64_encode(&bytes)))
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
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const DECODE: [u8; 128] = {
        let mut table = [255u8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 { table[chars[i] as usize] = i as u8; i += 1; }
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
        if bits >= 8 { bits -= 8; result.push((buf >> bits) as u8); buf &= (1 << bits) - 1; }
    }
    Ok(result)
}

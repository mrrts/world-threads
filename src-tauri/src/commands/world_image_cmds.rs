use crate::ai::openai::{self, ImageRequest};
use crate::ai::orchestrator;
use crate::db::queries::*;
use crate::db::Database;
use crate::commands::portrait_cmds::PortraitsDir;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

fn build_world_image_prompt(world: &World) -> String {
    let mut parts = vec![
        "Hand-painted watercolor landscape in a lush, realistic illustration style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones with pops of verdant green and sky blue.".to_string(),
        "Gentle diffused natural lighting, nostalgic and contemplative mood, wide panoramic composition.".to_string(),
        "Wide establishing shot, rich environmental detail, no characters or people in the scene.".to_string(),
    ];

    parts.push(format!("World name: {}", world.name));

    if !world.description.is_empty() {
        let desc = if world.description.len() > 500 {
            format!("{}...", &world.description[..500])
        } else {
            world.description.clone()
        };
        parts.push(format!("Setting: {desc}"));
    }

    let tags: Vec<String> = world.tone_tags.as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    if !tags.is_empty() {
        parts.push(format!("Mood: {}", tags.join(", ")));
    }

    parts.push("CRITICAL: The image must contain absolutely no text, no words, no letters, no numbers, no writing, no labels, no titles, no captions, no watermarks, no signatures, no UI elements, no characters or people.".to_string());

    parts.join(" ")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldImageInfo {
    pub image_id: String,
    pub world_id: String,
    pub prompt: String,
    pub is_active: bool,
    pub source: String,
    pub created_at: String,
    pub data_url: String,
}

fn image_to_info(img: &WorldImage, dir: &std::path::Path) -> WorldImageInfo {
    let file_path = dir.join(&img.file_name);
    let data_url = if file_path.exists() {
        let bytes = std::fs::read(&file_path).unwrap_or_default();
        format!("data:image/png;base64,{}", base64_encode(&bytes))
    } else {
        String::new()
    };
    WorldImageInfo {
        image_id: img.image_id.clone(),
        world_id: img.world_id.clone(),
        prompt: img.prompt.clone(),
        is_active: img.is_active,
        source: img.source.clone(),
        created_at: img.created_at.clone(),
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

#[derive(Debug, Deserialize, Default)]
pub struct WorldFormHint {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tone_tags: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn generate_world_image_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    world_id: String,
    form_hint: Option<WorldFormHint>,
) -> Result<WorldImageInfo, String> {
    let (mut world, mc) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let w = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let mc = orchestrator::load_model_config(&conn);
        (w, mc)
    };

    if let Some(hint) = form_hint {
        if let Some(name) = hint.name { world.name = name; }
        if let Some(desc) = hint.description { world.description = desc; }
        if let Some(tags) = hint.tone_tags { world.tone_tags = tags; }
    }

    let prompt = build_world_image_prompt(&world);
    log::info!("[WorldImage] Generating for '{}': {:.120}...", world.name, prompt);

    let request = ImageRequest {
        model: mc.image_model.clone(),
        prompt: prompt.clone(),
        n: 1,
        size: mc.landscape_size().to_string(),
        quality: mc.image_quality().to_string(),
        response_format: mc.image_response_format(),
        output_format: mc.image_output_format(),
    };

    let response = openai::generate_image_with_base(&mc.openai_api_base(), &api_key, &request).await?;
    let b64 = response.data.first()
        .and_then(|d| d.image_b64())
        .ok_or_else(|| "No image data in response".to_string())?;

    let image_bytes = base64_decode(b64)
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let image_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("world_{image_id}.png");
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    let file_path = dir.join(&file_name);
    std::fs::write(&file_path, &image_bytes).map_err(|e| format!("Failed to save image: {e}"))?;

    log::info!("[WorldImage] Saved {} ({} bytes)", file_name, image_bytes.len());

    let img = WorldImage {
        image_id: image_id.clone(),
        world_id: world_id.clone(),
        prompt,
        file_name,
        is_active: true,
        source: "generated".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = conn.execute("UPDATE world_images SET is_active = 0 WHERE world_id = ?1", rusqlite::params![world_id]);
        create_world_image(&conn, &img).map_err(|e| e.to_string())?;
    }

    Ok(image_to_info(&img, dir))
}

#[tauri::command]
pub fn list_world_images_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    world_id: String,
) -> Result<Vec<WorldImageInfo>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let images = list_world_images(&conn, &world_id).map_err(|e| e.to_string())?;
    Ok(images.iter().map(|i| image_to_info(i, &portraits_dir.0)).collect())
}

#[tauri::command]
pub fn get_active_world_image_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    world_id: String,
) -> Result<Option<WorldImageInfo>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(get_active_world_image(&conn, &world_id).map(|i| image_to_info(&i, &portraits_dir.0)))
}

#[tauri::command]
pub fn set_active_world_image_cmd(
    db: State<Database>,
    world_id: String,
    image_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    set_active_world_image(&conn, &world_id, &image_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_chat_background_cmd(
    db: State<Database>,
    character_id: String,
) -> Result<Option<ChatBackground>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    Ok(get_chat_background(&conn, &character_id))
}

#[tauri::command]
pub fn update_chat_background_cmd(
    db: State<Database>,
    bg: ChatBackground,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    upsert_chat_background(&conn, &bg).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_world_image_with_prompt_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    api_key: String,
    world_id: String,
    custom_prompt: String,
) -> Result<WorldImageInfo, String> {
    let mc = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        orchestrator::load_model_config(&conn)
    };

    let mut prompt_parts = vec![
        "Hand-painted watercolor landscape in a lush, realistic illustration style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones with pops of verdant green and sky blue.".to_string(),
        "Gentle diffused natural lighting, nostalgic and contemplative mood.".to_string(),
        "Wide establishing shot, rich environmental detail.".to_string(),
    ];
    prompt_parts.push(custom_prompt.clone());
    prompt_parts.push("CRITICAL: The image must contain absolutely no text, no words, no letters, no numbers, no writing, no labels, no titles, no captions, no watermarks, no signatures, no UI elements.".to_string());

    let prompt = prompt_parts.join(" ");
    log::info!("[WorldImage] Custom prompt for '{}': {:.120}...", world_id, prompt);

    let request = ImageRequest {
        model: mc.image_model.clone(),
        prompt: prompt.clone(),
        n: 1,
        size: mc.landscape_size().to_string(),
        quality: mc.image_quality().to_string(),
        response_format: mc.image_response_format(),
        output_format: mc.image_output_format(),
    };

    let response = openai::generate_image_with_base(&mc.openai_api_base(), &api_key, &request).await?;
    let b64 = response.data.first()
        .and_then(|d| d.image_b64())
        .ok_or_else(|| "No image data in response".to_string())?;

    let image_bytes = base64_decode(b64)
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let image_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("world_{image_id}.png");
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &image_bytes)
        .map_err(|e| format!("Failed to save image: {e}"))?;

    let img = WorldImage {
        image_id,
        world_id: world_id.clone(),
        prompt: custom_prompt,
        file_name,
        is_active: false,
        source: "generated".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_world_image(&conn, &img).map_err(|e| e.to_string())?;
    }

    Ok(image_to_info(&img, dir))
}

#[tauri::command]
pub fn upload_world_image_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    world_id: String,
    image_data: String,
    label: String,
) -> Result<WorldImageInfo, String> {
    let raw = if image_data.contains(',') {
        image_data.split(',').nth(1).unwrap_or(&image_data)
    } else {
        &image_data
    };

    let image_bytes = base64_decode(raw)
        .map_err(|e| format!("Failed to decode image: {e}"))?;

    let image_id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("world_{image_id}.png");
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;
    std::fs::write(dir.join(&file_name), &image_bytes)
        .map_err(|e| format!("Failed to save image: {e}"))?;

    let img = WorldImage {
        image_id,
        world_id: world_id.clone(),
        prompt: label,
        file_name,
        is_active: false,
        source: "uploaded".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_world_image(&conn, &img).map_err(|e| e.to_string())?;
    }

    Ok(image_to_info(&img, dir))
}

// ─── Unified gallery: every image in a world ─────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct GalleryItem {
    pub id: String,
    pub source_id: String,
    pub file_name: String,
    pub data_url: String,
    pub prompt: String,
    pub category: String,
    pub label: String,
    pub is_archived: bool,
    pub tags: Vec<String>,
    pub created_at: String,
}

fn file_to_data_url(dir: &Path, file_name: &str) -> String {
    let path = dir.join(file_name);
    if path.exists() {
        let bytes = std::fs::read(&path).unwrap_or_default();
        format!("data:image/png;base64,{}", base64_encode(&bytes))
    } else {
        String::new()
    }
}

fn parse_tags(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

#[tauri::command]
pub fn list_world_gallery_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    world_id: String,
) -> Result<Vec<GalleryItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let dir = &portraits_dir.0;
    let mut items: Vec<GalleryItem> = Vec::new();

    {
        let mut stmt = conn.prepare(
            "SELECT image_id, file_name, prompt, source, is_active, is_archived, tags, created_at FROM world_images WHERE world_id = ?1 ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;
        let wid = world_id.clone();
        let rows = stmt.query_map(rusqlite::params![world_id], |row| {
            let src: String = row.get(3)?;
            let label = if src == "uploaded" { "Uploaded" } else { "Generated" };
            Ok(GalleryItem {
                id: row.get(0)?,
                source_id: wid.clone(),
                file_name: row.get(1)?,
                data_url: String::new(),
                prompt: row.get(2)?,
                category: "world".to_string(),
                label: format!("World · {label}"),
                is_archived: row.get(5)?,
                tags: parse_tags(&row.get::<_, String>(6)?),
                created_at: row.get(7)?,
            })
        }).map_err(|e| e.to_string())?;
        for r in rows {
            let mut item = r.map_err(|e| e.to_string())?;
            item.data_url = file_to_data_url(dir, &item.file_name);
            items.push(item);
        }
    }

    {
        let mut stmt = conn.prepare(
            "SELECT p.portrait_id, p.file_name, p.prompt, p.is_active, p.is_archived, p.tags, p.created_at, c.display_name, p.character_id
             FROM character_portraits p
             JOIN characters c ON c.character_id = p.character_id
             WHERE c.world_id = ?1
             ORDER BY p.created_at DESC"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(rusqlite::params![world_id], |row| {
            Ok(GalleryItem {
                id: row.get(0)?,
                source_id: row.get(8)?,
                file_name: row.get(1)?,
                data_url: String::new(),
                prompt: row.get(2)?,
                category: "character".to_string(),
                label: row.get(7)?,
                is_archived: row.get(4)?,
                tags: parse_tags(&row.get::<_, String>(5)?),
                created_at: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?;
        for r in rows {
            let mut item = r.map_err(|e| e.to_string())?;
            item.data_url = file_to_data_url(dir, &item.file_name);
            items.push(item);
        }
    }

    if let Ok(profile) = get_user_profile(&conn, &world_id) {
        if !profile.avatar_file.is_empty() {
            let data_url = file_to_data_url(dir, &profile.avatar_file);
            if !data_url.is_empty() {
                items.push(GalleryItem {
                    id: format!("user_{}", world_id),
                    source_id: world_id.clone(),
                    file_name: profile.avatar_file.clone(),
                    data_url,
                    prompt: String::new(),
                    category: "user".to_string(),
                    label: profile.display_name,
                    is_archived: false,
                    tags: vec![],
                    created_at: profile.updated_at,
                });
            }
        }
    }

    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(items)
}

#[tauri::command]
pub fn archive_gallery_item_cmd(
    db: State<Database>,
    item_id: String,
    category: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    match category.as_str() {
        "world" => conn.execute("UPDATE world_images SET is_archived = 1 WHERE image_id = ?1", rusqlite::params![item_id]),
        "character" => conn.execute("UPDATE character_portraits SET is_archived = 1 WHERE portrait_id = ?1", rusqlite::params![item_id]),
        _ => return Err("Cannot archive this item type".to_string()),
    }.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn unarchive_gallery_item_cmd(
    db: State<Database>,
    item_id: String,
    category: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    match category.as_str() {
        "world" => conn.execute("UPDATE world_images SET is_archived = 0 WHERE image_id = ?1", rusqlite::params![item_id]),
        "character" => conn.execute("UPDATE character_portraits SET is_archived = 0 WHERE portrait_id = ?1", rusqlite::params![item_id]),
        _ => return Err("Cannot unarchive this item type".to_string()),
    }.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_gallery_item_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    item_id: String,
    category: String,
    file_name: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    match category.as_str() {
        "world" => conn.execute("DELETE FROM world_images WHERE image_id = ?1", rusqlite::params![item_id]),
        "character" => conn.execute("DELETE FROM character_portraits WHERE portrait_id = ?1", rusqlite::params![item_id]),
        _ => return Err("Cannot delete this item type".to_string()),
    }.map_err(|e| e.to_string())?;
    let path = portraits_dir.0.join(&file_name);
    let _ = std::fs::remove_file(path);
    Ok(())
}

/// Save a cropped image into the same category as its source.
/// For "character" crops, `source_id` must be the character_id.
/// For "user" crops, `source_id` is ignored (avatar is per-world).
#[tauri::command]
pub fn save_crop_cmd(
    db: State<Database>,
    portraits_dir: State<PortraitsDir>,
    world_id: String,
    source_category: String,
    source_id: String,
    image_data: String,
) -> Result<GalleryItem, String> {
    let dir = &portraits_dir.0;
    std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create dir: {e}"))?;

    let raw = image_data.find(",").map(|i| &image_data[i + 1..]).unwrap_or(&image_data);
    let bytes = base64_decode(raw).map_err(|e| format!("Failed to decode crop: {e}"))?;

    let id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("crop_{id}.png");
    std::fs::write(dir.join(&file_name), &bytes).map_err(|e| format!("Failed to save crop: {e}"))?;

    let now = chrono::Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&vec!["Crop"]).unwrap_or_else(|_| "[\"Crop\"]".to_string());
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (category, label) = match source_category.as_str() {
        "character" => {
            let char_name: String = conn.query_row(
                "SELECT display_name FROM characters WHERE character_id = ?1",
                rusqlite::params![source_id],
                |r| r.get(0),
            ).unwrap_or_else(|_| "Character".to_string());
            conn.execute(
                "INSERT INTO character_portraits (portrait_id, character_id, prompt, file_name, is_active, is_archived, tags, created_at) VALUES (?1, ?2, ?3, ?4, 0, 0, ?5, ?6)",
                rusqlite::params![id, source_id, "Cropped", file_name, tags_json, now],
            ).map_err(|e| e.to_string())?;
            ("character".to_string(), char_name)
        }
        "user" => {
            // Save as a world image tagged as user crop, since user_profiles only holds one avatar_file
            conn.execute(
                "INSERT INTO world_images (image_id, world_id, prompt, file_name, is_active, source, is_archived, tags, created_at) VALUES (?1, ?2, ?3, ?4, 0, 'crop', 0, ?5, ?6)",
                rusqlite::params![id, world_id, "Cropped from avatar", file_name, tags_json, now],
            ).map_err(|e| e.to_string())?;
            ("world".to_string(), "World · Crop".to_string())
        }
        _ => {
            conn.execute(
                "INSERT INTO world_images (image_id, world_id, prompt, file_name, is_active, source, is_archived, tags, created_at) VALUES (?1, ?2, ?3, ?4, 0, 'crop', 0, ?5, ?6)",
                rusqlite::params![id, world_id, "Cropped from world image", file_name, tags_json, now],
            ).map_err(|e| e.to_string())?;
            ("world".to_string(), "World · Crop".to_string())
        }
    };

    Ok(GalleryItem {
        id,
        source_id: source_id.clone(),
        file_name: file_name.clone(),
        data_url: file_to_data_url(dir, &file_name),
        prompt: "Cropped".to_string(),
        category,
        label,
        is_archived: false,
        tags: vec!["Crop".to_string()],
        created_at: now,
    })
}

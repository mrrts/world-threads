use std::path::PathBuf;
use std::collections::HashMap;
use tauri::State;

use crate::ai::{openai, orchestrator};
use crate::db::Database;
use crate::db::queries::{get_setting, set_setting};

pub struct AudioDir(pub PathBuf);

/// Delete all audio files for a message (all tones + legacy format).
pub fn delete_audio_for_message(audio_dir: &std::path::Path, message_id: &str) {
    // New format: {message_id}_{tone}.mp3
    if let Ok(entries) = std::fs::read_dir(audio_dir) {
        let prefix = format!("{message_id}_");
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if (name.starts_with(&prefix) || name == format!("{message_id}.mp3")) && name.ends_with(".mp3") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

/// Generate speech for a message with a specific tone. Returns the audio bytes (MP3).
/// Cached as `{message_id}_{tone_lower}.mp3`. Also saves the last-selected tone.
#[tauri::command]
pub async fn generate_speech_cmd(
    db: State<'_, Database>,
    audio_dir: State<'_, AudioDir>,
    api_key: String,
    message_id: String,
    text: String,
    character_id: String,
    tone: Option<String>,
) -> Result<Vec<u8>, String> {
    let tone_key = tone.as_deref().unwrap_or("auto").to_lowercase();

    // Check if this message+tone combo is already cached
    let file_path = audio_dir.0.join(format!("{message_id}_{tone_key}.mp3"));
    if file_path.exists() {
        // Still save last tone even if cached
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = set_setting(&conn, &format!("last_tone.{message_id}"), &tone_key);
        return std::fs::read(&file_path).map_err(|e| format!("Failed to read audio: {e}"));
    }

    // Also check legacy file ({message_id}.mp3) for "auto" tone
    if tone_key == "auto" {
        let legacy_path = audio_dir.0.join(format!("{message_id}.mp3"));
        if legacy_path.exists() {
            // Rename to new format
            let _ = std::fs::rename(&legacy_path, &file_path);
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let _ = set_setting(&conn, &format!("last_tone.{message_id}"), &tone_key);
            return std::fs::read(&file_path).map_err(|e| format!("Failed to read audio: {e}"));
        }
    }

    // Look up voice setting for this character
    let voice = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_setting(&conn, &format!("voice.{character_id}"))
            .ok()
            .flatten()
            .unwrap_or_else(|| "ash".to_string())
    };

    let model_config = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        orchestrator::load_model_config(&conn)
    };

    // Prepend tone bracket if not auto
    let input = match tone.as_deref() {
        Some(t) if t != "Auto" && t != "auto" => format!("Narration style:\n{t}\n\nText:\n{text}"),
        _ => text,
    };

    let request = openai::TtsRequest {
        model: "gpt-4o-mini-tts".to_string(),
        input,
        voice,
    };

    let bytes = openai::text_to_speech(&model_config.openai_api_base(), &api_key, &request).await?;

    // Cache to disk
    std::fs::write(&file_path, &bytes).map_err(|e| format!("Failed to cache audio: {e}"))?;

    // Save last-selected tone
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = set_setting(&conn, &format!("last_tone.{message_id}"), &tone_key);
    }

    Ok(bytes)
}

/// Generate a voice sample. Cached as `sample_{voice}_{tone}.mp3`.
#[tauri::command]
pub async fn generate_voice_sample_cmd(
    audio_dir: State<'_, AudioDir>,
    api_key: String,
    voice: String,
    tone: Option<String>,
) -> Result<Vec<u8>, String> {
    let tone_key = tone.as_deref().unwrap_or("auto");
    let file_path = audio_dir.0.join(format!("sample_{voice}_{}.mp3", tone_key.to_lowercase()));
    if file_path.exists() {
        return std::fs::read(&file_path).map_err(|e| format!("Failed to read sample: {e}"));
    }

    let voice_label = format!("{}{}", &voice[..1].to_uppercase(), &voice[1..]);
    let base_text = format!("Hi, I'm {voice_label}! How are you doing today?");
    let input = match tone.as_deref() {
        Some(t) if t != "Auto" => format!("Narration style:\n{t}\n\nText:\n{base_text}"),
        _ => base_text,
    };

    let request = openai::TtsRequest {
        model: "gpt-4o-mini-tts".to_string(),
        input,
        voice: voice.clone(),
    };

    let bytes = openai::text_to_speech("https://api.openai.com/v1", &api_key, &request).await?;
    std::fs::write(&file_path, &bytes).map_err(|e| format!("Failed to cache sample: {e}"))?;
    Ok(bytes)
}

/// Cached audio info: maps message_id → list of cached tones, plus last-selected tones.
#[derive(serde::Serialize)]
pub struct CachedAudioInfo {
    /// message_id → vec of tone keys (e.g. ["auto", "excited", "sad"])
    pub cached: HashMap<String, Vec<String>>,
    /// message_id → last selected tone key
    pub last_tones: HashMap<String, String>,
}

/// List all cached audio files and last-selected tones.
#[tauri::command]
pub async fn list_cached_audio_cmd(
    db: State<'_, Database>,
    audio_dir: State<'_, AudioDir>,
) -> Result<CachedAudioInfo, String> {
    let mut cached: HashMap<String, Vec<String>> = HashMap::new();

    let entries = std::fs::read_dir(&audio_dir.0).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("sample_") || !name.ends_with(".mp3") {
            continue;
        }
        let stem = name.strip_suffix(".mp3").unwrap();

        // New format: {message_id}_{tone}.mp3 — tone is always the last segment after last '_'
        // UUIDs contain hyphens not underscores, so split on last '_'
        if let Some(last_underscore) = stem.rfind('_') {
            let msg_id = &stem[..last_underscore];
            let tone = &stem[last_underscore + 1..];
            cached.entry(msg_id.to_string()).or_default().push(tone.to_string());
        } else {
            // Legacy format: {message_id}.mp3 — treat as "auto"
            cached.entry(stem.to_string()).or_default().push("auto".to_string());
        }
    }

    // Load last-selected tones from settings
    let mut last_tones: HashMap<String, String> = HashMap::new();
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        for msg_id in cached.keys() {
            if let Ok(Some(tone)) = get_setting(&conn, &format!("last_tone.{msg_id}")) {
                last_tones.insert(msg_id.clone(), tone);
            }
        }
    }

    Ok(CachedAudioInfo { cached, last_tones })
}

/// Delete all cached audio for a message.
#[tauri::command]
pub async fn delete_message_audio_cmd(
    db: State<'_, Database>,
    audio_dir: State<'_, AudioDir>,
    message_id: String,
) -> Result<(), String> {
    delete_audio_for_message(&audio_dir.0, &message_id);
    // Also clear the last_tone setting
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM settings WHERE key = ?1", rusqlite::params![format!("last_tone.{message_id}")])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Get cached speech audio for a message, if it exists.
#[tauri::command]
pub async fn get_speech_cmd(
    audio_dir: State<'_, AudioDir>,
    message_id: String,
) -> Result<Option<Vec<u8>>, String> {
    let file_path = audio_dir.0.join(format!("{message_id}.mp3"));
    if file_path.exists() {
        let bytes = std::fs::read(&file_path).map_err(|e| format!("Failed to read audio: {e}"))?;
        Ok(Some(bytes))
    } else {
        Ok(None)
    }
}

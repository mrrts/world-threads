use crate::ai::{openai::{self, StreamingRequest}, orchestrator};
use crate::db::queries::*;
use crate::db::Database;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_memory_artifacts_cmd(
    db: State<Database>,
    subject_id: String,
    artifact_type: String,
) -> Result<Vec<MemoryArtifact>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_memory_artifacts(&conn, &subject_id, &artifact_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_thread_summary_cmd(
    db: State<Database>,
    character_id: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
    Ok(get_thread_summary(&conn, &thread.thread_id))
}

/// Generate a fresh on-demand summary for a character's chat thread.
#[tauri::command]
pub async fn generate_chat_summary_cmd(
    db: State<'_, Database>,
    app_handle: AppHandle,
    api_key: String,
    character_id: String,
) -> Result<String, String> {
    let (character, recent_msgs, model_config, user_name) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let thread = get_thread_for_character(&conn, &character_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_messages(&conn, &thread.thread_id, 50).map_err(|e| e.to_string())?;
        let user_name = get_user_profile(&conn, &character.world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the protagonist".to_string());
        (character, recent_msgs, model_config, user_name)
    };

    if recent_msgs.is_empty() {
        return Ok("No conversation yet.".to_string());
    }

    let conversation: Vec<String> = recent_msgs.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
        .map(|m| format!("[{}] {}", m.role, m.content))
        .collect();

    let messages = vec![
        openai::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Summarize the recent conversation between {user} and {char}. \
                 Write a substantial narrative summary (12-24 sentences) covering the key events, \
                 emotional beats, and where things currently stand. Include a few key specific details — \
                 names, places, actions, or things said that capture the texture of the conversation. \
                 Write in third person. Refer to the human as \"{user}\", never as \"the user\" or \"you\". \
                 Refer to {char} by name.",
                user = user_name, char = character.display_name,
            ),
        },
        openai::ChatMessage {
            role: "user".to_string(),
            content: format!("Recent messages:\n{}", conversation.join("\n")),
        },
    ];

    let request = StreamingRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.5),
        max_completion_tokens: Some(1600),
        stream: true,
    };

    openai::chat_completion_stream(
        &model_config.chat_api_base(), &api_key, &request, &app_handle, "summary-token",
    ).await
}

/// Generate a fresh on-demand summary for a group chat thread.
#[tauri::command]
pub async fn generate_group_chat_summary_cmd(
    db: State<'_, Database>,
    app_handle: AppHandle,
    api_key: String,
    group_chat_id: String,
) -> Result<String, String> {
    let (characters, recent_msgs, model_config, user_name) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let recent_msgs = list_group_messages(&conn, &gc.thread_id, 50).map_err(|e| e.to_string())?;
        let user_name = get_user_profile(&conn, &gc.world_id)
            .ok().map(|p| p.display_name).unwrap_or_else(|| "the protagonist".to_string());

        let char_ids: Vec<String> = gc.character_ids.as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let characters: Vec<Character> = char_ids.iter()
            .filter_map(|id| get_character(&conn, id).ok())
            .collect();

        (characters, recent_msgs, model_config, user_name)
    };

    if recent_msgs.is_empty() {
        return Ok("No conversation yet.".to_string());
    }

    let char_names: Vec<String> = characters.iter().map(|c| c.display_name.clone()).collect();

    let conversation: Vec<String> = recent_msgs.iter()
        .filter(|m| m.role != "illustration" && m.role != "video")
        .map(|m| {
            let speaker = if m.role == "user" { "User".to_string() }
                else if let Some(sid) = &m.sender_character_id {
                    characters.iter().find(|c| &c.character_id == sid)
                        .map(|c| c.display_name.clone()).unwrap_or_else(|| m.role.clone())
                } else { m.role.clone() };
            format!("[{}] {}", speaker, m.content)
        })
        .collect();

    let messages = vec![
        openai::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Summarize the recent group conversation involving {user} and {chars}. \
                 Write a substantial narrative summary (12-24 sentences) covering the key events, \
                 emotional beats, and where things currently stand. Include a few key specific details — \
                 names, places, actions, or things said that capture the texture of the conversation. \
                 Write in third person. Refer to the human as \"{user}\", never as \"the user\" or \"you\". \
                 Refer to each character by name.",
                user = user_name, chars = char_names.join(" and "),
            ),
        },
        openai::ChatMessage {
            role: "user".to_string(),
            content: format!("Recent messages:\n{}", conversation.join("\n")),
        },
    ];

    let request = StreamingRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.5),
        max_completion_tokens: Some(1600),
        stream: true,
    };

    openai::chat_completion_stream(
        &model_config.chat_api_base(), &api_key, &request, &app_handle, "summary-token",
    ).await
}

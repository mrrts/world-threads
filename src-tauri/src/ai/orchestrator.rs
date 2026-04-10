use crate::ai::openai::{self, ChatRequest, ResponseFormat};
use crate::ai::prompts;
use crate::db::queries::*;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelConfig {
    pub dialogue_model: String,
    pub tick_model: String,
    pub memory_model: String,
    pub embedding_model: String,
    pub image_model: String,
    pub vision_model: String,
    #[serde(default = "default_provider")]
    pub ai_provider: String,
    #[serde(default = "default_lmstudio_url")]
    pub lmstudio_url: String,
}

fn default_provider() -> String { "openai".to_string() }
fn default_lmstudio_url() -> String { "http://127.0.0.1:1234".to_string() }

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            dialogue_model: "gpt-4o".to_string(),
            tick_model: "gpt-4o-mini".to_string(),
            memory_model: "gpt-4o-mini".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
            image_model: "gpt-image-1.5".to_string(),
            vision_model: "gpt-4.1".to_string(),
            ai_provider: default_provider(),
            lmstudio_url: default_lmstudio_url(),
        }
    }
}

impl ModelConfig {
    /// Base URL for chat completions — follows the provider toggle.
    pub fn chat_api_base(&self) -> String {
        if self.ai_provider == "lmstudio" {
            format!("{}/v1", self.lmstudio_url.trim_end_matches('/'))
        } else {
            "https://api.openai.com/v1".to_string()
        }
    }

    /// Image quality string appropriate for the configured image model.
    pub fn image_quality(&self) -> &str {
        if self.image_model.starts_with("gpt-image") {
            "medium"
        } else {
            "standard"
        }
    }

    /// The response_format field for the image request (dall-e models).
    pub fn image_response_format(&self) -> Option<String> {
        if self.image_model.starts_with("gpt-image") { None } else { Some("b64_json".to_string()) }
    }

    /// The output_format field for the image request (gpt-image models).
    pub fn image_output_format(&self) -> Option<String> {
        if self.image_model.starts_with("gpt-image") { Some("png".to_string()) } else { None }
    }

    /// Landscape image size appropriate for the configured image model.
    pub fn landscape_size(&self) -> &str {
        if self.image_model.starts_with("gpt-image") {
            "1536x1024"
        } else {
            "1792x1024"
        }
    }

    /// Base URL for embeddings and image generation — always OpenAI.
    pub fn openai_api_base(&self) -> String {
        "https://api.openai.com/v1".to_string()
    }
}

pub fn load_model_config(conn: &Connection) -> ModelConfig {
    let get = |key: &str, default: &str| -> String {
        get_setting(conn, key)
            .ok()
            .flatten()
            .unwrap_or_else(|| default.to_string())
    };
    ModelConfig {
        dialogue_model: get("model.dialogue", "gpt-4o"),
        tick_model: get("model.tick", "gpt-4o-mini"),
        memory_model: get("model.memory", "gpt-4o-mini"),
        embedding_model: get("model.embedding", "text-embedding-3-small"),
        image_model: get("model.image", "gpt-image-1.5"),
        vision_model: get("model.vision", "gpt-4.1"),
        ai_provider: get("ai_provider", "openai"),
        lmstudio_url: get("lmstudio_url", "http://127.0.0.1:1234"),
    }
}

pub fn save_model_config(conn: &Connection, config: &ModelConfig) -> Result<(), rusqlite::Error> {
    set_setting(conn, "model.dialogue", &config.dialogue_model)?;
    set_setting(conn, "model.tick", &config.tick_model)?;
    set_setting(conn, "model.memory", &config.memory_model)?;
    set_setting(conn, "model.embedding", &config.embedding_model)?;
    set_setting(conn, "model.image", &config.image_model)?;
    set_setting(conn, "model.vision", &config.vision_model)?;
    set_setting(conn, "ai_provider", &config.ai_provider)?;
    set_setting(conn, "lmstudio_url", &config.lmstudio_url)?;
    Ok(())
}

pub async fn run_dialogue_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    world: &World,
    character: &Character,
    recent_messages: &[Message],
    retrieved_snippets: &[String],
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    response_length: Option<&str>,
) -> Result<(String, Option<openai::Usage>), String> {
    let system = prompts::build_dialogue_system_prompt(world, character, user_profile, mood_directive, response_length);
    let messages = prompts::build_dialogue_messages(&system, recent_messages, retrieved_snippets);

    let request = ChatRequest {
        model: model.to_string(),
        messages,
        temperature: Some(1.0),
        max_completion_tokens: Some(1024),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let reply = response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from model".to_string())?;

    Ok((reply, response.usage))
}

/// Try to extract a JSON object from a response that may contain surrounding text.
fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape_next = false;
    for (i, ch) in raw[start..].char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(&raw[start..start + i + 1]);
                }
            }
            _ => {}
        }
    }
    None
}

pub async fn run_memory_update_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    character: &Character,
    thread_summary: &str,
    recent_messages: &[Message],
) -> Result<(serde_json::Value, Option<openai::Usage>), String> {
    let messages =
        prompts::build_memory_update_prompt(character, thread_summary, recent_messages);

    let is_openai = base_url.contains("openai.com");
    let request = ChatRequest {
        model: model.to_string(),
        messages,
        temperature: Some(0.3),
        max_completion_tokens: Some(600),
        response_format: if is_openai {
            Some(ResponseFormat { format_type: "json_object".to_string() })
        } else {
            None
        },
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from model".to_string())?;

    let val: serde_json::Value = serde_json::from_str::<serde_json::Value>(&raw)
        .or_else(|_| {
            extract_json_object(&raw)
                .ok_or_else(|| format!("No JSON found in memory response.\nRaw: {raw}"))
                .and_then(|s| serde_json::from_str(s).map_err(|e| format!("Failed to parse memory update: {e}")))
        })?;
    Ok((val, response.usage))
}

pub async fn generate_reaction_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    character: &Character,
    user_message: &str,
    assistant_reply: &str,
) -> Result<(Option<String>, Option<openai::Usage>), String> {
    let system = format!(
        r#"You are {name}, reacting to a text message. You may react with a SINGLE emoji or choose not to react.

Your personality: {identity}

RULES:
- Only react if it feels natural — not every message deserves a reaction.
- React to the USER's last message (not your own reply).
- Choose an emoji that fits your character's personality and emotional state.
- Respond with ONLY the emoji character (e.g. ❤️ or 😂) or the word NONE if no reaction.
- Never explain your choice. Just the emoji or NONE."#,
        name = character.display_name,
        identity = if character.identity.is_empty() { "a complex character".to_string() } else { character.identity.clone() },
    );

    let messages = vec![
        openai::ChatMessage { role: "system".to_string(), content: system },
        openai::ChatMessage { role: "user".to_string(), content: user_message.to_string() },
        openai::ChatMessage { role: "assistant".to_string(), content: assistant_reply.to_string() },
        openai::ChatMessage {
            role: "user".to_string(),
            content: "React to the user's message above with a single emoji, or say NONE.".to_string(),
        },
    ];

    let request = ChatRequest {
        model: model.to_string(),
        messages,
        temperature: Some(0.9),
        max_completion_tokens: Some(8),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let usage = response.usage;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    if raw.is_empty() || raw.to_uppercase() == "NONE" {
        return Ok((None, usage));
    }

    let trimmed = raw.chars().take(4).collect::<String>();
    if trimmed.chars().any(|c| !c.is_ascii()) {
        Ok((Some(trimmed), usage))
    } else {
        Ok((None, usage))
    }
}

pub async fn run_narrative_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    world: &World,
    character: &Character,
    recent_messages: &[Message],
    retrieved_snippets: &[String],
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    narration_tone: Option<&str>,
    narration_instructions: Option<&str>,
) -> Result<(String, Option<openai::Usage>), String> {
    let system = prompts::build_narrative_system_prompt(world, character, user_profile, mood_directive, narration_tone, narration_instructions);

    let mut msgs = Vec::new();

    let mut system_content = system.clone();
    if !retrieved_snippets.is_empty() {
        system_content.push_str("\n\nRELEVANT MEMORIES:\n");
        for s in retrieved_snippets {
            system_content.push_str(&format!("- {s}\n"));
        }
    }

    msgs.push(openai::ChatMessage {
        role: "system".to_string(),
        content: system_content,
    });

    for m in recent_messages {
        msgs.push(openai::ChatMessage {
            role: if m.role == "narrative" { "assistant".to_string() } else { m.role.clone() },
            content: if m.role == "narrative" {
                format!("[Narrative] {}", m.content)
            } else {
                m.content.clone()
            },
        });
    }

    msgs.push(openai::ChatMessage {
        role: "user".to_string(),
        content: "Write a narrative beat for this moment.".to_string(),
    });

    let request = ChatRequest {
        model: model.to_string(),
        messages: msgs,
        temperature: Some(1.0),
        max_completion_tokens: Some(512),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let reply = response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from model".to_string())?;

    Ok((reply, response.usage))
}

pub async fn generate_embeddings_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    texts: Vec<String>,
) -> Result<(Vec<Vec<f32>>, u32), String> {
    openai::create_embeddings_with_base(base_url, api_key, model, texts).await
}

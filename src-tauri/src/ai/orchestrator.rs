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
    /// User-declared context window for the local model, in tokens. Used to
    /// chunk long novelization prompts safely. The user sees this in the
    /// settings UI in 10k increments ("40k", "50k", ...); internally we aim
    /// for ~60% of this value per call to leave room for the system prompt,
    /// completion budget, and tokenizer variance.
    #[serde(default = "default_lmstudio_context_tokens")]
    pub lmstudio_context_tokens: u32,
}

fn default_provider() -> String { "openai".to_string() }
fn default_lmstudio_url() -> String { "http://127.0.0.1:1234".to_string() }
fn default_lmstudio_context_tokens() -> u32 { 40_000 }

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
            lmstudio_context_tokens: default_lmstudio_context_tokens(),
        }
    }
}

impl ModelConfig {
    /// Safe per-call prompt-token budget when running locally. Aims below the
    /// user's declared context window to leave room for system prompt +
    /// completion + tokenizer variance.
    pub fn safe_local_prompt_budget(&self) -> u32 {
        ((self.lmstudio_context_tokens as f64) * 0.6) as u32
    }

    /// Token budget available for *dialogue history* specifically. Starts
    /// from `safe_local_prompt_budget()` and reserves headroom for the
    /// system prompt (~4-6k), retrieval snippets (~2k), and response
    /// generation. The remainder is what we can fill with recent messages.
    ///
    /// Auto-scales with the user's declared local context setting: bump
    /// `lmstudio_context_tokens` up and dialogue memory deepens without
    /// touching any other config.
    pub fn safe_history_budget(&self) -> u32 {
        const RESERVED_HEADROOM: u32 = 8_000;
        self.safe_local_prompt_budget().saturating_sub(RESERVED_HEADROOM)
    }
}

impl ModelConfig {
    /// True when the configured chat provider is a local backend (LMStudio,
    /// and later llama.cpp / ollama / etc.). Small local models benefit from
    /// tighter, more declarative prompts than frontier models do.
    pub fn is_local(&self) -> bool {
        self.ai_provider == "lmstudio"
    }

    /// Base URL for chat completions — follows the provider toggle.
    pub fn chat_api_base(&self) -> String {
        if self.ai_provider == "lmstudio" {
            format!("{}/v1", self.lmstudio_url.trim_end_matches('/'))
        } else {
            "https://api.openai.com/v1".to_string()
        }
    }

    /// Apply a per-chat provider override onto this loaded config. The
    /// setting value is expected to be "" (no override), "lmstudio", or
    /// "openai". When overriding to a provider that differs from what
    /// `dialogue_model` was configured for, swaps in the companion model
    /// setting so the model ID matches the wire format of the target
    /// provider. No-ops on empty/unknown values.
    ///
    /// Called at the start of every dialogue/reaction command so the call
    /// transparently uses the user's per-chat preference. The override is
    /// only in scope for this ModelConfig instance — it does NOT modify
    /// saved settings.
    pub fn apply_provider_override(&mut self, conn: &Connection, override_key: &str) {
        let Some(ov) = get_setting(conn, override_key).ok().flatten() else { return; };
        match ov.as_str() {
            "openai" => {
                self.ai_provider = "openai".to_string();
                // Frontier model lives in its own setting so this works
                // even when the user's global dialogue_model is an
                // LM-Studio-only model ID like "llama-3.1-8b-instruct".
                let frontier_model = get_setting(conn, "model.dialogue_frontier")
                    .ok()
                    .flatten()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "gpt-4o".to_string());
                self.dialogue_model = frontier_model;
            }
            "lmstudio" => {
                self.ai_provider = "lmstudio".to_string();
                // When going to local, the user's globally-configured
                // dialogue_model is assumed to be a local model ID. If
                // their global is frontier, they'll need to manage this
                // manually — honest limitation, flagged in the UI copy.
            }
            _ => {}
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
        lmstudio_context_tokens: get("lmstudio_context_tokens", "40000")
            .parse::<u32>()
            .unwrap_or(40_000),
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
    set_setting(conn, "lmstudio_context_tokens", &config.lmstudio_context_tokens.to_string())?;
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
    group_context: Option<&prompts::GroupContext>,
    character_names: Option<&std::collections::HashMap<String, String>>,
    tone: Option<&str>,
    local_model: bool,
    mood_chain: &[String],
    leader: Option<&str>,
) -> Result<(String, Option<openai::Usage>), String> {
    let system = prompts::build_dialogue_system_prompt(world, character, user_profile, mood_directive, response_length, group_context, tone, local_model, mood_chain, leader);
    let messages = prompts::build_dialogue_messages(&system, recent_messages, retrieved_snippets, character_names);

    // Token caps sit ~25% above the sentence counts we instruct (see
    // prompts::response_length_block). Disobedient local models get some
    // extra room before they trip the cap, and trim_to_last_complete_sentence
    // cleans up anything that still runs over.
    let token_limit = match response_length {
        Some("Short") => Some(190),
        Some("Medium") => Some(320),
        Some("Long") => Some(1300),
        _ => None, // Auto — no limit, let the model decide
    };
    let request = ChatRequest {
        model: model.to_string(),
        messages,
        temperature: Some(0.95),
        max_completion_tokens: token_limit,
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let choice = response.choices.first()
        .ok_or_else(|| "No response from model".to_string())?;
    let raw = choice.message.content.clone();

    // Salvage mid-sentence cutoffs. When the model's reply is terminated by
    // max_completion_tokens (finish_reason == "length"), trim back to the
    // last complete sentence so the user never sees a half-finished word.
    // Then balance any unclosed openers (", *, or () so dialogue/action
    // markup never dangles. Natural stops are returned as-is.
    let reply = if choice.finish_reason.as_deref() == Some("length") {
        let trimmed = trim_to_last_complete_sentence(&raw);
        let base = if trimmed.is_empty() { raw.as_str() } else { trimmed.as_str() };
        balance_trailing_openers(base)
    } else {
        raw
    };

    Ok((reply, response.usage))
}

/// Walk backward through `s` and return the substring ending at the last
/// sentence-terminating punctuation (., !, ?, …), inclusive of any trailing
/// closing quotes, brackets, and asterisks.
///
/// Returns an empty string if no terminator is found — callers should fall
/// back to the original text (still passed through `balance_trailing_openers`)
/// in that case.
pub fn trim_to_last_complete_sentence(s: &str) -> String {
    let trimmed = s.trim_end();
    if trimmed.is_empty() { return String::new(); }

    let chars: Vec<(usize, char)> = trimmed.char_indices().collect();
    for i in (0..chars.len()).rev() {
        let (byte_idx, c) = chars[i];
        if !matches!(c, '.' | '!' | '?' | '…') { continue; }
        let mut end = byte_idx + c.len_utf8();
        // Pull in trailing closing punctuation that belongs to this sentence
        // (closing quotes, brackets, markdown-italics asterisks).
        let mut j = i + 1;
        while j < chars.len() {
            let (_, nc) = chars[j];
            if matches!(nc, '"' | '\'' | '»' | '”' | '’' | ')' | ']' | '}' | '*') {
                end += nc.len_utf8();
                j += 1;
            } else {
                break;
            }
        }
        return trimmed[..end].to_string();
    }
    String::new()
}

/// Append closing characters for any unclosed openers in `s`. Handles double
/// quotes, asterisk pairs (used for action beats like `*smiles*`), and
/// parentheses. The LLM may end mid-way through `"some dialogue` or
/// `*she turned` and we'd rather render `"some dialogue."` / `*she turned.*`
/// than leave the markup dangling.
pub fn balance_trailing_openers(s: &str) -> String {
    let mut stars: usize = 0;
    let mut paren_depth: i32 = 0;
    let mut dquotes: usize = 0;
    for c in s.chars() {
        match c {
            '*' => stars += 1,
            '(' => paren_depth += 1,
            ')' => { if paren_depth > 0 { paren_depth -= 1; } }
            '"' => dquotes += 1,
            _ => {}
        }
    }
    let mut out = s.to_string();
    // Close parens first (they're typically the innermost markup), then
    // asterisks (action tags), then quotes (outermost dialogue wrap).
    while paren_depth > 0 {
        out.push(')');
        paren_depth -= 1;
    }
    if stars % 2 == 1 { out.push('*'); }
    if dquotes % 2 == 1 { out.push('"'); }
    out
}

/// Streaming variant of run_dialogue_with_base — emits tokens via Tauri events.
/// Not currently called by any caller — kept for future reactivation when
/// dialogue goes streaming end-to-end.
#[allow(dead_code)]
pub async fn run_dialogue_streaming(
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
    group_context: Option<&prompts::GroupContext>,
    character_names: Option<&std::collections::HashMap<String, String>>,
    tone: Option<&str>,
    local_model: bool,
    app_handle: &tauri::AppHandle,
    event_name: &str,
    mood_chain: &[String],
    leader: Option<&str>,
) -> Result<String, String> {
    let system = prompts::build_dialogue_system_prompt(world, character, user_profile, mood_directive, response_length, group_context, tone, local_model, mood_chain, leader);
    let messages = prompts::build_dialogue_messages(&system, recent_messages, retrieved_snippets, character_names);

    let token_limit = match response_length {
        Some("Short") => Some(150),
        Some("Medium") => Some(250),
        Some("Long") => Some(1024),
        _ => None,
    };
    let request = openai::StreamingRequest {
        model: model.to_string(),
        messages,
        temperature: Some(0.95),
        max_completion_tokens: token_limit,
        stream: true,
    };

    openai::chat_completion_stream(base_url, api_key, &request, app_handle, event_name).await
}

/// Streaming variant of run_narrative_with_base — emits tokens via Tauri events.
/// Not currently called — kept for future reactivation when narrative
/// generation goes streaming end-to-end.
#[allow(dead_code)]
pub async fn run_narrative_streaming(
    base_url: &str,
    api_key: &str,
    model: &str,
    world: &World,
    character: &Character,
    additional_cast: Option<&[&Character]>,
    recent_messages: &[Message],
    retrieved_snippets: &[String],
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    narration_tone: Option<&str>,
    narration_instructions: Option<&str>,
    app_handle: &tauri::AppHandle,
    event_name: &str,
) -> Result<String, String> {
    let system = prompts::build_narrative_system_prompt(world, character, additional_cast, user_profile, mood_directive, narration_tone, narration_instructions);

    let mut msgs = Vec::new();
    let mut system_content = system.clone();
    if !retrieved_snippets.is_empty() {
        system_content.push_str("\n\nRELEVANT MEMORIES:\n");
        for s in retrieved_snippets {
            system_content.push_str(&format!("- {s}\n"));
        }
    }
    msgs.push(openai::ChatMessage { role: "system".to_string(), content: system_content });

    let mut last_time: Option<String> = None;
    for m in recent_messages {
        if m.role == "illustration" || m.role == "video" { continue; }
        if let Some(ref wt) = m.world_time {
            if last_time.as_deref() != Some(wt) {
                let formatted = wt.split(' ').map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        Some(first) => first.to_uppercase().to_string() + &c.as_str().to_lowercase(),
                        None => String::new(),
                    }
                }).collect::<Vec<_>>().join(" ");
                msgs.push(openai::ChatMessage {
                    role: "system".to_string(),
                    content: format!("[It is now {formatted}.]"),
                });
                last_time = Some(wt.clone());
            }
        }
        msgs.push(openai::ChatMessage {
            role: if m.role == "narrative" || m.role == "context" { "assistant".to_string() } else { m.role.clone() },
            content: if m.role == "context" {
                format!("[Additional Context from Another Chat] {}", m.content)
            } else if m.role == "narrative" {
                format!("[Narrative] {}", m.content)
            } else {
                m.content.clone()
            },
        });
    }

    let user_prompt = if let Some(instructions) = narration_instructions {
        if !instructions.is_empty() {
            format!("Write a narrative beat for this moment.\n\nIMPORTANT DIRECTION — you MUST follow this:\n{instructions}")
        } else {
            "Write a narrative beat for this moment.".to_string()
        }
    } else {
        "Write a narrative beat for this moment.".to_string()
    };
    msgs.push(openai::ChatMessage { role: "user".to_string(), content: user_prompt });

    let request = openai::StreamingRequest {
        model: model.to_string(),
        messages: msgs,
        temperature: Some(0.95),
        max_completion_tokens: Some(1024),
        stream: true,
    };

    openai::chat_completion_stream(base_url, api_key, &request, app_handle, event_name).await
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

/// Generate a short emoji-like reaction from a character to a just-exchanged
/// message pair. Not currently wired up — reactions were disabled in
/// Ask the model for the single emoji the character would *feel* toward
/// this user message — a read of the atmosphere, not a rating of the
/// message's aptness.
///
/// Takes the preceding few messages as scene context so the picker can
/// match register (a reflective exchange gets a quiet reaction; a joke
/// gets a light one). Takes the thread's mood_reduction as a soft
/// continuity hint — not a direct echo.
///
/// Cheap call: ~12 output tokens, temperature 0.9. On failure the caller
/// should fall back to `prompts::pick_character_reaction_emoji`.
pub async fn pick_character_reaction_via_llm(
    base_url: &str,
    api_key: &str,
    model: &str,
    user_message: &str,
    mood_reduction: &[String],
    recent_context: &[Message],
) -> Result<String, String> {
    let atmosphere = if mood_reduction.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nFelt weather of the thread so far (most recent first): {}. \
             Let this inform the register, not the choice.",
            mood_reduction.join(" ")
        )
    };

    // Compact scene: last ~4 messages before the target, rendered as
    // "role: content" lines. Each content clipped to ~240 chars so the
    // prompt stays short.
    let scene = {
        let tail: Vec<String> = recent_context.iter()
            .rev()
            .take(4)
            .rev()
            .map(|m| {
                let clipped: String = m.content.chars().take(240).collect();
                let role = if m.role == "user" { "USER" } else { "CHARACTER" };
                format!("{role}: {clipped}")
            })
            .collect();
        if tail.is_empty() { String::new() } else {
            format!("\n\nImmediate scene (most recent last):\n{}", tail.join("\n"))
        }
    };

    let system = format!(
        r#"You pick a single emoji that captures what the CHARACTER FEELS in response to the user's message. The character's own internal state — not the theme of the conversation, not the atmosphere of the scene, not the topic being discussed.

Lean hard on FACES and HEARTS. These carry felt state directly. Ambient or thematic emojis (🕊️, 🕯️, 🌙, 🌧️, ⏳, ✝️, 🌱) are permitted ONLY if they genuinely represent the character's interior *right now* — not just the mood in the air or the subject of conversation. When in doubt, reach for a face.

Guidelines (tendencies, not walls — the goal is the truest single-emoji read of the character's state):
- Output ONLY the emoji character. No text, no quotes, no punctuation.
- FEELING OVER THEME, USUALLY. A topic-match — 💔 for grief-talk, ✝️ for faith-talk, 🌧️ for a talk about sadness — is usually the lazy instinct: it rates the subject instead of feeling anything. Default to a face that captures what the character feels hearing it: softened (🥺), held (🤗), quiet ache (😔). BUT if the character's felt state genuinely IS the thematic emoji — heart actually heavy, reverence actually welling — then it's right. The test: picking it because of the topic = skip; picking it because it IS the feeling = keep.
- FEELING OVER AMBIENCE, USUALLY. Don't paint the weather when you could paint the interior. A reverent conversation doesn't automatically mean 🕊️ — it might mean 🥹 (moved) or 🫣 (struck). Same exception: if the ambient emoji IS the character's interior right now, use it.
- LITERAL READS ARE OCCASIONALLY RIGHT. "Land a moment" → 🎯 is usually the laziest possible pick, reading words instead of register. But if the character genuinely feels a small targeting-click of understanding, 🎯 can be exactly it. Literal reads have to earn their way in; don't pick them as a default.
- ACHIEVEMENT-FAMILY EMOJIS, SPARINGLY. 🎯 💯 ✅ 🏆 👏 💪 (and 🔥-for-approval) occasionally catch real humor, whimsy, or shared delight — and when they do, they're great. Most of the time they collapse into rating the message. Reach for one only when it IS the character's felt state, not when it evaluates the user's.
- MATCH THE REGISTER. Light moment → light feeling. Reverent moment → quiet feeling. Heavy moment → held feeling.{scene}{atmosphere}"#
    );

    let messages = vec![
        openai::ChatMessage { role: "system".to_string(), content: system },
        openai::ChatMessage { role: "user".to_string(), content: user_message.to_string() },
    ];

    let request = ChatRequest {
        model: model.to_string(),
        messages,
        temperature: Some(0.7),
        max_completion_tokens: Some(12),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    let cleaned: String = raw.chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '"' | '\'' | '.' | ',' | '!' | '?' | ':' | ';'))
        .take(8)
        .collect();

    if cleaned.is_empty() || !cleaned.chars().any(|c| !c.is_ascii()) {
        return Err(format!("no emoji in LLM response: {raw:?}"));
    }

    Ok(cleaned)
}

/// Weave a deeper-truth moment from a conversation into a subject's
/// existing description. Used by the Promote-to-Canon flow: the user
/// decides a specific message reveals something about a character (or
/// themselves) that their description should now reflect. The model
/// revises the description to integrate the truth organically rather
/// than appending a sentence.
pub async fn generate_canon_weave_description(
    base_url: &str,
    api_key: &str,
    model: &str,
    subject_label: &str,
    current_description: &str,
    context_messages: &[Message],
    source_message: &Message,
    source_speaker_label: &str,
) -> Result<(String, Option<openai::Usage>), String> {
    let rendered_context: Vec<String> = context_messages.iter()
        .map(|m| {
            let role = if m.message_id == source_message.message_id { "★ SOURCE" }
                else if m.role == "user" { "USER" }
                else { "CHARACTER" };
            let clipped: String = m.content.chars().take(600).collect();
            format!("[{role}] {clipped}")
        })
        .collect();
    let context_block = rendered_context.join("\n\n");

    let system = r#"You revise a subject's prose description to integrate a specific moment of deeper truth that has come up in a conversation. You are NOT appending a sentence. You are NOT summarizing. You are rewriting the description so that the truth revealed in the moment is now quietly present in the portrait — deeper, refined, more exactly what this subject is.

Preserve the voice, length, and overall structure of the original description. Keep anything that was already true. The revision should feel like the same description, but wiser — as if the writer now knows something they didn't before, and that knowledge has colored the whole portrait.

Do NOT reference the moment directly in the revised text. Do NOT name the conversation. The moment informs the revision; it does not appear in it.

Return ONLY the revised description prose. No preamble, no quotes, no commentary."#.to_string();

    let user = format!(
        "SUBJECT: {subject_label}\n\nCURRENT DESCRIPTION:\n{current_description}\n\nTHE REVEALING MOMENT (with surrounding context; ★ marks the source line):\n{context_block}\n\nSOURCE LINE:\n{source_speaker_label}: {source_content}\n\nWrite the revised description.",
        subject_label = subject_label,
        current_description = current_description,
        context_block = context_block,
        source_speaker_label = source_speaker_label,
        source_content = source_message.content,
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.7),
        max_completion_tokens: Some(2000),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let usage = response.usage;
    let text = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    if text.is_empty() {
        return Err("empty weave response".to_string());
    }

    Ok((text, usage))
}

/// chat_cmds to keep cost down. Kept for future reactivation.
#[allow(dead_code)]
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
        temperature: Some(0.95),
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
    additional_cast: Option<&[&Character]>,
    recent_messages: &[Message],
    retrieved_snippets: &[String],
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    narration_tone: Option<&str>,
    narration_instructions: Option<&str>,
) -> Result<(String, Option<openai::Usage>), String> {
    let system = prompts::build_narrative_system_prompt(world, character, additional_cast, user_profile, mood_directive, narration_tone, narration_instructions);

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

    let mut last_time: Option<String> = None;
    for m in recent_messages {
        if m.role == "illustration" || m.role == "video" {
            continue;
        }
        if let Some(ref wt) = m.world_time {
            if last_time.as_deref() != Some(wt) {
                let formatted = wt.split(' ').map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        Some(first) => first.to_uppercase().to_string() + &c.as_str().to_lowercase(),
                        None => String::new(),
                    }
                }).collect::<Vec<_>>().join(" ");
                msgs.push(openai::ChatMessage {
                    role: "system".to_string(),
                    content: format!("[It is now {formatted}.]"),
                });
                last_time = Some(wt.clone());
            }
        }
        msgs.push(openai::ChatMessage {
            role: if m.role == "narrative" || m.role == "context" { "assistant".to_string() } else { m.role.clone() },
            content: if m.role == "context" {
                format!("[Additional Context from Another Chat] {}", m.content)
            } else if m.role == "narrative" {
                format!("[Narrative] {}", m.content)
            } else {
                m.content.clone()
            },
        });
    }

    // Put custom instructions in the user message where the model prioritizes them
    let user_prompt = if let Some(instructions) = narration_instructions {
        if !instructions.is_empty() {
            format!("Write a narrative beat for this moment.\n\nIMPORTANT DIRECTION — you MUST follow this:\n{instructions}")
        } else {
            "Write a narrative beat for this moment.".to_string()
        }
    } else {
        "Write a narrative beat for this moment.".to_string()
    };

    msgs.push(openai::ChatMessage {
        role: "user".to_string(),
        content: user_prompt,
    });

    let request = ChatRequest {
        model: model.to_string(),
        messages: msgs,
        temperature: Some(0.95),
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

/// Two-step illustration: generate a scene description, then create an image with reference portraits.
pub async fn generate_illustration_with_base(
    base_url: &str,
    openai_base_url: &str,
    api_key: &str,
    chat_model: &str,
    image_model: &str,
    image_quality: &str,
    image_size: &str,
    image_output_format: Option<&str>,
    world: &World,
    character: &Character,
    additional_cast: Option<&[&Character]>,
    recent_messages: &[Message],
    user_profile: Option<&UserProfile>,
    reference_images: &[Vec<u8>],
    custom_instructions: Option<&str>,
    has_previous_scene: bool,
    include_scene_summary: bool,
    all_character_names: Option<&[String]>,
    character_names_map: Option<&std::collections::HashMap<String, String>>,
) -> Result<(String, Vec<u8>, Option<openai::Usage>), String> {
    // Step 1: Generate scene description (if requested)
    let (scene_description, chat_usage) = if include_scene_summary {
        let scene_messages = prompts::build_scene_description_prompt(world, character, additional_cast, user_profile, recent_messages, character_names_map);

        let scene_request = ChatRequest {
            model: chat_model.to_string(),
            messages: scene_messages,
            temperature: Some(0.95),
            max_completion_tokens: Some(500),
            response_format: None,
        };

        let scene_response = openai::chat_completion_with_base(base_url, api_key, &scene_request).await?;
        let desc = scene_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No scene description from model".to_string())?;

        log::info!("[Illustration] Scene description ({} chars): {:.200}", desc.len(), desc);
        (desc, scene_response.usage)
    } else {
        log::info!("[Illustration] Skipping scene summary (user opted out)");
        (String::new(), None)
    };

    // Step 2: Generate illustration with reference portraits
    let user_name = user_profile
        .map(|p| p.display_name.as_str())
        .unwrap_or("the human");

    // Determine lighting from world time of day
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
            _ => "Gentle diffused natural lighting, nostalgic and contemplative mood.",
        })
        .unwrap_or("Gentle diffused natural lighting, nostalgic and contemplative mood.");

    let mut prompt_parts = vec![
        "Hand-painted watercolor illustration in a lush, realistic style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones with pops of verdant green and sky blue.".to_string(),
        time_lighting.to_string(),
        "Wide cinematic composition showing two characters in a scene together.".to_string(),
    ];

    // Describe reference images in order: user avatar, character portrait(s), then optional previous scene
    if let Some(names) = all_character_names {
        // Group chat: user + multiple characters
        let mut idx = 1;
        let mut descriptions = Vec::new();
        descriptions.push(format!("Reference image {} is {}.", idx, user_name));
        idx += 1;
        for name in names {
            descriptions.push(format!("Reference image {} is {}.", idx, name));
            idx += 1;
        }
        descriptions.push("ALL characters MUST appear in the illustration, recognizable from their reference images.".to_string());
        if has_previous_scene {
            descriptions.push(format!(
                "Reference image {} is the PREVIOUS scene. Use it for visual continuity of setting, \
                 character positions, and atmosphere, but advance the scene to match the new description.", idx
            ));
        }
        prompt_parts.push(descriptions.join(" "));
    } else {
        // Individual chat: user + one character
        if reference_images.len() >= 2 {
            prompt_parts.push(format!(
                "The first reference image is {user}. The second reference image is {char}. \
                 Both characters MUST appear in the illustration, recognizable from their reference images.",
                user = user_name,
                char = character.display_name,
            ));
        } else if reference_images.len() == 1 {
            prompt_parts.push(format!(
                "The reference image is {char}. They must appear in the illustration, recognizable from the reference.",
                char = character.display_name,
            ));
        }

        if has_previous_scene && reference_images.len() >= 3 {
            prompt_parts.push(
                "The third reference image is the PREVIOUS scene. Use it for visual continuity of setting, \
                 character positions, and atmosphere, but advance the scene to match the new description.".to_string()
            );
        }
    }

    if !scene_description.is_empty() {
        prompt_parts.push(format!("SCENE:\n{scene_description}"));
    }

    if let Some(instructions) = custom_instructions {
        if !instructions.is_empty() {
            prompt_parts.push(format!("USER'S SPECIFIC REQUEST: {instructions}"));
        }
    }

    prompt_parts.push("CRITICAL: The image must contain absolutely no text, no words, no letters, no numbers, no writing, no labels, no titles, no captions, no watermarks, no signatures, no UI elements, no names.".to_string());

    let prompt = prompt_parts.join(" ");

    let image_response = openai::generate_image_edit_with_base(
        openai_base_url, api_key, image_model,
        &prompt, reference_images,
        image_size, image_quality,
        image_output_format,
    ).await?;

    let b64 = image_response.data.first()
        .and_then(|d| d.image_b64())
        .ok_or_else(|| "No image data in response".to_string())?;

    // Decode base64 to bytes
    let image_bytes = openai_base64_decode(b64)?;

    Ok((scene_description, image_bytes, chat_usage))
}

/// Simple base64 decoder for image data.
fn openai_base64_decode(input: &str) -> Result<Vec<u8>, String> {
    openai_base64_decode_pub(input)
}

/// Public base64 decoder (used by chat_cmds for adjust_illustration).
pub fn openai_base64_decode_pub(input: &str) -> Result<Vec<u8>, String> {
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

    let input = input.trim().trim_end_matches('=');
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0u32;
    for &b in input.as_bytes() {
        if b == b'\n' || b == b'\r' || b == b' ' { continue; }
        let val = if (b as usize) < 128 { DECODE[b as usize] } else { 255 };
        if val == 255 { return Err(format!("Invalid base64 character: {}", b as char)); }
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Ok(out)
}

pub async fn generate_embeddings_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    texts: Vec<String>,
) -> Result<(Vec<Vec<f32>>, u32), String> {
    openai::create_embeddings_with_base(base_url, api_key, model, texts).await
}

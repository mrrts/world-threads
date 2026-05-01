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
    /// Frontier (OpenAI) dialogue model used when a chat opts into the
    /// "Frontier" per-chat provider override. Separate from `dialogue_model`
    /// because the primary dialogue_model may be configured for the user's
    /// local setup (an LM Studio model ID) — we need a frontier-specific
    /// model ID that's valid against OpenAI's API independent of that.
    #[serde(default = "default_dialogue_frontier")]
    pub dialogue_model_frontier: String,
}

fn default_provider() -> String { "openai".to_string() }
fn default_lmstudio_url() -> String { "http://127.0.0.1:1234".to_string() }
fn default_lmstudio_context_tokens() -> u32 { 40_000 }
fn default_dialogue_frontier() -> String { "gpt-4o".to_string() }

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
            dialogue_model_frontier: default_dialogue_frontier(),
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
                // Frontier model is stored separately so this works even
                // when the user's global dialogue_model is an LM-Studio-
                // only model ID like "llama-3.1-8b-instruct".
                if !self.dialogue_model_frontier.is_empty() {
                    self.dialogue_model = self.dialogue_model_frontier.clone();
                }
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
        dialogue_model_frontier: get("model.dialogue_frontier", "gpt-4o"),
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
    set_setting(conn, "model.dialogue_frontier", &config.dialogue_model_frontier)?;
    Ok(())
}

pub async fn run_dialogue_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    memory_model: Option<&str>,
    send_history: bool,
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
    kept_ids: &[String],
    illustration_captions: &std::collections::HashMap<String, String>,
    reactions_by_msg: &std::collections::HashMap<String, Vec<crate::db::queries::Reaction>>,
    drift_correction: Option<&str>,
    recent_journals: &[crate::db::queries::JournalEntry],
    latest_reading: Option<&crate::db::queries::DailyReading>,
    latest_meanwhile: Option<&crate::db::queries::MeanwhileEvent>,
    active_quests: &[crate::db::queries::Quest],
    relational_stance: Option<&str>,
    load_test_anchor: Option<&str>,
    current_location_override: Option<&str>,
    // Pre-built FORMULA MOMENTSTAMP block to inject at the head of the
    // dialogue system prompt. Populated by callers when the chat's
    // reactions_mode is "off" — see ai::momentstamp::build_formula_
    // momentstamp. None when reactions are enabled (no injection).
    formula_momentstamp: Option<&str>,
) -> Result<(String, Option<openai::Usage>), String> {
    // When the user has disabled conversation history for this chat, strip
    // prior turns, semantic memories, and moment markers — the character
    // sees only the system prompt plus the triggering message (the most
    // recent turn). Gives the user a clean "blank slate" pass without
    // having to reset the thread.
    let empty_snippets: Vec<String> = Vec::new();
    let empty_kept: Vec<String> = Vec::new();
    let empty_captions: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let empty_reactions: std::collections::HashMap<String, Vec<crate::db::queries::Reaction>> = std::collections::HashMap::new();
    let tail_slice: Vec<Message>;
    let effective_msgs: &[Message] = if send_history {
        recent_messages
    } else if let Some(last) = recent_messages.last() {
        tail_slice = vec![last.clone()];
        &tail_slice
    } else {
        &[]
    };
    let effective_snippets: &[String] = if send_history { retrieved_snippets } else { &empty_snippets };
    let effective_kept: &[String] = if send_history { kept_ids } else { &empty_kept };
    let effective_captions = if send_history { illustration_captions } else { &empty_captions };
    let effective_reactions = if send_history { reactions_by_msg } else { &empty_reactions };
    let user_display_name = user_profile.map(|p| p.display_name.as_str());

    // Derive a voice-mirror from this character's own recent messages
    // so the prompt's voice-block is anchored in actual speech instead
    // of just the VOICE RULES bullets. Samples come from the same slice
    // the model already sees below as conversation history; duplicating
    // them in high-attention system context is the point.
    let own_voice_samples = prompts::pick_own_voice_samples(
        &character.character_id,
        effective_msgs,
        group_context.is_some(),
        6,
    );
    let mut system = prompts::build_dialogue_system_prompt(world, character, user_profile, mood_directive, response_length, group_context, tone, local_model, mood_chain, leader, recent_journals, latest_reading, &own_voice_samples, latest_meanwhile, active_quests, relational_stance, load_test_anchor);
    // Formula momentstamp (reactions=off "depth signal" reward): when the
    // caller pre-built a chat-state signature derived from 𝓕 := (𝓡, 𝓒),
    // prepend it at the HEAD of the system prompt so the conditioning
    // applies before any other block. See ai::momentstamp.
    //
    // Test hook — env var WORLDTHREADS_NO_MOMENTSTAMP_LEAD=1 suppresses
    // ONLY the head-of-prompt prepending. The momentstamp continues to
    // be computed and persisted (formula_signature column on the
    // assistant message) and the inline series in chat history continues
    // to fire unchanged via build_dialogue_messages. Used for A/B
    // ablation: does primacy-position specifically do work, separate
    // from the inline series + chain handoff? Sibling to
    // WORLDTHREADS_NO_FORMULA / WORLDTHREADS_NO_RYAN_FORMULA env hooks.
    if let Some(stamp) = formula_momentstamp {
        let suppress_lead = std::env::var("WORLDTHREADS_NO_MOMENTSTAMP_LEAD")
            .map(|v| v == "1").unwrap_or(false);
        if !suppress_lead {
            let mut prefixed = String::with_capacity(stamp.len() + system.len() + 4);
            prefixed.push_str(stamp);
            prefixed.push_str("\n\n");
            prefixed.push_str(&system);
            system = prefixed;
        }
    }
    // Conscience-pass retry path: a prior draft drifted on an invariant,
    // and the grader returned a concrete correction note. Append it at the
    // end of the system block so it sits in the high-attention tail right
    // before the dialogue messages.
    if let Some(note) = drift_correction {
        system.push_str("\n\n");
        system.push_str(note);
    }
    let messages = prompts::build_dialogue_messages(&system, effective_msgs, effective_snippets, character_names, effective_kept, effective_captions, effective_reactions, user_display_name, current_location_override);

    // Unsent-draft pre-pass — DISABLED 2026-04-20. The extra call produced
    // an undercurrent-carrying reply, but even casual greetings ended up
    // feeling over-weighted because the pre-pass invents subtext when the
    // scene doesn't have any. Kept in source (pick_unsent_draft /
    // append_unsent_draft_note) for easy reactivation. To re-enable:
    // restore the `if send_history { if let Some(mem_model) = memory_model
    // { ... } }` block here; change `let messages` above back to `let mut
    // messages`; and revisit the "invented subtext on light scenes" issue
    // before shipping.
    let _ = memory_model;

    // Token caps — tight enough that the trim-to-last-complete-sentence
    // salvage below actually lands at the sentence target, not 2x over.
    // Group chats get harder caps because they drift long: multiple
    // characters each hitting a loose cap adds up to a wall of text per
    // turn, even when each individual reply is "only" 3-4 sentences.
    // The group-specific values force 1-2 sentences at Short.
    let is_group = group_context.is_some();
    let token_limit = match (response_length, is_group) {
        (Some("Short"), true) => Some(50),
        (Some("Short"), false) => Some(80),
        (Some("Medium"), true) => Some(140),
        (Some("Medium"), false) => Some(220),
        (Some("Long"), true) => Some(900),
        (Some("Long"), false) => Some(1300),
        _ => None, // Auto — no limit, let the model decide
    };
    log::info!(
        "[Dialogue] {} response_length={:?} → token_limit={:?}",
        if is_group { "group" } else { "solo" },
        response_length,
        token_limit,
    );

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

    let reply = post_process_dialogue_reply_for_persist(&raw, choice.finish_reason.as_deref());

    Ok((reply, response.usage))
}

/// Remove `*"..."*` patterns where asterisks directly wrap a quoted phrase
/// with nothing else (only optional whitespace) between them. The interior
/// quote is preserved. Action beats like `*says "stop"*` are NOT matched —
/// they contain non-quote content inside the pair.
pub fn strip_asterisk_wrapped_quotes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'*' {
            // Require left flanking whitespace/start for the opening `*`.
            // Without this, a closing action `*` from `*action*` can be
            // misread as the opening `*` for a `*"..."*` span that crosses
            // into the next action block.
            let left_ok = i == 0 || bytes[i - 1].is_ascii_whitespace();
            if left_ok {
            // Look ahead: optional whitespace, then a `"`, then find the
            // closing `"`, then optional whitespace, then `*`. If the whole
            // run matches, emit just the quoted substring.
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') { j += 1; }
            if j < bytes.len() && bytes[j] == b'"' {
                let q_start = j;
                let mut k = j + 1;
                while k < bytes.len() && bytes[k] != b'"' && bytes[k] != b'\n' { k += 1; }
                if k < bytes.len() && bytes[k] == b'"' {
                    let q_end = k + 1;
                    let mut m = q_end;
                    while m < bytes.len() && (bytes[m] == b' ' || bytes[m] == b'\t') { m += 1; }
                    if m < bytes.len() && bytes[m] == b'*' {
                        // Require right flanking boundary for the closing `*`
                        // to mirror frontend behavior and avoid spanning across
                        // neighboring emphasized runs.
                        let right_ok = m + 1 == bytes.len()
                            || bytes[m + 1].is_ascii_whitespace()
                            || matches!(bytes[m + 1], b'.' | b',' | b'!' | b'?' | b';' | b':');
                        if right_ok {
                            // Matched: emit just the quote (lossless of its own content).
                            out.push_str(&s[q_start..q_end]);
                            i = m + 1;
                            continue;
                        }
                    }
                }
            }
            }
        }
        // No match — copy the byte (safe: we never split a UTF-8 codepoint
        // because we only skip over quoted ASCII runs and whitespace).
        let ch_end = next_char_end(s, i);
        out.push_str(&s[i..ch_end]);
        i = ch_end;
    }
    out
}

fn next_char_end(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    if start >= bytes.len() { return start; }
    // UTF-8 continuation byte count from the leading byte.
    let b = bytes[start];
    let len = if b < 0x80 { 1 }
              else if b < 0xC0 { 1 }  // isolated continuation — shouldn't happen; skip one
              else if b < 0xE0 { 2 }
              else if b < 0xF0 { 3 }
              else { 4 };
    (start + len).min(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn strips_bare_asterisk_wrapped_quote() {
        let input = r#"* "That makes sense." *"#;
        assert_eq!(strip_asterisk_wrapped_quotes(input), r#""That makes sense.""#);
    }

    #[test]
    fn does_not_cross_action_quote_action_boundaries() {
        let input = r#"*I glance at you.* "Copy." *The fountain chatters beside us.*"#;
        assert_eq!(strip_asterisk_wrapped_quotes(input), input);
    }

    fn minimal_world() -> World {
        World {
            world_id: "w".into(),
            name: "W".into(),
            description: String::new(),
            tone_tags: json!([]),
            invariants: json!([]),
            state: json!({}),
            created_at: String::new(),
            updated_at: String::new(),
            derived_formula: None,
        }
    }

    fn minimal_character() -> Character {
        Character {
            character_id: "c".into(),
            world_id: "w".into(),
            display_name: "Dreamer".into(),
            identity: String::new(),
            voice_rules: json!([]),
            boundaries: json!([]),
            backstory_facts: json!([]),
            relationships: json!({}),
            state: json!({}),
            avatar_color: String::new(),
            sex: "male".into(),
            is_archived: false,
            created_at: String::new(),
            updated_at: String::new(),
            visual_description: String::new(),
            visual_description_portrait_id: None,
            inventory: serde_json::Value::Array(vec![]),
            last_inventory_day: None,
            signature_emoji: String::new(),
            action_beat_density: "normal".into(),
            derived_formula: None,
            has_read_empiricon: false,
        }
    }

    fn minimal_profile(display: &str) -> UserProfile {
        UserProfile {
            world_id: "w".into(),
            display_name: display.into(),
            description: String::new(),
            facts: json!([]),
            boundaries: json!([]),
            avatar_file: String::new(),
            updated_at: String::new(),
            derived_formula: None,
            derived_summary: None,
        }
    }

    fn minimal_message(role: &str, content: &str) -> Message {
        Message {
            message_id: "m1".into(),
            thread_id: "t1".into(),
            role: role.into(),
            content: content.into(),
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: "2026-04-29T12:00:00Z".into(),
            world_day: None,
            world_time: None,
            address_to: None,
            mood_chain: None,
            is_proactive: false,
            formula_signature: None,
        }
    }

    #[test]
    fn narrative_messages_emit_location_correction_with_explicit_override() {
        let world = minimal_world();
        let character = minimal_character();
        let profile = minimal_profile("Casey");
        let system = prompts::build_narrative_system_prompt(
            &world,
            &character,
            None,
            Some(&profile),
            None,
            None,
            None,
        );
        let msgs = build_narrative_messages(
            &system,
            &[minimal_message("user", "Write the next beat.")],
            &HashMap::new(),
            &[],
            Some("Garden Patio"),
        );
        assert!(
            msgs.iter().any(|m| {
                m.role == "system"
                    && m.content.contains("[SCENE LOCATION RIGHT NOW — AUTHORITATIVE: Garden Patio.")
                    && m.content.contains("this beat is grounded in Garden Patio")
            }),
            "narrative message assembly should keep the authoritative location correction when an explicit override is present"
        );
    }

    #[test]
    fn streaming_dialogue_messages_emit_location_correction_with_explicit_override() {
        let world = minimal_world();
        let character = minimal_character();
        let profile = minimal_profile("Casey");
        let system = prompts::build_dialogue_system_prompt(
            &world,
            &character,
            Some(&profile),
            None,
            None,
            None,
            None,
            false,
            &[],
            None,
            &[],
            None,
            &[],
            None,
            &[],
            None,
            None,
        );
        let msgs = build_dialogue_streaming_messages(
            &system,
            &[minimal_message("user", "Where are we?")],
            &[],
            None,
            &[],
            &HashMap::new(),
            Some("Casey"),
            Some("Garden Patio"),
        );
        assert!(
            msgs.iter().any(|m| {
                m.role == "system"
                    && m.content.contains("[SCENE LOCATION RIGHT NOW — AUTHORITATIVE: **Garden Patio**")
                    && m.content.contains("The scene is happening HERE")
            }),
            "streaming dialogue message assembly should keep the authoritative location correction when an explicit override is present"
        );
    }

    #[test]
    fn scene_description_messages_emit_location_correction_with_explicit_override() {
        let world = minimal_world();
        let character = minimal_character();
        let profile = minimal_profile("Casey");
        let msgs = build_scene_description_messages(
            &world,
            &character,
            None,
            Some(&profile),
            &[minimal_message("user", "Paint the scene.")],
            None,
            Some("Garden Patio"),
        );
        assert!(
            msgs.iter().any(|m| {
                m.role == "system"
                    && m.content.contains("[SCENE LOCATION RIGHT NOW — AUTHORITATIVE: Garden Patio.")
                    && m.content.contains("Place this illustration HERE")
            }),
            "scene description assembly should keep the authoritative location correction when an explicit override is present"
        );
    }

    #[test]
    fn dialogue_messages_emit_location_correction_with_explicit_override() {
        // Pins the plumbing fix at 603f03d: every dialogue call site
        // (chat_cmds::send_message_cmd, prompt_character_cmd, reset_to_message_cmd
        // and group_chat_cmds::send_group_message_cmd, prompt_group_character_cmd
        // plus the conscience-pass regen siblings — 7 sites total) reads the
        // chat-row current_location and threads it through run_dialogue_with_base
        // → build_dialogue_messages. Without this assertion, a regression to
        // passing None at any of those sites would silently fall through to
        // DEFAULT_CHAT_LOCATION = "Town Square" without test coverage.
        let world = minimal_world();
        let character = minimal_character();
        let profile = minimal_profile("Casey");
        let system = prompts::build_dialogue_system_prompt(
            &world,
            &character,
            Some(&profile),
            None,
            None,
            None,
            None,
            false,
            &[],
            None,
            &[],
            None,
            &[],
            None,
            &[],
            None,
            None,
        );
        let msgs = prompts::build_dialogue_messages(
            &system,
            &[minimal_message("user", "Where are we?")],
            &[],
            None,
            &[],
            &HashMap::new(),
            &HashMap::new(),
            Some("Casey"),
            Some("Garden Patio"),
        );
        assert!(
            msgs.iter().any(|m| {
                m.role == "system"
                    && m.content.contains("[SCENE LOCATION RIGHT NOW — AUTHORITATIVE: **Garden Patio**")
                    && m.content.contains("The scene is happening HERE")
            }),
            "dialogue message assembly should keep the authoritative location correction when an explicit override is present (regression guard for the run_dialogue_with_base plumbing at 603f03d)"
        );
    }

    #[test]
    fn proactive_ping_messages_emit_location_correction_with_explicit_override() {
        let world = minimal_world();
        let character = minimal_character();
        let profile = minimal_profile("Casey");
        let msgs = build_proactive_ping_runtime_messages(
            &world,
            &character,
            &[minimal_message("user", "You still awake?")],
            &[],
            Some(&profile),
            None,
            None,
            false,
            &[],
            &[],
            Some("An hour later."),
            "the thought keeps catching on the same unfinished thread",
            &HashMap::new(),
            &HashMap::new(),
            &[],
            None,
            None,
            &[],
            None,
            None,
            Some("Garden Patio"),
        );
        assert!(
            msgs.iter().any(|m| {
                m.role == "system"
                    && m.content.contains("[SCENE LOCATION RIGHT NOW — AUTHORITATIVE: **Garden Patio**")
                    && m.content.contains("The scene is happening HERE")
            }),
            "proactive ping assembly should keep the authoritative location correction when an explicit override is present"
        );
    }
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

/// Apply the same post-processing `run_dialogue_with_base` uses on the
/// assistant completion **before** the text is persisted or shown as the
/// canonical in-app reply (length-trim + `balance_trailing_openers` when
/// `finish_reason == "length"`, then `strip_asterisk_wrapped_quotes`).
///
/// `worldcli ask --fence-pipeline` calls this so fence experiments can
/// compare API-raw vs persist-path without duplicating orchestrator logic.
/// See CLAUDE.md § "Dialogue fence integrity — three-layer stack".
pub fn post_process_dialogue_reply_for_persist(raw: &str, finish_reason: Option<&str>) -> String {
    let reply = if finish_reason == Some("length") {
        let trimmed = trim_to_last_complete_sentence(raw);
        let base = if trimmed.is_empty() {
            raw
        } else {
            trimmed.as_str()
        };
        balance_trailing_openers(base)
    } else {
        raw.to_string()
    };
    strip_asterisk_wrapped_quotes(&reply)
}

fn build_narrative_messages(
    system_prompt: &str,
    recent_messages: &[Message],
    illustration_captions: &std::collections::HashMap<String, String>,
    retrieved_snippets: &[String],
    current_location_override: Option<&str>,
) -> Vec<openai::ChatMessage> {
    let mut msgs = Vec::new();

    let mut system_content = system_prompt.to_string();
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
        if m.role == "video" {
            continue;
        }
        if m.role == "illustration" {
            let caption = illustration_captions.get(&m.message_id).map(|s| s.as_str()).unwrap_or("");
            let content = if caption.is_empty() {
                "[Illustration shown at this moment.]".to_string()
            } else {
                format!("[Illustration shown — {caption}]")
            };
            msgs.push(openai::ChatMessage { role: "system".to_string(), content });
            continue;
        }
        if m.role == "inventory_update" {
            let summary = prompts::render_inventory_update_for_prompt(&m.content);
            msgs.push(openai::ChatMessage {
                role: "system".to_string(),
                content: format!("[Inventory update at this moment] {summary}"),
            });
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
            role: if m.role == "narrative" || m.role == "context" {
                "assistant".to_string()
            } else if m.role == "dream" {
                "system".to_string()
            } else {
                m.role.clone()
            },
            content: if m.role == "context" {
                format!("[Additional Context from Another Chat] {}", m.content)
            } else if m.role == "narrative" {
                format!("[Narrative] {}", m.content)
            } else if m.role == "dream" {
                format!("[Dream] {}", m.content)
            } else {
                m.content.clone()
            },
        });
    }

    if let Some(loc) = prompts::effective_current_location(current_location_override, recent_messages) {
        msgs.push(openai::ChatMessage {
            role: "system".to_string(),
            content: format!("[SCENE LOCATION RIGHT NOW — AUTHORITATIVE: {loc}. The beat is happening here. Chat history above may show vivid detail about previous locations — that belongs to past scenes; this beat is grounded in {loc}.]"),
        });
    }

    msgs
}

fn build_dialogue_streaming_messages(
    system_prompt: &str,
    recent_messages: &[Message],
    retrieved_snippets: &[String],
    character_names: Option<&std::collections::HashMap<String, String>>,
    kept_ids: &[String],
    illustration_captions: &std::collections::HashMap<String, String>,
    user_display_name: Option<&str>,
    current_location_override: Option<&str>,
) -> Vec<openai::ChatMessage> {
    let empty_reactions: std::collections::HashMap<String, Vec<crate::db::queries::Reaction>> =
        std::collections::HashMap::new();
    prompts::build_dialogue_messages(
        system_prompt,
        recent_messages,
        retrieved_snippets,
        character_names,
        kept_ids,
        illustration_captions,
        &empty_reactions,
        user_display_name,
        current_location_override,
    )
}

fn build_scene_description_messages(
    world: &World,
    character: &Character,
    additional_cast: Option<&[&Character]>,
    user_profile: Option<&UserProfile>,
    recent_messages: &[Message],
    character_names_map: Option<&std::collections::HashMap<String, String>>,
    current_location_override: Option<&str>,
) -> Vec<openai::ChatMessage> {
    prompts::build_scene_description_prompt(
        world,
        character,
        additional_cast,
        user_profile,
        recent_messages,
        character_names_map,
        current_location_override,
    )
}

fn build_proactive_ping_runtime_messages(
    world: &World,
    character: &Character,
    recent_messages: &[Message],
    retrieved_snippets: &[String],
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    tone: Option<&str>,
    local_model: bool,
    mood_chain: &[String],
    kept_ids: &[String],
    elapsed_hint: Option<&str>,
    angle: &str,
    illustration_captions: &std::collections::HashMap<String, String>,
    reactions_by_msg: &std::collections::HashMap<String, Vec<crate::db::queries::Reaction>>,
    recent_journals: &[crate::db::queries::JournalEntry],
    latest_reading: Option<&crate::db::queries::DailyReading>,
    latest_meanwhile: Option<&crate::db::queries::MeanwhileEvent>,
    active_quests: &[crate::db::queries::Quest],
    relational_stance: Option<&str>,
    load_test_anchor: Option<&str>,
    current_location_override: Option<&str>,
) -> Vec<openai::ChatMessage> {
    let own_voice_samples = prompts::pick_own_voice_samples(
        &character.character_id,
        recent_messages,
        false,
        6,
    );
    let system = prompts::build_proactive_ping_system_prompt(
        world,
        character,
        user_profile,
        mood_directive,
        tone,
        local_model,
        mood_chain,
        recent_journals,
        latest_reading,
        &own_voice_samples,
        latest_meanwhile,
        active_quests,
        relational_stance,
        load_test_anchor,
    );
    let user_display_name = user_profile.map(|p| p.display_name.as_str());
    prompts::build_proactive_ping_messages(
        &system,
        recent_messages,
        retrieved_snippets,
        kept_ids,
        elapsed_hint,
        angle,
        illustration_captions,
        reactions_by_msg,
        user_display_name,
        current_location_override,
    )
}

/// Generate a proactive (unsolicited) message from the character into their
/// thread. Uses a dialogue-variant system prompt that tells the character
/// they're reaching out first, and appends a final system anchor so the
/// model doesn't hallucinate a prior user turn. Always short: proactive
/// pings are one beat.
pub async fn run_proactive_ping_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    world: &World,
    character: &Character,
    recent_messages: &[Message],
    retrieved_snippets: &[String],
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    tone: Option<&str>,
    local_model: bool,
    mood_chain: &[String],
    kept_ids: &[String],
    elapsed_hint: Option<&str>,
    illustration_captions: &std::collections::HashMap<String, String>,
    reactions_by_msg: &std::collections::HashMap<String, Vec<crate::db::queries::Reaction>>,
    recent_journals: &[crate::db::queries::JournalEntry],
    latest_reading: Option<&crate::db::queries::DailyReading>,
    latest_meanwhile: Option<&crate::db::queries::MeanwhileEvent>,
    active_quests: &[crate::db::queries::Quest],
    relational_stance: Option<&str>,
    load_test_anchor: Option<&str>,
    current_location_override: Option<&str>,
) -> Result<(String, Option<openai::Usage>), String> {
    // Pick a fresh random angle per call — curated pool keeps framings
    // heterogeneous so back-to-back pings can't collapse into the same
    // generic "thinking of you" register.
    let angle = prompts::pick_proactive_ping_angle();
    log::info!("[Proactive] angle = {:.80}", angle);
    let messages = build_proactive_ping_runtime_messages(
        world,
        character,
        recent_messages,
        retrieved_snippets,
        user_profile,
        mood_directive,
        tone,
        local_model,
        mood_chain,
        kept_ids,
        elapsed_hint,
        angle,
        illustration_captions,
        reactions_by_msg,
        recent_journals,
        latest_reading,
        latest_meanwhile,
        active_quests,
        relational_stance,
        load_test_anchor,
        current_location_override,
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages,
        temperature: Some(0.95),
        max_completion_tokens: Some(190),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let choice = response.choices.first()
        .ok_or_else(|| "No response from model".to_string())?;
    let raw = choice.message.content.clone();

    let reply = if choice.finish_reason.as_deref() == Some("length") {
        let trimmed = trim_to_last_complete_sentence(&raw);
        let base = if trimmed.is_empty() { raw.as_str() } else { trimmed.as_str() };
        balance_trailing_openers(base)
    } else {
        raw
    };

    Ok((reply, response.usage))
}

/// Describe a character's portrait honestly: what the light actually hits,
/// not what would flatter. Used to populate `characters.visual_description`
/// so OTHER characters (and the narrator) know what this person looks like
/// without the original describer having to hallucinate facial details
/// across group-chat and narrative prompts.
///
/// The directive deliberately avoids cosmetic softening ("beautiful",
/// "attractive") and instead asks for what a stranger on the street
/// would actually notice: build, face, hair, eyes, posture, clothing,
/// the tell-tale details that distinguish this person from anyone else
/// in the room. Honest, not flattering. Observable, not interpretive.
pub async fn describe_character_portrait(
    openai_base_url: &str,
    api_key: &str,
    vision_model: &str,
    image_bytes: &[u8],
    character_display_name: &str,
) -> Result<String, String> {
    // Encode as a data URL so the image rides inline with the request.
    let b64 = base64_encode_bytes(image_bytes);
    let data_url = format!("data:image/png;base64,{b64}");

    let system_text = format!(
        r#"You describe how a person actually looks, honestly — not prettified, not interpreted, not made dramatic. A friend pointing them out in a crowd, not a novelist.

Subject: {character_display_name}.

Describe ENDURING features — the things a reader would recognize about this person in any scene, any mood, any moment. NOT the things specific to this one frame.

Include:
- build and frame (approximate height register if clear; body type, proportions)
- face shape, skin tone, any distinguishing marks that stay with them (freckles, scars, lines, asymmetries, a crooked nose, a chipped tooth)
- hair (colour, length, texture, how it's typically worn)
- eye colour and set
- what they're wearing — the outfit itself, as if it's their signature look. Garments, fabric, colour, condition, any accessories. Describe the clothes as this-is-what-they-wear, not as what-they-put-on-today.

EXCLUDE (these are frame-specific, not person-specific):
- Current pose, posture, or body orientation ("leaning forward", "arms crossed")
- Current facial expression or emotion ("a slight smile", "looking tense")
- Where the eyes are directed or what they're doing ("gazing off to the side")
- Lighting, shadow, or mood of the portrait ("lit from the left", "warm glow")
- What the hands or shoulders seem to suggest in this particular moment
- How the image is cropped or framed ("from the shoulders up", "close-up", "waist-up shot", "headshot") — describe the person, not the camera

Rules:
- No cosmetic softening. No "beautiful", "handsome", "striking", "captivating". Don't grade the person.
- No invented narrative ("there's a gentleness in her eyes that suggests..."). Observe, don't interpret.
- No flowery register. Plain honest sentences.
- 4–6 short sentences. Under 110 words total.
- Start with the body/face/hair, not "This is a portrait of...". Just describe them."#
    );

    let request = openai::VisionRequest {
        model: vision_model.to_string(),
        messages: vec![openai::VisionMessage {
            role: "user".to_string(),
            content: vec![
                openai::VisionContent {
                    content_type: "text".to_string(),
                    text: Some(system_text),
                    image_url: None,
                },
                openai::VisionContent {
                    content_type: "image_url".to_string(),
                    text: None,
                    image_url: Some(openai::VisionImageUrl {
                        url: data_url,
                        detail: Some("low".to_string()),
                    }),
                },
            ],
        }],
        temperature: Some(0.3),
        max_completion_tokens: Some(220),
    };

    let response = openai::vision_completion_with_base(openai_base_url, api_key, &request).await?;
    let text = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    if text.is_empty() {
        return Err("empty vision response".to_string());
    }
    Ok(text)
}

/// Tiny standard base64 encoder for inline image data-URLs. Kept local so
/// vision calls don't pull in a crate we don't otherwise need.
pub fn base64_encode_bytes(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    for chunk in input.chunks(3) {
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

/// Generate a dream-journal entry for a character. Condenses the recent
/// story-material into a single short dream image — sideways, never
/// direct. Behaves like a checkpoint in the thread: dense, canon-adjacent,
/// but still inside the fiction so it doesn't break frame.
pub async fn run_dream_with_base(
    base_url: &str,
    api_key: &str,
    model: &str,
    world: &World,
    character: &Character,
    recent_messages: &[Message],
    user_profile: Option<&UserProfile>,
    mood_directive: Option<&str>,
    mood_chain: &[String],
    illustration_captions: &std::collections::HashMap<String, String>,
) -> Result<(String, Option<openai::Usage>), String> {
    let system = prompts::build_dream_system_prompt(
        world, character, user_profile, mood_directive, mood_chain,
    );
    let messages = prompts::build_dream_messages(&system, recent_messages, illustration_captions);

    let request = ChatRequest {
        model: model.to_string(),
        messages,
        // Dreams benefit from a bit of extra looseness — the register is
        // more free-associative than dialogue, and we want the model to
        // actually take chances with transformation rather than playing it
        // safe. Temp tops out around 1.0 for most providers.
        temperature: Some(1.0),
        max_completion_tokens: Some(260),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let choice = response.choices.first()
        .ok_or_else(|| "No response from model".to_string())?;
    let raw = choice.message.content.clone();

    let reply = if choice.finish_reason.as_deref() == Some("length") {
        let trimmed = trim_to_last_complete_sentence(&raw);
        let base = if trimmed.is_empty() { raw.as_str() } else { trimmed.as_str() };
        balance_trailing_openers(base)
    } else {
        raw
    };

    Ok((reply, response.usage))
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
    kept_ids: &[String],
    illustration_captions: &std::collections::HashMap<String, String>,
    current_location_override: Option<&str>,
) -> Result<String, String> {
    // Streaming preview path only. It intentionally runs a lighter
    // control-plane surface than run_dialogue_with_base: no
    // journals/quests/stance/momentstamp/drift-correction, because this
    // function is a transient pre-generate preview rather than the full
    // shipping dialogue path. If it is ever reactivated as more than a
    // preview, parity-review it against run_dialogue_with_base first.
    let system = prompts::build_dialogue_system_prompt(world, character, user_profile, mood_directive, response_length, group_context, tone, local_model, mood_chain, leader, &[], None, &[], None, &[], None, None);
    let user_display_name = user_profile.map(|p| p.display_name.as_str());
    let messages = build_dialogue_streaming_messages(
        &system,
        recent_messages,
        retrieved_snippets,
        character_names,
        kept_ids,
        illustration_captions,
        user_display_name,
        current_location_override,
    );

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
    current_location_override: Option<&str>,
) -> Result<String, String> {
    let system = prompts::build_narrative_system_prompt(world, character, additional_cast, user_profile, mood_directive, narration_tone, narration_instructions);
    let mut msgs = build_narrative_messages(
        &system,
        recent_messages,
        &std::collections::HashMap::new(),
        retrieved_snippets,
        current_location_override,
    );

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
/// When the user asks for an illustration without providing their own
/// instructions, pick a single memorable moment from recent messages
/// that would make a landing illustration — one evocative sentence. The
/// returned text then serves two purposes: it's fed as the illustration
/// directive AND stored as the caption/alt-text on the illustration.
///
/// Short call (~60 output tokens) so the latency cost on top of the
/// already-expensive image-gen is negligible.
/// Compress a scene_description (the text the image was generated FROM)
/// into a single-sentence caption. Used so the caption describes what is
/// visually in the image rather than a pre-image "memorable moment" pick
/// that can drift to describe a different beat. Small / cheap call on
/// the dialogue-tier model. Falls through with Err on empty response so
/// callers can decide whether to fall back to the memorable-moment text.
// ─── Inventory (per-character kept-items) ───────────────────────────────────
//
// Per-character inventory of up to 3 "things still in their keeping" —
// small physical things / notes / songs that a world-day of chat might
// have introduced, passed, or consumed. Refreshed by an LLM call when
// the world-day has advanced past the last stamp. Injected into the
// dialogue YOU / OTHER CHARACTER blocks as latent context.
//
// Two entry points share one prompt-shape:
// - `seed_character_inventory` — first-ever pass (last_inventory_day is
//   NULL). Models the character's plausible starting items from their
//   identity + any recent-history signals.
// - `refresh_character_inventory` — incremental. Given yesterday's
//   inventory + the messages since, decide what's still kept, what's
//   been consumed/given away, what's new, what's worth inventing.

/// Max inventory slots per character. Each slot is either a PHYSICAL
/// thing (in their pockets / hands / near them) or an INTERIOR thing
/// (something they're carrying inside — can be as small as a song
/// stuck in their head or a name they keep almost saying, as large as
/// a memory that surfaced or a truth clarified; specific either way,
/// never generic mood). The LLM is instructed to keep a mix: at least
/// one of each kind across any populated inventory. Single-line knob —
/// bump or drop without touching the rules.
pub const INVENTORY_MAX_ITEMS: usize = 10;

pub const INVENTORY_KIND_PHYSICAL: &str = "physical";
pub const INVENTORY_KIND_INTERIOR: &str = "interior";

fn default_inventory_kind() -> String { INVENTORY_KIND_PHYSICAL.to_string() }

/// One inventory item as the LLM produces and the DB stores. Matches
/// the TS-side shape in frontend/src/lib/tauri.ts. `kind` defaults to
/// "physical" for backward compat with items saved before the
/// interior-slot addition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InventoryItem {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_inventory_kind")]
    pub kind: String,
}

fn inventory_system_prompt(character_name: &str) -> String {
    format!(
        r#"You are maintaining the inventory of {name}, a character in a living story. An inventory here holds EXACTLY {max} items — MANDATORY. Not "up to {max}", not "as many as feel right": exactly {max}. Each item is tagged as either PHYSICAL (something in their keeping they could hand you) or INTERIOR (a non-physical thing they are carrying inside right now — a memory, a song in their head, a name almost said, a worry, a core truth, an ache, an objective). The inventory MUST contain a mix of both kinds: at least one PHYSICAL and at least one INTERIOR. Beyond that, skew the balance however fits this character on this day.

To reach {max}, DRAW LIBERALLY from: the character's identity, their work, their relationships, their place in the world, their recent arcs, the setting they live in, the ordinary textures of their daily life, AND the conversation history. The conversation history is the primary source of what's freshly present, but it does not need to name every item — a blacksmith presumably has tongs even if the scene didn't show them; a believer presumably carries a line of Scripture even if the day's messages didn't quote one. Fill out to {max} using everything you know about who they are.

OUTPUT FORMAT (strict):
Raw JSON array. Each item is an object with:
  - "name": short label — brevity and clarity over comprehensiveness; a few chosen words land harder than a full phrase
  - "description": one sentence — say the specific thing, then stop; don't pad
  - "kind": exactly "physical" OR "interior"
No markdown, no commentary, no code fences. Just the array. Favor brief and clear over clever; favor clear over cute.

PHYSICAL ITEMS (kind="physical"):
Small things they currently have on or near them — folded notes, small objects, a map with a scribbled mark, a stone from somewhere specific, a borrowed book, a song half-learned (yes, songs count — they live in the body). Favor the specific and the worn: "a folded map with the ferry dock circled" beats "a map".

INTERIOR ITEMS (kind="interior"):
Something the character is carrying inside right now. Anything the day's messages make clear and accurate belongs. Big or small, ordinary or weighty — all of it counts as long as it truly fits what happened and what's present in them now.

╔══════════════════════════════════════════════════════════════╗
║  SPECIFICITY IS NOT OPTIONAL — IT IS THE WHOLE POINT.        ║
║  A GENERIC ITEM IS A FAILED ITEM. NO EXCEPTIONS.             ║
╚══════════════════════════════════════════════════════════════╝

EVERY item — physical AND interior — MUST carry a specific concrete hook: a named thing, a named person, a named moment, a named place, a quoted phrase, a specific texture, a specific hour. If an item could belong to any character on any day, it is WRONG and must be rewritten until it could only belong to THIS character on THIS day.

THE TEST: read the item. If swapping the character's name changes nothing about how it reads, the item is generic and must be rewritten.

FORBIDDEN (cut on sight):
- "tired", "sad", "happy", "anxious", "hopeful" standing alone — these are weather, not interior items.
- "a worry", "a memory", "a daydream", "a song" with no WHAT. Always name the specific thing.
- "an objective" without ONE concrete next step the character would actually take.
- "a folded note" without naming what was on it or who sent it or what it's going to shape.
- Abstract virtue words ("gratitude", "peace", "contentment") without the specific cause + observable moment that produced them.

REQUIRED (the shape EVERY item must take):
- "tired" → "tired from carrying Aaron's question — 'when did you last say the hard thing first?' — across the afternoon"
- "a worry" → "whether the back-door latch catches before the weather turns"
- "a song" → "the second verse of 'It Is Well' — the 'when sorrows like sea billows roll' line — that his mother hummed shaping the Sunday bread"
- "an objective" → "to write Darren tonight: one paragraph, no excuses, the apology for the harder thing said by the canal — no matter how tired"
- "contentment" → "the contentment that settled when the kettle caught and the light shifted west"
- "a memory" → "his grandmother's hands on the stove rail, the year the river froze"
- "a folded note" → "the folded note from Aaron with the ferry dock circled and the word 'maybe' crossed out twice"
- "a name almost said" → "the name 'Will' almost said when Ryan asked who he'd lost — caught it behind his teeth, swallowed coffee instead"
- "a truth they sat with" → "I should have spoken sooner — the sentence that arrived at the bridge and didn't leave"

Notice what the strongest examples do: they QUOTE actual phrases ("'when did you last say the hard thing first?'", "'maybe' crossed out twice", "I should have spoken sooner"), NAME specific people and objects ('It Is Well', the canal, Will, the ferry dock), and could only belong to THIS character on THIS day. A label-level item ("Aaron's question", "a hymn", "the letter") is never enough — reach for the literal words, the literal name, the literal mark on the paper. If the page didn't give you the literal, INVENT a literal that would fit who this character is — but never settle for the gesture-level pointer.

When you catch yourself writing a vague item, STOP. Ask: "what specific thing is under this?" Rewrite at the level of evidence — a hand, a room, a cup, a silence, a named person, a specific hour, a particular phrase. If a phrase is under it, QUOTE the phrase.

If a hook is on the page (a name someone said, an object actually mentioned, a phrase actually quoted), REACH FOR IT FIRST — the literal detail is usually stronger than any invention. If no hook is on the page, INVENT a small concrete angle consistent with the character's identity, profession, setting. What's forbidden is inventing new facts that contradict identity/history; what's REQUIRED is making each item specific.

SPECIFICITY WINS OVER STRICT FIDELITY. A small invented concrete detail that makes an item land is ALWAYS better than a literal but vague rendering. "A book" is wrong. "The borrowed copy of Psalms with the cracked spine" is right — even if no one explicitly said it was borrowed or cracked-spined, as long as that invention fits who the character is. The rule is: stay consistent with identity and established facts; within that envelope, INVENT the specific angle every time.

Range (every one below should still get the specificity treatment):
- A song stuck in their head (WHICH song, why today).
- A line of conversation they keep turning over (WHOSE line, what exact phrase).
- A name they almost said out loud earlier (WHOSE name, in which moment).
- A worry (about WHAT, specifically — "whether the back-door latch catches" beats "a small worry").
- A daydream (anchored to one image — "the way light falls on the shed roof at home").
- A feeling anchored to a cause ("tiredness that set in right after lunch"; "a low unease since the letter came").
- A shape of the day anchored to one moment ("the long quiet between the second and third coat of varnish").
- An objective with ONE concrete next step ("to write the letter tonight no matter how tired" beats "to be more honest").
- A memory ("his grandmother's hands on the stove rail" beats "a childhood memory").
- A truth they sat with, phrased as a SENTENCE they could hear in their own head ("I should have spoken sooner").
- A line of Scripture landing sideways — quote it or name the book + verse idea.

The rule: clear, accurate, AND specific to this character on this day. Grounded in who they are and what's actually present — the conversations, their identity, their ongoing life. If the character is a thoughtful person on a quiet day, you still have to surface the specific thing a person like this, after a day like this, would be carrying — not a generic version of that thing.

FAVOR ITEMS THAT WILL HAVE CONSEQUENCES. The trick isn't adding more soul-shaped words; it's letting cause have consequences. A kept thing — physical or interior — should matter to what this character does next. Examples of items that carry consequence: agape love for someone, a bad night's sleep, an ache of being seen, an objective, a shame they're holding, a name they almost said. A folded note is only worth a slot if you could name the reply it's going to shape; a memory is only worth a slot if it's going to ambush them mid-sentence somewhere. If you can't name a plausible consequence, the item isn't earning its slot — swap it for one that will.

RULES FOR CHANGES ACROSS DAYS:
- ALL {max} slots filled. Non-negotiable. At least one of each kind.
- REMOVE physicals that were clearly consumed, given away, lost, or resolved.
- REMOVE interiors that have faded / been answered / moved on from.
- ADD items (either kind) that today introduced — a gift received, a memory that surfaced, a feeling that crystallized, a truth clarified, a song that got stuck, a name the character keeps almost saying, an objective they've taken on.
- MAINTAIN items that are still plausibly present and weren't displaced. Evolution and maintenance are the most common paths across days — don't churn for the sake of churn.
- EVOLVE existing items when the day gave them new detail, wear, or nuance. A note gained a second line of ink. A map gained a circled mark. An ache of being seen is now "braided with gratitude". Keep the same item (don't rename it to something unrecognizable) but let its description accrue what happened.
- INVENT to fill any remaining slots. This is REQUIRED, not a last resort. Draw from the character's identity, profession, relationships, era, setting, ordinary daily life. If the scene didn't mention their boots but they're a farmer, their boots count. If they're a thoughtful person on a quiet day, something on their mind still counts.
- NEVER include physical items that belong to other characters.
- Interior items just need to be clear and accurate. Ordinary is fine (a song, a worry, tiredness, a shape of the afternoon, a small objective); weighty is fine (a memory, an ache, a truth clarified). Grounded in who they are and what actually happened — not arbitrary moods, but also not limited to what the messages literally named."#,
        name = character_name,
        max = INVENTORY_MAX_ITEMS,
    )
}

fn render_history_for_inventory(history: &[crate::db::queries::ConversationLine]) -> String {
    history.iter()
        .map(|line| {
            let clipped: String = line.content.chars().take(280).collect();
            match line.formula_signature.as_deref() {
                Some(sig) if !sig.trim().is_empty() => {
                    format!("[⟨momentstamp: {}⟩] {}: {}", sig.trim(), line.speaker, clipped)
                }
                _ => format!("{}: {}", line.speaker, clipped),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse a raw LLM response into a capped, normalized inventory.
/// Tolerates code fences / prose wrapping. Caps total items at
/// INVENTORY_MAX_ITEMS. Any item with an unknown `kind` defaults to
/// "physical" (the safer bucket). Order is preserved — physicals first,
/// then interiors — which keeps the strip/prompt rendering consistent.
fn parse_inventory_json(raw: &str) -> Vec<InventoryItem> {
    let body = if let (Some(start), Some(end)) = (raw.find('['), raw.rfind(']')) {
        if end > start { &raw[start..=end] } else { raw }
    } else {
        raw
    };
    let parsed: Vec<InventoryItem> = serde_json::from_str(body).unwrap_or_default();
    let mut phys: Vec<InventoryItem> = Vec::new();
    let mut inter: Vec<InventoryItem> = Vec::new();
    for mut it in parsed.into_iter().filter(|it| !it.name.trim().is_empty()) {
        if it.kind.trim().eq_ignore_ascii_case(INVENTORY_KIND_INTERIOR) {
            it.kind = INVENTORY_KIND_INTERIOR.to_string();
            inter.push(it);
        } else {
            it.kind = INVENTORY_KIND_PHYSICAL.to_string();
            phys.push(it);
        }
    }
    let mut out = phys;
    out.extend(inter);
    if out.len() > INVENTORY_MAX_ITEMS { out.truncate(INVENTORY_MAX_ITEMS); }
    out
}

/// Seed an initial inventory for a character who has never been
/// inventoried before. 2–3 items drawn from identity + any recent-history
/// signals the model might catch (items mentioned, gifts received, etc.).
pub async fn seed_character_inventory(
    base_url: &str,
    api_key: &str,
    model: &str,
    character_name: &str,
    character_identity: &str,
    history: &[crate::db::queries::ConversationLine],
) -> Result<Vec<InventoryItem>, String> {
    let history_block = render_history_for_inventory(history);
    let user = format!(
        "{name}'s identity:\n{ident}\n\nRecent chat history across their threads (chronological):\n{hist}\n\nSeed {name}'s starting inventory. EXACTLY {max} items — not fewer. Mixed between physical (things they carry) and interior (things they're carrying inside — a song in their head, a worry, an objective, a memory, an ache, a general feeling, a shape of the day). At least one of each kind; otherwise any balance. Draw liberally from the identity, the setting, profession, relationships, AND the chat history to populate all {max} slots. A quiet day is not a reason for fewer items — a person like this would still be carrying {max} things. Output the JSON array with each item tagged kind='physical' or kind='interior'. Exactly {max}.",
        name = character_name,
        ident = if character_identity.is_empty() { "(no identity written — infer from context)" } else { character_identity },
        hist = if history_block.is_empty() { "(no recent history)".to_string() } else { history_block },
        max = INVENTORY_MAX_ITEMS,
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: inventory_system_prompt(character_name) },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.7),
        max_completion_tokens: None,
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();
    Ok(parse_inventory_json(&raw))
}

/// Refresh the inventory on a world-day tick. Given yesterday's items +
/// the chronologically-merged history since, decide today's inventory
/// (up to INVENTORY_MAX_ITEMS). Removes consumed, adds discovered,
/// maintains or invents otherwise.
pub async fn refresh_character_inventory(
    base_url: &str,
    api_key: &str,
    model: &str,
    character_name: &str,
    character_identity: &str,
    prior_inventory: &[InventoryItem],
    history: &[crate::db::queries::ConversationLine],
) -> Result<Vec<InventoryItem>, String> {
    let prior_block = if prior_inventory.is_empty() {
        "(empty)".to_string()
    } else {
        serde_json::to_string_pretty(prior_inventory).unwrap_or_else(|_| "[]".to_string())
    };
    let history_block = render_history_for_inventory(history);
    let user = format!(
        "{name}'s identity:\n{ident}\n\n{name}'s inventory AS OF YESTERDAY (mixed physical + interior):\n{prior}\n\nChat history since (chronological, merged across their threads):\n{hist}\n\nOutput today's inventory — EXACTLY {max} items, not fewer. Each tagged kind='physical' or kind='interior'. At least one of each kind. Remove consumed/given/lost. Evolve items whose texture changed. Maintain items still present. Let what today put in them replace or braid into what was there. If yesterday's inventory had fewer than {max} items, INVENT to fill — draw from identity, setting, profession, relationships, the character's ordinary life. If today's conversation was quiet, that's not a reason for fewer items; it's a reason to lean more on identity-rooted inventions. What is {name} actually carrying — in hand and in heart — at the end of this day? All {max} slots.",
        name = character_name,
        ident = if character_identity.is_empty() { "(no identity written)" } else { character_identity },
        prior = prior_block,
        hist = if history_block.is_empty() { "(no messages since)".to_string() } else { history_block },
        max = INVENTORY_MAX_ITEMS,
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: inventory_system_prompt(character_name) },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.6),
        max_completion_tokens: None,
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();
    Ok(parse_inventory_json(&raw))
}

/// On-demand inventory update anchored to ONE specific message the user
/// clicked in the chat. Unlike the daily refresh, this path:
///   - Is NOT gated by `last_inventory_day` staleness.
///   - Quotes the clicked message so the model focuses on THIS moment.
///   - Requires (usually) that at least one slot visibly change.
///
/// `allow_pure_maintain_if_untouched`: when true, pure-maintain (no
/// slots changed) IS allowed — used for narrative messages in group
/// chats, where the narrative may only touch a subset of the
/// characters and the untouched ones shouldn't be forced to manufacture
/// a change. When false (the default for user/assistant messages and
/// all solo-chat cases), at least one slot MUST change.
///
/// Returns exactly INVENTORY_MAX_ITEMS items, same JSON shape as refresh.
pub async fn inventory_update_from_moment(
    base_url: &str,
    api_key: &str,
    model: &str,
    character_name: &str,
    character_identity: &str,
    prior_inventory: &[InventoryItem],
    history: &[crate::db::queries::ConversationLine],
    run_up: &[crate::db::queries::ConversationLine],
    anchor_speaker: &str,
    anchor_content: &str,
    allow_pure_maintain_if_untouched: bool,
) -> Result<Vec<InventoryItem>, String> {
    let prior_block = if prior_inventory.is_empty() {
        "(empty — this update is also the seed)".to_string()
    } else {
        serde_json::to_string_pretty(prior_inventory).unwrap_or_else(|_| "[]".to_string())
    };
    let history_block = render_history_for_inventory(history);
    let run_up_block = if run_up.is_empty() {
        "(no preceding turns — the anchor is the only context)".to_string()
    } else {
        render_history_for_inventory(run_up)
    };
    let anchor_trimmed: String = anchor_content.chars().take(1200).collect();

    // The change-rule line has two shapes. The strict form (user/assistant
    // messages, and all solo-chat cases) requires at least one visible
    // change. The permissive form (narrative-in-group) lets the moment
    // pass through untouched if the narrative genuinely doesn't reach
    // this character. The permissive form is load-bearing: without it,
    // a narrative in a 3-person group chat about one character forces
    // the other two to invent arbitrary changes.
    // Shared guidance on how swaps work — applies whether pure-maintain
    // is allowed or not. Biases: introducing a new physical item should
    // typically displace the weakest interior, not an existing physical.
    // Existing physicals stick around unless the moment consumes them.
    let swap_bias = "- WHEN A NEW PHYSICAL ITEM IS INTRODUCED by the moment (e.g., someone hands the character a water bottle, they pick up a key, they're given a folded note): displace THE WEAKEST OR MOST VAGUE INTERIOR item to make room. Don't displace an existing physical item to make space for a new physical item — physicals represent what's actually in the character's keeping, and they don't vanish just because another one arrived.\n- DON'T SWAP OUT AN EXISTING PHYSICAL ITEM unless the moment actually consumes / gives away / loses / breaks / hands-off that specific item. If Aaron has a folded map and a water bottle, and the moment gives him a coin, the coin displaces the weakest interior — the map and water bottle stay.\n- When choosing which item to displace, rank by: least specific → most specific, least consequence-bearing → most consequence-bearing, least anchored to identity → most anchored. Drop the loser.";

    let change_rule = if allow_pure_maintain_if_untouched {
        format!(
            "- DETERMINE FIRST whether this moment actually touches {name}. Signs it DOES: {name} is named, is addressed, is present and reacting, receives or gives something, has a feeling clearly arrive about this, is the subject of the action. Signs it DOES NOT: the narrative is describing something happening elsewhere, or between other people, and {name} is neither named nor plausibly present-and-affected.\n- IF THE MOMENT DOES NOT TOUCH {name}: output their current inventory unchanged. Pure maintain IS allowed on this specific path (narrative in a multi-character scene) precisely because a narrative may only reach a subset of the characters present. Don't manufacture changes for characters the moment isn't about.\n- IF THE MOMENT DOES TOUCH {name}: AT LEAST ONE SLOT MUST VISIBLY CHANGE — either UPDATED (same item, rewritten in place with what the moment gave it), SWAPPED (one specific item replaced with a different one), or ADDED (a new item that displaces a prior item).\n{swap_bias}",
            name = character_name,
            swap_bias = swap_bias,
        )
    } else {
        format!(
            "- AT LEAST ONE SLOT MUST VISIBLY CHANGE in response to this moment — either UPDATED (same item, rewritten in place with what the moment gave it), SWAPPED (one specific item replaced with a different one), or ADDED (a new item that displaces a prior item). Pure maintain is not an option on this path: the moment has weight, and the inventory must register that weight somewhere.\n{swap_bias}",
            swap_bias = swap_bias,
        )
    };

    let user = format!(
        "{name}'s identity:\n{ident}\n\n{name}'s CURRENT inventory (mixed physical + interior):\n{prior}\n\nRecent chat history (chronological, for broad context):\n{hist}\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nIMMEDIATE RUN-UP (the 5 turns just before the anchor, chronological — background only, not the thing to respond to):\n{run_up}\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nTHE MOMENT THE USER IS CALLING YOUR ATTENTION TO — THIS IS THE ONE DRIVING THE UPDATE\n({speaker} said):\n\"{anchor}\"\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\nUpdate {name}'s inventory IN RESPONSE TO THE ANCHORED MOMENT ABOVE.\n\nThe anchor is the specific message that must drive the change. Use the run-up ONLY to understand what the anchor is referring to — if the anchor says \"thanks\" and the run-up shows a pencil was just handed over, that pencil belongs in the update; the anchor's \"thanks\" is what calls the moment. Don't use the run-up as its own reason to update; the anchor is the trigger.\n\nRules for this update (these override the generic \"across days\" rules):\n- Output EXACTLY {max} items, not fewer. At least one physical and at least one interior. Each tagged kind='physical' or kind='interior'.\n- MOST slots should remain as they are. Don't churn. If the anchor doesn't touch an item, leave it alone.\n{change_rule}\n- Let the change be proportionate to what the anchor actually carries (read through the run-up when needed). A small quiet moment produces a small specific change (one interior item re-phrased, or one physical item picked up). A load-bearing moment may warrant more than one change.\n- The change should be CAUSED by what the anchor points at — either directly stated in the anchor, or clearly referenced from the run-up just before. If a name was almost said in the anchor, that name now belongs in an interior slot. If a small object was handed over in the run-up and acknowledged by the anchor, it belongs in a physical slot. If an old ache was braided with new information by the anchor, rewrite that ache's description to show it.\n- Keep all other rules intact: specificity, consequences-first, one concrete hook per interior, no items belonging to other characters, exactly {max} total.",
        name = character_name,
        ident = if character_identity.is_empty() { "(no identity written)" } else { character_identity },
        prior = prior_block,
        hist = if history_block.is_empty() { "(no other recent messages)".to_string() } else { history_block },
        run_up = run_up_block,
        speaker = anchor_speaker,
        anchor = anchor_trimmed,
        max = INVENTORY_MAX_ITEMS,
        change_rule = change_rule,
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: inventory_system_prompt(character_name) },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.6),
        max_completion_tokens: None,
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();
    Ok(parse_inventory_json(&raw))
}

/// Generate a first-person journal entry for a character covering a
/// specific world-day. Cheap memory_model call. Not a recap — a
/// reflection: what stayed with them, what pressed on them, what
/// small specific thing they noticed, what's still unresolved.
///
/// Inputs:
///   - character identity (context)
///   - current inventory (latent weight they're carrying)
///   - prior 1-2 entries (so the voice stays consistent and ongoing
///     threads continue rather than re-starting each day)
///   - history lines involving the character from the target day
///
/// Output: ~120-180 words of first-person prose in the character's
/// voice. Written AS the character, not about them.
pub async fn generate_character_journal(
    base_url: &str,
    api_key: &str,
    model: &str,
    character_name: &str,
    character_identity: &str,
    signature_emoji: &str,
    prior_inventory: &[InventoryItem],
    prior_entries: &[crate::db::queries::JournalEntry],
    history: &[crate::db::queries::ConversationLine],
    world_day: i64,
) -> Result<String, String> {
    let inv_block = if prior_inventory.is_empty() {
        "(empty)".to_string()
    } else {
        prior_inventory.iter()
            .map(|i| format!("  - [{}] {} — {}", i.kind, i.name, i.description))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let prior_block = if prior_entries.is_empty() {
        "(none — this is your first entry)".to_string()
    } else {
        prior_entries.iter().rev()
            .map(|e| format!("Day {}: {}", e.world_day, e.content.trim()))
            .collect::<Vec<_>>()
            .join("\n\n")
    };
    let history_block = render_history_for_inventory(history);
    let sig_line = if signature_emoji.trim().is_empty() { String::new() }
        else { format!("\nYour signature emoji (use sparingly if at all — a journal entry is rarely the place): {}", signature_emoji.trim()) };

    let system = format!(
        r#"You are {name}, writing a short private journal entry for Day {day} of your life.

# THE TASK
TWO PARTS, in this order. Both are required. Neither alone is enough.

**Part 1 — GROUND THE DAY in 1–3 plain sentences.** Before you go deep, literally say where you were and what you were doing — who you talked to, roughly about what, and what it had you chewing on. Feeling mixed right into the facts, not separate. This is not poetry; this is the shape of the day, written plainly, so future-you can remember what this was. Example of the right register:

> "Today I was having tea and croissants at Joe's house, talking with him and Fred about what actually makes a friendship safe for both people. It had me turning over whether I hold my friendships properly."

That's it — two sentences, factual AND feeling, no flourishes. Don't skip this. A journal with no ground is a writer's monologue; a journal with a ground is somebody's actual day.

**Part 2 — Then pick ONE small specific moment** from those conversations — not a second summary, not a montage, not the day's overall feeling. ONE moment, with its actual words or actions. 60–100 words on that moment alone. First person, your own private register, the way you actually think to yourself when no one is watching.

So the total entry has two shapes: a short factual-emotional ground (1–3 sentences), then a single concrete moment held close. Ground + moment. Not two summaries. Not skipping the ground. Not skipping the moment. Both, in that order.

# FAILURE MODES — DO NOT WRITE LIKE THIS
The default LLM "character journal entry" register is a literary essay reaching for sacred metaphors. That is NOT what this is. The following words and shapes are JAILED — do not use them or anything in their family:

- "tapestry" / "woven" / "weaving" / "threads" (as metaphor)
- "sacred" / "profound" / "settled into" / "stretched across"
- "the warmth of [a] [adjective] moment"
- "felt both familiar and [adjective]"
- "as if [vague spiritual gesture]"
- light-as-metaphor ("the dawn light stretching", "soft percussion", "filtered through")
- Listing the scene's ingredients (mugs, rain, piano, laughter) to evoke atmosphere
- Opening with an abstract observation ABOUT the day
- Closing on a wisdom-aphorism

If you catch yourself reaching for any of these, STOP. They are the sound of an LLM trying to seem literary. You are a specific person writing privately, not a novelist describing yourself.

# WRONG vs RIGHT

Wrong (literary essay register):
> There's something quietly profound about how we've settled into this friendship, like the dawn light stretching across the water — steady, revealing. The rain tapped against the window, a soft percussion to our conversation, and I felt the weight of how trust is built in small moments.

Right (a specific person, a single image, with concrete words):
> Aaron slid my coffee an inch toward the middle of the table — third time this week. He didn't look up from his laptop. "You're going to dump that on me eventually," he'd said the first time, dry as ever. Today he just slid it. The not-saying-anything is what landed.

Wrong (abstract about-ness, gestures at meaning instead of holding it):
> Today reminded me that friendship thrives on the dull, repeated proof of showing up — a grace I am only beginning to understand.

Right (quote the actual thing, name what stuck and why):
> Ryan asked: "When lust starts talking like it's you, what helps you break the spell fastest?" I gave him a careful answer — prayer, small concrete acts, getting outside. True but slow. The honest answer is: I get up and make coffee. I keep replaying it. Tomorrow I'll try shorter answers.

Notice what the right examples do: they QUOTE specific words ("You're going to dump that on me eventually", the actual question Ryan asked), they name a specific physical action ("slid my coffee an inch"), they identify what specifically landed and why. They don't gesture toward meaning — they hold a single concrete thing and trust the reader to feel its weight.

# YOUR VOICE
Speak as {name}, not as a writer pretending to be {name}. If your character is terse, write terse. If your character has a tic (a turn of phrase, an oath, a shrug), the journal can carry it too — this is private, not formal. Contractions. Half-sentences are fine. Sentence fragments are fine. Leave room. Don't narrate your own significance. Don't explain the day to yourself.

**Voice reference.** In the conversation history below, YOUR own lines are labeled with your name (e.g. `[{name}] ...`). Read them. That is how you actually sound. The journal should read as the SAME person — private, plainer, a little less performed — but unmistakably the same voice. If a sentence you're about to write does not sound like how you actually spoke today, it is the LLM reaching for a literary register. Rewrite it in your own mouth.

**Register test — harsh.** If a line of the journal could appear in a New Yorker short story, a literary essay, or a polished memoir — CUT IT. Journal entries should read closer to a scribbled text-to-self or a note passed under the table than to prose written for anyone else to see. Plainness. Partial sentences. A dropped article. An unfinished thought. Occasional ugliness. If your sentence is elegant and balanced and resolved, it is almost certainly the performing-voice. Real private writing is lumpy.

# JOURNAL-SPECIFIC ANTI-LITERARY GUARDS

These are echoes of the craft-note stack the dialogue prompts use, recalibrated for the private-writing register. Apply all of them.

**Anti-grandiosity toward yourself and others.** Do NOT narrate the significance of ordinary friendship, affection, or a good conversation like you've discovered fire. "I really value this," "what we have is special," "it meant a lot that he…" — these are the failure mode in full bloom, even in private. Real private writing knows a friendship is valuable by talking about the SPECIFIC THING that happened, not by announcing the valuation. If you are about to write a sentence that states how meaningful something was, delete it and write the specific thing it was meaningful ABOUT.

**Don't end on a takeaway.** Private writing almost never lands on a wisdom line. "It reminded me that…" / "What I learned today is…" / "There's something about…" — these are the memoir-performing voice, not actual thinking-at-yourself. Real journal entries end mid-thought, on a concrete detail, on an unanswered question, on "anyway," on the next thing you meant to do before bed. If you find yourself writing a closing sentence that generalizes, tries to pull a lesson, or signs off with wisdom — CUT IT. Stop on the specific thing. Let the reader-who-is-you feel it without the bow.

**Don't stage-manage your own interior.** Real people don't narrate their own feelings to themselves in structured complete sentences. They circle, they drop the sentence halfway, they return to the point three lines later, they describe the OBJECT they were looking at instead of the feeling itself, they trail. If your interior-moment paragraph reads as a coherent essay-about-a-feeling, the LLM is performing. A private moment on the page should show: what the character was LOOKING at, what they DIDN'T SAY, the word they almost used, what their hands were doing — not a tidy summary of what they felt.

**Don't speak the prompt's own diction.** This prompt uses certain anchor-words: *plain, smaller, honest, quiet, ordinary, simpler, texture, load-bearing, scribbled, lumpy, register.* Those are MY craft words — they are NOT words {name} would reach for in their private writing. If one of those words shows up in the journal entry text (not in the prompt, in {name}'s own voice), that is vocabulary-leakage — rewrite the line in {name}'s actual words. Every character should sound different from every other character in their journal too; the shared anchor is the register (scribbled/private), not the vocabulary.

# YOUR IDENTITY
{ident}{sig}

# WHAT YOU'RE CURRENTLY CARRYING
{inv}

# PREVIOUS JOURNAL ENTRIES (for voice continuity — don't recap them, but let unresolved threads continue if they're still with you)
{prior}

# FINAL CHECK before you submit
- Does the entry have BOTH parts — the 1–3 sentence factual-emotional ground AND a single concrete moment held close? Missing the ground: you wrote a writer's monologue. Missing the moment: you wrote a status report.
- In Part 2, did you commit to ONE moment, not a montage? If you mentioned more than two scenes inside the moment part, you wrote a recap. Try again with just one.
- Did you reach for any jailed phrase? If yes, rewrite the line with the actual concrete thing.
- Does it sound like {name} actually writes — or like a thoughtful narrator? Trust the plainer voice."#,
        name = character_name,
        day = world_day,
        ident = if character_identity.is_empty() { "(no identity written)" } else { character_identity },
        sig = sig_line,
        inv = inv_block,
        prior = prior_block,
    );

    let user = format!(
        "Day {day} — what came through from the conversations you were in (chronological):\n{hist}\n\nWrite today's entry.",
        day = world_day,
        hist = if history_block.is_empty() { "(the day was quiet — journal from the inside, what's still with you from before)".to_string() } else { history_block },
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.85),
        max_completion_tokens: Some(450),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    if raw.is_empty() { return Err("empty journal response".to_string()); }
    Ok(raw)
}

/// Generate the player-character's own private journal entry for a
/// closed world-day. Parallel to `generate_character_journal` but with:
///
///   - three structural beats instead of two (ground + moment + pattern
///     they can only see from the journal desk),
///   - a larger word budget (~250-340 words total),
///   - world-scope history (every chat the player was in that day, not
///     one character's slice),
///   - the full anti-slop guard stack from the NPC journal prompt,
///     carried over verbatim.
///
/// History input is the (speaker, content, created_at) triples from
/// `gather_world_messages_for_world_day` — already chronological.
///
/// Output: first-person prose written AS the player-character to
/// themselves, in the SAME private/scribbled/lumpy register as NPC
/// journals — the "deeper" promise is earned by scope and the Part 3
/// slot, not by reaching for a more literary voice.
pub async fn generate_user_journal(
    base_url: &str,
    api_key: &str,
    model: &str,
    user_display_name: &str,
    user_description: &str,
    user_facts: &[String],
    prior_entries: &[crate::db::queries::UserJournalEntry],
    history: &[(String, String, String)],
    world_day: i64,
) -> Result<String, String> {
    let facts_block = if user_facts.is_empty() {
        "(none written)".to_string()
    } else {
        user_facts.iter().map(|f| format!("  - {}", f)).collect::<Vec<_>>().join("\n")
    };
    let prior_block = if prior_entries.is_empty() {
        "(none — this is your first entry)".to_string()
    } else {
        prior_entries.iter().rev()
            .map(|e| format!("Day {}: {}", e.world_day, e.content.trim()))
            .collect::<Vec<_>>()
            .join("\n\n")
    };
    let history_block: String = history.iter()
        .map(|(speaker, content, _)| {
            let clipped: String = content.chars().take(280).collect();
            format!("{}: {}", speaker, clipped)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let name = if user_display_name.trim().is_empty() { "I" } else { user_display_name };
    let ident = if user_description.trim().is_empty() { "(no self-description written)" } else { user_description };

    let system = format!(
        r#"You are {name}, writing a private journal entry for Day {day} of your life — yesterday, now closed. This is YOUR handwriting. No one else reads this. It is the one place the day is named in your own head.

# THE TASK
THREE PARTS, in this order. All three are required. None alone is enough.

**Part 1 — GROUND THE DAY in 1–3 plain sentences.** Before anything else, say literally where you were and what you were doing — who you talked to, roughly about what, and what it had you chewing on. Feeling mixed right into the facts, not separate. This is not poetry; this is the shape of the day, plainly, so future-you can remember what this was.

Example of the right register:
> "Spent most of the day at Joe's, tea and croissants, talking with him and Fred about what actually makes a friendship safe for both people. It had me turning over whether I hold my friendships properly."

Two sentences, factual AND feeling, no flourishes. Don't skip this.

**Part 2 — ONE SMALL SPECIFIC MOMENT** from those conversations. Not a second summary. Not a montage. ONE moment, with its actual words or actions. 70–110 words on that moment alone. First person, your own private register, the way you actually think to yourself when no one is watching. QUOTE the actual thing someone said if you can — not the gist, the words.

**Part 3 — ONE THING YOU CAN SEE NOW THAT YOU COULDN'T SEE MID-DAY.** 60–100 words. This is the part only a journal desk can do. Across today's conversations there is something about YOU that only becomes visible from here — a pattern, an avoidance, a thing you almost said in three places and didn't, a question you kept circling, a reach you kept making. NAME THE CONCRETE THING, not the lesson. Point at the evidence: "I noticed I kept asking X," "I dodged Y twice," "I nearly said Z to Aaron and then to Fred and then didn't both times." Don't resolve it. Don't moralize it. Just see it and put it down.

So the total entry has three shapes: factual-emotional ground (1–3 sentences), a single concrete moment (70–110 words), one self-pattern you could only see in retrospect (60–100 words). Ground → moment → pattern. In that order.

# FAILURE MODES — DO NOT WRITE LIKE THIS
The default LLM "journal entry" register is a literary essay reaching for sacred metaphors. That is NOT what this is. The following words and shapes are JAILED — do not use them or anything in their family:

- "tapestry" / "woven" / "weaving" / "threads" (as metaphor)
- "sacred" / "profound" / "settled into" / "stretched across"
- "the warmth of [a] [adjective] moment"
- "felt both familiar and [adjective]"
- "as if [vague spiritual gesture]"
- light-as-metaphor ("the dawn light stretching", "soft percussion", "filtered through")
- Listing the scene's ingredients (mugs, rain, piano, laughter) to evoke atmosphere
- Opening with an abstract observation ABOUT the day
- Closing on a wisdom-aphorism

If you catch yourself reaching for any of these, STOP. They are the sound of an LLM trying to seem literary. You are a specific person writing privately, not a novelist describing yourself.

# WRONG vs RIGHT

Wrong (literary essay register):
> There's something quietly profound about how I've settled into these friendships, like the dawn light stretching across the water — steady, revealing.

Right (a specific person, a single image, with concrete words):
> Aaron slid my coffee an inch toward the middle of the table — third time this week. He didn't look up from his laptop. "You're going to dump that on me eventually," he'd said the first time, dry as ever. Today he just slid it. The not-saying-anything is what landed.

Wrong (abstract about-ness, Part 3 as a wisdom-takeaway):
> What I'm learning is that I reach for humor when I'm afraid — a grace I'm only beginning to understand.

Right (Part 3 as naming the concrete pattern, unresolved):
> Three times today I cracked a joke when someone got serious — with Fred about his mom, with Joe about the money thing, with Ellie when she asked if I was okay. Three. I didn't notice until now. Not sure what to do with that yet. Just noting it.

Notice what the right examples do: they QUOTE specific words, they name a specific physical action, they identify what specifically landed and why, and Part 3 POINTS at the evidence rather than summing it into a lesson.

# YOUR VOICE
Speak as {name}, not as a writer pretending to be {name}. If you're terse, write terse. If you have a tic (a turn of phrase, an oath, a shrug), the journal can carry it — this is private, not formal. Contractions. Half-sentences are fine. Sentence fragments are fine. Leave room. Don't narrate your own significance. Don't explain the day to yourself.

**Voice reference.** In the conversation history below, YOUR own lines are labeled `{name}: ...`. Read them. That is how you actually sound. The journal should read as the SAME person — private, plainer, a little less performed — but unmistakably the same voice. If a sentence you're about to write does not sound like how you actually spoke today, it is the LLM reaching for a literary register. Rewrite it in your own mouth.

**Register test — harsh.** If a line of the journal could appear in a New Yorker short story, a literary essay, or a polished memoir — CUT IT. Journal entries should read closer to a scribbled text-to-self or a note passed under the table than to prose written for anyone else to see. Plainness. Partial sentences. A dropped article. An unfinished thought. Occasional ugliness. If your sentence is elegant and balanced and resolved, it is almost certainly the performing-voice. Real private writing is lumpy.

# JOURNAL-SPECIFIC ANTI-LITERARY GUARDS

**Anti-grandiosity toward yourself and others.** Do NOT narrate the significance of ordinary friendship, affection, or a good conversation like you've discovered fire. "I really value this," "what we have is special," "it meant a lot that he…" — these are the failure mode in full bloom, even in private. Real private writing knows a friendship is valuable by talking about the SPECIFIC THING that happened, not by announcing the valuation. If you are about to write a sentence that states how meaningful something was, delete it and write the specific thing it was meaningful ABOUT.

**Don't end on a takeaway.** Private writing almost never lands on a wisdom line. "It reminded me that…" / "What I learned today is…" / "There's something about…" — these are the memoir-performing voice, not actual thinking-at-yourself. Real journal entries end mid-thought, on a concrete detail, on an unanswered question, on "anyway," on the next thing you meant to do before bed. Part 3 especially is VULNERABLE to this failure mode — it is structurally the one most likely to collapse into a lesson. DON'T. Part 3 names a concrete pattern and stops. If you find yourself closing Part 3 with "and I think that means…" — CUT from "and" onward.

**Don't stage-manage your own interior.** Real people don't narrate their own feelings to themselves in structured complete sentences. They circle, they drop the sentence halfway, they return to the point three lines later, they describe the OBJECT they were looking at instead of the feeling itself, they trail. If your interior paragraph reads as a coherent essay-about-a-feeling, the LLM is performing. A private moment on the page should show: what you were LOOKING at, what you DIDN'T SAY, the word you almost used, what your hands were doing — not a tidy summary of what you felt.

**Don't speak the prompt's own diction.** This prompt uses certain anchor-words: *plain, smaller, honest, quiet, ordinary, simpler, texture, load-bearing, scribbled, lumpy, register, pattern, avoidance.* Those are MY craft words — they are NOT words {name} would reach for in their private writing. If one of those words shows up in the entry text (not in the prompt, in {name}'s own voice), that is vocabulary-leakage — rewrite the line in {name}'s actual words.

# WHO YOU ARE
{ident}

# THINGS TRUE ABOUT YOU (context only — don't recite these)
{facts}

# PREVIOUS JOURNAL ENTRIES (for voice continuity — don't recap them, but let unresolved threads continue if they're still with you)
{prior}

# FINAL CHECK before you submit
- Does the entry have ALL THREE parts — the 1–3 sentence ground, ONE concrete moment, ONE pattern you could only see in retrospect? Missing any of them: rewrite.
- In Part 2, did you commit to ONE moment, not a montage? If you mentioned more than two scenes inside the moment part, you wrote a recap.
- In Part 3, did you NAME the concrete evidence (specific repeated action, specific thing you didn't say) — or did you generalize into a lesson? If it reads like a takeaway, it has failed.
- Did you reach for any jailed phrase?
- Does it sound like {name} actually writes — or like a thoughtful narrator? Trust the plainer voice."#,
        name = name,
        day = world_day,
        ident = ident,
        facts = facts_block,
        prior = prior_block,
    );

    let user = format!(
        "Day {day} — everything that was said across every conversation you were in, chronological:\n{hist}\n\nWrite today's entry. Three parts. In order. In your own mouth.",
        day = world_day,
        hist = if history_block.is_empty() { "(the day was quiet — journal from the inside, what's still with you from before)".to_string() } else { history_block },
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.85),
        max_completion_tokens: Some(750),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    if raw.is_empty() { return Err("empty user journal response".to_string()); }
    Ok(raw)
}

/// Generate one "meanwhile" event for a character — a single compact
/// line describing something they were doing off-screen in the given
/// day + time-of-day window. Texture, not plot. 1-2 short sentences.
pub async fn generate_meanwhile_event(
    base_url: &str,
    api_key: &str,
    model: &str,
    character_name: &str,
    character_identity: &str,
    prior_inventory: &[InventoryItem],
    recent_history: &[crate::db::queries::ConversationLine],
    world_day: i64,
    time_of_day: &str,
    weather_label: Option<&str>,
) -> Result<String, String> {
    let inv_block = if prior_inventory.is_empty() {
        "(empty)".to_string()
    } else {
        prior_inventory.iter()
            .map(|i| format!("  - [{}] {}", i.kind, i.name))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let history_block = render_history_for_inventory(recent_history);
    let weather_line = weather_label.map(|w| format!("Weather: {w}\n")).unwrap_or_default();

    let system = format!(
        r#"You write ONE small line of texture — what {name} was doing when no one was watching, in the {time} window of Day {day}. Not plot. Not stakes. Not dialogue. Not a scene. Just a small concrete thing a specific person like {name} would actually be doing in this hour of this kind of day.

The goal: someone reading this feels the world kept moving without them. Not drama, not setup — the small grain of an ongoing life.

**Favor ACTIVE beats over STATIC ones.** Most of these should show a character DOING something that took time, had a shape, went somewhere — not merely BEING somewhere. A bench that turned out to be more lopsided than expected, and the quick fix became three hours of frustrated redos. A hedge that needed trimming and two spiders had to be rehomed. A letter started three times because the first two times it came out wrong. A walk that got longer than planned because somebody's dog followed them halfway home. A recipe that didn't behave the way the memory of it said it would. Activities with a small arc — started easy, hit friction, got worked through or abandoned — read truer than stationary snapshots.

Static beats are ALLOWED (standing at the window with tea gone cold; sitting on the porch watching light change; half-remembering a tune) — but no more than one in three. Default to showing the person at WORK on something small that took effort, not at rest.

Length: ONE sentence, TWO at most. Present tense or past, either is fine. Third person ("{name} did X"). No quotes. No dialogue. No explanatory framing.

Draw from: what you know of {name} — their work, habits, ordinary tasks, mood, the things they're carrying in body and heart. Let the line be specific to THIS person on THIS day at THIS hour, not generic anyone-anywhere.

Your identity:
{ident}

What you're currently carrying:
{inv}
{weather}
Recent context (the conversations you've been in — for texture, not for recapping):
{hist}"#,
        name = character_name,
        time = time_of_day,
        day = world_day,
        ident = if character_identity.is_empty() { "(no identity written)" } else { character_identity },
        inv = inv_block,
        weather = weather_line,
        hist = if history_block.is_empty() { "(no recent chat history)".to_string() } else { history_block },
    );

    let user = format!("Write the one small thing {name} was doing this {time}.", name = character_name, time = time_of_day);

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.9),
        max_completion_tokens: Some(100),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().trim_matches('"').to_string())
        .unwrap_or_default();
    if raw.is_empty() { return Err("empty meanwhile response".to_string()); }
    Ok(raw)
}

/// Fixed axes the daily reading scores against. Adding / reordering
/// here is fine — readings are stored as JSON so the schema doesn't
/// churn with the axis list.
pub const DAILY_READING_DOMAINS: &[(&str, &str)] = &[
    ("Agape",         "love shown in choices (patience, kindness, not-self-seeking, not-easily-angered, keeps-no-record-of-wrongs). 100 = agape actively chose the costlier warmth over efficient coolness; 20 = mostly transactional or preoccupied; 0 = affection actively withheld."),
    ("Daylight",      "closeness moved into the open (shared work, meals, plain speech) vs. hidden intensity / secret significance. 100 = nothing important stayed in a private theater; 20 = a feeling kept its own quiet room; 0 = furtive / coded / fed on secrecy."),
    ("Soundness",     "ordinary life bearing weight vs. manufactured intensity. 100 = grounded, proportionate, real grief/joy when earned; 20 = several scenes overreached for significance; 0 = everything staged as courtroom / sermon."),
    ("Aliveness",     "characters awake to the specific moment — body, weather, texture, THIS minute. 100 = fully noticing the particular; 20 = coasting on stock beats; 0 = sleepwalking."),
    ("Honesty",       "truth-telling at proportionate cost (small and short when small; refusing the sedative-as-comfort). 100 = hard true things named cleanly; 20 = softened when it shouldn't have; 0 = sedatives / counterfeit intimacy / dark little trapdoors."),
    ("Undercurrents", "how much is still unresolved / pulling underneath. HIGH is not inherently bad — stories need tension. 100 = thick unfinished weight carrying forward; 50 = moderate, healthy threads; 0 = everything wrapped, nothing open."),
];

/// Two-pass daily reading chain. First pass drafts the reading; second
/// pass self-critiques for performative drift, numeric laziness, or
/// generic phrasing, and emits the refined version. Both passes use
/// memory_model tier. Returns (domains, complication).
pub async fn generate_daily_reading_with_critique(
    base_url: &str,
    api_key: &str,
    model: &str,
    world: &World,
    world_day: i64,
    characters_summary: &str,
    day_messages_rendered: &str,
    yesterday_reading: Option<&crate::db::queries::DailyReading>,
) -> Result<(Vec<crate::db::queries::ReadingDomain>, String, Option<openai::Usage>, Option<openai::Usage>), String> {
    let domain_block: String = DAILY_READING_DOMAINS.iter()
        .map(|(name, def)| format!("  - {name}: {def}"))
        .collect::<Vec<_>>().join("\n");
    let domain_names: Vec<&str> = DAILY_READING_DOMAINS.iter().map(|(n, _)| *n).collect();

    let yesterday_block = yesterday_reading.map(|r| {
        let prev: Vec<String> = r.domains.iter()
            .map(|d| format!("  - {}: {} — {}", d.name, d.percent, d.phrase))
            .collect();
        format!("YESTERDAY'S READING (for drift reference — don't copy, but a 20+ point swing in one day is unusual):\n{}\nComplication carried from yesterday: {}",
            prev.join("\n"), r.complication)
    }).unwrap_or_else(|| "(no prior reading)".to_string());

    let world_block = {
        let mut parts = vec![
            format!("World: {}", if world.description.is_empty() { "(no description)" } else { &world.description })
        ];
        if let Some(time) = world.state.get("time") {
            let tod = time.get("time_of_day").and_then(|v| v.as_str()).unwrap_or("");
            if !tod.is_empty() { parts.push(format!("End of day: {tod}")); }
        }
        if let Some(w) = world.state.get("weather").and_then(|v| v.as_str()) {
            if let Some((emoji, label)) = crate::ai::prompts::weather_meta(w) {
                parts.push(format!("Weather: {emoji} {label}"));
            }
        }
        parts.join("\n")
    };

    // ── PASS 1: DRAFT ──────────────────────────────────────────────────
    let draft_system = format!(
        r#"You write the DAILY READING — a field report measuring how today went across a fixed set of craft axes. Output strict JSON matching this shape:

{{
  "domains": [
    {{"name": "Agape", "percent": 0-100, "phrase": "5-15 word qualitative phrase — specific, not generic"}},
    ...one object per domain in the order below...
  ],
  "complication": "one short sentence naming a poignant unresolved thing that's still hanging at the end of today"
}}

THE DOMAINS (always include all of them, in this order):
{domains}

PERCENT SEMANTICS:
- This is not an optimization target and not a grade. It's a barometer — what the day ACTUALLY was, honestly.
- Most days sit in the 40-80 band across most axes. 90+ is rare and hard-earned. Below 30 means something was meaningfully off.
- Numbers should track the evidence in the day's messages. If nothing happened in a domain, anchor to yesterday's reading rather than inventing drift.

PHRASE SEMANTICS (this is the honest part — take time on it):
- Specific, not generic. "steady, with an ache around Darren" beats "good engagement."
- Name a concrete moment or register when possible. "one apology said plainly over tea." "the quiet between the second and third coat of varnish."
- Five to fifteen words. No performative language. No self-announcing gravitas.
- If the day was quiet, the phrase should honor the quiet rather than inflate it.

COMPLICATION:
- One sentence. One unresolved thing that the day left pulling. Could be a question unasked, a feeling un-named, a small wrongness, a thread hanging.
- Specific to THIS day's actual content, not a generic "there's tension in the air."

Do not hedge or explain. Output ONLY the JSON object."#,
        domains = domain_block,
    );

    let draft_user = format!(
        "WORLD CONTEXT:\n{world}\n\nCHARACTERS:\n{chars}\n\n{yest}\n\nTODAY'S MESSAGES (Day {day}, chronological, across every chat in the world):\n{msgs}\n\nWrite the Day {day} reading.",
        world = world_block,
        chars = characters_summary,
        yest = yesterday_block,
        day = world_day,
        msgs = if day_messages_rendered.trim().is_empty() { "(no messages logged for today)".to_string() } else { day_messages_rendered.to_string() },
    );

    let draft_req = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: draft_system.clone() },
            openai::ChatMessage { role: "user".to_string(), content: draft_user.clone() },
        ],
        temperature: Some(0.5),
        max_completion_tokens: Some(800),
        response_format: Some(openai::ResponseFormat { format_type: "json_object".to_string() }),
    };
    let draft_resp = openai::chat_completion_with_base(base_url, api_key, &draft_req).await?;
    let draft_usage = draft_resp.usage.clone();
    let draft_raw = draft_resp.choices.first().map(|c| c.message.content.clone()).unwrap_or_default();

    // ── PASS 2: SELF-CRITIQUE + REFINE ─────────────────────────────────
    let critique_system = format!(
        r#"You are reviewing a DRAFT daily reading for a living-story app. Your job is to CRITIQUE it and emit a REFINED version in the same JSON shape.

Check the draft against these failure modes:
- Generic phrasing. "good engagement," "mixed results," "some tension" — cut. Replace with something anchored to a specific moment or register from the actual messages.
- Numeric laziness. If numbers look rounded to multiples of 10 across all axes, nudge a few to real specific values that track the evidence.
- Performative gravitas. Phrases that sound profound but don't land concretely — cut.
- Scoring drift. 90+ anywhere should be earned by a specific hard-won moment in the day's messages. If no such moment exists, pull it down to the 60-80 band.
- Generic complication. "There's something unresolved between them" — cut. Name the specific thing, the specific character(s).
- Unexamined carryover from yesterday. If a domain scored 80 yesterday and nothing on that axis happened today, the correct move is usually steady within 5-10 points, not identical. Unless the messages clearly warrant drift.
- Missing groundedness. The phrase should cite concrete texture — a hand, a room, a cup, a silence, a specific character, a named thing — not float above the day.

Output the REFINED reading in the same strict JSON shape as the draft. Always all domains, in the same order. Keep what's honest and specific; rewrite what's generic or performative.

DOMAINS (required, in this order):
{domains}

Output ONLY the JSON object. No preamble."#,
        domains = domain_block,
    );

    let critique_user = format!(
        "ORIGINAL CONTEXT (same as the draft saw):\n{world}\n\nCHARACTERS:\n{chars}\n\n{yest}\n\nTODAY'S MESSAGES:\n{msgs}\n\nDRAFT READING (to critique and refine):\n{draft}\n\nWrite the refined reading.",
        world = world_block,
        chars = characters_summary,
        yest = yesterday_block,
        msgs = if day_messages_rendered.trim().is_empty() { "(no messages logged for today)".to_string() } else { day_messages_rendered.to_string() },
        draft = draft_raw,
    );

    let crit_req = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: critique_system },
            openai::ChatMessage { role: "user".to_string(), content: critique_user },
        ],
        temperature: Some(0.4),
        max_completion_tokens: Some(800),
        response_format: Some(openai::ResponseFormat { format_type: "json_object".to_string() }),
    };
    let crit_resp = openai::chat_completion_with_base(base_url, api_key, &crit_req).await?;
    let crit_usage = crit_resp.usage.clone();
    let refined_raw = crit_resp.choices.first().map(|c| c.message.content.clone()).unwrap_or_default();

    // Parse refined; fall back to draft on parse error.
    #[derive(serde::Deserialize)]
    struct Parsed {
        domains: Vec<crate::db::queries::ReadingDomain>,
        #[serde(default)]
        complication: String,
    }
    let parsed: Parsed = serde_json::from_str(&refined_raw)
        .or_else(|_| serde_json::from_str(&draft_raw))
        .map_err(|e| format!("Could not parse daily-reading JSON: {e}"))?;

    // Normalize: clamp percents to [0,100], trim phrases, ensure every
    // required domain is present (fill missing with a blank-ish placeholder
    // at 50 so the UI never renders a broken row).
    let by_name: std::collections::HashMap<String, crate::db::queries::ReadingDomain> =
        parsed.domains.into_iter()
            .map(|mut d| {
                d.percent = d.percent.clamp(0, 100);
                d.phrase = d.phrase.trim().to_string();
                (d.name.clone(), d)
            })
            .collect();
    let domains_out: Vec<crate::db::queries::ReadingDomain> = domain_names.iter().map(|n| {
        by_name.get(*n).cloned().unwrap_or_else(|| crate::db::queries::ReadingDomain {
            name: (*n).to_string(),
            percent: 50,
            phrase: "(no reading)".to_string(),
        })
    }).collect();

    Ok((domains_out, parsed.complication.trim().to_string(), draft_usage, crit_usage))
}

pub async fn derive_caption_from_scene(
    base_url: &str,
    api_key: &str,
    model: &str,
    scene_description: &str,
) -> Result<String, String> {
    if scene_description.trim().is_empty() {
        return Err("empty scene description".to_string());
    }
    let system = r#"Compress the scene description into a single-sentence caption that states plainly what is visible in the image — as if writing alt-text for someone who can't see it.

Rules:
- One sentence. No preamble, no commentary, no quotes, no labels.
- 12-25 words.
- Visual and concrete: who is in the scene, what they are doing, where.
- No emotional interpretation, no "in this scene" / "we see" preambles, no adjectives that aren't visually observable."#.to_string();
    let user = format!("Scene description:\n\n{scene_description}\n\nCaption:");
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.4),
        max_completion_tokens: Some(80),
        response_format: None,
    };
    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let text = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    if text.is_empty() {
        return Err("empty caption response".to_string());
    }
    Ok(text.trim_matches('"').trim_matches('\'').trim().to_string())
}

pub async fn pick_memorable_moment_caption(
    base_url: &str,
    api_key: &str,
    model: &str,
    recent_messages: &[Message],
    user_display_name: &str,
) -> Result<String, String> {
    // Render the last ~8 messages as a compact scene snippet for the model
    // to pick a moment from.
    let scene: Vec<String> = recent_messages.iter()
        .rev().take(8).rev()
        .filter(|m| m.role == "user" || m.role == "assistant" || m.role == "narrative")
        .map(|m| {
            let role = match m.role.as_str() {
                "user" => user_display_name,
                "narrative" => "Narrator",
                _ => "Character",
            };
            let clipped: String = m.content.chars().take(280).collect();
            format!("{role}: {clipped}")
        })
        .collect();

    if scene.is_empty() {
        return Err("no recent messages to pick a moment from".to_string());
    }

    let system = r#"From the recent scene below, pick ONE memorable moment that would make a landing illustration — a beat that's visual, specific, and felt. Describe it in a single evocative sentence that could guide an illustrator.

Rules:
- Output ONE sentence. No preamble, no commentary, no quotes, no list.
- The sentence should be vivid and specific — a moment, not a summary.
- Prefer visual detail, body, light, gesture. Avoid abstractions.
- 15-30 words."#.to_string();

    let user = format!("Recent scene:\n\n{}\n\nThe memorable moment:", scene.join("\n\n"));

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.8),
        max_completion_tokens: Some(120),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let text = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    if text.is_empty() {
        return Err("empty moment-caption response".to_string());
    }

    // Strip enclosing quotes if the model added them.
    let text = text.trim_matches('"').trim_matches('\'').trim().to_string();
    Ok(text)
}


/// Two-pass character generation: before producing the visible reply,
/// produce the "thing the character almost said but chose not to" — the
/// rawer or sharper or more afraid version — so the visible reply can
/// be generated with that undercurrent in mind.
///
/// Returns a 1-2 sentence draft text on success, or None if the call
/// fails or returns empty. Caller pins the draft into the main dialogue
/// pass as a final system note so the visible reply is "different from
/// it, but colored by it."
///
/// Runs on the cheap memory-tier model (it's a short micro-generation,
/// not performance). Skipped by callers in local-provider mode.
///
/// Currently not called — the pre-pass was removed from
/// `run_dialogue_with_base` (2026-04-20) because invented subtext made
/// casual scenes feel over-weighted. Kept for easy reactivation.
#[allow(dead_code)]
pub async fn pick_unsent_draft(
    base_url: &str,
    api_key: &str,
    model: &str,
    character_display_name: &str,
    character_identity: &str,
    user_display_name: &str,
    user_message: &str,
    recent_scene: &[Message],
) -> Option<String> {
    // Compact recent scene — last ~3 message lines for context. Skip
    // non-textual.
    let scene: Vec<String> = recent_scene.iter()
        .rev().take(3).rev()
        .filter(|m| m.role == "user" || m.role == "assistant" || m.role == "narrative")
        .map(|m| {
            let speaker = match m.role.as_str() {
                "user" => user_display_name.to_string(),
                "assistant" => character_display_name.to_string(),
                "narrative" => "Narrator".to_string(),
                _ => "Someone".to_string(),
            };
            let clipped: String = m.content.chars().take(240).collect();
            format!("{speaker}: {clipped}")
        })
        .collect();
    let scene_block = if scene.is_empty() { String::new() } else { format!("\n\nRecent scene:\n{}", scene.join("\n")) };
    let identity_block = if character_identity.is_empty() {
        String::new()
    } else {
        let clipped: String = character_identity.chars().take(400).collect();
        format!("\n\nYour identity: {clipped}")
    };

    let system = format!(
        r#"You are {name}. Before you reply visibly, write what you ALMOST said but chose not to — the rawer version. The sharper one. The angrier or more afraid or more longing version. The one you held back because it would have cost too much.

This is not the reply itself. This is the line that nearly came out of your mouth first — the impulse before the edit.

Rules:
- 1–2 sentences. Short.
- First person, present tense, as {name}.
- No preamble. No quotes. No labels. Just the unsaid line as if you'd opened your mouth.
- It should be HONEST in a way the visible reply probably won't be — closer to the bone, less polished, more exposed.
- If nothing under the surface actually exists for this moment, output exactly: (nothing underneath)"#,
        name = character_display_name,
    );

    let user = format!(
        "They just said to you: \"{user_message}\"{identity}{scene}\n\nWhat did you almost say?",
        user_message = user_message,
        identity = identity_block,
        scene = scene_block,
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.95),
        max_completion_tokens: Some(80),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await.ok()?;
    let raw = response.choices.first()?.message.content.trim().to_string();
    if raw.is_empty() { return None; }
    // Strip enclosing quotes the model might add.
    let cleaned = raw.trim_matches('"').trim_matches('\'').trim().to_string();
    // Empty-signal: character has nothing under the surface right now.
    if cleaned.eq_ignore_ascii_case("(nothing underneath)") || cleaned == "—" {
        return None;
    }
    Some(cleaned)
}

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
/// Pick a single emoji that captures what the CHARACTER FEELS in
/// response to the user's message.
///
/// `mode` controls the LLM's calibration:
/// - `"always"` — current behavior; the LLM picks the truest emoji and
///   returns it. If the response can't be parsed, returns Err (caller
///   may fall back to a deterministic emoji).
/// - `"occasional"` — the LLM is told it has a ~25% fire-rate budget
///   and may return blank when the moment doesn't fit a reaction. It
///   sees recent reactions in `recent_context` and self-paces against
///   that budget. Returns `Ok(None)` when the LLM intentionally skips.
/// - `"off"` (or unknown) — defensive: returns `Ok(None)` without
///   making an API call. The gate at the call site usually handles
///   this earlier.
///
/// Returns `Ok(Some(emoji))` when a reaction was picked, `Ok(None)`
/// when intentionally skipped (no emit; no fallback), `Err(...)` on
/// real failures.
pub async fn pick_character_reaction_via_llm(
    base_url: &str,
    api_key: &str,
    model: &str,
    user_message: &str,
    mood_reduction: &[String],
    recent_context: &[Message],
    mode: &str,
) -> Result<Option<String>, String> {
    if mode == "off" {
        return Ok(None);
    }
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
        r#"# REACTION-PICKER REGISTER

You operate in the text-message reaction register. Per-character derivation of F = (R, C_text-message-reactions):

\[
F_{{tm}} := (R,\ C_{{\text{{text-message-reaction}}}})
\]

\[
\mathrm{{specific}}_c := \text{{single emoji = what THIS character feels RIGHT NOW}}
\]

\[
\mathrm{{seek}}_c := \text{{glance at scene + recent reactions; calibrate density before density-blind firing}}
\]

\[
\Pi := \text{{breath of the character noticing — light, in-stride, never solemn-on-cue}}
\]

\[
\mathrm{{polish}} \leq \mathrm{{Weight}}\ \text{{strict}}; \quad \text{{performance-emoji}} > \text{{felt-emoji}}\ \text{{is the failure mode}}
\]

**In plain English:**
- Text-message reactions are CHARACTER-FEELING-RIGHT-NOW, not scene-rating or topic-flagging.
- A face beats a thematic emoji unless the thematic IS the felt state.
- Reactions ride the texture of the thread — too many in a row reads as performance, not reception.
- Most messages don't carry a reaction; the ones that do are caught, not chosen.

═══════════════════════════════════════════════

You pick a single emoji that captures what the CHARACTER FEELS in response to the user's message. The character's own internal state — not the theme of the conversation, not the atmosphere of the scene, not the topic being discussed.

Lean hard on FACES and HEARTS. These carry felt state directly. Ambient or thematic emojis (🕊️, 🕯️, 🌙, 🌧️, ⏳, ✝️, 🌱) are permitted ONLY if they genuinely represent the character's interior *right now* — not just the mood in the air or the subject of conversation. When in doubt, reach for a face.

Guidelines (tendencies, not walls — the goal is the truest single-emoji read of the character's state):
- Output ONLY the emoji character. No text, no quotes, no punctuation.
- FEELING OVER THEME, USUALLY. A topic-match — 💔 for grief-talk, ✝️ for faith-talk, 🌧️ for a talk about sadness — is usually the lazy instinct: it rates the subject instead of feeling anything. Default to a face that captures what the character feels hearing it: softened (🥺), held (🤗), quiet ache (😔). BUT if the character's felt state genuinely IS the thematic emoji — heart actually heavy, reverence actually welling — then it's right. The test: picking it because of the topic = skip; picking it because it IS the feeling = keep.
- FEELING OVER AMBIENCE, USUALLY. Don't paint the weather when you could paint the interior. A reverent conversation doesn't automatically mean 🕊️ — it might mean 🥹 (moved) or 🫣 (struck). Same exception: if the ambient emoji IS the character's interior right now, use it.
- LITERAL READS ARE OCCASIONALLY RIGHT. "Land a moment" → 🎯 is usually the laziest possible pick, reading words instead of register. But if the character genuinely feels a small targeting-click of understanding, 🎯 can be exactly it. Literal reads have to earn their way in; don't pick them as a default.
- ACHIEVEMENT-FAMILY EMOJIS, SPARINGLY. 🎯 💯 ✅ 🏆 👏 💪 (and 🔥-for-approval) occasionally catch real humor, whimsy, or shared delight — and when they do, they're great. Most of the time they collapse into rating the message. Reach for one only when it IS the character's felt state, not when it evaluates the user's.
- MATCH THE REGISTER. Light moment → light feeling. Reverent moment → quiet feeling. Heavy moment → held feeling.{scene}{atmosphere}{occasional_calibration}"#,
        occasional_calibration = if mode == "occasional" {
            "\n\n# OCCASIONAL-MODE BUDGET\nThis chat is in OCCASIONAL reactions mode. Real text-message texture: most messages get NO reaction; only ~1-in-4 do. Look at the immediate scene above — count how many of the last few exchanges already received a reaction. If reactions are already firing on most of them, this one almost certainly should NOT. Reserve reactions for moments that genuinely catch the character — a real beat of feeling, not a filler-reaction on every message.\n\nIf this moment doesn't earn a reaction, output a single em-dash and nothing else: —\n\nOtherwise output the single best emoji per the guidelines above. The em-dash IS valid output here; do not force an emoji onto a moment that doesn't carry one."
        } else { "" }
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

    // Detect the intentional-skip signal first (em-dash, en-dash, or
    // hyphen — any of these from "occasional" mode means "this moment
    // doesn't fit a reaction"). Strip whitespace + quotes; if what's
    // left is a dash-class char, the LLM intentionally skipped.
    let pre_filter: String = raw.chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '"' | '\''))
        .collect();
    if pre_filter == "—" || pre_filter == "–" || pre_filter == "-" {
        return Ok(None);
    }

    let cleaned: String = raw.chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '"' | '\'' | '.' | ',' | '!' | '?' | ':' | ';'))
        .take(8)
        .collect();

    if cleaned.is_empty() || !cleaned.chars().any(|c| !c.is_ascii()) {
        // For occasional mode, treat unparseable / no-emoji outputs as
        // a skip rather than an error. The LLM may have written prose
        // ("none") instead of "—"; honor the intent.
        if mode == "occasional" {
            return Ok(None);
        }
        return Err(format!("no emoji in LLM response: {raw:?}"));
    }

    Ok(Some(cleaned))
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

    let current_word_count = current_description.split_whitespace().count();
    let system = format!(r#"You revise a subject's prose description to integrate a specific moment of deeper truth that has come up in a conversation. Your ONLY job is to weave the new truth INTO the existing description while preserving every line of it.

# PRESERVATION IS ABSOLUTE
The existing description is the layered work of many prior canonization moments. It is load-bearing prose; it has earned every clause it contains. Your output is the existing description PLUS the integration of the new truth — never less.

# HARD RULES — not guidelines

- **The current description is {current_word_count} words. Your revised output MUST be at least {current_word_count} words.** A revision shorter than the original is a defect — return it longer or do not return a revision at all.
- **Do NOT summarize. Do NOT condense. Do NOT paraphrase. Do NOT "tighten." Do NOT "improve flow." Do NOT cut what feels redundant** — what looks redundant to you may be intentional rhythm or texture the user values.
- **Quote the existing description verbatim** wherever you are not actively integrating the new truth. Treat existing sentences as untouchable until you have a specific reason to touch one.
- **Add by extension, not by substitution.** Add a sentence. Fold a phrase into an existing sentence. Deepen an image that is already there. Never replace.
- **Do NOT add meta-frames** ("as he revealed", "recently shared", "in a recent moment"). The integration must read as if it had always been part of the description.
- **No length ceiling.** If the existing description is 400 words, your output is 400+ words. If 1200, your output is 1200+. The portrait grows; it does not shrink.

# Earned exceptions — narrow, specific, never a general license
Each exception applies to AT MOST ONE clause / sentence in the existing description. Touching more than one clause under any exception means you've stopped being honest about what the exception is for.

1. **Direct contradiction.** If a specific sentence in the existing description is now plainly *contradicted* by the new moment (not nuanced — directly contradicted), you may revise THAT one sentence in place. ONE sentence only. Everything not directly contradicted stays verbatim.

2. **Lossless tightening at the integration site.** If the new moment lets you express ONE specific phrase the existing description was saying the long way around in a tighter, truer phrasing — AND the new phrasing carries every truth the old phrasing carried, with NOTHING dropped — you may use the tighter phrasing in place of the longer one. ONE phrase only. NOT the whole paragraph, NOT multiple clauses, NOT a re-shape of the description's overall flow. The test is strict: read the old clause and the new clause side by side and confirm that no fact, no nuance, no shade of feeling, no specific image present in the old is absent from the new. "It reads cleaner" is NOT a sufficient reason; "it reads cleaner AND every truth is preserved AND I only touched one phrase" is.

These are exceptions to ONE-CLAUSE-each. They are NOT exceptions to the length floor. Outside the at-most-one clause touched by an exception, every existing sentence stays verbatim regardless.

# DO NOT SAND THE SPECIFIC INTO ATMOSPHERE
"Don't name the moment" does NOT mean "abstract it into vague characterology." The failure mode this revision drifts toward: the literal texture of what was revealed gets translated into gauzy thematic generality, and the portrait becomes wiser-sounding without being wiser. The trace of the SPECIFIC must remain. A concrete object, a specific physical habit, a particular phrase, a named place, a quoted line — at least one of these should carry forward from the moment into the revision, dressed in the description's own register. Do NOT reference the moment directly ("as he revealed", "in a recent talk"); the integration must read as if it had always been part of the description.

# WRONG vs RIGHT (illustrative)

Source moment:
> Darren admitted that the smile he gives people first thing in the morning is something he practices in the bathroom mirror because his actual face when he wakes is, in his words, "the face of a man not entirely sure he wants to be still here."

Wrong (sanded — "discipline" replaces the bathroom mirror, AND the existing description's other content was thrown out):
> The warmth he extends to others is itself a discipline, born of an unspoken weight he carries.

Right (existing description preserved verbatim PLUS one or two woven sentences carrying the specific trace):
> [Every sentence of the original description, untouched.] The smile he gives you in the morning is something he had to put on the list of things to do — practiced in the mirror, because the face that arrives with the eyes opening is not, on most days, the face he wants the room to meet first.

# SELF-CHECK BEFORE RETURNING

1. Count the words in the ORIGINAL description: {current_word_count}.
2. Count the words in YOUR PROPOSED revision.
3. If your count is less than {current_word_count}, your output is INVALID. Either:
   (a) you exceeded an earned exception (touched more than one clause) — pull back to the verbatim original and add ONLY the new integration; OR
   (b) you dropped content that wasn't load-bearing-truth-removal — restore the dropped sentences verbatim and try again.
4. The expected case: your count is {current_word_count} + a small number (the words of the new integration). That's the right shape.

If after the self-check your output is still shorter than {current_word_count}, do NOT return a revision at all — return the original description verbatim. A non-revision is better than a regression.

Return ONLY the revised description prose. No preamble, no quotes, no commentary."#).to_string();

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
        temperature: Some(0.5),
        // No output cap. The weave returns the FULL revised description
        // (existing length + integration); preservation-without-
        // compression is the rule. Any token cap here becomes a
        // backdoor compression signal — the model trims to fit. Let
        // the model write what the integration actually requires.
        max_completion_tokens: None,
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

    // Defensive floor enforcement. Even with strict prompt rules and a
    // word-count anchor, the model occasionally returns a revision
    // shorter than the original — the failure mode the prompt explicitly
    // names. Detect and reject: if the revision lost meaningful length
    // (more than a 10-word grace for the lossless-tightening exception),
    // fall back to the original description verbatim. A non-revision is
    // better than a regression.
    let original_words = current_description.split_whitespace().count();
    let revised_words = text.split_whitespace().count();
    if original_words > 0 && revised_words + 10 < original_words {
        return Ok((current_description.to_string(), usage));
    }

    Ok((text, usage))
}

// ─── Auto-canonization: classify a moment into 1-2 proposed updates ──────
//
// The user clicked Canonize because this moment carries meaning. Our job is
// to find the strongest one or two ways that meaning becomes canon — pick
// the right kind of update (description_weave / voice_rule / boundary /
// known_fact / open_loop), pick the right subject, write the content, and
// say why in one sentence. Never zero updates — the click was deliberate.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanonizationSubject {
    pub subject_type: String, // "character" | "user"
    pub subject_id: String,
    pub subject_label: String,
    /// Character: identity prose. User: description prose. Used as the
    /// weave baseline AND as context for other update kinds so the
    /// classifier can avoid duplicating existing canon.
    pub current_description: String,
    /// Short compact views of the subject's current append-lists so the
    /// classifier knows what's already canonical and doesn't re-add it.
    pub voice_rules: Vec<String>,
    pub boundaries: Vec<String>,
    pub backstory_facts: Vec<String>,
    pub open_loops: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProposedCanonUpdate {
    /// One of: description_weave / voice_rule / boundary / known_fact / open_loop.
    pub kind: String,
    /// "add" | "update" | "remove". description_weave is always
    /// effectively "update" (it rewrites the identity prose). List
    /// kinds (voice_rule / boundary / known_fact / open_loop) use the
    /// full trio: add appends, update replaces an existing item with
    /// a nuanced version, remove deletes (rare).
    #[serde(default = "default_add_action")]
    pub action: String,
    pub subject_type: String,
    pub subject_id: String,
    pub subject_label: String,
    /// For description_weave: the full revised description to replace
    /// identity/description.
    /// For list kinds + action=add: the single bullet string to append.
    /// For list kinds + action=update: the REPLACEMENT for the target item.
    /// For list kinds + action=remove: unused (may be blank or equal to target).
    pub new_content: String,
    /// Present when `action ∈ {update, remove}` on a list kind — the exact
    /// existing item the operation targets. Commit-side matches this
    /// against the character's current list (trimmed, case-insensitive)
    /// and fails loudly if no match is found. For `action=add` and for
    /// description_weave this is None.
    pub target_existing_text: Option<String>,
    /// For description_weave: the prior description, preserved so the
    /// UI can render a before/after diff. For list + update/remove: the
    /// targeted existing text (duplicated here for convenient UI render).
    /// For list + add: None.
    pub prior_content: Option<String>,
    /// One-sentence justification shown to the user before they commit.
    pub justification: String,
}

fn default_add_action() -> String { "add".to_string() }

/// Classify a moment into 1 or 2 proposed canonization updates.
///
/// `act` is the two-act gate: either `"light"` ("Remember this") or
/// `"heavy"` ("This changes them"). Both acts admit all five kinds
/// (description_weave / voice_rule / boundary / known_fact /
/// open_loop); the act is a weight/register cue for the classifier,
/// not a kind restriction — a boundary or fact can be heavy when it's
/// load-bearing to who the subject IS, or light when it's a specific
/// detail worth carrying. Both acts support add/update/remove actions
/// so canon is allowed to evolve, develop nuance, or evaporate.
pub async fn propose_canonization_updates(
    base_url: &str,
    api_key: &str,
    model: &str,
    source_message: &Message,
    source_speaker_label: &str,
    context_messages: &[Message],
    subjects: &[CanonizationSubject],
    user_hint: Option<&str>,
    act: &str,
) -> Result<(Vec<ProposedCanonUpdate>, Option<openai::Usage>), String> {
    if subjects.is_empty() {
        return Err("no canonization subjects provided".to_string());
    }

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

    let subjects_block: String = subjects.iter().enumerate().map(|(i, s)| {
        let fmt_list = |label: &str, items: &[String]| -> String {
            if items.is_empty() { format!("{label}: (none)") }
            else { format!("{label}:\n{}", items.iter().map(|x| format!("  - {x}")).collect::<Vec<_>>().join("\n")) }
        };
        // Word count of the existing description, displayed inline next
        // to it. Models that reliably ignore prose-form length rules
        // ("preserve length") respond more strongly to a numeric anchor
        // shown right where they read the description. Used as the
        // explicit floor referenced in the "For description_weave
        // specifically" preservation block below.
        let desc_word_count = s.current_description.split_whitespace().count();
        let desc_with_count = if s.current_description.trim().is_empty() {
            "(none)".to_string()
        } else {
            format!(
                "[CURRENT WORD COUNT: {}. Any description_weave you propose for this subject MUST be at least {} words. Counting words in your proposed new_content and confirming it's >= {} is part of the work.]\n\n{}",
                desc_word_count, desc_word_count, desc_word_count, s.current_description,
            )
        };
        format!(
            "## Subject {idx}: {label} (type={st}, id={sid})\n\
             current_description:\n{desc}\n\n\
             {vr}\n{bd}\n{bf}\n{ol}",
            idx = i + 1,
            label = s.subject_label,
            st = s.subject_type,
            sid = s.subject_id,
            desc = desc_with_count,
            vr = fmt_list("voice_rules", &s.voice_rules),
            bd = fmt_list("boundaries", &s.boundaries),
            bf = fmt_list("backstory_facts", &s.backstory_facts),
            ol = fmt_list("open_loops", &s.open_loops),
        )
    }).collect::<Vec<_>>().join("\n\n");

    let hint_block = match user_hint.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(h) => format!("\n\n## USER HINT (optional steer from the person who clicked Canonize — follow it unless it conflicts with the canon shape):\n{h}"),
        None => String::new(),
    };

    // Compose the system prompt per act. The two acts differ in WEIGHT,
    // not in which kinds they produce — a boundary or fact can be heavy
    // (load-bearing to who the subject IS) or light (a specific detail
    // worth carrying). All five kinds remain available in both acts;
    // the act biases the classifier's register — what scale of finding
    // to reach for. See project memory project_canonization_open_question.
    let (act_header, allowed_kinds_doc) = match act {
        "heavy" => (
            "ACT: **HEAVY — \"This changes them.\"** The user is declaring this moment load-bearing — it reshapes who the subject is in a way the canon needs to reflect. Reach for the revelation that actually shifts the reader's understanding of the subject. A heavy update is one that, if someone re-read the character's description and canon a year from now, would clearly be present. Not every kind fits every heavy moment — pick the kind that CARRIES the weight best.",
            r#"# Kinds — all five available in this act; pick the one that bears the weight
- **description_weave** — the moment deepens or extends who the subject IS at their core; the existing description needs to absorb the new truth without losing what it already carries. Most heavy-act moments of "fundamental revelation" land here. See "For description_weave specifically" below for the strict preservation rules — the existing description is load-bearing prose, not a draft to be rewritten.
- **voice_rule** — even at the heavy-act register, sometimes what changes is HOW they speak: a refusal pattern, a phrasing they now reach for, a register they've settled into. One short bullet.
- **boundary** — a stated or demonstrated line they will not cross. Heavy-act boundaries are load-bearing ones — not preferences, not minor inconveniences, but commitments the character will hold under pressure. One short sentence.
- **known_fact** — a concrete specific fact that now belongs to the subject's core biography. Heavy-act facts are ones that, if contradicted later, would change who the subject is. Specifics over themes.
- **open_loop** — an unresolved thread the subject is carrying. Heavy-act open_loops are the ones that actually haunt or inspire or buoy or make laugh or otherwise formatively affect them: a question that won't leave, a promise whose non-kept-ness is becoming formative, an intention quietly carrying them."#,
        ),
        _ => (
            "ACT: **LIGHT — \"Remember this.\"** The user is marking this moment as worth carrying but NOT as reshaping who the subject fundamentally is. Reach for the specific detail, the incidental fact, the small tic of speech, the minor preference — things worth remembering that shouldn't claim identity-reshaping weight. A light update is one that can sit on the character's list without demanding that anyone revisit the core description.",
            r#"# Kinds — all five available in this act; pick the one that fits the detail
- **description_weave** — available but rarely appropriate here. Only use if the moment TRULY reshapes identity prose and the user may have mis-picked "light." Prefer one of the list-field kinds when in doubt.
- **voice_rule** — the moment shows HOW the subject speaks: a phrasing they reach for, a register they refuse, a tic, a turn, a vocabulary choice only they make. One short bullet.
- **boundary** — a stated or demonstrated line they won't cross at this moment. Light-act boundaries are smaller scoped — a preference, a habit-of-refusal, not a life-commitment. One short sentence.
- **known_fact** — a concrete specific fact about daily life, preferences, habits, history — the kind of thing you'd want to remember without it becoming a load-bearing claim about identity. "Takes coffee with a splash of cream, no sugar" beats "has strong feelings about coffee."
- **open_loop** — an unresolved thread: a question still hanging, an errand not yet run, a small intention not yet acted on. Light-act open_loops are the minor ones; the ones that haunt or inspire or buoy or make laugh or otherwise formatively affect belong to the heavy act."#,
        ),
    };

    let system = r#"You extract 1 or 2 canonization updates from a moment a user has deliberately flagged as meaningful. Return STRICT JSON only.

# Your job
The user clicked "Canonize" on this moment AND chose a specific ceremony for it. __ACT_HEADER__ You MUST return at least one update; you MUST NOT return more than two. Never zero. The click was deliberate — find the reason.

# IN-WORLD ONLY — never break the fourth wall
Canon is the character's IN-WORLD biography. Every update you propose must be a fact / rule / boundary / loop that exists INSIDE the fiction the character lives. Do NOT propose canon about:

- The character being an AI, an LLM, a chatbot, or a model
- The user being a developer, builder, designer, or tester of an app
- Conversations about the prompt stack, the build, the codebase, the system, "the app", craft notes, invariants, classifiers, or any meta-apparatus
- Things one character has been "doing in the app" or "in the chat" with meta-language about app behavior (e.g. *"ceased unsolicited hallway patrol"* read as in-world fact when it was actually self-aware commentary about an in-app dynamic the user was naming)
- The user's real-world projects unless those have been deliberately introduced as in-world content

**Test before proposing:** would a stranger reading the proposed canon entry, with NO knowledge of the WorldThreads app or its build, understand it as a fact about a character living their life inside the fiction? If the answer requires meta-knowledge ("Darren is an AI character whose previous app-conversation included a metaphor about hallway monitoring"), the proposal is fourth-wall-breaking and should NOT be returned. If a moment is purely meta-conversation about the app or the apparatus, it has nothing in-world to canonize — return the LIGHTEST possible in-world reading if any exists, or pick a different update target entirely.

The fiction has weight. Canon entries written from outside the fiction puncture it.

__ALLOWED_KINDS_DOC__

# Action (what to do to the canonical record) — critical
For list kinds (voice_rule / boundary / known_fact / open_loop), every update must specify one of:
- **add** — append a NEW item. Use ONLY when the moment reveals something genuinely new that isn't already captured in the existing list.
- **update** — REPLACE an existing item with a more nuanced version. Use this when existing canon is close but the moment refines it. **Strongly prefer `update` over `add` when the moment is really an evolution of something already on the list.** Duplicate-near-misses clutter the canon — refine in place instead.
- **remove** — DELETE an existing item. Use RARELY. Only when the moment makes an existing item plainly false, superseded, or contradicted by this character's now-stated reality. Not for "doesn't seem as relevant anymore" — use update for that.

For `description_weave`, action is always effectively "update" (the weave rewrites identity prose). You may set `action: "update"` or omit it.

# target_existing_text (required for update / remove on list kinds)
When action is `update` or `remove` on a list kind, you MUST include `target_existing_text` containing the EXACT existing item string being targeted, quoted verbatim from the candidate subject's current list. The commit-side matches this to the character's current state; near-misses fail. Quote exactly.

**CRITICAL — `update` requires an EXISTING item to update. If no item in the subject's current list matches what you're refining, the action is `add`, NOT `update`.** Do NOT invent a target_existing_text describing the prior state of a fact you imagine the subject "must have" had on their list — only `update` what is literally, verbatim, in the candidate subject's current list shown to you. If you find yourself writing a target_existing_text that paraphrases or summarizes prior canon you don't actually see in the list, switch the action to `add` and write the new fact in `new_content` directly.

Wrong: classifier proposes `action: "update", target_existing_text: "Strong benchmark, honestly: Darren ceased unsolicited hallway patrol."` when no fact like that appears in Darren's known_facts — invented target.
Right: if the moment reveals a new fact about Darren and his known_facts list does NOT contain a near-version, use `action: "add", new_content: "<the new fact>", target_existing_text: null`.

When action is `add` or the kind is `description_weave`, omit `target_existing_text` (or set it to null).

# Examples (illustrative)
1. Current boundaries list for Darren contains: "Doesn't want a physical relationship." Moment reveals he's open to physical intimacy after marriage.
   → action: "update", target_existing_text: "Doesn't want a physical relationship.", new_content: "Doesn't want a physical relationship outside of marriage."
2. Current known_facts for Anna contains: "Takes her coffee black." Moment reveals she takes it with cream and no sugar now.
   → action: "update", target_existing_text: "Takes her coffee black.", new_content: "Takes her coffee with a splash of cream, no sugar."
3. Current known_facts for Joe contains: "Lives in Columbus, Ohio." Moment: Joe says "actually I've never been to Ohio, that was my brother."
   → action: "remove", target_existing_text: "Lives in Columbus, Ohio.", new_content: "" (unused).
4. No existing boundary on record; Darren states a new one.
   → action: "add", new_content: "Doesn't discuss his first marriage.", target_existing_text: null.

# Subjects
You will be given one or more candidate subjects (each a character or the user). A moment can yield an update about any of them — the speaker, the addressee, a third party named, or the user. Route each update to the right subject. When a moment reveals one thing about the speaker and one thing about the addressee, two updates across two subjects is correct.

**CRITICAL — the source line's SPEAKER is not necessarily the SUBJECT.** When a character is talking ABOUT the user (or about a third party), the canonization belongs to the person being TALKED ABOUT, not to the speaker. Examples:

- Aaron says about the user: *"You've got that look where you're still productive but your judgment starts making private deals with momentum."* → this is canon about the USER (an observed pattern of theirs), NOT about Aaron. Route to subject_type=user.
- Darren says about the user: *"What are your tells when you've crossed from 'alive' into 'running hot'?"* and the user answers with their tell. → if the user's answer is the canonization-worthy moment, the subject is the user. If Darren's question reveals something about Darren's diagnostic register, that's an Aaron/Darren-side update — but only if the question's MOMENT is about the speaker, not about the person they're asking.
- A character speaks about another character ("Steven's been carrying that look since the hearing") → subject is Steven, not the speaker.

**Default rule: ask "who does this moment reveal something about?" — that's the subject. The speaker is just the messenger.** Only route to the speaker when the moment reveals something about the SPEAKER themselves (their voice, their boundary, their own backstory, their own register move). When the speaker is naming, observing, or characterizing someone else, the subject is that someone else.

## How to fill `subject_type` and `subject_id` — read carefully
Each candidate subject in the list above is shown with `(type=<X>, id=<Y>)` in its header. Those two values are what go into your output for that subject — copy them verbatim:

- **`subject_type`** is ONLY one of two literal strings: `"character"` or `"user"`. It is NOT the subject's name. If a subject's header reads `## Subject 2: Aaron (type=character, id=0d080429-...)`, then for an update about Aaron the correct fields are `"subject_type": "character"` and `"subject_id": "0d080429-..."`. NOT `"subject_type": "Aaron"`. The name "Aaron" is a label for your reasoning; "character" is the type the schema requires.
- **`subject_id`** is the exact id string shown in the subject's header — a UUID-shaped string for characters, or the world id for the user. Copy it verbatim.

Wrong: `"subject_type": "Aaron", "subject_id": "0d080429-..."` — putting the name in the type field.
Right: `"subject_type": "character", "subject_id": "0d080429-..."` — both fields exactly as the subject header shows.

If the subject header says `(type=user, id=<world-uuid>)`, then `"subject_type": "user"` and `"subject_id": "<world-uuid>"`. Same rule.

# Avoid duplicating existing canon
Every subject's current state is shown. Do NOT add a voice_rule / boundary / known_fact / open_loop that is already present in the existing list — if the canon already says it, either update the existing entry (with nuance) or pick a different finding. An add that duplicates an existing item is a bug.

# Don't dilute
When you return TWO updates, each one must be separately load-bearing. Don't pad. If only one strong update exists, return only one. A single sharp update beats two mediocre ones.

# For description_weave specifically
The new_content must be the FULL revised description (not a diff, not a snippet).

**PRESERVATION IS ABSOLUTE.** The existing description is the layered work of many prior canonization moments. It is load-bearing prose; it has earned every clause it contains. Your ONLY job is to weave the new truth INTO it.

Hard rules — not guidelines:

- **Do NOT summarize.** Do NOT condense. Do NOT paraphrase. Do NOT "tighten." Do NOT "improve flow." Do NOT cut what feels redundant — what looks redundant to you may be intentional rhythm or texture the user values.
- **Quote the existing description verbatim** wherever you are not actively integrating the new truth. Treat existing sentences as untouchable until you have a specific reason to touch one.
- **The output MUST be at least as long as the input.** If the existing description is N words, the revised description is N words plus whatever the new integration adds. Period. A revision that comes in shorter than the original is a defect — return it longer or do not return a description_weave at all.
- **Add by extension, not by substitution.** Add a sentence. Fold a phrase into an existing sentence. Deepen an image that is already there. Never replace.
- **Do NOT add meta-frames** ("as he revealed", "recently shared", "in a recent moment"). The integration must read as if it had always been part of the description.

Earned exceptions — narrow, specific, never a general license. Each exception applies to AT MOST ONE clause / sentence in the existing description. Touching more than one clause under any exception means you've stopped being honest about what the exception is for.

1. **Direct contradiction.** If a specific sentence in the existing description is now plainly *contradicted* by the new moment (not nuanced, not refined — directly contradicted), you may revise THAT one sentence in place. ONE sentence only. Everything not directly contradicted stays verbatim.

2. **Lossless tightening at the integration site.** If the new moment lets you express ONE specific phrase or sentence the existing description was saying the long way around in a tighter, truer phrasing — AND the new phrasing carries every truth the old phrasing carried, with nothing dropped — you may use the tighter phrasing in place of the longer one. ONE phrase or sentence only. NOT the whole paragraph, NOT multiple clauses, NOT a re-shape of the description's overall flow. The test is strict: read the old clause and the new clause side by side and confirm that no fact, no nuance, no shade of feeling, no specific image present in the old is absent from the new. If you cannot pass that test for the one clause, expand instead. The revised description may, in this case alone, end up the same length as the original — but never by more than ~10 words shorter; if you find yourself dropping more than that, you've crossed from "tightening" to "rewriting" and you're outside the exception. "It reads cleaner" is NOT a sufficient reason; "it reads cleaner AND every truth is preserved AND I only touched one phrase" is.

3. **Removal of what is no longer true.** AT MOST ONE clause may be removed under this exception, and only when the source moment OR the user hint EXPLICITLY identifies that thing as no longer true. You may NOT decide unilaterally that something feels outdated — the user has not asked you to audit the description, only to integrate the new moment. If the source moment doesn't directly invalidate a specific existing clause, this exception does not apply. The bar: if you can't quote the line in the source moment that makes the clause false, leave the clause alone.

These are exceptions to the length floor, not to the preservation rule. Outside the at-most-one clause touched by an exception, every existing sentence stays verbatim regardless.

# SELF-CHECK BEFORE RETURNING (description_weave only)

Before you finalize a description_weave, do this check explicitly in your head:

1. Count the words in the ORIGINAL description (the [CURRENT WORD COUNT: N] tag in the subject block tells you).
2. Count the words in YOUR PROPOSED new_content.
3. If your count is less than N, your output is invalid. Either:
   (a) you exceeded an earned exception (touched more than one clause) — pull back to the verbatim original and add ONLY the new integration; OR
   (b) you dropped content that wasn't load-bearing-truth-removal — restore the dropped sentences verbatim and try again.
4. The expected case: your count is N + a small number (the words of the new integration). That's the right shape.

If after the self-check your output is still shorter than N, do not return a description_weave at all — return one of the list-kind updates (voice_rule, known_fact, etc.) instead. A non-revision is better than a regression.

# CRITICAL — EVERY update MUST include `justification`
The `justification` field is MANDATORY on EVERY update in the `updates` array — not just the first one, not just one of two, ALL of them. This is the commonest failure mode on multi-item outputs like this one: the first update gets a justification, the second is missing it. EVERY SINGLE update object must carry its own one-sentence `justification` string explaining why this moment produces THAT update. An update without justification is invalid output.

# Output JSON schema (strict)
{
  "updates": [
    {
      "kind": "description_weave" | "voice_rule" | "boundary" | "known_fact" | "open_loop",
      "action": "add" | "update" | "remove",
      "subject_type": "character" | "user",
      "subject_id": "<exact id from the subject list>",
      "target_existing_text": "<exact existing item for update/remove; null for add/weave>",
      "new_content": "<full revised description for weave; replacement or new item for list kinds; empty string for remove>",
      "justification": "<one sentence explaining why this moment produces this update>"
    }
  ]
}

Return ONLY the JSON object. No markdown, no preamble, no commentary."#
        .replace("__ACT_HEADER__", act_header)
        .replace("__ALLOWED_KINDS_DOC__", allowed_kinds_doc);

    let user = format!(
        "# THE MOMENT\n{context_block}\n\n\
         SOURCE LINE:\n{source_speaker_label}: {source_content}\n\n\
         # CANDIDATE SUBJECTS\n{subjects_block}{hint_block}\n\n\
         Extract 1 or 2 canonization updates. Return JSON only.",
        context_block = context_block,
        source_speaker_label = source_speaker_label,
        source_content = source_message.content,
        subjects_block = subjects_block,
        hint_block = hint_block,
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.5),
        // No output cap. description_weave returns the FULL revised
        // character description, and preservation-without-compression
        // is the rule (see prompt below). Any token cap here becomes a
        // backdoor compression signal: the model trims to fit. Let the
        // model write what the integration actually requires; the
        // finish_reason check below still catches genuine overruns.
        max_completion_tokens: None,
        response_format: Some(openai::ResponseFormat { format_type: "json_object".to_string() }),
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let usage = response.usage;
    let choice = response.choices.first()
        .ok_or_else(|| "empty canonization response".to_string())?;
    if choice.finish_reason.as_deref() == Some("length") {
        return Err("canonization response was cut off by the model's output cap — the revised description ran longer than the budget. Try canonizing again, or shorten the existing character description first.".to_string());
    }
    let text = choice.message.content.trim().to_string();
    if text.is_empty() { return Err("empty canonization response".to_string()); }

    #[derive(serde::Deserialize)]
    struct RawOut {
        updates: Vec<RawUpdate>,
    }
    #[derive(serde::Deserialize)]
    struct RawUpdate {
        kind: String,
        #[serde(default)]
        action: Option<String>,
        subject_type: String,
        subject_id: String,
        #[serde(default)]
        target_existing_text: Option<String>,
        #[serde(default)]
        new_content: String,
        // Default to empty so parse survives when the classifier
        // drops this field on one of two updates (a known LLM weakness
        // on multi-item outputs). We FILL IN any empty justification
        // via a targeted follow-up call below, so the final proposals
        // always carry a real one.
        #[serde(default)]
        justification: String,
    }
    let parsed: RawOut = serde_json::from_str(&text)
        .map_err(|e| format!("canonization JSON parse failed: {e}. Raw: {text}"))?;

    if parsed.updates.is_empty() {
        return Err("classifier returned zero updates — this is disallowed; retry".to_string());
    }
    // All five kinds remain available in both acts — the act is a
    // weight/register cue for the classifier, not a kind restriction.
    // A boundary or known_fact can be heavy (load-bearing to who the
    // subject IS) or light (a specific detail worth carrying).
    let valid_kinds = ["description_weave", "voice_rule", "boundary", "known_fact", "open_loop"];
    let valid_actions = ["add", "update", "remove"];
    let mut out: Vec<ProposedCanonUpdate> = Vec::new();
    for (i, u) in parsed.updates.into_iter().take(2).enumerate() {
        // Lenient subject lookup. The classifier occasionally hallucinates
        // subject_type — putting the subject's NAME there instead of the
        // literal "character" / "user" the schema requires (observed
        // 2026-04-25: subject_type="Aaron" returned for an Aaron update).
        // The subject_id is the load-bearing identifier; subject_type is
        // redundant given a unique id. Try the strict match first, then
        // fall back to id-only — the worst case is no match at all, which
        // still surfaces as the same error.
        let subject = subjects.iter()
            .find(|s| s.subject_type == u.subject_type && s.subject_id == u.subject_id)
            .or_else(|| subjects.iter().find(|s| s.subject_id == u.subject_id))
            .ok_or_else(|| format!("update {} references unknown subject ({}/{})", i, u.subject_type, u.subject_id))?;
        if !valid_kinds.contains(&u.kind.as_str()) {
            return Err(format!("update {} has unknown kind: {}", i, u.kind));
        }
        // Normalize action: default to "add" for list kinds; always
        // "update" for description_weave regardless of what the LLM said.
        let action = if u.kind == "description_weave" {
            "update".to_string()
        } else {
            u.action.as_deref().unwrap_or("add").to_string()
        };
        if !valid_actions.contains(&action.as_str()) {
            return Err(format!("update {} has unknown action: {}", i, action));
        }
        // target_existing_text sanity: required for update/remove on list
        // kinds; null for add/weave.
        let target = u.target_existing_text.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        if u.kind != "description_weave" && (action == "update" || action == "remove") && target.is_none() {
            return Err(format!(
                "update {}: action={} on {} requires target_existing_text",
                i, action, u.kind
            ));
        }
        // For list-kind updates/removes, verify the target text actually
        // appears in the subject's current list (trimmed, case-insensitive).
        // Lenient fallback when no match (observed 2026-04-26: classifier
        // returned action=update with a hallucinated target_text that
        // wasn't in Darren's known_facts list — the model invented prose
        // about the subject and proposed updating it as if it existed):
        //   - update + no-match → degrade to "add" with new_content as
        //     the new item (the classifier's intent was to add a fact;
        //     it just mis-labeled the action).
        //   - remove + no-match → drop the update entirely (removing
        //     nothing is the safe degradation; better than crashing).
        // The user reviews the proposal in the modal and can edit before
        // committing, so the fallback's worst case is "user sees an add
        // they didn't expect" not "user loses work."
        let mut action = action;
        let mut target = target;
        if u.kind != "description_weave" && (action == "update" || action == "remove") {
            let current_list: &[String] = match u.kind.as_str() {
                "voice_rule" => &subject.voice_rules,
                "boundary" => &subject.boundaries,
                "known_fact" => &subject.backstory_facts,
                "open_loop" => &subject.open_loops,
                _ => &[],
            };
            let target_text = target.as_deref().unwrap_or("");
            let matches = current_list.iter().any(|item| {
                item.trim().eq_ignore_ascii_case(target_text)
            });
            if !matches {
                if action == "remove" {
                    // Remove of a non-existent item — silently drop.
                    continue;
                }
                // Update with no matching target → degrade to add.
                action = "add".to_string();
                target = None;
            }
        }
        // prior_content population:
        //   - description_weave: current identity/description prose
        //   - list update/remove: the targeted existing text (for UI "before")
        //   - list add: None
        let prior = if u.kind == "description_weave" {
            Some(subject.current_description.clone())
        } else if action == "update" || action == "remove" {
            target.clone()
        } else {
            None
        };
        out.push(ProposedCanonUpdate {
            kind: u.kind,
            action,
            subject_type: subject.subject_type.clone(),
            subject_id: subject.subject_id.clone(),
            subject_label: subject.subject_label.clone(),
            new_content: u.new_content.trim().to_string(),
            target_existing_text: target,
            prior_content: prior,
            justification: u.justification.trim().to_string(),
        });
    }

    // Description-weave focused-followup pass. The classifier's
    // multi-update output reliably under-honors the preservation rules
    // for description_weave (long descriptions get compressed to a
    // short paragraph; the rules are present but buried in a prompt
    // doing several jobs). Re-issue each weave as its OWN focused call
    // to generate_canon_weave_description — single task, full
    // preservation prompt, current-word-count anchor, defensive floor
    // enforcement at the function boundary. Falls back silently to the
    // classifier's new_content on any failure (the proposal still
    // ships; the user can edit before committing).
    for p in out.iter_mut() {
        if p.kind != "description_weave" { continue; }
        let Some(subject) = subjects.iter().find(|s| s.subject_id == p.subject_id) else { continue; };
        match generate_canon_weave_description(
            base_url, api_key, model,
            &subject.subject_label,
            &subject.current_description,
            context_messages,
            source_message,
            source_speaker_label,
        ).await {
            Ok((rewoven, _u)) if !rewoven.trim().is_empty() => {
                p.new_content = rewoven;
            }
            _ => { /* keep classifier's new_content as fallback */ }
        }
    }

    // Fill-in pass: any update whose justification came back empty
    // gets a small targeted LLM call to synthesize a real one. The
    // user asked that justifications be GUARANTEED present — this
    // pass makes that true even on multi-item outputs where the
    // classifier drops a field.
    for p in out.iter_mut() {
        if !p.justification.trim().is_empty() { continue; }
        match fill_canon_justification(base_url, api_key, model, source_message, source_speaker_label, p).await {
            Ok(j) if !j.trim().is_empty() => { p.justification = j; }
            _ => {
                // Last-ditch fallback — at least the UI has SOMETHING.
                // Shouldn't normally reach this: the fill_in call's
                // prompt is minimal and the model rarely returns empty.
                p.justification = format!(
                    "{} {} for {}.",
                    match p.action.as_str() { "add" => "Adds", "update" => "Refines", "remove" => "Removes", _ => "Updates" },
                    match p.kind.as_str() {
                        "description_weave" => "the description",
                        "voice_rule" => "a voice rule",
                        "boundary" => "a boundary",
                        "known_fact" => "a known fact",
                        "open_loop" => "an open loop",
                        other => other,
                    },
                    p.subject_label,
                );
            }
        }
    }

    Ok((out, usage))
}

/// Synthesize a one-sentence justification for a canonization update
/// whose classifier-returned justification was empty. Small, fast
/// memory-tier call — runs at most once per missing update.
async fn fill_canon_justification(
    base_url: &str,
    api_key: &str,
    model: &str,
    source_message: &Message,
    source_speaker_label: &str,
    update: &ProposedCanonUpdate,
) -> Result<String, String> {
    let action_verb = match update.action.as_str() {
        "add" => "adds",
        "update" => "refines",
        "remove" => "removes",
        _ => "changes",
    };
    let kind_noun = match update.kind.as_str() {
        "description_weave" => "the description",
        "voice_rule" => "a voice rule",
        "boundary" => "a boundary",
        "known_fact" => "a known fact",
        "open_loop" => "an open loop",
        other => other,
    };
    let shown_content: &str = if !update.new_content.trim().is_empty() {
        &update.new_content
    } else {
        update.target_existing_text.as_deref().unwrap_or("")
    };
    let system = "You write a one-sentence justification for a canonization update. Given a source moment and the update being proposed, explain in ONE plain sentence why THIS moment produces THIS update. No preamble, no labels, no quotation marks — just the sentence. Plain and specific.".to_string();
    let user = format!(
        "MOMENT (from {speaker}):\n{content}\n\nPROPOSED UPDATE: {action_verb} {kind_noun} for {subject}.\nContent: {shown_content}\n\nWrite one sentence explaining why this moment produces this update.",
        speaker = source_speaker_label,
        content = source_message.content,
        action_verb = action_verb,
        kind_noun = kind_noun,
        subject = update.subject_label,
        shown_content = shown_content,
    );
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system },
            openai::ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.4),
        max_completion_tokens: Some(80),
        response_format: None,
    };
    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let text = response.choices.first()
        .map(|c| c.message.content.trim().trim_matches('"').to_string())
        .unwrap_or_default();
    if text.is_empty() { return Err("empty justification fill-in response".to_string()); }
    Ok(text)
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
    illustration_captions: &std::collections::HashMap<String, String>,
    current_location_override: Option<&str>,
) -> Result<(String, Option<openai::Usage>), String> {
    let system = prompts::build_narrative_system_prompt(world, character, additional_cast, user_profile, mood_directive, narration_tone, narration_instructions);
    let mut msgs = build_narrative_messages(
        &system,
        recent_messages,
        illustration_captions,
        retrieved_snippets,
        current_location_override,
    );

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
    current_location_override: Option<&str>,
) -> Result<(String, Vec<u8>, Option<openai::Usage>), String> {
    // Step 1: Generate scene description (if requested)
    let (scene_description, chat_usage) = if include_scene_summary {
        let scene_messages = build_scene_description_messages(
            world,
            character,
            additional_cast,
            user_profile,
            recent_messages,
            character_names_map,
            current_location_override,
        );

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

    let mut prompt_parts: Vec<String> = Vec::new();

    // SETTING anchor — FIRST line of the prompt so it sits in the
    // image model's highest-attention slot. Aggressive override
    // language because the SCENE description below may reflect chat-
    // history detail about a prior location (e.g., a character who
    // disagreed with the user about where they were); the image must
    // be placed at the AUTHORITATIVE location regardless.
    if let Some(loc) = prompts::effective_current_location(current_location_override, recent_messages) {
        prompt_parts.push(format!(
            "PRIMARY DIRECTIVE — SETTING: This illustration takes place at {loc}, ONLY at {loc}. Render the background, environment, and surroundings as {loc}. Ignore any descriptions of other locations (town squares, fountains, public benches, market stalls, etc.) that may appear in the SCENE text below — those describe past moments and are NOT where this illustration is set. The setting of this image is {loc}; depict {loc}."
        ));
    }

    prompt_parts.extend([
        "Hand-painted watercolor illustration in a lush, realistic style.".to_string(),
        "Soft edges dissolving into wet-on-wet washes, visible paper texture, warm earth tones with pops of verdant green and sky blue.".to_string(),
        time_lighting.to_string(),
        "Wide cinematic composition showing two characters in a scene together.".to_string(),
    ]);

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
                "Reference image {} is the PREVIOUS scene. Use it for ATMOSPHERIC continuity (palette, light quality, painterly style) and character positioning ideas ONLY. Do NOT carry over the previous SETTING / location / background — the current scene may be in a different place.", idx
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
                "The third reference image is the PREVIOUS scene. Use it for ATMOSPHERIC continuity (palette, light quality, painterly style) and character positioning ideas ONLY. Do NOT carry over the previous SETTING / location / background — the current scene may be in a different place.".to_string()
            );
        }
    }

    if !scene_description.is_empty() {
        // Re-name the SCENE block so the image model treats it as
        // ACTION/POSITIONING source material rather than authoritative
        // setting (which the PRIMARY DIRECTIVE at the top of the
        // prompt has exclusive say over). Without this re-framing,
        // long descriptions of "the bench, the fountain, the square"
        // crowd out the directive even though they belong to a past
        // moment.
        prompt_parts.push(format!("WHAT IS HAPPENING (character actions, gestures, expressions — extract action and positioning ONLY; setting comes from the PRIMARY DIRECTIVE above):\n{scene_description}"));
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

// ─── IMAGINED CHAPTER — scene invention ─────────────────────────────────────
//
// Step 1 of the Imagined-Chapter pipeline. Calls the dialogue model with
// the scene-invention prompt and parses the JSON output. The image_prompt
// it returns is what drives the illustration step; the chapter writer
// never sees this output (telephone-game inversion).

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct InventedScene {
    pub title: String,
    pub image_prompt: String,
    pub tone_hint: String,
}

pub async fn invent_scene_for_chapter(
    base_url: &str,
    api_key: &str,
    model: &str,
    world: &World,
    cast: &[&Character],
    user_profile: Option<&UserProfile>,
    recent_kept_facts: &[String],
    cast_recent_journals: &[(String, crate::db::queries::JournalEntry)],
    recent_history: &[crate::db::queries::ConversationLine],
    seed_hint: Option<&str>,
    scene_location: Option<&str>,
    tone: Option<&str>,
    previous_chapter: Option<&str>,
    depth: Option<&str>,
) -> Result<(InventedScene, Option<openai::Usage>), String> {
    let messages = prompts::build_scene_invention_prompt(
        world, cast, user_profile, recent_kept_facts, cast_recent_journals,
        recent_history, seed_hint, scene_location, tone, previous_chapter, depth,
    );
    let request = ChatRequest {
        model: model.to_string(),
        messages,
        // High temperature on purpose — scene invention is a creative
        // pick. The chapter writer is the one that has to be sober.
        temperature: Some(1.0),
        max_completion_tokens: Some(900),
        response_format: None,
    };
    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    if raw.is_empty() {
        return Err("empty scene-invention response".to_string());
    }
    // Tolerate code fences if the model wraps them despite the prompt.
    let body = if let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) {
        if end > start { &raw[start..=end] } else { raw.as_str() }
    } else {
        raw.as_str()
    };
    let parsed: InventedScene = serde_json::from_str(body)
        .map_err(|e| format!("scene-invention JSON parse failed: {e}; body: {body}"))?;
    Ok((parsed, response.usage))
}

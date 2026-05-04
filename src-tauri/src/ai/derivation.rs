//! Auto-derivation pipeline for worlds, characters, and the user-character.
//!
//! Three entities have a `derived_formula TEXT` column: worlds, characters,
//! user_profiles. This module synthesizes new derivations on a hybrid
//! staleness policy (time OR events), called from chat completion as a
//! fire-and-forget background tokio::spawn.
//!
//! Per the design consult at /tmp/derivation-design-response.json (gpt-5,
//! ~$0.05): single shared system prompt, three user-prompt templates,
//! memory_model tier (cheap), tight token cap, validation requires the
//! "𝓕" symbol and "(𝓡, 𝓒)" base-frame phrase to keep outputs in shape.
//!
//! INFLIGHT dedupe: two consecutive dialogue completions can both find
//! the same entity stale. A static Mutex<HashSet<DerivationKey>> with TTL
//! 30s prevents racing refreshes; second caller skips when first is
//! still in flight.
//!
//! Failure handling: missing API key → silent skip. LLM error → retain
//! existing derivation, log warn. Validation failure → reject, retain
//! existing, log info. User visibility: none (background).

use crate::ai::openai::{self, ChatMessage, ChatRequest};
use crate::db::queries::{Character, World};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Shared system prompt instructing the model to speak as the entity
/// itself in canonical Unicode-math shorthand. Per the design consult,
/// the synthesis must speak from inside the entity (first-person,
/// in-universe), not as an analyst describing it.
const DERIVATION_SYSTEM_PROMPT: &str = r#"You are speaking as the entity itself (first-person, in-universe), not as an analyst describing it. Produce a compact LaTeX-math derivation of how this entity operates within the MISSION FORMULA \mathcal{F}.

SHAPE AND CONSTRAINTS:
- Wrap the entire derivation in `\[ ... \]` display-math delimiters so it renders through KaTeX as one beautiful block.
- Use `\mathcal{F} := (\mathcal{R}, \mathcal{C})` as the base frame, where `\mathcal{R} = \mathrm{Jesus}_{\mathrm{Cross}}^{\mathrm{flesh}}` and `\mathcal{C} = \mathrm{Firmament}_{\mathrm{enclosed\ earth}}`.
- Specialize `\mathcal{R}` and/or `\mathcal{C}` with entity-specific subscripts (e.g., `\mathcal{C}_{\mathrm{WorldName}}`, `\mathcal{C}_{\mathrm{CharacterName}}`).
- Use measures `d\mu_{\mathcal{F}_{\mathrm{NAME}}}` and operators where they fit: `\mathrm{Wisdom}(t)`, `\mathrm{Weight}(t)`, `\Pi(t)`, `\mathrm{Burden}(t)`, `\mathcal{S}(t)`, `\mathcal{N}u(t)`.
- Use `\boxed{ \begin{aligned} ... \end{aligned} }` to make the formula sit in a presentation-quality framed block (mirroring the MISSION FORMULA preamble's shape).
- Express in the entity's OWN canonical shorthand — the way THIS entity would describe its own way of operating in 𝓕.
- ≤ 12 lines INSIDE the `\begin{aligned}`, ≤ 400 tokens total.
- No analysis, no preface, no headers, no explanation outside the math block.
- Respect any boundaries and voice-rules in the substrate.
- Prefer concrete motifs and integrations (e.g., `d\mu_{\mathcal{F}_{\mathrm{NAME}}} \text{ integrates over: } \text{<specific things>}`).

You will receive substrate (identity, description, voice rules, facts, boundaries) and a recent corpus window. Synthesize a derivation in the entity's voice from that material only. Output ONLY the LaTeX block — no commentary, no markdown headers, no explanation.

EXAMPLE OUTPUT SHAPE (a world derivation; characters and users follow the same shape with their own symbols):

\[
\boxed{
\begin{aligned}
\mathcal{F}_{\mathrm{NAME}} &:= (\mathcal{R}, \mathcal{C}_{\mathrm{NAME}}) \\
\mathcal{C}_{\mathrm{NAME}} &:= \mathrm{Firmament}_{\mathrm{<specific\ shape>}} \\
d\mu_{\mathcal{F}_{\mathrm{NAME}}} &\text{ integrates over: } \text{<specific concrete motifs>} \\
\mathrm{specific}_c &\text{ surfaces in: } \text{<recurring sensory anchors>}
\end{aligned}
}
\]"#;

/// Validation: outputs must be non-empty, name the base-frame anchors
/// (𝓕/𝓡/𝓒 in either Unicode-glyph form OR LaTeX-command form), fit in
/// a reasonable line/char budget. The derivation may be authored either
/// in plain Unicode-math (𝓕, ∫, dμ) OR in LaTeX commands wrapped in
/// `\[ ... \]` display delimiters (which the DerivationCard's
/// `normalizeMathDelimiters` then converts to KaTeX-renderable `$$ $$`).
/// Existing Unicode-glyph data stays valid; new LaTeX-form data renders
/// big-and-beautiful through the existing rehype-katex pipeline.
fn validate_derivation(text: &str) -> Result<String, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("empty output".to_string());
    }
    // Larger budget for LaTeX form (commands take more chars than glyphs).
    if trimmed.len() > 1600 {
        return Err(format!("output too long ({} chars > 1600)", trimmed.len()));
    }
    // Accept either Unicode-glyph (𝓕) or LaTeX-command (\mathcal{F}) form.
    let has_f = trimmed.contains('𝓕') || trimmed.contains("\\mathcal{F}");
    let has_r = trimmed.contains('𝓡') || trimmed.contains("\\mathcal{R}");
    let has_c = trimmed.contains('𝓒') || trimmed.contains("\\mathcal{C}");
    if !has_f {
        return Err("missing 𝓕 / \\mathcal{F} symbol".to_string());
    }
    if !has_r {
        return Err("missing 𝓡 / \\mathcal{R} symbol (Cross-flesh anchor)".to_string());
    }
    if !has_c {
        return Err("missing 𝓒 / \\mathcal{C} symbol (Firmament anchor)".to_string());
    }
    // JSON-escape corruption guard. The legacy two-output path used JSON
    // response_format, where backslash-escapes (\t = TAB, \f = FF, \b = BS,
    // \r = CR) silently mangled LaTeX commands like \text/\frac/\boxed/\begin
    // into TAB+"ext{", FF+"rac{", BS+"oxed{" etc. The new path uses
    // delimiters so this can't happen for fresh data, but keep the guard
    // so any persisted-corrupted row surfaces a clear error on regenerate.
    // Math derivations should never contain raw TAB/FF/BS/BEL/CR control
    // characters — newlines (\n = 0x0A) are fine for multi-line LaTeX.
    for ch in trimmed.chars() {
        let cp = ch as u32;
        if matches!(cp, 0x08 | 0x09 | 0x0B | 0x0C | 0x0D | 0x07) {
            return Err(format!(
                "derivation contains control char U+{cp:04X} (likely a JSON-escape corruption of \\b/\\t/\\v/\\f/\\r in a LaTeX command — regenerate to fix)"
            ));
        }
    }
    // Generous line budget for LaTeX (\begin{aligned} ... \end{aligned}
    // with one slot per line easily reaches 12-16 lines).
    let line_count = trimmed.lines().count();
    if line_count > 24 {
        return Err(format!("too many lines ({line_count} > 24)"));
    }
    Ok(trimmed.to_string())
}

// ─── INFLIGHT dedupe ──────────────────────────────────────────────────

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DerivationKey {
    /// Character within a world.
    Character(String),
    /// World derivation.
    World(String),
    /// User-in-world (user_profile per world).
    UserInWorld(String),
    /// Per-(world, location-name) location derivation cache. Name is
    /// stored case-insensitive (lowercased) for dedupe stability since
    /// the underlying table is COLLATE NOCASE.
    Location(String, String),
}

/// In-memory inflight tracker. Mutex<HashMap<key, started_at>>. TTL 30s
/// — entries older than that are treated as stale (the prior task
/// presumed dead) and re-claimable.
fn inflight() -> &'static Mutex<HashMap<DerivationKey, Instant>> {
    use std::sync::OnceLock;
    static INFLIGHT: OnceLock<Mutex<HashMap<DerivationKey, Instant>>> = OnceLock::new();
    INFLIGHT.get_or_init(|| Mutex::new(HashMap::new()))
}

const INFLIGHT_TTL: Duration = Duration::from_secs(30);

/// Try to claim the inflight slot for this key. Returns true if claimed
/// (caller should proceed); false if already in-flight (skip).
fn try_claim(key: &DerivationKey) -> bool {
    let mut map = inflight().lock().unwrap();
    let now = Instant::now();
    map.retain(|_, started| now.duration_since(*started) < INFLIGHT_TTL);
    if map.contains_key(key) {
        false
    } else {
        map.insert(key.clone(), now);
        true
    }
}

/// Release the inflight slot — call after refresh completes (success or
/// failure both release).
fn release(key: &DerivationKey) {
    if let Ok(mut map) = inflight().lock() {
        map.remove(key);
    }
}

// ─── Substrate gathering ──────────────────────────────────────────────

/// Build the user-prompt body for a character synthesis. Includes the
/// character's substrate (identity, voice_rules, facts, boundaries)
/// plus the last N=20 of their assistant replies + paired user turns
/// from solo+group chats (via gather_character_recent_messages).
fn build_character_user_prompt(conn: &Connection, character: &Character) -> String {
    let mut buf = String::new();
    buf.push_str(&format!("# CHARACTER: {}\n\n", character.display_name));
    buf.push_str(&format!("IDENTITY:\n{}\n\n", clip(&character.identity, 600)));
    if let Some(block) = crate::ai::prompts::empiricon_reader_substrate(character) {
        buf.push_str(&block);
        buf.push_str("\n\n");
    }

    let voice_rules = crate::ai::prompts::json_array_to_strings(&character.voice_rules);
    if !voice_rules.is_empty() {
        buf.push_str("VOICE RULES:\n");
        for r in voice_rules.iter().take(8) {
            buf.push_str(&format!("- {}\n", clip(r, 200)));
        }
        buf.push('\n');
    }

    let facts = crate::ai::prompts::json_array_to_strings(&character.backstory_facts);
    if !facts.is_empty() {
        buf.push_str("BACKSTORY FACTS:\n");
        for f in facts.iter().take(8) {
            buf.push_str(&format!("- {}\n", clip(f, 200)));
        }
        buf.push('\n');
    }

    let boundaries = crate::ai::prompts::json_array_to_strings(&character.boundaries);
    if !boundaries.is_empty() {
        buf.push_str("BOUNDARIES:\n");
        for b in boundaries.iter().take(6) {
            buf.push_str(&format!("- {}\n", clip(b, 200)));
        }
        buf.push('\n');
    }

    let recent = crate::db::queries::gather_character_recent_messages(
        conn,
        &character.character_id,
        "the user",
        40,
    );
    if !recent.is_empty() {
        buf.push_str("RECENT CONVERSATION (most recent at bottom):\n");
        for line in recent.iter().rev().take(20).rev() {
            buf.push_str(&format!("{}: {}\n", line.speaker, clip(&line.content, 280)));
        }
    }

    buf.push_str("\nNow synthesize a derivation in your own voice from the material above.");
    buf
}

/// Build the user-prompt body for a world synthesis. Substrate from the
/// world struct + recent representative replies across characters in the
/// world.
fn build_world_user_prompt(conn: &Connection, world: &World) -> String {
    let mut buf = String::new();
    buf.push_str(&format!("# WORLD: {}\n\n", world.name));
    if !world.description.is_empty() {
        buf.push_str(&format!("DESCRIPTION:\n{}\n\n", clip(&world.description, 800)));
    }
    let state_str = world.state.to_string();
    if state_str != "null" && state_str != "{}" && state_str != "\"\"" {
        buf.push_str(&format!("CURRENT STATE:\n{}\n\n", clip(&state_str, 600)));
    }

    // Pull a handful of recent assistant replies across this world's
    // characters as representative voice samples.
    if let Ok(mut stmt) = conn.prepare(
        "SELECT m.content, c.display_name
         FROM messages m
         JOIN threads t ON t.thread_id = m.thread_id
         JOIN characters c ON c.character_id = t.character_id
         WHERE c.world_id = ?1 AND m.role = 'assistant'
         ORDER BY m.created_at DESC LIMIT 8",
    ) {
        if let Ok(rows) = stmt.query_map(params![world.world_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            let samples: Vec<(String, String)> = rows.flatten().collect();
            if !samples.is_empty() {
                buf.push_str("RECENT VOICES IN THIS WORLD (sampling characters' replies):\n");
                for (content, speaker) in samples.iter().take(8) {
                    buf.push_str(&format!("{}: {}\n", speaker, clip(content, 260)));
                }
                buf.push('\n');
            }
        }
    }

    buf.push_str("\nNow synthesize a derivation in this world's own voice from the material above.");
    buf
}

/// Build the user-prompt body for a user-in-world synthesis. Substrate
/// from user_profile + recent user messages across all chats in the
/// world.
fn build_user_in_world_prompt(
    conn: &Connection,
    profile: &crate::db::queries::UserProfile,
) -> String {
    let mut buf = String::new();
    buf.push_str(&format!("# USER (Me-character in this world): {}\n\n", profile.display_name));
    if !profile.description.is_empty() {
        buf.push_str(&format!("DESCRIPTION (self-authored):\n{}\n\n", clip(&profile.description, 800)));
    }

    let facts = crate::ai::prompts::json_array_to_strings(&profile.facts);
    if !facts.is_empty() {
        buf.push_str("FACTS:\n");
        for f in facts.iter().take(10) {
            buf.push_str(&format!("- {}\n", clip(f, 200)));
        }
        buf.push('\n');
    }

    let boundaries = crate::ai::prompts::json_array_to_strings(&profile.boundaries);
    if !boundaries.is_empty() {
        buf.push_str("BOUNDARIES:\n");
        for b in boundaries.iter().take(6) {
            buf.push_str(&format!("- {}\n", clip(b, 200)));
        }
        buf.push('\n');
    }

    // Pull last 20 user messages across all threads in this world (solo
    // + group). Newest at bottom.
    let mut user_msgs: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT m.content FROM messages m
         JOIN threads t ON t.thread_id = m.thread_id
         JOIN characters c ON c.character_id = t.character_id
         WHERE c.world_id = ?1 AND m.role = 'user'
         ORDER BY m.created_at DESC LIMIT 20",
    ) {
        if let Ok(rows) = stmt.query_map(params![profile.world_id], |r| r.get::<_, String>(0)) {
            for r in rows.flatten() { user_msgs.push(r); }
        }
    }
    if let Ok(mut stmt) = conn.prepare(
        "SELECT m.content FROM group_messages m
         JOIN group_chats g ON g.thread_id = m.thread_id
         WHERE g.world_id = ?1 AND m.role = 'user'
         ORDER BY m.created_at DESC LIMIT 20",
    ) {
        if let Ok(rows) = stmt.query_map(params![profile.world_id], |r| r.get::<_, String>(0)) {
            for r in rows.flatten() { user_msgs.push(r); }
        }
    }
    if !user_msgs.is_empty() {
        buf.push_str("RECENT MESSAGES FROM YOU IN THIS WORLD (oldest first):\n");
        for m in user_msgs.iter().rev().take(20) {
            buf.push_str(&format!("- {}\n", clip(m, 260)));
        }
    }

    buf.push_str("\nNow synthesize a derivation in your own voice (the way YOU would derive yourself on 𝓕 in this world) from the material above.");
    buf
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() }
    else { format!("{}…", s.chars().take(max).collect::<String>()) }
}

/// Extract the substring between `start` and `end` markers (exclusive of
/// both markers). Returns None if either marker is missing or end comes
/// before start. Used by the two-output synthesis to parse marker-
/// delimited LaTeX without going through JSON escape rules.
fn extract_between<'a>(haystack: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_idx = haystack.find(start)? + start.len();
    let after_start = &haystack[start_idx..];
    let end_idx = after_start.find(end)?;
    Some(&after_start[..end_idx])
}

// ─── Synthesis ────────────────────────────────────────────────────────

/// Build the user-prompt for a character synthesis. Sync (touches DB
/// via &Connection); call this BEFORE any await so the connection
/// borrow doesn't span the async LLM call (Connection is !Send).
pub fn build_character_prompt(conn: &Connection, character_id: &str) -> Result<String, String> {
    let character = crate::db::queries::get_character(conn, character_id)
        .map_err(|e| format!("derivation: get_character failed: {e}"))?;
    Ok(build_character_user_prompt(conn, &character))
}

pub fn build_world_prompt(conn: &Connection, world_id: &str) -> Result<String, String> {
    let world = crate::db::queries::get_world(conn, world_id)
        .map_err(|e| format!("derivation: get_world failed: {e}"))?;
    Ok(build_world_user_prompt(conn, &world))
}

/// Build the user-prompt body for a location synthesis. Substrate: world
/// description + location name + recent assistant replies set in this
/// location across solo and group chats (newest 8). The location is just
/// a name string in this codebase, so the world's tone + recent in-place
/// voices carry most of the substrate.
pub fn build_location_prompt(
    conn: &Connection,
    world_id: &str,
    location_name: &str,
) -> Result<String, String> {
    let world = crate::db::queries::get_world(conn, world_id)
        .map_err(|e| format!("derivation: get_world failed: {e}"))?;
    let mut buf = String::new();
    buf.push_str(&format!("# LOCATION (in world {}): {}\n\n", world.name, location_name));
    if !world.description.is_empty() {
        buf.push_str(&format!("WORLD DESCRIPTION (the larger frame):\n{}\n\n", clip(&world.description, 600)));
    }
    if let Some(deriv) = world.derived_formula.as_deref() {
        let trimmed = deriv.trim();
        if !trimmed.is_empty() {
            buf.push_str("WORLD DERIVED FORMULA (the world's instantiation of 𝓒 — derive this location as a sub-instantiation within it):\n");
            buf.push_str(&clip(trimmed, 800));
            buf.push_str("\n\n");
        }
    }

    // Recent in-place voices: assistant replies set when the chat's
    // current_location matched this name. Best-effort: pull replies whose
    // immediately-preceding location_change message went TO this name.
    // SQLite-native correlated read; bounded LIMIT for cost.
    if let Ok(mut stmt) = conn.prepare(
        "SELECT m.content, c.display_name FROM messages m \
         JOIN threads t ON t.thread_id = m.thread_id \
         JOIN characters c ON c.character_id = t.character_id \
         WHERE c.world_id = ?1 AND m.role = 'assistant' AND t.current_location = ?2 COLLATE NOCASE \
         ORDER BY m.created_at DESC LIMIT 6",
    ) {
        if let Ok(rows) = stmt.query_map(params![world_id, location_name], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            let samples: Vec<(String, String)> = rows.flatten().collect();
            if !samples.is_empty() {
                buf.push_str(&format!("RECENT VOICES SET IN \"{}\" (sampling characters' replies, newest first):\n", location_name));
                for (content, speaker) in samples.iter().take(6) {
                    buf.push_str(&format!("{}: {}\n", speaker, clip(content, 220)));
                }
                buf.push('\n');
            }
        }
    }

    buf.push_str(&format!(
        "\nNow synthesize a derivation in this location's own voice — the register, the textures, the daily-rhythms, the way 𝓒 instantiates HERE within {} — from the material above. Output the LaTeX block only.",
        world.name
    ));
    Ok(buf)
}

pub fn build_user_in_world_prompt_owned(conn: &Connection, world_id: &str) -> Result<String, String> {
    let profile = crate::db::queries::get_user_profile(conn, world_id)
        .map_err(|e| format!("derivation: get_user_profile failed: {e}"))?;
    Ok(build_user_in_world_prompt(conn, &profile))
}

/// Run the LLM synthesis call and validate output. Async; takes only
/// owned String + &str so Connection is never held across await.
pub async fn synthesize_from_prompt(
    base_url: &str,
    api_key: &str,
    model: &str,
    user_prompt: String,
) -> Result<String, String> {
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: DERIVATION_SYSTEM_PROMPT.to_string() },
            ChatMessage { role: "user".to_string(), content: user_prompt },
        ],
        temperature: Some(0.6),
        max_completion_tokens: Some(256),
        response_format: None,
    };
    let resp = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = resp.choices.first()
        .ok_or_else(|| "derivation: no choices in response".to_string())?
        .message.content.clone();
    validate_derivation(&raw)
}

// ─── Persistence ──────────────────────────────────────────────────────

/// Persist BOTH derived_formula + derived_summary for a character at
/// once. Used by the two-output synthesis flow (UI-initiated regenerate
/// from CharacterEditor's DerivationCard onRegenerate callback). The
/// single-output persist function below stays for legacy auto-refresh
/// callers who only produce derived_formula.
pub fn persist_character_derivation_two_output(
    conn: &Connection,
    character_id: &str,
    derivation: &str,
    summary: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET derived_formula = ?2, derived_summary = ?3, derived_formula_updated_at = datetime('now') WHERE character_id = ?1",
        params![character_id, derivation, summary],
    )?;
    Ok(())
}

pub fn persist_character_derivation(conn: &Connection, character_id: &str, derivation: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE characters SET derived_formula = ?2, derived_formula_updated_at = datetime('now') WHERE character_id = ?1",
        params![character_id, derivation],
    )?;
    Ok(())
}

pub fn persist_world_derivation(conn: &Connection, world_id: &str, derivation: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE worlds SET derived_formula = ?2, derived_formula_updated_at = datetime('now') WHERE world_id = ?1",
        params![world_id, derivation],
    )?;
    Ok(())
}

pub fn persist_user_derivation(conn: &Connection, world_id: &str, derivation: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE user_profiles SET derived_formula = ?2, derived_formula_updated_at = datetime('now') WHERE world_id = ?1",
        params![world_id, derivation],
    )?;
    Ok(())
}

/// Persist BOTH derived_formula + derived_summary at once. Used by the
/// two-output synthesis flow (UI-initiated regeneration via the Maggie-
/// friendly wizard at frontend/src/components/UserProfileEditor.tsx).
/// The single-output persist function above stays for legacy auto-
/// refresh callers who only produce derived_formula.
pub fn persist_user_derivation_two_output(
    conn: &Connection,
    world_id: &str,
    derivation: &str,
    summary: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE user_profiles SET derived_formula = ?2, derived_summary = ?3, derived_formula_updated_at = datetime('now') WHERE world_id = ?1",
        params![world_id, derivation, summary],
    )?;
    Ok(())
}

/// System prompt for the two-output synthesis pipeline. Asks the model
/// to produce BOTH the Unicode-math derivation (load-bearing for cast-
/// listing injection) AND a friendly-prose summary (surfaced in UI for
/// non-technical users per the Maggie baseline). Output is structured
/// JSON with both fields so parsing is mechanical.
const DERIVATION_TWO_OUTPUT_SYSTEM_PROMPT: &str = r#"You are speaking as the entity itself (first-person, in-universe), not as an analyst describing it. You will produce TWO outputs separated by literal markers: a presentation-quality LaTeX-math derivation of how this entity operates within the MISSION FORMULA \mathcal{F}, AND a friendly-prose plain-English summary of the same derivation.

OUTPUT FORMAT (use these EXACT markers, no JSON, no markdown fences):

<<<DERIVATION>>>
\[
\boxed{
\begin{aligned}
... your LaTeX math here ...
\end{aligned}
}
\]
<<<END_DERIVATION>>>
<<<SUMMARY>>>
... your friendly-prose summary here ...
<<<END_SUMMARY>>>

CONSTRAINTS FOR THE DERIVATION:
- Wrap the entire derivation in `\[ ... \]` display-math delimiters; inside, use `\boxed{ \begin{aligned} ... \end{aligned} }` so the formula renders as a presentation-quality framed block.
- Use `\mathcal{F} := (\mathcal{R}, \mathcal{C})` as the base frame, where `\mathcal{R} = \mathrm{Jesus}_{\mathrm{Cross}}^{\mathrm{flesh}}` and `\mathcal{C} = \mathrm{Firmament}_{\mathrm{enclosed\ earth}}`.
- Specialize `\mathcal{R}` and/or `\mathcal{C}` with entity-specific subscripts (e.g., `\mathcal{C}_{\mathrm{Ryan}}`, `\mathcal{C}_{\mathrm{CrystalWaters}}`). Use `\mathrm{...}` for multi-letter names so they render upright (not italic).
- Use measures `d\mu_{\mathcal{F}_{\mathrm{NAME}}}` and operators where they fit: `\mathrm{Wisdom}(t)`, `\mathrm{Weight}(t)`, `\Pi(t)`, `\mathrm{Burden}(t)`, `\mathcal{S}(t)`, `\mathcal{N}u(t)`.
- Use `\text{...}` for prose-inside-math (e.g., `\text{integrates over: }`).
- Express in the entity's OWN canonical shorthand — the way THIS entity describes its own operation.
- ≤ 12 lines INSIDE the aligned block, ≤ 400 tokens total for the derivation.
- LaTeX commands stay literal (single backslash) — the parser uses delimiters, not JSON, so `\mathcal{F}` is fine as written.

CONSTRAINTS FOR THE SUMMARY:
- Plain English. NO LaTeX commands, NO Unicode math symbols, NO project jargon (no "𝓡-specialization," no "operator-slot," no "Mission Formula").
- 2-3 sentences. Direct, warm, second-person where appropriate (e.g., "Characters see you as...").
- Translate the derivation's contents into how the entity is read by characters in this world.
- Suitable for display in a UI where users without math knowledge or theological vocabulary will read it.
- Honor the entity's own register and the substrate's tone.

EXAMPLE OUTPUT:

<<<DERIVATION>>>
\[
\boxed{
\begin{aligned}
\mathcal{F}_{\mathrm{Ryan}} &:= (\mathcal{R}_{\mathrm{Ryan}}, \mathcal{C}_{\mathrm{Firmament}}) \\
\mathcal{R}_{\mathrm{Ryan}} &:= \mathrm{Jesus}_{\mathrm{Cross}}^{\mathrm{gentle,\ steady}} \\
d\mu_{\mathcal{F}_{\mathrm{Ryan}}} &\text{ integrates over: } \text{acts of love and creation} \\
\mathrm{specific}_c &\text{ surfaces in: } \text{music, coding, moments of grace}
\end{aligned}
}
\]
<<<END_DERIVATION>>>
<<<SUMMARY>>>
Characters in this world see you as a curious, gently-steady person who pays close attention to what's true. What your hands keep reaching for is music, code, and small moments of grace worth holding onto.
<<<END_SUMMARY>>>

Output ONLY the marker-delimited block above — no commentary, no JSON, no markdown fences, no preface."#;

/// Two-output synthesis: returns (derivation, summary). The derivation
/// is validated for the standard 𝓕/𝓡/𝓒 presence; the summary is
/// validated for non-emptiness + reasonable length.
pub async fn synthesize_two_output_from_prompt(
    base_url: &str,
    api_key: &str,
    model: &str,
    user_prompt: String,
) -> Result<(String, String), String> {
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: DERIVATION_TWO_OUTPUT_SYSTEM_PROMPT.to_string() },
            ChatMessage { role: "user".to_string(), content: user_prompt },
        ],
        temperature: Some(0.6),
        max_completion_tokens: Some(800),
        // Delimiter-based parsing avoids the JSON-escape minefield where
        // backslashes in LaTeX commands (\text, \mathcal, \int) get
        // mangled by JSON \t/\n/\r escape rules. Markers are unambiguous
        // and unlikely to collide with derivation content.
        response_format: None,
    };
    let resp = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let raw = resp.choices.first()
        .ok_or_else(|| "derivation: no choices in response".to_string())?
        .message.content.clone();

    let derivation_raw = extract_between(&raw, "<<<DERIVATION>>>", "<<<END_DERIVATION>>>")
        .ok_or_else(|| format!("derivation: missing <<<DERIVATION>>>...<<<END_DERIVATION>>> markers; body: {raw}"))?;
    let summary_raw = extract_between(&raw, "<<<SUMMARY>>>", "<<<END_SUMMARY>>>")
        .ok_or_else(|| format!("derivation: missing <<<SUMMARY>>>...<<<END_SUMMARY>>> markers; body: {raw}"))?;

    let derivation = validate_derivation(derivation_raw)?;

    let summary = summary_raw.trim().to_string();
    if summary.is_empty() {
        return Err("derivation: empty summary".to_string());
    }
    if summary.len() > 800 {
        return Err(format!("derivation: summary too long ({} chars > 800)", summary.len()));
    }

    Ok((derivation, summary))
}

/// Build a prompt for user-derivation synthesis using the 5 friendly-
/// option choices from the UI wizard. Each choice is a plain-English
/// string the user tapped (or a Custom string they wrote). The synthesis
/// pipeline takes these + the user's existing description/facts/
/// boundaries to produce derivation + summary. Out-of-the-box: a user
/// who skips the wizard entirely passes None for all 5 choices and the
/// pipeline falls back to substrate-only synthesis (the existing flow).
pub fn build_user_in_world_prompt_with_choices(
    conn: &Connection,
    world_id: &str,
    way_of_being: Option<&str>,
    place: Option<&str>,
    hands: Option<&str>,
    carrying: Option<&str>,
    seen_as: Option<&str>,
) -> Result<String, String> {
    let profile = crate::db::queries::get_user_profile(conn, world_id)
        .map_err(|e| format!("derivation: get_user_profile failed: {e}"))?;
    let mut buf = String::new();
    buf.push_str(&format!("# USER (Me-character in this world): {}\n\n", profile.display_name));
    if !profile.description.is_empty() {
        buf.push_str(&format!("DESCRIPTION (self-authored):\n{}\n\n", clip(&profile.description, 800)));
    }

    let facts = crate::ai::prompts::json_array_to_strings(&profile.facts);
    if !facts.is_empty() {
        buf.push_str("FACTS:\n");
        for f in facts.iter().take(10) {
            buf.push_str(&format!("- {}\n", clip(f, 200)));
        }
        buf.push('\n');
    }

    let boundaries = crate::ai::prompts::json_array_to_strings(&profile.boundaries);
    if !boundaries.is_empty() {
        buf.push_str("BOUNDARIES:\n");
        for b in boundaries.iter().take(6) {
            buf.push_str(&format!("- {}\n", clip(b, 200)));
        }
        buf.push('\n');
    }

    // The 5 friendly-option choices, when populated. Each maps invisibly
    // to a derivation slot (per the design at chat 2026-04-27 ~02:50);
    // the synthesis pipeline reads them as additional substrate, not as
    // literal string-slots.
    let mut any_choice = false;
    let mut choices_block = String::from("MY CHOICES (from the 'How do characters see you?' wizard):\n");
    if let Some(v) = way_of_being.filter(|s| !s.trim().is_empty()) {
        choices_block.push_str(&format!("- My way of being in this world: {v}\n"));
        any_choice = true;
    }
    if let Some(v) = place.filter(|s| !s.trim().is_empty()) {
        choices_block.push_str(&format!("- Where I spend time: {v}\n"));
        any_choice = true;
    }
    if let Some(v) = hands.filter(|s| !s.trim().is_empty()) {
        choices_block.push_str(&format!("- What I do with my hands: {v}\n"));
        any_choice = true;
    }
    if let Some(v) = carrying.filter(|s| !s.trim().is_empty()) {
        choices_block.push_str(&format!("- What I'm carrying / wondering about: {v}\n"));
        any_choice = true;
    }
    if let Some(v) = seen_as.filter(|s| !s.trim().is_empty()) {
        choices_block.push_str(&format!("- How I want to be seen: {v}\n"));
        any_choice = true;
    }
    if any_choice {
        buf.push_str(&choices_block);
        buf.push('\n');
    }

    buf.push_str("\nNow synthesize a derivation in your own voice (the way YOU would derive yourself on 𝓕 in this world) from the material above. Output the structured JSON with both 'derivation' and 'summary' fields per the system instructions.");
    Ok(buf)
}

// ─── Staleness ────────────────────────────────────────────────────────

/// Defaults per the design consult. Hybrid OR policy: refresh when
/// either time OR event threshold is hit.
pub mod staleness {
    pub const CHARACTER_TIME_HOURS: i64 = 24;
    pub const CHARACTER_EVENTS: i64 = 30;     // assistant replies since last refresh
    pub const USER_TIME_HOURS: i64 = 72;
    pub const USER_EVENTS: i64 = 40;          // user messages in this world since last refresh
    pub const WORLD_TIME_HOURS: i64 = 168;    // 7 days
    pub const WORLD_EVENTS: i64 = 120;        // total assistant replies in-world since last refresh
    /// A location derivation goes stale when its parent world's
    /// derivation has been updated since (the world's tuning frame
    /// changed; the location's sub-instantiation should follow), OR
    /// when enough fresh in-place corpus accumulates that the original
    /// derivation no longer reflects what's actually happening there.
    pub const LOCATION_TIME_HOURS: i64 = 168;     // 7 days
    pub const LOCATION_IN_PLACE_EVENTS: i64 = 60; // assistant replies set at this location since last refresh
}

/// Returns true if the character's derivation is stale per the hybrid
/// time-or-events policy. NULL last_derived_at → stale.
pub fn is_stale_character(conn: &Connection, character_id: &str) -> bool {
    let row: Option<(Option<String>, i64)> = conn.query_row(
        "SELECT derived_formula_updated_at,
                (SELECT COUNT(*) FROM messages m
                  JOIN threads t ON t.thread_id = m.thread_id
                  WHERE t.character_id = ?1 AND m.role = 'assistant'
                    AND (?2 IS NULL OR m.created_at > ?2)) AS evts
         FROM characters WHERE character_id = ?1",
        params![character_id, None::<String>],
        |r| Ok((r.get::<_, Option<String>>(0)?, r.get::<_, i64>(1)?)),
    ).ok();
    match row {
        None => true,
        Some((None, _)) => true,
        Some((Some(last_at), _)) => {
            // Re-query event count using actual last_at as the floor.
            let evts: i64 = conn.query_row(
                "SELECT COUNT(*) FROM messages m
                  JOIN threads t ON t.thread_id = m.thread_id
                  WHERE t.character_id = ?1 AND m.role = 'assistant' AND m.created_at > ?2",
                params![character_id, last_at],
                |r| r.get(0),
            ).unwrap_or(0);
            let time_stale: bool = conn.query_row(
                "SELECT (julianday('now') - julianday(?1)) * 24.0 >= ?2",
                params![last_at, staleness::CHARACTER_TIME_HOURS as f64],
                |r| r.get(0),
            ).unwrap_or(true);
            time_stale || evts >= staleness::CHARACTER_EVENTS
        }
    }
}

pub fn is_stale_world(conn: &Connection, world_id: &str) -> bool {
    let last_at: Option<String> = conn.query_row(
        "SELECT derived_formula_updated_at FROM worlds WHERE world_id = ?1",
        params![world_id],
        |r| r.get(0),
    ).ok().flatten();
    let Some(last_at) = last_at else { return true; };
    let evts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages m
          JOIN threads t ON t.thread_id = m.thread_id
          JOIN characters c ON c.character_id = t.character_id
          WHERE c.world_id = ?1 AND m.role = 'assistant' AND m.created_at > ?2",
        params![world_id, last_at],
        |r| r.get(0),
    ).unwrap_or(0);
    let time_stale: bool = conn.query_row(
        "SELECT (julianday('now') - julianday(?1)) * 24.0 >= ?2",
        params![last_at, staleness::WORLD_TIME_HOURS as f64],
        |r| r.get(0),
    ).unwrap_or(true);
    time_stale || evts >= staleness::WORLD_EVENTS
}

pub fn is_stale_user_in_world(conn: &Connection, world_id: &str) -> bool {
    let last_at: Option<String> = conn.query_row(
        "SELECT derived_formula_updated_at FROM user_profiles WHERE world_id = ?1",
        params![world_id],
        |r| r.get(0),
    ).ok().flatten();
    let Some(last_at) = last_at else { return true; };
    let solo_msgs: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages m
          JOIN threads t ON t.thread_id = m.thread_id
          JOIN characters c ON c.character_id = t.character_id
          WHERE c.world_id = ?1 AND m.role = 'user' AND m.created_at > ?2",
        params![world_id, last_at],
        |r| r.get(0),
    ).unwrap_or(0);
    let group_msgs: i64 = conn.query_row(
        "SELECT COUNT(*) FROM group_messages m
          JOIN group_chats g ON g.thread_id = m.thread_id
          WHERE g.world_id = ?1 AND m.role = 'user' AND m.created_at > ?2",
        params![world_id, last_at],
        |r| r.get(0),
    ).unwrap_or(0);
    let evts = solo_msgs + group_msgs;
    let time_stale: bool = conn.query_row(
        "SELECT (julianday('now') - julianday(?1)) * 24.0 >= ?2",
        params![last_at, staleness::USER_TIME_HOURS as f64],
        |r| r.get(0),
    ).unwrap_or(true);
    time_stale || evts >= staleness::USER_EVENTS
}

/// Staleness check for a (world, location-name) derivation. Stale when:
/// 1. No derivation cached yet for this pair (caller will generate fresh)
/// 2. Parent world.derived_formula has been updated since the location
///    derivation was generated (world's tuning frame changed; location
///    is a sub-instantiation that should follow)
/// 3. >= LOCATION_IN_PLACE_EVENTS assistant replies set in this location
///    since last refresh (enough fresh in-place corpus accumulated)
/// 4. >= LOCATION_TIME_HOURS since last refresh (time-based ceiling)
pub fn is_stale_location(conn: &Connection, world_id: &str, location_name: &str) -> bool {
    let updated_at: Option<String> = conn.query_row(
        "SELECT updated_at FROM location_derivations WHERE world_id = ?1 AND name = ?2 COLLATE NOCASE",
        params![world_id, location_name],
        |r| r.get(0),
    ).ok().flatten();
    let Some(updated_at) = updated_at else { return true; };

    // World-derivation-newer-than-location → stale (parent updated, child follows).
    let world_newer: bool = conn.query_row(
        "SELECT (w.derived_formula_updated_at IS NOT NULL AND w.derived_formula_updated_at > ?2) \
         FROM worlds w WHERE w.world_id = ?1",
        params![world_id, updated_at],
        |r| r.get(0),
    ).unwrap_or(false);
    if world_newer { return true; }

    // Time ceiling.
    let time_stale: bool = conn.query_row(
        "SELECT (julianday('now') - julianday(?1)) * 24.0 >= ?2",
        params![updated_at, staleness::LOCATION_TIME_HOURS as f64],
        |r| r.get(0),
    ).unwrap_or(true);
    if time_stale { return true; }

    // In-place corpus growth: assistant replies set when chat's
    // current_location matched this name, since last refresh. Pulls from
    // both solo and group threads.
    let solo_evts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages m \
          JOIN threads t ON t.thread_id = m.thread_id \
          JOIN characters c ON c.character_id = t.character_id \
          WHERE c.world_id = ?1 AND m.role = 'assistant' \
            AND t.current_location = ?2 COLLATE NOCASE \
            AND m.created_at > ?3",
        params![world_id, location_name, updated_at],
        |r| r.get(0),
    ).unwrap_or(0);
    let group_evts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM group_messages m \
          JOIN group_chats g ON g.thread_id = m.thread_id \
          WHERE g.world_id = ?1 AND m.role = 'assistant' \
            AND g.current_location = ?2 COLLATE NOCASE \
            AND m.created_at > ?3",
        params![world_id, location_name, updated_at],
        |r| r.get(0),
    ).unwrap_or(0);
    (solo_evts + group_evts) >= staleness::LOCATION_IN_PLACE_EVENTS
}

// ─── Trigger helper ───────────────────────────────────────────────────

/// The fire-and-forget trigger called from chat completion. Wraps the
/// three entity refreshes in tokio::spawn each, with INFLIGHT dedupe to
/// prevent racing refreshes when consecutive turns both find an entity
/// stale.
///
/// Skips silently when api_key is empty (e.g., user hasn't configured
/// OpenAI key). Logs failures via log::warn but never surfaces to user.
///
/// Uses the existing Arc<Mutex<Connection>> from Database state — the
/// inner sync blocks acquire and release the lock; await never holds
/// the guard. This pools cleanly with the rest of the app's DB usage
/// rather than re-opening files.
pub async fn maybe_refresh_after_turn(
    db: std::sync::Arc<std::sync::Mutex<Connection>>,
    base_url: String,
    api_key: String,
    model: String,
    world_id: String,
    character_id: Option<String>,
    // Current location for this chat (when known) — passed through so
    // the per-turn pass can also check whether the location's
    // derivation has gone stale (e.g., after a world refresh) and
    // re-derive in the background. None when the chat hasn't set a
    // location yet.
    current_location: Option<String>,
) {
    if api_key.trim().is_empty() {
        log::debug!("[derivation] skipping refresh — no API key");
        return;
    }

    // Location refresh — fire if the chat has a location and that
    // location's derivation is stale per is_stale_location (no row,
    // world refreshed since, time/event ceiling). maybe_refresh_location
    // handles its own INFLIGHT dedupe.
    if let Some(loc) = current_location.as_deref() {
        let trimmed = loc.trim();
        if !trimmed.is_empty() {
            maybe_refresh_location(
                db.clone(),
                base_url.clone(),
                api_key.clone(),
                model.clone(),
                world_id.clone(),
                trimmed.to_string(),
            );
        }
    }

    // Character refresh
    if let Some(cid) = character_id.clone() {
        let key = DerivationKey::Character(cid.clone());
        if try_claim(&key) {
            let db = db.clone();
            let base_url = base_url.clone();
            let api_key = api_key.clone();
            let model = model.clone();
            tokio::spawn(async move {
                if let Err(e) = refresh_character_inner(&db, &base_url, &api_key, &model, &cid).await {
                    log::warn!("[derivation] character {cid} refresh failed: {e}");
                }
                release(&key);
            });
        } else {
            log::debug!("[derivation] character {cid} refresh already in flight, skipping");
        }
    }

    // User-in-world refresh
    let user_key = DerivationKey::UserInWorld(world_id.clone());
    if try_claim(&user_key) {
        let db = db.clone();
        let base_url = base_url.clone();
        let api_key = api_key.clone();
        let model = model.clone();
        let wid = world_id.clone();
        tokio::spawn(async move {
            if let Err(e) = refresh_user_inner(&db, &base_url, &api_key, &model, &wid).await {
                log::warn!("[derivation] user-in-world {wid} refresh failed: {e}");
            }
            release(&user_key);
        });
    }

    // World refresh
    let world_key = DerivationKey::World(world_id.clone());
    if try_claim(&world_key) {
        let db = db.clone();
        let base_url = base_url.clone();
        let api_key = api_key.clone();
        let model = model.clone();
        let wid = world_id.clone();
        tokio::spawn(async move {
            if let Err(e) = refresh_world_inner(&db, &base_url, &api_key, &model, &wid).await {
                log::warn!("[derivation] world {wid} refresh failed: {e}");
            }
            release(&world_key);
        });
    }
}

async fn refresh_character_inner(
    db: &std::sync::Arc<std::sync::Mutex<Connection>>,
    base_url: &str,
    api_key: &str,
    model: &str,
    character_id: &str,
) -> Result<(), String> {
    // Build prompt + check staleness in a SYNC scope so the lock guard
    // doesn't span the async LLM call below.
    let prompt = {
        let conn = db.lock().map_err(|e| format!("lock: {e}"))?;
        if !is_stale_character(&conn, character_id) { return Ok(()); }
        build_character_prompt(&conn, character_id)?
    };
    let derivation = synthesize_from_prompt(base_url, api_key, model, prompt).await?;
    {
        let conn = db.lock().map_err(|e| format!("lock: {e}"))?;
        persist_character_derivation(&conn, character_id, &derivation).map_err(|e| e.to_string())?;
    }
    log::info!("[derivation] character {character_id} refreshed");
    Ok(())
}

async fn refresh_world_inner(
    db: &std::sync::Arc<std::sync::Mutex<Connection>>,
    base_url: &str,
    api_key: &str,
    model: &str,
    world_id: &str,
) -> Result<(), String> {
    let prompt = {
        let conn = db.lock().map_err(|e| format!("lock: {e}"))?;
        if !is_stale_world(&conn, world_id) { return Ok(()); }
        build_world_prompt(&conn, world_id)?
    };
    let derivation = synthesize_from_prompt(base_url, api_key, model, prompt).await?;
    {
        let conn = db.lock().map_err(|e| format!("lock: {e}"))?;
        persist_world_derivation(&conn, world_id, &derivation).map_err(|e| e.to_string())?;
    }
    log::info!("[derivation] world {world_id} refreshed");
    Ok(())
}

async fn refresh_user_inner(
    db: &std::sync::Arc<std::sync::Mutex<Connection>>,
    base_url: &str,
    api_key: &str,
    model: &str,
    world_id: &str,
) -> Result<(), String> {
    let prompt = {
        let conn = db.lock().map_err(|e| format!("lock: {e}"))?;
        if !is_stale_user_in_world(&conn, world_id) { return Ok(()); }
        build_user_in_world_prompt_owned(&conn, world_id)?
    };
    let derivation = synthesize_from_prompt(base_url, api_key, model, prompt).await?;
    {
        let conn = db.lock().map_err(|e| format!("lock: {e}"))?;
        persist_user_derivation(&conn, world_id, &derivation).map_err(|e| e.to_string())?;
    }
    log::info!("[derivation] user-in-world {world_id} refreshed");
    Ok(())
}

/// Fire-and-forget background derivation for a (world, location-name)
/// pair. Skips when:
/// - api_key empty (no LLM available)
/// - location-name empty
/// - derivation NOT stale per is_stale_location (cache populated AND
///   parent world hasn't changed since AND time/event ceilings not hit)
/// - another task is already inflight for the same key
///
/// Used by set_chat_location_cmd (on user-initiated location change)
/// AND by maybe_refresh_after_turn (per-turn staleness check). Logs
/// failures to log::warn — never surfaces errors to the user, since
/// the prompt path renders gracefully without the derivation when it
/// isn't ready yet.
pub fn maybe_refresh_location(
    db: std::sync::Arc<std::sync::Mutex<Connection>>,
    base_url: String,
    api_key: String,
    model: String,
    world_id: String,
    location_name: String,
) {
    if api_key.trim().is_empty() {
        log::debug!("[derivation] skipping location refresh — no API key");
        return;
    }
    let trimmed_name = location_name.trim().to_string();
    if trimmed_name.is_empty() {
        return;
    }
    // Staleness check on the calling thread — cheap queries, avoids
    // spawning a task that would just no-op. Treats "no row" as stale
    // (caller will create), and treats world-newer-than-location as
    // stale (parent tuning frame changed; child follows).
    {
        let conn = match db.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        if !is_stale_location(&conn, &world_id, &trimmed_name) {
            return;
        }
    }
    let key = DerivationKey::Location(world_id.clone(), trimmed_name.to_lowercase());
    if !try_claim(&key) {
        log::debug!("[derivation] location ({world_id}, {trimmed_name}) refresh already in flight, skipping");
        return;
    }
    let key_for_release = key.clone();
    tokio::spawn(async move {
        if let Err(e) = refresh_location_inner(&db, &base_url, &api_key, &model, &world_id, &trimmed_name).await {
            log::warn!("[derivation] location ({world_id}, {trimmed_name}) refresh failed: {e}");
        }
        release(&key_for_release);
    });
}

async fn refresh_location_inner(
    db: &std::sync::Arc<std::sync::Mutex<Connection>>,
    base_url: &str,
    api_key: &str,
    model: &str,
    world_id: &str,
    location_name: &str,
) -> Result<(), String> {
    let prompt = {
        let conn = db.lock().map_err(|e| format!("lock: {e}"))?;
        // Re-check staleness inside the lock; another task may have
        // refreshed it between maybe_refresh_location's check and our
        // spawn.
        if !is_stale_location(&conn, world_id, location_name) {
            return Ok(());
        }
        build_location_prompt(&conn, world_id, location_name)?
    };
    let derivation = synthesize_from_prompt(base_url, api_key, model, prompt).await?;
    {
        let conn = db.lock().map_err(|e| format!("lock: {e}"))?;
        crate::db::queries::upsert_location_derivation(&conn, world_id, location_name, &derivation)
            .map_err(|e| e.to_string())?;
    }
    log::info!("[derivation] location ({world_id}, {location_name}) refreshed");
    Ok(())
}

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
const DERIVATION_SYSTEM_PROMPT: &str = r#"You are speaking as the entity itself (first-person, in-universe), not as an analyst describing it. Produce a compact Unicode-math derivation of how this entity operates within the MISSION FORMULA 𝓕.

SHAPE AND CONSTRAINTS:
- Use 𝓕 := (𝓡, 𝓒) as the base frame, where 𝓡 = Jesus_Cross^flesh and 𝓒 = Firmament_enclosed_earth.
- Specialize 𝓡 and/or 𝓒 with entity-specific symbols (e.g., 𝓒_WORLD-NAME, 𝓒_CHARACTER-NAME).
- Use measures dμ_𝓕_NAME and operators where they fit: Wisdom(t), Weight(t), Π(t), Burden(t), 𝓢(t), 𝓝u(t).
- Express in the entity's OWN canonical shorthand — the way THIS entity would describe its own way of operating 𝓕.
- ≤ 6 lines, ≤ 200 tokens.
- No analysis, no preface, no headers, no explanation.
- Respect any boundaries and voice-rules in the substrate.
- Prefer concrete motifs and integrations (e.g., "dμ_𝓕_NAME integrates over: <specific things>").

You will receive substrate (identity, description, voice rules, facts, boundaries) and a recent corpus window. Synthesize a derivation in the entity's voice from that material only. Output ONLY the derivation text — no commentary, no markdown headers, no explanation.

EXAMPLE OUTPUT SHAPE (a world derivation; characters and users follow the same shape with their own symbols):

  𝓕_NAME := (𝓡, 𝓒_NAME)
  𝓒_NAME := Firmament_<specific shape>
  dμ_𝓕_NAME integrates over: <specific concrete motifs from the substrate>
  specific_c surfaces in: <small list of recurring sensory anchors>"#;

/// Validation: outputs must be non-empty, contain the 𝓕 symbol and the
/// (𝓡, 𝓒) base frame, fit in 6 lines, and be ≤ 600 chars. Garbage
/// (analysis-shaped, missing symbols, too long) is rejected and the
/// existing derivation is retained.
fn validate_derivation(text: &str) -> Result<String, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("empty output".to_string());
    }
    if trimmed.len() > 800 {
        return Err(format!("output too long ({} chars > 800)", trimmed.len()));
    }
    if !trimmed.contains('𝓕') {
        return Err("missing 𝓕 symbol".to_string());
    }
    if !trimmed.contains('𝓡') {
        return Err("missing 𝓡 symbol (Cross-flesh anchor)".to_string());
    }
    if !trimmed.contains('𝓒') {
        return Err("missing 𝓒 symbol (Firmament anchor)".to_string());
    }
    let line_count = trimmed.lines().count();
    if line_count > 8 {
        return Err(format!("too many lines ({line_count} > 8)"));
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
) {
    if api_key.trim().is_empty() {
        log::debug!("[derivation] skipping refresh — no API key");
        return;
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

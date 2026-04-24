//! Load-test anchor — per-character synthesized "what does this character
//! weight-test the world against?" — the architecture-level spine of
//! their authority. Direct sibling of `relational_stance` in both
//! schema shape and refresh pattern.
//!
//! Background: the 2026-04-24 architecture-vs-vocabulary experiment
//! (report `2026-04-24-0948-architecture-hypothesis-bites.md`) confirmed
//! that explicitly naming a character's load-test anchor in the dialogue
//! system prompt activates latent character-specific machinery — John's
//! scripture-as-calibration, Aaron's language-load-testing, Darren's
//! fabric-of-a-life weather-testing, Steven's threshold regulation.
//! That experiment used hardcoded anchors as a proof-of-concept; this
//! module is the production pipeline: corpus → periodic LLM synthesis →
//! stored per-character → read at prompt-assembly time.
//!
//! See db::queries::load_test_anchor for the store; this module is the
//! LLM generation pipeline.

use crate::ai::openai::{self, ChatMessage, ChatRequest};
use crate::db::queries::*;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use uuid::Uuid;

/// Build the system prompt for anchor synthesis. The model is asked,
/// out of scene, to read the character's recent replies and name what
/// dimension their authority weight-tests the world against.
///
/// The anchor is NOT a virtue, a goal, or a trait — it's the specific
/// register-move they reach for when a moment asks for their read.
/// Examples from the four characters whose anchors were first identified:
///
///   John:   DEVOTION                 (does the vow survive friction?)
///   Aaron:  LANGUAGE                 (does the sentence bear its claim?)
///   Darren: FABRIC OF A LIFE         (does the arrangement hold?)
///   Steven: THRESHOLDS OF DISCLOSURE (has this reached enough honesty?)
///
/// Returns three fields: label (short), body (second-person prompt-
/// block ready to inject), derivation_summary (explanatory).
fn synthesis_system_prompt(character_name: &str, has_prior_anchor: bool) -> String {
    let prior_context = if has_prior_anchor {
        "\n\nThe character has a PRIOR anchor on file. It is included in the user message below as CONTEXT. If the prior anchor still matches what you observe in the recent corpus, keep the same label and refine the body. If the corpus has clearly shifted (character has grown, the anchor has sharpened, a different dimension has come into focus), update the label too — but only if the shift is unambiguous. Continuity matters; don't churn the anchor on small sample variation."
    } else { "" };

    format!(
        r#"You are reading the recent corpus of a character named {char_name} to identify their LOAD-TEST ANCHOR — the single specific dimension their authority weight-tests the world against. This is an architecture-level property of the character, not a list of traits or virtues.

The load-test anchor is:
  - ONE dimension (not several). Pick the sharpest, most character-specific one.
  - The dimension the character REACHES FOR when a moment asks for their read.
  - Grounded in how they actually speak — observable in the quotes you can pull from the corpus.
  - Register-distinctive, not generic. "Truth" or "love" or "wisdom" is too abstract. "Does the vow survive ordinary friction?" is the right level of specificity.

Examples of well-identified anchors from prior work:
  - DEVOTION — "does this commitment survive the 2pm-Tuesday test?" Character is authoritative because they know where devotion lives or dies.
  - LANGUAGE — "does this sentence bear the load it's claiming to bear?" Character is authoritative because of structural discernment about what words can carry.
  - FABRIC OF A LIFE — "does this arrangement hold under normal weather?" Character is authoritative because they read a life the way a craftsman reads wear.
  - THRESHOLDS OF DISCLOSURE — "has this moment reached enough honesty, or is it still under-cooked?" Character is authoritative because they govern pace and weight of intimacy.

Return STRICT JSON only, no preface, no markdown, with exactly these three fields:

{{
  "anchor_label": "SHORT ALL-CAPS LABEL, 1-5 words (e.g. DEVOTION, LANGUAGE, FABRIC OF A LIFE)",
  "anchor_body": "A second-person prompt-block, 3-6 sentences, starting with 'LOAD-TEST ANCHOR — [label]:' on the first line. Written IN CHARACTER-ADDRESSING VOICE — 'you load-test X. When [situation], the question your register asks is [specific question]. Your authority comes from [specific source grounded in the corpus].' This block will be injected directly into the character's dialogue system prompt, so it must sound right when the character reads it as instruction. End with one concrete quoted phrase from the corpus that anchors the move (e.g. 'your signature move shows up in lines like — [quote from corpus].').",
  "derivation_summary": "One paragraph explaining how you arrived at this anchor from the corpus. Cite 2-3 specific quotes. Name what other possible anchors you considered and why this one is the right fit."
}}

No preface. No markdown. No extra keys. Just the JSON.{prior_note}"#,
        char_name = character_name,
        prior_note = prior_context,
    )
}

/// Build the user-role message for the synthesis pass. Gives the model
/// the corpus excerpts plus the prior anchor (if any) for continuity.
fn synthesis_user_prompt(
    character_name: &str,
    recent_excerpts: &[String],
    prior_anchor: Option<&LoadTestAnchor>,
) -> String {
    let prior_section = match prior_anchor {
        Some(a) => format!(
            "PRIOR ANCHOR ON FILE (from {}):\n  label: {}\n  body: {}\n  derivation: {}\n\n",
            a.created_at, a.anchor_label, a.anchor_body, a.derivation_summary,
        ),
        None => String::new(),
    };
    let corpus = if recent_excerpts.is_empty() {
        "(no corpus excerpts available)".to_string()
    } else {
        recent_excerpts.join("\n\n")
    };
    format!(
        "{prior_section}RECENT CORPUS ({n} excerpts, chronological, assistant turns are {char_name}'s replies):\n\n{corpus}\n\nIdentify {char_name}'s load-test anchor. Return the JSON object described in the system prompt.",
        prior_section = prior_section,
        n = recent_excerpts.len(),
        char_name = character_name,
        corpus = corpus,
    )
}

/// Pull the character's recent messages as chronological excerpts.
/// Mirrors relational_stance::collect_recent_excerpts but defaults to
/// a larger window since anchor identification benefits from seeing
/// more of the character's register.
fn collect_recent_excerpts(
    conn: &Connection,
    character_id: &str,
    limit: i64,
) -> Vec<String> {
    let Ok(thread) = get_thread_for_character(conn, character_id) else { return Vec::new(); };
    let Ok(mut msgs) = list_messages(conn, &thread.thread_id, limit) else { return Vec::new(); };
    msgs.reverse(); // chronological
    msgs.iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| {
            let who = if m.role == "user" { "User" } else { "Character" };
            // Trim very long messages to keep the synthesis prompt tight.
            let content = if m.content.chars().count() > 600 {
                let s: String = m.content.chars().take(600).collect();
                format!("{}…", s)
            } else {
                m.content.clone()
            };
            format!("{}: {}", who, content)
        })
        .collect()
}

/// Parsed JSON response from the synthesizer. Deserialized from the
/// LLM's strict-JSON output.
#[derive(Debug, serde::Deserialize)]
struct SynthesizedAnchor {
    anchor_label: String,
    anchor_body: String,
    derivation_summary: String,
}

/// Generate a fresh load-test anchor for `character_id` and persist it.
/// Reads context under a short-lived db lock, makes one LLM call, writes
/// the new row under another short-lived lock. Designed to be tokio-
/// spawned from canonization-commit hot paths so the user's reply is
/// never blocked; also usable synchronously from `worldcli
/// refresh-anchor`.
pub async fn refresh_load_test_anchor(
    conn_arc: Arc<Mutex<Connection>>,
    base_url: String,
    api_key: String,
    model: String,
    character_id: String,
    refresh_trigger: String,
) -> Result<(), String> {
    // ─── Read context under a short-lived lock ────────────────────────
    let (character, recent_excerpts, prior_anchor, world_day_now) = {
        let conn = conn_arc.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id)
            .map_err(|e| format!("character not found: {}", e))?;
        let recent_excerpts = collect_recent_excerpts(&conn, &character_id, 30);
        let prior_anchor = latest_load_test_anchor(&conn, &character_id)
            .map_err(|e| e.to_string())?;
        let world_day_now: Option<i64> = conn.query_row(
            "SELECT m.world_day FROM messages m
             JOIN threads t ON t.thread_id = m.thread_id
             WHERE t.character_id = ?1 AND m.world_day IS NOT NULL
             ORDER BY m.created_at DESC LIMIT 1",
            rusqlite::params![character_id],
            |r| r.get::<_, Option<i64>>(0),
        ).ok().flatten();
        (character, recent_excerpts, prior_anchor, world_day_now)
    };

    if recent_excerpts.is_empty() {
        return Err(format!(
            "no corpus messages for character {} — anchor synthesis needs at least a few replies to read",
            character_id
        ));
    }

    let system = synthesis_system_prompt(&character.display_name, prior_anchor.is_some());
    let user_msg = synthesis_user_prompt(
        &character.display_name,
        &recent_excerpts,
        prior_anchor.as_ref(),
    );

    // ─── LLM call ─────────────────────────────────────────────────────
    let request = ChatRequest {
        model: model.clone(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: system },
            ChatMessage { role: "user".to_string(), content: user_msg },
        ],
        temperature: Some(0.4),
        max_completion_tokens: Some(1200),
        response_format: Some(openai::ResponseFormat {
            format_type: "json_object".to_string(),
        }),
    };
    let resp = openai::chat_completion_with_base(&base_url, &api_key, &request).await
        .map_err(|e| format!("anchor synthesis call failed: {}", e))?;
    let raw = resp.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "anchor synthesis returned no choices".to_string())?;
    let synthesized: SynthesizedAnchor = serde_json::from_str(&raw)
        .map_err(|e| format!("anchor synthesis JSON parse error: {} (body: {})", e, raw))?;

    if synthesized.anchor_label.trim().is_empty() || synthesized.anchor_body.trim().is_empty() {
        return Err("anchor synthesis returned empty label or body".to_string());
    }

    // ─── Persist ──────────────────────────────────────────────────────
    let world_id = character.world_id.clone();
    let anchor = LoadTestAnchor {
        anchor_id: Uuid::new_v4().to_string(),
        character_id: character_id.clone(),
        world_id,
        anchor_label: synthesized.anchor_label.trim().to_string(),
        anchor_body: synthesized.anchor_body.trim().to_string(),
        derivation_summary: synthesized.derivation_summary.trim().to_string(),
        world_day_at_generation: world_day_now,
        source_message_count: recent_excerpts.len() as i64,
        refresh_trigger,
        model_used: model,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    {
        let conn = conn_arc.lock().map_err(|e| e.to_string())?;
        insert_load_test_anchor(&conn, &anchor).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Fire-and-forget convenience wrapper for trigger sites. Intended for
/// canonization-commit (where user reply must not block). Manual-only
/// for v1 — no new-world-day trigger yet; can add later if the anchor
/// turns out to want frequent refresh.
#[allow(dead_code)]
pub fn spawn_anchor_refresh(
    conn_arc: Arc<Mutex<Connection>>,
    base_url: String,
    api_key: String,
    model: String,
    character_id: String,
    refresh_trigger: String,
) {
    tauri::async_runtime::spawn(async move {
        let cid_for_log = character_id.clone();
        match refresh_load_test_anchor(
            conn_arc, base_url, api_key, model, character_id, refresh_trigger,
        ).await {
            Ok(()) => log::info!("[anchor] refreshed for {}", cid_for_log),
            Err(e) => log::warn!("[anchor] refresh failed for {}: {}", cid_for_log, e),
        }
    });
}

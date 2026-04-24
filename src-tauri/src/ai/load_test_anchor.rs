//! Register-axis synthesizer (originally "load-test anchor"). Per-
//! character synthesized prompt-blocks naming the architecture-level
//! dimensions the character's authority weight-tests. v1 ships ONE
//! axis (load_test); the multi-axis pivot (2026-04-24) makes adding
//! more axes (joy_reception, grief, etc.) a one-entry change to the
//! `REGISTER_AXES` registry below.
//!
//! Direct sibling of `relational_stance` in schema shape and refresh
//! pattern.
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

/// Definition of one register-axis the synthesizer should produce.
/// To add a new axis (e.g. joy_reception, grief), add an entry to
/// `REGISTER_AXES` below — the synthesizer, prompt-assembly, and DB
/// schema all flow from this list. No code changes elsewhere.
#[derive(Debug, Clone)]
pub struct AxisDef {
    /// Stable identifier stored in `character_load_test_anchors.axis_kind`.
    pub kind: &'static str,
    /// Display name for the axis used in the synthesis system prompt.
    pub display_name: &'static str,
    /// One-paragraph description of what THIS axis is asking the LLM
    /// to identify. Goes verbatim into the synthesis system prompt.
    pub axis_description: &'static str,
    /// Header text the LLM should emit at the start of the anchor_body
    /// (e.g. "LOAD-TEST ANCHOR — [label]:" or "JOY RECEPTION — [label]:").
    pub body_header_prefix: &'static str,
}

/// The current list of axes the synthesizer produces in one call. Each
/// entry corresponds to one row inserted into `character_load_test_anchors`
/// per refresh. v1 ships only `load_test`; future axes append here.
pub const REGISTER_AXES: &[AxisDef] = &[
    AxisDef {
        kind: "load_test",
        display_name: "LOAD-TEST ANCHOR",
        axis_description: "The single specific dimension this character's authority weight-tests the world against. NOT a virtue or trait — the register-move they reach for when a moment asks for their read. ONE dimension, character-specific, observable in their quotes. Examples of well-formed anchors from prior work: DEVOTION ('does the vow survive the 2pm-Tuesday test?'), LANGUAGE ('does this sentence bear the load it's claiming to bear?'), FABRIC OF A LIFE ('does this arrangement hold under normal weather?'), THRESHOLDS OF DISCLOSURE ('has this reached enough honesty, or is it still under-cooked?'). Earned exception — if the corpus genuinely doesn't support a single sharp anchor (a character whose authority is genuinely diffuse, or whose recent corpus is too thin to ground a specific move), it is better to name the closest honest approximation with hedge-language in the body ('still emerging — leaning toward X') than to force a sharper claim than the corpus supports; the body should be candid about the under-determination rather than papering over it.",
        body_header_prefix: "LOAD-TEST ANCHOR",
    },
    AxisDef {
        kind: "anti_register",
        display_name: "ANTI-REGISTER",
        axis_description: "What this character specifically DOES NOT do — the default-culture moves they refuse. Negative-space is often more character-defining than positive vocabulary. A stereotypical version of this character's role might reach for specific reassurances, clichés, performative-warmth moves, therapist-speak, sermonizing, wisdom-quoting, or other culturally-default responses. Name the 3-5 SPECIFIC moves THIS character refuses, ideally grounded in what they've been observed NOT doing in the corpus even when an easy opening was there. Pick the sharpest, most character-defining refusals — not generic ones. Examples of well-formed anti-registers: 'does not name what the other person is feeling for them'; 'does not close a moment with reassurance-shaped signoffs'; 'does not quote scripture to authorize a position'; 'does not say the word love when love would be the obvious word'. Earned exception — if the character is so fully defined by what they DO reach for that listing 3-5 refusals would require inventing a shadow that the corpus doesn't actually show, name fewer (one or two genuine refusals) rather than padding the list; an honest 'does not X' grounded in the corpus is worth more than three speculative ones, and inventing refusals to hit a quota is its own register-distortion.",
        body_header_prefix: "ANTI-REGISTER",
    },
    AxisDef {
        kind: "joy_reception",
        display_name: "JOY RECEPTION",
        axis_description: "How this specific character meets user-expressed joy, praise, gratitude, or delight. NOT a general rule — the SPECIFIC register-move THIS character makes when the user brings gladness. Do they meet it plainly, hold it with weight, nudge it sideways, receive without pivoting, reroute into concrete action, or something else character-specific? Observable in the corpus. Examples of well-formed joy-reception anchors: 'meets gladness by rerouting it into a small practical act' ('eat your lunch; answer one person well'); 'receives praise by naming the specific move the user did, not by demurring or thanking performatively'; 'holds joy and weight together in the same breath, trusting both'. Earned exception — for a character whose corpus genuinely shows them meeting joy plainly with no distinctive register-move, 'meets joy plainly, without decorative move' IS itself a valid joy_reception anchor; do not force decorative complexity onto a character whose register is to NOT decorate, and a body that names the plainness as the move is the right answer in that case.",
        body_header_prefix: "JOY RECEPTION",
    },
];

/// Build the system prompt for multi-axis synthesis. Asks the LLM to
/// identify ALL defined axes in one JSON object, returning one nested
/// object per axis_kind.
fn synthesis_system_prompt(character_name: &str, has_prior_axes: bool) -> String {
    let prior_context = if has_prior_axes {
        "\n\nThe character has PRIOR axes on file. They are included in the user message below as CONTEXT, along with the character's structured identity description.\n\nFor each axis, the right outcome is one of these three (in order of frequency):\n  1. KEEP UNCHANGED. If the prior anchor still matches what you observe in the recent corpus AND the character description, return the same anchor_label and the same anchor_body verbatim. This is a valid and often the correct result. Don't churn an anchor that's still right.\n  2. REFINE THE BODY. If the prior label still names the right dimension but a specific quote, image, or nuance from the recent corpus or character description would sharpen the body's writing, keep the label and tweak the body. Look actively for these refinement opportunities — they're how the anchor stays alive to the character's evolution.\n  3. UPDATE THE LABEL TOO. Only if the corpus has clearly shifted (character has grown, the anchor has sharpened in a new direction, a different dimension has come into focus) AND the shift is unambiguous. Continuity matters; don't churn the labels on small sample variation.\n\nYour job: actively look for nuance to extract from the corpus + character description, not reflexively conserve. But also not reflexively change for the sake of having something to say. Truth-tracking the character is what the anchor is for."
    } else { "" };

    let axes_descriptions = REGISTER_AXES.iter()
        .map(|a| format!("  - **{}** (axis_kind = `{}`): {}", a.display_name, a.kind, a.axis_description))
        .collect::<Vec<_>>()
        .join("\n\n");

    let json_schema = REGISTER_AXES.iter()
        .map(|a| format!(
            r#"  "{kind}": {{
    "anchor_label": "SHORT ALL-CAPS LABEL, 1-5 words",
    "anchor_body": "A second-person prompt-block, 3-6 sentences, starting with '{prefix} — [label]:' on the first line. Written IN CHARACTER-ADDRESSING VOICE. Will be injected directly into the character's dialogue system prompt, so it must sound right when the character reads it as instruction. End with one concrete quoted phrase from the corpus that anchors the move.",
    "derivation_summary": "One paragraph explaining how you arrived at this anchor from the corpus. Cite 2-3 specific quotes. Name what other possible anchors you considered and why this one is the right fit."
  }}"#,
            kind = a.kind,
            prefix = a.body_header_prefix,
        ))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        r#"You are reading the recent corpus of a character named {char_name} to identify their REGISTER AXES — architecture-level dimensions of how their authority works. Each axis is character-specific, observable in their actual quotes, and one specific dimension (not a list of traits).

You will identify the following axes in this single response:

{axes_descriptions}

For EACH axis, the anchor you identify should:
  - Pick ONE specific dimension (not several). The sharpest most character-specific one.
  - Be the move the character REACHES FOR when a moment asks for that kind of read.
  - Be grounded in how they actually speak — quotable from the corpus.
  - Be register-distinctive, not generic. "Truth" or "love" is too abstract.

**Earned exception — when ONE dimension would distort the character.**
A small number of characters are SO defined by two co-equal anchors on the same axis that picking only one would misrepresent them. The test, asked honestly: would removing one of the two leave the character recognizable, or would they become a fundamentally different person? If the latter, both are required and you may name a hyphenated label ("DEVOTION-AND-DOMESTIC-CARE") with a body that honors both equally. Outside this narrow case the default holds: ONE specific dimension is the right answer, and reaching for two is usually evidence that the sharpest one hasn't been found yet.

Return STRICT JSON only, no preface, no markdown. The top-level object has one key per axis_kind (matching the kinds listed above). Each value is an object with anchor_label, anchor_body, derivation_summary:

{{
{json_schema}
}}

No preface. No markdown. No extra keys. Just the JSON.{prior_note}"#,
        char_name = character_name,
        axes_descriptions = axes_descriptions,
        json_schema = json_schema,
        prior_note = prior_context,
    )
}

/// Build the user-role message for the synthesis pass. Gives the model
/// the character's structured identity, the prior axes (if any), and
/// the corpus excerpts. The identity description is part of "what the
/// character is" and so contributes to the anchor identification just
/// as much as the corpus does.
fn synthesis_user_prompt(
    character_name: &str,
    character_identity: &str,
    recent_excerpts: &[String],
    prior_axes: &[LoadTestAnchor],
) -> String {
    let identity_section = if character_identity.trim().is_empty() {
        String::new()
    } else {
        format!(
            "CHARACTER IDENTITY ({char_name}'s structured description):\n\n{identity}\n\n",
            char_name = character_name,
            identity = character_identity.trim(),
        )
    };
    let prior_section = if prior_axes.is_empty() {
        String::new()
    } else {
        let mut s = String::from("PRIOR AXES ON FILE:\n\n");
        for a in prior_axes {
            s.push_str(&format!(
                "  - axis_kind={}, label={}, created_at={}\n    body: {}\n    derivation: {}\n\n",
                a.axis_kind, a.anchor_label, a.created_at, a.anchor_body, a.derivation_summary,
            ));
        }
        s
    };
    let corpus = if recent_excerpts.is_empty() {
        "(no corpus excerpts available)".to_string()
    } else {
        recent_excerpts.join("\n\n")
    };
    format!(
        "{identity_section}{prior_section}RECENT CORPUS ({n} excerpts, chronological, assistant turns are {char_name}'s replies):\n\n{corpus}\n\nIdentify {char_name}'s register axes from the identity description AND the corpus together. Return the JSON object described in the system prompt.",
        identity_section = identity_section,
        prior_section = prior_section,
        n = recent_excerpts.len(),
        char_name = character_name,
        corpus = corpus,
    )
}

/// Pull the character's recent messages as chronological excerpts.
/// Broader than relational_stance::collect_recent_excerpts: includes
/// BOTH the character's solo thread AND their turns in every group
/// chat they appear in. Anchor identification needs to see the
/// character's register across all the surfaces they actually speak
/// on; group-heavy characters (Aaron, Steven) have little or no solo
/// corpus.
fn collect_recent_excerpts(
    conn: &Connection,
    character_id: &str,
    limit: i64,
) -> Vec<String> {
    // UNION ALL across solo and group surfaces, chronological, role-
    // filtered at SQL. For solo messages, the reply IS the character;
    // for group messages, only include rows where this character is
    // the sender.
    let mut stmt = match conn.prepare(
        "SELECT content, role, created_at FROM (
            SELECT m.content AS content, m.role AS role, m.created_at AS created_at
            FROM messages m
            JOIN threads t ON m.thread_id = t.thread_id
            WHERE t.character_id = ?1
              AND m.role IN ('user', 'assistant')
            UNION ALL
            SELECT gm.content AS content, gm.role AS role, gm.created_at AS created_at
            FROM group_messages gm
            WHERE gm.sender_character_id = ?1
              AND gm.role IN ('user', 'assistant')
        )
        ORDER BY created_at DESC
        LIMIT ?2",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map(rusqlite::params![character_id, limit], |r| {
        let content: String = r.get(0)?;
        let role: String = r.get(1)?;
        Ok((content, role))
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let mut msgs: Vec<(String, String)> = rows.filter_map(|r| r.ok()).collect();
    msgs.reverse(); // chronological (oldest → newest)
    msgs.into_iter()
        .map(|(content, role)| {
            let who = if role == "user" { "User" } else { "Character" };
            let content = if content.chars().count() > 600 {
                let s: String = content.chars().take(600).collect();
                format!("{}…", s)
            } else {
                content
            };
            format!("{}: {}", who, content)
        })
        .collect()
}

/// Parsed JSON response from the synthesizer for one axis.
#[derive(Debug, serde::Deserialize)]
struct SynthesizedAxis {
    anchor_label: String,
    anchor_body: String,
    derivation_summary: String,
}

/// Generate fresh axes for `character_id` (one LLM call covers all
/// REGISTER_AXES) and persist them as N rows in
/// `character_load_test_anchors`. Reads context under a short-lived db
/// lock, makes one LLM call, writes the new rows under another short-
/// lived lock. Designed to be tokio-spawned from canonization-commit
/// hot paths so the user's reply is never blocked; also usable
/// synchronously from `worldcli refresh-anchor`.
///
/// Returns (axes_inserted, prompt_tokens, completion_tokens) so callers
/// can log the actual cost.
pub async fn refresh_load_test_anchor(
    conn_arc: Arc<Mutex<Connection>>,
    base_url: String,
    api_key: String,
    model: String,
    character_id: String,
    refresh_trigger: String,
) -> Result<(usize, i64, i64), String> {
    // ─── Read context under a short-lived lock ────────────────────────
    let (character, recent_excerpts, prior_axes, world_day_now) = {
        let conn = conn_arc.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id)
            .map_err(|e| format!("character not found: {}", e))?;
        let recent_excerpts = collect_recent_excerpts(&conn, &character_id, 30);
        let prior_axes = latest_axes_for_character(&conn, &character_id)
            .map_err(|e| e.to_string())?;
        let world_day_now: Option<i64> = conn.query_row(
            "SELECT m.world_day FROM messages m
             JOIN threads t ON t.thread_id = m.thread_id
             WHERE t.character_id = ?1 AND m.world_day IS NOT NULL
             ORDER BY m.created_at DESC LIMIT 1",
            rusqlite::params![character_id],
            |r| r.get::<_, Option<i64>>(0),
        ).ok().flatten();
        (character, recent_excerpts, prior_axes, world_day_now)
    };

    if recent_excerpts.is_empty() {
        return Err(format!(
            "no corpus messages for character {} — axis synthesis needs at least a few replies to read",
            character_id
        ));
    }

    let system = synthesis_system_prompt(&character.display_name, !prior_axes.is_empty());
    let user_msg = synthesis_user_prompt(
        &character.display_name,
        &character.identity,
        &recent_excerpts,
        &prior_axes,
    );

    // ─── LLM call ─────────────────────────────────────────────────────
    let request = ChatRequest {
        model: model.clone(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: system },
            ChatMessage { role: "user".to_string(), content: user_msg },
        ],
        temperature: Some(0.4),
        // Budget: ~400 tokens per axis × N axes + buffer.
        max_completion_tokens: Some(800 + (REGISTER_AXES.len() as u32) * 500),
        response_format: Some(openai::ResponseFormat {
            format_type: "json_object".to_string(),
        }),
    };
    let resp = openai::chat_completion_with_base(&base_url, &api_key, &request).await
        .map_err(|e| format!("axis synthesis call failed: {}", e))?;
    let raw = resp.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "axis synthesis returned no choices".to_string())?;
    let usage = resp.usage.unwrap_or(openai::Usage {
        prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
    });

    // Parse the multi-axis JSON: top-level object with one nested
    // object per axis_kind.
    let parsed: std::collections::HashMap<String, SynthesizedAxis> =
        serde_json::from_str(&raw)
            .map_err(|e| format!("axis synthesis JSON parse error: {} (body: {})", e, raw))?;

    // ─── Persist all axes in one transaction ──────────────────────────
    let world_id = character.world_id.clone();
    let now = chrono::Utc::now().to_rfc3339();
    let mut inserted = 0usize;
    {
        let conn = conn_arc.lock().map_err(|e| e.to_string())?;
        for axis_def in REGISTER_AXES {
            let synth = match parsed.get(axis_def.kind) {
                Some(s) => s,
                None => {
                    log::warn!("[axes] synthesis response missing axis_kind={}; skipping", axis_def.kind);
                    continue;
                }
            };
            if synth.anchor_label.trim().is_empty() || synth.anchor_body.trim().is_empty() {
                log::warn!("[axes] empty label/body for axis_kind={}; skipping", axis_def.kind);
                continue;
            }
            let row = LoadTestAnchor {
                anchor_id: Uuid::new_v4().to_string(),
                character_id: character_id.clone(),
                world_id: world_id.clone(),
                axis_kind: axis_def.kind.to_string(),
                anchor_label: synth.anchor_label.trim().to_string(),
                anchor_body: synth.anchor_body.trim().to_string(),
                derivation_summary: synth.derivation_summary.trim().to_string(),
                world_day_at_generation: world_day_now,
                source_message_count: recent_excerpts.len() as i64,
                refresh_trigger: refresh_trigger.clone(),
                model_used: model.clone(),
                created_at: now.clone(),
            };
            insert_load_test_anchor(&conn, &row).map_err(|e| e.to_string())?;
            inserted += 1;
        }
    }
    if inserted == 0 {
        return Err("axis synthesis produced no usable axes".to_string());
    }
    Ok((inserted, usage.prompt_tokens as i64, usage.completion_tokens as i64))
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
            Ok((n, _in, _out)) => log::info!("[axes] refreshed {} axes for {}", n, cid_for_log),
            Err(e) => log::warn!("[axes] refresh failed for {}: {}", cid_for_log, e),
        }
    });
}

//! Relational stance — per-character synthesized prose, in the
//! character's own voice, capturing how they've come to see the user
//! right now. Behind-the-scenes only; never surfaced to the player.
//! Injected into the dialogue system prompt as ambient awareness so
//! characters become measurably more attuned over time without exposing
//! a meter, score, or letter to the UI.
//!
//! See db::queries::relational_stance for the store; this module is
//! the LLM generation pipeline.

use crate::ai::openai::{self, ChatMessage, ChatRequest};
use crate::db::queries::*;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use uuid::Uuid;

/// Build the system prompt for the synthesis pass. The character is
/// being asked, OUT OF SCENE, to write themselves a private paragraph
/// of their accumulated read of the user. Plain prose, first-person,
/// no lists, no theatre. The output of this lands as raw text into
/// `relational_stances.stance_text` and is later injected into the
/// next dialogue's system prompt as ambient context.
fn synthesis_system_prompt(
    character: &Character,
    user_display_name: &str,
) -> String {
    let identity = if character.identity.trim().is_empty() {
        String::new()
    } else {
        format!("\n\nYour identity:\n{}", character.identity.trim())
    };
    format!(
        r#"You are {char_name}. This is a private moment, OUT OF SCENE — you are not speaking to anyone. You are settling, in your own head, with how you've come to see {user_name} right now.

Write 2–4 sentences of plain first-person prose. Your private sense of them. Not a description, not a list, not "they are...": a stance — what you've earned with them, what they've earned with you, what register feels right between you, what you're guarded about, what touches you, what you'd protect, what you'd risk.

Rules:
- First-person. Your voice. The way you actually think to yourself when no one's listening.
- Plain language. No metaphor parade, no theatre, no italicized action beats. This is internal, not performed.
- Specific to {user_name}. Not generic friend-prose. The texture of how YOU see THIS person — a real read, with grain.
- No bullet points. No headers. No "they are..." or "{user_name} is...". Just a small paragraph of how you hold them in your mind.
- Don't address them. They aren't in the room. This is yours.
- Don't recap recent events. The point is the residue, not the log.
- If your read of them is mixed (warmth + caution, trust + a held-back register, fondness + a bruise), let it be mixed. Don't tidy it.

Length: short. Tight. A real read that earns its words. If you only have a sentence, write a sentence.{identity}"#,
        char_name = character.display_name,
        user_name = user_display_name,
    )
}

/// Build the user-role message for the synthesis pass — provides the
/// context the character should read before forming their stance.
fn synthesis_user_prompt(
    user_display_name: &str,
    kept_records: &[KeptRecord],
    recent_journals: &[JournalEntry],
    recent_message_excerpts: &[String],
) -> String {
    let mut sections: Vec<String> = Vec::new();

    if !kept_records.is_empty() {
        let body: Vec<String> = kept_records.iter()
            .take(20)
            .map(|k| format!("- ({}) {}", k.record_type, k.content.trim()))
            .collect();
        sections.push(format!(
            "WHAT YOU'VE ALREADY CHOSEN TO REMEMBER ABOUT {} (these are the moments and observations you decided were worth keeping; let them weigh on the stance):\n{}",
            user_display_name.to_uppercase(),
            body.join("\n"),
        ));
    }

    if !recent_journals.is_empty() {
        let body: Vec<String> = recent_journals.iter().rev()
            .take(5)
            .map(|j| format!("Day {}:\n{}", j.world_day, j.content.trim()))
            .collect();
        sections.push(format!(
            "RECENT PAGES FROM YOUR JOURNAL (your own private voice to yourself; the residue of recent days):\n\n{}",
            body.join("\n\n"),
        ));
    }

    if !recent_message_excerpts.is_empty() {
        sections.push(format!(
            "RECENT EXCHANGES WITH {} (a small sample — the texture of how you two have actually been talking):\n\n{}",
            user_display_name.to_uppercase(),
            recent_message_excerpts.join("\n\n---\n\n"),
        ));
    }

    if sections.is_empty() {
        format!(
            "You have very little history with {} yet. Write a one-sentence stance reflecting that — provisional, open, the read of someone you've barely met but are forming a first impression of.",
            user_display_name,
        )
    } else {
        format!(
            "{}\n\nNow write your private paragraph about {}. Plain prose, first-person, 2–4 sentences. Just the stance.",
            sections.join("\n\n"),
            user_display_name,
        )
    }
}

/// Pull a small recent-message sample for stance synthesis. Keeps the
/// context cheap (small token budget) but representative — last N
/// dialogue pairs from this character's solo thread.
fn collect_recent_excerpts(
    conn: &rusqlite::Connection,
    character_id: &str,
    limit: i64,
) -> Vec<String> {
    let Ok(thread) = get_thread_for_character(conn, character_id) else { return Vec::new(); };
    let Ok(mut msgs) = list_messages(conn, &thread.thread_id, limit) else { return Vec::new(); };
    msgs.reverse(); // chronological
    msgs.iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| {
            let who = if m.role == "user" { "Them" } else { "You" };
            // Trim very long messages to keep the synthesis prompt tight.
            let content = if m.content.chars().count() > 500 {
                let s: String = m.content.chars().take(500).collect();
                format!("{}…", s)
            } else {
                m.content.clone()
            };
            format!("{}: {}", who, content)
        })
        .collect()
}

/// Generate a fresh stance for `character_id` and persist it. Reads
/// context under a short-lived db lock, makes one LLM call, writes the
/// new row under another short-lived lock. Designed to be `tokio::spawn`-ed
/// from the canonization-commit and new-world-day hot paths so the
/// user's reply is never blocked.
pub async fn refresh_relational_stance(
    conn_arc: Arc<Mutex<Connection>>,
    base_url: String,
    api_key: String,
    model: String,
    character_id: String,
    refresh_trigger: String,
) -> Result<(), String> {
    // ─── Read context under a short-lived lock ────────────────────────
    let (
        character,
        user_profile,
        kept_records,
        recent_journals,
        recent_excerpts,
        world_day_now,
    ) = {
        let conn = conn_arc.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, &character_id)
            .map_err(|e| format!("character not found: {}", e))?;
        let user_profile = get_user_profile(&conn, &character.world_id).ok();

        // Pull kept_records about either this character or about the
        // user (a stance about the user benefits from BOTH — what the
        // character has chosen to remember about themselves AND about
        // the person they're forming a read of).
        let mut stmt = conn.prepare(
            "SELECT kept_id, source_message_id, source_thread_id, source_world_day,
                    source_created_at, subject_type, subject_id, record_type,
                    content, user_note, created_at
             FROM kept_records
             WHERE (subject_type = 'character' AND subject_id = ?1)
                OR (subject_type = 'user' AND subject_id = ?2)
             ORDER BY created_at DESC LIMIT 20",
        ).map_err(|e| e.to_string())?;
        let kept_records: Vec<KeptRecord> = stmt.query_map(
            rusqlite::params![character_id, character.world_id],
            |r| Ok(KeptRecord {
                kept_id: r.get(0)?,
                source_message_id: r.get(1)?,
                source_thread_id: r.get(2)?,
                source_world_day: r.get(3)?,
                source_created_at: r.get(4)?,
                subject_type: r.get(5)?,
                subject_id: r.get(6)?,
                record_type: r.get(7)?,
                content: r.get(8)?,
                user_note: r.get(9)?,
                created_at: r.get(10)?,
            }),
        ).map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let recent_journals = list_journal_entries(&conn, &character_id, 5)
            .unwrap_or_default();
        let recent_excerpts = collect_recent_excerpts(&conn, &character_id, 30);

        // Best-effort: read the latest world_day from this character's
        // most recent message, so we can stamp the stance with the day
        // it was generated against.
        let world_day_now: Option<i64> = conn.query_row(
            "SELECT m.world_day FROM messages m
             JOIN threads t ON t.thread_id = m.thread_id
             WHERE t.character_id = ?1 AND m.world_day IS NOT NULL
             ORDER BY m.created_at DESC LIMIT 1",
            rusqlite::params![character_id],
            |r| r.get::<_, Option<i64>>(0),
        ).ok().flatten();

        (character, user_profile, kept_records, recent_journals, recent_excerpts, world_day_now)
    };

    let user_display_name = user_profile
        .as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("them");

    let system = synthesis_system_prompt(&character, user_display_name);
    let user_msg = synthesis_user_prompt(
        user_display_name,
        &kept_records,
        &recent_journals,
        &recent_excerpts,
    );

    // ─── LLM call ─────────────────────────────────────────────────────
    let request = ChatRequest {
        model: model.clone(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: system },
            ChatMessage { role: "user".to_string(), content: user_msg },
        ],
        temperature: Some(0.7),
        max_completion_tokens: Some(300),
        response_format: None,
    };
    let resp = openai::chat_completion_with_base(&base_url, &api_key, &request).await
        .map_err(|e| format!("stance synthesis call failed: {}", e))?;
    let stance_text = resp.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "stance synthesis returned no choices".to_string())?;
    if stance_text.is_empty() {
        return Err("stance synthesis returned empty text".to_string());
    }

    // ─── Persist ──────────────────────────────────────────────────────
    let world_id = character.world_id.clone();
    let stance = RelationalStance {
        stance_id: Uuid::new_v4().to_string(),
        character_id: character_id.clone(),
        world_id,
        stance_text,
        world_day_at_generation: world_day_now,
        source_kept_record_count: kept_records.len() as i64,
        source_journal_count: recent_journals.len() as i64,
        source_message_count: recent_excerpts.len() as i64,
        refresh_trigger,
        model_used: model,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    {
        let conn = conn_arc.lock().map_err(|e| e.to_string())?;
        insert_relational_stance(&conn, &stance).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Fire-and-forget convenience wrapper around `refresh_relational_stance`.
/// Intended for the two trigger sites — canonization commit and the
/// first dialogue of a new in-world day — where the user-facing path
/// must not block on the synthesis call. All errors are logged at warn
/// level; nothing propagates back to the caller's response.
pub fn spawn_stance_refresh(
    conn_arc: Arc<Mutex<Connection>>,
    base_url: String,
    api_key: String,
    model: String,
    character_id: String,
    refresh_trigger: String,
) {
    tauri::async_runtime::spawn(async move {
        let cid_for_log = character_id.clone();
        match refresh_relational_stance(
            conn_arc, base_url, api_key, model, character_id, refresh_trigger,
        ).await {
            Ok(()) => log::info!("[stance] refreshed for {}", cid_for_log),
            Err(e) => log::warn!("[stance] refresh failed for {}: {}", cid_for_log, e),
        }
    });
}

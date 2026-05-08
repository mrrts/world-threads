use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use tauri::State;

#[tauri::command]
pub fn add_reaction_cmd(
    db: State<Database>,
    message_id: String,
    emoji: String,
    reactor: String,
) -> Result<Reaction, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    // Toggle: if this exact reaction already exists on the direct target,
    // remove it (and the propagated siblings) instead of adding.
    let existing: Option<String> = conn.query_row(
        "SELECT reaction_id FROM reactions WHERE message_id = ?1 AND emoji = ?2 AND reactor = ?3",
        rusqlite::params![message_id, emoji, reactor],
        |r| r.get(0),
    ).ok();

    let contributing = get_contributing_message_ids(&conn, &message_id);

    if existing.is_some() {
        // Symmetric removal — every sibling that was propagated to on add.
        for mid in &contributing {
            remove_reaction(&conn, mid, &emoji, &reactor).ok();
        }
        return Err("removed".to_string());
    }

    // Fan out: one row per contributing message (target + preceding burst +
    // the user message that triggered the burst). The primary Reaction
    // returned is the direct-target one, so the UI can attribute the click.
    let created_at = Utc::now().to_rfc3339();
    let mut primary: Option<Reaction> = None;
    for mid in &contributing {
        // Skip if this message already has this (emoji, reactor) — keeps
        // the operation idempotent if the user double-clicks quickly.
        let dup: Option<String> = conn.query_row(
            "SELECT reaction_id FROM reactions WHERE message_id = ?1 AND emoji = ?2 AND reactor = ?3",
            rusqlite::params![mid, emoji, reactor],
            |r| r.get(0),
        ).ok();
        if dup.is_some() {
            continue;
        }

        let r = Reaction {
            reaction_id: uuid::Uuid::new_v4().to_string(),
            message_id: mid.clone(),
            emoji: emoji.clone(),
            reactor: reactor.clone(),
            created_at: created_at.clone(),
            sender_character_id: None,
        };
        add_reaction(&conn, &r).map_err(|e| e.to_string())?;
        if primary.is_none() && mid == &message_id {
            primary = Some(r);
        }
    }

    // Feed the thread's mood-reduction ring buffer — this is the closed
    // loop that seeds the next AGENCY chain from the user's own reactions.
    // Only user-emitted reactions count; a character-emitted reaction is
    // not a user signal. Deduped in-buffer by push_mood_reduction.
    if reactor == "user" {
        if let Some(thread_id) = get_thread_id_for_message(&conn, &message_id) {
            let _ = push_mood_reduction(&conn, &thread_id, &emoji);
        }
    }

    Ok(primary.unwrap_or_else(|| Reaction {
        reaction_id: String::new(),
        message_id,
        emoji,
        reactor,
        created_at,
        sender_character_id: None,
    }))
}

#[tauri::command]
pub fn remove_reaction_cmd(
    db: State<Database>,
    message_id: String,
    emoji: String,
    reactor: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    remove_reaction(&conn, &message_id, &emoji, &reactor).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_reactions_cmd(
    db: State<Database>,
    message_ids: Vec<String>,
) -> Result<Vec<Reaction>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    get_reactions_for_messages(&conn, &message_ids).map_err(|e| e.to_string())
}

/// Expose the thread's current mood-reduction ring buffer so the UI can
/// surface the "what's the thread feeling" signal. Looks up the thread by
/// either a character_id (individual chat) or a group_chat_id.
#[tauri::command]
pub fn get_mood_reduction_cmd(
    db: State<Database>,
    character_id: Option<String>,
    group_chat_id: Option<String>,
) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let thread_id = if let Some(gc_id) = group_chat_id {
        let gc = get_group_chat(&conn, &gc_id).map_err(|e| e.to_string())?;
        gc.thread_id
    } else if let Some(ch_id) = character_id {
        let thread = get_thread_for_character(&conn, &ch_id).map_err(|e| e.to_string())?;
        thread.thread_id
    } else {
        return Err("character_id or group_chat_id required".to_string());
    };
    Ok(get_thread_mood_reduction(&conn, &thread_id))
}

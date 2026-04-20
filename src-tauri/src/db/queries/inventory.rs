use rusqlite::{params, Connection};

/// One rendered line of a character's recent conversation history, merged
/// across every thread they participate in (solo + groups) and sorted by
/// real-time `created_at` ascending. Built for the inventory seed/refresh
/// LLM calls: the model needs to see what the character has lived
/// through lately, in order, without any "in a chat between X, Y, Z:"
/// framing — just speaker-labeled lines.
#[derive(Debug, Clone)]
pub struct ConversationLine {
    /// Who spoke: the user's display name, a character's display name,
    /// or a generic role label for narrative / dream / context lines.
    pub speaker: String,
    /// Raw content of the message. Illustration / video messages are
    /// skipped entirely by the query — the content is never binary.
    pub content: String,
    /// ISO timestamp; kept alongside so callers can emit relative-time
    /// or chronology markers if they want.
    #[allow(dead_code)]
    pub created_at: String,
}

/// Gather recent messages from every thread the character participates
/// in (their solo thread plus every group they're a member of), merged
/// into a single chronological sequence by real-world `created_at`.
///
/// Returns up to `limit` lines, newest-end. Skips illustration/video
/// messages (binary payloads) and non-textual roles (system, context).
/// User display name is resolved once and used for any message with
/// role="user".
pub fn gather_character_recent_messages(
    conn: &Connection,
    character_id: &str,
    user_display_name: &str,
    limit: usize,
) -> Vec<ConversationLine> {
    // Resolve this character's display name for labeling assistant lines
    // (and the solo thread is all about them, so we need it).
    let my_name: String = conn
        .query_row(
            "SELECT display_name FROM characters WHERE character_id = ?1",
            params![character_id],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "Character".to_string());

    // Per-member display-name map. Keyed by character_id. Used so group
    // assistant lines can be labeled by whichever character actually
    // spoke (sender_character_id) rather than blanket "Character".
    let mut names: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    names.insert(character_id.to_string(), my_name.clone());

    // Group threads the character is in — collected first so we can also
    // resolve every member name for labeling.
    let group_threads: Vec<String> = {
        let mut out: Vec<String> = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT thread_id, character_ids FROM group_chats
             WHERE EXISTS (SELECT 1 FROM json_each(character_ids) WHERE value = ?1)",
        ) {
            if let Ok(rows) = stmt.query_map(params![character_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            }) {
                for row in rows.flatten() {
                    out.push(row.0);
                    if let Ok(ids) = serde_json::from_str::<Vec<String>>(&row.1) {
                        for id in ids {
                            if !names.contains_key(&id) {
                                if let Ok(n) = conn.query_row(
                                    "SELECT display_name FROM characters WHERE character_id = ?1",
                                    params![id],
                                    |r| r.get::<_, String>(0),
                                ) {
                                    names.insert(id, n);
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    };

    // Solo thread (if any).
    let solo_thread: Option<String> = conn
        .query_row(
            "SELECT thread_id FROM threads WHERE character_id = ?1",
            params![character_id],
            |r| r.get(0),
        )
        .ok();

    let mut all: Vec<ConversationLine> = Vec::new();

    // Solo thread messages.
    if let Some(tid) = solo_thread.as_deref() {
        let lim = (limit as i64).max(1);
        if let Ok(mut stmt) = conn.prepare(
            "SELECT role, content, created_at, sender_character_id FROM messages
             WHERE thread_id = ?1 AND role NOT IN ('illustration', 'video', 'system', 'context')
             ORDER BY created_at DESC LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![tid, lim], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            }) {
                for (role, content, created_at, sender_id) in rows.flatten() {
                    all.push(ConversationLine {
                        speaker: label_for(&role, sender_id.as_deref(), &names, &my_name, user_display_name),
                        content,
                        created_at,
                    });
                }
            }
        }
    }

    // Group thread messages.
    for tid in &group_threads {
        let lim = (limit as i64).max(1);
        if let Ok(mut stmt) = conn.prepare(
            "SELECT role, content, created_at, sender_character_id FROM group_messages
             WHERE thread_id = ?1 AND role NOT IN ('illustration', 'video', 'system', 'context')
             ORDER BY created_at DESC LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![tid, lim], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                ))
            }) {
                for (role, content, created_at, sender_id) in rows.flatten() {
                    all.push(ConversationLine {
                        speaker: label_for(&role, sender_id.as_deref(), &names, &my_name, user_display_name),
                        content,
                        created_at,
                    });
                }
            }
        }
    }

    // Merge chronologically ascending; tail to `limit`.
    all.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    if all.len() > limit {
        let drop = all.len() - limit;
        all.drain(0..drop);
    }
    all
}

fn label_for(
    role: &str,
    sender_id: Option<&str>,
    names: &std::collections::HashMap<String, String>,
    my_name: &str,
    user_display_name: &str,
) -> String {
    match role {
        "user" => user_display_name.to_string(),
        "assistant" => sender_id
            .and_then(|id| names.get(id).cloned())
            .unwrap_or_else(|| my_name.to_string()),
        "narrative" => "Narrator".to_string(),
        "dream" => "Dream".to_string(),
        other => other.to_string(),
    }
}

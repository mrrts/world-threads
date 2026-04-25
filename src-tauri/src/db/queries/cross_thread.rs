use rusqlite::{params, Connection};

/// A block of recent messages from one of a character's OTHER threads —
/// their solo thread if we're currently generating in a group, or one of
/// their group threads if we're in their solo. Injected into the dialogue
/// retrieval context as a single labeled snippet so the character
/// remembers what happened in their other chats.
#[derive(Debug, Clone)]
pub struct CrossThreadBlock {
    /// Human-readable POV label, e.g. "your one-on-one chat with Ryan" or
    /// "the group chat with Bob and Carol". Written from the focal
    /// character's perspective.
    pub label: String,
    /// ISO timestamp of the newest message in this block. Used to order
    /// blocks across threads and to format the relative-time prefix.
    pub newest_at: String,
    /// Multi-line rendered text ready to be appended as a retrieval
    /// snippet. Includes the `[From {label}, {time ago}]` header and one
    /// `Speaker: content` line per message, chronological.
    pub rendered: String,
}

/// Fetch recent cross-thread context for a character from each other
/// thread they participate in (solo + all groups), excluding the thread
/// we're currently generating in.
///
/// Returns blocks sorted by newest-first, capped at `max_other_threads`.
/// Each block has up to `per_thread_limit` messages.
///
/// Skips threads whose latest activity is older than `recency_cap_days`
/// (None = no cutoff). This prevents dredging up a long-dormant chat the
/// user never engages with.
pub fn list_cross_thread_recent_for_character(
    conn: &Connection,
    character_id: &str,
    current_thread_id: &str,
    per_thread_limit: i64,
    max_other_threads: usize,
    user_name: &str,
) -> Vec<CrossThreadBlock> {
    let mut blocks: Vec<CrossThreadBlock> = Vec::new();

    // Character's solo thread (if any).
    let solo_thread_id: Option<String> = conn
        .query_row(
            "SELECT thread_id FROM threads WHERE character_id = ?1",
            params![character_id],
            |r| r.get(0),
        )
        .ok();

    if let Some(tid) = solo_thread_id.as_deref() {
        if tid != current_thread_id {
            if let Some(block) = render_solo_block(conn, tid, character_id, user_name, per_thread_limit) {
                blocks.push(block);
            }
        }
    }

    // Group threads the character is a member of. Uses json_each to
    // filter; rusqlite ships with JSON1 enabled.
    let mut stmt = match conn.prepare(
        "SELECT gc.thread_id, gc.character_ids
         FROM group_chats gc
         WHERE EXISTS (SELECT 1 FROM json_each(gc.character_ids) WHERE value = ?1)",
    ) {
        Ok(s) => s,
        Err(_) => return blocks,
    };
    let group_rows: Vec<(String, String)> = stmt
        .query_map(params![character_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .ok()
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    for (thread_id, character_ids_json) in group_rows {
        if thread_id == current_thread_id {
            continue;
        }
        if let Some(block) = render_group_block(
            conn,
            &thread_id,
            character_id,
            &character_ids_json,
            user_name,
            per_thread_limit,
        ) {
            blocks.push(block);
        }
    }

    // Cap to max_other_threads, keeping the newest threads (sort
    // newest-first first, take top N), then RE-SORT chronologically
    // (oldest first) so the snippet reads like a chat history with
    // the most-recent material closest to the end of the prompt — LLM
    // attention is recency-weighted at the end, so what's freshest
    // should land last.
    blocks.sort_by(|a, b| b.newest_at.cmp(&a.newest_at));
    blocks.truncate(max_other_threads);
    blocks.sort_by(|a, b| a.newest_at.cmp(&b.newest_at));
    blocks
}

fn render_solo_block(
    conn: &Connection,
    thread_id: &str,
    character_id: &str,
    user_name: &str,
    limit: i64,
) -> Option<CrossThreadBlock> {
    // Character name for labeling the assistant role in rendered lines.
    let character_name: String = conn
        .query_row(
            "SELECT display_name FROM characters WHERE character_id = ?1",
            params![character_id],
            |r| r.get(0),
        )
        .ok()?;

    // Fetch last N textual messages in chronological order.
    let mut stmt = conn
        .prepare(
            "SELECT role, content, created_at FROM messages
             WHERE thread_id = ?1 AND role NOT IN ('illustration', 'video', 'context', 'system', 'imagined_chapter')
             ORDER BY created_at DESC LIMIT ?2",
        )
        .ok()?;
    let mut rows: Vec<(String, String, String)> = stmt
        .query_map(params![thread_id, limit], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();
    rows.reverse();
    if rows.is_empty() {
        return None;
    }

    let newest_at = rows.last().map(|r| r.2.clone()).unwrap_or_default();
    let lines: Vec<String> = rows
        .iter()
        .map(|(role, content, _)| render_line(role, content, &character_name, user_name))
        .collect();

    let label = format!("your one-on-one chat with {user_name}");
    let rendered = format!(
        "[From {label} — {weather}]\n{body}",
        label = label,
        weather = super::weathering::weathering_label(&newest_at),
        body = lines.join("\n"),
    );
    Some(CrossThreadBlock { label, newest_at, rendered })
}

fn render_group_block(
    conn: &Connection,
    thread_id: &str,
    focal_character_id: &str,
    character_ids_json: &str,
    user_name: &str,
    limit: i64,
) -> Option<CrossThreadBlock> {
    // Parse character_ids JSON.
    let ids: Vec<String> = serde_json::from_str(character_ids_json).ok()?;
    // Build a sender_character_id → display_name map for rendering + label.
    let mut name_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut other_names: Vec<String> = Vec::new();
    for cid in &ids {
        let name: Option<String> = conn
            .query_row(
                "SELECT display_name FROM characters WHERE character_id = ?1",
                params![cid],
                |r| r.get(0),
            )
            .ok();
        if let Some(n) = name {
            if cid != focal_character_id {
                other_names.push(n.clone());
            }
            name_map.insert(cid.clone(), n);
        }
    }

    // Fetch last N textual messages in chronological order.
    let mut stmt = conn
        .prepare(
            "SELECT role, content, sender_character_id, created_at FROM group_messages
             WHERE thread_id = ?1 AND role NOT IN ('illustration', 'video', 'context', 'system', 'imagined_chapter')
             ORDER BY created_at DESC LIMIT ?2",
        )
        .ok()?;
    let mut rows: Vec<(String, String, Option<String>, String)> = stmt
        .query_map(params![thread_id, limit], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .ok()?
        .filter_map(|r| r.ok())
        .collect();
    rows.reverse();
    if rows.is_empty() {
        return None;
    }

    let newest_at = rows.last().map(|r| r.3.clone()).unwrap_or_default();
    let lines: Vec<String> = rows
        .iter()
        .map(|(role, content, sender_id, _)| {
            let speaker = match role.as_str() {
                "user" => user_name.to_string(),
                "assistant" => sender_id
                    .as_ref()
                    .and_then(|id| name_map.get(id).cloned())
                    .unwrap_or_else(|| "A character".to_string()),
                "narrative" => "Narrator".to_string(),
                "dream" => sender_id
                    .as_ref()
                    .and_then(|id| name_map.get(id).cloned())
                    .map(|n| format!("{n} (dream)"))
                    .unwrap_or_else(|| "A dream".to_string()),
                _ => "Someone".to_string(),
            };
            format!("{speaker}: {}", truncate_line(content))
        })
        .collect();

    let label = if other_names.is_empty() {
        "a group chat".to_string()
    } else {
        format!("the group chat with {}", join_names(&other_names))
    };
    let rendered = format!(
        "[From {label} — {weather}]\n{body}",
        label = label,
        weather = super::weathering::weathering_label(&newest_at),
        body = lines.join("\n"),
    );
    Some(CrossThreadBlock { label, newest_at, rendered })
}

fn render_line(role: &str, content: &str, character_name: &str, user_name: &str) -> String {
    let speaker = match role {
        "user" => user_name,
        "assistant" => character_name,
        "narrative" => "Narrator",
        "dream" => "(dream)",
        _ => "Someone",
    };
    format!("{speaker}: {}", truncate_line(content))
}

/// Clip to ~200 chars with an ellipsis. Keeps cross-thread snippets from
/// blowing up the prompt when the original message was long.
fn truncate_line(s: &str) -> String {
    const MAX: usize = 200;
    let trimmed = s.trim();
    if trimmed.chars().count() <= MAX {
        return trimmed.to_string();
    }
    let clipped: String = trimmed.chars().take(MAX).collect();
    format!("{clipped}…")
}

/// Join character names as "A", "A and B", or "A, B, and C".
fn join_names(names: &[String]) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => {
            let head = &names[..names.len() - 1];
            let tail = &names[names.len() - 1];
            format!("{}, and {tail}", head.join(", "))
        }
    }
}


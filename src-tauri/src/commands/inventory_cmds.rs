use crate::ai::orchestrator::{self, InventoryItem};
use crate::db::queries::*;
use crate::db::Database;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

/// How many world-days can pass before an inventory is considered stale
/// and the focus-trigger refreshes it. 1 = refresh on any new day.
/// Adjustable knob — later could be swapped to a finer-grained stamp
/// (day + time_of_day) without changing the call sites.
pub const INVENTORY_STALE_DAYS: i64 = 1;

/// How many recent chronologically-merged messages to feed the seed /
/// refresh LLM as context. Caps token cost and attention dilution.
pub const INVENTORY_HISTORY_LIMIT: usize = 40;

/// Result shape for inventory refresh: the current items plus a flag
/// indicating whether a refresh actually ran this call (vs. no-op).
/// Frontend uses `refreshed` to decide whether to display a subtle
/// "inventory updated" cue.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InventoryRefreshResult {
    pub character_id: String,
    pub inventory: Vec<InventoryItem>,
    /// true if this call actually ran seed or refresh; false if the
    /// inventory was still fresh and we returned cached.
    pub refreshed: bool,
    /// "seed" | "refresh" | "noop" | "moment"
    pub mode: String,
    /// Diff produced by moment-anchored updates. Empty on seed/refresh
    /// callers — only the moment path fills these. Full items so the
    /// card can render names WITH descriptions (the fuller text with
    /// nuances the LLM put into each item).
    #[serde(default)]
    pub added: Vec<InventoryItem>,
    /// Items present in both prior and new with different descriptions.
    #[serde(default)]
    pub updated: Vec<InventoryItem>,
    /// Items that were in prior but not in new. Name-only — the item
    /// is gone from the inventory, so there's no "new" description to
    /// show; we just name what dropped off.
    #[serde(default)]
    pub removed: Vec<String>,
}

fn current_world_day(world: &World) -> i64 {
    world
        .state
        .get("time")
        .and_then(|t| t.get("day_index"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

fn parse_inventory(raw: &Value) -> Vec<InventoryItem> {
    raw.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| serde_json::from_value::<InventoryItem>(v.clone()).ok())
                .filter(|it| !it.name.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Run the check-and-possibly-refresh flow for one character. Returns
/// the current inventory either way. Pure function of state + LLM; no
/// side effects outside DB writes and the LLM call.
///
/// Trigger logic:
///   - last_inventory_day IS NULL → SEED.
///   - current_world_day - last_inventory_day >= INVENTORY_STALE_DAYS → REFRESH.
///   - otherwise → NOOP (return the stored inventory as-is).
pub async fn refresh_one_character_inventory(
    db: &Database,
    api_key: &str,
    character_id: &str,
) -> Result<InventoryRefreshResult, String> {
    let (character, world, model_config, user_name) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let user_name = get_user_profile(&conn, &character.world_id)
            .ok()
            .map(|p| p.display_name)
            .unwrap_or_else(|| "the human".to_string());
        (character, world, model_config, user_name)
    };

    let today = current_world_day(&world);
    let prior_items = parse_inventory(&character.inventory);

    let mode = match character.last_inventory_day {
        None => "seed",
        Some(last) if today - last >= INVENTORY_STALE_DAYS => "refresh",
        _ => "noop",
    };

    if mode == "noop" {
        return Ok(InventoryRefreshResult {
            character_id: character.character_id.clone(),
            inventory: prior_items,
            refreshed: false,
            mode: "noop".to_string(),
            ..Default::default()
        });
    }

    // Skip LLM entirely if there's no API key / local-only mode with no
    // memory model. Leave prior inventory untouched but advance the
    // stamp so we don't spam failed attempts.
    if api_key.trim().is_empty() {
        return Ok(InventoryRefreshResult {
            character_id: character.character_id.clone(),
            inventory: prior_items,
            refreshed: false,
            mode: "noop".to_string(),
            ..Default::default()
        });
    }

    let history = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        gather_character_recent_messages(
            &conn,
            &character.character_id,
            &user_name,
            INVENTORY_HISTORY_LIMIT,
        )
    };

    let base = model_config.chat_api_base();
    let model = &model_config.memory_model;

    let new_items = match mode {
        "seed" => orchestrator::seed_character_inventory(
            &base, api_key, model,
            &character.display_name, &character.identity,
            &history,
        ).await,
        _ => orchestrator::refresh_character_inventory(
            &base, api_key, model,
            &character.display_name, &character.identity,
            &prior_items, &history,
        ).await,
    };

    let new_items = match new_items {
        Ok(items) => items,
        Err(e) => {
            log::warn!("[Inventory] {} {} failed: {e}", mode, character.display_name);
            // Don't advance the stamp on failure — we'll try again on next focus.
            return Ok(InventoryRefreshResult {
                character_id: character.character_id.clone(),
                inventory: prior_items,
                refreshed: false,
                mode: "noop".to_string(),
                ..Default::default()
            });
        }
    };

    // Persist the new inventory and advance the stamp to today.
    // Snapshot the PRIOR state first so a later "reset to here" can
    // rewind the character's keeping along with the messages.
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = snapshot_inventory_pre_mutation(&conn, &character.character_id, mode);
        let json = serde_json::to_value(&new_items).unwrap_or(Value::Array(vec![]));
        let _ = set_character_inventory(&conn, &character.character_id, &json, Some(today));
    }

    log::info!(
        "[Inventory] {} {} — {} items",
        mode, character.display_name, new_items.len(),
    );

    Ok(InventoryRefreshResult {
        character_id: character.character_id.clone(),
        inventory: new_items,
        refreshed: true,
        mode: mode.to_string(),
        ..Default::default()
    })
}

/// Tauri entry point for a single character's focus-trigger refresh.
/// Called by the frontend when the user interacts with a solo chat.
#[tauri::command]
pub async fn refresh_character_inventory_cmd(
    db: State<'_, Database>,
    api_key: String,
    character_id: String,
) -> Result<InventoryRefreshResult, String> {
    refresh_one_character_inventory(&db, &api_key, &character_id).await
}

/// Tauri entry point for a group chat's focus-trigger refresh. Kicks
/// off one refresh task per group member in parallel so none of them
/// blocks the others. Returns the (possibly updated) inventory for each.
#[tauri::command]
pub async fn refresh_group_inventories_cmd(
    db: State<'_, Database>,
    api_key: String,
    group_chat_id: String,
) -> Result<Vec<InventoryRefreshResult>, String> {
    let char_ids: Vec<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let gc = get_group_chat(&conn, &group_chat_id).map_err(|e| e.to_string())?;
        gc.character_ids
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    };

    // Fan out concurrently — each inventory call is a memory-tier
    // LLM roundtrip, so they parallelize on the wire. SQLite access
    // serializes on the inner Mutex, which is fine since DB time is
    // tiny compared to the LLM wait. No tokio::spawn (avoids 'static
    // lifetime): join_all polls all futures on the current task.
    let db_ref: &Database = db.inner();
    let futs = char_ids.into_iter().map(|cid| {
        let key = api_key.clone();
        async move { refresh_one_character_inventory(db_ref, &key, &cid).await }
    });
    let results = futures_util::future::join_all(futs).await;
    let out: Vec<InventoryRefreshResult> = results.into_iter().filter_map(|r| r.ok()).collect();
    Ok(out)
}

/// Fetch (role, content, sender_character_id, created_at) for a message
/// by id. Checks both `messages` and `group_messages` tables — the click
/// that drove this call can originate from either. Returns None if the
/// message has been deleted between the click and the command landing.
struct MessageAnchor {
    role: String,
    content: String,
    sender_character_id: Option<String>,
    created_at: String,
    thread_id: String,
    /// True if this row came from `group_messages` (needed so the
    /// run-up query targets the correct table).
    is_group: bool,
}

fn get_message_anchor(conn: &rusqlite::Connection, message_id: &str)
    -> Option<MessageAnchor>
{
    let try_table = |table: &str, is_group: bool| -> Option<MessageAnchor> {
        conn.query_row(
            &format!("SELECT role, content, sender_character_id, created_at, thread_id FROM {} WHERE message_id = ?1", table),
            rusqlite::params![message_id],
            |r| Ok(MessageAnchor {
                role: r.get(0)?,
                content: r.get(1)?,
                sender_character_id: r.get(2)?,
                created_at: r.get(3)?,
                thread_id: r.get(4)?,
                is_group,
            }),
        ).ok()
    };
    try_table("messages", false).or_else(|| try_table("group_messages", true))
}

/// Fetch the N messages that arrived strictly before the anchor's
/// timestamp in the same thread, excluding meta / structural roles.
/// Returned chronological (oldest first) so the model reads them as a
/// run-up. Used to give the inventory-update LLM context for the turn
/// right before the clicked message — e.g. if the user says "thanks"
/// and you click that message, the pencil-handed-over line from the
/// turn before should still be visible.
fn fetch_run_up_before(
    conn: &rusqlite::Connection,
    anchor: &MessageAnchor,
    limit: usize,
    user_display_name: &str,
    active_character_name: &str,
) -> Vec<crate::db::queries::ConversationLine> {
    let table = if anchor.is_group { "group_messages" } else { "messages" };
    let sql = format!(
        "SELECT role, content, sender_character_id, created_at
         FROM {} WHERE thread_id = ?1 AND created_at < ?2
           AND role NOT IN ('illustration','video','system','context','inventory_update')
         ORDER BY created_at DESC LIMIT ?3",
        table
    );
    let mut out: Vec<crate::db::queries::ConversationLine> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map(
            rusqlite::params![anchor.thread_id, anchor.created_at, limit as i64],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            )),
        ) {
            for (role, content, sender, created_at) in rows.flatten() {
                // Resolve speaker the same way the anchor label does so the
                // run-up and the anchor quote are written in consistent
                // registers.
                let speaker = match role.as_str() {
                    "user" => user_display_name.to_string(),
                    "narrative" => "Narrative voice".to_string(),
                    "dream" => "Dream".to_string(),
                    _ => {
                        if let Some(cid) = sender.as_deref() {
                            get_character(conn, cid).map(|c| c.display_name)
                                .unwrap_or_else(|_| active_character_name.to_string())
                        } else {
                            active_character_name.to_string()
                        }
                    }
                };
                out.push(crate::db::queries::ConversationLine {
                    speaker,
                    content,
                    created_at,
                });
            }
        }
    }
    out.reverse();
    out
}

/// Resolve the "speaker label" for an anchor message. For user messages,
/// uses the user's display_name (falling back to "The human"). For an
/// assistant message, uses the sender character's display_name if we can
/// find it, otherwise the active character's name. For narrative, a
/// scene-voice label that reads right in the prompt.
fn anchor_speaker_label(
    conn: &rusqlite::Connection,
    role: &str,
    sender_character_id: Option<&str>,
    world_id: &str,
    active_character_name: &str,
) -> String {
    match role {
        "user" => get_user_profile(conn, world_id)
            .ok()
            .map(|p| p.display_name)
            .unwrap_or_else(|| "The human".to_string()),
        "narrative" => "Narrative voice".to_string(),
        _ => {
            if let Some(cid) = sender_character_id {
                if let Ok(c) = get_character(conn, cid) {
                    return c.display_name;
                }
            }
            active_character_name.to_string()
        }
    }
}

/// Core "update from moment" flow — bypasses the staleness gate, quotes
/// the clicked message for the model. When `allow_pure_maintain` is
/// false (user/assistant clicks, and solo-chat narrative) at least one
/// slot MUST change. When true (narrative clicked in a group chat,
/// per-member fan-out) the model is permitted to leave the inventory
/// untouched if the narrative doesn't reach this character.
async fn update_one_inventory_from_message(
    db: &Database,
    api_key: &str,
    character_id: &str,
    message_id: &str,
    allow_pure_maintain: bool,
) -> Result<InventoryRefreshResult, String> {
    if api_key.trim().is_empty() {
        return Err("no API key".to_string());
    }

    let (character, world, model_config, history, anchor_speaker, anchor_content, run_up) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let character = get_character(&conn, character_id).map_err(|e| e.to_string())?;
        let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
        let model_config = orchestrator::load_model_config(&conn);
        let user_name = get_user_profile(&conn, &character.world_id)
            .ok()
            .map(|p| p.display_name)
            .unwrap_or_else(|| "the human".to_string());
        let history = gather_character_recent_messages(
            &conn,
            &character.character_id,
            &user_name,
            INVENTORY_HISTORY_LIMIT,
        );
        let anchor = get_message_anchor(&conn, message_id)
            .ok_or_else(|| "Message not found for inventory update".to_string())?;
        let speaker = anchor_speaker_label(
            &conn, &anchor.role, anchor.sender_character_id.as_deref(),
            &character.world_id, &character.display_name,
        );
        // Pull 5 messages from the same thread immediately before the
        // anchor — the "run-up" that gives the LLM the immediate context
        // the clicked message may rely on (e.g. "thanks" after a pencil
        // was handed over the turn before).
        let run_up = fetch_run_up_before(
            &conn, &anchor, 5, &user_name, &character.display_name,
        );
        (character, world, model_config, history, speaker, anchor.content, run_up)
    };

    let today = current_world_day(&world);
    let prior_items = parse_inventory(&character.inventory);
    let base = model_config.chat_api_base();
    let model = &model_config.memory_model;

    let new_items = orchestrator::inventory_update_from_moment(
        &base, api_key, model,
        &character.display_name, &character.identity,
        &prior_items, &history,
        &run_up,
        &anchor_speaker, &anchor_content,
        allow_pure_maintain,
    ).await.map_err(|e| {
        log::warn!("[Inventory] moment-update for {} failed: {e}", character.display_name);
        e
    })?;

    // Diff prior vs new by item name (case-insensitive). Added / updated
    // carry the FULL item (name + description + kind) so the chat-history
    // card can render the fuller text with its nuances — not just the
    // short label. Removed carries names only since the item is gone.
    let norm = |s: &str| s.trim().to_lowercase();
    let prior_map: std::collections::HashMap<String, &InventoryItem> =
        prior_items.iter().map(|p| (norm(&p.name), p)).collect();
    let new_map: std::collections::HashMap<String, &InventoryItem> =
        new_items.iter().map(|p| (norm(&p.name), p)).collect();
    let mut added: Vec<InventoryItem> = Vec::new();
    let mut updated: Vec<InventoryItem> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    for n in &new_items {
        let key = norm(&n.name);
        match prior_map.get(&key) {
            None => added.push(n.clone()),
            Some(p) if p.description.trim() != n.description.trim() => updated.push(n.clone()),
            _ => {}
        }
    }
    for p in &prior_items {
        let key = norm(&p.name);
        if !new_map.contains_key(&key) {
            removed.push(p.name.clone());
        }
    }

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let _ = snapshot_inventory_pre_mutation(&conn, &character.character_id, "moment");
        let json = serde_json::to_value(&new_items).unwrap_or(Value::Array(vec![]));
        let _ = set_character_inventory(&conn, &character.character_id, &json, Some(today));
    }
    let _ = message_id; // anchor id no longer persisted — diff lives in the inventory_update message inserted by the dispatcher.

    log::info!(
        "[Inventory] moment-update for {} — {} items (anchor: {}) +{}/~{}/-{}",
        character.display_name, new_items.len(), anchor_speaker,
        added.len(), updated.len(), removed.len(),
    );

    Ok(InventoryRefreshResult {
        character_id: character.character_id.clone(),
        inventory: new_items,
        refreshed: true,
        mode: "moment".to_string(),
        added,
        updated,
        removed,
    })
}

/// Resolve who the inventory update should target based on the clicked
/// message. Returns a list of character_ids (one for most cases, many
/// for narrative-in-group fan-out).
///
/// Routing rules:
/// - **assistant**: the message's `sender_character_id` (the character who
///   spoke). Falls back to the solo-thread's character if sender is null.
/// - **user**: the addressee.
///   - Solo thread → the thread's single character.
///   - Group thread → run `detect_direct_address` first; if unambiguous,
///     use it. Otherwise fall back to `llm_pick_addressee` (same helper
///     that group chat uses to pick responders). Last resort: the
///     most-recent assistant speaker, or the first member.
/// - **narrative**: everyone in the chat.
///   - Solo → the one character.
///   - Group → fan out to all members.
pub struct ResolvedTargets {
    pub targets: Vec<(String, bool)>,
    pub thread_id: String,
    pub is_group: bool,
}

async fn resolve_inventory_targets(
    db: &Database,
    api_key: &str,
    message_id: &str,
) -> Result<ResolvedTargets, String> {
    use crate::commands::group_chat_cmds;

    // Load message + thread context up front so we can release the
    // mutex before any awaits.
    let (role, content, sender_character_id, thread_id, model_config, user_name, group_info, solo_char_id, members, recent, sender_of_last_assistant) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let (role, content, sender_character_id, thread_id) = conn.query_row(
            "SELECT role, content, sender_character_id, thread_id FROM messages WHERE message_id = ?1",
            rusqlite::params![message_id],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            )),
        ).or_else(|_| conn.query_row(
            "SELECT role, content, sender_character_id, thread_id FROM group_messages WHERE message_id = ?1",
            rusqlite::params![message_id],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            )),
        )).map_err(|_| "Message not found".to_string())?;

        // Is this thread a group chat? If so, get its member list.
        let group_info: Option<(String, Vec<String>)> = conn.query_row(
            "SELECT group_chat_id, character_ids FROM group_chats WHERE thread_id = ?1",
            rusqlite::params![&thread_id],
            |r| {
                let gc_id: String = r.get(0)?;
                let ids_json: String = r.get(1)?;
                let ids: Vec<String> = serde_json::from_str::<serde_json::Value>(&ids_json)
                    .ok()
                    .and_then(|v| v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()))
                    .unwrap_or_default();
                Ok((gc_id, ids))
            },
        ).ok();

        // Solo fallback: the thread's character_id (may be NULL in group threads).
        let solo_char_id: Option<String> = conn.query_row(
            "SELECT character_id FROM threads WHERE thread_id = ?1",
            rusqlite::params![&thread_id],
            |r| r.get::<_, Option<String>>(0),
        ).ok().flatten();

        let model_config = orchestrator::load_model_config(&conn);

        // For group user-message routing, we need Character structs + recent context.
        let (members, recent, sender_of_last_assistant): (Vec<Character>, Vec<Message>, Option<String>) = if let Some((_, ref ids)) = group_info {
            let chars: Vec<Character> = ids.iter()
                .filter_map(|id| get_character(&conn, id).ok())
                .collect();
            let world_id = chars.first().map(|c| c.world_id.clone()).unwrap_or_default();
            let recent: Vec<Message> = list_group_messages(&conn, &thread_id, 12).unwrap_or_default();
            let last_assistant = recent.iter().rev()
                .find(|m| m.role == "assistant")
                .and_then(|m| m.sender_character_id.clone());
            let _ = world_id;
            (chars, recent, last_assistant)
        } else {
            (Vec::new(), Vec::new(), None)
        };

        let world_id = if let Some((_, ref ids)) = group_info {
            ids.iter().filter_map(|id| get_character(&conn, id).ok()).next().map(|c| c.world_id)
        } else if let Some(ref cid) = solo_char_id {
            get_character(&conn, cid).ok().map(|c| c.world_id)
        } else {
            None
        };
        let user_name = world_id.and_then(|wid| get_user_profile(&conn, &wid).ok())
            .map(|p| p.display_name)
            .unwrap_or_else(|| "the human".to_string());

        (role, content, sender_character_id, thread_id, model_config, user_name, group_info, solo_char_id, members, recent, sender_of_last_assistant)
    };

    let is_group = group_info.is_some();

    // Each entry: (character_id, allow_pure_maintain). The flag is only
    // set on narrative-in-group fan-out, where the narrative may only
    // reach a subset of the characters present.
    let targets: Vec<(String, bool)> = match role.as_str() {
        "assistant" => {
            let id = sender_character_id.or(solo_char_id)
                .ok_or_else(|| "Assistant message has no sender character".to_string())?;
            vec![(id, false)]
        }
        "user" => {
            if !is_group {
                solo_char_id.map(|id| (id, false)).into_iter().collect()
            } else {
                // Try direct address first (fast path, no LLM cost).
                let direct = group_chat_cmds::detect_direct_address(&content, &members);
                let picked_id = if direct.len() == 1 {
                    Some(direct[0].clone())
                } else if !members.is_empty() {
                    let llm_pick = group_chat_cmds::llm_pick_addressee(
                        api_key, &model_config, &content, &recent, &members, &user_name, 8,
                    ).await;
                    llm_pick.or(sender_of_last_assistant).or_else(|| Some(members[0].character_id.clone()))
                } else {
                    None
                };
                match picked_id {
                    Some(id) => vec![(id, false)],
                    None => return Err("Group chat has no members".to_string()),
                }
            }
        }
        "narrative" => {
            if is_group {
                // Fan out to every member, but ALLOW pure-maintain: the
                // narrative may only name one or two of them, and the
                // untouched members shouldn't be forced to manufacture
                // a change. The prompt handles the branching inside.
                members.iter().map(|c| (c.character_id.clone(), true)).collect()
            } else {
                // Solo narrative still forces at least one change —
                // there's only one character, and the user clicked on
                // a narrative about them.
                solo_char_id.map(|id| (id, false)).into_iter().collect()
            }
        }
        other => return Err(format!("Inventory update not supported for role '{other}'")),
    };

    if targets.is_empty() {
        return Err("Could not resolve any inventory target for this message".to_string());
    }
    Ok(ResolvedTargets { targets, thread_id, is_group })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInventoryForMomentResponse {
    pub results: Vec<InventoryRefreshResult>,
    /// The inserted "[Inventory updated:] ..." message row that lives in
    /// chat history. None when every target was a pure-maintain on the
    /// narrative-in-group path (no character actually changed, so there
    /// was nothing to announce).
    pub new_message: Option<Message>,
}

/// One change entry in the JSON body of an inventory_update message.
/// Serialized as the message `content` so the frontend can render each
/// change with the item's full description, not just its short name.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventoryChangeEntry {
    pub character_name: String,
    /// "added" | "updated" | "swapped_out"
    pub action: String,
    pub name: String,
    /// Full item description. Empty string on "swapped_out" since the
    /// item is gone and there's no new description to show.
    #[serde(default)]
    pub description: String,
    /// "physical" | "interior" | "" (empty on swapped_out).
    #[serde(default)]
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventoryUpdateMessageBody {
    pub changes: Vec<InventoryChangeEntry>,
}

/// Build the JSON body of the "[Inventory updated:] ..." message.
/// Returns None when no character actually changed — the caller skips
/// the message insertion in that case. The content is ALWAYS emitted
/// as JSON prefixed with the verbatim "[Inventory updated:]\n" line so
/// that (a) the frontend card can parse structured data, and (b) any
/// fallback prose render still reads as an inventory-update message.
fn build_inventory_update_content(
    results: &[InventoryRefreshResult],
    id_to_name: &std::collections::HashMap<String, String>,
) -> Option<String> {
    let mut changes: Vec<InventoryChangeEntry> = Vec::new();
    for r in results {
        let name = id_to_name.get(&r.character_id).cloned().unwrap_or_else(|| "Character".to_string());
        for item in &r.added {
            changes.push(InventoryChangeEntry {
                character_name: name.clone(),
                action: "added".to_string(),
                name: item.name.clone(),
                description: item.description.clone(),
                kind: item.kind.clone(),
            });
        }
        for item in &r.updated {
            changes.push(InventoryChangeEntry {
                character_name: name.clone(),
                action: "updated".to_string(),
                name: item.name.clone(),
                description: item.description.clone(),
                kind: item.kind.clone(),
            });
        }
        // Show removes only when they outnumber adds — the balanced
        // ones are the implicit other-half of each swap that the
        // "added" entries already name.
        if r.removed.len() > r.added.len() {
            for removed_name in r.removed.iter().skip(r.added.len()) {
                changes.push(InventoryChangeEntry {
                    character_name: name.clone(),
                    action: "swapped_out".to_string(),
                    name: removed_name.clone(),
                    description: String::new(),
                    kind: String::new(),
                });
            }
        }
    }
    if changes.is_empty() { return None; }
    let body = InventoryUpdateMessageBody { changes };
    let json = serde_json::to_string(&body).ok()?;
    Some(format!("[Inventory updated:]\n{json}"))
}

/// Unified on-demand inventory update: routes based on the clicked
/// message's role and whether the chat is solo or group. See
/// `resolve_inventory_targets` for routing rules. Inserts a new
/// "[Inventory updated:]" message into the chat history and returns it.
#[tauri::command]
pub async fn update_inventory_for_moment_cmd(
    db: State<'_, Database>,
    api_key: String,
    message_id: String,
) -> Result<UpdateInventoryForMomentResponse, String> {
    let resolved = resolve_inventory_targets(&db, &api_key, &message_id).await?;
    let thread_id = resolved.thread_id.clone();
    let is_group = resolved.is_group;

    let db_ref: &Database = db.inner();
    let futs = resolved.targets.into_iter().map(|(cid, allow_maintain)| {
        let key = api_key.clone();
        let mid = message_id.clone();
        async move { update_one_inventory_from_message(db_ref, &key, &cid, &mid, allow_maintain).await }
    });
    let results_raw = futures_util::future::join_all(futs).await;
    let results: Vec<InventoryRefreshResult> = results_raw.into_iter().filter_map(|r| r.ok()).collect();

    // Resolve display names for every character that was updated.
    let id_to_name: std::collections::HashMap<String, String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        results.iter()
            .filter_map(|r| get_character(&conn, &r.character_id).ok().map(|c| (c.character_id, c.display_name)))
            .collect()
    };

    let content = build_inventory_update_content(&results, &id_to_name);
    let new_message: Option<Message> = if let Some(body) = content {
        let now = chrono::Utc::now().to_rfc3339();
        let msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread_id.clone(),
            role: "inventory_update".to_string(),
            content: body,
            tokens_estimate: 0,
            sender_character_id: None,
            created_at: now,
            world_day: None,
            world_time: None,
            address_to: None,
            mood_chain: None,
            is_proactive: false,
        };
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let insert = if is_group {
            create_group_message(&conn, &msg)
        } else {
            create_message(&conn, &msg)
        };
        match insert {
            Ok(_) => Some(msg),
            Err(e) => {
                log::warn!("[Inventory] failed to insert inventory_update message: {e}");
                None
            }
        }
    } else { None };

    // Persist shorthand diff records keyed to the TRIGGER message so the
    // frontend can paint an "Inventory updated · +X, ~Y" badge under that
    // message across reloads. One row per character actually changed.
    // The table is `ON CONFLICT DO UPDATE`, so re-clicking overwrites the
    // old record rather than accumulating.
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        for r in &results {
            let added_names: Vec<String> = r.added.iter().map(|i| i.name.clone()).collect();
            let updated_names: Vec<String> = r.updated.iter().map(|i| i.name.clone()).collect();
            if let Err(e) = crate::db::queries::record_inventory_update(
                &conn, &message_id, &r.character_id,
                &added_names, &updated_names, &r.removed,
            ) {
                log::warn!("[Inventory] record_inventory_update failed for {}: {e}", r.character_id);
            }
        }
    }

    Ok(UpdateInventoryForMomentResponse { results, new_message })
}

/// Fetch every inventory-update record for the given message_ids.
/// Returns a flat vec; the frontend groups by `message_id`. Used to
/// paint "Inventory updated · +X, ~Y" badges under any trigger message
/// that has produced an update, persisted across sessions.
#[tauri::command]
pub fn get_inventory_updates_for_messages_cmd(
    db: State<'_, Database>,
    message_ids: Vec<String>,
) -> Result<Vec<crate::db::queries::InventoryUpdateRecord>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_inventory_updates_for_messages(&conn, &message_ids)
        .map_err(|e| e.to_string())
}

/// User-edit entry point from the character settings page. Blindly
/// stores whatever the user typed (trimmed + capped to max items).
/// Stamps `last_inventory_day` to the current world-day so the
/// refresh check doesn't blow the user's edit away on next focus.
#[tauri::command]
pub fn set_character_inventory_cmd(
    db: State<'_, Database>,
    character_id: String,
    inventory: Vec<InventoryItem>,
) -> Result<Vec<InventoryItem>, String> {
    // Normalize kind, drop empties, preserve order (physicals first,
    // then interiors) so the strip / prompt render stays predictable.
    // Total capped at INVENTORY_MAX_ITEMS — no per-kind cap; the mix
    // is a soft rule enforced in the LLM prompt, not here.
    let mut phys: Vec<InventoryItem> = Vec::new();
    let mut inter: Vec<InventoryItem> = Vec::new();
    for it in inventory.into_iter() {
        let kind = if it.kind.trim().eq_ignore_ascii_case(orchestrator::INVENTORY_KIND_INTERIOR) {
            orchestrator::INVENTORY_KIND_INTERIOR.to_string()
        } else {
            orchestrator::INVENTORY_KIND_PHYSICAL.to_string()
        };
        let normalized = InventoryItem {
            name: it.name.trim().to_string(),
            description: it.description.trim().to_string(),
            kind,
        };
        if normalized.name.is_empty() { continue; }
        if normalized.kind == orchestrator::INVENTORY_KIND_INTERIOR {
            inter.push(normalized);
        } else {
            phys.push(normalized);
        }
    }
    let mut cleaned: Vec<InventoryItem> = phys;
    cleaned.extend(inter);
    if cleaned.len() > orchestrator::INVENTORY_MAX_ITEMS {
        cleaned.truncate(orchestrator::INVENTORY_MAX_ITEMS);
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let character = get_character(&conn, &character_id).map_err(|e| e.to_string())?;
    let world = get_world(&conn, &character.world_id).map_err(|e| e.to_string())?;
    let today = current_world_day(&world);
    let _ = snapshot_inventory_pre_mutation(&conn, &character_id, "user_edit");
    let json = serde_json::to_value(&cleaned).unwrap_or(Value::Array(vec![]));
    set_character_inventory(&conn, &character_id, &json, Some(today))
        .map_err(|e| e.to_string())?;
    Ok(cleaned)
}

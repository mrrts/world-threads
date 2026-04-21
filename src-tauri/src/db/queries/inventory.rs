use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

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
             WHERE thread_id = ?1 AND role NOT IN ('illustration', 'video', 'system', 'context', 'inventory_update')
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
             WHERE thread_id = ?1 AND role NOT IN ('illustration', 'video', 'system', 'context', 'inventory_update')
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

// ─── Inventory update records ───────────────────────────────────────────
//
// One row per (message_id, character_id) pair recording the shorthand
// diff from a moment-anchored inventory update. Used for the PERSISTENT
// BADGE under any trigger message showing "Inventory updated · added X,
// updated Y" — name-only, no descriptions. The detailed version lives
// in the inventory_update message card that also gets inserted into
// chat history; this table is purely for badging the trigger message
// so the user can see at a glance which messages they've already used
// to update inventory.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryUpdateRecord {
    pub message_id: String,
    pub character_id: String,
    /// Display name of the character at record-fetch time. Denormalized
    /// for the frontend's convenience so the badge can read "added X to
    /// Darren" without a second lookup. Falls back to empty string when
    /// the character was deleted after the record was written (the FK
    /// cascade should remove such orphans, but the column is still
    /// emitted as empty to keep the shape stable).
    pub character_name: String,
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub removed: Vec<String>,
    pub created_at: String,
}

/// Write (or replace) the record for one (message_id, character_id) pair.
/// Safe no-op when all three lists are empty — we don't want phantom
/// "no-change" records cluttering the UI.
pub fn record_inventory_update(
    conn: &Connection,
    message_id: &str,
    character_id: &str,
    added: &[String],
    updated: &[String],
    removed: &[String],
) -> Result<(), rusqlite::Error> {
    if added.is_empty() && updated.is_empty() && removed.is_empty() {
        return Ok(());
    }
    let added_json = serde_json::to_string(added).unwrap_or_else(|_| "[]".to_string());
    let updated_json = serde_json::to_string(updated).unwrap_or_else(|_| "[]".to_string());
    let removed_json = serde_json::to_string(removed).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO inventory_update_records (message_id, character_id, added, updated, removed, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
         ON CONFLICT(message_id, character_id) DO UPDATE SET
           added = excluded.added,
           updated = excluded.updated,
           removed = excluded.removed,
           created_at = excluded.created_at",
        params![message_id, character_id, added_json, updated_json, removed_json],
    )?;
    Ok(())
}

// ─── Inventory snapshots (time-travel ledger) ───────────────────────────
//
// One row per pre-mutation state, keyed by snapshot_id (UUID). Written
// BEFORE each inventory mutation so a snapshot at time T represents
// "what the inventory was just before the change at T." The reset path
// picks the latest snapshot whose `created_at <= target.created_at` and
// restores it — world and keeping rewind together.
//
// Capped at MAX_SNAPSHOTS_PER_CHARACTER via trim-on-insert. Old rows
// fall off quietly; we don't surface the history anywhere, so old
// snapshots only exist to serve resets in the window they cover.

const MAX_SNAPSHOTS_PER_CHARACTER: i64 = 50;

#[derive(Debug, Clone)]
pub struct InventorySnapshotData {
    /// Inventory JSON as stored (same shape as characters.inventory).
    pub inventory_json: String,
    pub last_inventory_day: Option<i64>,
}

/// Read the character's CURRENT inventory + last_inventory_day, insert
/// a snapshot row with the given `trigger` label, and trim older rows
/// beyond the cap. A no-op if the character row isn't found. Callers
/// should run this BEFORE writing the new inventory so the snapshot
/// captures the state about to be overwritten.
pub fn snapshot_inventory_pre_mutation(
    conn: &Connection,
    character_id: &str,
    trigger: &str,
) -> Result<(), rusqlite::Error> {
    let current: Option<(String, Option<i64>)> = conn
        .query_row(
            "SELECT inventory, last_inventory_day FROM characters WHERE character_id = ?1",
            params![character_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?)),
        )
        .ok();
    let Some((inv_json, last_day)) = current else { return Ok(()); };

    let snapshot_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO character_inventory_snapshots
           (snapshot_id, character_id, inventory, last_inventory_day, created_at, trigger)
         VALUES (?1, ?2, ?3, ?4, datetime('now'), ?5)",
        params![snapshot_id, character_id, inv_json, last_day, trigger],
    )?;

    // Trim to newest N per character. Use a rowid-scoped DELETE so we
    // don't accidentally nuke rows from other characters.
    conn.execute(
        "DELETE FROM character_inventory_snapshots
         WHERE character_id = ?1
           AND snapshot_id NOT IN (
               SELECT snapshot_id FROM character_inventory_snapshots
               WHERE character_id = ?1
               ORDER BY created_at DESC
               LIMIT ?2
           )",
        params![character_id, MAX_SNAPSHOTS_PER_CHARACTER],
    )?;

    Ok(())
}

/// Find the latest snapshot for this character with created_at <= the
/// given ISO timestamp. Returns None when no snapshot precedes that
/// time — caller should leave the inventory alone in that case.
pub fn get_inventory_snapshot_at_or_before(
    conn: &Connection,
    character_id: &str,
    at_or_before_iso: &str,
) -> Option<InventorySnapshotData> {
    conn.query_row(
        "SELECT inventory, last_inventory_day FROM character_inventory_snapshots
         WHERE character_id = ?1 AND created_at <= ?2
         ORDER BY created_at DESC LIMIT 1",
        params![character_id, at_or_before_iso],
        |r| Ok(InventorySnapshotData {
            inventory_json: r.get::<_, String>(0)?,
            last_inventory_day: r.get::<_, Option<i64>>(1)?,
        }),
    ).ok()
}

/// Fetch all records whose message_id is in `message_ids`. Returns a
/// flat vector — the frontend groups by message_id itself. Batched via
/// a single query with a dynamic IN clause.
pub fn get_inventory_updates_for_messages(
    conn: &Connection,
    message_ids: &[String],
) -> Result<Vec<InventoryUpdateRecord>, rusqlite::Error> {
    if message_ids.is_empty() { return Ok(Vec::new()); }
    let placeholders = message_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT r.message_id, r.character_id, COALESCE(c.display_name, ''), r.added, r.updated, r.removed, r.created_at
         FROM inventory_update_records r
         LEFT JOIN characters c ON c.character_id = r.character_id
         WHERE r.message_id IN ({placeholders})
         ORDER BY r.created_at DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let params_vec: Vec<&dyn rusqlite::ToSql> = message_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    let rows = stmt.query_map(&params_vec[..], |r| {
        let added_json: String = r.get(3)?;
        let updated_json: String = r.get(4)?;
        let removed_json: String = r.get(5)?;
        Ok(InventoryUpdateRecord {
            message_id: r.get(0)?,
            character_id: r.get(1)?,
            character_name: r.get(2)?,
            added: serde_json::from_str(&added_json).unwrap_or_default(),
            updated: serde_json::from_str(&updated_json).unwrap_or_default(),
            removed: serde_json::from_str(&removed_json).unwrap_or_default(),
            created_at: r.get(6)?,
        })
    })?;
    rows.collect()
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

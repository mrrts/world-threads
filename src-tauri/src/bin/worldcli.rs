//! worldcli — direct character access for craft work.
//!
//! Out-of-band tool used by Claude Code to converse with the user's
//! characters and inspect db state WITHOUT the exchange appearing in
//! the UI. Built around the "ask the character" pattern codified in
//! CLAUDE.md: developer asks a character a meta question in their own
//! voice; character generates the principle in-register; developer
//! lifts it into prompts.rs as a craft note.
//!
//! The binary reuses the workspace lib (`app_lib`) wholesale — same
//! prompt-building pipeline as the Tauri app, so character voice and
//! behavior match what the user sees in their UI.
//!
//! Sessions persist to a separate `dev_chat_sessions` /
//! `dev_chat_messages` schema that the UI never reads from. Safe to
//! create freely without polluting the user's chat history, kept
//! records, journals, or any visible surface.

use clap::{Parser, Subcommand};
use rusqlite::params;
use std::path::PathBuf;

use app_lib::ai::{openai, orchestrator, prompts};
use app_lib::ai::prompts::json_array_to_strings;
use app_lib::db::{queries::*, Database};

#[derive(Parser)]
#[command(
    name = "worldcli",
    about = "Direct character access for craft work (Claude Code dev tool)",
    long_about = "Out-of-band CLI for conversing with WorldThreads characters and \
                  inspecting db state. Used by Claude Code to mine craft material \
                  via the 'ask the character' pattern. Exchanges are persisted to a \
                  dev-only schema invisible to the UI."
)]
struct Cli {
    /// Path to worldthreads.db. Defaults to the macOS app data dir.
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    /// OpenAI API key. Default: OPENAI_API_KEY env var.
    #[arg(long, global = true)]
    api_key: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List all worlds in the db.
    ListWorlds,

    /// List characters, optionally filtered by world.
    ListCharacters {
        #[arg(long)]
        world: Option<String>,
    },

    /// Show full character record (identity, voice rules, boundaries, etc.).
    ShowCharacter {
        character_id: String,
    },

    /// Show full world record (description, weather, time, invariants).
    ShowWorld {
        world_id: String,
    },

    /// Recent messages in a character's solo thread (newest last).
    RecentMessages {
        character_id: String,
        #[arg(long, default_value_t = 30)]
        limit: i64,
    },

    /// All kept_records (canon entries) for a character.
    KeptRecords {
        character_id: String,
    },

    /// A character's journal entries (newest first).
    Journals {
        character_id: String,
    },

    /// Active quests in a world.
    Quests {
        #[arg(long)]
        world: Option<String>,
    },

    /// Ask a character a single message. Streams the reply to stdout.
    /// If --session is given, persists to a named dev-session for follow-ups.
    Ask {
        character_id: String,
        message: String,
        /// Persist this exchange to a named dev-session so future asks can build on it.
        #[arg(long)]
        session: Option<String>,
    },

    /// Show a dev-session's conversation so far.
    SessionShow {
        name: String,
    },

    /// Clear a dev-session's history (keeps the session row).
    SessionClear {
        name: String,
    },

    /// List all dev-sessions.
    SessionList,
}

fn default_db_path() -> PathBuf {
    // macOS default. Other platforms: user must pass --db.
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("com.worldthreads.app")
        .join("worldthreads.db")
}

/// Read the user's OpenAI API key from the macOS keychain at the
/// documented dev-CLI location. Returns None if the key isn't there
/// or `security` isn't available (non-macOS platforms).
///
/// User adds the key once with:
///   security add-generic-password -s "WorldThreadsCLI" -a "openai" -w "<sk-...>"
fn read_api_key_from_keychain() -> Option<String> {
    let out = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s", "WorldThreadsCLI",
            "-a", "openai",
            "-w",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let key = String::from_utf8(out.stdout).ok()?;
    let trimmed = key.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

/// Resolve API key with this precedence:
///   1. Explicit --api-key flag
///   2. OPENAI_API_KEY env var
///   3. macOS keychain (service "WorldThreadsCLI", account "openai")
fn resolve_api_key(flag: Option<&str>) -> Option<String> {
    if let Some(k) = flag {
        let t = k.trim();
        if !t.is_empty() { return Some(t.to_string()); }
    }
    if let Ok(k) = std::env::var("OPENAI_API_KEY") {
        let t = k.trim();
        if !t.is_empty() { return Some(t.to_string()); }
    }
    read_api_key_from_keychain()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);
    if !db_path.exists() {
        return Err(format!(
            "Database not found at {}. Pass --db <path> or run the WorldThreads app once to create it.",
            db_path.display()
        )
        .into());
    }

    let db = Database::open(&db_path)?;

    match cli.cmd {
        Cmd::ListWorlds => cmd_list_worlds(&db),
        Cmd::ListCharacters { world } => cmd_list_characters(&db, world.as_deref()),
        Cmd::ShowCharacter { character_id } => cmd_show_character(&db, &character_id),
        Cmd::ShowWorld { world_id } => cmd_show_world(&db, &world_id),
        Cmd::RecentMessages { character_id, limit } => cmd_recent_messages(&db, &character_id, limit),
        Cmd::KeptRecords { character_id } => cmd_kept_records(&db, &character_id),
        Cmd::Journals { character_id } => cmd_journals(&db, &character_id),
        Cmd::Quests { world } => cmd_quests(&db, world.as_deref()),
        Cmd::Ask { character_id, message, session } => {
            let api_key = resolve_api_key(cli.api_key.as_deref())
                .ok_or("No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"")?;
            cmd_ask(&db, &api_key, &character_id, &message, session.as_deref()).await
        }
        Cmd::SessionShow { name } => cmd_session_show(&db, &name),
        Cmd::SessionClear { name } => cmd_session_clear(&db, &name),
        Cmd::SessionList => cmd_session_list(&db),
    }
}

// ─── Read-only context queries ──────────────────────────────────────────

fn cmd_list_worlds(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let worlds = list_worlds(&conn)?;
    if worlds.is_empty() {
        println!("(no worlds)");
        return Ok(());
    }
    for w in worlds {
        println!("{}\t{}", w.world_id, w.name);
    }
    Ok(())
}

fn cmd_list_characters(db: &Database, world: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let world_ids: Vec<String> = match world {
        Some(w) => vec![w.to_string()],
        None => list_worlds(&conn)?.into_iter().map(|w| w.world_id).collect(),
    };
    for wid in world_ids {
        let chars = list_characters(&conn, &wid)?;
        for c in chars {
            // tab-separated for easy parsing
            let archived_tag = if c.is_archived { " [archived]" } else { "" };
            println!("{}\t{}\t{}{}", c.character_id, wid, c.display_name, archived_tag);
        }
    }
    Ok(())
}

fn cmd_show_character(db: &Database, character_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let c = get_character(&conn, character_id)?;
    println!("character_id: {}", c.character_id);
    println!("display_name: {}", c.display_name);
    println!("world_id: {}", c.world_id);
    println!("sex: {}", c.sex);
    println!("archived: {}", c.is_archived);
    println!();
    println!("# IDENTITY");
    println!("{}", c.identity);
    let voice = json_array_to_strings(&c.voice_rules);
    if !voice.is_empty() {
        println!("\n# VOICE RULES");
        for r in voice { println!("- {}", r); }
    }
    let bounds = json_array_to_strings(&c.boundaries);
    if !bounds.is_empty() {
        println!("\n# BOUNDARIES");
        for b in bounds { println!("- {}", b); }
    }
    let backstory = json_array_to_strings(&c.backstory_facts);
    if !backstory.is_empty() {
        println!("\n# BACKSTORY FACTS");
        for f in backstory { println!("- {}", f); }
    }
    if !c.visual_description.is_empty() {
        println!("\n# VISUAL DESCRIPTION");
        println!("{}", c.visual_description);
    }
    Ok(())
}

fn cmd_show_world(db: &Database, world_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let w = get_world(&conn, world_id)?;
    println!("world_id: {}", w.world_id);
    println!("name: {}", w.name);
    if let Some(t) = w.state.get("time") {
        println!("time: {}", serde_json::to_string(t).unwrap_or_default());
    }
    if let Some(weather) = w.state.get("weather").and_then(|v| v.as_str()) {
        println!("weather: {}", weather);
    }
    println!();
    println!("# DESCRIPTION");
    println!("{}", w.description);
    let invariants = json_array_to_strings(&w.invariants);
    if !invariants.is_empty() {
        println!("\n# INVARIANTS");
        for inv in invariants { println!("- {}", inv); }
    }
    Ok(())
}

fn cmd_recent_messages(db: &Database, character_id: &str, limit: i64) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let thread = get_thread_for_character(&conn, character_id)?;
    let mut msgs = list_messages(&conn, &thread.thread_id, limit)?;
    // list_messages returns newest first; reverse for chronological print.
    msgs.reverse();
    for m in msgs {
        // Truncate role tag for readability
        let tag = match m.role.as_str() {
            "user" => "USER".to_string(),
            "assistant" => "CHAR".to_string(),
            other => other.to_uppercase(),
        };
        println!("[{}] {}: {}", m.created_at, tag, m.content.chars().take(400).collect::<String>());
    }
    Ok(())
}

fn cmd_kept_records(db: &Database, character_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT kept_id, record_type, content, source_world_day, created_at \
         FROM kept_records WHERE subject_type = 'character' AND subject_id = ?1 \
         ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![character_id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<i64>>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    for row in rows.flatten() {
        let (id, kind, content, day, created_at) = row;
        let day_str = day.map(|d| format!(" day {}", d)).unwrap_or_default();
        println!("[{}] {}{} ({})", created_at, kind, day_str, id);
        println!("  {}", content.chars().take(500).collect::<String>());
    }
    Ok(())
}

fn cmd_journals(db: &Database, character_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let entries = list_journal_entries(&conn, character_id, 20).unwrap_or_default();
    for e in entries {
        println!("--- Day {} ({}) ---", e.world_day, e.created_at);
        println!("{}", e.content);
        println!();
    }
    Ok(())
}

fn cmd_quests(db: &Database, world: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let world_ids: Vec<String> = match world {
        Some(w) => vec![w.to_string()],
        None => list_worlds(&conn)?.into_iter().map(|w| w.world_id).collect(),
    };
    for wid in world_ids {
        let quests = list_quests(&conn, &wid).unwrap_or_default();
        if quests.is_empty() { continue; }
        println!("=== World {} ===", wid);
        for q in quests {
            let status = if q.completed_at.is_some() { "DONE" }
                else if q.abandoned_at.is_some() { "ABANDONED" }
                else { "ACTIVE" };
            println!("[{}] {}\t{}", status, q.quest_id, q.title);
            if !q.description.trim().is_empty() {
                println!("  {}", q.description.chars().take(300).collect::<String>());
            }
        }
    }
    Ok(())
}

// ─── The main thing: ask a character ────────────────────────────────────

async fn cmd_ask(
    db: &Database,
    api_key: &str,
    character_id: &str,
    message: &str,
    session: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build all the prompt context inside one lock-acquire, drop the
    // lock before the async API call, then re-acquire if we need to
    // persist to a session.
    let (system_prompt, model_config, prior_messages, session_id) = {
        let conn = db.conn.lock().unwrap();
        let character = get_character(&conn, character_id)?;
        let world = get_world(&conn, &character.world_id)?;
        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let recent_journals = list_journal_entries(&conn, character_id, 1).unwrap_or_default();
        let active_quests = list_active_quests(&conn, &character.world_id).unwrap_or_default();

        let system_prompt = prompts::build_dialogue_system_prompt(
            &world,
            &character,
            user_profile.as_ref(),
            None, // mood_directive
            Some("Auto"), // response_length — let model pick for dev work
            None, // group_context
            None, // tone
            false, // local_model
            &[],  // mood_chain
            None, // leader
            &recent_journals,
            None, // latest_reading
            &[],  // own_voice_samples — skipped for v1
            None, // latest_meanwhile — skipped for v1
            &active_quests,
        );

        let model_config = orchestrator::load_model_config(&conn);

        // Resolve session and prior messages if --session was given.
        let (session_id, prior_messages): (Option<String>, Vec<(String, String)>) = match session {
            None => (None, Vec::new()),
            Some(name) => {
                let existing: Option<String> = conn.query_row(
                    "SELECT session_id FROM dev_chat_sessions WHERE name = ?1",
                    params![name], |r| r.get(0),
                ).ok();
                let id = match existing {
                    Some(id) => id,
                    None => {
                        let new_id = uuid::Uuid::new_v4().to_string();
                        conn.execute(
                            "INSERT INTO dev_chat_sessions (session_id, name, character_id) VALUES (?1, ?2, ?3)",
                            params![new_id, name, character_id],
                        )?;
                        new_id
                    }
                };
                let mut stmt = conn.prepare(
                    "SELECT role, content FROM dev_chat_messages \
                     WHERE session_id = ?1 ORDER BY created_at ASC"
                )?;
                let rows: Vec<(String, String)> = stmt
                    .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?)))?
                    .filter_map(|r| r.ok())
                    .collect();
                (Some(id), rows)
            }
        };

        (system_prompt, model_config, prior_messages, session_id)
    };

    // Build request: system + prior session turns + new user message.
    let mut messages = vec![openai::ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
    }];
    for (role, content) in prior_messages {
        messages.push(openai::ChatMessage { role, content });
    }
    messages.push(openai::ChatMessage {
        role: "user".to_string(),
        content: message.to_string(),
    });

    let request = openai::ChatRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.95),
        max_completion_tokens: None,
        response_format: None,
    };

    let response = openai::chat_completion_with_base(
        &model_config.chat_api_base(),
        api_key,
        &request,
    ).await?;

    let reply = response
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    println!("{}", reply);

    if let Some(id) = session_id {
        let conn = db.conn.lock().unwrap();
        let user_msg_id = uuid::Uuid::new_v4().to_string();
        let asst_msg_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO dev_chat_messages (message_id, session_id, role, content) VALUES (?1, ?2, 'user', ?3)",
            params![user_msg_id, id, message],
        )?;
        conn.execute(
            "INSERT INTO dev_chat_messages (message_id, session_id, role, content) VALUES (?1, ?2, 'assistant', ?3)",
            params![asst_msg_id, id, reply],
        )?;
    }

    Ok(())
}

// ─── Session management ─────────────────────────────────────────────────

fn cmd_session_show(db: &Database, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let session_id: Option<String> = conn.query_row(
        "SELECT session_id FROM dev_chat_sessions WHERE name = ?1",
        params![name], |r| r.get(0),
    ).ok();
    let Some(session_id) = session_id else {
        println!("(no session named '{}')", name);
        return Ok(());
    };
    let mut stmt = conn.prepare(
        "SELECT role, content, created_at FROM dev_chat_messages \
         WHERE session_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![session_id], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
    })?;
    for row in rows.flatten() {
        let (role, content, created_at) = row;
        let tag = match role.as_str() {
            "user" => "YOU",
            "assistant" => "CHAR",
            other => other,
        };
        println!("[{}] {}: {}", created_at, tag, content);
        println!();
    }
    Ok(())
}

fn cmd_session_clear(db: &Database, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let session_id: Option<String> = conn.query_row(
        "SELECT session_id FROM dev_chat_sessions WHERE name = ?1",
        params![name], |r| r.get(0),
    ).ok();
    if let Some(id) = session_id {
        let n = conn.execute("DELETE FROM dev_chat_messages WHERE session_id = ?1", params![id])?;
        println!("Cleared {} messages from session '{}' (session row preserved).", n, name);
    } else {
        println!("(no session named '{}')", name);
    }
    Ok(())
}

fn cmd_session_list(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT s.name, s.character_id, s.created_at, COUNT(m.message_id) as msg_count \
         FROM dev_chat_sessions s \
         LEFT JOIN dev_chat_messages m ON m.session_id = s.session_id \
         GROUP BY s.session_id ORDER BY s.created_at DESC"
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, i64>(3)?,
        ))
    })?;
    for row in rows.flatten() {
        let (name, character_id, created_at, count) = row;
        let cid = character_id.unwrap_or_else(|| "(none)".to_string());
        println!("{}\t{}\t{}\t{} msgs", name, cid, created_at, count);
    }
    Ok(())
}

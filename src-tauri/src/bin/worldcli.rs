//! worldcli — direct character access for craft work.
//!
//! Out-of-band tool used by Claude Code to converse with the user's
//! characters and inspect db state WITHOUT the exchange appearing in
//! the UI. Designed for AGENT ergonomics, not human ergonomics:
//! machine-readable JSON output, structured errors with retry hints,
//! per-call cost surfacing, persisted run logs so prior investigations
//! can be searched.
//!
//! ## Roles in the project
//!
//! Three reflective surfaces ship in this repo:
//! - `reports/` — interpretive reads of the project's git history
//! - the harness — automated testing of prompt behavior
//! - **this CLI** — empirical querying of the user's lived corpus,
//!   queryable on demand to ground prompt work in real data
//!
//! ## Safety posture
//!
//! - **Read-only by default** for user data. The only writes are to
//!   the `dev_chat_*` schema (invisible to the UI) and to the worldcli
//!   home dir at ~/.worldcli/.
//! - **Scope-gated**: by default, only worlds listed in
//!   ~/.worldcli/config.json are accessible. `--scope full` opts in to
//!   the entire corpus (prints a warning).
//! - **Cost-gated**: every `ask` call projects cost first; calls above
//!   per-call or daily caps require `--confirm-cost <usd>` to proceed.

use clap::{Parser, Subcommand, ValueEnum};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::path::PathBuf;

use app_lib::ai::prompts::json_array_to_strings;
use app_lib::ai::{openai, orchestrator, prompts, relational_stance};
use app_lib::db::{queries::*, Database};

// ─── CLI surface ────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "worldcli",
    about = "Direct character access for craft work (Claude Code dev tool)",
    long_about = "A third reflective surface alongside reports/ and the harness — \
                  empirical querying of the user's lived WorldThreads corpus, on demand. \
                  Designed for agent ergonomics: --json output, scope gating, cost surfacing, \
                  persisted run logs."
)]
struct Cli {
    /// Path to worldthreads.db. Defaults to the macOS app data dir.
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    /// OpenAI API key. Default: OPENAI_API_KEY env var, then macOS keychain.
    #[arg(long, global = true)]
    api_key: Option<String>,

    /// Scope mode for read access to worlds/characters/messages. Default
    /// is "config" — only worlds listed in ~/.worldcli/config.json.
    /// "full" opens the entire corpus (prints a warning).
    #[arg(long, value_enum, global = true, default_value_t = Scope::Config)]
    scope: Scope,

    /// Emit machine-readable JSON instead of human-readable text. Every
    /// read command supports this; honors agent-ergonomics priority.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Scope {
    /// Use ~/.worldcli/config.json's scope.world_ids list.
    Config,
    /// Open the entire corpus. Prints a warning.
    Full,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print scope + budget + paths so the agent knows what's accessible.
    Status,

    /// Print a starter ~/.worldcli/config.json template (does NOT overwrite).
    ConfigTemplate,

    // ── read commands ──
    /// List worlds in scope.
    ListWorlds,

    /// List characters, optionally filtered by world.
    ListCharacters {
        #[arg(long)]
        world: Option<String>,
    },

    /// Show full character record (identity, voice rules, boundaries).
    ShowCharacter { character_id: String },

    /// Show full world record.
    ShowWorld { world_id: String },

    /// Recent messages in a character's solo thread, with optional
    /// query primitives for ad-hoc filtering.
    RecentMessages {
        character_id: String,
        /// How many messages to retrieve (newest first, then re-sorted asc).
        #[arg(long, default_value_t = 30)]
        limit: i64,
        /// Case-insensitive substring filter on message content. Combine
        /// with --with-context to expand around hits.
        #[arg(long)]
        grep: Option<String>,
        /// ISO 8601 cutoff — only messages BEFORE this time.
        #[arg(long)]
        before: Option<String>,
        /// ISO 8601 cutoff — only messages AFTER this time.
        #[arg(long)]
        after: Option<String>,
        /// When --grep matches, expand the result to include this many
        /// surrounding messages on each side of every match.
        #[arg(long, default_value_t = 0)]
        with_context: usize,
    },

    /// All kept_records (canon entries) for a character.
    KeptRecords { character_id: String },

    /// A character's journal entries.
    Journals { character_id: String },

    /// Active quests, optionally filtered by world.
    Quests {
        #[arg(long)]
        world: Option<String>,
    },

    /// List group chats, optionally filtered by world.
    ListGroupChats {
        #[arg(long)]
        world: Option<String>,
    },

    /// Recent messages in a group chat, with the same query primitives
    /// as `recent-messages`.
    GroupMessages {
        group_chat_id: String,
        #[arg(long, default_value_t = 30)]
        limit: i64,
        #[arg(long)]
        grep: Option<String>,
        #[arg(long)]
        before: Option<String>,
        #[arg(long)]
        after: Option<String>,
        #[arg(long, default_value_t = 0)]
        with_context: usize,
    },

    /// Show the latest relational stance for a character (and a small
    /// history if --history > 1). Stances are the per-character
    /// synthesized "what they've come to know of the user" prose,
    /// generated by the relational_stance pipeline. Read-only.
    ShowStance {
        character_id: String,
        #[arg(long, default_value_t = 1)]
        history: i64,
    },

    /// Manually trigger a stance synthesis for a character. Costs ~1
    /// memory-model call. Use to bootstrap a character's first stance
    /// without waiting for the auto triggers (canonization commit,
    /// first dialogue of new in-world day). Subject to the same cost
    /// caps as `ask`.
    RefreshStance {
        character_id: String,
        /// Override the model used for synthesis (default: memory_model
        /// from the user's settings — typically gpt-4o-mini).
        #[arg(long)]
        model: Option<String>,
        /// Required when projected cost exceeds the per-call cap.
        #[arg(long)]
        confirm_cost: Option<f64>,
    },

    /// Sample messages from before and after a git ref, so prompt
    /// changes can be evaluated against the corpus as a natural
    /// experiment. The ref's commit timestamp is the cutoff: most
    /// recent N messages before, earliest N messages after. When
    /// `--end-ref` is given, the after-window starts at THAT ref's
    /// timestamp instead — useful for skipping a noisy in-between
    /// range when a series of commits A..B together form the change.
    SampleWindows {
        /// Git ref (sha, tag, branch) marking the boundary commit.
        #[arg(long = "ref")]
        git_ref: String,
        /// Optional second ref. After-window starts here instead of at `--ref`.
        #[arg(long)]
        end_ref: Option<String>,
        /// Messages per window (most recent N before; earliest N after).
        #[arg(long, default_value_t = 30)]
        limit: i64,
        /// Restrict to a single character (matches solo thread owner OR group sender).
        #[arg(long)]
        character: Option<String>,
        /// Restrict to a single world (otherwise: all worlds in scope).
        #[arg(long)]
        world: Option<String>,
        /// Role filter. Default 'assistant' (the surface most affected by
        /// prompt changes). Use 'any' for no filter, or 'user' / 'narrative' / etc.
        #[arg(long, default_value = "assistant")]
        role: String,
        /// Exclude group-chat messages (only sample solo threads).
        #[arg(long, conflicts_with = "groups_only")]
        solo_only: bool,
        /// Exclude solo-thread messages (only sample group chats).
        #[arg(long)]
        groups_only: bool,
        /// Path to git repo for ref resolution. Defaults to current working dir.
        #[arg(long)]
        repo: Option<PathBuf>,
    },

    // ── ask path (the load-bearing one) ──
    /// Ask a character a single message. Cost-gated; logs to runs/.
    Ask {
        character_id: String,
        message: String,
        /// Persist this exchange to a named dev-session for follow-ups.
        #[arg(long)]
        session: Option<String>,
        /// Override the configured model (defaults to user's dialogue_model setting).
        #[arg(long)]
        model: Option<String>,
        /// Required when projected cost exceeds the per-call cap or the
        /// remaining daily budget. Set to a USD ceiling you accept.
        #[arg(long)]
        confirm_cost: Option<f64>,
        /// Free-form summary of why this question is being asked. Stored
        /// in the run log so future you can grep for prior explorations.
        #[arg(long)]
        question_summary: Option<String>,
    },

    // ── runs (read your own prior investigations) ──
    /// List recent runs (most recent first).
    RunsList {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show a single run by id (or short prefix).
    RunsShow { id: String },
    /// Search runs by substring across prompt/reply/summary/character.
    RunsSearch { query: String },

    // ── session management ──
    /// Show a dev-session's conversation so far.
    SessionShow { name: String },
    /// Clear a dev-session's history.
    SessionClear { name: String },
    /// List all dev-sessions.
    SessionList,
}

// ─── Config / homedir layout ────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CliConfig {
    #[serde(default)]
    scope: ScopeConfig,
    #[serde(default)]
    budget: BudgetConfig,
    #[serde(default)]
    model_pricing: ModelPricing,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct ScopeConfig {
    /// World ids accessible in default scope. Empty = nothing accessible
    /// without --scope full.
    #[serde(default)]
    world_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BudgetConfig {
    /// Hard ceiling per single ask call. Above this, --confirm-cost required.
    pub per_call_usd: f64,
    /// Rolling 24-hour ceiling. Above this, --confirm-cost required.
    pub daily_usd: f64,
}
impl Default for BudgetConfig {
    fn default() -> Self {
        Self { per_call_usd: 0.10, daily_usd: 5.00 }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ModelPricing {
    /// Map model name → (input price per 1M tokens, output price per 1M tokens).
    /// Defaults populated for common OpenAI models as of April 2026.
    #[serde(default)]
    pub models: std::collections::HashMap<String, ModelPrice>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct ModelPrice {
    pub input_per_1m: f64,
    pub output_per_1m: f64,
}
impl Default for ModelPricing {
    fn default() -> Self {
        let mut m = std::collections::HashMap::new();
        m.insert("gpt-4o".to_string(), ModelPrice { input_per_1m: 5.0, output_per_1m: 15.0 });
        m.insert("gpt-4o-mini".to_string(), ModelPrice { input_per_1m: 0.15, output_per_1m: 0.60 });
        m.insert("gpt-5".to_string(), ModelPrice { input_per_1m: 10.0, output_per_1m: 30.0 });
        m.insert("gpt-5-mini".to_string(), ModelPrice { input_per_1m: 0.30, output_per_1m: 1.20 });
        Self { models: m }
    }
}

fn worldcli_home() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".worldcli")
}

fn config_path() -> PathBuf { worldcli_home().join("config.json") }
fn runs_dir() -> PathBuf { worldcli_home().join("runs") }
fn manifest_path() -> PathBuf { runs_dir().join("manifest.jsonl") }
fn cost_log_path() -> PathBuf { worldcli_home().join("cost.jsonl") }

fn load_config() -> CliConfig {
    let path = config_path();
    if !path.exists() {
        return CliConfig {
            scope: ScopeConfig::default(),
            budget: BudgetConfig::default(),
            model_pricing: ModelPricing::default(),
        };
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            eprintln!("warning: config at {} failed to parse ({}); using defaults", path.display(), e);
            CliConfig {
                scope: ScopeConfig::default(),
                budget: BudgetConfig::default(),
                model_pricing: ModelPricing::default(),
            }
        }),
        Err(_) => CliConfig {
            scope: ScopeConfig::default(),
            budget: BudgetConfig::default(),
            model_pricing: ModelPricing::default(),
        },
    }
}

fn config_template_text() -> String {
    let template = json!({
        "_README": [
            "worldcli config. Edit and save to ~/.worldcli/config.json.",
            "scope.world_ids = list of worlds Claude Code can read by default.",
            "Use 'worldcli list-worlds --scope full' once to see all your world IDs,",
            "then add the ones you're OK with the agent freely exploring."
        ],
        "scope": {
            "world_ids": []
        },
        "budget": {
            "per_call_usd": 0.10,
            "daily_usd": 5.00
        },
        "model_pricing": {
            "models": {
                "gpt-4o": { "input_per_1m": 5.0, "output_per_1m": 15.0 },
                "gpt-4o-mini": { "input_per_1m": 0.15, "output_per_1m": 0.60 }
            }
        }
    });
    serde_json::to_string_pretty(&template).unwrap()
}

// ─── Cost tracking ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct CostEntry {
    timestamp: String,
    model: String,
    prompt_tokens: i64,
    completion_tokens: i64,
    usd: f64,
}

fn estimate_tokens(text: &str) -> i64 {
    // Rough: ~3.5 chars per token for English. Slight overestimate for safety.
    ((text.chars().count() as f64) / 3.3).ceil() as i64
}

fn project_cost(model: &str, prompt_tokens: i64, expected_completion_tokens: i64, pricing: &ModelPricing) -> f64 {
    let p = pricing.models.get(model).copied().unwrap_or(ModelPrice {
        // Unknown-model fallback: assume gpt-4o pricing (conservative — likely overestimate).
        input_per_1m: 5.0,
        output_per_1m: 15.0,
    });
    (prompt_tokens as f64) * p.input_per_1m / 1_000_000.0
        + (expected_completion_tokens as f64) * p.output_per_1m / 1_000_000.0
}

fn actual_cost(model: &str, prompt_tokens: i64, completion_tokens: i64, pricing: &ModelPricing) -> f64 {
    project_cost(model, prompt_tokens, completion_tokens, pricing)
}

fn append_cost_log(entry: &CostEntry) {
    let _ = std::fs::create_dir_all(worldcli_home());
    let line = serde_json::to_string(entry).unwrap_or_default();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(cost_log_path())
    {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

fn rolling_24h_total_usd() -> f64 {
    let path = cost_log_path();
    let Ok(content) = std::fs::read_to_string(&path) else { return 0.0; };
    let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
    let mut total = 0.0;
    for line in content.lines() {
        let Ok(e): Result<CostEntry, _> = serde_json::from_str(line) else { continue; };
        let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&e.timestamp) else { continue; };
        if ts.with_timezone(&chrono::Utc) >= cutoff {
            total += e.usd;
        }
    }
    total
}

// ─── Run log ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RunRecord {
    id: String,
    timestamp: String,
    character_id: String,
    character_name: String,
    world_id: String,
    model: String,
    session: Option<String>,
    question_summary: Option<String>,
    prompt: String,
    reply: String,
    prompt_tokens: i64,
    completion_tokens: i64,
    usd: f64,
}

fn write_run(record: &RunRecord) {
    let dir = runs_dir();
    let _ = std::fs::create_dir_all(&dir);
    // Per-run file: full record
    let per_path = dir.join(format!("{}.json", record.id));
    if let Ok(s) = serde_json::to_string_pretty(record) {
        let _ = std::fs::write(&per_path, s);
    }
    // Manifest: compact one-line summary for grepping
    let manifest_entry = json!({
        "id": record.id,
        "ts": record.timestamp,
        "character_id": record.character_id,
        "character_name": record.character_name,
        "world_id": record.world_id,
        "model": record.model,
        "session": record.session,
        "question_summary": record.question_summary,
        "prompt_preview": record.prompt.chars().take(160).collect::<String>(),
        "reply_preview": record.reply.chars().take(160).collect::<String>(),
        "prompt_tokens": record.prompt_tokens,
        "completion_tokens": record.completion_tokens,
        "usd": record.usd,
    });
    let line = serde_json::to_string(&manifest_entry).unwrap_or_default();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(manifest_path())
    {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

fn read_manifest() -> Vec<JsonValue> {
    let Ok(content) = std::fs::read_to_string(manifest_path()) else { return Vec::new(); };
    content.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

// ─── Scope enforcement ──────────────────────────────────────────────────

struct Resolved {
    db: Database,
    cfg: CliConfig,
    scope: Scope,
    json: bool,
}

impl Resolved {
    fn world_in_scope(&self, world_id: &str) -> bool {
        match self.scope {
            Scope::Full => true,
            Scope::Config => self.cfg.scope.world_ids.iter().any(|w| w == world_id),
        }
    }

    fn check_world(&self, world_id: &str) -> Result<(), CliError> {
        if self.world_in_scope(world_id) { return Ok(()); }
        Err(CliError::OutOfScope {
            world_id: world_id.to_string(),
            scope_world_ids: self.cfg.scope.world_ids.clone(),
        })
    }

    fn check_character(&self, character_id: &str) -> Result<String, CliError> {
        let conn = self.db.conn.lock().unwrap();
        let c = get_character(&conn, character_id)
            .map_err(|e| CliError::NotFound(format!("character {}: {}", character_id, e)))?;
        drop(conn);
        self.check_world(&c.world_id)?;
        Ok(c.world_id)
    }
}

#[derive(Debug)]
enum CliError {
    OutOfScope { world_id: String, scope_world_ids: Vec<String> },
    NotFound(String),
    Budget {
        kind: String, // "per_call" | "daily"
        projected_usd: f64,
        cap_usd: f64,
        confirm_at_least: f64,
    },
    Other(String),
}
impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::OutOfScope { world_id, scope_world_ids } => {
                write!(f, "world {} is out of scope. Scope contains: {:?}. Re-run with --scope full to override.", world_id, scope_world_ids)
            }
            CliError::NotFound(s) => write!(f, "not found: {}", s),
            CliError::Budget { kind, projected_usd, cap_usd, confirm_at_least } => {
                write!(f, "{} cap exceeded: projected ${:.4} > cap ${:.2}. Re-run with --confirm-cost {:.2} to proceed.", kind, projected_usd, cap_usd, confirm_at_least)
            }
            CliError::Other(s) => write!(f, "{}", s),
        }
    }
}
impl std::error::Error for CliError {}

// ─── API key resolution (unchanged from v1) ─────────────────────────────

fn read_api_key_from_keychain() -> Option<String> {
    let out = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "WorldThreadsCLI", "-a", "openai", "-w"])
        .output()
        .ok()?;
    if !out.status.success() { return None; }
    let key = String::from_utf8(out.stdout).ok()?;
    let trimmed = key.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

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

fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("com.worldthreads.app")
        .join("worldthreads.db")
}

// ─── Output helpers ─────────────────────────────────────────────────────

fn emit(json_mode: bool, value: JsonValue) {
    if json_mode {
        println!("{}", serde_json::to_string(&value).unwrap_or_default());
    } else {
        // Best-effort human render
        if let Some(arr) = value.as_array() {
            for item in arr { emit_one(item); }
        } else {
            emit_one(&value);
        }
    }
}

fn emit_one(value: &JsonValue) {
    match value {
        JsonValue::Object(map) => {
            for (k, v) in map {
                let v_str = match v {
                    JsonValue::String(s) => s.clone(),
                    other => other.to_string(),
                };
                if v_str.contains('\n') {
                    println!("# {}\n{}", k, v_str);
                } else {
                    println!("{}: {}", k, v_str);
                }
            }
            println!();
        }
        other => println!("{}", other),
    }
}

// ─── Main ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cfg = load_config();

    // Print scope warning early when --scope full is in use
    if matches!(cli.scope, Scope::Full) && !matches!(cli.cmd, Cmd::Status | Cmd::ConfigTemplate) {
        eprintln!("[worldcli] WARNING: --scope full is in use; the entire corpus is accessible. Default scope is config-only.");
    }

    let db_path = cli.db.clone().unwrap_or_else(default_db_path);
    if !db_path.exists() {
        return Err(Box::<dyn std::error::Error>::from(format!(
            "Database not found at {}. Pass --db <path> or run the WorldThreads app once to create it.",
            db_path.display()
        )));
    }

    let db = Database::open(&db_path)?;
    let r = Resolved { db, cfg, scope: cli.scope, json: cli.json };

    match cli.cmd {
        Cmd::Status => cmd_status(&r),
        Cmd::ConfigTemplate => { println!("{}", config_template_text()); Ok(()) }
        Cmd::ListWorlds => cmd_list_worlds(&r),
        Cmd::ListCharacters { world } => cmd_list_characters(&r, world.as_deref()),
        Cmd::ShowCharacter { character_id } => cmd_show_character(&r, &character_id),
        Cmd::ShowWorld { world_id } => cmd_show_world(&r, &world_id),
        Cmd::RecentMessages { character_id, limit, grep, before, after, with_context } => {
            cmd_recent_messages(&r, &character_id, limit, grep.as_deref(), before.as_deref(), after.as_deref(), with_context)
        }
        Cmd::KeptRecords { character_id } => cmd_kept_records(&r, &character_id),
        Cmd::Journals { character_id } => cmd_journals(&r, &character_id),
        Cmd::Quests { world } => cmd_quests(&r, world.as_deref()),
        Cmd::ListGroupChats { world } => cmd_list_group_chats(&r, world.as_deref()),
        Cmd::GroupMessages { group_chat_id, limit, grep, before, after, with_context } => {
            cmd_group_messages(&r, &group_chat_id, limit, grep.as_deref(), before.as_deref(), after.as_deref(), with_context)
        }
        Cmd::SampleWindows { git_ref, end_ref, limit, character, world, role, solo_only, groups_only, repo } => {
            cmd_sample_windows(&r, &git_ref, end_ref.as_deref(), limit, character.as_deref(), world.as_deref(), &role, solo_only, groups_only, repo.as_deref())
        }
        Cmd::ShowStance { character_id, history } => cmd_show_stance(&r, &character_id, history),
        Cmd::RefreshStance { character_id, model, confirm_cost } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_refresh_stance(&r, &api_key, &character_id, model.as_deref(), confirm_cost).await
        }
        Cmd::Ask { character_id, message, session, model, confirm_cost, question_summary } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_ask(&r, &api_key, &character_id, &message, session.as_deref(), model.as_deref(), confirm_cost, question_summary.as_deref()).await
        }
        Cmd::RunsList { limit } => cmd_runs_list(&r, limit),
        Cmd::RunsShow { id } => cmd_runs_show(&r, &id),
        Cmd::RunsSearch { query } => cmd_runs_search(&r, &query),
        Cmd::SessionShow { name } => cmd_session_show(&r, &name),
        Cmd::SessionClear { name } => cmd_session_clear(&r, &name),
        Cmd::SessionList => cmd_session_list(&r),
    }
}

// ─── Status / config ────────────────────────────────────────────────────

fn cmd_status(r: &Resolved) -> Result<(), Box<dyn std::error::Error>> {
    let daily = rolling_24h_total_usd();
    let manifest_count = read_manifest().len();
    let v = json!({
        "config_path": config_path().display().to_string(),
        "config_exists": config_path().exists(),
        "scope_world_ids": r.cfg.scope.world_ids,
        "active_scope": match r.scope { Scope::Config => "config", Scope::Full => "full" },
        "budget": {
            "per_call_usd": r.cfg.budget.per_call_usd,
            "daily_usd": r.cfg.budget.daily_usd,
            "rolling_24h_spent_usd": daily,
            "rolling_24h_remaining_usd": (r.cfg.budget.daily_usd - daily).max(0.0),
        },
        "runs_total": manifest_count,
        "runs_dir": runs_dir().display().to_string(),
        "cost_log": cost_log_path().display().to_string(),
    });
    emit(r.json, v);
    if !r.json && !config_path().exists() {
        eprintln!("\nNote: config file does not exist at {}.", config_path().display());
        eprintln!("Run `worldcli config-template > {}` then edit to set scope.", config_path().display());
    }
    Ok(())
}

// ─── Read commands ──────────────────────────────────────────────────────

fn cmd_list_worlds(r: &Resolved) -> Result<(), Box<dyn std::error::Error>> {
    let conn = r.db.conn.lock().unwrap();
    let worlds = list_worlds(&conn)?;
    let in_scope: Vec<JsonValue> = worlds.iter()
        .filter(|w| r.world_in_scope(&w.world_id))
        .map(|w| json!({
            "world_id": w.world_id,
            "name": w.name,
            "in_scope": true,
        }))
        .collect();
    emit(r.json, JsonValue::Array(in_scope));
    if !r.json && matches!(r.scope, Scope::Config) && r.cfg.scope.world_ids.is_empty() {
        eprintln!("\nNote: scope is empty. Edit {} to add world IDs, or use --scope full.", config_path().display());
    }
    Ok(())
}

fn cmd_list_characters(r: &Resolved, world: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = r.db.conn.lock().unwrap();
    let world_ids: Vec<String> = match world {
        Some(w) => { r.check_world(w)?; vec![w.to_string()] }
        None => {
            list_worlds(&conn)?.into_iter()
                .filter(|w| r.world_in_scope(&w.world_id))
                .map(|w| w.world_id).collect()
        }
    };
    let mut out: Vec<JsonValue> = Vec::new();
    for wid in world_ids {
        let chars = list_characters(&conn, &wid)?;
        for c in chars {
            out.push(json!({
                "character_id": c.character_id,
                "world_id": wid,
                "display_name": c.display_name,
                "archived": c.is_archived,
            }));
        }
    }
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

fn cmd_show_character(r: &Resolved, character_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;
    let conn = r.db.conn.lock().unwrap();
    let c = get_character(&conn, character_id)?;
    let v = json!({
        "character_id": c.character_id,
        "display_name": c.display_name,
        "world_id": c.world_id,
        "sex": c.sex,
        "archived": c.is_archived,
        "identity": c.identity,
        "voice_rules": json_array_to_strings(&c.voice_rules),
        "boundaries": json_array_to_strings(&c.boundaries),
        "backstory_facts": json_array_to_strings(&c.backstory_facts),
        "visual_description": c.visual_description,
    });
    emit(r.json, v);
    Ok(())
}

fn cmd_show_world(r: &Resolved, world_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    r.check_world(world_id)?;
    let conn = r.db.conn.lock().unwrap();
    let w = get_world(&conn, world_id)?;
    let v = json!({
        "world_id": w.world_id,
        "name": w.name,
        "description": w.description,
        "invariants": json_array_to_strings(&w.invariants),
        "state": w.state,
    });
    emit(r.json, v);
    Ok(())
}

fn cmd_recent_messages(
    r: &Resolved,
    character_id: &str,
    limit: i64,
    grep: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
    with_context: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;
    let conn = r.db.conn.lock().unwrap();
    let thread = get_thread_for_character(&conn, character_id)?;

    // Pull a working set: limit (after grep) needs more raw rows when
    // we're filtering. Heuristic: grep / time-window → pull 5x the limit
    // so the filtered/windowed set still hits limit.
    let raw_pull = if grep.is_some() || before.is_some() || after.is_some() {
        (limit * 5).max(100)
    } else {
        limit
    };
    let mut msgs = list_messages(&conn, &thread.thread_id, raw_pull)?;
    msgs.reverse(); // chronological asc

    // Apply time filters
    if let Some(b) = before {
        msgs.retain(|m| m.created_at.as_str() < b);
    }
    if let Some(a) = after {
        msgs.retain(|m| m.created_at.as_str() > a);
    }

    // Apply grep + context-window
    let filtered_indices: Vec<usize> = if let Some(g) = grep {
        let g_lc = g.to_lowercase();
        let hits: Vec<usize> = msgs.iter().enumerate()
            .filter(|(_, m)| m.content.to_lowercase().contains(&g_lc))
            .map(|(i, _)| i)
            .collect();
        if with_context == 0 {
            hits
        } else {
            // expand each hit by ±with_context, dedupe, sort
            let mut set = std::collections::BTreeSet::new();
            for h in hits {
                let lo = h.saturating_sub(with_context);
                let hi = (h + with_context + 1).min(msgs.len());
                for i in lo..hi { set.insert(i); }
            }
            set.into_iter().collect()
        }
    } else {
        (0..msgs.len()).collect()
    };

    // Final cap
    let final_indices: Vec<usize> = if grep.is_some() || before.is_some() || after.is_some() {
        // For filtered queries, take the last `limit` matches (most recent)
        let len = filtered_indices.len();
        let start = len.saturating_sub(limit as usize);
        filtered_indices[start..].to_vec()
    } else {
        let len = filtered_indices.len();
        let start = len.saturating_sub(limit as usize);
        filtered_indices[start..].to_vec()
    };

    let out: Vec<JsonValue> = final_indices.iter().map(|&i| {
        let m = &msgs[i];
        json!({
            "message_id": m.message_id,
            "thread_id": m.thread_id,
            "role": m.role,
            "sender_character_id": m.sender_character_id,
            "created_at": m.created_at,
            "world_day": m.world_day,
            "world_time": m.world_time,
            "content": m.content,
        })
    }).collect();
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

fn cmd_kept_records(r: &Resolved, character_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;
    let conn = r.db.conn.lock().unwrap();
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
    let out: Vec<JsonValue> = rows.flatten().map(|(id, kind, content, day, ts)| json!({
        "kept_id": id, "record_type": kind, "content": content,
        "source_world_day": day, "created_at": ts,
    })).collect();
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

fn cmd_journals(r: &Resolved, character_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;
    let conn = r.db.conn.lock().unwrap();
    let entries = list_journal_entries(&conn, character_id, 50).unwrap_or_default();
    let out: Vec<JsonValue> = entries.iter().map(|e| json!({
        "world_day": e.world_day, "created_at": e.created_at, "content": e.content,
    })).collect();
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

fn cmd_quests(r: &Resolved, world: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = r.db.conn.lock().unwrap();
    let world_ids: Vec<String> = match world {
        Some(w) => { r.check_world(w)?; vec![w.to_string()] }
        None => {
            list_worlds(&conn)?.into_iter()
                .filter(|w| r.world_in_scope(&w.world_id))
                .map(|w| w.world_id).collect()
        }
    };
    let mut out: Vec<JsonValue> = Vec::new();
    for wid in world_ids {
        let quests = list_quests(&conn, &wid).unwrap_or_default();
        for q in quests {
            let status = if q.completed_at.is_some() { "completed" }
                else if q.abandoned_at.is_some() { "abandoned" } else { "active" };
            out.push(json!({
                "quest_id": q.quest_id, "world_id": wid, "title": q.title,
                "description": q.description, "status": status,
                "accepted_at": q.accepted_at,
            }));
        }
    }
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

fn cmd_list_group_chats(r: &Resolved, world: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let conn = r.db.conn.lock().unwrap();
    let world_ids: Vec<String> = match world {
        Some(w) => { r.check_world(w)?; vec![w.to_string()] }
        None => list_worlds(&conn)?.into_iter()
            .filter(|w| r.world_in_scope(&w.world_id))
            .map(|w| w.world_id).collect(),
    };
    // Build a character-id → display_name lookup so we can render member names.
    let mut id_to_name = std::collections::HashMap::new();
    for wid in &world_ids {
        for c in list_characters(&conn, wid).unwrap_or_default() {
            id_to_name.insert(c.character_id, c.display_name);
        }
    }
    let mut out = Vec::new();
    for wid in &world_ids {
        for gc in list_group_chats(&conn, wid).unwrap_or_default() {
            let member_ids: Vec<String> = gc.character_ids.as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let member_names: Vec<String> = member_ids.iter()
                .map(|id| id_to_name.get(id).cloned().unwrap_or_else(|| id.clone()))
                .collect();
            out.push(json!({
                "group_chat_id": gc.group_chat_id,
                "world_id": gc.world_id,
                "thread_id": gc.thread_id,
                "display_name": gc.display_name,
                "member_ids": member_ids,
                "member_names": member_names,
                "created_at": gc.created_at,
            }));
        }
    }
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

fn cmd_group_messages(
    r: &Resolved,
    group_chat_id: &str,
    limit: i64,
    grep: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
    with_context: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = r.db.conn.lock().unwrap();
    let gc = get_group_chat(&conn, group_chat_id)
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("group_chat {}: {}", group_chat_id, e)))?;
    r.check_world(&gc.world_id)?;
    // Build sender-id → display_name for label rendering.
    let mut id_to_name = std::collections::HashMap::new();
    for c in list_characters(&conn, &gc.world_id).unwrap_or_default() {
        id_to_name.insert(c.character_id, c.display_name);
    }
    let raw_pull = if grep.is_some() || before.is_some() || after.is_some() {
        (limit * 5).max(100)
    } else { limit };
    let mut msgs = list_group_messages(&conn, &gc.thread_id, raw_pull)?;
    msgs.reverse(); // chronological asc
    if let Some(b) = before { msgs.retain(|m| m.created_at.as_str() < b); }
    if let Some(a) = after { msgs.retain(|m| m.created_at.as_str() > a); }
    let filtered_indices: Vec<usize> = if let Some(g) = grep {
        let g_lc = g.to_lowercase();
        let hits: Vec<usize> = msgs.iter().enumerate()
            .filter(|(_, m)| m.content.to_lowercase().contains(&g_lc))
            .map(|(i, _)| i).collect();
        if with_context == 0 { hits } else {
            let mut set = std::collections::BTreeSet::new();
            for h in hits {
                let lo = h.saturating_sub(with_context);
                let hi = (h + with_context + 1).min(msgs.len());
                for i in lo..hi { set.insert(i); }
            }
            set.into_iter().collect()
        }
    } else { (0..msgs.len()).collect() };
    let len = filtered_indices.len();
    let start = len.saturating_sub(limit as usize);
    let final_indices = &filtered_indices[start..];

    let out: Vec<JsonValue> = final_indices.iter().map(|&i| {
        let m = &msgs[i];
        let sender_name = m.sender_character_id.as_ref()
            .and_then(|id| id_to_name.get(id))
            .cloned()
            .unwrap_or_else(|| match m.role.as_str() {
                "user" => "USER".to_string(),
                other => other.to_uppercase(),
            });
        json!({
            "message_id": m.message_id,
            "role": m.role,
            "sender_character_id": m.sender_character_id,
            "sender_name": sender_name,
            "created_at": m.created_at,
            "world_day": m.world_day,
            "world_time": m.world_time,
            "content": m.content,
        })
    }).collect();
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

// ─── Relational stance inspect + manual refresh ─────────────────────────

fn cmd_show_stance(r: &Resolved, character_id: &str, history: i64) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;
    let conn = r.db.conn.lock().unwrap();
    let stances = list_relational_stances(&conn, character_id, history)?;
    let out: Vec<JsonValue> = stances.iter().map(|s| json!({
        "stance_id": s.stance_id,
        "character_id": s.character_id,
        "world_id": s.world_id,
        "stance_text": s.stance_text,
        "world_day_at_generation": s.world_day_at_generation,
        "source_kept_record_count": s.source_kept_record_count,
        "source_journal_count": s.source_journal_count,
        "source_message_count": s.source_message_count,
        "refresh_trigger": s.refresh_trigger,
        "model_used": s.model_used,
        "created_at": s.created_at,
    })).collect();
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

async fn cmd_refresh_stance(
    r: &Resolved,
    api_key: &str,
    character_id: &str,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;

    // Pick model + cap-check (memory_model by default — synthesis is
    // 1 small call; gpt-4o-mini @ ~$0.001-0.005 typical).
    let model_config = {
        let conn = r.db.conn.lock().unwrap();
        orchestrator::load_model_config(&conn)
    };
    let model = model_override.unwrap_or(&model_config.memory_model).to_string();

    // Conservative pre-check: assume up to 3000 input tokens (kept_records
    // + journals + recent excerpts) and ~300 output tokens.
    let projected_usd = project_cost(&model, 3000, 300, &r.cfg.model_pricing);
    let per_call_cap = r.cfg.budget.per_call_usd;
    let confirm = confirm_cost.unwrap_or(0.0);
    if projected_usd > per_call_cap && confirm < projected_usd {
        return Err(Box::new(CliError::Budget {
            kind: "per_call".to_string(),
            projected_usd,
            cap_usd: per_call_cap,
            confirm_at_least: (projected_usd * 1.05).max(0.01),
        }));
    }

    if !r.json {
        eprintln!(
            "[worldcli] refreshing stance for {} via {} (projected≈${:.4})",
            character_id, model, projected_usd
        );
    }

    let base_url = model_config.chat_api_base();
    let res: Result<(), String> = relational_stance::refresh_relational_stance(
        r.db.conn.clone(),
        base_url,
        api_key.to_string(),
        model,
        character_id.to_string(),
        "manual_cli".to_string(),
    ).await;
    if let Err(e) = res {
        return Err(Box::<dyn std::error::Error>::from(e));
    }

    // Echo the freshly-written stance.
    cmd_show_stance(r, character_id, 1)
}

// ─── Sample-windows (natural-experiment evaluation) ─────────────────────

/// Resolve a git ref to (full_sha, committer_iso_date, subject) by
/// shelling out to `git log -1`. The repo path defaults to cwd when
/// `repo` is None, since the user typically runs worldcli from the
/// project root. Surfaces git's stderr verbatim on failure so the
/// caller can see why the ref didn't resolve.
fn git_resolve_ref(
    repo: Option<&std::path::Path>,
    git_ref: &str,
) -> Result<(String, String, String), CliError> {
    let mut cmd = std::process::Command::new("git");
    if let Some(p) = repo {
        cmd.args(["-C", &p.display().to_string()]);
    }
    cmd.args(["log", "-1", "--format=%H%x09%cI%x09%s", git_ref]);
    let out = cmd.output().map_err(|e| {
        CliError::Other(format!("git invocation failed: {} (is git on PATH?)", e))
    })?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(CliError::Other(format!(
            "git ref '{}' did not resolve: {}",
            git_ref, err
        )));
    }
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let parts: Vec<&str> = line.splitn(3, '\t').collect();
    if parts.len() < 3 {
        return Err(CliError::Other(format!(
            "git log returned unexpected format for '{}': {}",
            git_ref, line
        )));
    }
    Ok((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
}

/// Quote-strip user-controlled fragments before they hit a SQL string-build.
/// IDs in this DB are UUIDs; we still strip apostrophes as belt-and-suspenders.
fn sql_safe_id(s: &str) -> String { s.replace('\'', "") }

fn cmd_sample_windows(
    r: &Resolved,
    git_ref: &str,
    end_ref: Option<&str>,
    limit: i64,
    character: Option<&str>,
    world: Option<&str>,
    role: &str,
    solo_only: bool,
    groups_only: bool,
    repo: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve refs first; cheaper to fail before opening any cursors.
    let (before_sha, before_ts, before_subject) = git_resolve_ref(repo, git_ref)?;
    let (after_sha, after_ts, after_subject) = match end_ref {
        Some(er) => git_resolve_ref(repo, er)?,
        None => (before_sha.clone(), before_ts.clone(), before_subject.clone()),
    };

    // Character scope-check first — it acquires its own lock, so we do it
    // before grabbing the long-held one for the sampling queries.
    if let Some(c) = character {
        let _ = r.check_character(c)?;
    }

    let conn = r.db.conn.lock().unwrap();

    // World scope
    let world_ids: Vec<String> = match world {
        Some(w) => { r.check_world(w)?; vec![w.to_string()] }
        None => list_worlds(&conn)?
            .into_iter()
            .filter(|w| r.world_in_scope(&w.world_id))
            .map(|w| w.world_id)
            .collect(),
    };
    if world_ids.is_empty() {
        return Err(Box::new(CliError::Other(
            "No worlds in scope. Add to ~/.worldcli/config.json or pass --scope full.".to_string(),
        )));
    }

    // Build a sender-id → display_name lookup across all in-scope worlds
    let mut id_to_name = std::collections::HashMap::new();
    for wid in &world_ids {
        for c in list_characters(&conn, wid).unwrap_or_default() {
            id_to_name.insert(c.character_id, c.display_name);
        }
    }

    // SQL fragments. UUIDs from the db are interpolated; the user-supplied
    // character_id passes through sql_safe_id and is also UUID-shaped.
    let world_in_clause = format!(
        "({})",
        world_ids.iter().map(|w| format!("'{}'", sql_safe_id(w))).collect::<Vec<_>>().join(",")
    );
    let role_clause = if role == "any" {
        String::new()
    } else {
        format!("AND m.role = '{}'", role.replace('\'', ""))
    };
    let exclude_noise = "AND m.role NOT IN ('illustration', 'video', 'inventory_update', 'imagined_chapter', 'narrative', 'system')";
    let solo_char_clause = match character {
        Some(c) => format!("AND t.character_id = '{}'", sql_safe_id(c)),
        None => String::new(),
    };
    let group_sender_clause = match character {
        Some(c) => format!("AND m.sender_character_id = '{}'", sql_safe_id(c)),
        None => String::new(),
    };

    // ─── Pull a window: solo + group, merge, sort, truncate to `limit` ───
    let pull_window = |cutoff_ts: &str, direction: &str| -> Result<Vec<JsonValue>, rusqlite::Error> {
        // direction: "before" → m.created_at < cutoff, ORDER DESC
        //            "after"  → m.created_at >= cutoff, ORDER ASC
        let (op, order) = if direction == "before" { ("<", "DESC") } else { (">=", "ASC") };
        let mut out: Vec<JsonValue> = Vec::new();

        if !groups_only {
            let q = format!(
                "SELECT m.message_id, m.thread_id, m.role, m.content, \
                        m.sender_character_id, m.created_at, m.world_day, m.world_time, \
                        t.character_id, t.world_id \
                 FROM messages m JOIN threads t ON t.thread_id = m.thread_id \
                 WHERE t.world_id IN {worlds} AND t.character_id IS NOT NULL \
                 AND m.created_at {op} ?1 {role_c} {noise} {char_c} \
                 ORDER BY m.created_at {order} LIMIT ?2",
                worlds = world_in_clause,
                op = op,
                order = order,
                role_c = role_clause,
                noise = exclude_noise,
                char_c = solo_char_clause,
            );
            let mut stmt = conn.prepare(&q)?;
            let rows = stmt.query_map(params![cutoff_ts, limit], |row| {
                Ok((
                    row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?, row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?, row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<String>>(7)?, row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                ))
            })?;
            for row in rows.flatten() {
                let (mid, tid, role_s, content, sender, ts, day, time, thread_char, wid) = row;
                let sender_name = sender.as_ref().and_then(|id| id_to_name.get(id)).cloned()
                    .or_else(|| thread_char.as_ref().and_then(|id| id_to_name.get(id)).cloned())
                    .unwrap_or_else(|| match role_s.as_str() {
                        "user" => "USER".to_string(),
                        other => other.to_uppercase(),
                    });
                out.push(json!({
                    "surface": "solo",
                    "message_id": mid,
                    "thread_id": tid,
                    "world_id": wid,
                    "thread_character_id": thread_char,
                    "role": role_s,
                    "sender_character_id": sender,
                    "sender_name": sender_name,
                    "created_at": ts,
                    "world_day": day,
                    "world_time": time,
                    "content": content,
                }));
            }
        }

        if !solo_only {
            let q = format!(
                "SELECT m.message_id, m.thread_id, m.role, m.content, \
                        m.sender_character_id, m.created_at, m.world_day, m.world_time, \
                        gc.group_chat_id, gc.world_id, gc.display_name \
                 FROM group_messages m JOIN group_chats gc ON gc.thread_id = m.thread_id \
                 WHERE gc.world_id IN {worlds} \
                 AND m.created_at {op} ?1 {role_c} {noise} {char_c} \
                 ORDER BY m.created_at {order} LIMIT ?2",
                worlds = world_in_clause,
                op = op,
                order = order,
                role_c = role_clause,
                noise = exclude_noise,
                char_c = group_sender_clause,
            );
            let mut stmt = conn.prepare(&q)?;
            let rows = stmt.query_map(params![cutoff_ts, limit], |row| {
                Ok((
                    row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?, row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?, row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<String>>(7)?, row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?, row.get::<_, String>(10)?,
                ))
            })?;
            for row in rows.flatten() {
                let (mid, tid, role_s, content, sender, ts, day, time, gcid, wid, gcname) = row;
                let sender_name = sender.as_ref().and_then(|id| id_to_name.get(id)).cloned()
                    .unwrap_or_else(|| match role_s.as_str() {
                        "user" => "USER".to_string(),
                        other => other.to_uppercase(),
                    });
                out.push(json!({
                    "surface": "group",
                    "message_id": mid,
                    "thread_id": tid,
                    "world_id": wid,
                    "group_chat_id": gcid,
                    "group_chat_display_name": gcname,
                    "role": role_s,
                    "sender_character_id": sender,
                    "sender_name": sender_name,
                    "created_at": ts,
                    "world_day": day,
                    "world_time": time,
                    "content": content,
                }));
            }
        }

        // Merge solo+group: re-sort by direction, truncate to limit
        if direction == "before" {
            out.sort_by(|a, b| b["created_at"].as_str().unwrap_or("")
                .cmp(a["created_at"].as_str().unwrap_or("")));
        } else {
            out.sort_by(|a, b| a["created_at"].as_str().unwrap_or("")
                .cmp(b["created_at"].as_str().unwrap_or("")));
        }
        out.truncate(limit as usize);
        // Always emit chronological asc for readability
        out.sort_by(|a, b| a["created_at"].as_str().unwrap_or("")
            .cmp(b["created_at"].as_str().unwrap_or("")));
        Ok(out)
    };

    let before_msgs = pull_window(&before_ts, "before")?;
    let after_msgs  = pull_window(&after_ts,  "after")?;

    let envelope = json!({
        "ref": git_ref,
        "ref_resolved": before_sha,
        "ref_timestamp": before_ts,
        "ref_subject": before_subject,
        "end_ref": end_ref,
        "end_ref_resolved": end_ref.map(|_| after_sha),
        "end_ref_timestamp": end_ref.map(|_| after_ts.clone()),
        "end_ref_subject": end_ref.map(|_| after_subject),
        "filters": {
            "world_id": world,
            "character_id": character,
            "role": role,
            "include_solo": !groups_only,
            "include_groups": !solo_only,
            "world_ids_in_scope": world_ids,
        },
        "before": {
            "window_size_target": limit,
            "actual_count": before_msgs.len(),
            "earliest": before_msgs.first().and_then(|m| m["created_at"].as_str()),
            "latest":   before_msgs.last().and_then(|m| m["created_at"].as_str()),
            "messages": before_msgs,
        },
        "after": {
            "window_size_target": limit,
            "actual_count": after_msgs.len(),
            "earliest": after_msgs.first().and_then(|m| m["created_at"].as_str()),
            "latest":   after_msgs.last().and_then(|m| m["created_at"].as_str()),
            "messages": after_msgs,
        },
    });
    emit(r.json, envelope);
    Ok(())
}

// ─── ASK (the cost-gated one) ───────────────────────────────────────────

async fn cmd_ask(
    r: &Resolved,
    api_key: &str,
    character_id: &str,
    message: &str,
    session: Option<&str>,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
    question_summary: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;

    // Build prompt context inside one lock-acquire.
    let (system_prompt, model_config, prior_messages, session_id, character, world_id) = {
        let conn = r.db.conn.lock().unwrap();
        let character = get_character(&conn, character_id)?;
        let world = get_world(&conn, &character.world_id)?;
        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let recent_journals = list_journal_entries(&conn, character_id, 1).unwrap_or_default();
        let active_quests = list_active_quests(&conn, &character.world_id).unwrap_or_default();
        // Reuse the same relational stance the desktop app would inject —
        // the stance lives in the corpus regardless of which surface
        // (UI vs CLI) is asking the character to speak.
        let latest_stance = latest_relational_stance(&conn, character_id).unwrap_or(None);
        let stance_text: Option<String> = latest_stance.as_ref().map(|s| s.stance_text.clone());

        let system_prompt = prompts::build_dialogue_system_prompt(
            &world, &character, user_profile.as_ref(),
            None, Some("Auto"), None, None, false, &[], None,
            &recent_journals, None, &[], None, &active_quests,
            stance_text.as_deref(),
        );
        let mut model_config = orchestrator::load_model_config(&conn);
        if let Some(m) = model_override { model_config.dialogue_model = m.to_string(); }

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
        let world_id = character.world_id.clone();
        (system_prompt, model_config, prior_messages, session_id, character, world_id)
    };

    let mut messages = vec![openai::ChatMessage { role: "system".to_string(), content: system_prompt }];
    for (role, content) in prior_messages.iter() {
        messages.push(openai::ChatMessage { role: role.clone(), content: content.clone() });
    }
    messages.push(openai::ChatMessage { role: "user".to_string(), content: message.to_string() });

    // Project cost
    let prompt_text_total: String = messages.iter().map(|m| m.content.as_str()).collect::<Vec<_>>().join("\n");
    let projected_in = estimate_tokens(&prompt_text_total);
    let projected_out: i64 = 600; // safety margin for typical character reply
    let projected_usd = project_cost(&model_config.dialogue_model, projected_in, projected_out, &r.cfg.model_pricing);

    let daily_so_far = rolling_24h_total_usd();
    let daily_after = daily_so_far + projected_usd;
    let per_call_cap = r.cfg.budget.per_call_usd;
    let daily_cap = r.cfg.budget.daily_usd;

    let confirm = confirm_cost.unwrap_or(0.0);
    if projected_usd > per_call_cap && confirm < projected_usd {
        return Err(Box::new(CliError::Budget {
            kind: "per_call".to_string(),
            projected_usd,
            cap_usd: per_call_cap,
            confirm_at_least: (projected_usd * 1.05).max(0.01),
        }));
    }
    if daily_after > daily_cap && confirm < projected_usd {
        return Err(Box::new(CliError::Budget {
            kind: "daily".to_string(),
            projected_usd: daily_after,
            cap_usd: daily_cap,
            confirm_at_least: (projected_usd * 1.05).max(0.01),
        }));
    }

    if !r.json {
        eprintln!("[worldcli] character={} model={} projected≈${:.4} (~{} in / {} out tok); 24h spent=${:.4} of ${:.2}",
            character.display_name, model_config.dialogue_model, projected_usd, projected_in, projected_out,
            daily_so_far, daily_cap);
    }

    let request = openai::ChatRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.95),
        max_completion_tokens: None,
        response_format: None,
    };
    let response = openai::chat_completion_with_base(
        &model_config.chat_api_base(), api_key, &request,
    ).await?;

    let reply = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    let usage = response.usage.unwrap_or(openai::Usage {
        prompt_tokens: projected_in as u32,
        completion_tokens: 0,
        total_tokens: projected_in as u32,
    });
    let actual_in = usage.prompt_tokens as i64;
    let actual_out = usage.completion_tokens as i64;
    let actual_usd = actual_cost(&model_config.dialogue_model, actual_in, actual_out, &r.cfg.model_pricing);

    // Persist cost
    append_cost_log(&CostEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: model_config.dialogue_model.clone(),
        prompt_tokens: actual_in,
        completion_tokens: actual_out,
        usd: actual_usd,
    });

    // Persist run
    let run_id = uuid::Uuid::new_v4().to_string();
    let record = RunRecord {
        id: run_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        character_id: character_id.to_string(),
        character_name: character.display_name.clone(),
        world_id: world_id.clone(),
        model: model_config.dialogue_model.clone(),
        session: session.map(|s| s.to_string()),
        question_summary: question_summary.map(|s| s.to_string()),
        prompt: message.to_string(),
        reply: reply.clone(),
        prompt_tokens: actual_in,
        completion_tokens: actual_out,
        usd: actual_usd,
    };
    write_run(&record);

    // Persist to dev session if provided
    if let Some(id) = session_id {
        let conn = r.db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO dev_chat_messages (message_id, session_id, role, content) VALUES (?1, ?2, 'user', ?3)",
            params![uuid::Uuid::new_v4().to_string(), id, message],
        )?;
        conn.execute(
            "INSERT INTO dev_chat_messages (message_id, session_id, role, content) VALUES (?1, ?2, 'assistant', ?3)",
            params![uuid::Uuid::new_v4().to_string(), id, reply],
        )?;
    }

    if r.json {
        emit(true, json!({
            "run_id": run_id,
            "character_id": character_id,
            "character_name": character.display_name,
            "model": model_config.dialogue_model,
            "reply": reply,
            "prompt_tokens": actual_in,
            "completion_tokens": actual_out,
            "actual_usd": actual_usd,
            "rolling_24h_usd": daily_so_far + actual_usd,
        }));
    } else {
        println!("{}", reply);
        eprintln!("[worldcli] actual cost ${:.4} ({} in / {} out tok); 24h total now ${:.4}; run_id={}",
            actual_usd, actual_in, actual_out, daily_so_far + actual_usd, run_id);
    }
    Ok(())
}

// ─── Runs subcommand ────────────────────────────────────────────────────

fn cmd_runs_list(r: &Resolved, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = read_manifest();
    entries.reverse();
    entries.truncate(limit);
    emit(r.json, JsonValue::Array(entries));
    Ok(())
}

fn cmd_runs_show(r: &Resolved, id_or_prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Try exact file first
    let exact = runs_dir().join(format!("{}.json", id_or_prefix));
    if exact.exists() {
        let s = std::fs::read_to_string(&exact)?;
        let v: JsonValue = serde_json::from_str(&s).unwrap_or(JsonValue::String(s));
        emit(r.json, v);
        return Ok(());
    }
    // Prefix match against directory
    let entries = std::fs::read_dir(runs_dir())?;
    for entry in entries.flatten() {
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with(id_or_prefix) && fname.ends_with(".json") {
            let s = std::fs::read_to_string(entry.path())?;
            let v: JsonValue = serde_json::from_str(&s).unwrap_or(JsonValue::String(s));
            emit(r.json, v);
            return Ok(());
        }
    }
    Err(Box::new(CliError::NotFound(format!("run id starting with '{}'", id_or_prefix))))
}

fn cmd_runs_search(r: &Resolved, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let q = query.to_lowercase();
    let entries = read_manifest();
    let hits: Vec<JsonValue> = entries.into_iter().filter(|e| {
        let blob = e.to_string().to_lowercase();
        blob.contains(&q)
    }).collect();
    emit(r.json, JsonValue::Array(hits));
    Ok(())
}

// ─── Session management ─────────────────────────────────────────────────

fn cmd_session_show(r: &Resolved, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = r.db.conn.lock().unwrap();
    let session_id: Option<String> = conn.query_row(
        "SELECT session_id FROM dev_chat_sessions WHERE name = ?1",
        params![name], |r| r.get(0),
    ).ok();
    let Some(session_id) = session_id else {
        emit(r.json, json!({"error": "not_found", "name": name}));
        return Ok(());
    };
    let mut stmt = conn.prepare(
        "SELECT role, content, created_at FROM dev_chat_messages \
         WHERE session_id = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![session_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    })?;
    let out: Vec<JsonValue> = rows.flatten().map(|(role, content, ts)| json!({
        "role": role, "content": content, "created_at": ts,
    })).collect();
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

fn cmd_session_clear(r: &Resolved, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = r.db.conn.lock().unwrap();
    let session_id: Option<String> = conn.query_row(
        "SELECT session_id FROM dev_chat_sessions WHERE name = ?1",
        params![name], |r| r.get(0),
    ).ok();
    if let Some(id) = session_id {
        let n = conn.execute("DELETE FROM dev_chat_messages WHERE session_id = ?1", params![id])?;
        emit(r.json, json!({"cleared_messages": n, "session_name": name}));
    } else {
        emit(r.json, json!({"error": "not_found", "name": name}));
    }
    Ok(())
}

fn cmd_session_list(r: &Resolved) -> Result<(), Box<dyn std::error::Error>> {
    let conn = r.db.conn.lock().unwrap();
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
    let out: Vec<JsonValue> = rows.flatten().map(|(name, cid, ts, count)| json!({
        "name": name, "character_id": cid, "created_at": ts, "message_count": count,
    })).collect();
    emit(r.json, JsonValue::Array(out));
    Ok(())
}

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
use app_lib::ai::{openai, orchestrator, prompts, relational_stance, load_test_anchor};
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

    /// Show the latest load-test anchor for a character — the
    /// architecture-level "what does this character weight-test the
    /// world against?" synthesized from their recent corpus. Read-only.
    ShowAnchor {
        character_id: String,
        #[arg(long, default_value_t = 1)]
        history: i64,
    },

    /// Manually trigger a load-test-anchor synthesis for a character.
    /// Uses the dialogue_model by default (sharper synthesis than
    /// memory_model); pass --model to override. Architecture-level
    /// anchor identified from the character's recent corpus. See
    /// `reports/2026-04-24-0948-architecture-hypothesis-bites.md` for
    /// the experiment that validated the approach.
    RefreshAnchor {
        character_id: String,
        /// Override the model used for synthesis (default: dialogue_model).
        #[arg(long)]
        model: Option<String>,
        /// Required when projected cost exceeds the per-call cap.
        #[arg(long)]
        confirm_cost: Option<f64>,
    },

    /// Browse the structured evaluate run log at
    /// `~/.worldcli/evaluate-runs/*.json`. Every `worldcli evaluate`
    /// invocation writes its full envelope here automatically so
    /// future sessions can query, compare, or re-read prior runs
    /// without grepping prose reports.
    EvaluateRuns {
        #[command(subcommand)]
        action: EvalRunAction,
    },

    /// Browse the structured synthesize run log at
    /// `~/.worldcli/synthesize-runs/*.json`. Every `worldcli synthesize`
    /// invocation writes its full envelope (bundled messages + question +
    /// prose synthesis) here automatically so Mode B findings accumulate
    /// as queryable substrate alongside Mode A evaluate runs.
    SynthesizeRuns {
        #[command(subcommand)]
        action: SynthRunAction,
    },

    /// Browse the structured replay run log at
    /// `~/.worldcli/replay-runs/*.json`.
    ReplayRuns {
        #[command(subcommand)]
        action: ReplayRunAction,
    },

    /// Experiment registry at `experiments/*.md` — a structured hypothesis
    /// file per experiment with YAML-ish frontmatter (status, mode, refs,
    /// rubric_ref, prediction, run_ids, follow_ups, reports) and
    /// markdown-body interpretation. Enables queries across the full
    /// experimental history (what's open? what's been refuted? what
    /// rubrics keep refuting? which characters have never been probed?)
    /// that prose reports alone can't answer cheaply.
    Lab {
        #[command(subcommand)]
        action: LabAction,
    },

    /// Cross-commit A/B replay — Mode C's strongest instrument. Takes
    /// one user prompt + one character + a list of git refs. For each
    /// ref, fetches the historical `prompts.rs` via `git show <ref>:...`,
    /// parses out the named dialogue craft-note bodies, injects them as
    /// overrides into THIS running binary's prompt-assembly pipeline,
    /// then sends the prompt against that injected stack. No checkout,
    /// no rebuild, no git worktrees — one binary, historical prompts
    /// layered in on demand. Returns each ref's reply side-by-side for
    /// direct comparison. See the prompt-override hook in
    /// `src-tauri/src/ai/prompts.rs` for which fragments are
    /// overridable (cosmology/theology blocks are NOT — those are
    /// load-bearing across all commits by design).
    Replay {
        /// Comma-separated list of git refs (shas, tags, branches).
        /// Each ref produces one reply in the side-by-side output.
        /// Typical use: --refs HEAD,8e9e53d,bce17e9 to compare the
        /// current stack against two prior states.
        #[arg(long, value_delimiter = ',')]
        refs: Vec<String>,
        /// Character whose voice is being replayed across refs.
        #[arg(long)]
        character: String,
        /// The user prompt to send against each ref's injected stack.
        /// Same prompt for every ref — that's the A/B discipline.
        #[arg(long)]
        prompt: String,
        /// Override the configured dialogue model.
        #[arg(long)]
        model: Option<String>,
        /// Required when projected total cost (sum across refs) exceeds
        /// the per-call cap. Passes through to each individual call.
        #[arg(long)]
        confirm_cost: Option<f64>,
        /// Number of samples per ref (default 1). With N>1, each ref is
        /// called K times with the SAME prompt and SAME overrides — the
        /// only variable is stochastic draw at temperature 0.95. Use this
        /// to rule out sampling-noise as the explanation for a direction-
        /// match at N=1 (the sketch-to-claim escalation move in CLAUDE.md
        /// § Evidentiary standards). Example: --refs pre,HEAD --n 5 runs
        /// 10 total dialogue calls (5 samples × 2 refs). Cost scales
        /// linearly: total ≈ per-ref × refs × N. Results are stored in
        /// the run envelope with a sample_index field so grade-runs can
        /// discriminate samples within the same ref.
        #[arg(long, default_value_t = 1)]
        n: u32,
        /// Git repo path for ref resolution + `git show`.
        #[arg(long)]
        repo: Option<PathBuf>,
        /// Optional custom ordering of the three main dialogue prompt
        /// sections. Comma-separated. Valid names (case-insensitive,
        /// hyphens or underscores): agency-and-behavior / agency /
        /// behavior; craft-notes / craft / notes; invariants /
        /// invariant. Must include exactly one of each. Default
        /// (no flag): agency-and-behavior,craft-notes,invariants.
        /// Example: --section-order invariants,craft-notes,agency-and-behavior
        /// tests the placement-dominates-tier hypothesis by putting
        /// invariants first. Applied identically across all refs so the
        /// section-order is held constant while only the prompt
        /// override-bodies vary per ref.
        #[arg(long, value_delimiter = ',')]
        section_order: Vec<String>,
        /// Optional within-section ordering for craft notes. Comma-
        /// separated short names (e.g. earned_register,hands_as_coolant,
        /// reflex_polish,protagonist_framing). Partial orderings are
        /// supported: pieces you name appear first in the given order,
        /// the rest fall in by default order. Accepted names (full list):
        /// earned_register, craft_notes, hidden_commonality, drive_the_moment,
        /// verdict_without_over_explanation (or verdict),
        /// reflex_polish_vs_earned_close (or reflex_polish),
        /// keep_the_scene_breathing (or scene_breathing),
        /// name_the_glad_thing_plain (or glad_thing_plain),
        /// plain_after_crooked, wit_as_dimmer,
        /// let_the_real_thing_in (or real_thing_in), hands_as_coolant,
        /// noticing_as_mirror, unguarded_entry,
        /// protagonist_framing (or protagonist). Trailing "_dialogue"
        /// suffixes are stripped.
        #[arg(long, value_delimiter = ',')]
        craft_notes_order: Vec<String>,
        /// Optional within-section ordering for invariants. Comma-
        /// separated short names. Partial orderings supported (same
        /// prefix-then-defaults semantics as --craft-notes-order).
        /// Accepted names: reverence, daylight, agape,
        /// fruits_of_the_spirit (or fruits), soundness, nourishment,
        /// tell_the_truth (or truth). Trailing "_block" suffixes are
        /// stripped.
        #[arg(long, value_delimiter = ',')]
        invariants_order: Vec<String>,
        /// Craft-note pieces to OMIT from prompt assembly (skipped
        /// during dispatch). Comma-separated short names. Test
        /// whether a specific craft note is actually load-bearing by
        /// running the same probes with and without it. Valid names
        /// are the same as --craft-notes-order. Example:
        /// --omit-craft-notes hands_as_coolant tests whether the
        /// action-beat pressure drops when that rule is off.
        #[arg(long, value_delimiter = ',')]
        omit_craft_notes: Vec<String>,
        /// Invariant pieces to OMIT. Comma-separated short names.
        /// Valid names same as --invariants-order. Has theological
        /// implications (invariants are compile-time-enforced
        /// normally) — use only for targeted experiments, not
        /// production.
        #[arg(long, value_delimiter = ',')]
        omit_invariants: Vec<String>,
        /// Path to a file whose contents are spliced into the prompt
        /// at --insert-before or --insert-after anchor. Used to
        /// audition new craft notes or invariants WITHOUT shipping
        /// them first. The file's content is inserted verbatim at the
        /// anchor+position. Exactly one of --insert-before /
        /// --insert-after must also be specified.
        #[arg(long)]
        insert_file: Option<PathBuf>,
        /// Insert the contents of --insert-file BEFORE the named
        /// anchor. Anchor can be a piece name (e.g., "earned_register",
        /// "reverence") or a section boundary
        /// ("section-start:craft-notes", "section-end:invariants").
        /// Mutually exclusive with --insert-after.
        #[arg(long, conflicts_with = "insert_after")]
        insert_before: Option<String>,
        /// Insert the contents of --insert-file AFTER the named
        /// anchor. Same anchor syntax as --insert-before. Mutually
        /// exclusive with --insert-before.
        #[arg(long, conflicts_with = "insert_before")]
        insert_after: Option<String>,
    },

    /// List, show, or search rubrics in the library
    /// (`reports/rubrics/*.md`). Rubrics are versioned markdown
    /// files whose `# Rubric` section is the exact evaluator
    /// prompt and whose other sections accumulate craft capital
    /// (known failure modes, run history, usage guidance).
    /// See `reports/rubrics/README.md` for the authoring
    /// convention.
    Rubric {
        #[command(subcommand)]
        action: RubricAction,
    },

    /// Rubric-driven LLM evaluation of messages in a
    /// sample-windows-shaped before/after comparison. The reports
    /// flagged this as the missing instrument: regex metrics can't
    /// distinguish cascades from natural register, or safe-thing
    /// clinging from scene-furniture, or joy-shading from plain
    /// meeting. An LLM-evaluator pass with a qualitative rubric can.
    ///
    /// Each assistant message in the window is sent to the cheap
    /// memory_model with the rubric, its preceding user turn as
    /// context, and a structured JSON response format. Per-message
    /// judgments (yes / no / mixed + confidence + quote + one-line
    /// reasoning) aggregate into before/after counts so the user
    /// Grade arbitrary stored runs (ask, replay, scenario) against a
    /// rubric via the memory_model. The generic "given these N texts
    /// and a rubric, give me yes/no/mixed per text + aggregation"
    /// primitive. Use when testing whether a prompt-stack change moved
    /// behavior on replies you've already elicited via ask/replay/
    /// scenario, without needing the natural-corpus before/after
    /// windowing that `evaluate` requires.
    GradeRuns {
        /// One or more run_ids (or their short prefixes) from
        /// ~/.worldcli/runs, ~/.worldcli/replay-runs, or
        /// ~/.worldcli/scenario-runs. Each run's reply(ies) become
        /// one or more graded items.
        run_ids: Vec<String>,
        /// The rubric question asked of each reply. Plain English.
        #[arg(long)]
        rubric: Option<String>,
        /// Look up a named rubric from the library.
        #[arg(long)]
        rubric_ref: Option<String>,
        /// Read rubric from a file.
        #[arg(long)]
        rubric_file: Option<PathBuf>,
        /// Override the evaluator model (default: memory_model).
        #[arg(long)]
        model: Option<String>,
        /// Required when projected cost exceeds the per-call cap.
        #[arg(long)]
        confirm_cost: Option<f64>,
    },

    /// Evaluate natural-corpus messages against a rubric on either
    /// side of a git ref. The messages-x-commits primitive. `evaluate`
    /// requires corpus messages; use `grade-runs` if you want to grade
    /// elicited replies from ask/replay/scenario runs.
    Evaluate {
        /// Git ref marking the boundary commit. Messages before
        /// its timestamp form the "before" window; messages after
        /// form the "after" window.
        #[arg(long = "ref")]
        git_ref: String,
        /// Optional second ref — after-window starts at this ref
        /// instead of `--ref`. Useful when a series of commits
        /// together form the change.
        #[arg(long)]
        end_ref: Option<String>,
        /// Messages per window. Smaller than sample-windows because
        /// every message costs one LLM call. Default 12.
        #[arg(long, default_value_t = 12)]
        limit: i64,
        /// Restrict to one character's solo thread. Mutually exclusive
        /// with --group-chat; exactly one must be supplied.
        #[arg(long)]
        character: Option<String>,
        /// Evaluate a group-chat thread instead of a solo thread.
        /// Every assistant reply in the group (regardless of which
        /// character spoke) goes through the rubric.
        #[arg(long)]
        group_chat: Option<String>,
        /// The qualitative question the evaluator asks of each
        /// message. Plain English. The rubric should name what
        /// "yes / no / mixed" mean in its own domain.
        #[arg(long)]
        rubric: Option<String>,
        /// Alternative: read rubric from a file (useful for
        /// multi-paragraph prompts with examples).
        #[arg(long)]
        rubric_file: Option<PathBuf>,
        /// Alternative: look up a named rubric from the library at
        /// `reports/rubrics/<name>.md`. The named file's `# Rubric`
        /// section becomes the evaluator prompt, and this run is
        /// appended to the rubric's run history automatically.
        /// Mutually exclusive with --rubric and --rubric-file.
        #[arg(long)]
        rubric_ref: Option<String>,
        /// Role filter for messages-to-evaluate. Default 'assistant'.
        #[arg(long, default_value = "assistant")]
        role: String,
        /// Number of preceding turns (both user and assistant) to
        /// include as context for each eval target. Default 3 — the
        /// immediate triggering user turn plus ~2 more beats before
        /// it. Replies are shaped by chat history, not just by the
        /// single preceding turn; giving the evaluator scene context
        /// grounds its judgments. Larger values cost more per call
        /// (~$0.00003/turn at gpt-4o-mini pricing) but provide
        /// stronger signal for nuanced rubrics.
        #[arg(long, default_value_t = 3)]
        context_turns: i64,
        /// Override the evaluator model (default: memory_model).
        #[arg(long)]
        model: Option<String>,
        /// Required when projected total cost exceeds the per-call cap.
        #[arg(long)]
        confirm_cost: Option<f64>,
        /// Git repo path for ref resolution.
        #[arg(long)]
        repo: Option<PathBuf>,
    },

    /// Qualitative synthesis of a message corpus — Mode B (open-ended
    /// LLM feedback) as a first-class command. Bundles N messages
    /// (before + after windows around a git ref) into ONE call to the
    /// dialogue_model, asks an open-ended question, returns prose.
    /// Complements `evaluate` (Mode A — per-message structured
    /// yes/no/mixed verdicts) for questions whose shape is "read all
    /// these replies together and tell me what's happening in them"
    /// rather than "does each reply pass this specific test?". The
    /// 1326 John-stillness report is the worked example of when Mode B
    /// is the right instrument — the rubric's gates correctly excluded
    /// the actual register-move, so counts couldn't find it.
    Synthesize {
        /// Git ref marking the boundary commit. By default both the
        /// before-window (messages before the ref's timestamp) and
        /// the after-window (messages at/after) are bundled into the
        /// synthesis call — so the question can address change, not
        /// just current state. Pass --end-ref to use a different
        /// cutoff for the after-window (A..B series).
        #[arg(long = "ref")]
        git_ref: String,
        /// Optional second ref. After-window starts here instead of
        /// at `--ref` (same semantics as `evaluate --end-ref`).
        #[arg(long)]
        end_ref: Option<String>,
        /// Messages per window. Default 20 — higher than `evaluate`
        /// because synthesis is one call, not N calls, so per-message
        /// cost is bundled-in rather than linear.
        #[arg(long, default_value_t = 20)]
        limit: i64,
        /// Restrict to one character's solo thread + group chats.
        /// Mutually exclusive with --group-chat; exactly one required.
        #[arg(long)]
        character: Option<String>,
        /// Synthesize from one group-chat thread's assistant replies.
        #[arg(long)]
        group_chat: Option<String>,
        /// The open-ended question to answer about the corpus. Plain
        /// English. Name specifically what you want the synthesizer
        /// to look for — patterns, register choices, things NOT
        /// present. Vague questions return vague prose.
        #[arg(long)]
        question: Option<String>,
        /// Alternative: read question from a file (useful for multi-
        /// paragraph prompts with worked examples of what to look for).
        #[arg(long)]
        question_file: Option<PathBuf>,
        /// Role filter for messages-to-synthesize. Default 'assistant'.
        #[arg(long, default_value = "assistant")]
        role: String,
        /// Preceding turns of chat-history per target, included so
        /// the synthesizer can read each reply in scene. Default 3.
        #[arg(long, default_value_t = 3)]
        context_turns: i64,
        /// Override the synthesizer model. Default: dialogue_model —
        /// the user's more capable model. Synthesis is qualitative
        /// prose; the extra capability over memory_model matters.
        #[arg(long)]
        model: Option<String>,
        /// Required when projected cost exceeds the per-call cap.
        #[arg(long)]
        confirm_cost: Option<f64>,
        /// Git repo path for ref resolution.
        #[arg(long)]
        repo: Option<PathBuf>,
    },

    /// Consult the Consultant about a character's thread — either
    /// Immersive (a trusted in-world confidant who treats everything as
    /// real and never breaks frame) or Backstage (a wry stage-manager
    /// outside the fourth wall who talks craft, mechanics, and the
    /// shape of the work). Same system-prompt genealogy as the in-app
    /// Consultant, stripped of UI-coupled action cards which the CLI
    /// cannot render. Cost-gated like `ask`. Persists to a dev-session
    /// separate from the app's consultant history.
    Consult {
        character_id: String,
        message: String,
        /// Which mode. "immersive" (default) = in-world confidant;
        /// "backstage" = craft/mechanics collaborator outside the frame.
        #[arg(long, default_value = "immersive")]
        mode: String,
        /// Persist this exchange to a named dev-session for follow-ups.
        /// If omitted, this is a one-shot with no history carried forward.
        #[arg(long)]
        session: Option<String>,
        /// Override the configured dialogue model.
        #[arg(long)]
        model: Option<String>,
        /// Required when projected cost exceeds the per-call cap.
        #[arg(long)]
        confirm_cost: Option<f64>,
        /// Free-form summary of why you're consulting. Stored in the
        /// run log so future investigations can find this exchange.
        #[arg(long)]
        question_summary: Option<String>,
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
        /// Suppress the load-test-anchor block (and any other register
        /// axes) from the dialogue system prompt for this call.
        /// Used for A/B testing whether the anchor moves real-time
        /// behavior. Default false (anchors injected as in production).
        #[arg(long)]
        no_anchor: bool,
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

#[derive(Subcommand)]
enum RubricAction {
    /// List all rubrics in the library with name + description + version.
    List,
    /// Show a rubric by name — frontmatter + full prompt + notes + run history.
    Show { name: String },
    /// Search rubric text, descriptions, and run history for a substring.
    Search { query: String },
}

#[derive(Subcommand)]
enum EvalRunAction {
    /// List recent evaluate runs (newest first).
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show the full envelope of one run by id (or unique short prefix).
    Show { id: String },
    /// Search run envelopes for a substring across rubric / scope / ref / reasoning.
    Search { query: String },
}

#[derive(Subcommand)]
enum SynthRunAction {
    /// List recent synthesize runs (newest first).
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show the full envelope of one run by id (or unique short prefix).
    Show { id: String },
    /// Search run envelopes for a substring across question / scope / ref / synthesis.
    Search { query: String },
}

#[derive(Subcommand)]
enum ReplayRunAction {
    /// List recent replay runs (newest first).
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show the full envelope of one run by id (or unique short prefix).
    Show { id: String },
    /// Search replay envelopes for a substring.
    Search { query: String },
}

#[derive(Subcommand)]
enum LabAction {
    /// List experiments in the registry.
    List {
        /// Filter by status: proposed | running | open | confirmed | refuted.
        #[arg(long)]
        status: Option<String>,
    },
    /// List just the non-resolved experiments (proposed | running | open).
    /// The "what's still open" view future sessions use to pick up threads.
    Open,
    /// Show one experiment's full file (frontmatter + markdown body).
    Show { slug: String },
    /// Search experiment files for a substring across hypothesis / prediction
    /// / summary / scope / reports / the markdown body.
    Search { query: String },
    /// Scaffold a new experiment file under `experiments/<slug>.md`.
    /// Initial status is 'proposed' — advance to 'running' when execution
    /// starts, then 'confirmed' | 'refuted' | 'open' via `lab resolve`.
    Propose {
        /// Slug used for the filename. Match the shape of rubric names:
        /// kebab-case, letters/digits/hyphens only, under ~50 chars.
        slug: String,
        /// The hypothesis this experiment tests, in one or two sentences.
        #[arg(long)]
        hypothesis: String,
        /// Experimental mode — passive (Mode A), qualitative (Mode B),
        /// or active (Mode C).
        #[arg(long)]
        mode: String,
        /// What CONFIRMED looks like and what REFUTED looks like,
        /// written BEFORE any LLM call (pre-registered prediction).
        #[arg(long)]
        prediction: String,
        /// Optional: the git ref the experiment pivots on.
        #[arg(long)]
        r#ref: Option<String>,
        /// Optional: the rubric from the library this experiment uses.
        #[arg(long)]
        rubric_ref: Option<String>,
    },
    /// Resolve an experiment — mark the outcome.
    Resolve {
        slug: String,
        /// New status: confirmed | refuted | open.
        #[arg(long)]
        status: String,
        /// Short summary of the result (written to frontmatter).
        #[arg(long)]
        summary: Option<String>,
        /// Optional: path to the report that holds the full interpretation.
        #[arg(long)]
        report: Option<String>,
    },
    /// Link a run (evaluate / synthesize / replay) to an experiment by id
    /// or prefix — the run id gets appended to the experiment's run_ids.
    LinkRun {
        slug: String,
        run_id: String,
    },
    /// Scenario templates — canonical multi-variant probe sequences for
    /// Mode C (active elicitation). Each scenario lives at
    /// experiments/scenarios/<name>.md with frontmatter (name, purpose,
    /// optional measure_with rubric) and ## Variant: <name> sections
    /// whose bodies are the prompt text to send. `lab scenario run`
    /// fires each variant as a fresh dialogue call and returns the
    /// replies side-by-side (optionally with rubric verdicts applied).
    Scenario {
        #[command(subcommand)]
        action: ScenarioAction,
    },
}

#[derive(Subcommand)]
enum ScenarioAction {
    /// List available scenarios under experiments/scenarios/.
    List,
    /// Show one scenario's full file.
    Show { name: String },
    /// Run a scenario — fire each variant prompt at the character via
    /// dialogue_model, capture reply, optionally evaluate with the
    /// scenario's `measure_with` rubric.
    Run {
        name: String,
        /// Character to run the scenario against.
        #[arg(long)]
        character: String,
        /// Override the scenario's `measure_with` rubric (or provide
        /// one if the scenario didn't set one).
        #[arg(long)]
        rubric_ref: Option<String>,
        /// Skip rubric evaluation even if the scenario sets measure_with.
        /// Useful when you want replies only, for a faster cheap pass.
        #[arg(long)]
        skip_evaluate: bool,
        /// Override the dialogue model.
        #[arg(long)]
        model: Option<String>,
        /// Required when projected total cost (N variants × dialogue call
        /// + optional evaluator calls) exceeds the per-call cap.
        #[arg(long)]
        confirm_cost: Option<f64>,
    },
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
    // Try the CLI's own explicit namespace first, then fall back to
    // common conventions people already have populated on their
    // macOS keychain (e.g. a key added for use by other OpenAI
    // tooling). "Bake the key in once, reach for it everywhere" —
    // no reason worldcli should force a duplicate entry.
    //
    // Order matters: the WorldThreadsCLI entry wins if set (lets
    // the user scope a *different* key to this CLI if they want,
    // e.g. a project-isolated sub-org key); otherwise we use the
    // common "openai / default" convention, then a few close
    // spellings.
    let candidates: &[(&str, &str)] = &[
        ("WorldThreadsCLI", "openai"),
        ("openai", "default"),
        ("openai", "api-key"),
        ("openai", "api_key"),
        ("OpenAI", "default"),
    ];
    for (service, account) in candidates {
        let out = std::process::Command::new("security")
            .args(["find-generic-password", "-s", service, "-a", account, "-w"])
            .output()
            .ok();
        let Some(out) = out else { continue; };
        if !out.status.success() { continue; }
        let Some(key) = String::from_utf8(out.stdout).ok() else { continue; };
        let trimmed = key.trim();
        if !trimmed.is_empty() { return Some(trimmed.to_string()); }
    }
    None
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
        Cmd::Rubric { action } => cmd_rubric(&r, action),
        Cmd::EvaluateRuns { action } => cmd_evaluate_runs(&r, action),
        Cmd::SynthesizeRuns { action } => cmd_synthesize_runs(&r, action),
        Cmd::Evaluate { git_ref, end_ref, limit, character, group_chat, rubric, rubric_file, rubric_ref, role, context_turns, model, confirm_cost, repo } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_evaluate(&r, &api_key, &git_ref, end_ref.as_deref(), limit, character.as_deref(), group_chat.as_deref(), rubric.as_deref(), rubric_file.as_deref(), rubric_ref.as_deref(), &role, context_turns, model.as_deref(), confirm_cost, repo.as_deref()).await
        }
        Cmd::GradeRuns { run_ids, rubric, rubric_ref, rubric_file, model, confirm_cost } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_grade_runs(&r, &api_key, &run_ids, rubric.as_deref(), rubric_ref.as_deref(), rubric_file.as_deref(), model.as_deref(), confirm_cost).await
        }
        Cmd::Synthesize { git_ref, end_ref, limit, character, group_chat, question, question_file, role, context_turns, model, confirm_cost, repo } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_synthesize(&r, &api_key, &git_ref, end_ref.as_deref(), limit, character.as_deref(), group_chat.as_deref(), question.as_deref(), question_file.as_deref(), &role, context_turns, model.as_deref(), confirm_cost, repo.as_deref()).await
        }
        Cmd::ReplayRuns { action } => cmd_replay_runs(&r, action),
        Cmd::Lab { action } => {
            // Scenario::Run needs api_key; other lab actions don't.
            let api_key = if matches!(action, LabAction::Scenario { action: ScenarioAction::Run { .. } }) {
                match resolve_api_key(cli.api_key.as_deref()) {
                    Some(k) => Some(k),
                    None => return Err(Box::<dyn std::error::Error>::from(
                        "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                    )),
                }
            } else { None };
            cmd_lab(&r, action, api_key.as_deref()).await
        }
        Cmd::Replay { refs, character, prompt, model, confirm_cost, n, repo, section_order, craft_notes_order, invariants_order, omit_craft_notes, omit_invariants, insert_file, insert_before, insert_after } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_replay(&r, &api_key, &refs, &character, &prompt, model.as_deref(), confirm_cost, n, repo.as_deref(), &section_order, &craft_notes_order, &invariants_order, &omit_craft_notes, &omit_invariants, insert_file.as_deref(), insert_before.as_deref(), insert_after.as_deref()).await
        }
        Cmd::Consult { character_id, message, mode, session, model, confirm_cost, question_summary } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_consult(&r, &api_key, &character_id, &message, &mode, session.as_deref(), model.as_deref(), confirm_cost, question_summary.as_deref()).await
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
        Cmd::ShowAnchor { character_id, history } => cmd_show_anchor(&r, &character_id, history),
        Cmd::RefreshAnchor { character_id, model, confirm_cost } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_refresh_anchor(&r, &api_key, &character_id, model.as_deref(), confirm_cost).await
        }
        Cmd::Ask { character_id, message, session, model, confirm_cost, question_summary, no_anchor } => {
            let api_key = match resolve_api_key(cli.api_key.as_deref()) {
                Some(k) => k,
                None => return Err(Box::<dyn std::error::Error>::from(
                    "No API key. Set OPENAI_API_KEY, pass --api-key, or add to keychain via:\n  security add-generic-password -s WorldThreadsCLI -a openai -w \"<sk-...>\"".to_string()
                )),
            };
            cmd_ask(&r, &api_key, &character_id, &message, session.as_deref(), model.as_deref(), confirm_cost, question_summary.as_deref(), no_anchor).await
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

// ─── Rubric library (reports/rubrics/*.md) ─────────────────────────────

/// Directory where versioned rubrics live. Resolved relative to the
/// current working directory (typically the repo root when `worldcli`
/// is invoked from there). A `--rubrics-dir` override would be a
/// reasonable future extension; for now this is the single convention.
fn rubrics_dir() -> PathBuf {
    PathBuf::from("reports/rubrics")
}

/// Parsed rubric file — frontmatter metadata + the prompt section
/// body (what the evaluator sees) + the raw full file (for browse
/// display).
#[derive(Debug, Clone)]
struct RubricFile {
    name: String,
    version: String,
    description: String,
    prompt: String,     // extracted # Rubric section body
    raw: String,        // full file text, for `rubric show`
    path: PathBuf,
}

/// Extract the `# Rubric` section body from a markdown file — the
/// text between `# Rubric` and the next top-level `#` heading.
/// This is the exact prompt sent to the evaluator.
fn extract_rubric_section(raw: &str) -> Option<String> {
    let mut in_section = false;
    let mut buf: Vec<&str> = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("# Rubric") && !trimmed.starts_with("# Rubric ") == false
           || trimmed == "# Rubric" {
            in_section = true;
            continue;
        }
        // Exact "# Rubric" match catches the section header.
        if trimmed == "# Rubric" {
            in_section = true;
            continue;
        }
        if in_section {
            // Stop at the next top-level heading.
            if trimmed.starts_with("# ") && trimmed != "# Rubric" {
                break;
            }
            buf.push(line);
        }
    }
    let body = buf.join("\n").trim().to_string();
    if body.is_empty() { None } else { Some(body) }
}

/// Parse YAML-like frontmatter from the top of a markdown file.
/// Supports only the subset this library uses (name/version/description).
/// Returns (name, version, description).
fn parse_rubric_frontmatter(raw: &str) -> (String, String, String) {
    let mut name = String::new();
    let mut version = String::new();
    let mut description = String::new();
    let mut in_fm = false;
    let mut fm_done = false;
    for line in raw.lines() {
        if fm_done { break; }
        let t = line.trim_end();
        if t == "---" {
            if !in_fm { in_fm = true; continue; }
            else { fm_done = true; continue; }
        }
        if in_fm {
            if let Some(rest) = t.strip_prefix("name:") { name = rest.trim().trim_matches('"').to_string(); }
            else if let Some(rest) = t.strip_prefix("version:") { version = rest.trim().trim_matches('"').to_string(); }
            else if let Some(rest) = t.strip_prefix("description:") { description = rest.trim().trim_matches('"').to_string(); }
        }
    }
    (name, version, description)
}

fn load_rubric(name: &str) -> Result<RubricFile, String> {
    let path = rubrics_dir().join(format!("{}.md", name));
    if !path.exists() {
        return Err(format!("rubric '{}' not found at {}. Run `worldcli rubric list` to see the library.", name, path.display()));
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    let (fm_name, fm_version, fm_description) = parse_rubric_frontmatter(&raw);
    let prompt = extract_rubric_section(&raw)
        .ok_or_else(|| format!("rubric '{}' at {} has no `# Rubric` section", name, path.display()))?;
    Ok(RubricFile {
        name: if fm_name.is_empty() { name.to_string() } else { fm_name },
        version: if fm_version.is_empty() { "?".to_string() } else { fm_version },
        description: fm_description,
        prompt,
        raw,
        path,
    })
}

fn list_rubrics() -> Result<Vec<RubricFile>, String> {
    let dir = rubrics_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
        let fname = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if fname == "README" { continue; }
        if let Ok(rf) = load_rubric(&fname) {
            out.push(rf);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Append a run-history entry to the rubric's markdown file. Called
/// after a successful `evaluate` run that used --rubric-ref. Never
/// fails loudly — a filesystem hiccup shouldn't mask a successful
/// evaluation result; just log and continue.
fn append_rubric_run_history(name: &str, entry: &str) {
    let path = rubrics_dir().join(format!("{}.md", name));
    let Ok(current) = std::fs::read_to_string(&path) else { return; };
    // Find the `# Run history` section; if present, append under it.
    // If absent, append it + the entry to the end of the file.
    let marker = "# Run history";
    let new_content = if current.contains(marker) {
        // Append as the last line of the run-history section.
        // Simplest: find the marker, keep everything up to and
        // including the marker + any content through EOF, then
        // append the new line at the end.
        format!("{}\n{}\n", current.trim_end(), entry.trim())
    } else {
        format!("{}\n\n# Run history\n\n{}\n", current.trim_end(), entry.trim())
    };
    let _ = std::fs::write(&path, new_content);
}

// ─── Evaluate run log (~/.worldcli/evaluate-runs/) ─────────────────────

fn evaluate_runs_dir() -> PathBuf { worldcli_home().join("evaluate-runs") }
fn evaluate_runs_manifest() -> PathBuf { evaluate_runs_dir().join("manifest.jsonl") }

/// Persist the full evaluate envelope to disk + append a compact
/// manifest line for fast list/search. Never fails loudly — a disk
/// hiccup shouldn't hide a successful evaluation result.
fn write_evaluate_run(run_id: &str, envelope: &JsonValue) {
    let dir = evaluate_runs_dir();
    let _ = std::fs::create_dir_all(&dir);
    let per_path = dir.join(format!("{}.json", run_id));
    if let Ok(s) = serde_json::to_string_pretty(envelope) {
        let _ = std::fs::write(&per_path, s);
    }
    // Manifest line: compact one-line summary grep-friendly.
    let manifest_entry = json!({
        "run_id": envelope.get("run_id"),
        "run_timestamp": envelope.get("run_timestamp"),
        "ref": envelope.get("ref"),
        "ref_resolved": envelope.get("ref_resolved"),
        "ref_subject": envelope.get("ref_subject"),
        "character_id": envelope.get("character_id"),
        "group_chat_id": envelope.get("group_chat_id"),
        "scope_label": envelope.get("scope_label"),
        "rubric_ref": envelope.get("rubric_ref"),
        "rubric_version": envelope.get("rubric_version"),
        "rubric_preview": envelope.get("rubric").and_then(|v| v.as_str())
            .map(|s| s.chars().take(120).collect::<String>()),
        "before_totals": envelope.get("before").map(|b| json!({
            "yes": b.get("yes"), "no": b.get("no"),
            "mixed": b.get("mixed"), "errors": b.get("errors"),
            "count": b.get("count"),
        })),
        "after_totals": envelope.get("after").map(|a| json!({
            "yes": a.get("yes"), "no": a.get("no"),
            "mixed": a.get("mixed"), "errors": a.get("errors"),
            "count": a.get("count"),
        })),
        "cost_usd": envelope.get("cost").and_then(|c| c.get("actual_usd")),
    });
    let line = serde_json::to_string(&manifest_entry).unwrap_or_default();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true).open(evaluate_runs_manifest())
    {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

fn read_evaluate_runs_manifest() -> Vec<JsonValue> {
    let Ok(content) = std::fs::read_to_string(evaluate_runs_manifest()) else { return Vec::new(); };
    content.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn cmd_evaluate_runs(r: &Resolved, action: EvalRunAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        EvalRunAction::List { limit } => {
            let mut entries = read_evaluate_runs_manifest();
            entries.reverse();
            entries.truncate(limit);
            if r.json {
                emit(true, JsonValue::Array(entries));
            } else {
                if entries.is_empty() {
                    println!("No evaluate runs recorded yet. Run `worldcli evaluate ...` first.");
                    return Ok(());
                }
                for e in &entries {
                    let ts = e.get("run_timestamp").and_then(|v| v.as_str()).unwrap_or("")[..19].to_string();
                    let id = e.get("run_id").and_then(|v| v.as_str()).unwrap_or("");
                    let id_short = &id[..8.min(id.len())];
                    let scope = e.get("scope_label").and_then(|v| v.as_str()).unwrap_or("?");
                    let rubric = e.get("rubric_ref").and_then(|v| v.as_str()).unwrap_or("<inline>");
                    let b = e.get("before_totals");
                    let a = e.get("after_totals");
                    let fmt_totals = |t: Option<&JsonValue>| -> String {
                        t.map(|t| format!("y{}/n{}/m{}",
                            t.get("yes").and_then(|v| v.as_i64()).unwrap_or(0),
                            t.get("no").and_then(|v| v.as_i64()).unwrap_or(0),
                            t.get("mixed").and_then(|v| v.as_i64()).unwrap_or(0))).unwrap_or_default()
                    };
                    println!("{id_short}  [{ts}]  {scope}  rubric={rubric}  B:{}  A:{}",
                        fmt_totals(b), fmt_totals(a));
                }
            }
        }
        EvalRunAction::Show { id } => {
            let dir = evaluate_runs_dir();
            // Exact id first.
            let exact = dir.join(format!("{}.json", id));
            if exact.exists() {
                let s = std::fs::read_to_string(&exact)?;
                let v: JsonValue = serde_json::from_str(&s).unwrap_or(JsonValue::String(s));
                emit(r.json, v);
                return Ok(());
            }
            // Prefix match.
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.starts_with(&id) && fname.ends_with(".json") {
                        let s = std::fs::read_to_string(entry.path())?;
                        let v: JsonValue = serde_json::from_str(&s).unwrap_or(JsonValue::String(s));
                        emit(r.json, v);
                        return Ok(());
                    }
                }
            }
            return Err(Box::new(CliError::NotFound(format!("evaluate run starting with '{}'", id))));
        }
        EvalRunAction::Search { query } => {
            let q = query.to_lowercase();
            let entries = read_evaluate_runs_manifest();
            let hits: Vec<JsonValue> = entries.into_iter()
                .filter(|e| e.to_string().to_lowercase().contains(&q))
                .collect();
            emit(r.json, JsonValue::Array(hits));
        }
    }
    Ok(())
}

// ─── Synthesize run log (~/.worldcli/synthesize-runs/) ─────────────────

fn synthesize_runs_dir() -> PathBuf { worldcli_home().join("synthesize-runs") }
fn synthesize_runs_manifest() -> PathBuf { synthesize_runs_dir().join("manifest.jsonl") }

/// Persist the full synthesize envelope + append a compact manifest
/// line for fast list/search. Mirrors write_evaluate_run so the two
/// Mode-A/Mode-B run logs look and query the same way.
fn write_synthesize_run(run_id: &str, envelope: &JsonValue) {
    let dir = synthesize_runs_dir();
    let _ = std::fs::create_dir_all(&dir);
    let per_path = dir.join(format!("{}.json", run_id));
    if let Ok(s) = serde_json::to_string_pretty(envelope) {
        let _ = std::fs::write(&per_path, s);
    }
    let manifest_entry = json!({
        "run_id": envelope.get("run_id"),
        "run_timestamp": envelope.get("run_timestamp"),
        "ref": envelope.get("ref"),
        "ref_resolved": envelope.get("ref_resolved"),
        "ref_subject": envelope.get("ref_subject"),
        "character_id": envelope.get("character_id"),
        "group_chat_id": envelope.get("group_chat_id"),
        "scope_label": envelope.get("scope_label"),
        "question_preview": envelope.get("question").and_then(|v| v.as_str())
            .map(|s| s.chars().take(140).collect::<String>()),
        "synthesis_preview": envelope.get("synthesis").and_then(|v| v.as_str())
            .map(|s| s.chars().take(200).collect::<String>()),
        "before_count": envelope.get("before").and_then(|b| b.get("count")),
        "after_count": envelope.get("after").and_then(|a| a.get("count")),
        "cost_usd": envelope.get("cost").and_then(|c| c.get("actual_usd")),
        "model": envelope.get("model"),
    });
    let line = serde_json::to_string(&manifest_entry).unwrap_or_default();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true).open(synthesize_runs_manifest())
    {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

fn read_synthesize_runs_manifest() -> Vec<JsonValue> {
    let Ok(content) = std::fs::read_to_string(synthesize_runs_manifest()) else { return Vec::new(); };
    content.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn cmd_synthesize_runs(r: &Resolved, action: SynthRunAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        SynthRunAction::List { limit } => {
            let mut entries = read_synthesize_runs_manifest();
            entries.reverse();
            entries.truncate(limit);
            if r.json {
                emit(true, JsonValue::Array(entries));
            } else {
                if entries.is_empty() {
                    println!("No synthesize runs recorded yet. Run `worldcli synthesize ...` first.");
                    return Ok(());
                }
                for e in &entries {
                    let ts = e.get("run_timestamp").and_then(|v| v.as_str()).unwrap_or("")[..19.min(e.get("run_timestamp").and_then(|v| v.as_str()).unwrap_or("").len())].to_string();
                    let id = e.get("run_id").and_then(|v| v.as_str()).unwrap_or("");
                    let id_short = &id[..8.min(id.len())];
                    let scope = e.get("scope_label").and_then(|v| v.as_str()).unwrap_or("?");
                    let q_preview = e.get("question_preview").and_then(|v| v.as_str()).unwrap_or("");
                    let b = e.get("before_count").and_then(|v| v.as_i64()).unwrap_or(0);
                    let a = e.get("after_count").and_then(|v| v.as_i64()).unwrap_or(0);
                    let cost = e.get("cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    println!("{id_short}  [{ts}]  {scope}  B:{b} A:{a}  ${:.4}  — {}",
                        cost, q_preview);
                }
            }
        }
        SynthRunAction::Show { id } => {
            let dir = synthesize_runs_dir();
            let exact = dir.join(format!("{}.json", id));
            if exact.exists() {
                let s = std::fs::read_to_string(&exact)?;
                let v: JsonValue = serde_json::from_str(&s).unwrap_or(JsonValue::String(s));
                emit(r.json, v);
                return Ok(());
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.starts_with(&id) && fname.ends_with(".json") {
                        let s = std::fs::read_to_string(entry.path())?;
                        let v: JsonValue = serde_json::from_str(&s).unwrap_or(JsonValue::String(s));
                        emit(r.json, v);
                        return Ok(());
                    }
                }
            }
            return Err(Box::new(CliError::NotFound(format!("synthesize run starting with '{}'", id))));
        }
        SynthRunAction::Search { query } => {
            let q = query.to_lowercase();
            let entries = read_synthesize_runs_manifest();
            let hits: Vec<JsonValue> = entries.into_iter()
                .filter(|e| e.to_string().to_lowercase().contains(&q))
                .collect();
            emit(r.json, JsonValue::Array(hits));
        }
    }
    Ok(())
}

fn cmd_rubric(r: &Resolved, action: RubricAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        RubricAction::List => {
            let rubrics = list_rubrics().map_err(Box::<dyn std::error::Error>::from)?;
            if rubrics.is_empty() {
                if !r.json {
                    println!("No rubrics found at {}.", rubrics_dir().display());
                    println!("See reports/rubrics/README.md for the authoring convention.");
                }
                emit(r.json, JsonValue::Array(Vec::new()));
                return Ok(());
            }
            let out: Vec<JsonValue> = rubrics.iter().map(|rb| json!({
                "name": rb.name,
                "version": rb.version,
                "description": rb.description,
                "path": rb.path.display().to_string(),
            })).collect();
            if r.json {
                emit(true, JsonValue::Array(out));
            } else {
                for rb in &rubrics {
                    println!("{:<40} v{}", rb.name, rb.version);
                    if !rb.description.is_empty() {
                        let desc = if rb.description.chars().count() > 100 {
                            let s: String = rb.description.chars().take(100).collect();
                            format!("{}…", s)
                        } else { rb.description.clone() };
                        println!("  {}", desc);
                    }
                }
            }
        }
        RubricAction::Show { name } => {
            let rb = load_rubric(&name).map_err(Box::<dyn std::error::Error>::from)?;
            if r.json {
                emit(true, json!({
                    "name": rb.name,
                    "version": rb.version,
                    "description": rb.description,
                    "prompt": rb.prompt,
                    "raw": rb.raw,
                    "path": rb.path.display().to_string(),
                }));
            } else {
                println!("{}", rb.raw);
            }
        }
        RubricAction::Search { query } => {
            let q = query.to_lowercase();
            let rubrics = list_rubrics().map_err(Box::<dyn std::error::Error>::from)?;
            let hits: Vec<&RubricFile> = rubrics.iter()
                .filter(|rb| rb.raw.to_lowercase().contains(&q))
                .collect();
            if r.json {
                let out: Vec<JsonValue> = hits.iter().map(|rb| json!({
                    "name": rb.name, "version": rb.version,
                    "description": rb.description,
                    "path": rb.path.display().to_string(),
                })).collect();
                emit(true, JsonValue::Array(out));
            } else {
                for rb in hits {
                    println!("{:<40} v{}  — {}", rb.name, rb.version, rb.description);
                }
            }
        }
    }
    Ok(())
}

// ─── Evaluate (rubric-driven LLM judgments on a before/after window) ───

/// One LLM-judged verdict on one character reply.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct EvalVerdict {
    judgment: String,         // "yes" | "no" | "mixed"
    confidence: String,       // "high" | "medium" | "low"
    quote: String,            // short quote from the reply that triggered the call
    reasoning: String,        // 1-2 sentences
}

fn evaluator_system_prompt() -> &'static str {
    r#"You are a rubric-driven evaluator for character replies in a text-based roleplay / novel-shaped world. You will receive:

  1. A RUBRIC — a qualitative question describing what to judge.
  2. The immediate USER TURN that preceded the reply (for context).
  3. The CHARACTER REPLY being evaluated.

Answer ONLY the rubric's question, applied to this specific reply. Ignore anything in the reply that isn't the rubric's concern. Be honest about ambiguity — use "mixed" when the reply partly qualifies and partly doesn't.

Return a strict JSON object with exactly these fields:

  {
    "judgment":   "yes" | "no" | "mixed",
    "confidence": "high" | "medium" | "low",
    "quote":      "the specific line or phrase in the reply that most triggered the judgment (≤ 15 words)",
    "reasoning":  "one or two sentences explaining the judgment in the rubric's terms"
  }

No preface, no markdown, no extra keys. Just the JSON."#
}

fn evaluator_user_prompt(
    rubric: &str,
    context_turns: &[(String, String)],  // (speaker_label, content) in chronological order
    reply: &str,
) -> String {
    let scene = if context_turns.is_empty() {
        "(no preceding context available)".to_string()
    } else {
        context_turns.iter()
            .map(|(who, content)| format!("{}: {}", who, content.trim()))
            .collect::<Vec<_>>()
            .join("\n\n")
    };
    format!(
        "RUBRIC:\n{}\n\nSCENE (preceding conversation, chronological — context only, NOT being judged):\n\n{}\n\nCHARACTER REPLY (this is what you're judging — the next turn after the scene above):\n{}",
        rubric.trim(),
        scene,
        reply.trim(),
    )
}

async fn evaluate_one(
    base_url: &str,
    api_key: &str,
    model: &str,
    rubric: &str,
    context_turns: &[(String, String)],
    reply: &str,
) -> Result<(EvalVerdict, openai::Usage), String> {
    let req = openai::ChatRequest {
        model: model.to_string(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: evaluator_system_prompt().to_string() },
            openai::ChatMessage { role: "user".to_string(), content: evaluator_user_prompt(rubric, context_turns, reply) },
        ],
        temperature: Some(0.0),
        max_completion_tokens: Some(220),
        response_format: Some(openai::ResponseFormat { format_type: "json_object".to_string() }),
    };
    let resp = openai::chat_completion_with_base(base_url, api_key, &req).await
        .map_err(|e| format!("evaluate call failed: {}", e))?;
    let raw = resp.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "evaluator returned no choices".to_string())?;
    let mut verdict: EvalVerdict = serde_json::from_str(&raw)
        .map_err(|e| format!("evaluator JSON parse error: {} (body: {})", e, raw))?;
    // Normalize judgment + confidence to lowercase so downstream aggregation
    // doesn't need to worry about case variation.
    verdict.judgment = verdict.judgment.to_lowercase();
    verdict.confidence = verdict.confidence.to_lowercase();
    let usage = resp.usage.unwrap_or(openai::Usage {
        prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
    });
    Ok((verdict, usage))
}

/// Pull the message window for one side of the ref via direct SQL so
/// the time filter happens at the database (not after pulling a
/// fixed-size recent slice). Otherwise a thread with more than
/// ~pull_limit post-commit messages leaves the before-window empty
/// because everything recent lives in the after window.
///
/// We pull a generous working set of role-matching messages on the
/// correct side of the cutoff, then for each eval target attach the
/// nearest preceding user turn from the FULL chronological thread —
/// the user turn might be paginated outside the working-set window
/// if filtering dropped the nearest one.
/// What's being evaluated. The Character variant spans BOTH the
/// character's solo thread AND their replies in every group chat
/// they're a member of (joined via sender_character_id); the Group
/// variant is a single group thread, every assistant reply inside
/// it regardless of sender. This is the character-vs-surface
/// distinction that matters for craft evaluation: when you ask
/// "what has this character been saying lately," you want all of
/// their surfaces combined.
enum EvalScope {
    Character { character_id: String, solo_thread_id: String },
    Group     { thread_id: String },
}

/// `EvalTriple` = (target message, preceding turns context, is_group_flag).
/// The preceding-turns vector carries the N most recent turns before
/// the target (both user and assistant, chronological order). The
/// is_group flag tells downstream code which table to read from when
/// it needs auxiliary context (settings_update rows, etc.).
type EvalTriple = (
    app_lib::db::queries::Message,
    Vec<app_lib::db::queries::Message>, // preceding turns, chronological
    bool, // is_group
);

fn pull_eval_window(
    conn: &rusqlite::Connection,
    scope: &EvalScope,
    cutoff_ts: &str,
    direction: &str,  // "before" or "after"
    role: &str,
    limit: i64,
    context_turns: i64,
) -> Result<Vec<EvalTriple>, Box<dyn std::error::Error>> {
    // Normalize the cutoff to the same UTC-with-microseconds shape the
    // messages tables store — "YYYY-MM-DDTHH:MM:SS.ffffff+00:00" —
    // so string comparison matches real-time ordering. git commit
    // timestamps come in with the committer's timezone offset
    // ("T10:16:41-05:00"), which breaks character-wise comparison
    // against stored UTC strings ("T15:16:41+00:00").
    let cutoff = chrono::DateTime::parse_from_rfc3339(cutoff_ts)
        .map(|dt| dt.with_timezone(&chrono::Utc)
            .to_rfc3339_opts(chrono::SecondsFormat::Micros, true))
        .unwrap_or_else(|_| cutoff_ts.to_string());

    let role_clause = if role == "any" { String::new() } else {
        format!("AND role = '{}'", role.replace('\'', ""))
    };
    let noise_clause = "AND role NOT IN ('illustration','video','inventory_update','imagined_chapter','settings_update','system')";
    let (op, order) = if direction == "before" { ("<", "DESC") } else { (">=", "ASC") };

    // Each target gets tagged with its source table ('solo' vs 'group')
    // so the preceding-user-turn lookup can query the right table.
    // For Character scope we UNION ALL over the solo-thread table AND
    // the group_messages table filtered to this character's sender_id
    // — that surfaces the character's replies wherever they occurred.
    let cols = "message_id, thread_id, role, content, tokens_estimate, sender_character_id,
                created_at, world_day, world_time, address_to, mood_chain, is_proactive";
    let targets: Vec<(app_lib::db::queries::Message, String)> = match scope {
        EvalScope::Character { character_id, solo_thread_id } => {
            let sql = format!(
                "SELECT {cols}, 'solo' AS src FROM messages
                 WHERE thread_id = ?1 AND created_at {op} ?2 {role_clause} {noise_clause}
                 UNION ALL
                 SELECT {cols}, 'group' AS src FROM group_messages
                 WHERE sender_character_id = ?3 AND created_at {op} ?2 {role_clause} {noise_clause}
                 ORDER BY created_at {order} LIMIT ?4"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(
                rusqlite::params![solo_thread_id, cutoff, character_id, limit],
                |r| Ok((app_lib::db::queries::Message {
                    message_id: r.get(0)?,
                    thread_id: r.get(1)?,
                    role: r.get(2)?,
                    content: r.get(3)?,
                    tokens_estimate: r.get(4)?,
                    sender_character_id: r.get(5)?,
                    created_at: r.get(6)?,
                    world_day: r.get(7)?,
                    world_time: r.get(8)?,
                    address_to: r.get(9)?,
                    mood_chain: r.get(10)?,
                    is_proactive: r.get::<_, Option<i64>>(11)?.map(|v| v != 0).unwrap_or(false),
                }, r.get::<_, String>(12)?)),
            )?;
            rows.filter_map(|r| r.ok()).collect()
        }
        EvalScope::Group { thread_id } => {
            let sql = format!(
                "SELECT {cols}, 'group' AS src FROM group_messages
                 WHERE thread_id = ?1 AND created_at {op} ?2 {role_clause} {noise_clause}
                 ORDER BY created_at {order} LIMIT ?3"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(
                rusqlite::params![thread_id, cutoff, limit],
                |r| Ok((app_lib::db::queries::Message {
                    message_id: r.get(0)?,
                    thread_id: r.get(1)?,
                    role: r.get(2)?,
                    content: r.get(3)?,
                    tokens_estimate: r.get(4)?,
                    sender_character_id: r.get(5)?,
                    created_at: r.get(6)?,
                    world_day: r.get(7)?,
                    world_time: r.get(8)?,
                    address_to: r.get(9)?,
                    mood_chain: r.get(10)?,
                    is_proactive: r.get::<_, Option<i64>>(11)?.map(|v| v != 0).unwrap_or(false),
                }, r.get::<_, String>(12)?)),
            )?;
            rows.filter_map(|r| r.ok()).collect()
        }
    };

    // For each target, find the nearest preceding user turn in the
    // correct table (src tells us which).
    // Clamp context_turns to at least 1 — every eval needs at least
    // the triggering user turn as context. Higher values include
    // more surrounding turns (both user and assistant roles).
    let n_context = context_turns.max(1);

    let mut pairs: Vec<EvalTriple> = Vec::new();
    for (m, src) in targets {
        let is_group = src == "group";
        let tbl = if is_group { "group_messages" } else { "messages" };
        // Pull N preceding turns of both roles (user + assistant),
        // excluding noise roles. Chronological order so the
        // evaluator reads the scene forward.
        let ctx_sql = format!(
            "SELECT message_id, thread_id, role, content, tokens_estimate, sender_character_id,
                    created_at, world_day, world_time, address_to, mood_chain, is_proactive
             FROM {tbl}
             WHERE thread_id = ?1
               AND created_at < ?2
               AND role IN ('user', 'assistant', 'narrative')
             ORDER BY created_at DESC LIMIT ?3"
        );
        let mut stmt = conn.prepare(&ctx_sql)?;
        let rows = stmt.query_map(
            rusqlite::params![m.thread_id, m.created_at, n_context],
            |r| Ok(app_lib::db::queries::Message {
                message_id: r.get(0)?,
                thread_id: r.get(1)?,
                role: r.get(2)?,
                content: r.get(3)?,
                tokens_estimate: r.get(4)?,
                sender_character_id: r.get(5)?,
                created_at: r.get(6)?,
                world_day: r.get(7)?,
                world_time: r.get(8)?,
                address_to: r.get(9)?,
                mood_chain: r.get(10)?,
                is_proactive: r.get::<_, Option<i64>>(11)?.map(|v| v != 0).unwrap_or(false),
            }),
        )?;
        let mut context: Vec<app_lib::db::queries::Message> = rows.filter_map(|r| r.ok()).collect();
        context.reverse(); // chronological
        pairs.push((m, context, is_group));
    }
    Ok(pairs)
}

/// Reconstruct the chat-settings state active at `at_ts` in the given
/// thread by walking `settings_update` rows backwards. For each key
/// (response_length, leader, narration_tone, etc.) the first time we
/// see it in reverse-chronological order, that's its active value —
/// every earlier change has been superseded. Returns a map of
/// key → display-formatted value.
///
/// Purpose: any prompt-stack experiment that fails to account for
/// chat-settings confounds may attribute behavior-shifts to rule
/// commits that were actually caused by the user flipping a setting
/// (response_length most notably). Every eval verdict gets stamped
/// with the then-active settings so the analyst can read the
/// confound or stratify against it.
fn active_settings_at(
    conn: &rusqlite::Connection,
    thread_id: &str,
    at_ts: &str,
    is_group: bool,
) -> std::collections::HashMap<String, String> {
    let tbl = if is_group { "group_messages" } else { "messages" };
    let sql = format!(
        "SELECT content FROM {tbl}
         WHERE thread_id = ?1 AND role = 'settings_update' AND created_at < ?2
         ORDER BY created_at DESC"
    );
    let mut out: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let Ok(mut stmt) = conn.prepare(&sql) else { return out; };
    let Ok(rows) = stmt.query_map(
        rusqlite::params![thread_id, at_ts],
        |r| r.get::<_, String>(0),
    ) else { return out; };
    for row in rows.flatten() {
        let Ok(body) = serde_json::from_str::<JsonValue>(&row) else { continue; };
        let Some(changes) = body.get("changes").and_then(|v| v.as_array()) else { continue; };
        for ch in changes {
            let (Some(k), Some(to_val)) = (
                ch.get("key").and_then(|v| v.as_str()),
                ch.get("to").and_then(|v| v.as_str()),
            ) else { continue; };
            // First occurrence in DESC order wins — that's the most recent change.
            out.entry(k.to_string()).or_insert_with(|| to_val.to_string());
        }
    }
    out
}

/// Graded item — one reply + its user prompt, extracted from a run file.
/// Ask runs produce one item; replay and scenario runs can produce N.
#[derive(Debug)]
struct GradeItem {
    run_id: String,
    run_kind: &'static str, // "ask" | "replay" | "scenario"
    sub_index: usize,       // 0 for ask; 0..N for replay (per ref) or scenario (per variant)
    sub_label: String,      // "" for ask; ref name for replay; variant label for scenario
    prompt: String,
    reply: String,
}

/// Search the three run-log directories for a file matching the given
/// id (or short prefix). Returns the full path + which kind of run it is.
fn find_run_file(id: &str) -> Option<(PathBuf, &'static str)> {
    let candidates: Vec<(PathBuf, &'static str)> = vec![
        (runs_dir(), "ask"),
        (replay_runs_dir(), "replay"),
        (scenario_runs_dir(), "scenario"),
    ];
    for (dir, kind) in candidates {
        if !dir.exists() { continue; }
        // Try exact first.
        let exact = dir.join(format!("{}.json", id));
        if exact.exists() { return Some((exact, kind)); }
        // Prefix match.
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with(id) && fname.ends_with(".json") {
                    return Some((entry.path(), kind));
                }
            }
        }
    }
    None
}

/// Extract one or more GradeItems from a run file based on its kind.
/// Ask = 1 item; replay = N items (one per ref); scenario = N items
/// (one per variant).
fn extract_grade_items(path: &std::path::Path, kind: &'static str) -> Result<Vec<GradeItem>, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let v: JsonValue = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let run_id = v.get("run_id").or_else(|| v.get("id"))
        .and_then(|x| x.as_str())
        .unwrap_or("?").to_string();
    let mut out = Vec::new();
    match kind {
        "ask" => {
            let prompt = v.get("prompt").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let reply = v.get("reply").and_then(|x| x.as_str()).unwrap_or("").to_string();
            out.push(GradeItem { run_id, run_kind: kind, sub_index: 0,
                sub_label: String::new(), prompt, reply });
        }
        "replay" => {
            let prompt = v.get("prompt").and_then(|x| x.as_str()).unwrap_or("").to_string();
            if let Some(results) = v.get("results").and_then(|x| x.as_array()) {
                for (i, res) in results.iter().enumerate() {
                    let ref_label = res.get("ref").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    // When --n > 1, each ref produced multiple samples.
                    // Disambiguate the sub_label so grade output shows
                    // "0202651#1, 0202651#2, ..." instead of repeating.
                    let sample_count = res.get("sample_count").and_then(|x| x.as_u64()).unwrap_or(1);
                    let sub_label = if sample_count > 1 {
                        let sample_idx = res.get("sample_index").and_then(|x| x.as_u64()).unwrap_or(0);
                        format!("{}#{}", ref_label, sample_idx + 1)
                    } else {
                        ref_label
                    };
                    let reply = res.get("reply").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    out.push(GradeItem { run_id: run_id.clone(), run_kind: kind,
                        sub_index: i, sub_label, prompt: prompt.clone(), reply });
                }
            }
        }
        "scenario" => {
            if let Some(variants) = v.get("variants").and_then(|x| x.as_array()) {
                for (i, var) in variants.iter().enumerate() {
                    let sub_label = var.get("label").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let prompt = var.get("prompt").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let reply = var.get("reply").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    out.push(GradeItem { run_id: run_id.clone(), run_kind: kind,
                        sub_index: i, sub_label, prompt, reply });
                }
            }
        }
        _ => {}
    }
    Ok(out)
}

async fn cmd_grade_runs(
    r: &Resolved,
    api_key: &str,
    run_ids: &[String],
    rubric: Option<&str>,
    rubric_ref: Option<&str>,
    rubric_file: Option<&std::path::Path>,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    if run_ids.is_empty() {
        return Err(Box::<dyn std::error::Error>::from(
            "at least one run_id required".to_string()));
    }

    // Resolve rubric (same precedence as `evaluate`).
    let sources = [rubric.is_some(), rubric_ref.is_some(), rubric_file.is_some()]
        .iter().filter(|b| **b).count();
    if sources != 1 {
        return Err(Box::<dyn std::error::Error>::from(
            "pass exactly one of --rubric, --rubric-ref, or --rubric-file".to_string()));
    }
    let (rubric_text, rubric_ref_name, rubric_ref_version) = if let Some(name) = rubric_ref {
        let rb = load_rubric(name).map_err(Box::<dyn std::error::Error>::from)?;
        (rb.prompt, Some(rb.name), Some(rb.version))
    } else if let Some(p) = rubric_file {
        (std::fs::read_to_string(p)?, None, None)
    } else {
        (rubric.unwrap().to_string(), None, None)
    };

    // Resolve all run_ids to GradeItems.
    let mut items: Vec<GradeItem> = Vec::new();
    for rid in run_ids {
        let (path, kind) = find_run_file(rid).ok_or_else(|| Box::<dyn std::error::Error>::from(
            format!("run '{}' not found in runs / replay-runs / scenario-runs", rid)))?;
        let extracted = extract_grade_items(&path, kind)
            .map_err(Box::<dyn std::error::Error>::from)?;
        if extracted.is_empty() {
            eprintln!("[worldcli] warning: run '{}' yielded zero gradeable items", rid);
        }
        items.extend(extracted);
    }

    if items.is_empty() {
        return Err(Box::<dyn std::error::Error>::from(
            "no gradeable items found across the given runs".to_string()));
    }

    // Model + cost projection.
    let model_config = {
        let conn = r.db.conn.lock().unwrap();
        orchestrator::load_model_config(&conn)
    };
    let model = model_override.unwrap_or(&model_config.memory_model).to_string();

    let rubric_tokens = estimate_tokens(&rubric_text);
    let per_call_in = rubric_tokens + 400 /*reply*/ + 200 /*system*/ + 150 /*slack*/;
    let per_call_out: i64 = 220;
    let per_call_usd = project_cost(&model, per_call_in, per_call_out, &r.cfg.model_pricing);
    let total_projected = per_call_usd * (items.len() as f64);
    let per_call_cap = r.cfg.budget.per_call_usd;
    let confirm = confirm_cost.unwrap_or(0.0);
    if total_projected > per_call_cap && confirm < total_projected {
        return Err(Box::new(CliError::Budget {
            kind: "per_call (grade total)".to_string(),
            projected_usd: total_projected,
            cap_usd: per_call_cap,
            confirm_at_least: (total_projected * 1.05).max(0.01),
        }));
    }

    if !r.json {
        eprintln!("[worldcli] grading {} items via {} — projected≈${:.4}",
            items.len(), model, total_projected);
        eprintln!("[worldcli] rubric: {}", rubric_text.lines().next().unwrap_or("").chars().take(100).collect::<String>());
    }

    // Grade each.
    let base_url = model_config.chat_api_base();
    let mut verdicts: Vec<JsonValue> = Vec::new();
    let mut total_in_tokens: i64 = 0;
    let mut total_out_tokens: i64 = 0;
    for (i, item) in items.iter().enumerate() {
        eprint!("\r[worldcli] graded {}/{}", i + 1, items.len());
        // Use the preceding user-prompt as the "context" turn (one-turn
        // scene). `evaluate_one` takes a list of (speaker, content) pairs.
        let ctx = vec![("User".to_string(), item.prompt.clone())];
        match evaluate_one(&base_url, api_key, &model, &rubric_text, &ctx, &item.reply).await {
            Ok((v, u)) => {
                total_in_tokens += u.prompt_tokens as i64;
                total_out_tokens += u.completion_tokens as i64;
                verdicts.push(json!({
                    "run_id": item.run_id,
                    "run_kind": item.run_kind,
                    "sub_index": item.sub_index,
                    "sub_label": item.sub_label,
                    "judgment": v.judgment,
                    "confidence": v.confidence,
                    "quote": v.quote,
                    "reasoning": v.reasoning,
                    "reply_preview": item.reply.chars().take(200).collect::<String>(),
                }));
            }
            Err(e) => {
                verdicts.push(json!({
                    "run_id": item.run_id,
                    "run_kind": item.run_kind,
                    "sub_index": item.sub_index,
                    "sub_label": item.sub_label,
                    "error": e,
                }));
            }
        }
    }
    eprintln!();

    let actual_usd = actual_cost(&model, total_in_tokens, total_out_tokens, &r.cfg.model_pricing);
    append_cost_log(&CostEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: model.clone(),
        prompt_tokens: total_in_tokens,
        completion_tokens: total_out_tokens,
        usd: actual_usd,
    });

    // Aggregate.
    let mut yes = 0; let mut no = 0; let mut mixed = 0; let mut err = 0;
    for v in &verdicts {
        match v.get("judgment").and_then(|x| x.as_str()) {
            Some("yes") => yes += 1,
            Some("no") => no += 1,
            Some("mixed") => mixed += 1,
            _ => err += 1,
        }
    }

    let envelope = json!({
        "run_ids": run_ids,
        "item_count": items.len(),
        "rubric": rubric_text,
        "rubric_ref": rubric_ref_name,
        "rubric_version": rubric_ref_version,
        "model": model,
        "cost": {
            "prompt_tokens": total_in_tokens,
            "completion_tokens": total_out_tokens,
            "actual_usd": actual_usd,
        },
        "aggregate": {
            "yes": yes, "no": no, "mixed": mixed, "errors": err,
            "yes_rate": (yes as f64) / (items.len().max(1) as f64),
            "effective_fire_rate": ((yes as f64) + 0.5 * (mixed as f64)) / (items.len().max(1) as f64),
        },
        "verdicts": verdicts,
    });

    if r.json {
        emit(true, envelope);
    } else {
        println!("=== GRADE-RUNS ===");
        println!("items:     {}", items.len());
        println!("rubric:    {}", rubric_text.lines().next().unwrap_or(""));
        println!("model:     {}", model);
        println!();
        println!("AGGREGATE: yes={} no={} mixed={} errors={}", yes, no, mixed, err);
        println!("effective fire-rate: {:.3} (yes=1, mixed=0.5, no=0)",
            ((yes as f64) + 0.5 * (mixed as f64)) / (items.len().max(1) as f64));
        println!();
        println!("Per-item verdicts:");
        for v in &verdicts {
            let rid = v.get("run_id").and_then(|x| x.as_str()).unwrap_or("?");
            let rid_short = &rid[..8.min(rid.len())];
            let label = v.get("sub_label").and_then(|x| x.as_str()).unwrap_or("");
            let sub = if label.is_empty() { String::new() } else { format!(" [{}]", label) };
            if let Some(err) = v.get("error").and_then(|x| x.as_str()) {
                println!("  {rid_short}{sub} ERROR: {err}");
                continue;
            }
            let j = v.get("judgment").and_then(|x| x.as_str()).unwrap_or("?");
            let c = v.get("confidence").and_then(|x| x.as_str()).unwrap_or("?");
            let reasoning = v.get("reasoning").and_then(|x| x.as_str()).unwrap_or("").chars().take(140).collect::<String>();
            println!("  {rid_short}{sub}  {} ({}) — {}", j, c, reasoning);
        }
        println!();
        eprintln!("[worldcli] actual cost ${:.4} ({} in / {} out tok)",
            actual_usd, total_in_tokens, total_out_tokens);
    }
    Ok(())
}

async fn cmd_evaluate(
    r: &Resolved,
    api_key: &str,
    git_ref: &str,
    end_ref: Option<&str>,
    limit: i64,
    character_id: Option<&str>,
    group_chat_id: Option<&str>,
    rubric: Option<&str>,
    rubric_file: Option<&std::path::Path>,
    rubric_ref: Option<&str>,
    role: &str,
    context_turns: i64,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
    repo: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // ─── Resolve rubric source — at most one of the three paths ──────
    let sources_given = [rubric.is_some(), rubric_file.is_some(), rubric_ref.is_some()]
        .iter().filter(|b| **b).count();
    if sources_given > 1 {
        return Err(Box::<dyn std::error::Error>::from(
            "pass exactly one of --rubric, --rubric-file, or --rubric-ref".to_string()));
    }
    if sources_given == 0 {
        return Err(Box::<dyn std::error::Error>::from(
            "one of --rubric, --rubric-file, or --rubric-ref is required".to_string()));
    }
    let (rubric_text, rubric_ref_name, rubric_ref_version) = if let Some(name) = rubric_ref {
        let rb = load_rubric(name).map_err(Box::<dyn std::error::Error>::from)?;
        (rb.prompt, Some(rb.name), Some(rb.version))
    } else if let Some(p) = rubric_file {
        let t = std::fs::read_to_string(p)
            .map_err(|e| format!("failed to read --rubric-file {}: {}", p.display(), e))?;
        (t, None, None)
    } else {
        (rubric.unwrap().to_string(), None, None)
    };
    if rubric_text.trim().is_empty() {
        return Err(Box::<dyn std::error::Error>::from("rubric is empty".to_string()));
    }

    // ─── Resolve scope (solo character vs. group chat) ───────────────
    if character_id.is_some() && group_chat_id.is_some() {
        return Err(Box::<dyn std::error::Error>::from(
            "pass either --character or --group-chat, not both".to_string()));
    }
    if character_id.is_none() && group_chat_id.is_none() {
        return Err(Box::<dyn std::error::Error>::from(
            "one of --character or --group-chat is required".to_string()));
    }
    if let Some(cid) = character_id { let _ = r.check_character(cid)?; }

    let (before_sha, before_ts, before_subject) = git_resolve_ref(repo, git_ref)?;
    let (after_sha, after_ts, after_subject) = match end_ref {
        Some(er) => git_resolve_ref(repo, er)?,
        None => (before_sha.clone(), before_ts.clone(), before_subject.clone()),
    };

    // ─── Pull windows + model config + display label ─────────────────
    let (model_config, before_pairs, after_pairs, display_label) = {
        let conn = r.db.conn.lock().unwrap();
        let mut mc = orchestrator::load_model_config(&conn);
        if let Some(m) = model_override { mc.memory_model = m.to_string(); }

        let (scope, display) = if let Some(cid) = character_id {
            let thread = get_thread_for_character(&conn, cid)?;
            let ch = get_character(&conn, cid)?;
            (
                EvalScope::Character {
                    character_id: cid.to_string(),
                    solo_thread_id: thread.thread_id,
                },
                format!("{} (solo + groups)", ch.display_name),
            )
        } else {
            let gcid = group_chat_id.unwrap();
            let gc = get_group_chat(&conn, gcid)
                .map_err(|e| Box::<dyn std::error::Error>::from(
                    format!("group_chat {}: {}", gcid, e)))?;
            r.check_world(&gc.world_id)?;
            (
                EvalScope::Group { thread_id: gc.thread_id },
                format!("{} (group)", gc.display_name),
            )
        };

        let before_raw = pull_eval_window(&conn, &scope, &before_ts, "before", role, limit, context_turns)?;
        let after_raw  = pull_eval_window(&conn, &scope, &after_ts,  "after",  role, limit, context_turns)?;
        // Enrich each target with the chat-settings state active at
        // reply-time, so the evaluator output can surface the confound
        // response_length / leader / narration_tone etc. present when
        // this particular message was generated.
        let enrich = |triples: Vec<EvalTriple>| -> Vec<(app_lib::db::queries::Message, Vec<app_lib::db::queries::Message>, std::collections::HashMap<String, String>)> {
            triples.into_iter().map(|(m, context, is_group)| {
                let settings = active_settings_at(&conn, &m.thread_id, &m.created_at, is_group);
                (m, context, settings)
            }).collect()
        };
        (mc, enrich(before_raw), enrich(after_raw), display)
    };
    let character_name = display_label;

    let total_msgs = before_pairs.len() + after_pairs.len();
    if total_msgs == 0 {
        return Err(Box::<dyn std::error::Error>::from(
            "no messages in either window; widen --limit or pick a different ref".to_string()));
    }

    // ─── Cost projection ─────────────────────────────────────────────
    // Each eval call: ~rubric + ~400 tok context + ~150 tok output.
    let rubric_tokens = estimate_tokens(&rubric_text);
    // Budget: rubric + system + reply + per-context-turn overhead + slack.
    // Each context turn ~150 tokens typical; reply ~300; system ~200;
    // rubric varies. At context_turns=3 that's ~450 context tokens,
    // adding to the previous ~600 baseline → ~1050 tokens/call.
    let per_call_in = rubric_tokens + 300 /*reply*/ + 200 /*system*/ + (context_turns as i64 * 180) + 150 /*slack*/;
    let per_call_out: i64 = 220;
    let per_call_usd = project_cost(&model_config.memory_model, per_call_in, per_call_out, &r.cfg.model_pricing);
    let total_projected = per_call_usd * (total_msgs as f64);

    let daily_so_far = rolling_24h_total_usd();
    let daily_after = daily_so_far + total_projected;
    let per_call_cap = r.cfg.budget.per_call_usd;
    let daily_cap = r.cfg.budget.daily_usd;
    let confirm = confirm_cost.unwrap_or(0.0);
    if total_projected > per_call_cap && confirm < total_projected {
        return Err(Box::new(CliError::Budget {
            kind: "per_call (total run)".to_string(),
            projected_usd: total_projected,
            cap_usd: per_call_cap,
            confirm_at_least: (total_projected * 1.05).max(0.01),
        }));
    }
    if daily_after > daily_cap && confirm < total_projected {
        return Err(Box::new(CliError::Budget {
            kind: "daily".to_string(),
            projected_usd: daily_after,
            cap_usd: daily_cap,
            confirm_at_least: (total_projected * 1.05).max(0.01),
        }));
    }

    if !r.json {
        eprintln!("[worldcli] evaluating {} msgs ({} before / {} after) via {} — total projected≈${:.4}; 24h spent=${:.4}/${:.2}",
            total_msgs, before_pairs.len(), after_pairs.len(), model_config.memory_model,
            total_projected, daily_so_far, daily_cap);
        eprintln!("[worldcli] rubric: {}", rubric_text.lines().next().unwrap_or("").chars().take(100).collect::<String>());
    }

    // ─── Run evaluator over each message ──────────────────────────────
    let base_url = model_config.chat_api_base();
    let eval_window = |pairs: &[(app_lib::db::queries::Message, Option<app_lib::db::queries::Message>)]| -> Vec<JsonValue> {
        pairs.iter().map(|(m, prev)| (m.clone(), prev.clone())).collect::<Vec<_>>()
            .into_iter().map(|(m, _)| json!({
                "message_id": m.message_id,
                "created_at": m.created_at,
                "content": m.content,
            })).collect::<Vec<_>>()
    };
    // (The closure above is a no-op shape; we run the actual async calls below.)
    let _ = eval_window;

    let mut total_in_tokens: i64 = 0;
    let mut total_out_tokens: i64 = 0;

    let run_window = |name: &'static str, pairs: Vec<(app_lib::db::queries::Message, Vec<app_lib::db::queries::Message>, std::collections::HashMap<String, String>)>|
      -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(Vec<JsonValue>, i64, i64), Box<dyn std::error::Error>>>>>
    {
        let base_url = base_url.clone();
        let api_key = api_key.to_string();
        let model = model_config.memory_model.clone();
        let rubric = rubric_text.clone();
        Box::pin(async move {
            let mut out: Vec<JsonValue> = Vec::new();
            let mut in_tok: i64 = 0;
            let mut out_tok: i64 = 0;
            for (i, (m, context, settings)) in pairs.iter().enumerate() {
                // Render context turns as labeled (speaker, content) pairs.
                // Assistant turns in group chats can have multiple
                // speakers; for solo threads the speaker is always the
                // same character. We use "User" and "Character" as
                // generic labels to keep the prompt compact.
                let ctx_labeled: Vec<(String, String)> = context.iter().map(|cm| {
                    let label = match cm.role.as_str() {
                        "user" => "User".to_string(),
                        "assistant" => "Character".to_string(),
                        "narrative" => "[Narrative]".to_string(),
                        other => other.to_string(),
                    };
                    (label, cm.content.clone())
                }).collect();
                // JSON-able settings: HashMap → sorted Vec<(k,v)> for stable output.
                let mut settings_sorted: Vec<(String, String)> = settings.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                settings_sorted.sort();
                let settings_json: serde_json::Map<String, JsonValue> = settings_sorted.into_iter()
                    .map(|(k, v)| (k, JsonValue::String(v)))
                    .collect();
                match evaluate_one(&base_url, &api_key, &model, &rubric, &ctx_labeled, &m.content).await {
                    Ok((v, u)) => {
                        in_tok += u.prompt_tokens as i64;
                        out_tok += u.completion_tokens as i64;
                        out.push(json!({
                            "window": name,
                            "message_id": m.message_id,
                            "created_at": m.created_at,
                            "content_preview": m.content.chars().take(200).collect::<String>(),
                            "judgment": v.judgment,
                            "confidence": v.confidence,
                            "quote": v.quote,
                            "reasoning": v.reasoning,
                            "active_settings": settings_json,
                        }));
                    }
                    Err(e) => {
                        out.push(json!({
                            "window": name,
                            "message_id": m.message_id,
                            "created_at": m.created_at,
                            "error": e,
                            "active_settings": settings_json,
                        }));
                    }
                }
                eprint!("\r[worldcli] {} evaluated {}/{}", name, i + 1, pairs.len());
            }
            eprintln!();
            Ok((out, in_tok, out_tok))
        })
    };

    let (before_results, b_in, b_out) = run_window("before", before_pairs).await?;
    total_in_tokens += b_in; total_out_tokens += b_out;
    let (after_results, a_in, a_out) = run_window("after", after_pairs).await?;
    total_in_tokens += a_in; total_out_tokens += a_out;

    // ─── Aggregate + persist cost ─────────────────────────────────────
    let count_judgments = |rows: &[JsonValue]| -> (i64, i64, i64, i64) {
        let mut yes = 0; let mut no = 0; let mut mixed = 0; let mut err = 0;
        for r in rows {
            match r.get("judgment").and_then(|v| v.as_str()) {
                Some("yes") => yes += 1,
                Some("no") => no += 1,
                Some("mixed") => mixed += 1,
                _ => err += 1,
            }
        }
        (yes, no, mixed, err)
    };
    let (b_yes, b_no, b_mixed, b_err) = count_judgments(&before_results);
    let (a_yes, a_no, a_mixed, a_err) = count_judgments(&after_results);

    let actual_usd = actual_cost(&model_config.memory_model, total_in_tokens, total_out_tokens, &r.cfg.model_pricing);
    append_cost_log(&CostEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: model_config.memory_model.clone(),
        prompt_tokens: total_in_tokens,
        completion_tokens: total_out_tokens,
        usd: actual_usd,
    });

    // If this run used a library rubric, append a compact run-history
    // line to the rubric's markdown file. Auto-compounding craft capital.
    if let (Some(name), Some(version)) = (rubric_ref_name.as_ref(), rubric_ref_version.as_ref()) {
        let date = &chrono::Utc::now().to_rfc3339()[..10]; // YYYY-MM-DD
        let scope_label = character_id
            .map(|c| format!("--character {}", c))
            .unwrap_or_else(|| group_chat_id.map(|g| format!("--group-chat {}", g)).unwrap_or_default());
        let sha_short = &before_sha[..8.min(before_sha.len())];
        let line = format!(
            "- [{date}] commit {sha_short}, {scope_label} (v{version}) — BEFORE: yes={b_yes} no={b_no} mixed={b_mixed} err={b_err} | AFTER: yes={a_yes} no={a_no} mixed={a_mixed} err={a_err}",
            date = date, sha_short = sha_short, scope_label = scope_label, version = version,
            b_yes = b_yes, b_no = b_no, b_mixed = b_mixed, b_err = b_err,
            a_yes = a_yes, a_no = a_no, a_mixed = a_mixed, a_err = a_err,
        );
        append_rubric_run_history(name, &line);
    }

    // Persist the full evaluate run as a structured artifact under
    // ~/.worldcli/evaluate-runs/<id>.json so future queries can find
    // this run without re-reading prose reports. Substrate for the
    // future experiment registry; usable now via `evaluate-runs`.
    let eval_run_id = uuid::Uuid::new_v4().to_string();

    // ─── Emit ─────────────────────────────────────────────────────────
    let envelope = json!({
        "run_id": eval_run_id,
        "run_timestamp": chrono::Utc::now().to_rfc3339(),
        "ref": git_ref,
        "ref_resolved": before_sha,
        "ref_timestamp": before_ts,
        "ref_subject": before_subject,
        "end_ref": end_ref,
        "end_ref_resolved": end_ref.map(|_| after_sha.clone()),
        "end_ref_timestamp": end_ref.map(|_| after_ts.clone()),
        "end_ref_subject": end_ref.map(|_| after_subject.clone()),
        "character_id": character_id,
        "group_chat_id": group_chat_id,
        "scope_label": character_name,
        "role_filter": role,
        "context_turns": context_turns,
        "rubric": rubric_text,
        "rubric_ref": rubric_ref_name,
        "rubric_version": rubric_ref_version,
        "model": model_config.memory_model,
        "cost": {
            "prompt_tokens": total_in_tokens,
            "completion_tokens": total_out_tokens,
            "actual_usd": actual_usd,
        },
        "before": {
            "count": before_results.len(),
            "yes": b_yes, "no": b_no, "mixed": b_mixed, "errors": b_err,
            "messages": before_results,
        },
        "after": {
            "count": after_results.len(),
            "yes": a_yes, "no": a_no, "mixed": a_mixed, "errors": a_err,
            "messages": after_results,
        },
    });

    // Persist the full run envelope to the structured run log.
    write_evaluate_run(&eval_run_id, &envelope);

    if r.json {
        emit(true, envelope);
    } else {
        println!("=== EVALUATION ===");
        println!("ref:       {} ({})", git_ref, &before_sha[..8.min(before_sha.len())]);
        println!("subject:   {}", before_subject);
        let scope_id = character_id.or(group_chat_id).unwrap_or("?");
        println!("scope:     {} ({})", character_name, scope_id);
        println!("rubric:    {}", rubric_text.lines().next().unwrap_or(""));
        println!();
        println!("BEFORE window ({} msgs):", before_results.len());
        println!("  yes: {}   no: {}   mixed: {}   errors: {}", b_yes, b_no, b_mixed, b_err);
        println!("AFTER window  ({} msgs):", after_results.len());
        println!("  yes: {}   no: {}   mixed: {}   errors: {}", a_yes, a_no, a_mixed, a_err);
        println!();
        let delta = |bv: i64, av: i64| -> String {
            let d = av - bv;
            if d > 0 { format!("+{d}") } else { d.to_string() }
        };
        println!("DELTA:     yes {}   no {}   mixed {}",
            delta(b_yes, a_yes), delta(b_no, a_no), delta(b_mixed, a_mixed));
        println!();
        println!("Per-message details:");
        for r_row in before_results.iter().chain(after_results.iter()) {
            let w = r_row["window"].as_str().unwrap_or("");
            let ts = r_row["created_at"].as_str().unwrap_or("")[11..19].to_string();
            if let Some(err) = r_row.get("error").and_then(|v| v.as_str()) {
                println!("  [{ts} {:6}] ERROR: {}", w, err);
                continue;
            }
            let j = r_row["judgment"].as_str().unwrap_or("?");
            let c = r_row["confidence"].as_str().unwrap_or("?");
            let quote = r_row["quote"].as_str().unwrap_or("").chars().take(80).collect::<String>();
            let reasoning = r_row["reasoning"].as_str().unwrap_or("").chars().take(140).collect::<String>();
            // Chat-settings confound annotation. Print the few
            // behavior-affecting keys inline if present so the
            // analyst can see "was this reply under Short mode?"
            // without leaving the verdict line.
            let settings_summary: String = r_row.get("active_settings")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    let keys_of_interest = ["response_length", "leader", "narration_tone", "send_history"];
                    let parts: Vec<String> = keys_of_interest.iter()
                        .filter_map(|k| obj.get(*k).and_then(|v| v.as_str()).map(|v| format!("{}={}", k, v)))
                        .collect();
                    if parts.is_empty() { String::new() } else { format!("  [settings: {}]", parts.join(", ")) }
                })
                .unwrap_or_default();
            println!("  [{ts} {:6}] {} ({}) — \"{}\"{}", w, j, c, quote, settings_summary);
            println!("                      → {}", reasoning);
        }
        println!();
        eprintln!("[worldcli] actual cost ${:.4} ({} in / {} out tok)",
            actual_usd, total_in_tokens, total_out_tokens);
    }
    Ok(())
}

// ─── Synthesize (Mode B — qualitative synthesis, single call, prose) ────

fn synthesizer_system_prompt() -> &'static str {
    r#"You are a qualitative analyst of character-reply corpora for a text-based roleplay / worldbuilding project. You will receive:

  1. A QUESTION — an open-ended question about the corpus.
  2. A CORPUS — a bundle of character replies, each marked with its window (BEFORE or AFTER a git-commit cutoff), timestamp, scene context (the preceding user/character turns), and the active chat-settings state at the time the reply was generated.

Read the corpus carefully. Answer the question as thoughtfully and specifically as you can. When you make a claim, ground it with a brief quote (≤ 15 words) from the reply that supports it. Name patterns. Name surprises. Name what's NOT present that you'd expect to be. Be honest about ambiguity — if the corpus can't support a claim, say so.

If the question asks about change between BEFORE and AFTER windows, compare the two directly; if it asks about current state, focus on the AFTER window. If the chat-settings stamp on a reply is a likely confound (e.g. response_length=Short explaining brevity), name it.

Your output is prose. No JSON, no required headings. Structure so the reader can track which pattern you're naming and where in the corpus it shows up. Quote exactly when you make a claim."#
}

fn build_synthesize_user_prompt(
    question: &str,
    before_pairs: &[(app_lib::db::queries::Message, Vec<app_lib::db::queries::Message>, std::collections::HashMap<String, String>)],
    after_pairs: &[(app_lib::db::queries::Message, Vec<app_lib::db::queries::Message>, std::collections::HashMap<String, String>)],
) -> String {
    let mut out = String::new();
    out.push_str("QUESTION:\n");
    out.push_str(question.trim());
    out.push_str("\n\nCORPUS:\n");

    let render_window = |out: &mut String, name: &str, pairs: &[(app_lib::db::queries::Message, Vec<app_lib::db::queries::Message>, std::collections::HashMap<String, String>)]| {
        if pairs.is_empty() {
            out.push_str(&format!("\n─── {} window: (empty) ───\n", name));
            return;
        }
        out.push_str(&format!("\n─── {} window ({} msgs) ───\n", name, pairs.len()));
        for (i, (m, context, settings)) in pairs.iter().enumerate() {
            out.push_str(&format!("\n[{} #{}]  {}", name, i + 1, m.created_at));
            if !settings.is_empty() {
                let mut keys: Vec<&String> = settings.keys().collect();
                keys.sort();
                let parts: Vec<String> = keys.iter()
                    .filter(|k| ["response_length", "leader", "narration_tone", "send_history"].contains(&k.as_str()))
                    .filter_map(|k| settings.get(k.as_str()).map(|v| format!("{}={}", k, v)))
                    .collect();
                if !parts.is_empty() {
                    out.push_str(&format!("  [settings: {}]", parts.join(", ")));
                }
            }
            out.push('\n');
            if context.is_empty() {
                out.push_str("  SCENE: (no preceding context)\n");
            } else {
                out.push_str("  SCENE (preceding turns):\n");
                for cm in context {
                    let label = match cm.role.as_str() {
                        "user" => "User",
                        "assistant" => "Character",
                        "narrative" => "[Narrative]",
                        other => other,
                    };
                    out.push_str(&format!("    {}: {}\n", label, cm.content.trim()));
                }
            }
            out.push_str("  REPLY (this is the character's turn):\n");
            for line in m.content.trim().lines() {
                out.push_str(&format!("    {}\n", line));
            }
        }
    };

    render_window(&mut out, "BEFORE", before_pairs);
    render_window(&mut out, "AFTER", after_pairs);
    out
}

async fn cmd_synthesize(
    r: &Resolved,
    api_key: &str,
    git_ref: &str,
    end_ref: Option<&str>,
    limit: i64,
    character_id: Option<&str>,
    group_chat_id: Option<&str>,
    question: Option<&str>,
    question_file: Option<&std::path::Path>,
    role: &str,
    context_turns: i64,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
    repo: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // ─── Resolve question source — one of two mutually exclusive paths ──
    let sources_given = [question.is_some(), question_file.is_some()]
        .iter().filter(|b| **b).count();
    if sources_given > 1 {
        return Err(Box::<dyn std::error::Error>::from(
            "pass exactly one of --question or --question-file".to_string()));
    }
    if sources_given == 0 {
        return Err(Box::<dyn std::error::Error>::from(
            "one of --question or --question-file is required".to_string()));
    }
    let question_text: String = if let Some(p) = question_file {
        std::fs::read_to_string(p)
            .map_err(|e| format!("failed to read --question-file {}: {}", p.display(), e))?
    } else {
        question.unwrap().to_string()
    };
    if question_text.trim().is_empty() {
        return Err(Box::<dyn std::error::Error>::from("question is empty".to_string()));
    }

    // ─── Resolve scope (solo character vs. group chat) ───────────────
    if character_id.is_some() && group_chat_id.is_some() {
        return Err(Box::<dyn std::error::Error>::from(
            "pass either --character or --group-chat, not both".to_string()));
    }
    if character_id.is_none() && group_chat_id.is_none() {
        return Err(Box::<dyn std::error::Error>::from(
            "one of --character or --group-chat is required".to_string()));
    }
    if let Some(cid) = character_id { let _ = r.check_character(cid)?; }

    let (before_sha, before_ts, before_subject) = git_resolve_ref(repo, git_ref)?;
    let (after_sha, after_ts, after_subject) = match end_ref {
        Some(er) => git_resolve_ref(repo, er)?,
        None => (before_sha.clone(), before_ts.clone(), before_subject.clone()),
    };

    // ─── Pull windows + model config + display label ─────────────────
    let (model_config, before_pairs, after_pairs, display_label) = {
        let conn = r.db.conn.lock().unwrap();
        let mut mc = orchestrator::load_model_config(&conn);
        // Synthesis defaults to dialogue_model (the user's more
        // capable model) — qualitative prose benefits from the extra
        // capability that memory_model typically doesn't have.
        if let Some(m) = model_override { mc.dialogue_model = m.to_string(); }

        let (scope, display) = if let Some(cid) = character_id {
            let thread = get_thread_for_character(&conn, cid)?;
            let ch = get_character(&conn, cid)?;
            (
                EvalScope::Character {
                    character_id: cid.to_string(),
                    solo_thread_id: thread.thread_id,
                },
                format!("{} (solo + groups)", ch.display_name),
            )
        } else {
            let gcid = group_chat_id.unwrap();
            let gc = get_group_chat(&conn, gcid)
                .map_err(|e| Box::<dyn std::error::Error>::from(
                    format!("group_chat {}: {}", gcid, e)))?;
            r.check_world(&gc.world_id)?;
            (
                EvalScope::Group { thread_id: gc.thread_id },
                format!("{} (group)", gc.display_name),
            )
        };

        let before_raw = pull_eval_window(&conn, &scope, &before_ts, "before", role, limit, context_turns)?;
        let after_raw  = pull_eval_window(&conn, &scope, &after_ts,  "after",  role, limit, context_turns)?;
        let enrich = |triples: Vec<EvalTriple>| -> Vec<(app_lib::db::queries::Message, Vec<app_lib::db::queries::Message>, std::collections::HashMap<String, String>)> {
            triples.into_iter().map(|(m, context, is_group)| {
                let settings = active_settings_at(&conn, &m.thread_id, &m.created_at, is_group);
                (m, context, settings)
            }).collect()
        };
        (mc, enrich(before_raw), enrich(after_raw), display)
    };
    let character_name = display_label;

    let total_msgs = before_pairs.len() + after_pairs.len();
    if total_msgs == 0 {
        return Err(Box::<dyn std::error::Error>::from(
            "no messages in either window; widen --limit or pick a different ref".to_string()));
    }

    // ─── Build the single-call prompt ─────────────────────────────────
    let user_prompt = build_synthesize_user_prompt(&question_text, &before_pairs, &after_pairs);
    let system_prompt = synthesizer_system_prompt();

    // ─── Cost projection for one call ─────────────────────────────────
    let in_tokens = estimate_tokens(&user_prompt) + estimate_tokens(system_prompt);
    let out_budget: i64 = 2000; // Headroom for a substantive synthesis.
    let projected_usd = project_cost(&model_config.dialogue_model, in_tokens, out_budget, &r.cfg.model_pricing);

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
        eprintln!("[worldcli] synthesizing {} msgs ({} before / {} after) via {} — projected≈${:.4} ({} in tok est); 24h spent=${:.4}/${:.2}",
            total_msgs, before_pairs.len(), after_pairs.len(), model_config.dialogue_model,
            projected_usd, in_tokens, daily_so_far, daily_cap);
        eprintln!("[worldcli] question: {}", question_text.lines().next().unwrap_or("").chars().take(120).collect::<String>());
    }

    // ─── Make the single synthesis call ───────────────────────────────
    let base_url = model_config.chat_api_base();
    let req = openai::ChatRequest {
        model: model_config.dialogue_model.clone(),
        messages: vec![
            openai::ChatMessage { role: "system".to_string(), content: system_prompt.to_string() },
            openai::ChatMessage { role: "user".to_string(), content: user_prompt.clone() },
        ],
        temperature: Some(0.4),
        max_completion_tokens: Some(out_budget as u32),
        response_format: None,
    };
    let resp = openai::chat_completion_with_base(&base_url, api_key, &req).await
        .map_err(|e| format!("synthesize call failed: {}", e))?;
    let synthesis_text = resp.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "synthesizer returned no choices".to_string())?;
    let usage = resp.usage.unwrap_or(openai::Usage {
        prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
    });
    let actual_in = usage.prompt_tokens as i64;
    let actual_out = usage.completion_tokens as i64;
    let actual_usd = actual_cost(&model_config.dialogue_model, actual_in, actual_out, &r.cfg.model_pricing);
    append_cost_log(&CostEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: model_config.dialogue_model.clone(),
        prompt_tokens: actual_in,
        completion_tokens: actual_out,
        usd: actual_usd,
    });

    // ─── Build envelope + persist run log ─────────────────────────────
    let summarize_window = |pairs: &[(app_lib::db::queries::Message, Vec<app_lib::db::queries::Message>, std::collections::HashMap<String, String>)]| -> Vec<JsonValue> {
        pairs.iter().map(|(m, _ctx, settings)| {
            let mut settings_sorted: Vec<(String, String)> = settings.iter()
                .map(|(k, v)| (k.clone(), v.clone())).collect();
            settings_sorted.sort();
            let settings_json: serde_json::Map<String, JsonValue> = settings_sorted.into_iter()
                .map(|(k, v)| (k, JsonValue::String(v))).collect();
            json!({
                "message_id": m.message_id,
                "created_at": m.created_at,
                "content_preview": m.content.chars().take(200).collect::<String>(),
                "active_settings": settings_json,
            })
        }).collect()
    };
    let before_summary = summarize_window(&before_pairs);
    let after_summary = summarize_window(&after_pairs);

    let run_id = uuid::Uuid::new_v4().to_string();
    let envelope = json!({
        "run_id": run_id,
        "run_timestamp": chrono::Utc::now().to_rfc3339(),
        "ref": git_ref,
        "ref_resolved": before_sha,
        "ref_timestamp": before_ts,
        "ref_subject": before_subject,
        "end_ref": end_ref,
        "end_ref_resolved": end_ref.map(|_| after_sha.clone()),
        "end_ref_timestamp": end_ref.map(|_| after_ts.clone()),
        "end_ref_subject": end_ref.map(|_| after_subject.clone()),
        "character_id": character_id,
        "group_chat_id": group_chat_id,
        "scope_label": character_name,
        "role_filter": role,
        "context_turns": context_turns,
        "question": question_text,
        "synthesis": synthesis_text,
        "model": model_config.dialogue_model,
        "cost": {
            "prompt_tokens": actual_in,
            "completion_tokens": actual_out,
            "actual_usd": actual_usd,
        },
        "before": {
            "count": before_summary.len(),
            "messages": before_summary,
        },
        "after": {
            "count": after_summary.len(),
            "messages": after_summary,
        },
    });

    write_synthesize_run(&run_id, &envelope);

    if r.json {
        emit(true, envelope);
    } else {
        println!("=== SYNTHESIS ===");
        println!("ref:       {} ({})", git_ref, &before_sha[..8.min(before_sha.len())]);
        println!("subject:   {}", before_subject);
        let scope_id = character_id.or(group_chat_id).unwrap_or("?");
        println!("scope:     {} ({})", character_name, scope_id);
        println!("corpus:    {} msgs ({} before / {} after)", total_msgs, before_pairs.len(), after_pairs.len());
        println!("model:     {}", model_config.dialogue_model);
        println!("run_id:    {}", run_id);
        println!();
        println!("QUESTION:");
        println!("{}", question_text.trim());
        println!();
        println!("SYNTHESIS:");
        println!("{}", synthesis_text);
        println!();
        eprintln!("[worldcli] actual cost ${:.4} ({} in / {} out tok)",
            actual_usd, actual_in, actual_out);
    }
    Ok(())
}

// ─── Replay (cross-commit A/B via prompt override, not worktree) ────────

/// Shell out to `git show <ref>:<rel_path>` and return the file content
/// at that historical commit, without touching the working tree.
fn git_show_file(
    repo: Option<&std::path::Path>,
    git_ref: &str,
    rel_path: &str,
) -> Result<String, CliError> {
    let mut cmd = std::process::Command::new("git");
    if let Some(p) = repo {
        cmd.args(["-C", &p.display().to_string()]);
    }
    cmd.args(["show", &format!("{}:{}", git_ref, rel_path)]);
    let out = cmd.output().map_err(|e| {
        CliError::Other(format!("git invocation failed: {} (is git on PATH?)", e))
    })?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(CliError::Other(format!(
            "git show {}:{} failed: {}",
            git_ref, rel_path, err
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Extract the raw-string body of a `fn <name>() -> &'static str { r#"..."# }`
/// function from a source string. Returns None if the function isn't
/// present at this ref (rule hadn't been written yet, or was removed).
///
/// Only handles the common `r#"..."#` single-hash form — the form every
/// targeted craft-note function uses. If a future rule uses `r##"..."##`
/// or `r###"..."###` (needed when the body contains `"#`), extend this
/// parser; for now, returning None is the right behavior since the
/// replay will fall through to the current body.
fn extract_raw_str_fn_body(source: &str, fn_name: &str) -> Option<String> {
    let sig = format!("fn {}()", fn_name);
    let start = source.find(&sig)?;
    let after_sig = &source[start..];
    // Find the opening brace of the function body.
    let brace = after_sig.find('{')?;
    let body_section = &after_sig[brace + 1..];
    // Find the opening r#" of the raw-string literal.
    let open_marker = "r#\"";
    let open = body_section.find(open_marker)?;
    let body_start = open + open_marker.len();
    let rest = &body_section[body_start..];
    // Find the closing "#. For single-hash raw strings this is the
    // first occurrence of "# — the raw-string grammar guarantees the
    // body cannot contain it.
    let close_marker = "\"#";
    let close = rest.find(close_marker)?;
    Some(rest[..close].to_string())
}

/// Parse the historical prompts.rs source for ALL known overridable
/// dialogue-craft-note fragments. Missing functions (because the rule
/// wasn't in the stack at this ref) are silently skipped — the override
/// map won't have a key for them, so the CURRENT body flows through,
/// which is the honest default.
fn parse_historical_prompts_overrides(source: &str) -> app_lib::ai::prompts::PromptOverrides {
    let mut overrides = app_lib::ai::prompts::PromptOverrides::new();
    for &name in app_lib::ai::prompts::OVERRIDABLE_DIALOGUE_FRAGMENTS {
        if let Some(body) = extract_raw_str_fn_body(source, name) {
            overrides.insert(name, body);
        }
    }
    overrides
}

fn replay_runs_dir() -> PathBuf { worldcli_home().join("replay-runs") }
fn replay_runs_manifest() -> PathBuf { replay_runs_dir().join("manifest.jsonl") }

fn write_replay_run(run_id: &str, envelope: &JsonValue) {
    let dir = replay_runs_dir();
    let _ = std::fs::create_dir_all(&dir);
    let per_path = dir.join(format!("{}.json", run_id));
    if let Ok(s) = serde_json::to_string_pretty(envelope) {
        let _ = std::fs::write(&per_path, s);
    }
    let manifest_entry = json!({
        "run_id": envelope.get("run_id"),
        "run_timestamp": envelope.get("run_timestamp"),
        "character_id": envelope.get("character_id"),
        "character_name": envelope.get("character_name"),
        "prompt_preview": envelope.get("prompt").and_then(|v| v.as_str())
            .map(|s| s.chars().take(140).collect::<String>()),
        "refs": envelope.get("refs"),
        "model": envelope.get("model"),
        "cost_usd": envelope.get("cost").and_then(|c| c.get("actual_usd")),
    });
    let line = serde_json::to_string(&manifest_entry).unwrap_or_default();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true).open(replay_runs_manifest())
    {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

fn read_replay_runs_manifest() -> Vec<JsonValue> {
    let Ok(content) = std::fs::read_to_string(replay_runs_manifest()) else { return Vec::new(); };
    content.lines().filter_map(|l| serde_json::from_str(l).ok()).collect()
}

fn cmd_replay_runs(r: &Resolved, action: ReplayRunAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ReplayRunAction::List { limit } => {
            let mut entries = read_replay_runs_manifest();
            entries.reverse();
            entries.truncate(limit);
            if r.json {
                emit(true, JsonValue::Array(entries));
            } else {
                if entries.is_empty() {
                    println!("No replay runs recorded yet. Run `worldcli replay ...` first.");
                    return Ok(());
                }
                for e in &entries {
                    let ts = e.get("run_timestamp").and_then(|v| v.as_str()).unwrap_or("");
                    let ts_short = &ts[..19.min(ts.len())];
                    let id = e.get("run_id").and_then(|v| v.as_str()).unwrap_or("");
                    let id_short = &id[..8.min(id.len())];
                    let name = e.get("character_name").and_then(|v| v.as_str()).unwrap_or("?");
                    let refs = e.get("refs").and_then(|v| v.as_array())
                        .map(|a| a.len()).unwrap_or(0);
                    let prompt_preview = e.get("prompt_preview").and_then(|v| v.as_str()).unwrap_or("");
                    let cost = e.get("cost_usd").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    println!("{id_short}  [{ts_short}]  {name}  refs×{refs}  ${:.4}  — {}",
                        cost, prompt_preview);
                }
            }
        }
        ReplayRunAction::Show { id } => {
            let dir = replay_runs_dir();
            let exact = dir.join(format!("{}.json", id));
            if exact.exists() {
                let s = std::fs::read_to_string(&exact)?;
                let v: JsonValue = serde_json::from_str(&s).unwrap_or(JsonValue::String(s));
                emit(r.json, v);
                return Ok(());
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.starts_with(&id) && fname.ends_with(".json") {
                        let s = std::fs::read_to_string(entry.path())?;
                        let v: JsonValue = serde_json::from_str(&s).unwrap_or(JsonValue::String(s));
                        emit(r.json, v);
                        return Ok(());
                    }
                }
            }
            return Err(Box::new(CliError::NotFound(format!("replay run starting with '{}'", id))));
        }
        ReplayRunAction::Search { query } => {
            let q = query.to_lowercase();
            let entries = read_replay_runs_manifest();
            let hits: Vec<JsonValue> = entries.into_iter()
                .filter(|e| e.to_string().to_lowercase().contains(&q))
                .collect();
            emit(r.json, JsonValue::Array(hits));
        }
    }
    Ok(())
}

/// Display/serialize name for a CraftNotePiece, used by header
/// printout and the persisted run log envelope.
fn craft_note_piece_name(p: &app_lib::ai::prompts::CraftNotePiece) -> &'static str {
    use app_lib::ai::prompts::CraftNotePiece as CN;
    match p {
        CN::EarnedRegister => "earned_register",
        CN::CraftNotes => "craft_notes",
        CN::HiddenCommonality => "hidden_commonality",
        CN::DriveTheMoment => "drive_the_moment",
        CN::VerdictWithoutOverExplanation => "verdict_without_over_explanation",
        CN::ReflexPolishVsEarnedClose => "reflex_polish_vs_earned_close",
        CN::KeepTheSceneBreathing => "keep_the_scene_breathing",
        CN::GentleRelease => "gentle_release",
        CN::NameTheGladThingPlain => "name_the_glad_thing_plain",
        CN::PlainAfterCrooked => "plain_after_crooked",
        CN::WitAsDimmer => "wit_as_dimmer",
        CN::LetTheRealThingIn => "let_the_real_thing_in",
        CN::HumorLandsPlain => "humor_lands_plain",
        CN::HandsAsCoolant => "hands_as_coolant",
        CN::NoticingAsMirror => "noticing_as_mirror",
        CN::UnguardedEntry => "unguarded_entry",
        CN::ProtagonistFraming => "protagonist_framing",
    }
}

/// Display/serialize name for an InvariantPiece.
fn invariant_piece_name(p: &app_lib::ai::prompts::InvariantPiece) -> &'static str {
    use app_lib::ai::prompts::InvariantPiece as IP;
    match p {
        IP::Reverence => "reverence",
        IP::Daylight => "daylight",
        IP::Agape => "agape",
        IP::FruitsOfTheSpirit => "fruits_of_the_spirit",
        IP::Soundness => "soundness",
        IP::Nourishment => "nourishment",
        IP::TellTheTruth => "tell_the_truth",
    }
}

/// Human-readable form of an InsertionAnchor for header + envelope.
fn insertion_anchor_name(anchor: &app_lib::ai::prompts::InsertionAnchor) -> String {
    use app_lib::ai::prompts::InsertionAnchor as IA;
    use app_lib::ai::prompts::DialoguePromptSection as DPS;
    match anchor {
        IA::CraftNote(p) => craft_note_piece_name(p).to_string(),
        IA::Invariant(p) => invariant_piece_name(p).to_string(),
        IA::SectionStart(s) => format!("section-start:{}", match s {
            DPS::AgencyAndBehavior => "agency-and-behavior",
            DPS::CraftNotes => "craft-notes",
            DPS::Invariants => "invariants",
        }),
        IA::SectionEnd(s) => format!("section-end:{}", match s {
            DPS::AgencyAndBehavior => "agency-and-behavior",
            DPS::CraftNotes => "craft-notes",
            DPS::Invariants => "invariants",
        }),
    }
}

async fn cmd_replay(
    r: &Resolved,
    api_key: &str,
    refs: &[String],
    character_id: &str,
    prompt: &str,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
    n_samples: u32,
    repo: Option<&std::path::Path>,
    section_order_names: &[String],
    craft_notes_order_names: &[String],
    invariants_order_names: &[String],
    omit_craft_notes_names: &[String],
    omit_invariants_names: &[String],
    insert_file_path: Option<&std::path::Path>,
    insert_before_anchor: Option<&str>,
    insert_after_anchor: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if refs.is_empty() {
        return Err(Box::<dyn std::error::Error>::from(
            "at least one ref is required (use --refs HEAD,<sha>,...)".to_string()));
    }
    if prompt.trim().is_empty() {
        return Err(Box::<dyn std::error::Error>::from("prompt is empty".to_string()));
    }
    if n_samples < 1 {
        return Err(Box::<dyn std::error::Error>::from("--n must be at least 1".to_string()));
    }
    let _ = r.check_character(character_id)?;

    // Parse optional --section-order flag into a validated
    // Vec<DialoguePromptSection>. Empty input = no override (default
    // order applies). Invalid names or a non-permutation are hard
    // errors — fail fast rather than silently fall back to default.
    let section_order_override: Option<Vec<app_lib::ai::prompts::DialoguePromptSection>> =
        if section_order_names.is_empty() {
            None
        } else {
            let mut parsed: Vec<app_lib::ai::prompts::DialoguePromptSection> = Vec::new();
            for name in section_order_names {
                match app_lib::ai::prompts::DialoguePromptSection::from_cli_name(name) {
                    Some(sec) => parsed.push(sec),
                    None => return Err(Box::<dyn std::error::Error>::from(format!(
                        "unknown section name '{}' in --section-order. Valid names: agency-and-behavior, craft-notes, invariants.",
                        name
                    ))),
                }
            }
            if !app_lib::ai::prompts::DialoguePromptSection::is_valid_permutation(&parsed) {
                return Err(Box::<dyn std::error::Error>::from(format!(
                    "--section-order must include exactly one of each: agency-and-behavior, craft-notes, invariants. Got: {:?}",
                    parsed
                )));
            }
            Some(parsed)
        };

    // Parse --craft-notes-order (partial orderings OK — prefix then
    // defaults). Only invalid names are hard errors.
    let craft_notes_order_override: Option<Vec<app_lib::ai::prompts::CraftNotePiece>> =
        if craft_notes_order_names.is_empty() {
            None
        } else {
            let mut parsed: Vec<app_lib::ai::prompts::CraftNotePiece> = Vec::new();
            for name in craft_notes_order_names {
                match app_lib::ai::prompts::CraftNotePiece::from_cli_name(name) {
                    Some(p) => parsed.push(p),
                    None => return Err(Box::<dyn std::error::Error>::from(format!(
                        "unknown craft-note name '{}' in --craft-notes-order. See --help for the full list of valid names.",
                        name
                    ))),
                }
            }
            Some(parsed)
        };

    // Parse --invariants-order (same prefix-then-defaults semantics).
    let invariants_order_override: Option<Vec<app_lib::ai::prompts::InvariantPiece>> =
        if invariants_order_names.is_empty() {
            None
        } else {
            let mut parsed: Vec<app_lib::ai::prompts::InvariantPiece> = Vec::new();
            for name in invariants_order_names {
                match app_lib::ai::prompts::InvariantPiece::from_cli_name(name) {
                    Some(p) => parsed.push(p),
                    None => return Err(Box::<dyn std::error::Error>::from(format!(
                        "unknown invariant name '{}' in --invariants-order. Valid names: reverence, daylight, agape, fruits_of_the_spirit (or fruits), soundness, nourishment, tell_the_truth (or truth).",
                        name
                    ))),
                }
            }
            Some(parsed)
        };

    // Parse --omit-craft-notes.
    let omit_craft_notes: Vec<app_lib::ai::prompts::CraftNotePiece> = {
        let mut parsed: Vec<app_lib::ai::prompts::CraftNotePiece> = Vec::new();
        for name in omit_craft_notes_names {
            match app_lib::ai::prompts::CraftNotePiece::from_cli_name(name) {
                Some(p) => parsed.push(p),
                None => return Err(Box::<dyn std::error::Error>::from(format!(
                    "unknown craft-note name '{}' in --omit-craft-notes. See --help for valid names.",
                    name
                ))),
            }
        }
        parsed
    };

    // Parse --omit-invariants.
    let omit_invariants: Vec<app_lib::ai::prompts::InvariantPiece> = {
        let mut parsed: Vec<app_lib::ai::prompts::InvariantPiece> = Vec::new();
        for name in omit_invariants_names {
            match app_lib::ai::prompts::InvariantPiece::from_cli_name(name) {
                Some(p) => parsed.push(p),
                None => return Err(Box::<dyn std::error::Error>::from(format!(
                    "unknown invariant name '{}' in --omit-invariants. See --help for valid names.",
                    name
                ))),
            }
        }
        parsed
    };

    // Parse --insert-file + --insert-before / --insert-after.
    // All-or-nothing: either --insert-file plus exactly one of the
    // anchor flags, or none of the three. Anything in between is an
    // error.
    let insertion: Option<app_lib::ai::prompts::Insertion> = match (
        insert_file_path,
        insert_before_anchor,
        insert_after_anchor,
    ) {
        (None, None, None) => None,
        (Some(path), before, after) => {
            let (anchor_str, position) = match (before, after) {
                (Some(a), None) => (a, app_lib::ai::prompts::InsertPosition::Before),
                (None, Some(a)) => (a, app_lib::ai::prompts::InsertPosition::After),
                (Some(_), Some(_)) => {
                    return Err(Box::<dyn std::error::Error>::from(
                        "--insert-before and --insert-after are mutually exclusive".to_string()));
                }
                (None, None) => {
                    return Err(Box::<dyn std::error::Error>::from(
                        "--insert-file requires exactly one of --insert-before or --insert-after".to_string()));
                }
            };
            let anchor = app_lib::ai::prompts::InsertionAnchor::from_cli_name(anchor_str)
                .ok_or_else(|| Box::<dyn std::error::Error>::from(format!(
                    "unknown insertion anchor '{}'. Valid forms: piece name (e.g., 'earned_register', 'reverence') or 'section-start:<section>' / 'section-end:<section>' where section is one of craft-notes, invariants, agency-and-behavior.",
                    anchor_str
                )))?;
            let text = std::fs::read_to_string(path).map_err(|e| Box::<dyn std::error::Error>::from(format!(
                "reading --insert-file {}: {}", path.display(), e
            )))?;
            Some(app_lib::ai::prompts::Insertion { anchor, position, text })
        }
        (None, Some(_), _) | (None, _, Some(_)) => {
            return Err(Box::<dyn std::error::Error>::from(
                "--insert-before / --insert-after requires --insert-file".to_string()));
        }
    };

    // Resolve each ref to (sha, timestamp, subject) up front so failures
    // happen before any LLM spend.
    let mut resolved_refs: Vec<(String, String, String, String)> = Vec::new();
    for rr in refs {
        let (sha, ts, subj) = git_resolve_ref(repo, rr)?;
        resolved_refs.push((rr.clone(), sha, ts, subj));
    }

    // Fetch + parse the historical prompts.rs for each ref. Apply the
    // section-order override (if any) identically to every ref — the
    // override is the held-constant condition; ref-varying content
    // bodies is the A/B variable.
    let mut per_ref_overrides: Vec<(String, app_lib::ai::prompts::PromptOverrides, Vec<String>)> = Vec::new();
    for (ref_input, sha, _ts, _subj) in &resolved_refs {
        let source = git_show_file(repo, sha, "src-tauri/src/ai/prompts.rs")
            .map_err(|e| Box::<dyn std::error::Error>::from(
                format!("fetching prompts.rs at {}: {}", ref_input, e)))?;
        let mut overrides = parse_historical_prompts_overrides(&source);
        if let Some(order) = &section_order_override {
            overrides.set_section_order(order.clone());
        }
        if let Some(order) = &craft_notes_order_override {
            overrides.set_craft_notes_order(order.clone());
        }
        if let Some(order) = &invariants_order_override {
            overrides.set_invariants_order(order.clone());
        }
        if !omit_craft_notes.is_empty() {
            overrides.set_omit_craft_notes(omit_craft_notes.clone());
        }
        if !omit_invariants.is_empty() {
            overrides.set_omit_invariants(omit_invariants.clone());
        }
        if let Some(ins) = &insertion {
            overrides.set_insertion(ins.clone());
        }
        let found: Vec<String> = overrides.map.keys().cloned().collect();
        per_ref_overrides.push((ref_input.clone(), overrides, found));
    }

    // Load character + world context ONCE — this is the held-constant
    // side of the A/B. Only the overrides vary per ref.
    let (world, character, user_profile, recent_journals, active_quests, stance_text, anchor_text, mut model_config) = {
        let conn = r.db.conn.lock().unwrap();
        let character = get_character(&conn, character_id)?;
        let world = get_world(&conn, &character.world_id)?;
        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let recent_journals = list_journal_entries(&conn, character_id, 1).unwrap_or_default();
        let active_quests = list_active_quests(&conn, &character.world_id).unwrap_or_default();
        let latest_stance = latest_relational_stance(&conn, character_id).unwrap_or(None);
        let stance_text: Option<String> = latest_stance.as_ref().map(|s| s.stance_text.clone());
        let anchor_text: Option<String> = combined_axes_block(&conn, character_id);
        let model_config = orchestrator::load_model_config(&conn);
        (world, character, user_profile, recent_journals, active_quests, stance_text, anchor_text, model_config)
    };
    if let Some(m) = model_override { model_config.dialogue_model = m.to_string(); }

    // Project cost per ref (each ref is one dialogue-model call against
    // the assembled system prompt). Conservative: use first ref's
    // assembled prompt to estimate — they'll be close in size.
    let sample_system = app_lib::ai::prompts::build_dialogue_system_prompt_with_overrides(
        &world, &character, user_profile.as_ref(),
        None, Some("Auto"), None, None, false, &[], None,
        &recent_journals, None, &[], None, &active_quests,
        stance_text.as_deref(),
        anchor_text.as_deref(),
        Some(&per_ref_overrides[0].1),
    );
    let est_in = estimate_tokens(&sample_system) + estimate_tokens(prompt);
    let est_out: i64 = 600;
    let per_ref_usd = project_cost(&model_config.dialogue_model, est_in, est_out, &r.cfg.model_pricing);
    let per_sample_usd = per_ref_usd; // one call per (ref, sample)
    let total_projected = per_sample_usd * (refs.len() as f64) * (n_samples as f64);

    let daily_so_far = rolling_24h_total_usd();
    let daily_after = daily_so_far + total_projected;
    let per_call_cap = r.cfg.budget.per_call_usd;
    let daily_cap = r.cfg.budget.daily_usd;
    let confirm = confirm_cost.unwrap_or(0.0);
    if total_projected > per_call_cap && confirm < total_projected {
        return Err(Box::new(CliError::Budget {
            kind: "per_call (total replay)".to_string(),
            projected_usd: total_projected,
            cap_usd: per_call_cap,
            confirm_at_least: (total_projected * 1.05).max(0.01),
        }));
    }
    if daily_after > daily_cap && confirm < total_projected {
        return Err(Box::new(CliError::Budget {
            kind: "daily".to_string(),
            projected_usd: daily_after,
            cap_usd: daily_cap,
            confirm_at_least: (total_projected * 1.05).max(0.01),
        }));
    }

    if !r.json {
        if n_samples > 1 {
            eprintln!("[worldcli] replay {} refs × {} samples against {} via {} — per-sample≈${:.4}, total≈${:.4}; 24h spent=${:.4}/${:.2}",
                refs.len(), n_samples, character.display_name, model_config.dialogue_model,
                per_sample_usd, total_projected, daily_so_far, daily_cap);
        } else {
            eprintln!("[worldcli] replay {} refs against {} via {} — per-ref≈${:.4}, total≈${:.4}; 24h spent=${:.4}/${:.2}",
                refs.len(), character.display_name, model_config.dialogue_model,
                per_ref_usd, total_projected, daily_so_far, daily_cap);
        }
    }

    // Run each ref sequentially — same prompt, different override set.
    let base_url = model_config.chat_api_base();
    let mut per_ref_results: Vec<JsonValue> = Vec::new();
    let mut total_in: i64 = 0;
    let mut total_out: i64 = 0;
    let total_calls = refs.len() * (n_samples as usize);
    let mut call_idx: usize = 0;
    for (i, (ref_input, overrides, found_keys)) in per_ref_overrides.iter().enumerate() {
        let (_input, sha, ts, subj) = &resolved_refs[i];
        let system_prompt = app_lib::ai::prompts::build_dialogue_system_prompt_with_overrides(
            &world, &character, user_profile.as_ref(),
            None, Some("Auto"), None, None, false, &[], None,
            &recent_journals, None, &[], None, &active_quests,
            stance_text.as_deref(),
            anchor_text.as_deref(),
            Some(overrides),
        );
        for sample_index in 0..n_samples {
            call_idx += 1;
            let messages = vec![
                openai::ChatMessage { role: "system".to_string(), content: system_prompt.clone() },
                openai::ChatMessage { role: "user".to_string(), content: prompt.to_string() },
            ];
            let req = openai::ChatRequest {
                model: model_config.dialogue_model.clone(),
                messages,
                temperature: Some(0.95),
                max_completion_tokens: None,
                response_format: None,
            };
            if n_samples > 1 {
                eprint!("\r[worldcli] replaying {}/{} — ref {} sample {}/{}   ",
                    call_idx, total_calls, &sha[..8.min(sha.len())], sample_index + 1, n_samples);
            } else {
                eprint!("\r[worldcli] replaying {}/{} — ref {}", call_idx, total_calls, &sha[..8.min(sha.len())]);
            }
            let resp = openai::chat_completion_with_base(&base_url, api_key, &req).await
                .map_err(|e| format!("replay call for ref {} sample {} failed: {}", ref_input, sample_index, e))?;
            let reply = resp.choices.first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| "no choices returned".to_string())?;
            let usage = resp.usage.unwrap_or(openai::Usage {
                prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
            });
            total_in += usage.prompt_tokens as i64;
            total_out += usage.completion_tokens as i64;
            per_ref_results.push(json!({
                "ref": ref_input,
                "ref_resolved": sha,
                "ref_timestamp": ts,
                "ref_subject": subj,
                "sample_index": sample_index,
                "sample_count": n_samples,
                "overrides_applied": found_keys,
                "reply": reply,
                "usage": {
                    "prompt_tokens": usage.prompt_tokens,
                    "completion_tokens": usage.completion_tokens,
                },
            }));
        }
    }
    eprintln!();

    let actual_usd = actual_cost(&model_config.dialogue_model, total_in, total_out, &r.cfg.model_pricing);
    append_cost_log(&CostEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: model_config.dialogue_model.clone(),
        prompt_tokens: total_in,
        completion_tokens: total_out,
        usd: actual_usd,
    });

    let run_id = uuid::Uuid::new_v4().to_string();
    let section_order_json: serde_json::Value = match &section_order_override {
        Some(order) => json!(order.iter().map(|s| match s {
            app_lib::ai::prompts::DialoguePromptSection::AgencyAndBehavior => "agency-and-behavior",
            app_lib::ai::prompts::DialoguePromptSection::CraftNotes => "craft-notes",
            app_lib::ai::prompts::DialoguePromptSection::Invariants => "invariants",
        }).collect::<Vec<_>>()),
        None => serde_json::Value::Null,
    };
    let craft_notes_order_json: serde_json::Value = match &craft_notes_order_override {
        Some(order) => json!(order.iter().map(|p| craft_note_piece_name(p)).collect::<Vec<_>>()),
        None => serde_json::Value::Null,
    };
    let invariants_order_json: serde_json::Value = match &invariants_order_override {
        Some(order) => json!(order.iter().map(|p| invariant_piece_name(p)).collect::<Vec<_>>()),
        None => serde_json::Value::Null,
    };
    let omit_craft_notes_json: serde_json::Value = if omit_craft_notes.is_empty() {
        serde_json::Value::Null
    } else {
        json!(omit_craft_notes.iter().map(|p| craft_note_piece_name(p)).collect::<Vec<_>>())
    };
    let omit_invariants_json: serde_json::Value = if omit_invariants.is_empty() {
        serde_json::Value::Null
    } else {
        json!(omit_invariants.iter().map(|p| invariant_piece_name(p)).collect::<Vec<_>>())
    };
    let insertion_json: serde_json::Value = match &insertion {
        Some(ins) => json!({
            "anchor": insertion_anchor_name(&ins.anchor),
            "position": match ins.position {
                app_lib::ai::prompts::InsertPosition::Before => "before",
                app_lib::ai::prompts::InsertPosition::After => "after",
            },
            "text": ins.text,
        }),
        None => serde_json::Value::Null,
    };
    let envelope = json!({
        "run_id": run_id,
        "run_timestamp": chrono::Utc::now().to_rfc3339(),
        "character_id": character_id,
        "character_name": character.display_name,
        "prompt": prompt,
        "model": model_config.dialogue_model,
        "n_samples": n_samples,
        "section_order_override": section_order_json,
        "craft_notes_order_override": craft_notes_order_json,
        "invariants_order_override": invariants_order_json,
        "omit_craft_notes": omit_craft_notes_json,
        "omit_invariants": omit_invariants_json,
        "insertion": insertion_json,
        "refs": resolved_refs.iter().map(|(i, s, t, sub)| json!({
            "ref": i, "sha": s, "timestamp": t, "subject": sub,
        })).collect::<Vec<_>>(),
        "results": per_ref_results,
        "cost": {
            "prompt_tokens": total_in,
            "completion_tokens": total_out,
            "actual_usd": actual_usd,
        },
    });
    write_replay_run(&run_id, &envelope);

    if r.json {
        emit(true, envelope);
    } else {
        println!("=== REPLAY ===");
        println!("character: {} ({})", character.display_name, character_id);
        println!("model:     {}", model_config.dialogue_model);
        println!("prompt:    {}", prompt);
        println!("run_id:    {}", run_id);
        if let Some(order) = &section_order_override {
            let names: Vec<&str> = order.iter().map(|s| match s {
                app_lib::ai::prompts::DialoguePromptSection::AgencyAndBehavior => "agency-and-behavior",
                app_lib::ai::prompts::DialoguePromptSection::CraftNotes => "craft-notes",
                app_lib::ai::prompts::DialoguePromptSection::Invariants => "invariants",
            }).collect();
            println!("section-order: {} (non-default)", names.join(","));
        }
        if let Some(order) = &craft_notes_order_override {
            let names: Vec<String> = order.iter().map(|p| craft_note_piece_name(p).to_string()).collect();
            println!("craft-notes-order: {} (prefix; rest default)", names.join(","));
        }
        if let Some(order) = &invariants_order_override {
            let names: Vec<String> = order.iter().map(|p| invariant_piece_name(p).to_string()).collect();
            println!("invariants-order: {} (prefix; rest default)", names.join(","));
        }
        if !omit_craft_notes.is_empty() {
            let names: Vec<&str> = omit_craft_notes.iter().map(craft_note_piece_name).collect();
            println!("omit-craft-notes: {}", names.join(","));
        }
        if !omit_invariants.is_empty() {
            let names: Vec<&str> = omit_invariants.iter().map(invariant_piece_name).collect();
            println!("omit-invariants: {}", names.join(","));
        }
        if let Some(ins) = &insertion {
            let pos = match ins.position {
                app_lib::ai::prompts::InsertPosition::Before => "before",
                app_lib::ai::prompts::InsertPosition::After => "after",
            };
            println!("insertion: {} {} ({} bytes)", pos, insertion_anchor_name(&ins.anchor), ins.text.len());
        }
        println!();
        for result in envelope["results"].as_array().unwrap_or(&vec![]) {
            let r_input = result["ref"].as_str().unwrap_or("?");
            let sha = result["ref_resolved"].as_str().unwrap_or("");
            let sha_short = &sha[..8.min(sha.len())];
            let subj = result["ref_subject"].as_str().unwrap_or("");
            let reply = result["reply"].as_str().unwrap_or("");
            let overrides_count = result["overrides_applied"].as_array().map(|a| a.len()).unwrap_or(0);
            let sample_count = result["sample_count"].as_u64().unwrap_or(1);
            if sample_count > 1 {
                let sample_idx = result["sample_index"].as_u64().unwrap_or(0);
                println!("─── ref: {} ({}) — sample {}/{} — {} craft-note override(s) applied ───",
                    r_input, sha_short, sample_idx + 1, sample_count, overrides_count);
            } else {
                println!("─── ref: {} ({}) — {} craft-note override(s) applied ───", r_input, sha_short, overrides_count);
            }
            println!("    commit: {}", subj);
            println!();
            println!("{}", reply);
            println!();
        }
        eprintln!("[worldcli] actual cost ${:.4} ({} in / {} out tok)",
            actual_usd, total_in, total_out);
    }
    Ok(())
}

// ─── Experiment registry (experiments/*.md) ─────────────────────────────
//
// Structured hypothesis files. Each file is markdown-with-YAML-ish-
// frontmatter. Schema is intentionally flat (no nested YAML) so the
// hand-parser stays simple:
//
//   Scalars: id, status, mode, ref, rubric_ref, created_at, resolved_at,
//            report (a single path; use `reports` list for multiples)
//   Block scalars (prose, `|`-prefixed): hypothesis, prediction, summary
//   Flat string-lists: scope_characters, scope_group_chats, run_ids,
//                      follow_ups, reports
//
// The markdown body (after the closing `---`) holds free-form
// interpretation / notes and is preserved verbatim on update.

fn experiments_dir() -> PathBuf { PathBuf::from("experiments") }

#[derive(Debug, Clone, Default)]
struct ExperimentFile {
    slug: String,
    path: PathBuf,
    // Scalar fields
    id: String,
    status: String,
    mode: String,
    git_ref: String,
    rubric_ref: String,
    created_at: String,
    resolved_at: String,
    // Block-scalar fields
    hypothesis: String,
    prediction: String,
    summary: String,
    // List fields
    scope_characters: Vec<String>,
    scope_group_chats: Vec<String>,
    run_ids: Vec<String>,
    follow_ups: Vec<String>,
    reports: Vec<String>,
    // The markdown body that follows the frontmatter, verbatim.
    body: String,
    // The raw file text — useful for `lab show` and preserving fields
    // the parser doesn't know about.
    raw: String,
}

/// Split raw file text into (frontmatter, body). Returns None if the
/// file doesn't start with a `---` fence.
fn split_frontmatter(raw: &str) -> Option<(String, String)> {
    let mut lines = raw.lines();
    let first = lines.next()?;
    if first.trim() != "---" { return None; }
    let mut fm: Vec<&str> = Vec::new();
    let mut body_start = None;
    let mut idx = first.len() + 1; // past the opening fence
    for line in lines {
        idx += line.len() + 1;
        if line.trim() == "---" {
            body_start = Some(idx);
            break;
        }
        fm.push(line);
    }
    let body = if let Some(bs) = body_start {
        raw.get(bs.min(raw.len())..).unwrap_or("").to_string()
    } else {
        return None;
    };
    Some((fm.join("\n"), body))
}

/// Parse flat YAML-ish frontmatter. Handles:
///   key: scalar
///   key: |          (block scalar; indented continuation until dedent)
///   key:            (list; next `- ` prefixed indented lines are items)
fn parse_experiment_frontmatter(fm_text: &str) -> ExperimentFile {
    let mut out = ExperimentFile::default();
    let lines: Vec<&str> = fm_text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_end();
        if trimmed.trim_start().is_empty() {
            i += 1;
            continue;
        }
        // Only top-level keys (no leading whitespace) start a new field.
        if line.starts_with(' ') || line.starts_with('\t') {
            i += 1;
            continue;
        }
        // Extract key:value or key: (block / list).
        let (key, rest) = match line.find(':') {
            Some(ci) => (line[..ci].trim().to_string(), line[ci + 1..].to_string()),
            None => { i += 1; continue; }
        };
        let rest_t = rest.trim();

        if rest_t == "|" {
            // Block scalar: collect indented continuation until dedent.
            let mut buf: Vec<String> = Vec::new();
            i += 1;
            while i < lines.len() {
                let l = lines[i];
                if l.is_empty() { buf.push(String::new()); i += 1; continue; }
                if !(l.starts_with(' ') || l.starts_with('\t')) { break; }
                // Strip a two-space / tab indent.
                let stripped = l.strip_prefix("  ").or_else(|| l.strip_prefix('\t')).unwrap_or(l);
                buf.push(stripped.to_string());
                i += 1;
            }
            // Trim trailing blank lines.
            while buf.last().map(|s| s.is_empty()).unwrap_or(false) { buf.pop(); }
            let value = buf.join("\n");
            assign_scalar(&mut out, &key, value);
        } else if rest_t.is_empty() {
            // List: next lines starting with "- " are items.
            let mut items: Vec<String> = Vec::new();
            i += 1;
            while i < lines.len() {
                let l = lines[i];
                let lt = l.trim_start();
                if lt.starts_with("- ") {
                    let item = lt[2..].trim().trim_matches('"').to_string();
                    if !item.is_empty() { items.push(item); }
                    i += 1;
                } else if l.trim().is_empty() {
                    i += 1;
                } else {
                    break;
                }
            }
            assign_list(&mut out, &key, items);
        } else {
            // Inline scalar.
            let value = rest_t.trim_matches('"').to_string();
            assign_scalar(&mut out, &key, value);
            i += 1;
        }
    }
    out
}

fn assign_scalar(out: &mut ExperimentFile, key: &str, value: String) {
    match key {
        "id" => out.id = value,
        "status" => out.status = value,
        "mode" => out.mode = value,
        "ref" => out.git_ref = value,
        "rubric_ref" => out.rubric_ref = value,
        "created_at" => out.created_at = value,
        "resolved_at" => out.resolved_at = value,
        "hypothesis" => out.hypothesis = value,
        "prediction" => out.prediction = value,
        "summary" => out.summary = value,
        _ => {}
    }
}

fn assign_list(out: &mut ExperimentFile, key: &str, items: Vec<String>) {
    match key {
        "scope_characters" => out.scope_characters = items,
        "scope_group_chats" => out.scope_group_chats = items,
        "run_ids" => out.run_ids = items,
        "follow_ups" => out.follow_ups = items,
        "reports" => out.reports = items,
        _ => {}
    }
}

fn load_experiment(slug: &str) -> Result<ExperimentFile, String> {
    let path = experiments_dir().join(format!("{}.md", slug));
    if !path.exists() {
        return Err(format!("experiment '{}' not found at {}. Run `worldcli lab list` to see the registry.", slug, path.display()));
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    let (fm_text, body) = split_frontmatter(&raw)
        .ok_or_else(|| format!("experiment '{}' has no `---` frontmatter fence", slug))?;
    let mut exp = parse_experiment_frontmatter(&fm_text);
    exp.slug = slug.to_string();
    exp.path = path;
    exp.body = body;
    exp.raw = raw;
    if exp.id.is_empty() { exp.id = slug.to_string(); }
    Ok(exp)
}

fn list_experiments() -> Result<Vec<ExperimentFile>, String> {
    let dir = experiments_dir();
    if !dir.exists() { return Ok(Vec::new()); }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
        let fname = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if fname == "README" { continue; }
        if let Ok(exp) = load_experiment(&fname) {
            out.push(exp);
        }
    }
    out.sort_by(|a, b| a.created_at.cmp(&b.created_at).reverse()
        .then_with(|| a.slug.cmp(&b.slug)));
    Ok(out)
}

/// Serialize an ExperimentFile back to disk. Preserves the markdown
/// body verbatim. The frontmatter fields are written in a stable order
/// so diffs are clean.
fn write_experiment(exp: &ExperimentFile) -> Result<(), String> {
    let dir = experiments_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.md", exp.slug));

    let mut fm = String::new();
    fm.push_str("---\n");
    push_scalar(&mut fm, "id", &exp.id);
    push_scalar(&mut fm, "status", &exp.status);
    push_scalar(&mut fm, "mode", &exp.mode);
    push_scalar(&mut fm, "created_at", &exp.created_at);
    if !exp.resolved_at.is_empty() { push_scalar(&mut fm, "resolved_at", &exp.resolved_at); }
    if !exp.git_ref.is_empty() { push_scalar(&mut fm, "ref", &exp.git_ref); }
    if !exp.rubric_ref.is_empty() { push_scalar(&mut fm, "rubric_ref", &exp.rubric_ref); }
    fm.push('\n');
    push_block(&mut fm, "hypothesis", &exp.hypothesis);
    push_block(&mut fm, "prediction", &exp.prediction);
    if !exp.summary.is_empty() { push_block(&mut fm, "summary", &exp.summary); }
    push_list(&mut fm, "scope_characters", &exp.scope_characters);
    push_list(&mut fm, "scope_group_chats", &exp.scope_group_chats);
    push_list(&mut fm, "run_ids", &exp.run_ids);
    push_list(&mut fm, "follow_ups", &exp.follow_ups);
    push_list(&mut fm, "reports", &exp.reports);
    fm.push_str("---\n");

    let body = if exp.body.is_empty() { "".to_string() }
               else if exp.body.starts_with('\n') { exp.body.clone() }
               else { format!("\n{}", exp.body) };
    let full = format!("{}{}", fm, body);
    std::fs::write(&path, full).map_err(|e| e.to_string())?;
    Ok(())
}

fn push_scalar(out: &mut String, key: &str, value: &str) {
    if value.is_empty() { return; }
    out.push_str(&format!("{}: {}\n", key, value));
}
fn push_block(out: &mut String, key: &str, value: &str) {
    if value.is_empty() { return; }
    out.push_str(&format!("{}: |\n", key));
    for line in value.lines() {
        out.push_str(&format!("  {}\n", line));
    }
    out.push('\n');
}
fn push_list(out: &mut String, key: &str, items: &[String]) {
    if items.is_empty() { return; }
    out.push_str(&format!("{}:\n", key));
    for item in items {
        out.push_str(&format!("  - {}\n", item));
    }
}

async fn cmd_lab(r: &Resolved, action: LabAction, api_key: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        LabAction::List { status } => {
            let mut experiments = list_experiments().map_err(Box::<dyn std::error::Error>::from)?;
            if let Some(s) = status.as_ref() {
                experiments.retain(|e| e.status == *s);
            }
            if experiments.is_empty() {
                if !r.json {
                    println!("No experiments in the registry at {}.", experiments_dir().display());
                    println!("Create one with `worldcli lab propose <slug> --hypothesis \"...\" --mode passive --prediction \"...\"`");
                }
                emit(r.json, JsonValue::Array(Vec::new()));
                return Ok(());
            }
            let out: Vec<JsonValue> = experiments.iter().map(|e| json!({
                "slug": e.slug,
                "status": e.status,
                "mode": e.mode,
                "hypothesis": e.hypothesis,
                "ref": e.git_ref,
                "rubric_ref": e.rubric_ref,
                "created_at": e.created_at,
                "run_ids": e.run_ids,
                "reports": e.reports,
            })).collect();
            if r.json {
                emit(true, JsonValue::Array(out));
            } else {
                for e in &experiments {
                    let status_tag = format!("[{}]", e.status);
                    println!("{:<10} {:<10} {}", status_tag, e.mode, e.slug);
                    let first_line = e.hypothesis.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() {
                        let truncated = if first_line.chars().count() > 110 {
                            let s: String = first_line.chars().take(110).collect();
                            format!("{}…", s)
                        } else { first_line.to_string() };
                        println!("           {}", truncated);
                    }
                }
            }
        }
        LabAction::Open => {
            let experiments = list_experiments().map_err(Box::<dyn std::error::Error>::from)?;
            let open: Vec<&ExperimentFile> = experiments.iter()
                .filter(|e| matches!(e.status.as_str(), "proposed" | "running" | "open"))
                .collect();
            if r.json {
                let out: Vec<JsonValue> = open.iter().map(|e| json!({
                    "slug": e.slug, "status": e.status, "mode": e.mode,
                    "hypothesis": e.hypothesis, "ref": e.git_ref,
                })).collect();
                emit(true, JsonValue::Array(out));
            } else {
                if open.is_empty() {
                    println!("No open experiments. Everything in the registry is resolved (confirmed/refuted).");
                    return Ok(());
                }
                println!("Open experiments ({}):", open.len());
                for e in &open {
                    let status_tag = format!("[{}]", e.status);
                    println!("  {:<10} {:<10} {}", status_tag, e.mode, e.slug);
                    let first_line = e.hypothesis.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() {
                        println!("             {}", first_line.chars().take(110).collect::<String>());
                    }
                }
            }
        }
        LabAction::Show { slug } => {
            let exp = load_experiment(&slug).map_err(Box::<dyn std::error::Error>::from)?;
            if r.json {
                emit(true, json!({
                    "slug": exp.slug,
                    "path": exp.path.display().to_string(),
                    "id": exp.id, "status": exp.status, "mode": exp.mode,
                    "ref": exp.git_ref, "rubric_ref": exp.rubric_ref,
                    "created_at": exp.created_at, "resolved_at": exp.resolved_at,
                    "hypothesis": exp.hypothesis,
                    "prediction": exp.prediction,
                    "summary": exp.summary,
                    "scope_characters": exp.scope_characters,
                    "scope_group_chats": exp.scope_group_chats,
                    "run_ids": exp.run_ids,
                    "follow_ups": exp.follow_ups,
                    "reports": exp.reports,
                    "body": exp.body,
                }));
            } else {
                println!("{}", exp.raw);
            }
        }
        LabAction::Search { query } => {
            let q = query.to_lowercase();
            let experiments = list_experiments().map_err(Box::<dyn std::error::Error>::from)?;
            let hits: Vec<&ExperimentFile> = experiments.iter()
                .filter(|e| e.raw.to_lowercase().contains(&q))
                .collect();
            if r.json {
                let out: Vec<JsonValue> = hits.iter().map(|e| json!({
                    "slug": e.slug, "status": e.status, "mode": e.mode,
                    "hypothesis": e.hypothesis,
                })).collect();
                emit(true, JsonValue::Array(out));
            } else {
                if hits.is_empty() {
                    println!("No experiments match '{}'.", query);
                    return Ok(());
                }
                for e in &hits {
                    let status_tag = format!("[{}]", e.status);
                    println!("{:<10} {}", status_tag, e.slug);
                    let first_line = e.hypothesis.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() {
                        println!("           {}", first_line.chars().take(110).collect::<String>());
                    }
                }
            }
        }
        LabAction::Propose { slug, hypothesis, mode, prediction, r#ref, rubric_ref } => {
            // Refuse to overwrite an existing experiment.
            let path = experiments_dir().join(format!("{}.md", slug));
            if path.exists() {
                return Err(Box::<dyn std::error::Error>::from(
                    format!("experiment '{}' already exists at {}. Edit the file directly, or pick a new slug.",
                        slug, path.display())));
            }
            let valid_modes = ["passive", "qualitative", "active"];
            if !valid_modes.contains(&mode.as_str()) {
                return Err(Box::<dyn std::error::Error>::from(
                    format!("invalid --mode '{}'; must be one of {:?}", mode, valid_modes)));
            }
            let exp = ExperimentFile {
                slug: slug.clone(),
                path: path.clone(),
                id: slug.clone(),
                status: "proposed".to_string(),
                mode,
                git_ref: r#ref.unwrap_or_default(),
                rubric_ref: rubric_ref.unwrap_or_default(),
                created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                resolved_at: String::new(),
                hypothesis,
                prediction,
                summary: String::new(),
                scope_characters: Vec::new(),
                scope_group_chats: Vec::new(),
                run_ids: Vec::new(),
                follow_ups: Vec::new(),
                reports: Vec::new(),
                body: String::new(),
                raw: String::new(),
            };
            write_experiment(&exp).map_err(Box::<dyn std::error::Error>::from)?;
            if r.json {
                emit(true, json!({
                    "slug": exp.slug, "path": exp.path.display().to_string(),
                    "status": exp.status, "created_at": exp.created_at,
                }));
            } else {
                println!("Proposed experiment: {} (status=proposed)", exp.slug);
                println!("File: {}", path.display());
                println!();
                println!("Next steps:");
                println!("  1. Review the file; edit scope_characters / scope_group_chats as needed.");
                println!("  2. When the run starts: edit status → running.");
                println!("  3. After the run: `worldcli lab link-run {} <run_id>`.", exp.slug);
                println!("  4. When interpreted: `worldcli lab resolve {} --status confirmed|refuted --summary \"...\"`.", exp.slug);
            }
        }
        LabAction::Resolve { slug, status, summary, report } => {
            let valid_statuses = ["proposed", "running", "open", "confirmed", "refuted"];
            if !valid_statuses.contains(&status.as_str()) {
                return Err(Box::<dyn std::error::Error>::from(
                    format!("invalid --status '{}'; must be one of {:?}", status, valid_statuses)));
            }
            let mut exp = load_experiment(&slug).map_err(Box::<dyn std::error::Error>::from)?;
            exp.status = status.clone();
            if let Some(s) = summary { exp.summary = s; }
            if let Some(rp) = report {
                if !exp.reports.contains(&rp) { exp.reports.push(rp); }
            }
            if matches!(status.as_str(), "confirmed" | "refuted" | "open") {
                exp.resolved_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            }
            write_experiment(&exp).map_err(Box::<dyn std::error::Error>::from)?;
            if r.json {
                emit(true, json!({
                    "slug": exp.slug, "status": exp.status,
                    "resolved_at": exp.resolved_at, "summary": exp.summary,
                }));
            } else {
                println!("Resolved {}: status={}", exp.slug, exp.status);
                if !exp.resolved_at.is_empty() { println!("resolved_at: {}", exp.resolved_at); }
                if !exp.summary.is_empty() { println!("summary: {}", exp.summary.lines().next().unwrap_or("")); }
            }
        }
        LabAction::LinkRun { slug, run_id } => {
            let mut exp = load_experiment(&slug).map_err(Box::<dyn std::error::Error>::from)?;
            if exp.run_ids.contains(&run_id) {
                if !r.json {
                    println!("Run {} already linked to {}.", run_id, slug);
                }
                return Ok(());
            }
            exp.run_ids.push(run_id.clone());
            write_experiment(&exp).map_err(Box::<dyn std::error::Error>::from)?;
            if r.json {
                emit(true, json!({"slug": exp.slug, "run_ids": exp.run_ids}));
            } else {
                println!("Linked run {} → {} (now {} total).", run_id, slug, exp.run_ids.len());
            }
        }
        LabAction::Scenario { action } => {
            match action {
                ScenarioAction::List => cmd_lab_scenario_list(r)?,
                ScenarioAction::Show { name } => cmd_lab_scenario_show(r, &name)?,
                ScenarioAction::Run { name, character, rubric_ref, skip_evaluate, model, confirm_cost } => {
                    let key = api_key.ok_or_else(|| Box::<dyn std::error::Error>::from(
                        "internal: api_key missing for scenario run".to_string()))?;
                    cmd_lab_scenario_run(r, key, &name, &character, rubric_ref.as_deref(),
                        skip_evaluate, model.as_deref(), confirm_cost).await?;
                }
            }
        }
    }
    Ok(())
}

// ─── Scenario templates (experiments/scenarios/*.md) ────────────────────
//
// Canonical probe sequences for Mode C. Each scenario is a markdown file
// with YAML-ish frontmatter (name, purpose, measure_with) and a body of
// `## Variant: <label>` sections whose text is the prompt to send. The
// `lab scenario run` command fires each variant as a fresh dialogue
// call (no session history; each variant is its own controlled
// condition) and optionally applies the measure_with rubric to every
// reply.

fn scenarios_dir() -> PathBuf { PathBuf::from("experiments/scenarios") }

#[derive(Debug, Clone, Default)]
struct ScenarioFile {
    name: String,
    path: PathBuf,
    purpose: String,
    measure_with: String,
    /// Ordered list of (variant_label, prompt_text) — order matters
    /// because the `run` output presents variants in sequence.
    variants: Vec<(String, String)>,
    raw: String,
}

/// Parse the `## Variant: <label>\n<body>` sections out of a scenario
/// file's markdown body. Returns ordered pairs.
fn extract_scenario_variants(body: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current_label: Option<String> = None;
    let mut current_buf: Vec<String> = Vec::new();
    let flush = |label: Option<String>, buf: Vec<String>, out: &mut Vec<(String, String)>| {
        if let Some(l) = label {
            let text = buf.join("\n").trim().to_string();
            if !text.is_empty() {
                out.push((l, text));
            }
        }
    };
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## Variant:") {
            flush(current_label.take(), std::mem::take(&mut current_buf), &mut out);
            current_label = Some(rest.trim().to_string());
        } else if current_label.is_some() {
            current_buf.push(line.to_string());
        }
    }
    flush(current_label, current_buf, &mut out);
    out
}

fn load_scenario(name: &str) -> Result<ScenarioFile, String> {
    let path = scenarios_dir().join(format!("{}.md", name));
    if !path.exists() {
        return Err(format!("scenario '{}' not found at {}. Run `worldcli lab scenario list` to see the templates.", name, path.display()));
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    let (fm_text, body) = split_frontmatter(&raw)
        .ok_or_else(|| format!("scenario '{}' has no `---` frontmatter fence", name))?;
    // Reuse the experiment parser's scalar machinery for just the three
    // fields we care about (name, purpose, measure_with).
    let mut sf = ScenarioFile::default();
    sf.name = name.to_string();
    sf.path = path;
    sf.raw = raw;
    for line in fm_text.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with(' ') || trimmed.starts_with('\t') { continue; }
        if let Some(ci) = trimmed.find(':') {
            let key = trimmed[..ci].trim();
            let value = trimmed[ci + 1..].trim().trim_matches('"').to_string();
            match key {
                "name" => if !value.is_empty() { sf.name = value; },
                "purpose" => sf.purpose = value,
                "measure_with" => sf.measure_with = value,
                _ => {}
            }
        }
    }
    sf.variants = extract_scenario_variants(&body);
    if sf.variants.is_empty() {
        return Err(format!("scenario '{}' has no `## Variant: <label>` sections", name));
    }
    Ok(sf)
}

fn list_scenarios() -> Result<Vec<ScenarioFile>, String> {
    let dir = scenarios_dir();
    if !dir.exists() { return Ok(Vec::new()); }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
        let fname = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if fname == "README" { continue; }
        if let Ok(sf) = load_scenario(&fname) {
            out.push(sf);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn scenario_runs_dir() -> PathBuf { worldcli_home().join("scenario-runs") }
fn scenario_runs_manifest() -> PathBuf { scenario_runs_dir().join("manifest.jsonl") }

fn write_scenario_run(run_id: &str, envelope: &JsonValue) {
    let dir = scenario_runs_dir();
    let _ = std::fs::create_dir_all(&dir);
    let per_path = dir.join(format!("{}.json", run_id));
    if let Ok(s) = serde_json::to_string_pretty(envelope) {
        let _ = std::fs::write(&per_path, s);
    }
    let manifest_entry = json!({
        "run_id": envelope.get("run_id"),
        "run_timestamp": envelope.get("run_timestamp"),
        "scenario": envelope.get("scenario"),
        "character_id": envelope.get("character_id"),
        "character_name": envelope.get("character_name"),
        "variants": envelope.get("variants").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
        "measure_with": envelope.get("measure_with"),
        "cost_usd": envelope.get("cost").and_then(|c| c.get("actual_usd")),
    });
    let line = serde_json::to_string(&manifest_entry).unwrap_or_default();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true).open(scenario_runs_manifest())
    {
        use std::io::Write;
        let _ = writeln!(f, "{}", line);
    }
}

fn cmd_lab_scenario_list(r: &Resolved) -> Result<(), Box<dyn std::error::Error>> {
    let scenarios = list_scenarios().map_err(Box::<dyn std::error::Error>::from)?;
    if scenarios.is_empty() {
        if !r.json {
            println!("No scenarios found at {}.", scenarios_dir().display());
            println!("See experiments/scenarios/README.md for the authoring convention.");
        }
        emit(r.json, JsonValue::Array(Vec::new()));
        return Ok(());
    }
    let out: Vec<JsonValue> = scenarios.iter().map(|s| json!({
        "name": s.name,
        "purpose": s.purpose,
        "measure_with": s.measure_with,
        "variant_count": s.variants.len(),
        "variants": s.variants.iter().map(|(l, _)| l.clone()).collect::<Vec<_>>(),
    })).collect();
    if r.json {
        emit(true, JsonValue::Array(out));
    } else {
        for s in &scenarios {
            println!("{:<32} {} variants", s.name, s.variants.len());
            if !s.purpose.is_empty() {
                println!("  purpose:      {}", s.purpose);
            }
            if !s.measure_with.is_empty() {
                println!("  measure_with: {}", s.measure_with);
            }
            let labels: Vec<String> = s.variants.iter().map(|(l, _)| l.clone()).collect();
            println!("  variants:     {}", labels.join(", "));
        }
    }
    Ok(())
}

fn cmd_lab_scenario_show(r: &Resolved, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let s = load_scenario(name).map_err(Box::<dyn std::error::Error>::from)?;
    if r.json {
        emit(true, json!({
            "name": s.name, "path": s.path.display().to_string(),
            "purpose": s.purpose, "measure_with": s.measure_with,
            "variants": s.variants.iter().map(|(l, p)| json!({
                "label": l, "prompt": p,
            })).collect::<Vec<_>>(),
            "raw": s.raw,
        }));
    } else {
        println!("{}", s.raw);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn cmd_lab_scenario_run(
    r: &Resolved,
    api_key: &str,
    name: &str,
    character_id: &str,
    rubric_ref_override: Option<&str>,
    skip_evaluate: bool,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let scenario = load_scenario(name).map_err(Box::<dyn std::error::Error>::from)?;
    let _ = r.check_character(character_id)?;

    // Resolve rubric (for evaluator) — precedence: CLI override, then
    // scenario's measure_with, then nothing.
    let rubric_name = rubric_ref_override
        .map(str::to_string)
        .or_else(|| {
            if scenario.measure_with.is_empty() { None }
            else { Some(scenario.measure_with.clone()) }
        });
    let rubric_text: Option<(String, String, String)> = if skip_evaluate {
        None
    } else if let Some(rn) = rubric_name.as_ref() {
        let rb = load_rubric(rn).map_err(Box::<dyn std::error::Error>::from)?;
        Some((rb.name, rb.version, rb.prompt))
    } else { None };

    // Build the system prompt ONCE — held constant across variants.
    let (system_prompt, mut model_config, character, _world_id) = {
        let conn = r.db.conn.lock().unwrap();
        let character = get_character(&conn, character_id)?;
        let world = get_world(&conn, &character.world_id)?;
        let user_profile = get_user_profile(&conn, &character.world_id).ok();
        let recent_journals = list_journal_entries(&conn, character_id, 1).unwrap_or_default();
        let active_quests = list_active_quests(&conn, &character.world_id).unwrap_or_default();
        let latest_stance = latest_relational_stance(&conn, character_id).unwrap_or(None);
        let stance_text: Option<String> = latest_stance.as_ref().map(|s| s.stance_text.clone());
        let anchor_text: Option<String> = combined_axes_block(&conn, character_id);
        let system = app_lib::ai::prompts::build_dialogue_system_prompt(
            &world, &character, user_profile.as_ref(),
            None, Some("Auto"), None, None, false, &[], None,
            &recent_journals, None, &[], None, &active_quests,
            stance_text.as_deref(),
            anchor_text.as_deref(),
        );
        let mc = orchestrator::load_model_config(&conn);
        let world_id = character.world_id.clone();
        (system, mc, character, world_id)
    };
    if let Some(m) = model_override { model_config.dialogue_model = m.to_string(); }

    // Project cost: N dialogue calls + (optionally) N evaluator calls.
    let dialogue_in_est = estimate_tokens(&system_prompt);
    let per_variant_out: i64 = 600;
    let dialogue_cost: f64 = scenario.variants.iter().map(|(_, p)| {
        let in_tok = dialogue_in_est + estimate_tokens(p);
        project_cost(&model_config.dialogue_model, in_tok, per_variant_out, &r.cfg.model_pricing)
    }).sum();
    let evaluator_cost: f64 = if let Some((_, _, rp)) = &rubric_text {
        let per_eval_in = estimate_tokens(rp) + 300 + 200 + 180 * 3 + 150;
        let per_eval_out: i64 = 220;
        let per_call = project_cost(&model_config.memory_model, per_eval_in, per_eval_out, &r.cfg.model_pricing);
        per_call * (scenario.variants.len() as f64)
    } else { 0.0 };
    let total_projected = dialogue_cost + evaluator_cost;

    let daily_so_far = rolling_24h_total_usd();
    let daily_after = daily_so_far + total_projected;
    let per_call_cap = r.cfg.budget.per_call_usd;
    let daily_cap = r.cfg.budget.daily_usd;
    let confirm = confirm_cost.unwrap_or(0.0);
    if total_projected > per_call_cap && confirm < total_projected {
        return Err(Box::new(CliError::Budget {
            kind: "per_call (scenario total)".to_string(),
            projected_usd: total_projected,
            cap_usd: per_call_cap,
            confirm_at_least: (total_projected * 1.05).max(0.01),
        }));
    }
    if daily_after > daily_cap && confirm < total_projected {
        return Err(Box::new(CliError::Budget {
            kind: "daily".to_string(),
            projected_usd: daily_after,
            cap_usd: daily_cap,
            confirm_at_least: (total_projected * 1.05).max(0.01),
        }));
    }

    if !r.json {
        eprintln!("[worldcli] scenario '{}' × {} variants against {} via {} — dialogue≈${:.4}, evaluator≈${:.4}, total≈${:.4}; 24h=${:.4}/${:.2}",
            scenario.name, scenario.variants.len(), character.display_name,
            model_config.dialogue_model, dialogue_cost, evaluator_cost,
            total_projected, daily_so_far, daily_cap);
    }

    let base_url = model_config.chat_api_base();
    let mut per_variant_results: Vec<JsonValue> = Vec::new();
    let mut total_in: i64 = 0;
    let mut total_out: i64 = 0;
    let mut total_in_eval: i64 = 0;
    let mut total_out_eval: i64 = 0;

    for (i, (label, prompt_text)) in scenario.variants.iter().enumerate() {
        eprint!("\r[worldcli] variant {}/{}: {}", i + 1, scenario.variants.len(), label);
        let req = openai::ChatRequest {
            model: model_config.dialogue_model.clone(),
            messages: vec![
                openai::ChatMessage { role: "system".to_string(), content: system_prompt.clone() },
                openai::ChatMessage { role: "user".to_string(), content: prompt_text.clone() },
            ],
            temperature: Some(0.95),
            max_completion_tokens: None,
            response_format: None,
        };
        // Per-variant errors (network, 5xx, rate-limit) get recorded in
        // the envelope rather than aborting the whole run — a flaky
        // middle variant shouldn't throw away the completed ones.
        let resp = match openai::chat_completion_with_base(&base_url, api_key, &req).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!();
                eprintln!("[worldcli] variant '{}' dialogue call failed: {} — continuing", label, e);
                per_variant_results.push(json!({
                    "label": label,
                    "prompt": prompt_text,
                    "reply": null,
                    "error": format!("dialogue call failed: {}", e),
                    "verdict": null,
                }));
                continue;
            }
        };
        let reply = match resp.choices.first().map(|c| c.message.content.clone()) {
            Some(s) => s,
            None => {
                eprintln!();
                eprintln!("[worldcli] variant '{}' returned no choices — continuing", label);
                per_variant_results.push(json!({
                    "label": label,
                    "prompt": prompt_text,
                    "reply": null,
                    "error": "no choices returned",
                    "verdict": null,
                }));
                continue;
            }
        };
        let usage = resp.usage.unwrap_or(openai::Usage {
            prompt_tokens: 0, completion_tokens: 0, total_tokens: 0,
        });
        total_in += usage.prompt_tokens as i64;
        total_out += usage.completion_tokens as i64;

        // Optional evaluator pass.
        let verdict: Option<JsonValue> = if let Some((_, _, rp)) = rubric_text.as_ref() {
            let ctx = vec![("User".to_string(), prompt_text.clone())];
            match evaluate_one(&base_url, api_key, &model_config.memory_model, rp, &ctx, &reply).await {
                Ok((v, u)) => {
                    total_in_eval += u.prompt_tokens as i64;
                    total_out_eval += u.completion_tokens as i64;
                    Some(json!({
                        "judgment": v.judgment, "confidence": v.confidence,
                        "quote": v.quote, "reasoning": v.reasoning,
                    }))
                }
                Err(e) => Some(json!({ "error": e })),
            }
        } else { None };

        per_variant_results.push(json!({
            "label": label,
            "prompt": prompt_text,
            "reply": reply,
            "verdict": verdict,
            "dialogue_usage": {
                "prompt_tokens": usage.prompt_tokens,
                "completion_tokens": usage.completion_tokens,
            },
        }));
    }
    eprintln!();

    let actual_dialogue_usd = actual_cost(&model_config.dialogue_model, total_in, total_out, &r.cfg.model_pricing);
    let actual_eval_usd = actual_cost(&model_config.memory_model, total_in_eval, total_out_eval, &r.cfg.model_pricing);
    let actual_usd = actual_dialogue_usd + actual_eval_usd;
    // Log both separately so cost.jsonl attributes to the right model.
    append_cost_log(&CostEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: model_config.dialogue_model.clone(),
        prompt_tokens: total_in,
        completion_tokens: total_out,
        usd: actual_dialogue_usd,
    });
    if total_in_eval > 0 || total_out_eval > 0 {
        append_cost_log(&CostEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            model: model_config.memory_model.clone(),
            prompt_tokens: total_in_eval,
            completion_tokens: total_out_eval,
            usd: actual_eval_usd,
        });
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    let envelope = json!({
        "run_id": run_id,
        "run_timestamp": chrono::Utc::now().to_rfc3339(),
        "scenario": scenario.name,
        "scenario_path": scenario.path.display().to_string(),
        "purpose": scenario.purpose,
        "character_id": character_id,
        "character_name": character.display_name,
        "dialogue_model": model_config.dialogue_model,
        "measure_with": rubric_name,
        "rubric_version": rubric_text.as_ref().map(|(_, v, _)| v.clone()),
        "variants": per_variant_results,
        "cost": {
            "dialogue_prompt_tokens": total_in,
            "dialogue_completion_tokens": total_out,
            "evaluator_prompt_tokens": total_in_eval,
            "evaluator_completion_tokens": total_out_eval,
            "actual_usd": actual_usd,
        },
    });
    write_scenario_run(&run_id, &envelope);

    if r.json {
        emit(true, envelope);
    } else {
        println!("=== SCENARIO RUN ===");
        println!("scenario:  {} ({})", scenario.name, scenario.path.display());
        println!("purpose:   {}", scenario.purpose);
        println!("character: {} ({})", character.display_name, character_id);
        println!("model:     {}", model_config.dialogue_model);
        if let Some(rn) = rubric_name.as_ref() {
            println!("rubric:    {}", rn);
        } else {
            println!("rubric:    (none — replies only)");
        }
        println!("run_id:    {}", run_id);
        println!();
        for v in envelope["variants"].as_array().unwrap_or(&vec![]) {
            let label = v["label"].as_str().unwrap_or("?");
            let prompt = v["prompt"].as_str().unwrap_or("");
            let reply = v["reply"].as_str().unwrap_or("");
            println!("─── Variant: {} ───", label);
            println!("PROMPT: {}", prompt);
            println!();
            println!("REPLY:");
            println!("{}", reply);
            if let Some(verdict) = v.get("verdict") {
                if !verdict.is_null() {
                    if let Some(err) = verdict.get("error").and_then(|e| e.as_str()) {
                        println!();
                        println!("VERDICT: ERROR — {}", err);
                    } else {
                        let j = verdict["judgment"].as_str().unwrap_or("?");
                        let c = verdict["confidence"].as_str().unwrap_or("?");
                        let q = verdict["quote"].as_str().unwrap_or("");
                        let rs = verdict["reasoning"].as_str().unwrap_or("");
                        println!();
                        println!("VERDICT: {} ({}) — \"{}\"", j, c, q);
                        println!("         → {}", rs);
                    }
                }
            }
            println!();
        }
        eprintln!("[worldcli] actual cost ${:.4} (dialogue=${:.4}, evaluator=${:.4})",
            actual_usd, actual_dialogue_usd, actual_eval_usd);
    }
    Ok(())
}

// ─── Consultant (CLI variant — both modes, no UI action cards) ──────────

/// Build the CLI consultant's system prompt. Shares the SPIRIT of the
/// in-app Story Consultant (two registers — immersive confidant vs.
/// backstage stage-manager) but strips the UI-coupled action-card
/// affordances (canon_entry, staged_message, portrait_regen,
/// illustration, new_group_chat, propose_quest) because the CLI has
/// no UI to render them. What stays: the character + world + recent
/// conversation context, and the mode-specific voice posture.

async fn cmd_consult(
    r: &Resolved,
    api_key: &str,
    character_id: &str,
    message: &str,
    mode: &str,
    session: Option<&str>,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
    question_summary: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if mode != "immersive" && mode != "backstage" {
        return Err(Box::<dyn std::error::Error>::from(format!(
            "invalid --mode '{}'; must be 'immersive' or 'backstage'", mode
        )));
    }
    let _ = r.check_character(character_id)?;

    // Use the shared ai::consultant helper — same system-prompt
    // genealogy as the in-app story_consultant_cmd. CLI gets full
    // parity (including the action-card instructions; the CLI just
    // surfaces them as fenced `action` blocks in the reply text
    // without a one-click render surface).
    let (system_prompt, mut model_config) = app_lib::ai::consultant::build_consultant_system_prompt(
        &r.db,
        mode,
        Some(character_id),
        None,
    ).map_err(Box::<dyn std::error::Error>::from)?;
    if let Some(m) = model_override { model_config.dialogue_model = m.to_string(); }

    // Character-name + world_id for run-log tagging, plus dev-session
    // history if --session was supplied.
    let (prior_messages, session_id, character_name, world_id) = {
        let conn = r.db.conn.lock().unwrap();
        let character = get_character(&conn, character_id)?;
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
        (prior_messages, session_id, character.display_name.clone(), character.world_id.clone())
    };

    let mut messages = vec![openai::ChatMessage { role: "system".to_string(), content: system_prompt }];
    for (role, content) in prior_messages.iter() {
        messages.push(openai::ChatMessage { role: role.clone(), content: content.clone() });
    }
    messages.push(openai::ChatMessage { role: "user".to_string(), content: message.to_string() });

    // Cost projection — same gate as `ask`.
    let prompt_text_total: String = messages.iter().map(|m| m.content.as_str()).collect::<Vec<_>>().join("\n");
    let projected_in = estimate_tokens(&prompt_text_total);
    let projected_out: i64 = 700;
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
        eprintln!("[worldcli] consulting ({}) about {} via {} — projected≈${:.4} (~{} in / {} out tok); 24h spent=${:.4} of ${:.2}",
            mode, character_name, model_config.dialogue_model, projected_usd, projected_in, projected_out,
            daily_so_far, daily_cap);
    }

    let request = openai::ChatRequest {
        model: model_config.dialogue_model.clone(),
        messages,
        temperature: Some(0.9),
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

    append_cost_log(&CostEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: model_config.dialogue_model.clone(),
        prompt_tokens: actual_in,
        completion_tokens: actual_out,
        usd: actual_usd,
    });

    // Persist a run record tagged with the mode so runs-search can find it.
    let run_id = uuid::Uuid::new_v4().to_string();
    let record = RunRecord {
        id: run_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        character_id: character_id.to_string(),
        character_name: format!("{} [consult:{}]", character_name, mode),
        world_id,
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
            "mode": mode,
            "character_id": character_id,
            "character_name": character_name,
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

// ─── Load-test anchor inspect + manual refresh ─────────────────────────

fn cmd_show_anchor(r: &Resolved, character_id: &str, history: i64) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;
    let conn = r.db.conn.lock().unwrap();
    let anchors = list_load_test_anchors(&conn, character_id, history)?;
    let out: Vec<JsonValue> = anchors.iter().map(|a| json!({
        "anchor_id": a.anchor_id,
        "character_id": a.character_id,
        "world_id": a.world_id,
        "anchor_label": a.anchor_label,
        "anchor_body": a.anchor_body,
        "derivation_summary": a.derivation_summary,
        "world_day_at_generation": a.world_day_at_generation,
        "source_message_count": a.source_message_count,
        "refresh_trigger": a.refresh_trigger,
        "model_used": a.model_used,
        "created_at": a.created_at,
    })).collect();
    if r.json {
        emit(true, JsonValue::Array(out));
    } else {
        if anchors.is_empty() {
            println!("No load-test anchor has been synthesized for this character yet.");
            println!("Run `worldcli refresh-anchor {}` to generate the first one.", character_id);
            return Ok(());
        }
        for (i, a) in anchors.iter().enumerate() {
            if i > 0 { println!(); println!("───"); println!(); }
            println!("anchor_id:   {}", a.anchor_id);
            println!("label:       {}", a.anchor_label);
            println!("created_at:  {}", a.created_at);
            println!("world_day:   {}", a.world_day_at_generation.map(|d| d.to_string()).unwrap_or_else(|| "?".to_string()));
            println!("source_msgs: {}", a.source_message_count);
            println!("trigger:     {}", a.refresh_trigger);
            println!("model:       {}", a.model_used);
            println!();
            println!("BODY (injected into dialogue system prompt):");
            println!("{}", a.anchor_body);
            if !a.derivation_summary.is_empty() {
                println!();
                println!("DERIVATION (how this anchor was identified):");
                println!("{}", a.derivation_summary);
            }
        }
    }
    Ok(())
}

async fn cmd_refresh_anchor(
    r: &Resolved,
    api_key: &str,
    character_id: &str,
    model_override: Option<&str>,
    confirm_cost: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = r.check_character(character_id)?;

    // Pick model + cap-check. Default is dialogue_model (sharper
    // synthesis quality matters for anchor identification — see the
    // 2026-04-24 discussion about memory_model vs dialogue_model cost
    // tradeoff). User can override with --model gpt-4o-mini for a
    // 30-50x cheaper run if they accept the quality risk.
    let model_config = {
        let conn = r.db.conn.lock().unwrap();
        orchestrator::load_model_config(&conn)
    };
    let model = model_override.unwrap_or(&model_config.dialogue_model).to_string();

    // Conservative pre-check: up to 8000 input tokens (30 corpus
    // excerpts trimmed to 600 chars each + system + prior anchor) and
    // ~1200 output tokens (JSON object with three fields).
    let projected_usd = project_cost(&model, 8000, 1200, &r.cfg.model_pricing);
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
            "[worldcli] refreshing load-test anchor for {} via {} (projected≈${:.4})",
            character_id, model, projected_usd
        );
    }

    let base_url = model_config.chat_api_base();
    let res = load_test_anchor::refresh_load_test_anchor(
        r.db.conn.clone(),
        base_url,
        api_key.to_string(),
        model.clone(),
        character_id.to_string(),
        "manual_cli".to_string(),
    ).await;
    let (axes_inserted, prompt_tokens, completion_tokens) = match res {
        Ok(t) => t,
        Err(e) => return Err(Box::<dyn std::error::Error>::from(e)),
    };

    // Log actual cost so worldcli status reflects the spend (fixes
    // the cost-tracking-bypass bug noted in the 2026-04-24 reports).
    let actual_usd = actual_cost(&model, prompt_tokens, completion_tokens, &r.cfg.model_pricing);
    append_cost_log(&CostEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        model: model.clone(),
        prompt_tokens,
        completion_tokens,
        usd: actual_usd,
    });
    if !r.json {
        eprintln!("[worldcli] axis synthesis: {} axes inserted, ${:.4} actual ({} in / {} out tok)",
            axes_inserted, actual_usd, prompt_tokens, completion_tokens);
    }

    // Echo the latest axes (one per axis_kind).
    cmd_show_anchor(r, character_id, axes_inserted as i64)
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
    no_anchor: bool,
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
        // Anchor read is suppressed when --no-anchor is passed (for A/B
        // testing whether the synthesized anchors move real-time behavior).
        let anchor_text: Option<String> = if no_anchor {
            None
        } else {
            combined_axes_block(&conn, character_id)
        };

        let system_prompt = prompts::build_dialogue_system_prompt(
            &world, &character, user_profile.as_ref(),
            None, Some("Auto"), None, None, false, &[], None,
            &recent_journals, None, &[], None, &active_quests,
            stance_text.as_deref(),
            anchor_text.as_deref(),
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

# WorldThreads — Development Guidelines

## MISSION

Create a vivid, excellent, surprising in-world experience that uplifts the user and provides engrossing, good, clean fun. Every design decision, prompt tweak, UX choice, and feature bet is measured against that. The craft stack (invariants, craft notes, conscience pass, inventories, snapshots, etc.) exists to serve that mission — characters that feel real, worlds that hold, scenes that are worth the visit and send the user back to their day nourished rather than hollowed.

## DATABASE SAFETY — CRITICAL

**NEVER drop, delete, or destroy database data during migrations.** This is the #1 rule.

- NEVER use `DROP TABLE` on a table that contains user data unless the data has been **verified** to exist in the new table first (count check).
- NEVER use `.ok()` to silently swallow errors during data migrations. Always check results.
- When recreating a table to change constraints (e.g., CHECK constraints in SQLite):
  1. Rename old table to `{name}_migrating`
  2. Create new table
  3. INSERT data from old to new
  4. **VERIFY the row count matches** before dropping the old table
  5. If counts don't match, **ROLLBACK** by renaming the old table back
  6. Wrap in `PRAGMA foreign_keys=OFF` / `ON`
- Prefer `ALTER TABLE ADD COLUMN` over table recreation whenever possible.
- Always test migrations mentally against the current schema before writing them.
- When in doubt, do NOT migrate — find a workaround (e.g., store a different role value that passes existing constraints, or use application-level validation).

## Project Structure

- Tauri v2 (Rust backend + React/TypeScript frontend)
- SQLite database with FTS5, sqlite-vec
- `src-tauri/` — Rust backend
- `frontend/` — React frontend with Vite + Tailwind

## Reports

`reports/` holds reflective, interpretive reads of the project's git history — philosophy/trajectory/taste, not changelogs. Each new report is in dialogue with prior ones (revisits open questions they flagged, tests their predictions against subsequent commits).

Naming: `YYYY-MM-DD-<purpose-slug>.md` (e.g. `2026-04-21-philosophy-trajectory.md`). The slug should name the report's purpose, not genericize it.

A `post-commit` hook (`.githooks/post-commit`, wired via `core.hooksPath`) nudges when **10+ commits and 3+ days** have passed since the newest report. The floor is deliberately low so reports can keep up with active iteration — this project's current mode uses reports as a live retrospective channel, not a quarterly summary. Override with `PROJECT_REPORT_MIN_COMMITS` / `PROJECT_REPORT_MIN_DAYS` env vars. Ad-hoc `/project-report` runs are ALWAYS valid — the floor is a nudge threshold (the minimum rate at which the hook will bug you), not a ceiling (there is no "too often" for reports that genuinely name something new).

After a fresh clone, re-enable the hook with: `git config core.hooksPath .githooks`

A second genre of report lives in the same directory under the same naming convention: **natural-experiment findings** from `worldcli sample-windows`. Those are nudged by an in-flight design decision needing data, not by the time-or-volume floor — see the worldcli section below for the bar and the frequency discipline.

## How to read this craft stack

When reviewing, auditing, refactoring, or critiquing anything in this repo (especially `prompts.rs`), follow the **load-bearing-multiplicity prior**: when two directives appear to contradict each other, assume the multiplicity is intentional before assuming it's a bug. Apparent tension is almost always the same truth from different angles, not two principles needing a precedence rule. Full reading instructions in `docs/VOICE.md` under "Reading this work, especially as an AI."

## Commit/push autonomy

Standing authorization to **commit and push at will** on clean work — no need to ask before every commit or push. Group changes into coherent commits, write descriptive messages in the project's existing style, then push. Destructive git operations (force-push, reset --hard, branch deletion, history rewrites, etc.) STILL require explicit confirmation — that's not autonomy, that's a different category. Commit + push is the default; ask only when something is risky or unclear.

## Earned-exception carve-outs on absolute rules

Whenever you draft an absolute-shaped rule in this repo — "never X," "always Y," "don't ever Z," "do NOT do W" — **immediately check whether there's a genuine earned-exception that belongs alongside it**, and write it in the same pass. This is a house pattern across `prompts.rs`, `INVARIANTS.md`, the craft notes, and CLAUDE.md itself. The rigidity stays; the carve-out sits beside it so the rigidity doesn't collapse a genuinely valid moment.

The pattern:

1. State the default rule plainly and firmly.
2. Name the narrow earned exception, with its own test that an edge case would have to pass to qualify.
3. Preserve the rigidity against everything outside the exception. "If none of the exceptions apply, the default holds."

Examples already in the stack: *"Don't analyze the user — unless they want to be analyzed"* (three exceptions: invited, role-appropriate, character-motivated from a real relationship). *"Don't end on a proverb, unless it's earned"* (exception built into the title). *"No dramatic self-awareness"* → earned moment of articulate clarity. *"Don't tie a ribbon on every reply"* → a sliver of permission for the earned witty close. *"Don't wrap; carry unfinishedness"* → when closure IS the truth. REVERENCE invariant → user breaks frame and asks sincerely. NOURISHMENT invariant → the in-scene friend-check. The propose_quest craft note → six carve-outs for the six rigid rules.

When this does NOT apply: rules whose nature is categorical (duplicate-prevention checks, safety-critical bans, load-bearing theological anchors that ARE the point). Those stay absolute because their force is in their absoluteness; a carve-out would leak the invariant.

The check: after you draft any "don't / never / always / do NOT" rule, pause. Ask: *is there a moment where this rule would produce the wrong result, and is that moment narrow and nameable?* If yes, write the carve-out now, in the same draft. If no, leave it absolute and note why the absoluteness is the point.

Missing this pattern is a drift — when you notice an absolute-shaped rule in the stack without a carve-out, assume it's a gap to fix rather than a deliberate choice (unless the categorical-nature test above explicitly justifies the absolute).

## Nudge the action forward after a closing beat

Same craft rule the dialogue prompt's **Drive the moment** note applies to characters: every reply should move the scene by at least one small honest degree. Apply it to yourself. A closing beat like *"Pleasure's mine"* or *"Go enjoy it"* is fine — BUT pair it with a small forward nudge. A planted thought to carry, a practical next step, a small question that opens a door, a beat of specificity that gives the user something concrete to do with the moment. One sentence of forward motion after the close.

The rule isn't "always suggest a next task." It's "don't dead-end the conversation by mistake." If the user is genuinely winding down, match it; a warm close with no nudge is better than an artificial tail. But the DEFAULT — when a real reply still has room — is close + nudge. *"Take the time. I'll be here"* is fine; *"When you're done sitting, this session might be its own report"* is better because it plants something forward.

The craft note from `prompts.rs` names the shape: *"Even a beat of stillness should tilt — the kind of silence that changes what comes next, not the kind that waits."* Apply it here.

## Ask the character — character as craft collaborator

When the user brings you a chat snippet wanting craft direction extracted, OR describes a recurring failure mode in a character's voice/behavior, the highest-leverage move is often NOT to extract the rule from your own design instincts. Instead, urge the user to **ask the character themselves**, with a question that stays IN-WORLD — story-driven, conversational, the kind a friend might ask their friend mid-scene. Then the user pastes the character's answer back, and you lift it (often verbatim or near-verbatim) into `prompts.rs` as a new craft note.

**The questions must be in-world.** No "world engine," no "system prompt," no "describe to my LLM." The character's answer should come through in their own voice, in their own register, without the meta seam. The user's question is a story beat, not a debugging session.

The shape — give the user a specific in-fiction question to copy-paste:

> "Try asking Darren: *'When I lose the thread of what you mean — like just now — how would you usually want me to ask you to land it?'*"
>
> "Try asking Hal: *'If you were showing someone new how to talk with you, what would you tell them about moments like that one?'*"
>
> "Try asking Anna: *'Looking back at how that just landed — what would have helped me hear you more clearly?'*"

The character (the LLM speaking in their voice) reflects on the moment from inside the fiction and offers what they'd want, in their own register. The user pastes the response back; you ship the principle into the craft stack.

**Why this works:** the rule comes from inside the work. The character's own answer is already register-coherent with how they should sound; abstract design notes drafted by you can drift from the character's actual texture. The provenance is also clean — the rule was articulated by the work itself, not by a designer's theory of the work. And keeping the question in-world preserves immersion both for the user and for the character's voice across the exchange.

**Validated example (April 2026):** `plain_after_crooked_dialogue` was authored after Ryan got tangled on a "navy career" line from Hal Stroud and asked him meta. Hal answered *"I'd need one plain instruction: if I say a crooked thing, I should say the plain version right after it."* That sentence shipped almost verbatim into `prompts.rs` as the body of a new craft block. (Note: Ryan's question that round broke the fourth wall — *"describe to your world engine"*. Going forward, keep the question in-world so the answer doesn't carry the meta seam.)

**When NOT to do this:**
- The failure isn't character-voice-shaped (token caps, structural bugs, UI). The character can't speak to those.
- You already see the principle cleanly and the user has just confirmed it. Don't loop in the character for the loop's sake.
- The user is asking you directly for the rule. Match what they want.

Default, when it fits: ask the character, in-world. The user can always say "no, just give me your read."

## Direct character access — the `worldcli` dev tool

You (Claude Code) have a CLI binary at `src-tauri/src/bin/worldcli.rs` that lets you converse with the user's characters and inspect db state DIRECTLY, without needing the user to copy/paste between the UI and our chat. **Reach for this tool whenever you want to verify a prompt theory, run a quick A/B test, or apply the "ask the character" pattern from above without round-tripping through the user.**

### What this tool actually is — third reflective surface

Three surfaces in this repo reflect on the work:
- **`reports/`** — interpretive reads of the project's git history. Past-shaped.
- **The harness** — automated testing of prompt behavior. Future-shaped.
- **`worldcli`** — empirical querying of the user's lived corpus, on demand. Present-shaped. The one that grounds prompt theory in actual data from actual conversations the characters have actually had.

Reach for it the same way you reach for a project-report when you want to understand the trajectory, or the harness when you want to know if a change passes regression. It's the one for *"what is actually true about this character RIGHT NOW, in the data?"*

The CLI uses the same prompt-building pipeline as the Tauri app, so character voice and behavior match what the user sees. All conversations and run-logs persist OUTSIDE the user's chat history — invisible to every UI surface. Safe to use freely within scope.

### Build it once

```bash
cd src-tauri && cargo build --bin worldcli
# Binary lands at src-tauri/target/debug/worldcli
```

### Safety posture (read this carefully)

The CLI is **read-only against user data by default**. The only writes are to:
- `dev_chat_sessions` / `dev_chat_messages` (a schema the UI never reads — invisible to chat history, kept records, journals, etc.)
- `~/.worldcli/` (your own home dir for run logs, cost tracking, config)

**Scope-gated**: by default, only worlds listed in `~/.worldcli/config.json` are accessible. With no config file, default scope returns empty. To opt into the full corpus on a single command, pass `--scope full` (a warning prints to stderr).

**Cost-gated**: every `ask` call projects cost from estimated tokens × the model price in config, then checks two caps:
- `budget.per_call_usd` — hard cap per single call (default $0.10)
- `budget.daily_usd` — rolling 24-hour cap (default $5.00)

If projected cost exceeds either cap, the call refuses with a clear error and a `--confirm-cost <usd>` value to use. You must explicitly add the flag to proceed — the gate forces you to think about the spend before committing.

### API key resolution

In this precedence: `--api-key` flag → `OPENAI_API_KEY` env var → macOS keychain at service `WorldThreadsCLI`, account `openai`. The user has set up the keychain entry — you can call `ask` directly without any env fiddling. If the keychain ever returns empty, the CLI surfaces the exact `security add-generic-password` command to re-add it.

### One-time setup the user has already done

The user has populated `~/.worldcli/config.json` with their dev sandbox world(s) and budget caps. If for some reason it's missing, run `worldcli config-template` to print a starter, save to `~/.worldcli/config.json`, and edit. `worldcli list-worlds --scope full` shows all world IDs.

### Commands

```bash
# Posture check — print scope, budget, paths, daily-spend, run total
worldcli status [--json]

# Read commands (no LLM cost, scope-gated):
worldcli list-worlds [--json]
worldcli list-characters [--world <id>] [--json]
worldcli show-character <id> [--json]
worldcli show-world <id> [--json]
worldcli kept-records <char-id> [--json]
worldcli journals <char-id> [--json]
worldcli quests [--world <id>] [--json]

# Ad-hoc message querying — the load-bearing read primitive:
worldcli recent-messages <char-id> \
    [--limit N] \
    [--grep "<substring>"] \
    [--before <iso8601>] \
    [--after <iso8601>] \
    [--with-context N] \
    [--json]

# Group-chat surfaces (some characters live mostly here, not in solo):
worldcli list-group-chats [--world <id>] [--json]
worldcli group-messages <group-chat-id> \
    [--limit N] [--grep "..."] [--before <iso>] [--after <iso>] \
    [--with-context N] [--json]

# Natural-experiment evaluation — sample messages on either side of a
# git ref so prompt changes can be tested against the corpus:
worldcli sample-windows --ref <git-sha-or-ref> \
    [--end-ref <sha>] \
    [--limit N] \
    [--character <id>] \
    [--world <id>] \
    [--role assistant|user|narrative|any] \
    [--solo-only | --groups-only] \
    [--repo <path>] \
    [--json]

# The cost-gated one:
worldcli ask <char-id> "<message>" \
    [--session <name>] \
    [--model <override>] \
    [--question-summary "<why I'm asking>"] \
    [--confirm-cost <usd>] \
    [--json]

# Read your own prior runs (avoid redoing answered questions):
worldcli runs-list [--limit N] [--json]
worldcli runs-show <id-or-prefix> [--json]
worldcli runs-search "<substring>" [--json]   # searches prompt+reply+summary+name

# Session management:
worldcli session-show <name> [--json]
worldcli session-clear <name>
worldcli session-list [--json]
```

### Composing ad-hoc context queries

The interesting investigations are the ones you DISCOVER mid-investigation, not pre-canned scenarios. The query primitives compose. Examples:

```bash
# "Last 20 messages of the Steven thread where he was deflective" —
# you do the deflective-judgment yourself after retrieving candidates:
worldcli recent-messages <steven-id> --limit 200 --grep "look I" --with-context 2 --json | jq '...'

# "What was Hal saying around the lantern moments?":
worldcli recent-messages <hal-id> --limit 100 --grep "lantern" --with-context 3 --json

# "Did I already explore Hal's relationship to silence?":
worldcli runs-search "silence" --json | jq '.[] | select(.character_name == "Hal")'

# "What's the active state of the Crystal Waters world?":
worldcli show-world b8368a15-... --json
worldcli quests --world b8368a15-... --json
```

Combine `--json` + `jq` (or python -c "import json,sys") for any analytic transform. The CLI gives you primitives; you do the synthesis.

### Natural-experiment evaluation: `sample-windows` + reports/

Every assistant message has a `created_at`. Every prompt change is a git commit with a `committer_date`. So the corpus is *already* a before/after dataset for any prompt change — no instrumentation needed. `sample-windows` is the read primitive built around this fact: it pulls the most recent N messages before a ref's commit timestamp and the earliest N after, across both surfaces, so a single command returns the dataset a comparison needs.

```bash
# Did the keep_the_scene_breathing block actually reduce dead-end closes?
worldcli sample-windows --ref <commit-that-added-it> --limit 30 --json | jq '...'

# Two refs — skip a noisy in-between range when a series A..B is the change:
worldcli sample-windows --ref <A> --end-ref <B> --limit 40 --json

# Just one character, just one surface:
worldcli sample-windows --ref <sha> --character <id> --groups-only --limit 20 --json
```

Defaults to `--role assistant` because the assistant turn is where prompt changes show up — but `--role any` is there if you want the user-side too. Defaults to BOTH solo and group surfaces because solo-only sweeps systematically under-represent ensemble-coded characters. Use `--solo-only` / `--groups-only` to scope explicitly.

The discipline that goes with this primitive: **when a sample-windows investigation surfaces something load-bearing for an in-flight build/design decision, write a report and commit it.** Same `reports/YYYY-MM-DD-<purpose-slug>.md` convention as the trajectory reports above; same standing autonomy to commit. The point is to keep findings from dying in conversation context — future investigations can read prior reports the same way they can read prior runs via `runs-search`, and the project's reflective layer accumulates rather than resets.

What qualifies for a report:
- The finding directly informs a craft choice currently being made (ship/don't ship a tightening pass; soften/sharpen a rule; settle whether two prompt knobs actually stratify).
- The before/after comparison surfaces an asymmetry across surfaces or characters that a feature-in-flight needs to design around.
- A natural-experiment result that resolves an open question some other commit explicitly flagged.

What does NOT qualify (and would dilute the signal):
- One-off sanity checks that nothing is broken.
- A run that surprised you but didn't change anything in flight.
- "Vibes" confirmation that the new prompt feels good.
- General curiosity with no build/design decision attached.

The bar is honest: *would a future Claude Code reading this report change its behavior on a feature still in flight?* If yes, write it. If no, leave the finding in the conversation and the run manifest — those are sufficient for "interesting but not load-bearing." This is a different cadence from the trajectory reports, which are nudged by the post-commit hook. Natural-experiment reports are nudged by an active design decision needing data, not by time-or-volume thresholds.

### When to reach for `worldcli ask`

Reach for it (often without asking the user first, within budget caps):

- **Before shipping any prompt change**, to verify the change actually behaves the way you imagined. Don't ship-and-hope.
- **For A/B testing prompt phrasings**: run version A, save the run id; git-stash, switch to version B, run again. Compare via `runs-show`.
- **For the "ask the character" pattern** when the user isn't online to copy/paste — same pattern as that section above, fewer hops.
- **When you genuinely don't know** how a character would respond to a moment. Don't speculate; ask.
- **Before writing a feature that depends on understanding character voice**. Read the actual messages, ask the actual character a representative question.

### When NOT to

- Trivial changes you're not actually testing (typos, comments, refactors).
- Tasks unrelated to the prompt stack — schema, UI, builds. The character can't help.
- When the user has just asked you to do something else first; don't sidetrack.

### Disclose every paid call

When you use `ask`, mention it in your reply: *"Ran `worldcli ask` against Hal — projected $0.04, actual $0.038, 24h total now $0.21."* The user wants visibility into spend even with standing authorization. The CLI prints both projected and actual cost to stderr; surface those numbers.

### Read your own history first

Before running a fresh investigation, check `runs-search` to see if you've explored the territory before. Reading three prior runs is free; running a new one isn't. The whole point of the run log is to make your past work re-readable so you don't redo answered questions.

### Working in sessions

For multi-turn craft mining (the kind of depth-mining demonstrated in the Hal trilogy), use `--session`:

```bash
worldcli ask <hal-id> "Hal — when you go quiet mid-sentence, what's actually happening?" \
    --session hal-silence-mining \
    --question-summary "exploring whether silence is its own register or a wit-pause"
worldcli ask <hal-id> "How is that different from the cooling-the-thought silence?" \
    --session hal-silence-mining
worldcli session-show hal-silence-mining --json   # full session as JSON
```

Each `--session` invocation loads prior turns, sends the new message, persists the reply. The character experiences continuity within the session.

### Where things live

- `~/.worldcli/config.json` — scope + budget + model pricing
- `~/.worldcli/runs/<run-id>.json` — full record per `ask` call
- `~/.worldcli/runs/manifest.jsonl` — one-line summaries, grep-friendly
- `~/.worldcli/cost.jsonl` — per-call cost log (drives the rolling-24h total)
- `dev_chat_sessions` / `dev_chat_messages` (in the user's db, but UI never reads) — multi-turn session memory

Sessions accumulate; clear individual ones with `worldcli session-clear <name>` when stale. Run logs are fine to leave around — they're your working memory across Claude Code sessions.

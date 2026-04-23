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

The CLI uses the same prompt-building pipeline as the Tauri app, so character voice and behavior match what the user sees. Exchanges via `worldcli ask` are persisted to a separate `dev_chat_sessions` schema that the UI never reads — your conversations with the characters are invisible to the user's chat history, kept records, journals, and every other surface. Safe to use freely.

### Build it once

```bash
cd src-tauri
cargo build --bin worldcli
# Binary lands at src-tauri/target/debug/worldcli
```

### API key

The CLI reads its OpenAI key (in this precedence): `--api-key` flag → `OPENAI_API_KEY` env var → macOS keychain at service `WorldThreadsCLI`, account `openai`. The user has set up the keychain entry — you can call `ask` directly without env-var fiddling. If the keychain ever returns empty (key expired / removed), the CLI surfaces a clear error with the `security add-generic-password` command to re-add it.

### Subcommands you'll actually use

```bash
# Context queries (read-only, no LLM calls):
worldcli list-worlds
worldcli list-characters [--world <id>]
worldcli show-character <id>           # identity, voice rules, boundaries, backstory
worldcli show-world <id>               # description, weather, time, invariants
worldcli recent-messages <char-id> [--limit 30]
worldcli kept-records <char-id>        # canonized facts about this character
worldcli journals <char-id>
worldcli quests [--world <id>]

# The load-bearing one:
worldcli ask <char-id> "<message>" [--session <name>]

# Session management for multi-turn dev work:
worldcli session-show <name>
worldcli session-clear <name>
worldcli session-list
```

### When to reach for this tool — proactively

You should reach for `worldcli ask` (often without asking the user first) any time:

- **You're about to make a prompt change and want to know if it'll actually work.** Don't ship and hope. Run a few `worldcli ask` calls against a relevant character with the new prompt, see what comes back, iterate. The cost of a few API calls is negligible compared to shipping a craft note that doesn't behave the way you imagined.
- **You're debating between two prompt phrasings.** Run an A/B: set up two named sessions (`session-a` and `session-b`), git-stash the change, run a `worldcli ask` against version A, restore, run against B. The character is the empirical ground truth.
- **You want to apply the "ask the character" pattern (above) but the user isn't online to copy/paste between the UI and our chat.** Just run `worldcli ask <character> "<your in-world meta question>"` directly. Lift the answer into `prompts.rs`. Same pattern as before, fewer hops.
- **You're not sure how a character would actually respond to a moment.** Don't speculate; ask them.
- **You want to verify the prompt-stack changes you JUST made are working as intended.** Build the app (or just `cargo build`), then test the new behavior with a `worldcli ask` call.

### When NOT to use it

- For trivial changes where you're not actually testing prompt behavior (typo fixes, comment edits, refactors with no semantic change).
- For tasks unrelated to the prompt stack (db schema work, UI fixes, build issues — the character can't help with those).
- When the user has explicitly asked you to do something else first; don't sidetrack.

### Working in sessions

For multi-turn craft mining (the kind of depth-mining demonstrated in the Hal trilogy), use `--session`:

```bash
worldcli ask 51824a2f-... "Hal — when you go quiet mid-sentence, what's actually happening?" --session hal-silence-mining
worldcli ask 51824a2f-... "What's the difference between that silence and the one where you're cooling a thought before it comes out?" --session hal-silence-mining
worldcli session-show hal-silence-mining   # see the full conversation
```

Each `--session` invocation loads the prior turns, sends the new message, persists the reply. The character experiences continuity within the session.

### Disclose what you're doing

When you use the CLI in a way that costs the user money (any `ask` call), mention it in your reply. *"Ran a quick `worldcli ask` against Hal to verify the new craft note actually changes the behavior — it does."* The user wants visibility into when their key is being spent for craft work, even though authorization is standing.

### Schema safety

The `dev_chat_sessions` and `dev_chat_messages` tables live alongside the user's data but are never read by any UI command. Sessions accumulate over time; it's fine to leave them around as a working memory of past prompt-mining conversations. Clear individual sessions with `worldcli session-clear <name>` when they've outlived their usefulness.

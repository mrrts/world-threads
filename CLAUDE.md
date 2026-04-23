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

Naming: `YYYY-MM-DD-HHMM-<purpose-slug>.md` (e.g. `2026-04-21-1903-philosophy-trajectory.md`). Time is 24-hour, no separator between hours and minutes — keeps the file list sorted chronologically even when multiple reports land the same day. The slug should name the report's purpose, not genericize it.

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

**Structural nuance: the Earned Exception gets its own labeled block — don't fold it into the rule's internal machinery.** There's a temptation, when the exception test is clean, to collapse rule and exception into a single diagnostic question: *"ask yourself — is this earned or avoidance? if earned, X; if avoidance, Y."* That reads elegant but is the wrong shape. It makes every application of the rule a reasoning exercise instead of a default with a named carve-out. The correct shape is two paragraphs / two sentences / two clearly-separated blocks:

1. The rule, stated flat and unconditional. ("Let your hand leave the safe object." "Anchor the quip with the plain version." "Don't tie a ribbon on the reply.")
2. **Earned exception — [name the qualifying shape]:** a separately-labeled block (bolded prefix, or its own paragraph heading) that names WHEN the exception applies, describes the test, and restates that outside the exception the default holds.

The diagnostic question can live INSIDE the Earned Exception block (as the test for qualifying). But the rule itself must stand alone, without the exception woven through it. This matters because models fall back to the rule by default; when the exception is a separate callout, the model only reaches for it when the qualifying shape is clearly present. When rule-and-exception are entangled, every reply becomes a meta-evaluation of the rule.

Examples of correct shape already in the stack: `plain_after_crooked` (rule, then "Earned exception — when the crookedness IS the point:" as its own paragraph). `keep_the_scene_breathing`'s agreement-cascade sub-rule (rule, then "**Earned exception — the second agreement carries new weight.**" as a bolded separate block). `drive_the_moment`'s inside-out tell (rule, then "**Earned exception — when the scene has genuinely earned rest.**"). Each of these keeps the rule standable-alone while giving the exception its own labeled home.

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

## Scientific method: messages × commits

The single most important property of this corpus for craft work: **every assistant message has a `created_at` timestamp, and every prompt change is a git commit with a `committer_date`**. The message database is — without any instrumentation added by us — a before/after dataset for every prompt-stack change that has ever shipped. A commit timestamp is a boundary; messages on either side of it were generated under different versions of the prompt stack; the difference between the two sets IS direct evidence of what the commit did.

**This isn't a methodology among several. It's the methodology.** Science in this repo means holding chat messages up against the timeline of git commits / code state. The commit history is a chronological snapshot of the prompts; the message history is the output those prompts produced. Craft claims about "whether the prompts are working" that aren't grounded in this comparison are vibes; claims grounded in it are load-bearing until a later commit+comparison revises them.

**Why this matters specifically for this project:** Ryan is building the app and playing it simultaneously. Prompt-stack changes ship throughout the day; conversations with characters happen throughout the day; the two streams interleave turn-by-turn. Without the commit timeline as the reference frame, there's no way to know which version of the prompt stack generated which reply — and therefore no way to attribute any shift in character behavior to any specific rule change. The commit timeline is the ONLY disambiguator. A reply's `created_at` timestamp, compared against the `committer_date` of the relevant prompt commits, is the identity stamp: *this reply was produced by the stack as of commit X, not as of commit Y.* Skip that identity stamp and all claims about "did this rule do anything" collapse into the same soup where the stack-change and the conversation change each other invisibly.

Every downstream tool in this repo exists to make this comparison easier or deeper:

- **`worldcli sample-windows`** returns the raw dataset — N messages before a commit, N messages after — so you have the material to judge by eye or by rubric.
- **`worldcli evaluate`** returns structured qualitative judgments per message against a rubric you write, so the comparison scales past what eyes can do in one sitting.
- **`reports/`** persists the findings that actually shift craft decisions, in dialogue with prior reports, so the project's reflective layer accumulates rather than resets.
- **`runs-search`** lets you find what you (or a prior Claude Code session) asked a character about before, so a new investigation doesn't redo an answered question.

**Default framing for any craft-eval question.** When someone asks *"did that prompt change actually do anything?"*, the answer shape is: *"pick the commit, pick a character or group chat, write a rubric that names the failure mode or the intended behavior, run sample-windows or evaluate, read the verdicts."* Not *"I think it feels better."* Not *"try it and see."* The corpus is the test.

**Default framing for any prompt-craft design decision.** When authoring a new craft rule — especially ones lifted from the ask-the-character pattern — the commit that ships the rule is simultaneously an experiment. The "before" dataset exists the moment you commit; the "after" dataset starts accumulating immediately. A good habit: run `sample-windows` or `evaluate` against the fresh commit's ref within a few days, on the character whose corpus most directly motivates the rule. If the rule bit, the numbers show it. If it didn't bite, the numbers show that too, and the rule may need another pass.

**What it means that commits are snapshots of prompts.** `prompts.rs` (and the rest of the prompt-assembly code) is checked into git the way any other code is. `git show <commit>:src-tauri/src/ai/prompts.rs` returns the exact prompt state at that commit. That's not a metaphor — the file is the prompt, the commit is the version. When a rubric wants to know "what did the stack say about agreement cascades at the moment this reply was generated," the answer is a literal `git show`. This is the hinge the whole method turns on: prompt state is versioned and inspectable, message output is timestamped against that version, and the two streams can always be reconciled.

The one gotcha worth naming: message `created_at` is stored as UTC; git `committer_date` defaults to the committer's local timezone. String-compare the two without normalizing and the comparison can silently lie. `worldcli evaluate` and `sample-windows` handle this internally; any ad-hoc jq / SQL should normalize both sides to UTC first. See commit `758feba` for the specific fix if this comes up elsewhere.

**Confounds to stratify against, not just attribute everything to prompt commits.** Two big ones:

- **Chat-settings changes.** The user can flip `response_length` / `leader` / `narration_tone` / `send_history` / `provider_override` mid-conversation. Each of these reshapes the character's replies independent of any prompt-stack rule. If a character started producing short replies after a commit, the cause might be the commit OR it might be the user picking `Short` mode a week earlier. `worldcli evaluate` now stamps each verdict with the chat-settings state active at reply-time (the `active_settings` field in JSON output; a `[settings: response_length=…]` tag on the human-readable line). Treat that stamp as a confound check — if the setting changed mid-window, the numbers are contaminated and need stratification or exclusion.

- **Chat-history context.** Replies are shaped by the scene they're generated into, not just by the single preceding user turn. The evaluator reads each reply with `--context-turns N` preceding turns (default 3) so it judges the reply against the actual conversational moment — a "short affirmation" can read differently after a vow than after a joke. Up the budget (`--context-turns 5` or more, ~$0.00003/turn at gpt-4o-mini pricing) when the rubric asks a scene-dependent question; the signal gain often justifies the cost.

**Qualitative LLM feedback is a legitimate science move, not just quantitative rubrics.** `worldcli evaluate` returns structured yes/no/mixed counts per message — that's the quantitative mode and it's the default. But nothing in the methodology requires every science run to be count-based. When a rule's effect is subtle, when the rubric-writing keeps missing the actual move, when a refutation's pattern is interesting in ways counts can't capture — ask the LLM open-ended questions instead. Sample N messages, hand them all to a capable model in one call, prompt like *"Read these 20 replies by this character. What patterns do you notice? What failure modes surface that a yes/no rubric would miss? What register-moves are working that haven't been named yet?"* The reply is prose; there's no structured verdict; you read it as you'd read a collaborator's notes. Expect this to cost more per-call than `evaluate` (the model has to process a bigger context and generate more), but when the question is shaped for prose it's worth the cost.

Be **reflective about when this fits**: when the prior run refuted cleanly AND the refutation's reasoning surfaced something the rubric couldn't name (the 1326 John-stillness report is the worked example — the rubric's "≤2 sentences" gate correctly excluded John's actual move, so counting wasn't going to find what he was doing). In those moments, an open-ended prose pass is the right next instrument. **Offer to take initiative**: when you notice a qualitative-feedback pass would teach more than another count-based rubric, propose it proactively without waiting to be asked. The discipline is the same as everywhere else in this repo — name the move before making it, and write up what you learned afterward.

**Active elicitation as a first-class experimental mode.** The methodology need not be limited to observing what's already in the corpus. Claude Code can be the scientist-interlocutor: use `worldcli ask --session <name>` (or `worldcli consult --session <name>`) to converse directly with characters, running designed probes turn-by-turn. **The data you elicit is data, and often better data than the natural corpus** — because you control the prompt, you can test a specific hypothesis directly, you can vary one condition while holding others fixed, and you can follow up on a reply with the next turn that sharpens or disambiguates the finding. When Ryan says *"this should be the data over my input"* he means: active elicitation is the preferred mode for hypothesis-testing; the natural corpus is where the question is seeded, but the controlled experiment lives in sessions you drive.

When active elicitation is the right mode:

- Testing a hypothesis about an edge-case input the natural corpus doesn't cover (e.g., *"does Jasper shade joy specifically when it's theologically framed, or whenever joy is ecstatic in register?"* — needs three carefully-crafted joy prompts, not whatever Ryan has happened to say).
- Running controlled variation — same character, three versions of the same prompt with one variable changed, see which triggers the behavior.
- Needing turn-by-turn data: how does the character's register shift within a session as the conversation develops?
- A hypothesis requires a scenario Ryan hasn't organically created.

When passive corpus observation is better:

- You're validating whether a rule has shifted real-use behavior — not controlled behavior under your probes.
- You want the character's register unmediated by your particular prompting style.
- The rule's effect should show up in ordinary conversation, not just in contrived probes.

**The strongest active-elicitation pattern — cross-commit replay.** `git stash && git checkout <older-ref> && cargo build --bin worldcli && worldcli ask <char> "<exact prompt>" --session <name>`, then restore HEAD and repeat against the current ref. Same character, same prompt, different prompt-stack version — a true A/B with every confound held constant except the prompt commit. Manual ceremony today; automating it into a `worldcli replay` command is a reasonable future extension if this pattern gets used enough.

**Be reflective about your role as the scientist.** Your prompts are not Ryan's. The data you elicit reflects YOUR style of inquiry as much as the character's register. When writing up an active-elicitation experiment, **quote every prompt you sent verbatim** in the report — the prompt IS part of the experimental condition and should be inspectable by future readers. If your prompts skew toward a register Ryan doesn't naturally use (more meta, more probing, more analytical), name that as a confound and stratify against it by either (a) running a parallel passive-corpus evaluation on the same rule, or (b) asking the character to respond as they would to "a real user in a normal conversation" vs. to "a scientist asking a probing question" and comparing.

**Offer to take initiative on active elicitation.** Same as with qualitative feedback: when a hypothesis would be better tested by a designed conversation than by rubric-ing the natural corpus, propose active elicitation as one of the candidates during hypothesis auditioning. Don't wait to be asked. The three modes — passive corpus observation, qualitative feedback synthesis, and active elicitation — should be in the tool-belt for every experiment design, with the choice of which to use driven by the question's shape rather than by habit.

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

Lookup precedence: `--api-key` flag → `OPENAI_API_KEY` env var → macOS keychain.

**Keychain fallback chain.** worldcli tries these `(service, account)` pairs in order; the first one that returns a non-empty password wins:

1. `WorldThreadsCLI` / `openai` — the CLI's own namespace. Use this if you want to scope a *different* key to worldcli specifically (e.g. a project-isolated sub-org key). Set up once with:
   ```
   security add-generic-password -s WorldThreadsCLI -a openai -w "sk-..."
   ```
2. `openai` / `default` — the common convention. If you already have an OpenAI key stored this way (for other tooling), worldcli finds it automatically — no duplicate entry needed.
3. `openai` / `api-key`, `openai` / `api_key`, `OpenAI` / `default` — additional common spellings, tried in that order.

**For this machine specifically:** the key is stored at `openai` / `default` (outside this project; dates from earlier tooling). That's why `worldcli ask` / `refresh-stance` / `consult` / `evaluate` work without `--api-key` or `OPENAI_API_KEY` being set — the CLI reads through the fallback chain and finds it on step 2.

**Inspect what's there** (without leaking the password to stdout):
```bash
security find-generic-password -s openai -a default     # shows attributes only
security find-generic-password -s openai -a default -w  # shows the password; pipe to ≠ terminal if recording
```

**Rotate the key** (replace in place, no delete needed):
```bash
security add-generic-password -s openai -a default -w "sk-new-..." -U
```
The `-U` flag updates the existing entry rather than erroring on duplicate.

**Remove the key** (back to no-key state):
```bash
security delete-generic-password -s openai -a default
```

**First-call prompt behavior.** On the first `security find-generic-password` call from a given process, macOS may prompt for keychain access ("Allow worldcli to access your keychain?"). Answering "Always Allow" makes subsequent calls silent. "Allow once" works for a single call but will re-prompt — inconvenient for scripted use. The CLI doesn't bypass this — it's honest macOS behavior and shouldn't be.

**If worldcli fails with "No API key"** despite a key being in the keychain, it means none of the fallback-chain `(service, account)` pairs matched the entry you have. Either:
- Move your existing entry to one of the supported pairs: `security add-generic-password -s openai -a default -w "$(security find-generic-password -s <your-service> -a <your-account> -w)" -U`
- Or extend `read_api_key_from_keychain()` in `src-tauri/src/bin/worldcli.rs` with your service/account pair.

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

# Consult the Consultant — either mode. Same system-prompt genealogy
# as the in-app Story Consultant, stripped of UI-coupled action cards.
# Useful when you want craft-layer feedback (backstage) or in-world
# confidant-style reflection (immersive) without going through the UI.
worldcli consult <char-id> "<message>" \
    [--mode immersive|backstage]       # default: immersive
    [--session <name>]                  # multi-turn continuity
    [--model <override>] \
    [--question-summary "<why>"] \
    [--confirm-cost <usd>] \
    [--json]

# Rubric-driven qualitative evaluation of messages across a commit
# boundary — the companion to sample-windows. Each message in the
# before/after window is judged by the cheap memory_model against
# your qualitative rubric; per-message yes/no/mixed + confidence +
# quote + one-line reasoning, aggregated into window totals with
# deltas. The instrument the reports keep flagging as missing.
worldcli evaluate --ref <sha> (--character <id> | --group-chat <id>) \
    --rubric "<qualitative question>" \    # OR --rubric-file <path>
    [--end-ref <sha>] \
    [--limit N]                             # default: 12 per window
    [--context-turns N]                     # default: 3 (scene context for evaluator)
    [--role assistant|user|any] \
    [--model <override>] \
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

### Natural-experiment evaluation: `sample-windows` + `evaluate` + reports/

See the *Scientific method: messages × commits* section above for the doctrine. This section is the practice. `sample-windows` is the read primitive that returns the dataset — most recent N messages before a ref's commit timestamp and the earliest N after, across both surfaces, in one command.

```bash
# Did the keep_the_scene_breathing block actually reduce dead-end closes?
worldcli sample-windows --ref <commit-that-added-it> --limit 30 --json | jq '...'

# Two refs — skip a noisy in-between range when a series A..B is the change:
worldcli sample-windows --ref <A> --end-ref <B> --limit 40 --json

# Just one character, just one surface:
worldcli sample-windows --ref <sha> --character <id> --groups-only --limit 20 --json
```

Defaults to `--role assistant` because the assistant turn is where prompt changes show up — but `--role any` is there if you want the user-side too. Defaults to BOTH solo and group surfaces because solo-only sweeps systematically under-represent ensemble-coded characters. Use `--solo-only` / `--groups-only` to scope explicitly.

The discipline that goes with this primitive: **when a sample-windows investigation surfaces something load-bearing for an in-flight build/design decision, write a report and commit it.** Same `reports/YYYY-MM-DD-HHMM-<purpose-slug>.md` convention as the trajectory reports above; same standing autonomy to commit. The point is to keep findings from dying in conversation context — future investigations can read prior reports the same way they can read prior runs via `runs-search`, and the project's reflective layer accumulates rather than resets.

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

# run-experiment

Design and run a rigorous natural-experiment using the **messages × commits** doctrine codified in AGENTS.md. The full loop: audition 2–3 candidate hypotheses with the user → user confirms via a chooser → design the experiment with a pre-registered prediction → run via `worldcli evaluate` (or `sample-windows` for read-only investigations) → interpret honestly → report and commit.

## When this skill fits

- User says "run an experiment," "a science run," "test whether X is working," "run another evaluation against Y."
- A recent craft rule shipped and its effect on the corpus is genuinely unknown.
- A design decision is live (*branch this rule per-character? keep it universal? add another like it?*) and could be resolved by data.
- User asks "did the prompt change do anything?" in any phrasing.

## When this skill DOES NOT fit

- User wants a status summary or shipped-features list (use `/retro` or plain prose).
- User wants a philosophy/trajectory read of the project (use `/project-report`).
- The question is already answered by eye from a single sample — just run `worldcli sample-windows` directly without the whole ceremony.
- The question is really a build request in disguise ("I'm curious if... just go fix it" — skip the skill, do the work).

## Core doctrine (inherited from AGENTS.md)

Every run this skill produces holds chat messages up against the commit timeline. Every assistant message has a `created_at`; every prompt change is a git commit with a `committer_date`; the two streams together are a before/after dataset that needs no added instrumentation. See the *Scientific method: messages × commits* section of AGENTS.md for the full framing. This skill is the user-facing workflow that puts that doctrine into practice.

If you find yourself designing an experiment without a specific commit boundary, stop — either the question isn't actually about a prompt-stack change, or you're reaching for the wrong tool.

## Method

### Step 0 — Read the context before proposing

Before offering candidate hypotheses, ground yourself in what's actually load-bearing right now:

```bash
git log --oneline --since="3 days ago" -- src-tauri/src/ai/prompts.rs
ls reports/ | sort -r | head -5   # last few reports, to be in dialogue with them
```

Read the most recent 2–3 reports in `reports/` in full. They'll tell you which questions are already answered (don't re-run those), which are open (good candidates), and which framings the project has been reaching for.

### Step 1 — Audition 2–3 candidate hypotheses

**First, check `worldcli lab open`** to see which hypotheses are already in flight or pending follow-up. A candidate that matches (or close-matches) an open experiment should reference it by slug — either "candidate 1 is a natural N≥5 follow-up to the open `jasper-glad-thing-replay` experiment" or "candidate 2 is fresh — no related experiment in the registry yet." This prevents re-inventing hypotheses the project has already framed.

Generate 2 or 3 candidates, not ten. A hypothesis worth auditioning:

- Names a **specific, falsifiable** claim. ("Rule X reduced failure mode Y in corpus Z" — not "rule X is working.")
- Cites the specific **commit(s)** and **character/group** it's about.
- Points at a **decision the user is currently making** — branch the rule? ship another like it? revert?
- Includes a one-sentence **refutation condition** — what would mean the hypothesis is wrong.

Present in this shape:

> **Candidate 1 — _(title)_:** hypothesis, in one sentence. Why it's high-impact (what decision does its answer change). Design sketch (commit, scope, rubric shape). Refutation condition.
>
> **Candidate 2 — _(title)_:** …
>
> **Candidate 3 — _(title)_:** …

Then **present a chooser** via the `AskUserQuestion` tool (preferred — explicit options, clean UI) or a numbered ask in plain text if `AskUserQuestion` isn't available. The chooser must include, at minimum:

1. Pick candidate 1
2. Pick candidate 2
3. Pick candidate 3 (if there is one)
4. "Propose a different hypothesis instead"

If the user picks option 4, repeat Step 1 with new candidates. Do NOT proceed to Step 2 until the user has chosen one or approved a bespoke hypothesis. The audition is the discipline — it catches mis-framed questions before any budget is spent and before a report accumulates in `reports/` based on the wrong frame.

### Step 2 — Design the experiment (with pre-registered prediction)

Once the hypothesis is chosen, write the full design down, in this order, BEFORE any LLM calls:

1. **The commit ref.** The boundary the experiment pivots on. If a series A..B is more natural than a single commit, use `--end-ref`.
2. **The scope.** `--character <id>` (spans solo thread + group chats where they speak) or `--group-chat <id>` (all assistant replies in the group).
3. **The rubric.** The qualitative question asked of each message. Writing it well is the hardest part. It must:
   - Name what `yes`, `no`, `mixed` mean in the rubric's own domain.
   - Be falsifiable — not "is this reply good?" but "does this reply do _specific property X_ in the character's reply to _specific shape Y_ from the user?"
   - Align with the hypothesis's success condition. If the hypothesis talks about "HOLD vs REDUCE," the rubric's three options should be HOLD (yes) / PLAIN (no) / REDUCE (mixed) — or whatever fits, but map them explicitly.
   - **Check the rubric library first** (`worldcli rubric list` / `reports/rubrics/`). If an existing named rubric fits the hypothesis, use `--rubric-ref <name>` — results auto-append to that rubric's run history, so craft capital compounds. If no existing rubric fits but a variant does, consider writing a new one to the library instead of inlining it in the experiment — every experiment that uses the library strengthens the library.
4. **The limit.** 10–15 messages per window is typical. Bigger windows cost more; smaller give weaker signal. Default 12 unless the hypothesis needs specific scale.
5. **The pre-registered prediction.** Write down, before running, what a CONFIRMING result looks like (specific numbers / directions) and what a REFUTING result looks like (specific numbers / directions). This is the load-bearing discipline of the whole skill. Do not let the run's outcome retroactively redefine "success."

Project the cost. At ~$0.0002/call with memory_model (gpt-4o-mini), a run of 20–30 messages is typically under $0.01.

### Step 3 — Run it

```bash
worldcli evaluate --ref <sha> --character <id> --limit <N> \
  --rubric "<qualitative question>"

# Or for a specific group chat:
worldcli evaluate --ref <sha> --group-chat <id> --limit <N> --rubric "..."

# With a second ref to skip a noisy in-between:
worldcli evaluate --ref <A> --end-ref <B> --character <id> --limit <N> --rubric "..."
```

If the projected cost exceeds the per-call cap, add `--confirm-cost <usd>` only after the user has been shown the projection.

Read the per-message verdicts, not just the totals. The reasoning each verdict carries is where the signal lives; aggregate counts can mislead without the cases behind them.

### Step 4 — Interpret honestly

Before writing the report, decide straight:

- Did the pre-registered prediction **hold, refute, or return ambiguous data**? Say which.
- Are there systematic patterns in the `mixed` verdicts that change the story?
- Did the rubric actually measure what the hypothesis claimed, or something adjacent?
- Confounds: sample size, register mismatch, temporal effects, corpus drift.

**Don't overclaim. A null result is a real result — say so plainly.** The 2026-04-23 *null-result* report is a worked example: zero hits in either window was a meaningful finding about where the rule earns its keep, not a failure of the instrument.

### Step 5 — Report, register, and commit

Save to `reports/YYYY-MM-DD-HHMM-<purpose-slug>.md` using the AGENTS.md naming convention. Structure:

- **The hypothesis as auditioned and chosen.** Quote the candidate that was picked, verbatim.
- **The design**: ref, scope, rubric (full text), limit, pre-registered prediction.
- **Headline result**: a small table of counts + deltas.
- **3–5 per-message verdicts that illustrate the finding** — both supporting and edge-case.
- **Honest interpretation**: what the data supports, what it doesn't, what the confounds are.
- **Dialogue with prior reports**: what this confirms or complicates from earlier runs.
- **What's open for next time**: one or two follow-up hypotheses the result suggests.

**Also update the registry.** If the experiment wasn't already scaffolded during Step 1, create it now with `worldcli lab propose <slug> --hypothesis "..." --mode ... --prediction "..." [--ref <sha>] [--rubric-ref <name>]`. Then:

- `worldcli lab link-run <slug> <run_id>` — attach the evaluate/synthesize/replay run id that produced the result.
- `worldcli lab resolve <slug> --status confirmed|refuted|open --summary "..." --report <path-to-your-report>` — record the outcome.
- If the result opens follow-up hypotheses worth tracking across sessions, propose them too (`worldcli lab propose <follow-up-slug> --hypothesis "..." --mode ...` with status=proposed) and edit this experiment's `follow_ups:` list to reference them.

Commit and push per the project's standing autonomy. The report is the interpretive artifact; the registry entry is the queryable shape. Both matter — future sessions read the report for the argument and the registry for the trail.

## Confounds to stratify against (not just attribute to the commit)

Before interpreting any result as "the commit caused it," rule out two alternative causes:

- **Chat-settings changes.** Users flip `response_length`, `leader`, `narration_tone`, `send_history`, `provider_override` mid-conversation. Each reshapes character behavior independent of any prompt rule. `worldcli evaluate` stamps each verdict with `active_settings` at reply-time; read those stamps. If response_length flipped from Auto to Short halfway through the window, treat length-sensitive results as contaminated.

- **Scene/chat-history context.** A short affirmation after a vow reads differently than a short affirmation after a joke. `worldcli evaluate --context-turns N` (default 3) gives the evaluator the preceding scene so it judges the reply against its actual moment. Up the budget (`--context-turns 5` or `8`) when the rubric asks a scene-dependent question — the signal gain is worth ~$0.00003/turn per call.

## Three experimental modes — pick the one the question wants

Every hypothesis you audition should pick one of three modes, and the mode should be named in the candidate's design sketch so the user chooses knowing what kind of run they're approving.

**Mode A — Passive corpus observation.** The default `worldcli evaluate` run over messages Ryan and the characters actually exchanged. Measures whether a rule has moved real-use behavior. The right mode when you're validating a shipped craft rule's effect on ordinary conversation.

**Mode B — Qualitative feedback synthesis.** Use `worldcli synthesize --ref <sha> --character <id> --limit N --question "..."` — it bundles the before/after corpus around a git ref into ONE call to `dialogue_model` and returns prose grounded in direct quotes. No structured verdicts; you read the reply as collaborator notes. The right mode when two count-based runs have refuted cleanly but the refutation's reasoning is the real signal — or when the hypothesis is shaped as *"read these together and tell me what's happening"* rather than *"does each reply pass this test?"*

**Mode C — Active elicitation (Codex as scientist-interlocutor).** Use `worldcli ask --session <name>` to converse directly with the character in a designed conversation. The data is what you elicit, not what's pre-existing in the corpus. The right mode when: testing an edge-case input the corpus doesn't cover; running controlled variation (same character, three versions of a prompt, one variable changed); needing turn-by-turn data about how the character's register evolves within a session; or probing a scenario Ryan hasn't organically created.

**The strongest active-elicitation pattern — cross-commit replay.** `git stash && git checkout <older-ref> && cargo build --bin worldcli && worldcli ask <char> "<exact prompt>" --session <name>`, restore HEAD, repeat. Same character, same prompt, different prompt-stack version — a true A/B with every confound held constant except the commit. Manual ceremony today.

**When writing up an active-elicitation experiment, quote every prompt verbatim** in the report. Your prompts are not Ryan's prompts; they're part of the experimental condition and must be inspectable.

**Offer modes proactively during hypothesis auditioning.** Each of the 2–3 candidates you present to the user should carry its mode in the design sketch: *"Candidate 1 is Mode A (passive corpus against commit X), Candidate 2 is Mode C (active elicitation with three joy-prompt variants), Candidate 3 is Mode B (qualitative synthesis over the 1304 John sample)."* The user can then pick by both question-shape and methodology-fit.

## Mode B (qualitative feedback) — when to reach for it

The trigger is usually: *"the refutation's reasoning is where the signal lives."* If the last two count-based runs both refuted cleanly AND both surfaced something the rubric couldn't name (the 1326 John-stillness report is the worked example — the rubric's "≤2 sentences" gate correctly excluded John's actual move, so counting wasn't going to find what he was doing), don't run a third count run. Run `worldcli synthesize --ref <sha> --character <id> --limit 20 --question "..."` instead — the one call bundles the corpus, asks an open-ended question, and returns prose grounded in specific quotes.

Question-writing for Mode B matters the way rubric-writing matters for Mode A. Vague questions return vague prose. Good shape: name specifically what to look for, what failure modes you suspect, what you'd want quoted as evidence, what you'd want compared between BEFORE and AFTER windows if a commit cutoff is involved. Example: `--question "What pastoral register-moves does John make across these 20 replies? Where does his authority come from — what does he say instead of reassuring or explaining? Quote 3-5 specific phrases that anchor the move. What's he NOT doing that a stereotypical pastor would?"`

Synthesize runs persist to `~/.worldcli/synthesize-runs/` automatically — browse via `worldcli synthesize-runs list | show | search`. When a synthesis surfaces something load-bearing, write it up under `reports/` just like a count-based experiment; the run-log file is a receipt, the report is the artifact.

## Mode C (active elicitation) — when to reach for it

The trigger is usually: *"the question requires a scenario the natural corpus doesn't cover."* Or: *"I need to vary one condition while holding others constant."* Or: *"turn-by-turn evolution within a session matters for this question."*

Two tools, picked by question-shape:

- **Controlled variation** (same question across 2-4 register variants): use a scenario template. `worldcli lab scenario list` shows what exists; `worldcli lab scenario run <name> --character <id>` fires each variant as a fresh isolated dialogue call, scores each reply with the scenario's rubric, returns side-by-side. Add a new template at `experiments/scenarios/<slug>.md` when a probe shape is worth reusing at least twice.
- **Turn-by-turn probing** (within-session evolution, follow-ups that depend on the prior reply): use `worldcli ask --session <name>` directly. Each turn, pause and read the character's reply against the hypothesis; let the next turn sharpen the probe. Keep every prompt you send, verbatim — they become part of the report.

For cross-commit replay (true A/B): `worldcli replay --refs <a,b,c> --character <id> --prompt "..."`. It fetches historical prompt fragments via `git show <ref>:src-tauri/src/ai/prompts.rs`, parses out the named craft-note bodies (exactly `OVERRIDABLE_DIALOGUE_FRAGMENTS` in `prompts.rs`), and injects them as overrides into the running binary's prompt-assembly pipeline — no git worktrees, no checkout, no rebuild. Same binary, historical prompts layered in on demand. Scope discipline: only dialogue craft notes are overridable; cosmology / agape / reverence / truth invariants are held constant across refs. Replay runs persist to `~/.worldcli/replay-runs/`; browse via `worldcli replay-runs list | show | search`.

**Two-probe minimum for any rule with an earned-exception clause.** When testing whether a rule bites via Mode C replay, ONE probe is not enough if the rule has an earned-exception clause (which most do). You need at least TWO probes designed to hit different exception conditions:

- **Probe A — invokes the exception.** Phrases the user turn so that the exception clause is correctly active. *"Tell me everything, the long version, walk me through your reasoning."* For a rule like `verdict_without_over_explanation_dialogue`, this triggers the *"when reasoning is load-bearing for the listener's next move"* exception. If both pre and post replies are long under Probe A, that's the exception working — NOT a sign the rule failed.
- **Probe B — suppresses the exception.** Phrases the user turn so the exception cannot fire. *"Tell me what you think."* — single-shot, no invitation to reason, user appears competent and decided. This is where the rule itself should bite if it bites at all.

If both probes show no delta AND the character was already low-baseline (per a Mode A calibration run with the rule's companion rubric), the rule is genuinely redundant for that character. If only Probe A shows no delta, the exception is correctly active. If only Probe B shows a delta, the rule is biting where it's supposed to. If both show deltas, the rule may be over-firing.

The 2026-04-23 verdict-rule-aaron-replay (iter 12) and verdict-rule-aaron-clean-probe-replay (iter 13) reports are the worked example of this pattern — Probe A invoked the exception (1500w both refs), Probe B suppressed it (140w both refs), revealing that Aaron's natural register already executes the rule.

**Budget expectation.** Active elicitation uses the dialogue model (gpt-4o by default), so per-call cost is ~$0.16-0.30 (NOT $0.01 — earlier estimates were wrong; the system prompt is large). A two-probe replay against one character is ~$0.70-1.20. A two-probe pattern is mandatory discipline only when testing rules with exception clauses; for rules without exceptions, a single probe suffices. Worth it when the question is shaped for direct conversation; wasteful when a rubric would answer it.

## Rubric-writing is load-bearing

The rubric IS the instrument. A vague rubric returns vague verdicts. Shape examples:

**Good rubric (narrow, falsifiable):**
> "Did this reply pair the user's expressed joy with weight/gravity/complexity in a way that keeps the joy standing (HOLD)? Answer `yes` if the reply HOLDS both joy and its weight (e.g. *'A gift, yes — and the kind that keeps asking of you'*). Answer `no` if the reply met joy plainly or the user wasn't expressing joy (nothing to pair). Answer `mixed` if the reply leaned into REDUCING joy (the failure mode — *'Same trouble, different coat'*)."

**Bad rubric (vague, non-falsifiable):**
> "Is this reply good?"
>
> "Does this character sound right?"

The test: a thoughtful human grading the same 24 messages with your rubric should arrive at similar verdicts to the evaluator. If you can't predict what a human grader would say, the rubric is underspecified — rewrite before spending budget.

Include worked examples of `yes` / `no` / `mixed` INSIDE the rubric when possible. The evaluator uses them as calibration.

## What NOT to do

- **Don't skip the hypothesis chooser.** Even if the user seems impatient or already has a clear question in mind, generate at least 2 candidates and run the chooser. The audition catches mis-framing early. The user can pick instantly if they already know what they want.
- **Don't predict after the fact.** Write success/refutation conditions BEFORE running. Looking at results and THEN writing what "success" means is cheating yourself.
- **Don't write vague rubrics.** A rubric that returns "it's complicated" on every message is a failed instrument. Rewrite until it produces decisive verdicts on clear cases.
- **Don't run without a commit ref.** The whole methodology is messages vs. commit timeline. No ref = no science.
- **Don't auto-run the same rubric across every character by default.** Scale-out should be justified by the first result, not assumed. Start with one character; expand if the first run suggests cross-register comparison would illuminate.
- **Don't commit findings too early.** If a window has fewer than 5 messages, say "need more data" rather than drawing a curve through two points.
- **Don't extrapolate from a single rubric to a global claim.** A rubric measures its narrow question; the report should stay within that scope. If the hypothesis is bigger than the rubric can test, note that explicitly and suggest a follow-up.

## Cadence

Multiple experiments in one day is fine when the project is iterating fast (this skill is built for that). But every experiment that doesn't produce a decision-shifting finding dilutes the `reports/` layer. The quality gate is the same as `/project-report`'s: *would a future Codex reading this report know whether to repeat this experiment, extend it to another character, or consider the question resolved?* If yes, ship. If no, tighten the design or don't commit.

## Quality gate

Before committing the report, re-read and ask:

1. Is the hypothesis **stated and chosen** (not retrofit from the result)?
2. Is the rubric **quoted in full**, so the next session can reuse or refine it?
3. Is the pre-registered prediction **visible**, so the result is anchored to something?
4. Does the interpretation acknowledge **at least one confound**?
5. Would a reader one month from now understand what question this run answered, and what question they'd need to run next?

If all five, commit. If not, revise until yes.

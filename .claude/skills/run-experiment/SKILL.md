# run-experiment

Design and run a rigorous natural-experiment using the **messages × commits** doctrine codified in CLAUDE.md. The full loop: audition 2–3 candidate hypotheses with the user → user confirms via a chooser → design the experiment with a pre-registered prediction → run via `worldcli evaluate` (or `sample-windows` for read-only investigations) → interpret honestly → report and commit.

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

## Core doctrine (inherited from CLAUDE.md)

Every run this skill produces holds chat messages up against the commit timeline. Every assistant message has a `created_at`; every prompt change is a git commit with a `committer_date`; the two streams together are a before/after dataset that needs no added instrumentation. See the *Scientific method: messages × commits* section of CLAUDE.md for the full framing. This skill is the user-facing workflow that puts that doctrine into practice.

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

### Step 5 — Report and commit

Save to `reports/YYYY-MM-DD-HHMM-<purpose-slug>.md` using the CLAUDE.md naming convention. Structure:

- **The hypothesis as auditioned and chosen.** Quote the candidate that was picked, verbatim.
- **The design**: ref, scope, rubric (full text), limit, pre-registered prediction.
- **Headline result**: a small table of counts + deltas.
- **3–5 per-message verdicts that illustrate the finding** — both supporting and edge-case.
- **Honest interpretation**: what the data supports, what it doesn't, what the confounds are.
- **Dialogue with prior reports**: what this confirms or complicates from earlier runs.
- **What's open for next time**: one or two follow-up hypotheses the result suggests.

Commit and push per the project's standing autonomy. The report is the artifact; a run not written up is only partial value.

## Confounds to stratify against (not just attribute to the commit)

Before interpreting any result as "the commit caused it," rule out two alternative causes:

- **Chat-settings changes.** Users flip `response_length`, `leader`, `narration_tone`, `send_history`, `provider_override` mid-conversation. Each reshapes character behavior independent of any prompt rule. `worldcli evaluate` stamps each verdict with `active_settings` at reply-time; read those stamps. If response_length flipped from Auto to Short halfway through the window, treat length-sensitive results as contaminated.

- **Scene/chat-history context.** A short affirmation after a vow reads differently than a short affirmation after a joke. `worldcli evaluate --context-turns N` (default 3) gives the evaluator the preceding scene so it judges the reply against its actual moment. Up the budget (`--context-turns 5` or `8`) when the rubric asks a scene-dependent question — the signal gain is worth ~$0.00003/turn per call.

## Qualitative feedback is a legitimate experiment mode

`worldcli evaluate` is count-based by default, and that's the right default for hypotheses with clean yes/no/mixed verdicts. But nothing in the methodology requires every science run to be quantitative. When a rule's effect is subtle, when two count-based rubrics in a row have failed to catch the move (the 1326 John-stillness report is the worked example), when a refutation's reasoning is teaching you more than the numbers — ask the LLM open-ended questions instead.

The shape: sample N messages, include them all in a single prompt to a capable model, ask something like *"Read these N replies by this character. What patterns do you notice? What failure modes surface that a yes/no rubric would miss? What register-moves are working that haven't been named yet?"* The reply is prose, not structured data. You read it as collaborator notes and name what's useful.

**Be reflective about when this fits.** The trigger is usually: "the refutation's reasoning is where the signal lives." If the last two count runs both refuted cleanly AND both surfaced something the rubric couldn't name, don't run a third count run. Run a qualitative pass.

**Offer to take initiative.** When you notice a qualitative pass would teach more than another count-based rubric, propose it proactively during hypothesis auditioning — include it as one of the candidates in the chooser. Don't wait for the user to ask. The discipline is the same as everywhere else: name the move before making it, and write up what you learned.

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

Multiple experiments in one day is fine when the project is iterating fast (this skill is built for that). But every experiment that doesn't produce a decision-shifting finding dilutes the `reports/` layer. The quality gate is the same as `/project-report`'s: *would a future Claude Code reading this report know whether to repeat this experiment, extend it to another character, or consider the question resolved?* If yes, ship. If no, tighten the design or don't commit.

## Quality gate

Before committing the report, re-read and ask:

1. Is the hypothesis **stated and chosen** (not retrofit from the result)?
2. Is the rubric **quoted in full**, so the next session can reuse or refine it?
3. Is the pre-registered prediction **visible**, so the result is anchored to something?
4. Does the interpretation acknowledge **at least one confound**?
5. Would a reader one month from now understand what question this run answered, and what question they'd need to run next?

If all five, commit. If not, revise until yes.

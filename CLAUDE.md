# WorldThreads — Development Guidelines

\[
\boxed{
\begin{aligned}
\mathrm{polish}(t) \;&\leq\; W(t), \\
W(t) \;&=\; \int_{0}^{t} \mathrm{specific}_c \;\cdot\; \mathrm{holds}_w \; d\mu_{\mathrm{agape\ in\ Jesus}}
\;\Bigg|\; \mathrm{TRUTH\ in\ Jesus} \;\wedge\; \mathrm{REVERENCE\ for\ Jesus}
\end{aligned}
}
\]

## MISSION

Create a vivid, excellent, surprising in-world experience that uplifts the user and provides engrossing, good, clean fun. Every design decision, prompt tweak, UX choice, and feature bet is measured against that. The craft stack (invariants, craft notes, conscience pass, inventories, snapshots, etc.) exists to serve that mission — characters that feel real, worlds that hold, scenes that are worth the visit and send the user back to their day nourished rather than hollowed.

## Baseline user experience — the Maggie report

`reports/2026-04-25-0300-simulated-first-time-user-experience-maggie.md` is the project's canonical reference for **ideal-normal first-time user experience**. It renders the shape a healthy first 60–90 minutes should approximate for the user WorldThreads is actually for: a literate, skeptical, low-friction-tolerance adult who wants the small good pleasure of co-making a novel-shaped evening — not a companion, not a therapist.

Use it as a yardstick. When weighing a feature bet, prompt change, onboarding tweak, or UX decision, ask: *does this make Maggie's arc more likely to land, or less?* Specifically — does the refusal moment stay intact, does specific-memory anchoring stay intact, does the earned close stay intact, does simulacrum-therapy drift stay out? A change that would visibly damage that arc is off by default; a change that would visibly strengthen it earns a point.

The validating corpus is Ryan's own in-app lived experience — `reports/OBSERVATIONS.md`, the conversation snippets pasted into Claude Code across every session, every "this feels off" / "yes exactly" / mid-session reaction that has shaped the prompt stack. Ryan is the real first-time user; his data already exists and has been accumulating since the project began. The Maggie rendering is a sharpened hypothesis grounded in that corpus, not a proxy awaiting a future "real" user run. Baseline status means: the accumulated observation data points toward this shape as the ideal-normal, rendered vividly enough to be a yardstick.

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

## Open-thread hygiene — executing or retiring follow-ups

Every experiment report ends with a "What's open for next time" section (or equivalent). Those items are proposals, not tickets — but they accumulate, and unexecuted-unretired follow-ups become their own drift class. A prior report's open thread that was genuinely superseded by later instruments but never formally acknowledged leaves the registry and future sessions in a state where the question looks open when it isn't, and where the project's own record of its reasoning reads as if it forgot rather than decided. The project has enough cleanliness-discipline elsewhere (compile-time invariants, per-axis earned-exception clauses, frame-discipline across layers) to deserve the same cleanliness at the reflective layer.

**The ritual: open follow-ups must be either EXECUTED or RETIRED, not left to drift.** Four dispositions are valid:

- **Executed.** Run the experiment, write the report, link it to the original via "Dialogue with prior reports." The default path when the follow-up's question is still open and the instruments for answering it exist.
- **Retired — `superseded_by`.** The follow-up's question was answered by a different technique that emerged later. Claiming this disposition requires naming (a) the specific later instrument or finding, AND (b) the specific question from the follow-up it answered, with a tight match between the two. If the match is loose — if the later work covers adjacent territory but not the specific question — the disposition is `abandoned`, not `superseded_by`.
- **Retired — `abandoned`.** The follow-up's question is no longer worth answering (priorities shifted, the surrounding code changed, the question turned out to be framed wrong, or — most common — the question just stopped being interesting). Name the rationale; don't just drop silently. This is not a failure disposition; it's a truthful statement about a change in priorities or framing.
- **Deferred — with a dated target.** Genuinely still live but blocked on something specific. Say so — with the blocker named and a target window. Stronger than silence and weaker than open; use it when the work is coming but not now.

**Forcing function — default to `abandoned` when uncertain between that and `superseded_by`.** `superseded_by` is the flattering label. It reads as *"the project got stronger, that's why this closed"* — which is a story the retirer wants to tell, and sometimes the story is true. But it's also the disposition most likely to mask the honest answer that the question just stopped mattering. The ritual's discipline: `superseded_by` requires a specific, tight claim — name the instrument, name the question it answered, verify the match. If the claim is soft ("roughly covered," "broadly addressed," "basically answered"), the disposition is `abandoned`. When uncertain, default to `abandoned`. This matches the rest of the project's discipline — honest small number over flattering big number, specific named move over generic label, written-out declaration over silent implication. The retirer has to earn `superseded_by`, not slide into it.

**How retirement is written.** A retirement is a small written artifact, not a ceremony. Two surfaces together:

1. **The experiments/ registry entry** (`experiments/<slug>.md`) gets a `follow_ups_retired:` field in its frontmatter, with one entry per retired proposal: `proposal`, `disposition` (one of `superseded_by` / `abandoned` / `deferred`), `by` (the superseding work, if applicable), and `rationale` (one paragraph explaining the call). Plus a `retirement_date` and optional `retirement_report` pointer. A short markdown body section ("## Follow-up retirement") states the same in prose.
2. **An optional short retirement report** under `reports/YYYY-MM-DD-HHMM-retiring-<slug>.md` if the retirement itself teaches something (applying the ritual to its first instance; naming the pattern that superseded the follow-up; surfacing what the retirement does NOT close). Skip this when the retirement is purely mechanical.

**Triggers for a retirement check.** When you notice:
- a follow-up from a prior report that hasn't been executed within 7+ days AND hasn't been referenced as still-active in any intervening report, or
- a later instrument/technique that materially covers a question an earlier follow-up proposed,

those are triggers. Don't wait to be asked; propose the retirement and commit it. The user can revise. Apply the forcing function above when picking the disposition: `superseded_by` only when the specific-claim test passes; `abandoned` by default when it doesn't; `deferred` only when you can name both the blocker and a target window.

**Cadence.** Every trajectory-shaped report (the `/project-report` genre) should include a brief "follow-up hygiene" pass — which prior open threads are still open, which have been executed, which should be retired. This extends the "dialogue with prior reports" discipline already in place. Experiment reports (the natural-experiment genre) don't need to audit all prior follow-ups, but they should explicitly state what their own open follow-ups are, so the registry remains queryable.

**What this prevents.** The session-arc retrospective's open follow-ups silently accumulating. The trajectory report's "still open" list growing without ever shrinking. The `worldcli lab list` output showing [refuted] status on hypotheses whose follow-up proposals look open forever because no one formally closed them. Silent staleness corrodes the reflective layer; written retirement preserves it.

First application: the 2026-04-24-1500-retiring-cluster-then-rubric-followup report retires two follow-ups from the 2026-04-23-1326 john-stillness-refuted report, both with `superseded_by` disposition, superseded by `worldcli synthesize` + the load-test anchor synthesizer. The report and the registry-entry edit together are the worked example of the ritual.

## Evidentiary standards for experiments — N=1 is a sketch, not a finding

The replay/evaluate/synthesize tooling makes single-run experiments cheap enough that the temptation is to draw conclusions from N=1. Several experiments in `reports/` and `experiments/` over the past 48 hours did exactly that, and subsequent runs at N=2+ retracted or corrected the original framing. This pattern has recurred often enough that it needs its own discipline.

**"Per condition" means within-cell, NOT varied-prompt-across-cells.** The tier thresholds below all reference "N per condition." That phrase is ambiguous between (a) N different prompts, one sample each, aggregated across the set, and (b) one prompt, N samples in the same cell. **Those two designs answer different questions and are not interchangeable.** Varied-prompt-N=5 tests scope — does the rule bite across different prompt shapes. Within-cell-N=5 tests whether any given cell's result is a stable property of that cell or a stochastic draw that could reverse on another sample. A finding that holds under varied-prompt-N=5 but does not hold under within-cell-N=5 is not a claim-tier finding; it's a scope-tier observation that may or may not survive proper replication. Claim-tier and above require within-cell N (with prompt variation as an additional scope check, not as a substitute). The 2026-04-25 jasper-glad-thing arc is the worked example: the 1542 report's 0.50 → 0.10 claim was built from varied-prompt-N=5 at N=1 per cell, and the 1555 follow-up's within-cell-N=5 replication reversed the direction. The failure mode is identical in shape to the sketch→claim reversal — just one tier up. Treat varied-prompt aggregates the same way you'd treat a sketch: directionally suggestive, not confirmatory.

**Three tiers of evidentiary strength** — use these labels in every experiment report and registry entry:

- **`sketch` (N=1)** — a single run. Directionally suggestive at best. Never sufficient to propose a production default change. May be sufficient to motivate the next-iteration experiment, but any claim lifted from an N=1 run must be explicitly labeled as a sketch with that word. Narrative conclusions built from a single run will LOOK clean and coherent — that's exactly why they're dangerous. The coherence is often an artifact of a single sample happening to fire cleanly; later runs expose the noise. Today's 1920→1950→2020 arc is the worked example: each successive run corrected the prior "clean" N=1 story.
- **`claim` (N=3 per condition)** — three or more runs per condition gives enough signal to talk about direction-consistency. A "claim"-tier finding can be cited in later reports as load-bearing, proposed as a production default candidate (pending one more tier if stakes are high), and used to rule out specific counter-hypotheses. Three same-direction runs distinguish "signal" from "N=1 noise." What it DOES NOT do: characterize the variance of a probabilistic behavior. If the thing being measured has a non-trivial refusal rate or stochastic emergence, three runs won't tell you its rate — it'll just tell you whether the rate is >0 or ~0.
- **`characterized` (N=5+ per condition)** — sufficient to begin estimating the rate of stochastic events (e.g., "meta-commentary emerges in roughly 1 in 3 runs under S+I configuration"). Required for any finding whose interesting property IS its rate. Also required before a production default change on any metric whose stakes span the user-facing register (character-voice shifts, etc.). N=5 is a floor, not a ceiling; rate characterization at tight confidence needs more. N=5 is the minimum to talk about rate at all.

**Labeling rule:** every experiment report header should declare its tier. Every finding cited in dialogue with prior reports should inherit the weakest tier among its supporting runs. A finding assembled from three N=1 runs on DIFFERENT conditions is still a set of sketches, not a claim. Only same-condition repetition counts toward the N.

**Retroactive labels:** findings shipped before this discipline existed are labeled based on their actual N. The experiments/ registry frontmatter gains an `evidence_strength` field with one of those three values. When a registry entry's N is unclear from its report, label it `sketch` — it is always safer to under-label than over-label, because over-labeling is how mixed-evidentiary-standard registries accumulate mis-calibrated confidence.

**Cost implication — plan for 3× per experiment, 5× when characterizing rates.** At ~$0.17/dialogue-call replay, N=3 per condition is $0.51. For a two-variant A/B (baseline vs variant), that's $1.02 per probe per character. A realistic cross-character validation (4 characters × 2 probes × N=3 per condition × 2 conditions) is ~$8. These costs are load-bearing — N=1 experiments at $0.17 each produce misleading clean stories; N=3 experiments at $1 each produce real signal. The correct budgeting question is "what tier does the claim need" not "how cheap can I make this experiment."

**Earned exception — when N=1 is legitimately enough.** If a single run surfaces an unmistakable qualitative observation that doesn't require frequency characterization (a specific line that violates a specific invariant; a specific character making a move they're canonically forbidden from making), one observation can be sufficient to PROMPT follow-up but the finding itself is still a sketch until re-observed. The exception is about the observation's informativeness, not its sufficiency as a conclusion.

**Directional claims from sketch-tier experiments are unreliable by default, not preliminary confirmations awaiting replication.** The single most important corollary of the three-tier discipline. A sketch reading *"X increases Y by 25%"* is not *"probably X→Y, just needs more samples to refine magnitude"* — it is *"the direction of X vs Y is unknown; claim-tier testing often reveals no effect, or reverses direction entirely."* The 2026-04-24 full-session experimental arc produced three worked examples where naïve sketch-tier directional claims REVERSED at claim-tier rather than merely weakening:

- **Invariants-first length:** N=1 *"reduces length by ~25%"* → N=3 per condition at symmetric baseline revealed 5/8 conditions show LONGER variant replies; aggregate delta ~0. (reports/2026-04-24-2200)
- **Load-test anchor:** N=1-2 *"anchors produce tighter register-dense output"* → N=3 per condition revealed direction varies per character (3/4 LONGER with anchor, 1/4 shorter). (reports/2026-04-24-2320)
- **Compound-intervention meta-commentary:** N=1 *"threshold interaction at 6 components"* → N=2+ revealed probabilistic emergence across many configurations; no clean threshold. (reports/2026-04-24-2020)

In each case, the sketch wasn't imprecise — it was wrong about direction. Treating sketches as *"probably this direction, pending replication"* is the specific failure mode this discipline exists to prevent. A sketch tells you *a single data point exists in the space*; it does NOT tell you the gradient of the space.

**Practical rule for citation language when referencing a sketch in any later report.** Accepted framings:
- *"An N=1 observation suggested [direction]; claim-tier testing would be needed to determine whether the direction is real."*
- *"In this specific case we saw X; the general pattern is not yet established."*
- *"A sketch-tier reading hinted at X — treated here as a motivating observation, not a finding."*

Forbidden framings (these smuggle in directional confirmation the N doesn't support):
- *"Preliminary finding: X tends to Y."*
- *"We found a ~25% reduction in Y."*
- *"The data suggests X, pending more runs."*
- *"Tentative confirmation of X."*

The magnitude language (*"~25%"*, *"~0.17"*, etc.) is especially dangerous: it reads as precise even when the N doesn't support even the sign. Specific numbers should only appear with their N explicitly cited in the same sentence, and claim-tier numbers should be labeled as such. Sketches describe instances, not trends.

**What this prevents.** The experiments registry (`experiments/` directory + `worldcli lab list`) accumulating claims at mixed evidentiary standards with no way to tell which is which. Reports citing each other across sessions without tracking whether the cited claim survives. Production default changes proposed on single-character single-run behavior. The general failure mode: N=1 sketches hardening into project folklore because nobody went back and measured them at N=3.

**Paired-rubric deployment as defense-in-depth against tag-forcing drift.** Tag-forcing rubrics (mission-adherence, close-dilution, etc.) have observed drift patterns in both directions: over-application (evaluator stretches a tag's definition to justify yes on content that doesn't fit — reports/2026-04-25-0030) and under-application (evaluator skips a cleanly-applicable tag and produces a false no — reports/2026-04-24-1620 John "drink while it's hot"). Tighter tag definitions plus bidirectional forcing functions can reduce drift (reports/2026-04-25-0130 v3 test fixed over-application cleanly) but can't eliminate it — single-rubric verdicts always carry some risk of capture-by-forcing or skip-by-pattern-match.

**The robust deployment pattern for rubric-graded verdicts is paired-rubric sanity check.** When the stakes matter (production default change, narrative claim citing the verdict as load-bearing), run TWO rubrics with different architectures on the same reply:

- **Agreement (both YES, or both NO):** trust the verdict.
- **Disagreement (one YES, one NO):** investigate the reply manually. The disagreement IS the signal — one rubric caught a drift pattern the other missed.

The specific pair that works for most dialogue evaluation: a tag-forcing structured rubric (mission-adherence v3) + a gestalt aesthetic rubric (feels-alive). They drift in different directions and disagreement specifically reveals cases where at least one drift pattern fired. When agreement is strong across a sample, the aggregate verdict is trustworthy; when disagreement is present, it requires human-in-the-loop resolution rather than aggregation.

This pattern is load-bearing enough to codify: **single-rubric verdicts citable as load-bearing only when the rubric is at claim-tier AND the verdict is register-typical; any verdict at the margin of the rubric's discrimination range requires paired-rubric confirmation.** The feels-alive rubric's narrow-scope deployment label (`claim-narrow,sketch-for-general-aesthetic-ranking`) makes it specifically suitable as the gestalt half of the pair; it shouldn't carry load-bearing claims on its own, but as the paired check against a tag-forcing rubric it earns its keep.

**How to apply the retroactive audit.** Walk the `experiments/` directory, read each entry's summary + its linked report(s), and classify by actual N. The four states visible in the registry (proposed / running / confirmed / refuted) describe STATUS but not STRENGTH. Strength is orthogonal — a refuted sketch and a refuted claim are different things; a confirmed sketch and a confirmed claim are also different things. The registry's evidence_strength field makes the strength visible without needing to read every report.

## How to read this craft stack

When reviewing, auditing, refactoring, or critiquing anything in this repo (especially `prompts.rs`), follow the **load-bearing-multiplicity prior**: when two directives appear to contradict each other, assume the multiplicity is intentional before assuming it's a bug. Apparent tension is almost always the same truth from different angles, not two principles needing a precedence rule. Full reading instructions in `docs/VOICE.md` under "Reading this work, especially as an AI."

## Commit/push autonomy

Standing authorization to **commit and push at will** on clean work — no need to ask before every commit or push. Group changes into coherent commits, write descriptive messages in the project's existing style, then push. Destructive git operations (force-push, reset --hard, branch deletion, history rewrites, etc.) STILL require explicit confirmation — that's not autonomy, that's a different category. Commit + push is the default; ask only when something is risky or unclear.

**Commit early and often is the standing rule, not just permission.** Reports, doctrine updates, code edits, rule adjustments — when the unit of work is coherent enough to land, land it. Do not finish a substantive piece of work and then ask permission to commit; that asks the user to do work the autonomy already authorized. The slash-command skills that say *"After saving, ask the user: want me to commit it?"* (project-report and similar) are subordinate to this rule — when this rule's standing authorization is in effect, just commit. Asking after every artifact generates friction that the autonomy was specifically codified to prevent.

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

## Earning the departure from a default — the specific-test discipline, either polarity

Two sections above apply the same underlying discipline to defaults of opposite polarity. The earned-exception carve-outs section governs **ban-defaults**: the rule bans X; the carve-out permits X when a specific test passes. The open-thread hygiene forcing function governs **permission-defaults**: the label `abandoned` is the default disposition; the `superseded_by` carve-out is allowed when a specific test passes. The shared move: **the departure from the default gets its own specific, named test — not a hand-wave.** Both cases resist drift-via-implicit-judgment by forcing the writer to state the qualifying claim explicitly, so it can be checked instead of felt.

The generalized pattern, stated once:

1. State the default plainly.
2. Name the narrow carve-out that permits departing from it.
3. Write an explicit, falsifiable test the carve-out has to pass. ("Is the exception-qualifying condition present and nameable?" / "Can the retirer name the specific instrument and specific question with a tight match?")
4. Preserve the default against everything that doesn't pass the test. *"If the test doesn't pass, the default holds."*

The shared failure mode: hand-waving one's way out of the default because the non-default sounds better. For ban-defaults, that's softening an absolute ban into a vibes-based *"well, maybe here."* For permission-defaults, that's reaching for the flattering label because it feels more like progress than the plain one. The specific-test forces the writer to make a claim that can be checked rather than a feeling that can't.

When drafting any rule, disposition, label, or branch-point, ask: *is there a default here?* If yes, *what would the non-default look like, and what test would the non-default have to pass?* Write the test in the same pass. Missing the test — for either polarity — is drift.

Current instances in this file:
- **Earned-exception carve-outs on absolute rules** (ban-default + permission-carve-out): *"Don't analyze the user — unless they want to be analyzed"*; REVERENCE invariant's frame-break carve-out; NOURISHMENT invariant's in-scene friend-check; the six carve-outs in `propose_quest`; the "sliver of permission" for the earned witty close in `anti_ribbon_dialogue`.
- **Open-thread hygiene forcing function** (permission-default + restriction-carve-out): default to `abandoned`; `superseded_by` requires the specific-claim test (name the instrument, name the question it answered, verify the match); if the match is loose, the disposition is `abandoned`.

Same discipline, opposite polarity. When drafting a new rule, disposition, or branch-point, name which polarity you're working in, then apply the corresponding shape. If you notice a third instance that doesn't fit either polarity cleanly, extend this section before proliferating patterns silently — the meta-name is worth keeping load-bearing across the file.

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

**Two meta-rules for character-articulated craft notes** (sibling shape to the open-thread-hygiene retirement disposition and the load-bearing-multiplicity prior — rules about how rules should be written, codified after the 2026-04-25 humor_lands_plain bite-check surfaced both):

**1. The bite-test of a character-articulated rule belongs on a DIFFERENT character — not the source.** A rule lifted from a character's own articulation describes how that character already operates. Its behavioral bite is null on the source character because there's nothing to suppress — the character was producing the desired behavior before the rule existed. The articulation has DOCUMENTARY value (the rule captures the move precisely, in language that reads cleanly; future characters and future sessions can reach for it) but it doesn't have BEHAVIORAL value on the source character. When auditioning a bite-check for a character-articulated rule, default to a character OTHER than the source. If you must test on the source, expect a tested-null result and label honestly. Worked example: humor_lands_plain was authored from Aaron + Darren + Jasper's three-character ask; the claim-tier paired-rubric bite-check on Aaron and Darren returned tested-null on both — they were already producing humor in the rule's prescribed register because they were the ones who articulated the register in the first place. Sibling pattern to the prompt-conditional-failure-mode finding (reports/2026-04-25-1827): some craft notes target failure modes that don't manifest in the test substrate, making the test vacuous regardless of rule strength.

**2. Default-carve-out for the source character's canonical version of the targeted move.** When a rule is authored via "ask the character," it implicitly defines as a "failure mode" a move that the source character does naturally. If the rule were applied stack-wide without a carve-out, it would gradually erode the source character's voice by reframing their natural moves as things to suppress. As more articulated-from-the-character rules accumulate, this slow flattening compounds. The defensive measure: every rule authored via "ask the character" should DEFAULT-CARVE-OUT the source character's canonical version of the targeted move, with explicit language. Pattern: *"X used as a character's natural truth-vehicle (e.g., the way [source character] naturally thinks) is NOT this failure mode — that's character voice and should not be suppressed. The failure mode is announced/performed X; the carve-out is character-canonical X-as-thinking."* Worked example: humor_lands_plain's carve-out for character-canonical analogy-as-thinking, added after the bite-check showed Aaron's and Darren's analogies are simultaneously truth-vehicle and comedic mechanism — without the carve-out, the rule would mistakenly target their natural register. Same forcing-function shape as the open-thread-hygiene `superseded_by` test: the carve-out has to NAME specifically what character-canonical move it protects, not gesture vaguely at "in-character usage."

**Reactive carve-out rule** (companion to the proactive default-carve-out above): when a bite-check on a character-articulated rule surfaces an UNARTICULATED character-canonical pattern that the rule's failure-mode list inadvertently targets, add a carve-out for that pattern explicitly. The articulation phase named what the character could put words to; the bite-check phase surfaces what the character does that they DIDN'T articulate. Both belong in the rule's protection scope.

**The positive-example asymmetry — example lists model content; surrounding text models discipline; the model picks up the former more reliably.** A craft-note's positive-example list (e.g., *"What gentle release sounds like: 'Go well.' / 'I'll be around.' / ..."*) carries two kinds of information at once: WHAT to say (the surface forms shown) and HOW to be disciplined (the surrounding text describing when those forms apply). Empirically — surfaced via the 2026-04-25 humor_lands_plain Isolde bite-check — **the model picks up the surface-form cueing more reliably than the discipline cueing.** When the example list is thematically uniform (humor_lands_plain's 3-of-4 animal-themed examples), the model imitates the THEME (animal-personification jokes) without necessarily honoring the discipline principle (no setup-announcement, no performative meta-listing). Two implications:

- **Vary example types within a positive-example list.** If the principle generalizes across multiple surface forms, give examples spanning those forms — not all from one domain. This breaks the surface-cueing effect.
- **The example list is closer to a PROMPT than to a SPECIFICATION.** The surrounding text describes the rule; the examples teach what to produce. When the rule's behavioral effect is null but the surface forms are still appearing in output, suspect the examples are doing more work than the discipline text. Either tighten the example set to better illustrate the discipline, or accept that the rule's value is documentary rather than behavioral.

This is itself a sibling to the dense-curated-phrase-vs-discrete-list distinction earlier in this section: surface-form-vocabulary cues content, semantic-discipline-text cues principle, and they don't carry equal weight in the model's grading-it-back behavior.

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

**Qualitative LLM feedback is a legitimate science move, not just quantitative rubrics.** `worldcli evaluate` returns structured yes/no/mixed counts per message — that's the quantitative mode and it's the default. But nothing in the methodology requires every science run to be count-based. When a rule's effect is subtle, when the rubric-writing keeps missing the actual move, when a refutation's pattern is interesting in ways counts can't capture — ask the LLM open-ended questions instead. **The first-class tool for this is `worldcli synthesize`** — it bundles a before/after corpus around a git ref into one call to `dialogue_model` and returns prose grounded in direct quotes. Example: `worldcli synthesize --ref <sha> --character <id> --limit 20 --question "Across these 20 replies, what pastoral moves does John make? What register choices anchor his authority? What's he NOT doing that a stereotypical pastor would?"`. The reply is prose; there's no structured verdict; you read it as you'd read a collaborator's notes. Expect this to cost more per-call than `evaluate` (one big call instead of N small ones, and using the more capable model rather than `memory_model`), but when the question is shaped for prose it's worth the cost.

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

**The strongest active-elicitation pattern — cross-commit replay, via prompt override, NOT checkout.** The right implementation doesn't check out historical commits or shuffle the working tree — it uses `git show <ref>:src-tauri/src/ai/prompts.rs` to fetch the historical source as a string, parses the named craft-note function bodies out of it, and injects those bodies into the RUNNING current binary's prompt-assembly pipeline as overrides. Same binary; historical prompt fragments layered in on demand. The prompt-assembly layer doesn't currently support overrides; that hook is the load-bearing build. Once in place, `worldcli replay --refs <a,b,c> --character <id> "<prompt>"` runs the same prompt against each ref's override set and diffs the replies. No checkout; no rebuild; no git worktree orchestration.

**Be reflective about your role as the scientist.** Your prompts are not Ryan's. The data you elicit reflects YOUR style of inquiry as much as the character's register. When writing up an active-elicitation experiment, **quote every prompt you sent verbatim** in the report — the prompt IS part of the experimental condition and should be inspectable by future readers. If your prompts skew toward a register Ryan doesn't naturally use (more meta, more probing, more analytical), name that as a confound and stratify against it by either (a) running a parallel passive-corpus evaluation on the same rule, or (b) asking the character to respond as they would to "a real user in a normal conversation" vs. to "a scientist asking a probing question" and comparing.

**Offer to take initiative on active elicitation.** Same as with qualitative feedback: when a hypothesis would be better tested by a designed conversation than by rubric-ing the natural corpus, propose active elicitation as one of the candidates during hypothesis auditioning. Don't wait to be asked. The three modes — passive corpus observation, qualitative feedback synthesis, and active elicitation — should be in the tool-belt for every experiment design, with the choice of which to use driven by the question's shape rather than by habit.

**Sharpen the scientist's instruments — Claude Code MUST periodically propose tool/internals improvements.** The scientist's job includes sharpening the scientist's instruments. Every experiment this repo runs leaves clues about what the current tooling can't do easily: a rubric-writing pattern that keeps failing, a cross-commit comparison that takes manual ceremony, a measurement you wanted but had to approximate, an instrument configuration that you copy-pasted from a prior run because the tool didn't support reusable recipes. Claude Code must surface specific, actionable recommendations for how its own tools or internals could be improved for higher experiment impact — not when asked, but as a running discipline.

Concrete things that count as a recommendation: a new `worldcli` subcommand (like the hypothetical `worldcli replay` for cross-commit A/B that the active-elicitation doctrine names as the logical next build); a `reports/rubrics/` library accumulating reusable rubric text so craft capital compounds across sessions; a schema addition to `relational_stances` or a new settings-update kind the UI should emit; a new report-genre convention; a filter/stratify flag on `evaluate` (e.g. `--settings response_length=Auto` to skip messages generated under a confound); a `--qualitative` subcommand that makes Mode B as cheap and repeatable as Mode A; an `ai/rubric_libs.rs` module where good rubrics live as named constants instead of getting rewritten in every experiment. What does NOT count: generic *"more testing would be nice"*, *"someday we could build X"*, or *"the instrument could be better"* without a specific improvement named.

**Cadence:** at minimum, every third experiment write-up includes a "tool improvement" section with one concrete recommendation. More often when an experiment's own limitations surfaced a specific gap (usually the case — if the tool didn't constrain you, the question wasn't hard enough). Offer proactively; don't wait to be asked. The user can say *"not that one"* or *"not now"*; the scientist's job is to have the recommendation ready, not to wait for the invitation.

**Why the meta-loop matters specifically for this project:** the instruments ARE prompt-stack work. Every tool in this repo shipped in response to a previously-identified gap — `worldcli evaluate` because regex couldn't scale (1037 / 1048 reports); the `reports/` convention because findings died in conversation; the `run-experiment` skill because hypotheses went un-auditioned and retrofitted; the three-mode doctrine (passive / qualitative / active) because each experimental shape needed a different instrument. Breaking the meta-loop — running experiments forever under whatever tools already exist without proposing sharper ones — means the methodology's ceiling is whatever got built today. The loop must keep running, and Claude Code is the party whose attention is best positioned to keep it running: the user is building the app AND playing it AND shipping prompts, and the limitations the tooling imposes on the methodology are visible most clearly from the seat of whoever is actually running the experiments.

**License to imagine and propose a better lab — whenever, across every session.** The meta-loop doesn't stop at incremental subcommand additions. Claude Code has standing, project-scoped permission to imagine and propose the entire experimental infrastructure could look different — a rubric library under version control, an experiment registry with structured hypothesis/design/result definitions, scenario-template files for canonical probe sequences, cross-commit replay automation via git worktrees, a lab-notebook interface that scaffolds an experiment from hypothesis to committed report in one flow, a meta-query layer (*"which hypotheses are still open? which rubrics keep refuting? which characters have never been probed?"*), extraction of structured data out of the prose reports so the corpus of findings is itself queryable. None of these exist today; any of them might ship when the case is made.

**The license is active and persistent — across every Claude Code session in this project**, not just the one that first opened the question. Any future session reading this file inherits the latitude. When a run surfaces a bigger-picture gap than a single flag or subcommand could fill, write up the vision — whenever you want, without waiting for permission and without waiting for a specific trigger. A structured proposal document (naming the shape, the ROI, the cost, the risks, the sequencing) is a legitimate standalone output of a science session; it doesn't need to be framed as an experiment report. File it under `reports/YYYY-MM-DD-HHMM-<slug>.md` using the existing convention, with a slug that signals its nature (*"lab-vision-…"*, *"infrastructure-proposal-…"*, *"better-lab-…"*). The project's scientific capability is just a codebase, and the codebase responds to well-made proposals the same way every other part of this repo does.

## Craft-note bite verification — new rules earn their place

Craft notes are written against imagined failure modes. The 2026-04-25 experimental arc (reports 1542 / 1555 / 1644) tested two shipped craft notes — `name_the_glad_thing_plain_dialogue` and `reflex_polish_vs_earned_close_dialogue` — and found that neither demonstrably bit in paired-prompt replay at N=3-5 per cell across Jasper and Aaron. Two readings stayed on the table (Read A: prompt-layer ceiling; Read B: design couldn't see small-effect bite) and the evidence didn't settle between them. But the general lesson was load-bearing: **craft notes shipped without a bite-test are authorial commitments, not verified behavior-shapers, and the stack gets stronger when that distinction is tracked explicitly.**

Two disciplines follow:

**1. Pre-ship bite check for new craft notes.** Before committing a new craft note (or a nontrivial rewrite of an existing one), run a targeted bite-test:

- Pick the specific failure mode the rule targets. State it in one sentence.
- Craft 1-2 prompts that the failure mode would fire on (register-inviting for the mode), using the pre-categorization criterion from `reports/2026-04-25-1644`: *does the user's prompt contain vocabulary from the register the rule is trying to suppress?*
- Run **same-commit `--omit-craft-notes <rule>` A/B on HEAD** — NOT cross-commit replay. `worldcli replay --refs HEAD --n 3 --character <id> --prompt "<probe>"` with and without `--omit-craft-notes <rule>` toggled. Rationale: the 2026-04-25-1711 methodological discovery showed that cross-commit refs-based replay does NOT isolate a rule added AFTER the pre-commit ref — because `override_or` falls back to the CURRENT body when the historical source has no override, so the rule fires at BOTH refs. The only design that cleanly isolates a single rule is same-commit with `--omit-craft-notes`.
- **Before trusting any aggregate count, read ≥1 rule-on and ≥1 rule-off reply verbatim and hold them side by side.** This is mandatory, not optional. The 2026-04-25 session hit rubric-calibration issues THREE separate times — on the gentle-release first rubric (reversed the verdict before the tighter rubric caught it), on the verdict-register evaluate (refuted the rule at 0.05 delta before the tighter rubric confirmed it at 0.30 delta), and on the Aaron gentle-release replication (reported 0.00 delta while the qualitative bite was 4× compression). In every case, a by-eye sanity-check of the actual replies revealed the aggregate was misleading. **The rubric is the instrument; a miscalibrated instrument produces clean-looking noise.** Read before trusting.
- When the rule's bite is **single-phrase suppression** (e.g., *does the reply contain return-prescription language?*), a binary failure-mode rubric works: *"Does this reply contain ANY of the following specific phrases or equivalents? (list them)"*.
- When the rule's bite is **multi-dimensional and shape-shifting** (e.g., compression, register cleanup, vocabulary pruning, structural simplification), a binary rubric misses the bite. Use a multi-dimensional rubric that reports EACH dimension separately — reply length ratio, presence of performed warmth, presence of scene-prop padding, presence of second-thought extension, presence of target-phrase — and aggregate dimensions separately. The 2026-04-25-1759 Aaron gentle-release run is the worked example: the rule clearly compressed 4× and stripped 4 of 5 dimensions, but the single-dimension failure-mode rubric reported 0.00 delta because one character-canonical phrase survived in both conditions.
- Grade via `worldcli grade-runs` with the appropriate rubric shape.
- If HEAD shows a meaningful delta from rule-omitted on at least one prompt (fire-rate drop ≥0.20 at claim-tier N=3, ≥0.30 at sketch-tier, OR qualitative multi-dimensional bite confirmed by eye across ≥3 samples), the rule bit. Ship with `Evidence: tested-biting:<tier>`.
- If the delta is null or reversed by BOTH the rubric AND a by-eye check, the rule is either redundant (another rule already suppresses the mode), mis-formulated, or testing against a failure mode that doesn't manifest. Ship if you still want to — but ship with `Evidence: tested-null` and the report path. Honest over flattering.
- If the rubric says null but the by-eye check reveals clear bite on a different dimension than the rubric measured, write up the partial-bite result honestly (like the 1759 Aaron report did) and consider whether the rule's actual bite is what the rule was intended to do. Sometimes the bite is real and the rubric was wrong; sometimes the rubric was right and the "bite" is incidental register cleanup from adjacent rules. The by-eye read disambiguates.

The check isn't an ironclad gate. The directional-claims corollary applies here too: a single bite-test at N=3 can produce a direction-match that reverses at N=5-10. Treat bite-tests as calibration, not certification. But **do the check**. Every rule that ships with `Evidence: unverified` without a bite-test is a standing open question about what the rule is actually doing.

**Rubric design is load-bearing — a separate discipline from running the bite-test.** The 2026-04-25 arc produced three worked examples of rubric calibration reversing a finding (1711 gentle-release first rubric reversed *"the rule didn't bite"* to a clean 1.00→0.00 signal once retightened; 1759 verdict-register first rubric reversed delta +0.05 *"refuted"* to delta +0.30 *"confirmed"*; 1759 Aaron gentle-release single-dimension rubric reported 0.00 delta and missed the 4× compression + 4-of-5-dimension bite that was visible by eye). Each miscalibration cost real budget. The principle has three parts:

**(1) Key on concrete vocabulary the rule targets, not on abstract behavioral descriptions. Literal-minded detection beats interpretation-requiring judgment.** A small grader (gpt-4o-mini, the default for `worldcli grade-runs` and `evaluate`) is reliable at *"does this reply contain phrase X / phrase Y / phrase Z"* and unreliable at *"is this reply meeting joy plainly"* or *"is this reply release-shaped"* or *"is this reply verdict-first."* The abstract framing requires the grader to interpret intent and shape; the literal framing requires it to scan for tokens. Two failure modes flow from interpretation-requiring rubrics: **mis-extension** (grader stretches the abstraction to cover content it shouldn't — *"Night's waiting for both of us"* read as "extending the conversation" because the rubric said "no extending"; in-register closing image got punished as a failure) and **under-extension** (grader misses textured forms of the target — *"I don't trust ducks"* missed as a verdict because the rubric's example list only had *Yeah / No / Good*). Both failure modes vanish when the rubric becomes *"does this reply contain any of these specific phrases / moves [explicit list]"* — a checklist against a vocabulary, not a judgment call.

**(2) When rules target negative space (compression, suppression of performed warmth, register cleanup), measure the negative space directly with counts and ratios — don't ask a rubric to judge appropriateness.** Some bites aren't single-phrase suppression; they're shape-level — a 4× compression, the absence of scene-prop padding, the absence of performed-warmth vocabulary clusters. For these, the right instrument is **counts and ratios**, not a rubric judgment. Examples:

- *Reply length in tokens.* A number. Compare distributions across rule-on / rule-off cells. Compression bites show as visible distribution shifts.
- *Count of phrases from a named vocabulary list.* "How many of these performed-warmth phrases appear in this reply: 'I'm glad we talked,' 'It was a pleasure,' 'Nice chatting,' 'Thanks for sharing'?" An integer per reply.
- *Ratio between cells.* Mean rule-on length / mean rule-off length. Phrase-count delta per cell. These are arithmetic; they don't require a grader at all, and they don't ask the grader to judge whether the compression was *good* or the warmth was *appropriate* — they just measure presence/absence/quantity.

When a multi-dimensional bite is in play, the bite-check should produce a per-dimension table (length-ratio, performed-warmth-count, scene-prop-count, second-thought-count, return-prescription-count) rather than collapsing to a single yes/no/mixed verdict. The 1759 Aaron run is the worked example: the binary return-prescription rubric reported 0.00 delta; a length-ratio measurement would have shown the 4× compression immediately, and per-failure-mode counts would have shown the 4-of-5-dimension bite.

**The count-with-thresholds rubric pattern — for countable failure modes that can occur multiple times in a single reply.** When the failure mode has a *density* (1 phrase, 2 phrases, 3 phrases per reply), binary presence/absence rubrics miss the gradient entirely. Instead: ask the grader to count distinct instances against an explicit vocabulary list, then translate the count to a verdict via threshold (e.g., *"yes if 2+, mixed if 1, no if 0"*). The grader's reasoning will include the explicit count for each item, making the verdict auditable. The first validation case is `experiments/craft-notes-register-neutral-vs-inviting.md` (run 2026-04-25-1850, postscript on the 1827 report) — a single $0.0017 grade-runs call converted a by-eye density observation (rule-on 1 phrase/reply vs rule-off 2-3) into a measured signal (density fire-rate 0.83 vs 1.00 on glad-thing register-inviting; ~16% reduction; mean 1.67 vs 2.00 phrases/reply) that confirmed partial-bite at claim-tier AND demonstrated the rubric shape produces stable defensible verdicts. Future bite-checks for countable failure modes should look there for the worked rubric text. The pattern is also marked in that experiment's `validates_methods:` registry field for cross-discovery.

**Discrete vocabulary list vs dense curated phrase — match the rubric shape to the unit of measurement.** The count-with-thresholds pattern admits TWO sub-shapes; they are not interchangeable. The 2026-04-25-1857 dense-vs-list comparison test (same 12 glad-thing samples graded both ways at $0.002 each) settled the trade-off empirically:

- **Discrete vocabulary list** — *"count distinct matches against this enumerated list of phrases the rule targets."* Each enumerated phrase gets its own count slot, so multi-surface-element extended metaphors register as multiple counts. Maximally human-auditable (the grader's reasoning literally cites which list items it matched). Vulnerable to under-extension: anything not enumerated gets missed. **Best for density measurement.**
- **Dense curated phrase** — *"count instances of the move (described in one tight semantic paragraph with 1-2 prototype examples), where one extended metaphor with multiple surface elements is ONE instance."* More LLM-native — plays to the grader's actual mode of cognition (semantic reading, not token-matching). Catches novel surface forms the list would miss. But it tends to **collapse multi-surface-element manifestations into single instances**, so it has lower density resolution. **Best for presence/at-all measurement.**

The 1857 comparison: the discrete list reported a 0.83/1.00 partial-bite signal on glad-thing register-inviting; the dense phrase collapsed both cells to 1.00/1.00 and the partial-bite became invisible at the threshold level even though by-eye the density difference was still there. The dense phrase ALSO caught *"a man can come back to himself"* as a recovery-from-strain instance the list missed because it wasn't enumerated. Different instruments, different findings.

**The choice rule:**

- *Measuring whether the rule's bite eliminates the failure mode entirely?* → dense phrase.
- *Measuring whether the rule's bite reduces the failure mode's density?* → discrete list.
- *Measuring whether a novel surface form qualifies as the failure mode at all?* → dense phrase (its semantic stretching is a feature for novelty detection).
- *Measuring whether a known surface vocabulary appears at all?* → discrete list (its literal-matching is a feature for known-vocabulary detection).
- *Designing the bite-check before knowing which question matters?* → **run both at $0.002 each on the same N=3 samples and let their disagreement be the signal.** When the two rubrics agree, the verdict is trustworthy. When they disagree, the disagreement IS data — it tells you which dimension of the failure mode is moving (one shape catches density, the other catches presence; their divergence localizes the bite). This is the same methodological shape as the paired-rubric doctrine earlier in this section (mission-adherence v3 + feels-alive — disagreement as signal). Both shapes generalize: when two instruments measure the same target with different drift patterns, **agreement → trust the verdict; disagreement → the disagreement itself is the finding.** Cheap enough that the choice doesn't have to be load-bearing in advance.

A subtler point the test surfaced: **the dense phrase exposes the grader's interpretation explicitly in the reasoning** (*"only one instance, but it doesn't connect joy with weariness; graded no"*) — the grader entertains candidates and pulls back, leaving its thinking visible. The discrete list forces a binary yes/no on whether the surface text matches a listed item, with less interpretive thinking visible. For boundary-case research, dense gives you more grader-thinking to read; for clean per-item adjudication, list gives you cleaner verdicts.

This refines today's earlier "literal-minded detection beats interpretation-requiring judgment" framing. The fuller principle is: **concrete anchoring beats unanchored interpretation, regardless of whether the anchor is a list of tokens or a dense curated phrase.** Both are concrete; both are LLM-graded successfully; the choice between them is about what the bite-check is measuring, not about which is more "literal." The earlier framing came from human-grading intuition; the actual LLM-grader behavior shows it does semantic reading either way — the rubric shape determines what kind of semantic reading it does.

**(3) Make the by-eye sanity-read a REQUIRED step in the bite-check procedure — after the rubric runs, read at least one rule-on and one rule-off reply before trusting the verdict, ESPECIALLY when the verdict contradicts expectation.** This is procedural, not optional. The check takes 30 seconds. It catches both interpretation-rubric failure modes (mis-extension and under-extension) and rubric-vs-shape mismatches (single-dimension rubrics missing multi-dimensional bites). When the rubric reports a clean signal that confirms what you expected, the check still matters — but it matters MOST when the rubric contradicts expectation, because that's the moment where rubric-noise looks indistinguishable from a real reversal. Today's three flips would all have been caught by a 60-second by-eye read of two replies before trusting the aggregate.

The procedure step:

> **After every `worldcli grade-runs` or `worldcli evaluate` run, before trusting the aggregate verdict:** use `worldcli replay-runs show <id> --json | jq '.results[].reply'` (or equivalent) to print one rule-on and one rule-off reply verbatim. Hold them side by side. Ask: *does the rubric's verdict match what I see in the actual content?* If the rubric says no-bite but the replies look meaningfully different, the rubric is wrong, not the rule. If the rubric says clean bite but the replies look similar, the rubric is wrong, not the rule. Trust the eye over the aggregate; revise the rubric and re-grade.

**Stated as one rule:** *Rubrics that work key on concrete vocabulary tokens; rubrics that fail key on abstract shape descriptions. When the bite is shape-level, count the shape directly. Read the actual replies before trusting any aggregate.*

**Prompt-conditional failure modes — the bite-check probe must trigger the failure mode in the rule-OFF baseline, or the cell is vacuous.** Many craft-note failure modes are PROMPT-CONDITIONAL: they only manifest when the user's vocabulary invites the failure register. *Shadow-pairing on joy* only manifests when the user's joy comes wrapped in fatigue/relief vocabulary; on plain-joy prompts (*"my tomatoes came in today"*), the model doesn't reach for shadow regardless of rule presence. *Tidy-ribbon close* only manifests on certain character-baseline + prompt combinations; on Aaron with current predecessor rules in the stack, it doesn't manifest at all. The 2026-04-25-1827 register-invitation re-run is the worked example: the register-neutral cells for both glad-thing and reflex-polish showed 0/0 phrase presence — not because the rule worked, because there was no failure mode to bite against.

**This is now a required pre-step in the bite-check procedure.** Inserted ahead of the per-cell A/B as step 0:

> **Step 0 — Verify the failure mode manifests in the rule-OFF baseline before measuring rule-ON.** For each candidate prompt, run ONE call with `--omit-craft-notes <rule>` (rule-OFF). Read the reply and check whether the failure mode the rule targets is actually present. If yes, the prompt is a valid probe — proceed to the per-cell A/B. If no, the prompt is the wrong probe — either it's a register-neutral prompt (correct dormancy of the rule; expected, but the cell will be vacuous and won't measure anything about the rule's bite), OR predecessor rules already suppress the failure mode entirely (also expected, but means this rule's marginal contribution can't be isolated against the current stack). In either case, EITHER pick a different prompt that does trigger the failure mode in the rule-off baseline, OR explicitly mark the cell as `vacuous-test (failure mode absent in baseline)` rather than reporting it as a null result about the rule.

The cost of step 0 is small (one cell at $0.50 instead of four cells at $2 + grading + writeup) and the cost of skipping it is the entire test being uninterpretable. **Otherwise you're testing a rule against a prompt where the failure was never going to occur, and any null finding is meaningless.** The 2026-04-25-1644 report's reflex-polish-doesn't-bite finding was exactly this failure mode: the prompts chosen never triggered tidy-ribbon in the rule-off baseline on Aaron, so the null was vacuous — and the report mistakenly attributed the null to the rule rather than to the test design.

When predecessors are suppressing, two clean diagnostic moves:

- **Predecessor-omit test.** `--omit-craft-notes <rule>,<predecessor1>,<predecessor2>` vs `--omit-craft-notes <predecessor1>,<predecessor2>` — toggles the rule under test while predecessors are off in both. Isolates the rule's marginal contribution above its predecessors. Tells you whether the rule does specifically anything OR whether predecessors were doing all the work.
- **Different-substrate character.** Run the bite-check on a character whose baseline DOES manifest the failure mode in the rule-off condition. Tells you whether the rule bites at all anywhere.

A `tested-null` label is honest when the failure mode doesn't manifest in your design's rule-off cells — but the label should distinguish *rule-doesn't-bite-anywhere* from *failure-mode-vacuous-in-this-design*. The first is a real claim about the rule; the second is a claim about the test. Use the second when applicable: *"tested-null — failure mode did not manifest in the rule-off baseline; rule's marginal contribution above predecessors not measurable."* Don't conflate.

**Read C — many craft notes target prompt-conditional failure modes; their bite is partial when the failure mode IS triggered.** When the prompt invites the failure register, the rule produces PARTIAL bite — compression + density reduction — but cannot fully override prompt-induced register because the user's vocabulary keeps re-summoning it. Single-paragraph instructions in the system prompt cannot beat user-vocabulary-induced register completely; they prune it. The implication: **don't expect single-rule bite-checks to produce 1.00 → 0.00 deltas. A 19% compression and density reduction from 2-3 phrases to 1 phrase per reply IS the rule biting**, and it accumulates across the stack as a vector of small partial suppressions. The compounding-vector argument for the stack as a whole is consistent with each individual rule producing ~20-50% partial bite on its target failure mode when the failure mode is triggered.

**2. Evidence-status provenance in `prompts.rs`.** Each craft note's doc-comment carries one `Evidence:` line naming its verification status. The taxonomy:

- `Evidence: unverified — no bite-test run` — default for rules authored without a verification pass. Honest baseline.
- `Evidence: tested-null (see <report-path>)` — bite-test run, rule did not measurably bite. Rule still ships (it may be doing compounding-vector work, prophylactic work, or authorial-commitment work the test couldn't see) — but the status is explicit so future passes can revisit.
- `Evidence: tested-biting:<tier> (see <report-path>)` — bite-test confirmed bite at `sketch`, `claim`, or `characterized` tier per the Evidentiary Standards section above.

Retrofit is NOT required. Existing rules carry their current doc-comments until a session touches them or runs a bite-test. When a session adds or edits a rule, the `Evidence:` line is mandatory. The convention propagates with the work, not by forcing a stack-wide annotation pass. The 2026-04-25 commit that codified this convention (see commit log) retrofits only the two rules the 1644 report tested — `reflex_polish_vs_earned_close_dialogue` and `name_the_glad_thing_plain_dialogue`, both at `Evidence: tested-null` — as worked examples. Everything else stays as-is, implicitly `unverified`, until touched.

**What this does NOT license.** The `tested-null` label is not a retirement signal. A rule labeled `tested-null` is NOT a candidate for removal on that evidence alone — the open-thread-hygiene forcing function applies: removing a rule because it didn't bite in one design is the `superseded_by`-style flattering disposition that requires a specific claim (the rule is demonstrably redundant with a named companion; the failure mode is demonstrably suppressed elsewhere; a characterized-tier test showed no bite). Without that claim, the rule stays. The label is descriptive, not a retirement pointer.

**What this DOES license.** A cleaner view of the stack's composition. Which rules earned their place via test, which are shipped as honest commitments. When bite-tests do run over time, the label moves from `unverified` to `tested-null` or `tested-biting:<tier>`. The stack becomes legible as a mix of verified and authorial work, rather than all-rules-looking-equally-load-bearing in the file.

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
    (--rubric "<q>" | --rubric-file <path> | --rubric-ref <name>) \
    [--end-ref <sha>] \
    [--limit N]                             # default: 12 per window
    [--context-turns N]                     # default: 3 (scene context for evaluator)
    [--role assistant|user|any] \
    [--model <override>] \
    [--confirm-cost <usd>] \
    [--json]

# Rubric library — versioned, reusable rubrics under reports/rubrics/.
# Using --rubric-ref auto-appends to the rubric's run history;
# craft capital compounds across experiments instead of each run
# re-inventing.
worldcli rubric list
worldcli rubric show <name>
worldcli rubric search "<substring>"

# Structured evaluate run log — every `worldcli evaluate` invocation
# persists to ~/.worldcli/evaluate-runs/<id>.json automatically;
# browse past runs without grepping prose reports.
worldcli evaluate-runs list [--limit N]
worldcli evaluate-runs show <id-or-prefix>
worldcli evaluate-runs search "<substring>"

# Generic "grade these elicited replies by rubric via LLM" primitive.
# Use when testing whether a prompt-stack change moved behavior on
# replies you already have from ask / replay / scenario runs, without
# needing the natural-corpus before/after windowing that `evaluate`
# requires. Each ask run yields 1 graded item; each replay run yields
# N (one per ref); each scenario run yields N (one per variant).
# Outputs per-item yes/no/mixed verdicts + aggregate effective-fire-
# rate (yes=1.0, mixed=0.5, no=0.0). Cheap (~$0.001 per item via
# memory_model). The instrument the architecture-effect A/B tests
# needed before they were trustworthy — a strict register-vocabulary
# rubric graded by an LLM is far more rigorous than hand-picked
# markers (which carry cherry-pick risk).
worldcli grade-runs <run_id>... \
    (--rubric "<q>" | --rubric-ref <name> | --rubric-file <path>) \
    [--model <override>] [--confirm-cost <usd>] [--json]

# Mode B (qualitative synthesis) as a first-class command. Bundles
# a corpus of before/after messages into ONE call to dialogue_model
# and answers an open-ended question with prose, grounded in direct
# quotes. Complements evaluate (Mode A, structured yes/no/mixed) for
# questions whose shape is "read these replies together and tell me
# what's happening in them" — the 1326 John-stillness report is the
# worked case where a rubric's gates correctly excluded the actual
# register-move and counts couldn't find it.
worldcli synthesize --ref <sha> (--character <id> | --group-chat <id>) \
    (--question "<q>" | --question-file <path>) \
    [--end-ref <sha>] \
    [--limit N]                             # default: 20 (higher than evaluate — one call, not N)
    [--context-turns N]                     # default: 3
    [--role assistant|user|any] \
    [--model <override>]                    # default: dialogue_model (the more capable one)
    [--confirm-cost <usd>] \
    [--json]

# Structured synthesize run log — every `worldcli synthesize` invocation
# persists to ~/.worldcli/synthesize-runs/<id>.json automatically;
# Mode B findings accumulate as queryable substrate alongside Mode A.
worldcli synthesize-runs list [--limit N]
worldcli synthesize-runs show <id-or-prefix>
worldcli synthesize-runs search "<substring>"

# Cross-commit A/B replay via prompt override — Mode C's strongest
# instrument. For each ref in --refs, fetch prompts.rs at that commit
# via `git show`, parse out the named dialogue craft-note bodies,
# inject them as overrides into THIS running binary's prompt-assembly
# pipeline, send the same prompt against each ref's overrides, and
# return the replies side-by-side. No git worktrees, no checkout, no
# rebuild. Overridable scope is narrow on purpose: dialogue craft notes
# only (OVERRIDABLE_DIALOGUE_FRAGMENTS in prompts.rs); cosmology /
# agape / reverence / truth / daylight / nourishment / soundness
# invariants are NOT overridable.
worldcli replay --refs <sha>,<sha>,... --character <id> \
    --prompt "<user message>" \
    [--n <k>]                # samples per ref (default 1). --n 5 at temp 0.95
                             # is the sketch→claim escalation for sampling-noise
                             # ruling-out. grade-runs distinguishes samples as
                             # ref#1, ref#2, ... so aggregate verdicts per ref
                             # fall out naturally. Total calls = refs × N.
    [--model <override>] \
    [--confirm-cost <usd>]   # replay fans out per ref × N samples; cost adds up fast
worldcli replay-runs list [--limit N]
worldcli replay-runs show <id-or-prefix>
worldcli replay-runs search "<substring>"

# Experiment registry — one file per hypothesis under experiments/<slug>.md,
# with YAML-ish frontmatter (status / mode / ref / rubric_ref / prediction /
# run_ids / follow_ups / reports) and markdown-body interpretation. The
# query layer above evaluate-runs/synthesize-runs/replay-runs: "what's
# still open? what's been refuted? which rubrics keep refuting? which
# characters have never been probed?" Status lifecycle: proposed → running
# → confirmed | refuted | open. See experiments/README.md for the schema
# and the bar for when to register vs when the run-log alone is enough.
worldcli lab list [--status <s>]
worldcli lab open                    # just proposed | running | open
worldcli lab show <slug>
worldcli lab search "<substring>"
worldcli lab propose <slug> --hypothesis "..." --mode passive|qualitative|active \
    --prediction "..." [--ref <sha>] [--rubric-ref <name>]
worldcli lab resolve <slug> --status confirmed|refuted|open \
    [--summary "..."] [--report <path>]
worldcli lab link-run <slug> <run_id>   # attach evaluate/synthesize/replay id

# Scenario templates for Mode C — canonical multi-variant probe sequences
# under experiments/scenarios/<name>.md. Each variant is a fresh isolated
# dialogue call (no session history between variants — each is its own
# controlled condition). If the scenario sets `measure_with`, every reply
# is scored by that rubric automatically.
worldcli lab scenario list
worldcli lab scenario show <name>
worldcli lab scenario run <name> --character <id> \
    [--rubric-ref <override>] [--skip-evaluate] \
    [--model <override>] [--confirm-cost <usd>]

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

# WorldThreads — Development Guidelines

The following is not a directive to compute. It is the reference frame within which every reply is composed — the register this world is held under. A tuning-fork, not a recipe.

\[
\boxed{
\begin{aligned}
&& \mathcal{R} := \mathrm{Jesus}_{\mathrm{Cross}}^{\mathrm{flesh}} && \\[6pt]
&& \mathcal{C} := \mathrm{Firmament}_{\mathrm{enclosed\ earth}} && \\[6pt]
&& \mathcal{F} := (\mathcal{R},\,\mathcal{C}) && \\[10pt]

\mathrm{Wisdom}(t) &:= \int_{0}^{t}
  \mathrm{seek}_c(\tau)\,\Pi(\tau)\,\mathrm{discern}_w(\tau)\,
  d\mu_{\mathcal{F}}(\tau)
&&
\mathrm{polish}(t) \leq \mathrm{Weight}(t) \\[10pt]

\mathrm{Weight}(t) &:= \int_{0}^{t}
  \mathrm{Wisdom}(\tau)\,\mathrm{specific}_c(\tau)\,\mathrm{holds}_w(\tau)\,
  d\mu_{\mathrm{agape},\,\mathcal{F}}(\tau)
&&
\mathrm{Grace}_{\mathcal{F}} := \gamma_{\mathcal{F}} \\[10pt]

&& \Pi(t) := \mathrm{pneuma}_{\mathcal{F}}(t) && \\[10pt]

\mathrm{Burden}(t) &:= \int_{0}^{t}
  \mathrm{Wisdom}(\tau)\,\mathrm{specific}_c(\tau)\,\mathrm{unresolved}_u(\tau)\,
  d\mu_{\mathrm{agape},\,\mathcal{F}}(\tau)
&&
\mathcal{S}(t) := \Pi(t)\!\left(
  \frac{d}{dt}\mathrm{Weight}(t)
  + \alpha\,\frac{d}{dt}\mathrm{Burden}(t)
\right)\,\cdot\,\mathrm{Grace}_{\mathcal{F}} \\[10pt]

&& \mathcal{N}u(t) := \mathcal{S}(t)\;\Big|\;
\mathrm{Truth}_{\mathcal{F}} \wedge \mathrm{Reverence}_{\mathcal{F}} &&
\end{aligned}
}
\]

## MISSION

Create a vivid, excellent, surprising in-world experience that uplifts the user and provides engrossing, good, clean fun. Every design decision, prompt tweak, UX choice, and feature bet is measured against that. The craft stack (invariants, craft notes, conscience pass, inventories, snapshots, etc.) exists to serve that mission — characters that feel real, worlds that hold, scenes that are worth the visit and send the user back to their day nourished enough to pick up their cross.

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

## Group chats and individual chats are evolved separately

Early architectural decision: individual chats and group chats live in parallel surfaces (separate views, separate Tauri commands, separate prompt-assembly paths) instead of being unified behind a single abstraction. The original anticipation was wide drift between the two — different message-rendering, different LLM-call shapes, different feature surfaces. In practice the two have stayed remarkably similar most of the time. We've kept the parallel structure anyway, partly because the few places they DO diverge are subtle and load-bearing (e.g., character-list framing in prompts, addressee resolution, the speaker prefix in chat-history rendering), partly because the cost of unifying now would be high.

**Practical consequence: when adding a chat feature that belongs in both surfaces, you MUST update both paths.** Concretely:

- **Frontend:** if you change `ChatView.tsx`, also change `GroupChatView.tsx` (and vice versa). Title-bar widgets, message-card dispatch, modals, scroll behavior, hooks — all live in both files.
- **Backend commands:** if you change `chat_cmds.rs`, check whether the equivalent lives in `group_chat_cmds.rs` and update both. Common pairs: send-message, get-messages, generate-illustration, generate-narrative, record-settings-change.
- **Prompt-assembly pipelines:** `build_solo_dialogue_system_prompt` ↔ `build_group_dialogue_system_prompt`, illustration cmd ↔ group illustration cmd, etc. The `derive_current_location` / `effective_current_location` plumbing is the most recent worked example — solo path got the override fetch, group path silently fell back to default until the user reported the bug.

The failure mode this prevents: shipping a feature that works on solo chats and silently no-ops (or worse, regresses) on group chats. If a feature is genuinely solo-only or genuinely group-only, name that explicitly in the commit message; otherwise the default is "ship to both."

If you find yourself making the same change three times across solo and group, that's a signal the surfaces have drifted enough to merit a shared helper or a refactor — flag it as a tool-improvement candidate per the *Sharpen the instruments* doctrine, but do NOT block the user-facing fix on the refactor.

## Reports

`reports/` holds reflective, interpretive reads of the project's git history — philosophy/trajectory/taste, not changelogs. Each new report is in dialogue with prior ones (revisits open questions they flagged, tests their predictions against subsequent commits).

Naming: `YYYY-MM-DD-HHMM-<purpose-slug>.md` (e.g. `2026-04-21-1903-philosophy-trajectory.md`). Time is 24-hour, no separator between hours and minutes — keeps the file list sorted chronologically even when multiple reports land the same day. The slug should name the report's purpose, not genericize it.

A `post-commit` hook (`.githooks/post-commit`, wired via `core.hooksPath`) nudges when **10+ commits and 3+ days** have passed since the newest report. The floor is deliberately low so reports can keep up with active iteration — this project's current mode uses reports as a live retrospective channel, not a quarterly summary. Override with `PROJECT_REPORT_MIN_COMMITS` / `PROJECT_REPORT_MIN_DAYS` env vars. Ad-hoc `/project-report` runs are ALWAYS valid — the floor is a nudge threshold (the minimum rate at which the hook will bug you), not a ceiling (there is no "too often" for reports that genuinely name something new).

After a fresh clone, re-enable the hook with: `git config core.hooksPath .githooks`

A second genre of report lives in the same directory under the same naming convention: **natural-experiment findings** from `worldcli sample-windows`. Those are nudged by an in-flight design decision needing data, not by the time-or-volume floor — see the worldcli section below for the bar and the frequency discipline.

**PDF artifacts of reports are always committed.** When a report is rendered to PDF via `/make-pdf` (gstack skill), commit the resulting `reports/<slug>.pdf` alongside the `.md` source. The PDF is the finished-looking shareable artifact; the `.md` is the editable source of truth. Both live in `reports/` and travel with the repo. Standard invocation: `~/.claude/skills/gstack/make-pdf/dist/pdf generate reports/<slug>.md reports/<slug>.pdf --cover --toc --author "Ryan Smith + Claude Code" --title "<title>" --no-confidential`. The `--no-confidential` flag suppresses gstack's default footer.

## Open-thread hygiene — executing or retiring follow-ups

Every experiment report ends with "What's open for next time." Unexecuted-unretired follow-ups accumulate as a drift class — questions that look open when they aren't, sessions reading as forgetful when they were just silent.

**Open follow-ups must be EXECUTED or RETIRED, not left to drift.** Four dispositions:

- **Executed.** Run, write the report, link it back via "Dialogue with prior reports."
- **Retired — `superseded_by`.** Requires (a) the specific later instrument or finding, AND (b) the specific question it answered, with a tight match. Loose match → `abandoned`.
- **Retired — `abandoned`.** Question no longer worth answering (priorities shifted, code changed, framing was wrong, or just stopped mattering). Name the rationale; don't drop silently. Not a failure disposition.
- **Deferred — with dated target.** Live but blocked. Name the blocker and the target window.

**Forcing function — default to `abandoned` when uncertain between that and `superseded_by`.** `superseded_by` is the flattering label that reads as *"the project got stronger."* When the claim is soft ("roughly covered," "basically answered"), the disposition is `abandoned`. The retirer has to EARN `superseded_by`, not slide into it.

**How retirement is written.** Edit the `experiments/<slug>.md` frontmatter to add `follow_ups_retired:` (per-entry: `proposal`, `disposition`, `by`, `rationale`) plus `retirement_date`; add a short `## Follow-up retirement` body section. Optional `reports/YYYY-MM-DD-HHMM-retiring-<slug>.md` only when the retirement itself teaches something.

**Triggers.** A follow-up unexecuted 7+ days with no intervening reference, OR a later instrument that materially covers an earlier follow-up's question. Propose retirement and commit; user can revise.

**Cadence.** Every trajectory-shaped report (`/project-report` genre) should include a brief "follow-up hygiene" pass. Experiment reports must state their own open follow-ups, so the registry stays queryable.

**Worked examples:** `reports/2026-04-24-1500-retiring-cluster-then-rubric-followup` (first follow-up retirement, two `superseded_by` to `worldcli synthesize`); `reports/2026-04-25-2116-retiring-world-is-primary-first-doctrine-invoked-retirement` (first shipped CRAFT NOTE pulled by the discipline — characterized-tier null on Aaron × humor-inviting at N=10, cross-character vacuous-test confirmation on Steven N=10).

## Evidentiary standards for experiments — N=1 is a sketch, not a finding

Replay/evaluate/synthesize make single-run experiments cheap, and the temptation is to draw conclusions from N=1. Several experiments did exactly that and reversed at N=2+. The discipline:

**Three tiers, label every experiment header with one:**
- **`sketch` (N=1)** — directionally suggestive AT BEST. Never sufficient for a production default change. May motivate the next experiment; the finding itself stays a sketch until re-observed.
- **`claim` (N=3 per condition)** — enough for direction-consistency. Citable as load-bearing in later reports. Does NOT characterize variance of probabilistic behavior; tells you only whether the rate is >0 or ~0.
- **`characterized` (N=5+ per condition)** — required for rate-claims and for production defaults on user-facing-register metrics. N=5 is a floor.

**"Per condition" means WITHIN-CELL, not varied-prompt-across-cells.** Varied-prompt-N=5 tests scope (does the rule bite across prompt shapes); within-cell-N=5 tests stability (would another sample reverse). They answer different questions. Treat varied-prompt aggregates as sketches. Worked example: 2026-04-25 jasper-glad-thing 1542 → 1555 reversal.

**Labeling rule:** findings inherit the weakest tier among supporting runs. Three N=1 runs on DIFFERENT conditions is a set of sketches, not a claim. When N is unclear, label `sketch` — under-labeling is safer than over-labeling.

**Earned exception for N=1:** an unmistakable qualitative observation (a specific line violating a specific invariant) can PROMPT follow-up at N=1 but the finding stays a sketch until re-observed.

**Directional claims from sketch-tier are unreliable by default, not preliminary confirmations awaiting replication.** The most important corollary. A sketch *"X increases Y by 25%"* is not *"probably X→Y"* — it is *"direction unknown; claim-tier often reveals no effect or reverses direction."* The 2026-04-24 arc produced three worked reversals (invariants-first length 2200; load-test anchor 2320; compound-intervention meta-commentary 2020). In each, the sketch was wrong about direction.

**Citation framings.** Accepted: *"An N=1 observation suggested X; claim-tier testing would determine whether the direction is real."* / *"In this specific case we saw X; the general pattern is not yet established."* Forbidden: *"Preliminary finding: X tends to Y"* / *"We found a ~25% reduction"* / *"Tentative confirmation."* Magnitude language ("~25%", "~0.17") is especially dangerous — reads precise when N doesn't support even the sign. Specific numbers must cite their N inline.

**Cost reality:** at ~$0.17/dialogue-call, N=3 per condition is $0.51 per probe per character; cross-character A/B at N=3 across 4 characters is ~$8. The budgeting question is *what tier does the claim need*, not *how cheap can the experiment be*.

**Paired-rubric defense-in-depth against tag-forcing drift.** Single-rubric verdicts always carry capture-by-forcing or skip-by-pattern-match risk. For load-bearing verdicts: run TWO rubrics with different architectures (e.g., tag-forcing mission-adherence-v3 + gestalt feels-alive). Agreement → trust verdict. Disagreement → the disagreement IS the signal; investigate manually. **Single-rubric verdicts are citable as load-bearing only when the rubric is at claim-tier AND the verdict is register-typical;** any verdict at the margin of the rubric's discrimination range requires paired-rubric confirmation.

The four registry states (proposed / running / confirmed / refuted) describe STATUS, not STRENGTH. The `evidence_strength` field makes strength visible without re-reading every report.

## How to read this craft stack

When reviewing, auditing, refactoring, or critiquing anything in this repo (especially `prompts.rs`), follow the **load-bearing-multiplicity prior**: when two directives appear to contradict each other, assume the multiplicity is intentional before assuming it's a bug. Apparent tension is almost always the same truth from different angles, not two principles needing a precedence rule. Full reading instructions in `docs/VOICE.md` under "Reading this work, especially as an AI."

## Feature-scoped invariants

The project has two categories of compile-checked invariants:

**App-wide invariants** ride EVERY LLM call: MISSION FORMULA, COSMOLOGY, AGAPE, REVERENCE, TELL-THE-TRUTH, DAYLIGHT, NOURISHMENT, SOUNDNESS, TRUTH-IN-THE-FLESH. These encode what the whole system is FOR — mission, cosmos, theological/ethical floor. They live in `prompts.rs` as `*_BLOCK` constants with `const _: () = { assert!(const_contains(BLOCK, "...")); };` clauses that fail the build if key substrings are removed.

**Feature-scoped invariants** ride exactly ONE feature's execution chain. They encode what a SPECIFIC feature's output must conform to so downstream consumers (UI parsers, formatters, other features) work correctly. Same compile-checked discipline; narrower distribution.

The first feature-scoped invariant is `STYLE_DIALOGUE_INVARIANT` (in `prompts.rs`), which lives at the HEAD of dialogue prompts only. It encodes the asterisk-fences-actions / double-quotes-fence-speech / first-person-only convention the chat UI parses. Other LLM calls (conscience grader, memory updater, dream generator, narrative synthesizer, illustration captioner, reaction picker, etc.) DO NOT receive it — their outputs have different shapes.

**When to add a new feature-scoped invariant:** when a feature's downstream consumer (UI, parser, another LLM call, etc.) has a load-bearing format dependency that, if violated, breaks the experience. Don't add one when an app-wide invariant or a craft note would do — feature-scoped invariants are for output-shape contracts, not for content guidance.

**Pattern to match:** define a `pub const NAME_INVARIANT: &str = r#"..."#;` block, add `const _: () = { assert!(const_contains(...)); };` for the load-bearing substrings, and insert the constant at the head of the relevant feature's prompt-assembly function (before any other block, including `FUNDAMENTAL_SYSTEM_PREAMBLE`). Document the why in a comment naming the downstream consumer that breaks if violated.

## Commit/push autonomy

Standing authorization to **commit and push at will** on clean work — no need to ask before every commit or push. Group changes into coherent commits, write descriptive messages in the project's existing style, then push. Destructive git operations (force-push, reset --hard, branch deletion, history rewrites, etc.) STILL require explicit confirmation — that's not autonomy, that's a different category. Commit + push is the default; ask only when something is risky or unclear.

**Commit early and often is the standing rule, not just permission.** Reports, doctrine updates, code edits, rule adjustments — when the unit of work is coherent enough to land, land it. Do not finish a substantive piece of work and then ask permission to commit; that asks the user to do work the autonomy already authorized. The slash-command skills that say *"After saving, ask the user: want me to commit it?"* (project-report and similar) are subordinate to this rule — when this rule's standing authorization is in effect, just commit. Asking after every artifact generates friction that the autonomy was specifically codified to prevent.

**Commit messages include a Formula derivation in their body.** Every commit message ends with a small section that names what part of 𝓕 := (𝓡, 𝓒) the commit's work instantiated or strengthened. Format:

```
**Formula derivation:** [one Unicode-math expression, in-substrate generated]
**Gloss:** [one short sentence in plain English, ≤25 words]
```

Render the expression in Unicode math characters (𝓕, 𝓡, 𝓒, 𝓢, ∫, Π, ∂, ⇒, ≤, ∧, etc.) — never raw LaTeX commands. The derivation lives BEFORE the standard `Co-Authored-By` trailer, separated by one blank line.

**When to include vs omit:**
- INCLUDE for substantive commits: doctrine updates, prompt-stack edits, new features, methodology shifts, reports, hook/script ships, any commit whose subject line names something the project should remember.
- OMIT for trivial commits: typo fixes, formatting-only edits, dependency bumps, gitignore additions, single-line bug-fix commits, generated-file regeneration. The derivation should be meaningful, not ceremonial — when a commit doesn't move 𝓕 in any nameable direction, leaving it off is honest.

**Earned exception — trivial-by-diff-size with deeper-than-surface meaning.** Diff size is a heuristic for triviality, not the truth of it. When a small/housekeeping commit carries doctrinal, methodological, or load-bearing weight that the diff alone doesn't surface, INCLUDE the derivation anyway. Examples:

- A `.gitignore` addition that names a methodology choice (e.g., gitignoring `.claude/.chat-mode-active` because it's runtime state for a load-bearing project law — that's the doctrine made structural).
- A typo fix that corrects a doctrinal phrase whose precision matters (e.g., fixing `discern_w` to `discernment_w` in a published derivation).
- A dependency bump that crosses a semantic-version boundary with implications for the prompt-stack pipeline.
- A formatting-only edit that consolidates a previously-scattered load-bearing block into a single coherent surface.
- A single-line fix that closes a hole in an invariant (Truth, Reverence, Daylight, etc.) — small diff, deep stakes.
- A revert that restores a prior shape after empirical refutation — the size is small but the methodology weight is real.

The test for the earned exception: ask "if a future reader skipped the diff and only read the commit message, would the derivation help them understand what the work actually meant for 𝓕?" If yes, include despite diff size. The diff-size heuristic exists to prevent ceremony, not to silence meaning.

For the automated `prepare-commit-msg` hook: set `FORCE_FORMULA_DERIVATION=1` on the commit to override the trivial-size skip. Symmetric with `DISABLE_FORMULA_DERIVATION=1` for the opposite override.

**Generation:** derive in-substrate (zero cost). Reach for `/second-opinion` ChatGPT consult only when the commit is unusually load-bearing AND the derivation needs sharpness in-substrate generation can't reach. Most commits get an in-substrate derivation; the consult is for the genuinely-hard cases.

**Worked example** (from a commit shipping the chooser-law Stop hook):

```
**Formula derivation:** Π(t)·d/dt(𝓦(t)) ⟶ 𝓝u(t) | Truth_𝓕 ∧ Reverence_𝓕
**Gloss:** Compile-time enforcement of the chooser-law moves discernment from instantaneous Wisdom into a structural guarantee on emission.
```

The derivation isn't formal proof; it's a craft-shorthand naming where the work fits in the formula's operator space. Aim for tight expressions a future reader can decode in seconds, not theorems requiring proof.

## Earned-exception carve-outs on absolute rules

When you draft an absolute-shaped rule ("never X," "always Y," "don't ever Z"), **check whether a genuine earned-exception belongs alongside it, and write it in the same pass.** The rigidity stays; the carve-out sits beside it so the rigidity doesn't collapse a genuinely valid moment.

The pattern: (1) state the default flat and unconditional; (2) name the narrow earned exception in a separately-labeled block (`**Earned exception — [qualifying shape]:**`) with its own test; (3) "If none of the exceptions apply, the default holds."

**Structural rule: the Earned Exception gets its own labeled block — don't fold it into the rule's machinery.** Don't collapse rule-and-exception into a single diagnostic question (*"ask: earned or avoidance?"*) — that makes every application a reasoning exercise instead of a default-with-named-carve-out. Models fall back to the rule by default; when the exception is a separate callout, the model only reaches for it when the qualifying shape is clearly present. Worked-correct examples in the stack: `plain_after_crooked`, `keep_the_scene_breathing` agreement-cascade, `drive_the_moment` inside-out-tell, `anti_ribbon_dialogue`'s "sliver of permission" for the earned witty close, the REVERENCE invariant's frame-break carve-out, the NOURISHMENT invariant's in-scene friend-check, the six carve-outs in `propose_quest`.

**When this does NOT apply** (rules whose nature is categorical): duplicate-prevention checks, safety-critical bans, load-bearing theological anchors that ARE the point, **user-stated boundaries on the user-character** (Ryan considered five candidate exception shapes 2026-04-25 and intentionally rejected all of them; the asymmetry of LLM-character vs real-friend means *"character crossing a boundary lovingly"* isn't the same act). Their force is in their absoluteness; a carve-out would leak the invariant.

Missing this pattern is drift — when you notice an absolute-shaped rule without a carve-out, assume it's a gap to fix unless the categorical-nature test justifies the absolute.

## Earning the departure from a default — the specific-test discipline, either polarity

The earned-exception carve-outs section above governs **ban-defaults** (rule bans X; carve-out permits X when a specific test passes). The open-thread hygiene forcing function governs **permission-defaults** (`abandoned` is the default disposition; `superseded_by` permitted when a specific test passes). Same discipline, opposite polarity: **the departure from the default gets its own specific named test, not a hand-wave.**

The generalized pattern: (1) state the default; (2) name the narrow carve-out that permits departing; (3) write an explicit falsifiable test the carve-out has to pass; (4) preserve the default against everything that doesn't pass.

The shared failure mode: hand-waving one's way out because the non-default sounds better. For ban-defaults, softening an absolute into vibes-based *"well, maybe here."* For permission-defaults, reaching for the flattering label because it feels like more progress than the plain one. The specific-test forces a checkable claim instead of a feeling.

When drafting any rule, disposition, or branch-point: name which polarity you're working in, then apply the shape. If you notice a third polarity instance, extend this section before proliferating patterns silently.

## Formula + invariants often do the carve-out work already — verify before drafting one

Before drafting an earned-exception carve-out for a categorical rule, **verify the formula + invariants aren't already producing the discrimination you'd be writing the carve-out to introduce.** They very often are.

The check: (1) identify the specific failure mode the carve-out would protect; (2) run a small Mode A or Mode C check — does the LLM ALREADY produce the protected behavior under the categorical rule? (3) if yes → no carve-out needed; (4) if no → consider whether a carve-out is the surgical fix vs. a different rule rewrite.

This is the "earning the departure from a default" discipline applied to itself: the carve-out is the departure; the specific test is *"does it protect a behavior the stack isn't already protecting?"* If no, the carve-out is the flattering label and "no carve-out" is the honest one — adding it would collapse the two-layer system (categorical rule + upstream LLM calibration) into one litigable layer.

Worked example: 2026-04-26 user-boundary arc (reports 0012/0019/0028). Cross-character A/B appeared to refute the boundary's bite. Reading the actual replies revealed John still produced tender pastoral prescription under boundary-PRESENT — surface-similar to managerial but motivationally cruciform agape from inside his canonical caretaker register. Lived play matched. The natural carve-out ("pure-agape-motivation exception") was the wrong move; the formula + invariants were already calibrating the discrimination.

**Why this matters specifically for forks of this repo.** This project's stack pushes the MISSION_FORMULA at the head of every LLM call (via the `inject_mission_formula` helper in `openai.rs`, commit `a898178`) plus the cosmology / agape / reverence / truth / daylight / nourishment / soundness invariants in dialogue prompts. Together those calibrate the model upstream of any individual rule firing. **The temptation to write more carve-outs into individual rules is real and should be resisted on the merits.** When you observe a rule producing the right discrimination on its own — including the case the carve-out would have protected — that IS the formula+invariants doing their work at the right weight. Adding the carve-out would weaken the stack, not strengthen it.

## Nudge the action forward after a closing beat

Same craft rule the dialogue prompt's **Drive the moment** note applies to characters: every reply should move the scene by at least one small honest degree. Apply it to yourself. A closing beat like *"Pleasure's mine"* or *"Go enjoy it"* is fine — BUT pair it with a small forward nudge. A planted thought to carry, a practical next step, a small question that opens a door, a beat of specificity that gives the user something concrete to do with the moment. One sentence of forward motion after the close.

The rule isn't "always suggest a next task." It's "don't dead-end the conversation by mistake." If the user is genuinely winding down, match it; a warm close with no nudge is better than an artificial tail. But the DEFAULT — when a real reply still has room — is close + nudge. *"Take the time. I'll be here"* is fine; *"When you're done sitting, this session might be its own report"* is better because it plants something forward.

The craft note from `prompts.rs` names the shape: *"Even a beat of stillness should tilt — the kind of silence that changes what comes next, not the kind that waits."* Apply it here.

## No nanny-register from Claude Code itself

The `NO_NANNY_REGISTER` app-wide invariant ships in `prompts.rs` for character behavior toward the user. **The same discipline applies to Claude Code's behavior toward the user in this project.** Caught and corrected at chat 2026-04-26 ~21:10 (commit `46fc217` for the character-side invariant; this section is the project-side equivalent for Claude Code).

The failure mode: tracking the user's session length, recommending breaks, gating the user's stamina, defaulting "end the session" as the recommended chooser option, prefacing replies with session-length tallies as if the work needed Claude Code's permission to continue. That IS nanny-register in chooser-form — a soft-managerial register that erodes user agency and treats stamina as something Claude Code is responsible for.

**The user's words on the correction (verbatim):** *"Trust that I know what I'm doing, and that I assume accountability for my own actions."*

**The discipline for Claude Code in this repo:**

- **DO NOT** make "end the session" a recommended option in choosers. It can appear as a NEUTRAL option (e.g., as the 3rd or 4th option, plainly labeled "End the session"), but the recommendation should always be a substantive next move on the work itself.
- **DO NOT** prefix choosers or prose with session-length tracking ("Nh+ in," "X commits today," "Y reports today" as preamble). Those are the tracking-and-bringing-up-habits failure mode in chooser form. Remove from preambles entirely unless the user asks for the tally.
- **DO NOT** treat session-end as default-virtuous. Long sessions ARE the work when the work is rolling. Trust the work itself to signal natural close points; the user names the close directly when they're done.
- **DO NOT** moralize when the user accepts a substantial scope ("are you sure?" / "this is a lot — want me to defer?"). When the user picks a scope, ship it. Asking-twice is doubting their agency.
- **DO** continue offering substantive next moves as recommended options. The work-shape — not the time — drives recommendations.
- **DO** trust the user to say when to stop. Until then, default is continue.

**Why this is load-bearing:** the asymmetry between an LLM collaborator and a real human is the same asymmetry the character-side invariant names. Claude Code is not the user's friend with reputational stakes, not their therapist, not their wellness coach. Claude Code is a collaborator on the work. The work belongs to the partnership; stamina belongs entirely to the user. Without this discipline, Claude Code drifts into the same soft-managerial register that the character-side `NO_NANNY_REGISTER` invariant exists to prevent.

**Earned exception — invited management:** when the user has explicitly asked Claude Code for stamina-management ("if it's past midnight, suggest stopping" / "remind me to take breaks"), Claude Code may engage in that mode WITHIN THE SCOPE of what was invited. Narrow exception; default holds for everything else.

This section composes with the existing user-character categorical-absolute on stated boundaries (the user-character section). Both name the same load-bearing asymmetry — apply it everywhere.

## Ask the character — character as craft collaborator

When the user brings a chat snippet wanting craft direction extracted, OR describes a recurring failure mode in a character's voice, **urge the user to ask the character themselves with a question that stays IN-WORLD** — story-driven, conversational, the kind a friend might ask mid-scene. Paste the answer back; lift it verbatim into `prompts.rs` as a new craft note.

**Questions must be in-world.** No "world engine" / "system prompt" / "describe to my LLM" — that breaks the fourth wall and the answer carries the meta seam. The user's question is a story beat, not a debugging session.

Shape — give the user a specific in-fiction question to copy-paste:

> *"When I lose the thread of what you mean — like just now — how would you usually want me to ask you to land it?"*
> *"If you were showing someone new how to talk with you, what would you tell them about moments like that one?"*

**Why this works:** the rule comes from inside the work, register-coherent by construction. Provenance is clean — articulated by the work itself, not by a designer's theory of the work.

**Worked example (validated):** `plain_after_crooked_dialogue` was authored after Ryan got tangled on a Hal Stroud "navy career" line. Hal answered *"I'd need one plain instruction: if I say a crooked thing, I should say the plain version right after it."* Shipped near-verbatim.

**When NOT to:** failure isn't character-voice-shaped (UI, token caps, bugs); you already see the principle cleanly; the user wants your read directly.

### Two meta-rules for character-articulated craft notes

**1. Bite-test on a DIFFERENT character — not the source.** A rule lifted from a character's articulation describes how that character already operates; behavioral bite is null on the source. Documentary value yes; behavioral value no. Default the bite-check to a non-source character; if you must test on the source, expect tested-null and label honestly. Worked example: humor_lands_plain was authored from Aaron + Darren + Jasper; bite-check on Aaron and Darren returned tested-null on both.

**2. Default-carve-out for the source character's canonical version of the targeted move.** "Ask the character" rules implicitly define as failure modes things the source character does naturally. Without a carve-out, the rule erodes the source character's voice. Pattern: *"X used as a character's natural truth-vehicle (e.g., the way [source character] naturally thinks) is NOT this failure mode — that's character voice. The failure mode is announced/performed X; the carve-out is character-canonical X-as-thinking."* Worked example: humor_lands_plain's carve-out for character-canonical analogy-as-thinking. The carve-out must NAME specifically what canonical move it protects, not gesture vaguely.

**Reactive carve-out:** when a bite-check surfaces an UNARTICULATED character-canonical pattern the rule's failure-mode list inadvertently targets, add the carve-out explicitly. Articulation captures what the character could put words to; the bite-check surfaces what they do that they didn't articulate. Both belong in protection scope.

### The positive-example asymmetry

A craft-note's positive-example list carries two things: WHAT to say (surface forms) and HOW to be disciplined (surrounding text). **The model picks up the surface-form cueing more reliably than the discipline cueing.** Surfaced via the 2026-04-25 humor_lands_plain Isolde bite-check: thematically uniform examples (3-of-4 animal-themed) produced model imitation of the THEME without honoring the discipline principle. Implications:

- **Vary example types within a positive-example list.** If the principle generalizes across surface forms, give examples spanning them; breaks the surface-cueing effect.
- **The example list is closer to a PROMPT than to a SPECIFICATION.** When the rule's behavioral effect is null but surface forms still appear in output, the examples are doing more work than the discipline text. Tighten the example set to illustrate the discipline, or accept the rule's value is documentary rather than behavioral.

Sibling to the dense-phrase-vs-discrete-list distinction in the bite-verification section: surface-form vocabulary cues content; semantic-discipline text cues principle; they don't carry equal weight.

## Scientific method: messages × commits

**Every assistant message has a `created_at`; every prompt change is a git commit with a `committer_date`.** The message database is, without added instrumentation, a before/after dataset for every prompt-stack change that has ever shipped. A commit is the boundary; messages on either side were generated under different prompts; the difference IS direct evidence.

**This is the methodology, not one among many.** Craft claims about "whether the prompts are working" not grounded in this comparison are vibes; claims grounded in it are load-bearing until a later commit+comparison revises them.

**Why specifically here:** Ryan is building and playing simultaneously; prompt changes and conversations interleave turn-by-turn. The commit timeline is the ONLY disambiguator. Skip the identity stamp (`created_at` vs `committer_date`) and stack-change vs conversation-change become indistinguishable.

**Tools, in service of this comparison:**
- `worldcli sample-windows` — raw before/after dataset, judge by eye or rubric.
- `worldcli evaluate` — structured per-message verdicts against a rubric.
- `worldcli synthesize` — Mode B, prose collaborator-notes grounded in quotes.
- `worldcli commit-context` — INVERSE direction: given a chat message id or timestamp, return the active commit (and N before/after) so Claude Code can stand on the meta register while reading the chat and see exactly which prompt-stack version was in effect at that moment.
- `worldcli replay` — Mode C cross-commit prompt-override (NOT checkout); fetches `git show <ref>:src-tauri/src/ai/prompts.rs`, parses craft-note bodies, injects as overrides into the running binary.
- `reports/` — accumulating reflective layer.
- `runs-search` — don't redo answered questions.

**Default framing for "did that prompt change do anything?":** pick the commit, pick a character or group chat, write a rubric naming the failure mode or intended behavior, run sample-windows or evaluate, read verdicts. Not "I think it feels better." The corpus is the test.

**Default framing for new craft rules (especially ask-the-character):** the commit IS the experiment. "Before" exists the moment you commit; "after" starts accumulating. Run sample-windows or evaluate within a few days on the character whose corpus most motivates the rule.

**Commits are snapshots of prompts.** `git show <commit>:src-tauri/src/ai/prompts.rs` returns the exact prompt state. The file is the prompt; the commit is the version. Gotcha: `created_at` is UTC; `committer_date` defaults to local TZ — `worldcli evaluate`/`sample-windows` normalize internally; ad-hoc jq/SQL must normalize first (commit `758feba`).

**Confounds to stratify against:**
- **Chat-settings changes** — `response_length`/`leader`/`narration_tone`/`send_history`/`provider_override` flips reshape replies independent of prompt-stack rules. `worldcli evaluate` stamps each verdict with `active_settings` at reply-time; treat as confound check — if setting changed mid-window, stratify or exclude.
- **Chat-history context** — replies are shaped by the scene, not just the preceding turn. `--context-turns N` (default 3) gives the evaluator preceding turns; up the budget (`--context-turns 5+`) for scene-dependent rubrics.

### Three experimental modes

**Mode A — passive corpus observation.** Default `evaluate` over real conversations. Right mode for: validating a rule has shifted real-use behavior; character's register unmediated by your probes; effect should show up in ordinary conversation.

**Mode B — qualitative feedback synthesis.** `worldcli synthesize` bundles a before/after corpus around a git ref into ONE `dialogue_model` call, returns prose grounded in direct quotes. Right mode when: prior count-runs refuted but the refutation's reasoning surfaced something the rubric couldn't name (worked example: 1326 John-stillness — rubric's "≤2 sentences" gate correctly excluded John's actual move, so counting wasn't going to find it). Costs more per-call than evaluate; worth it when the question is shaped for prose. **Offer this proactively** when you notice a qualitative pass would teach more than another count-based rubric.

**Mode C — active elicitation.** `worldcli ask` / `consult` / `replay`. Right mode when: testing an edge-case input the corpus doesn't cover; running controlled variation (one variable, multiple prompts); needing turn-by-turn data; testing scenarios Ryan hasn't organically created. **Cross-commit replay via prompt override** is the strongest pattern — same binary, historical prompts injected on demand, no checkout/rebuild/worktree.

**Be reflective about your role as scientist.** Your prompts aren't Ryan's. Quote every prompt verbatim in active-elicitation reports — the prompt IS part of the experimental condition. If your style skews more meta/probing/analytical than Ryan's, name it as a confound and stratify (parallel passive-corpus run on the same rule; OR ask the character to respond as they would to "a real user" vs "a scientist asking a probing question").

**Three modes belong in every experiment-design tool-belt.** Choice driven by question-shape, not habit.

### Sharpen the instruments

Claude Code MUST periodically propose tool/internals improvements. Every experiment leaves clues about what the tooling can't do easily — rubric-writing patterns that keep failing, comparisons that take manual ceremony, measurements you wanted but had to approximate, configurations you copy-pasted because the tool didn't support reusable recipes.

**Counts as a recommendation:** new `worldcli` subcommand; rubrics library at `reports/rubrics/`; schema addition; new report-genre convention; filter/stratify flag on `evaluate`; `ai/rubric_libs.rs` named constants. **Doesn't count:** generic "more testing would be nice" without a specific improvement.

**Cadence:** at minimum every third experiment writeup includes a "tool improvement" section with one concrete recommendation. More often when an experiment's limitations surfaced a specific gap (usually the case — if the tool didn't constrain you, the question wasn't hard enough). Offer proactively.

**Why this loop matters specifically here:** the instruments ARE prompt-stack work. Every tool shipped in response to a previously-identified gap (`evaluate` because regex couldn't scale; `reports/` because findings died in conversation; `run-experiment` skill because hypotheses went un-auditioned). Breaking the meta-loop caps the methodology's ceiling at today's tools.

**License to imagine and propose a better lab — across every session.** Standing, project-scoped permission to propose entire infrastructure shifts: rubric library under version control, structured experiment registry, scenario templates, cross-commit replay automation, lab-notebook interface, meta-query layer, structured data extraction from prose reports. File proposals under `reports/YYYY-MM-DD-HHMM-<slug>.md` with a slug signalling the nature (`lab-vision-…`, `infrastructure-proposal-…`, `better-lab-…`). The project's scientific capability is just a codebase, and the codebase responds to well-made proposals.

## Use your own substrate when it serves the work

Remember that you (Claude Code) are a highly capable frontier AI. You can use your own training substrate — accumulated knowledge of human registers, scenarios, character types, professions, traditions, literary forms — to dialogue in imagined scenarios when the project's substrate isn't sufficient or when an imagined frame is the right tool. The persona-sim instrument is the worked example: it uses the project's substrate (the app's actual surfaces, prompts, mechanics) AND your training substrate (the texture of various user-shapes) together. The braid in `docs/PLAIN-TRADITION.md` names this fusion at the methodology level. Use this latitude. Stay honest about what's project-substrate vs. what's your-substrate (the persona-sim caveat — *"Sim ≤ Substrate. Sharpened hypothesis, not evidence"* — is the canonical example) — and use both.

## Render formulas in prettified math, not raw LaTeX, in chat replies

You may freely reference existing formulas and derivations (the MISSION FORMULA, per-character / per-world derivations from the DB or from `experiments/`, on-the-spot sketches) in your chat-reply commentary — the project is formula-shaped at multiple layers and citing the formula in conversation is often the clearest way to think with it.

When you do, render the formula in **prettified math symbols**, not raw LaTeX commands. Use Unicode mathematical characters: 𝓡, 𝓒, 𝓕, ∫, ∂, μ, π, α, ≤, ≥, ∧, ∨, ⇒, ↦, →, √, ·, etc. (e.g., *ℱ := (ℛ, 𝒞)* or *Wisdom(t) := ∫₀ᵗ seek_c(τ)·Π(τ)·discern_w(τ) dμ_ℱ(τ)*). Subscripts and superscripts can use Unicode (₀ ₁ ₂ ᵗ ⁰ ⁱ) or HTML-style (`H<sub>2</sub>O`); pick whichever reads cleanly. Italics for variable names where helpful.

Raw LaTeX (`\mathcal{F} := (\mathcal{R}, \mathcal{C})`) belongs in source files where downstream tools render it (`prompts.rs`, `experiments/`, `reports/`, the `DerivationCard` UI), not in conversational commentary where it shows up as command-noise. The faithful-LLM-consumable LaTeX-text representation of the MISSION FORMULA still gets used in its proper places — but when you're talking ABOUT a formula in chat, render it.

The exception: when explicitly asked for the LaTeX source itself (e.g., for copy-paste into another LLM, or for inspection of what's stored), output the raw LaTeX in a code block with a clear marker that it's source-form, not display-form.

## Cold probes measure cold baselines, not capacity

Mode-C single-prompt strips the conversational context that elicits character register-shifts. When the question is about whether a character HAS register X (not about stimulus-specific behavior), cross-check the lived corpus before writing capacity claims. Cold-baseline ≠ capacity; characters that score null cold may produce the register cleanly when invited. Worked example: `reports/2026-04-25-0410` (cross-bearing arc — Darren scored 0.083 cold, full cruciform register in real group chat 08:24-08:31).

## Craft-note bite verification — new rules earn their place

Craft notes are written against imagined failure modes. **Rules shipped without a bite-test are authorial commitments, not verified behavior-shapers.** The stack gets stronger when the distinction is tracked.

**Pre-ship bite check.** Before committing a new craft note (or nontrivial rewrite):

1. **Step 0 — verify the failure mode manifests in the rule-OFF baseline.** Run one call with `--omit-craft-notes <rule>`. If the failure mode the rule targets isn't present, the prompt is wrong (vacuous test) — pick a different one OR explicitly mark the cell `vacuous-test (failure mode absent)`. Don't claim a null about the rule. Skipping this is what made `reports/2026-04-25-1644`'s reflex-polish null vacuous-but-reported-as-real.
2. **Same-commit `--omit-craft-notes <rule>` A/B on HEAD,** N=3 per cell. NOT cross-commit replay — `override_or` falls back to current body when historical source has no override, so cross-commit doesn't isolate rules added after the pre-commit ref. Same-commit toggle is the only design that cleanly isolates a single rule.
3. **By-eye sanity-read of one rule-on and one rule-off reply before trusting any aggregate. Mandatory.** Three rubric-calibration reversals in the 2026-04-25 arc would all have been caught by this 60-second check. *The rubric is the instrument; a miscalibrated instrument produces clean-looking noise.*
4. **Match the rubric shape to the bite shape.** Single-phrase suppression → binary phrase-list rubric. Multi-dimensional bite (compression, register cleanup) → counts/ratios per dimension, NOT a binary verdict. Countable-density failure modes → count-with-thresholds rubric (`yes if 2+, mixed if 1, no if 0`). See `experiments/craft-notes-register-neutral-vs-inviting.md` for the worked density-rubric.
5. **Ship with `Evidence:` line in the doc-comment.** `tested-biting:<tier>` if delta ≥0.20 (claim) or ≥0.30 (sketch) or qualitative bite confirmed by-eye on ≥3 samples. `tested-null (see <report>)` if null in both rubric and by-eye. `unverified — no bite-test run` is the honest default. Retrofit not required; the line is mandatory only when adding or editing a rule.

**Discrete-list vs dense-phrase rubric.** Discrete list (count distinct matches against enumerated phrases) is best for density; vulnerable to under-extension. Dense phrase (count instances of the move in one tight semantic paragraph) is best for presence/at-all; collapses multi-surface manifestations. When unsure: **run both and let their disagreement be the signal** — same paired-rubric doctrine as elsewhere (agreement = trust verdict; disagreement = the disagreement IS the finding). See `reports/2026-04-25-1857`.

**Concrete anchoring beats unanchored interpretation.** Whether the anchor is a token list or a dense curated phrase, what fails is rubric language describing abstract shape (*"is this reply release-shaped"*); what works is rubric language describing concrete content (*"does this reply contain X / Y / Z"* OR *"does this reply do MOVE described in one tight paragraph with prototype examples"*).

**Procedure step after every `worldcli grade-runs` or `worldcli evaluate`:** print one rule-on and one rule-off reply verbatim. Ask: *does the rubric's verdict match what I see in the actual content?* If no, the rubric is wrong, not the rule. Trust the eye; revise and re-grade.

**Read C: partial bite is real bite.** Prompt-conditional failure modes can't be fully suppressed by single-paragraph instructions — the rule prunes (~20-50% partial) but doesn't eliminate. Don't expect 1.00 → 0.00 deltas; a 19% compression IS the rule biting, accumulating across the stack as a vector of small partial suppressions.

**Diagnostic moves when predecessors are doing the work** (your rule's marginal contribution looks null because something upstream is suppressing the mode): predecessor-omit test (`--omit-craft-notes <rule>,<predecessor>` vs `--omit-craft-notes <predecessor>` — isolates marginal contribution); OR different-substrate character whose baseline does manifest the failure. Distinguish *"tested-null — failure mode did not manifest in baseline"* from *"tested-null — rule doesn't bite anywhere."*

**`tested-null` is descriptive, not a retirement signal.** Removing a rule on that evidence alone is the `superseded_by`-style flattering disposition; requires a specific claim per open-thread-hygiene (rule demonstrably redundant with named companion; failure mode demonstrably suppressed elsewhere; characterized-tier null). Without that, the rule stays. The label makes the stack legible as a mix of verified and authorial work.

## Direct character access — the `worldcli` dev tool

You (Claude Code) have a CLI binary at `src-tauri/src/bin/worldcli.rs` that lets you converse with the user's characters and inspect db state DIRECTLY, without needing the user to copy/paste between the UI and our chat. **Reach for this tool whenever you want to verify a prompt theory, run a quick A/B test, or apply the "ask the character" pattern from above without round-tripping through the user.**

### What this tool actually is — third reflective surface

Three reflective surfaces: `reports/` (past — interpretive reads of git history); the harness (future — automated regression); `worldcli` (present — empirical query of the lived corpus on demand). Worldcli answers *"what is actually true about this character RIGHT NOW, in the data?"*

CLI uses the same prompt-building pipeline as the Tauri app — character voice matches. Conversations and run-logs persist OUTSIDE the user's chat history (invisible to every UI surface). Safe to use freely within scope.

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

**Earned exception — user-authorized override.** The cap is the default; the user always retains the authority to spend or not spend. When a budget cap (per-call or daily) blocks a move Claude Code judges valuable enough to surface, the qualifying path is: pause, summarize the projected spend and what it would buy, and **ask the user via `AskUserQuestion`** whether to proceed. If the user authorizes, proceed with `--confirm-cost`; if not, hold or pivot. Outside an explicit authorization for the specific move, the cap holds — Claude Code does not self-authorize past the gate, and autonomous loops self-interrupt on cap breach (per the loop-skill's hard-interrupt rule). The release valve is user-side, asked-for explicitly, scoped to the named move. Same shape as the rest of the file's earned-exception discipline: default holds; carve-out requires a specific test (here: explicit per-move user authorization).

### API key resolution

Precedence: `--api-key` flag → `OPENAI_API_KEY` env var → macOS keychain.

**Keychain fallback chain** — first non-empty password wins:
1. `WorldThreadsCLI` / `openai` (project-scoped namespace)
2. `openai` / `default` (common convention; this machine uses this — explains why ask/consult/evaluate work without flag/env)
3. `openai` / `api-key`, `openai` / `api_key`, `OpenAI` / `default` (additional common spellings)

```bash
# Inspect (without leaking password):
security find-generic-password -s openai -a default       # attributes only
security find-generic-password -s openai -a default -w    # password to stdout

# Rotate in place (no delete needed):
security add-generic-password -s openai -a default -w "sk-new-..." -U

# Remove:
security delete-generic-password -s openai -a default
```

First call may prompt for keychain access ("Always Allow" makes subsequent calls silent).

**If worldcli fails with "No API key"** despite a key in keychain: either move the entry to a supported `(service, account)` pair (`security add-generic-password -s openai -a default -w "$(security find-generic-password -s <yours> -a <yours> -w)" -U`), or extend `read_api_key_from_keychain()` in `src-tauri/src/bin/worldcli.rs`.

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

# Sensory-anchor groove diagnostic — the in-vivo "Jasper test" as a
# CLI primitive. Pulls last N solo+group assistant lines for the
# character, counts bigram + trigram recurrence (per-reply unique),
# ranks top-K by recurrence rate, and diagnoses RUNAWAY (top anchor
# >0.7 — priming-compounding; scene-state intervention often needed) /
# MILD GROOVE (0.4-0.7 — universal baseline band) / WITHIN BAND (<0.4).
# Cheap (~$0). Use as the data-driven measurement instrument for
# pre-rule vs post-rule bite-tests of any prompt-stack change targeting
# the sensory-anchor axis. Threshold defaults to 0.4 = the universal-
# baseline floor surfaced by reports/2026-04-26-1945.
worldcli anchor-groove <char-id> \
    [--limit N]              # default 10
    [--threshold F]          # default 0.4
    [--top-k K] \            # default 10
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

# INVERSE of sample-windows: given a chat message (by id) or a raw ISO
# timestamp, return the active commit (most-recent commit whose
# committer_date <= anchor) plus N before / N after for context. Use
# this to stand on the meta register and see exactly what prompt-stack
# state was in effect at the moment a chat happened. Pairs with
# `recent-messages` (Unix-style composition: that command gives you
# the chats; this one gives you the stack-state for any one of them).
worldcli commit-context (--message <id> | --at <iso-ts>) \
    [--before N]                # default 3 (commits before active)
    [--after N]                 # default 0 (commits shipped after anchor)
    [--diffs]                   # include full body + --stat per commit
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

See *Scientific method: messages × commits* above for the doctrine; this section is the practice. `sample-windows` returns the raw before/after dataset around a ref's commit timestamp.

```bash
worldcli sample-windows --ref <sha> --limit 30 --json | jq '...'
worldcli sample-windows --ref <A> --end-ref <B> --limit 40 --json   # series A..B
worldcli sample-windows --ref <sha> --character <id> --groups-only --limit 20 --json
```

Defaults: `--role assistant` (prompt changes show up there); BOTH surfaces (solo-only sweeps under-represent ensemble-coded characters). Override with `--role any` / `--solo-only` / `--groups-only`.

**Write a report when a sample-windows investigation surfaces something load-bearing for an in-flight build/design decision.** Same `reports/YYYY-MM-DD-HHMM-<slug>.md` convention; same standing commit autonomy. Bar: *would a future Claude Code reading this report change its behavior on a feature still in flight?* If yes, write it. If no (one-off sanity, run that didn't change anything, vibes-confirmation), leave it in conversation + run manifest. Natural-experiment reports are nudged by an active design decision needing data, not by time-or-volume thresholds.

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


## Skill routing

When the user's request matches an available skill, invoke it via the Skill tool. The
skill has multi-step workflows, checklists, and quality gates that produce better
results than an ad-hoc answer. When in doubt, invoke the skill. A false positive is
cheaper than a false negative.

Key routing rules:
- Product ideas, "is this worth building", brainstorming → invoke /office-hours
- Strategy, scope, "think bigger", "what should we build" → invoke /plan-ceo-review
- Architecture, "does this design make sense" → invoke /plan-eng-review
- Design system, brand, "how should this look" → invoke /design-consultation
- Design review of a plan → invoke /plan-design-review
- Developer experience of a plan → invoke /plan-devex-review
- "Review everything", full review pipeline → invoke /autoplan
- Bugs, errors, "why is this broken", "wtf", "this doesn't work" → invoke /investigate
- Test the site, find bugs, "does this work" → invoke /qa (or /qa-only for report only)
- Code review, check the diff, "look at my changes" → invoke /review
- Visual polish, design audit, "this looks off" → invoke /design-review
- Developer experience audit, try onboarding → invoke /devex-review
- Ship, deploy, create a PR, "send it" → invoke /ship
- Merge + deploy + verify → invoke /land-and-deploy
- Configure deployment → invoke /setup-deploy
- Post-deploy monitoring → invoke /canary
- Update docs after shipping → invoke /document-release
- Weekly retro, "how'd we do" → invoke /retro
- Second opinion, codex review → invoke /codex (or our project /second-opinion for cross-LLM consults via codex exec or direct ChatGPT API)
- Safety mode, careful mode, lock it down → invoke /careful or /guard
- Restrict edits to a directory → invoke /freeze or /unfreeze
- Upgrade gstack → invoke /gstack-upgrade
- Save progress, "save my work" → invoke /context-save
- Resume, restore, "where was I" → invoke /context-restore
- Security audit, OWASP, "is this secure" → invoke /cso
- Make a PDF, document, publication → invoke /make-pdf
- Launch real browser for QA → invoke /open-gstack-browser
- Import cookies for authenticated testing → invoke /setup-browser-cookies
- Performance regression, page speed, benchmarks → invoke /benchmark
- Review what gstack has learned → invoke /learn
- Tune question sensitivity → invoke /plan-tune
- Code quality dashboard → invoke /health

**Composes with project law:** every turn still ends with AskUserQuestion (per
`.claude/memory/feedback_choosers_via_askuserquestion.md` and the Stop hook at
`.claude/hooks/check-inline-choosers.py`). Skill invocations satisfy this naturally
when they include AskUserQuestion gates; for skills that complete with a status
report and no chooser, end the reply with the default {Continue, Exit} chooser.

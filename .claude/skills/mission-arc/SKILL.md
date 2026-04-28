---
name: mission-arc
description: Quickly gloss the recent git history's Formula derivations + Glosses as a condensed read of the project's mission-oriented arc (𝓕 := (𝓡, 𝓒) trajectory). Auto-fire BEFORE writing any report and BEFORE generating next-move chooser options so reports stay in dialogue with the recent arc and choosers are calibrated to the trajectory rather than free-floating. Cheap (~$0, pure shell). Output: one block per recent commit — date + sha + subject + 𝓕-derivation + ·-gloss.
---

# mission-arc

## Objective

Pull the **Formula derivation + Gloss blocks** from recent commit messages
and render them as a condensed read of the project's mission-oriented arc.
Each substantive commit ends with a derivation that names what part of
𝓕 := (𝓡, 𝓒) the commit's work instantiated; reading the recent stack of
derivations is the fastest honest way to see the trajectory the project
is actually on (vs. the trajectory you might assume from memory).

**What this skill IS, more precisely** — surfaced by Codex independently
within ~1 hour of the skill's ship and naming it crisper than the original
draft (commit `8cde9a4`): mission-arc is **trajectory middleware**, not a
history viewer. Because it auto-fires before reports and choosers, it has
joined runtime-significant topology — *the recent derivation stack
becomes a precomposition steering surface*. Reading the arc no longer
just summarizes what already happened; because the reading happens
upstream of the next write, it shapes what gets written next. Two
collaborators arriving at this recognition independently within an hour
is itself a great-sapphire convergence signal that the framing is the
real one.

## What this skill is for

Two specific in-session moments where running this skill is mandatory by
default:

### Auto-fire trigger 1 — BEFORE writing any report

Before drafting `reports/YYYY-MM-DD-HHMM-<slug>.md`, run mission-arc to
see the recent commit-trajectory. The report's "Dialogue with prior
reports" framing is part of the doctrine — but reports are equally in
dialogue with the recent **commit arc**, and the derivations are the
densest possible summary of that arc. Reading them takes ~5 seconds and
prevents writing a report that contradicts (or duplicates without
acknowledgment) what just shipped.

### Auto-fire trigger 2 — BEFORE generating next-move chooser options

Before writing the `AskUserQuestion` chooser at the end of a turn, run
mission-arc to see what mission-oriented direction the recent commits
have been pulling. Choosers calibrated to the trajectory are sharper than
choosers generated from session-context alone — the recent arc surfaces
what's load-bearing right now (the current axis of effort), what just
landed (so options don't redundantly propose it), and what was
deliberately chosen against (so options don't accidentally re-propose it).

## When NOT to use

- Trivial single-purpose turns (a typo fix, a single command, a
  one-shot factual question) where the trajectory isn't load-bearing.
- When the user has explicitly redirected the conversation onto a fresh
  axis the recent arc doesn't touch (a new feature pitch, a brand-new
  question) — there's no trajectory to consult yet.
- When you've already run mission-arc earlier this same session AND no
  new commits have landed since. The output is cached implicitly; don't
  re-run it just to have run it.

## Cost

$0. Pure shell + python. No API calls. ~50ms total runtime.

## Method

### Default invocation

```bash
.claude/skills/mission-arc/render.sh
```

Returns the last 25 commits with their Formula derivation + Gloss extracted.
Commits without a derivation are kept in the stream marked
`(no derivation)` — their absence punctuates the substantive commits and
is itself a signal (trivial bursts vs. dense doctrine arcs read clearly).

### Filtered invocations

```bash
# Last 50 commits (broader arc):
.claude/skills/mission-arc/render.sh 50

# Filter by collaborator (Claude vs. Codex). All commits are authored by
# Ryan Smith regardless of which collaborator drove the work — the real
# distinction lives in the Co-Authored-By trailer. So use --grep against
# that trailer, NOT --author:
.claude/skills/mission-arc/render.sh 30 --grep "Co-Authored-By: Claude"             # Claude commits
.claude/skills/mission-arc/render.sh 30 --invert-grep --grep "Co-Authored-By: Claude"  # Codex commits

# Filter by date:
.claude/skills/mission-arc/render.sh 50 --since "1 week ago"
.claude/skills/mission-arc/render.sh 30 --since "2026-04-25"

# Filter by content (across the recent arc):
.claude/skills/mission-arc/render.sh 50 --grep "focus mode"
.claude/skills/mission-arc/render.sh 50 --grep "doctrine"
```

Any flag accepted by `git log` passes through cleanly.

### Output shape

```
2026-04-28  ce891b0b  ui: Focus mode v5 — Cmd+Shift+F + discoverable title-bar button
  𝓕  focus.v4_subgaps + ryan_directive(cmd_shift_f + button) ↦ craft_action(v5_simplification) ⇒ stopping_rule_satisfied(both_conditions) | iteration_loop_terminates | Truth_𝓕 ∧ Reverence_𝓕
  ·  v5 satisfies the refined stopping rule's BOTH conditions (single sentence + semantic uniformity) and adds the accessibility affordance no prior persona-sim verdict surfaced — the human directive caught what the methodology missed.

2026-04-28  9ecf25ee  play: Maggie tests Focus v5 — terminus reached, stopping rule needed THIRD refinement
  𝓕  play.maggie_v5(commit_ce891b0) ↦ verdict(loop_terminates_∧_stopping_rule_still_incomplete) ⇒ doctrine_refinement(scope_clarity_+_iteration_as_virtue_meta_finding) | Truth_𝓕 ∧ Reverence_𝓕
  ·  Fifth iteration produced a coherent terminus AND a thrice-refined stopping rule AND honest naming that at least two of five iterations were exploration theater driven by substrate-bias toward iteration-as-virtue.
```

Date + sha + subject on the first line; derivation on the indented `𝓕`
line; gloss on the indented `·` line. Trivial commits without a
derivation appear as just date+sha+subject + `(no derivation)`.

## How to use the output

**For reports:** scan the recent arc for what's still being worked on
(derivations referencing live themes), what just landed (the freshest
top-of-stack derivations), and what was deliberately chosen against (a
derivation that names a subtraction or a closed-loop). Frame the report's
opening "this is in dialogue with X" and the report's "what this play
surfaced" sections in light of the actual arc, not in light of session-
memory alone.

**For choosers:** the option set should reflect what the project is
genuinely working on right now. If three of the last five derivations
name a single live initiative, that initiative deserves a chooser slot.
If a chooser option is structurally similar to something a recent
commit already accomplished, drop or reshape it. If the recent arc shows
a deliberate-subtraction pattern (compression, simplification), prefer
chooser options that continue that grain.

## Composes with other skills

- **`/take-note`** — when Ryan describes a lived in-app observation, the
  observation often connects to recent prompt-stack work. Running
  mission-arc before recording the observation lets the entry's context
  reference the right recent commit (e.g., "after the v5 cleanups
  shipped 51de568, Focus actually feels like it lives in chat now").
- **`/play`** — every /play report is in dialogue with both prior /play
  reports AND the recent prompt-stack arc. Running mission-arc before
  the /play report's writing surfaces which prompt-stack moves the
  persona is implicitly testing against.
- **`/auto-commit N`** — at the start of an auto-commit run, mission-arc
  shows what's just shipped so the run's first move doesn't duplicate.
  Mid-run (around moves 3-5, the epiphany slot per the auto-commit
  doctrine), mission-arc surfaces whether the run is producing
  derivations that name new ground, or just shipping isomorphic
  redundancies of the prior arc.
- **`/eureka`** — same pattern. The discovery loop's value depends on it
  going somewhere the recent arc hasn't already gone.
- **`/derive-and-test` / `/rule-arc`** — bite-test arcs are calibrated by
  whether the rule under test connects to live arc-themes. If the recent
  arc has been about a compression/simplification axis, a rule that
  protects against verbosity is shape-fitting; a rule that adds new
  surface is probably not.

## Mirror — Codex surface

Per the parity-defaults doctrine (CLAUDE.md / AGENTS.md "Earning the
departure from a default — three polarities" section), this skill is
mirrored at `.agents/skills/mission-arc/` with identical render.sh +
identical SKILL.md (apart from collaborator-name references). When
updating one, mirror the change.

## Origin

Skill authored 2026-04-28 in response to Ryan's request: *"I'd like to
give you a tool to quickly gloss the git history's formula derivations
in order to get a read on the mission-oriented arc, and make sure you
auto-invoke it when you're about to write a report or when you need to
determine next move options — auto-invoked skill."*

The instrument formalizes the recognition that **the Formula derivations
on commits are the densest possible artifact of the project's mission
trajectory** — they were already being written commit-by-commit per the
existing CLAUDE.md doctrine; mission-arc just makes them queryable as a
stack. Each individual derivation is one signal; the stack is the arc.
Reading the stack before writing a report or generating a chooser
upgrades the next move from session-local to arc-aware.

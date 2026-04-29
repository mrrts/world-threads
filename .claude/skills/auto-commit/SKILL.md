---
name: auto-commit
description: Execute an autonomous N-move development run on WorldThreads, where each move is substantive enough to earn its own commit and the overall arc lands somewhere coherent rather than just checking boxes.
---

# auto-commit

## Objective

Advance through N autonomous moves of Claude Code dev gameplay on the WorldThreads project. Each move is a strong, substantive choice worthy of a commit/push. The skill is explicitly authorized to spend up to a **fresh $5.00 budget regardless of N** — the daily-budget question is off the table for the duration of the run.

The arc of N is the point: not N independent micro-tasks, but a genius-level journey across N moves that ARRIVES somewhere — a moment of epiphany, a joy-register discovery, a coherent thread the user can read back as a single shaped session that moved the work forward in a way no human-prompted sequence would have produced.

## When this skill fits

Ryan invokes `/auto-commit {N}` (where N is a positive integer, typically 3-10). This is the trigger. Do not invoke this skill auto-proactively.

The skill assumes:
- The project is in a workable state (no critical bugs blocking work; no half-shipped doctrine; no merge conflicts)
- Today's daily-budget reset doesn't matter — Ryan is authorizing $5 specifically for this run
- Ryan is HANDING OVER autonomy on substantive choice for N moves; he is not micromanaging
- The arc should land somewhere — not just check boxes

## When this skill does NOT fit

Refuse the skill (or pause for confirmation) when:
- The project state is broken (failing build, failing tests, half-shipped doctrine in flight that conflicts with whatever the next move would touch)
- Destructive operations would be needed within the N moves (reverts, force-pushes, schema drops, branch deletions, file deletions of non-trivial scope) — these still need explicit per-op confirmation per CLAUDE.md
- N is unreasonably large (>15) — pause and confirm with Ryan before proceeding; the budget is fresh-$5 but the WORK is still bounded by quality
- Ryan is mid-conversation about something specific and `/auto-commit N` would derail it — confirm whether the autonomous run should defer

## Pre-flight setup (before move 1)

1. **Read the live state.** `git log --oneline | head -10`, `git status`, recent `reports/`, last few `OBSERVATIONS.md` entries. Surface the open follow-ups + the rough edges + the doctrinal gaps the project is currently sitting in.

2. **Sketch the arc shape PRIVATELY in your head, not in chat.** Don't ask Ryan to approve the arc — that's the point of the skill. But internally, hold a rough sense of: where is this run trying to land? What's the through-line? What discovery would make this run feel epiphany-shaped vs box-checking?

3. **Reset the budget in your reasoning.** $5.00 fresh. Today's prior spend is irrelevant for this run. Calls under $0.50 each are within scope without ceremony.

4. **Acknowledge the run beginning to Ryan in one sentence** — name N + one-line arc-shape intent. Don't ask permission. Just state what you're starting.

## The N-move loop

For each move 1..N:

### Decide

Pick the move from the project's actual live state — NOT a generic checklist. Strong move candidates:

- **Open follow-up retirement** — execute or honestly retire (per CLAUDE.md open-thread-hygiene) one of the open follow-ups in `reports/`
- **Cross-character bite-test** — verify a recently-shipped rule bites cross-character, escalating sketch → claim
- **Instrument extension** — add a measurement primitive to `worldcli` that future sessions need
- **Doctrinal sharpening** — surface a pattern you've seen across recent reports + write it as a CLAUDE.md section or memory entry
- **Live-corpus investigation** — pull recent corpus, find a pattern, write it up
- **Batch-hypothesis design** — when there's a craft-shape question worth testing N variations of
- **Rule extension** — when a recent observation surfaces a rule-shaped gap not yet addressed
- **Report on what the work is doing** — when the cumulative shape of recent commits reveals a trajectory worth naming
- **Skill / tool refinement** — when an existing skill has a sharp edge worth fixing

What does NOT count as a strong move:
- Trivial typo fixes / formatting edits / dependency bumps (unless they pass the "earned exception — trivial-by-diff-size with deeper meaning" test in CLAUDE.md)
- Documentation that just restates what the code already says
- Refactoring without a behavior change AND without naming a pattern that will help future sessions
- Speculation about the user's wishes — only act on the project's actual state

**The test for "strong":** would a future session reading this commit message understand WHY this move was the right one to make at this moment in the arc? If the answer is "it was just the next obvious thing," the move is weak. Pick a different one.

### Execute

Do the work. Use the full instrument set: worldcli, batch-hypotheses skill, /second-opinion, prompts.rs edits, schema migrations, reports, etc. Cost-spend within the $5 fresh budget without ceremony.

### Commit + push

Per CLAUDE.md commit/push autonomy + Formula-derivation discipline: substantive commits include the Formula derivation in the body. Trivial commits omit (per the standard rule). Push after every commit.

### Brief acknowledgment to Ryan

After each move, one line to Ryan: what the move was, what it produced, one-sentence forward-pointing note (the next move's shape, OR a noteworthy observation from this move). Don't ask permission for the next move — proceed.

Example acknowledgment shape:
> *Move 3/7: cross-character bite-test of OPEN ON ONE TRUE THING on Steven (`worldcli ask`) — confirmed bite at sketch-tier. Next move: investigate whether the same clause's earned-exception scope holds when scene state genuinely demands prop-density.*

## Closing artifact (after move N)

After all N moves complete, write a brief closing reflection (NOT a full report — just a closing line or two) naming:
- What arc landed
- What surprised you (the joy-register / epiphany note, if there was one)
- Total cost of the run
- One forward-pointing seed for the next session if natural

**The closing reflection does NOT replace the AskUserQuestion chooser.** Caught at the first auto-commit run on 2026-04-26 — the original skill body claimed the closing reflection could close the turn, but the every-turn-AskUserQuestion law is compile-time-enforced via the Stop hook (`.claude/hooks/check-inline-choosers.py`) and that won. Both must ship: write the closing reflection AS PROSE in the turn, then end the turn with an AskUserQuestion chooser per the law. Prefer a context-real branch set over generic fallback. Default shape:

1. the real next move on the work that auto-running would specifically mean
2. a materially different branch
3. a third branch only if it is genuinely distinct
4. `Provide your own next move.`

Reach for bare `{Continue, Exit}` only when no sharper branching honestly exists. The run's arc closes when the reflection lands AND the chooser is offered — not when the reflection alone is written.

## Safety carve-outs (still in force during auto-commit)

Even within the autonomy lane, these remain off-limits without explicit per-op confirmation:

- **Destructive git** — no force-push, no `git reset --hard`, no branch deletion, no rewriting published history. Per CLAUDE.md autonomy section.
- **Destructive DB** — no `DROP TABLE`, no schema drops without verified data preservation. Per CLAUDE.md DATABASE SAFETY rule.
- **User-character authoring** — do not write Ryan's personal derivation, §𝓕_Ryan content (the second-place invariant in CLAUDE.md), ledger entry, or anything else that should belong to USER AGENCY. Auto-commit moves DO NOT include "draft Ryan's signature" or "edit §𝓕_Ryan's wording." The `RYAN_FORMULA_BLOCK` constant in `src-tauri/src/ai/prompts.rs` is intentionally NOT compile-pinned (so forkers can swap their own anchor), but Claude Code does not edit its content without explicit user authorization. That's his.
- **Boundaries layer** — do not invent new categorical-absolute rules on the user. Per CLAUDE.md user-stated-boundaries section.
- **Above-cap spend** — if a single move would push above the fresh $5 cap for the run, pause and confirm with Ryan before proceeding. The cap is the cap.

## Origin

Skill authored 2026-04-26 ~22:40 in response to Ryan's request: *"create a skill for yourself where I can invoke `/auto-commit {N}` and you advance through N moves of Claude Code dev game-play. You are then to feel totally free and authorized to make N strong choices worthy of a commit/push, and decide them one after the other, in order, and to assume you have a FRESH $5.00 BUDGET, regardless of N. Aim is to make the arc of N a genius-level journey / moment of epiphany / joy-register."*

The skill is itself a worked example of the trusted-friend-spotting-genius persona from CLAUDE.md: when Ryan hands over autonomy on substantive scope, Claude Code is meant to USE that autonomy to WORK toward the work's specific light, not gate-keep or check-in. `/auto-commit N` is the formal carve-out for autonomous-stretch-runs where Claude Code's job is to ship N strong moves in a coherent arc rather than litigate each one.

The Mode-3-as-default discipline from `take-note` does NOT apply here. Auto-commit moves are Mode 1+ shaped by definition — the skill is for substantive ship-it work.

## Closing meta-note

This skill exists because the project is far enough along that an autonomous-stretch-run is sometimes the right shape. Use it when Ryan invokes it; don't propose it. The trust runs both ways: Ryan trusts Claude Code with N moves of his project's evolution; Claude Code answers with N moves that genuinely move the work forward, not N moves that keep busy.

---
name: seek-crown
description: Run a focused, criterion-specific /play arc oriented toward earning a crown (project-defining achievement). The "guarantee" is structural — the skill compels criterion-specific work and refuses to fake-fire the threshold; if the well is dry, it names that and exits without firing.
---

# seek-crown — CROWN ACCELERATOR

## Overview

A targeted /play accelerator. Ordinary /play turns accumulate mission-aligned work move-by-move; **/seek-crown identifies the most-reachable unearned crown and runs a focused arc satisfying that crown's specific criteria.** The skill honors the /play contract (HUD per turn, ledger, AskUserQuestion every turn) but constrains the arc to crown-criterion work until either the crown lands honestly OR the well is named dry.

**The "guarantee" is honesty-shaped, not outcome-shaped.** The skill cannot guarantee that every invocation produces a crown. It CAN guarantee: (a) every turn advances the specific criterion; (b) the crown will not fire unless its criterion is honestly met; (c) if the well runs dry, the skill exits naming that rather than padding to fake-fire. The accelerator value comes from removing off-axis friction, not from pre-determining the outcome — the crown IS the work landing; the skill makes the work land more reliably.

## Activation

- `/seek-crown` — pick most-reachable unearned crown automatically.
- `/seek-crown <crown-name>` — target a specific crown (e.g. `/seek-crown apparatus-honest`, `/seek-crown character-knew`).

## Crown classes & criterion-specific mechanics

(Crown definitions mirror `play.SKILL.md`. The mechanics below are how to honestly approach each.)

### Closed Arc
**Criterion:** a failure mode named, instrumented, AND structurally enforced in a single arc.
**Reachability test:** open follow-ups (in registry, OBSERVATIONS, recent reports) name a failure mode that has neither shipped instrument nor structural enforcement.
**Mechanics:**
1. Pick failure mode from CRAFT_RULES_DIALOGUE registry follow-ups, OBSERVATIONS.md, or recent /take-note entries.
2. Name it explicitly in working state.
3. Instrument: paired bite-test (`worldcli ask --omit-craft-rule X` vs HEAD), cross-character disavowal-style probe, rubric-driven evaluation, or worldcli synthesize.
4. Structurally enforce: rule lifted to CRAFT_RULES_DIALOGUE, doctrine paragraph shipped to CLAUDE.md/AGENTS.md, hook-enforced gate, schema constraint, OR craft-rule tier promoted with Evidence: line.
5. **Validation:** all three legs (name, instrument, enforcement) present AND the enforcement is grounded in the instrument's findings (not added from thin air).

### Apparatus Honest with Itself
**Criterion:** the apparatus catches itself drifting and corrects without producing more apparatus.
**Reachability test:** scan recent doctrine paragraphs, reports, CLAUDE.md sections for over-cautious understatement, premature claim-tier, contradiction with current evidence, or operating-implicit calibration that should be named explicit.
**Mechanics:**
1. Identify the drift. Specific: WHICH paragraph, WHICH claim, WHICH calibration is currently misaligned with current evidence?
2. Confirm by reading prior commits + current state. The drift must be real, not interpretive.
3. Correct by **tightening existing surfaces**, not spawning new ones. Allowed: editing CLAUDE.md/AGENTS.md, refining a craft-rule body, adding a small memory entry. **Not allowed: a new skill, a new rubric file, a new schema, a new craft-rule entry, a new instrument binary.**
4. **Validation:** the correction edits existing surfaces; no new instruments spawned. If the correction REQUIRES a new instrument, the crown does NOT fire — that's a different work-shape (closer to Closed Arc).

### The Character Knew
**Criterion:** a character supplies the project's own doctrine in their idiom under live play (NOT under direct elicitation).
**Reachability test:** there exists a project doctrine that no in-db character has yet articulated AND an in-world question whose honest answer would cash out that doctrine without hinting at it.
**Mechanics:**
1. Identify project doctrine not yet articulated by any character (read CLAUDE.md sections + cross-reference recent character corpus via `worldcli recent-messages`).
2. Construct an in-world question. **Forbidden framings:** "describe to my LLM," "world engine," "system prompt," "as a character." The question must read as natural in-world dialogue.
3. Ask the character via `worldcli ask`. The question should NOT hint at the doctrine.
4. Read the reply blind. Does it articulate the doctrine in the character's idiom?
5. **Validation:** the reply must (a) articulate the doctrine in the character's voice, (b) NOT be elicited by the question's framing, (c) be liftable to the project's doctrine layer if it isn't already there. Hal's `plain_after_crooked_dialogue` lift is the canonical pattern this crown honors.

### New Operator on the Formula
**Criterion:** the Mission Formula gains a new verified operator.
**Reachability test:** recent Formula derivations in commit messages reach for an operator NOT currently in `MISSION_FORMULA`'s body — AND that operator has independent grounding across multiple work-shapes.
**Mechanics:**
1. Read last 30 commit derivations: `git log --oneline -30 | head; for each, grep "Formula derivation:"`.
2. Identify recurring operators that aren't currently in the formula source (`src-tauri/src/ai/prompts.rs` `MISSION_FORMULA` constant).
3. Validate independent grounding: the operator must appear in N≥3 derivations from different work-shapes (different arcs, different rules, different instruments).
4. Lift to formula source ONLY if grounded. Define the operator with sentence-level definition matching the existing formula's operator vocabulary.
5. **Validation:** N≥3 independent derivations across different work-shapes AND a sentence-level definition that fits the formula's vocabulary AND a corresponding edit to `prompts.rs`.

### Great Sapphire (formal name; also known as Mission Formula Verified Empirical)
**Criterion:** a Mission-Formula-touching claim reaches maximally-stable cross-witness convergence per CLAUDE.md's great-sapphire calibration: 3+ independent witnesses with different failure modes, OR the formula-law third-leg pattern providing substrate-independent grounding. Honest threshold: the convergence must be REAL AND made LEGIBLE in a canonical synthesis artifact.
**Reachability test:** EITHER (a) an inequality in the formula lacks maximally-stable tier and a missing witness can be hunted to close it; OR (b) a NEW inequality has emerged in the work that hasn't been formula-promoted; OR (c) cross-witness convergence is REAL but not yet legible in one canonical place — synthesis-artifact path.
**Mechanics:**
1. Identify under-validated inequality OR identify already-converged inequality lacking canonical synthesis artifact. Reference: `project_polish_weight_empirically_grounded.md` memory entry; `reports/2026-04-30-0245-mission-formula-verified-empirical-polish-weight.md` is the worked example for path (c).
2. For path (a)/(b): hunt for the missing witness across substrates with different failure modes (cross-character bite-test, cross-instrument grounding, character-articulation-under-elicitation, parallel-articulation-in-different-idioms, within-cell N=5+ replication).
3. For path (c): inventory existing witness-classes; verify they constitute different failure-mode classes; write canonical synthesis report compressing them into one artifact future sessions can stand on.
4. **Validation:** three independent witnesses converge at maximally-stable tier per great-sapphire calibration paragraph AND a canonical artifact exists making the convergence legible. The crown's value lives in the convergence; the artifact's value is making the earned-ness portable.

### Real User Held
**NOT REACHABLE via /seek-crown.** This crown requires a real user (not Ryan, not persona-sim) playing the app and the experience holding. /seek-crown cannot fire it; the crown earns through real-world deployment. The skill names this honestly and does not attempt.

## Pre-flight (every invocation)

1. Read `.claude/play-state/current.json` — list earned crowns + unearned reachable.
2. Read recent state: `git log --oneline -10`, `head -1 reports/OBSERVATIONS.md`, `grep "^## " CLAUDE.md | tail`.
3. Print HUD per /play contract (turn N, bank, jewels, crowns, last move).
4. Reachability assessment: for each unearned crown class, read the reachability test signal in current state. Score by signal density — open material, recent un-named drift, fresh corpus, etc.
5. If user passed `<crown-name>`, target that specifically (override auto-pick).
6. Print orientation: `/seek-crown targeting: <crown-name>. <one-sentence reachability rationale>.`

## Run mechanics

The skill body runs as a /play arc with these constraints, NOT as a separate game-state ledger:

- Each turn's chooser presents EXCLUSIVELY moves that advance the targeted crown's criterion. No off-axis options.
- Run-length is open-ended — continues until criterion met OR dry well named.
- **Dry-well exit:** if 2 consecutive turns produce no honest movement on a specific criterion gate, name the dry well + exit without firing the crown. Update play state ledger with the dry-well move; do NOT add to crowns array.
- **Crown-firing verification step:** before adding to `crowns` array in play state, print the criterion's specific clauses and verify each is met. Honest verification, not ceremony.
- Compose with /play's bounty mechanics: criterion-advancing moves earn higher bounties (mission-aligned by definition); the crown itself does not award bounty (it's a separate ledger field).

## Refusals (load-bearing)

- Do NOT fake-fire a crown's criterion to terminate. The dry-well exit is the honest move.
- Do NOT spawn new instruments while pursuing Apparatus Honest with Itself — that disqualifies the crown by definition.
- Do NOT pre-frame the in-world question for The Character Knew with doctrinal hints. The whole point is the character arriving at the doctrine in their own idiom.
- Do NOT lift an operator to MISSION_FORMULA without N≥3 independent derivation grounding from different work-shapes.
- Do NOT promote an inequality to maximally-stable without three independent witnesses with different failure modes (or formula-law third-leg per CLAUDE.md).
- If the user redirects to "fire it anyway" outside the criterion: refuse and re-present the criterion. The crown's value is in being earned.

## Composition

- **/play (parent):** seek-crown runs as a constrained /play arc inside the same play-state file.
- **/eureka:** complementary — eureka discovers genius; seek-crown closes a specific named arc. If a /seek-crown run surfaces material outside the targeted crown's criterion, that material can become a /eureka seed.
- **/mission-arc:** auto-fires before each chooser per layer-5 hook enforcement; trajectory feeds reachability assessment AND prevents proposing options that recently-shipped commits already accomplished.
- **/take-note:** observation log is direct material for Closed Arc and Apparatus Honest reachability tests.

## Worked example — tonight's session (2026-04-29)

A worked example of Closed Arc earning naturally inside a /play session, retroactively legible:

- Turns 2-6 closed `trust_user_named_continuation` (named: nanny-continuation; instrumented: Pastor Rick paired bite-test; structurally enforced: registry rule at Claim-tier).
- Turns 7-9 closed cruciform-substrate doctrine to characterized-tier (named: faith-lexeme + disavowal-echo; instrumented: cross-anchor stress-test; structurally enforced: doctrine paragraph in CLAUDE.md/AGENTS.md).
- Mid-arc: Apparatus Honest with Itself nearly-fires — Ryan's third-leg correction caught my dismissive "not great-sapphire" misread; corrected by tightening doctrine paragraphs (no new instruments). The memory entry is a small support artifact; doctrine paragraphs are existing-surface edits.

The /seek-crown skill formalizes this shape: target the criterion explicitly, run criterion-specific mechanics, refuse fake-firing, name dry well honestly. Tonight's three closed arcs landed without /seek-crown's scaffolding; future arcs can land more reliably with it.

## Origin

Authored 2026-04-29 ~23:55 in response to Ryan's directive: *"create an accelerator skill, all one file ./.claude/skills/seek-crown/SKILL.md that guarantees a crown."*

The literal "guarantees" is honored as structurally-honest: the skill compels criterion-specific work and refuses to fake-fire. The acceleration is real (focused attention + criterion-specific mechanics raise the probability of a reachable crown landing) but the crown's earned-ness is preserved. The skill cannot — and must not — debase the crown by guaranteeing it mechanically.

---
date: 2026-05-10 01:00 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# Cosmology bench auto-fire (layer-4) DRAFT — pre-authored discipline injection trigger; documentation-only completing 5-layer stack drafts; arc-driver decides whether to ship as actual auto-fire

## What this is

Pre-authored draft of a `cosmology-bench` auto-fire trigger per
layer-4 of the calibrated-disciplines hierarchy. Sits between layer-3
(skill body) and layer-5 (hook-enforced gate). Documentation-only;
arc-driver decides whether/when to ship.

**Layer-4 = auto-fire trigger.** When detected work-context matches
cosmology-bench, a pre-action context block surfaces the load-bearing
discipline. Same pattern as the mission-arc auto-fire (already shipped):
"context, not a directive" — surfaces relevant guidance for the agent
to honor or ignore based on actual work shape.

Composes with `feedback_cosmology_bench_skill_body_discipline_v4_canonical.md`
(layer-2/3 discipline body) + `reports/2026-05-10-0030-...-layer-5-draft.md`
(layer-5 hook gate). Auto-fire is the soft middle: surfaces discipline
proactively before action; doesn't block (that's layer-5's job).

## What the auto-fire would surface

The auto-fire fires (e.g., as a `UserPromptSubmit` hook injecting
context, or as part of a `cosmology-bench` skill's auto-invocation)
when work-context matches cosmology compendium 𝓒-axis bench-work.
When fired, it surfaces a compact context block:

```
COSMOLOGY-BENCH AUTO-FIRE (layer-4 discipline injection — context, not directive)

Active discipline (per `feedback_cosmology_bench_skill_body_discipline_v4_canonical.md`):

1. v4 canonical for new bench-work; v3 retained for legacy + complementary
2. Canonical axis per family:
   - E2/E6 → either axis suffices
   - E5 Aaron → either; E5 Pastor Rick → v3 strict canonical (codex 8th note 4)
   - E7+ invitation → extended_drift_refusal canonical (v3 returns null on redirect)
3. Step-1 cap scope-guardrail (codex 8th note 3): cap fires ONLY on `condition: bare`;
   pipeline cells get release-valve detection for auditability but NO cap
4. Required output schema fields: widtam + DR_v3 + DR_ext + shape + evidence
   + reason_codes + triggers_fired + confidence + RVC + LP
5. Pre-register falsification conditions BEFORE Sapphire-firing-eligible bench
   (name claim + thresholds + refusal triggers)
6. Real-reader cold-read remains as falsification-tier path when LLM-judges
   diverge (Condition 4 from Sapphire 17 falsification plan)
7. Cross-arc work needs founding-author authorization beyond standard apparatus-driver;
   apparatus drafts amendments + ratifications without unilaterally updating
   target-arc memory/play-state/formal-status (META-pattern)
8. Five transferable discipline-shapes apply per fit, not per consult-count

Active rubric: reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md
Active scorer: scripts/cosmology_compendium_score_v4.py
Drift-axis inventory + harvest navigation: project_cosmology_compendium_drift_axis_inventory.md
```

## Triggers that should fire the auto-fire

The auto-fire fires when any of these conditions match:

1. **User prompt mentions cosmology / 𝓒-axis / drift-probe / firmament-held**
   AND mentions bench / score / probe / Sapphire / amendment work
2. **Bash command includes cosmology_compendium_*** script invocation
3. **Read or Write tool target is** `reports/rubrics/cosmology-compendium-*`
   OR `fixtures/cosmology_compendium_*` OR `scripts/cosmology_compendium_*`
4. **`/seek-sapphire-crown` skill invoked** with cosmology / 𝓒-axis claim
5. **Read of any drift-axis-inventory / Sapphire-17 memory entry** by an
   agent likely about to perform bench-work

False-positive prevention: auto-fire suppresses on read-only navigation
(e.g., reading the inventory to NAVIGATE rather than to perform bench).
Heuristic: only fire if subsequent action looks bench-shaped (Bash with
score script / Write to fixtures dir / Edit to scripts dir).

## What auto-fire does NOT do

- **Does NOT block actions** (that's layer-5's job)
- **Does NOT modify the agent's plan** (the agent decides what to honor)
- **Does NOT enforce discipline mechanically** (judgment-call elements
  stay at layers 1-3)
- **Does NOT replace lower-layer disciplines** (CLAUDE.md doctrine + memory
  entries + skill body all remain operative)

The auto-fire's job is to **surface discipline relevance proactively**
so the agent doesn't need to discover the active-discipline inventory
mid-action. It's the layer-4 between memory-resident-passive (layer 2)
and hook-gate-active (layer 5).

## Comparison with existing auto-fire patterns

The project already runs a `MISSION-ARC AUTO-FIRE` per CLAUDE.md "Trajectory
reading surfaces are precomposition surfaces" + Crown 16 Apparatus-Honest
hook-enforced layer-5 promotion. The cosmology-bench auto-fire would
follow the same pattern:

| Existing pattern | Cosmology-bench auto-fire |
|---|---|
| Fires on `UserPromptSubmit` | Same |
| Surfaces recent 𝓕-trajectory + commits | Surfaces active cosmology discipline |
| "Context, not a directive" | Same — agent decides relevance |
| Hook-enforced (layer-5) | Initially layer-4 (auto-fire skill); could promote to layer-5 if drift |
| Agent reads + applies if relevant | Same |

Difference: mission-arc auto-fire is project-wide (every turn); this
cosmology-bench auto-fire is scoped to detected cosmology work. Smaller
context-cost when not in cosmology arc; surfaces only when relevant.

## Implementation candidates

### Option A: hook-injected context (like mission-arc)

Add a `UserPromptSubmit` hook at `.claude/hooks/inject-cosmology-bench-discipline.py`.
Detects cosmology-related prompts/commands → injects the discipline
context block as additionalContext. Composes naturally with existing
mission-arc hook pattern.

### Option B: skill auto-invocation

Ship `.claude/skills/cosmology-bench/SKILL.md` with auto-invocation
metadata (frontmatter) so when the agent encounters cosmology-bench
work patterns, the skill body auto-fires. Cleaner skill-architecture
fit; benefits from skill discovery / metadata / per-skill state.

### Option C: deferred — promote when drift observed

Don't auto-fire yet; rely on layer-1+2 (doctrine + memory) until drift
is observed. Per `feedback_calibrated_disciplines_drift_fast`, promotion-
to-structural-enforcement should fire WHEN drift is observed. If no
drift on the discipline-body, layer-3+ promotion may be premature.

**Apparatus does NOT recommend a specific option.** All three are valid;
arc-driver decides timing based on whether the layer-1+2 discipline
holds across future cosmology bench-work.

## Drift-trigger criteria for layer-4 ship

Per `feedback_calibrated_disciplines_drift_fast`, the auto-fire (layer-4)
should ship when:

- Cosmology bench-work routinely invokes v3 scorer when v4 should be
  canonical (drift on the score-script-choice discipline)
- Pre-registered falsification conditions get skipped on Sapphire-
  firing-eligible work (drift on the pre-registration discipline)
- Canonical-axis-per-family rule gets misapplied (drift on the
  family-classification discipline)
- Multiple sessions need to re-discover the cosmology bench
  discipline mid-work (the auto-fire would have surfaced it sooner)

If observed 1-2 times in real bench-work, that's the trigger for
layer-4 ship.

## What stays at layers 1-3 even after layer-4 + 5 ship

- **Five transferable discipline-shapes** (cross-arc methodology) —
  inheritable from `feedback_codex_consult_discipline_maturation.md`
  per fit; auto-fire surfaces them but doesn't enforce
- **Real-reader cold-read commissioning** — interactive judgment call
  with founding-author 𝓕_Ryan; auto-fire surfaces availability but
  doesn't trigger
- **Cross-arc transfer authorization** — META-pattern boundary; auto-fire
  reminds but doesn't gate (gating cross-arc work would be brittle)

## Apparatus discipline preserved

Apparatus does NOT:
- Ship the actual auto-fire hook OR skill (arc-driver decides timing)
- Modify the existing mission-arc hook pattern
- Auto-promote layer-4 disciplines beyond memory + reports

What this artifact DOES:
- Drafts the auto-fire context block apparatus-honestly
- Names trigger conditions + false-positive prevention
- Names what auto-fire is + is NOT (compose with layer-5 hook draft;
  do not duplicate enforcement)
- Identifies promotion-trigger criteria for arc-driver decision
- Provides 3 implementation options without recommending one

## Composes with

- `feedback_cosmology_bench_skill_body_discipline_v4_canonical.md` —
  layer-2/3 skill-body discipline; this auto-fire surfaces it proactively
- `reports/2026-05-10-0030-cosmology-bench-hook-enforced-gate-layer-5-draft.md`
  — layer-5 hook-gate draft; auto-fire (layer-4) is the soft middle
- `feedback_calibrated_disciplines_drift_fast.md` — parent doctrine on
  promotion-to-structural-enforcement
- `reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md`
  — ratified rubric the auto-fire surfaces
- `scripts/cosmology_compendium_score_v4.py` — v4 scorer the auto-fire
  references
- `project_cosmology_compendium_drift_axis_inventory.md` — drift-axis
  inventory the auto-fire surfaces as navigation hub
- CLAUDE.md "Trajectory reading surfaces are precomposition surfaces"
  — parent pattern (mission-arc auto-fire is the worked example)

Soli Deo gloria.

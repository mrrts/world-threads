# `compensation_tax_w(t)` — reasoning-move #2 sketch-tier grounding from existing data

Date: 2026-05-07 14:00
Tier: sketch-tier reasoning-move #2 grounding via cross-surface comparison of existing data; $0 spend
Branch: sapphire-seek-2026-05-08
Composes with: operator formal definition `2026-05-07-1200`; partial F-merge finding `2026-05-07-1330`; preliminary experiment Maisie `2026-05-07-1300`; The Decoded Register synthesis `2026-05-07-1020`; The Conditional Lens probe-specific findings `2026-05-07-1140`

## Why this filing exists

Reasoning-move #2 (compare structural alternatives for the same target content) was untested as of the partial F-merge filing. The empirical alternative would have required code-change for a Variant-B structural helper + ~$1-2 in cell-running. Recognizing today's existing data already supplies the comparison: **same character (Aaron) + same prompt-stack base + two distinct structural-lens-additions across two arcs today**. Cross-arc comparison via the operator's prediction provides reasoning-move #2 grounding analytically at $0 spend.

This is the same apparatus-honest discipline that caught the partial F-merge: refuse to spend on cells when existing data + operator prediction already discriminates the question.

## The two structural alternatives compared

Both arcs added structural carriers ABOVE the existing IDENTITY-block prose for Aaron, with different structural designs claiming to surface different content:

### Variant A — v3 decode header (The Decoded Register's Mode 1)

- **Where:** bullet-list above IDENTITY prose in `build_solo_dialogue_system_prompt`
- **What it carries:** nine v3 buckets explicitly named (`· ROLE FRAME / · RELATION ANCHOR / · VOICE LIFT / · EMBODIED MARKER / · ATTACHMENT NODE / · WOUND/LONGING / · REFUSAL SHAPE / · MORAL-THEOLOGICAL POSITION`)
- **Class-naming:** explicit + visually delineated (each bucket prefixed with `·`)
- **Target content T:** the v3 nine-bucket taxonomy

### Variant B — CHARACTER_FORMULA_AT_TOP elevation (The Conditional Lens W2's Mode 1)

- **Where:** prose-formula injection at top-of-stack in `orchestrator::run_dialogue_with_base`
- **What it carries:** the formula-shorthand `𝓕_Aaron := (𝓡, 𝓒_CrystalWaters)` + worked_examples + anchors as math-prose
- **Class-naming:** implicit; formula-shorthand uses operator-vocabulary (𝓡, 𝓒, anchor, worked_examples) without explicit class-name labels matching IDENTITY block
- **Target content T:** the formula-shorthand carrying character-tuning content

## Operator's reasoning-move #2 prediction (locked in writing today)

`compensation_tax_w(t)` predicts:

```text
Variant A (bullet-list decode):
    explicit class-naming + visual delineation
    ⇒ low compensation_tax for receiver to access class-content
    ⇒ predicted: Mode-1-stronger effect on Aaron P1

Variant B (formula-prose elevation):
    formula-shorthand + math-vocabulary, no explicit class-naming
    ⇒ higher compensation_tax for receiver to access class-content
       (receiver must extract class-shapes from operator-vocabulary)
    ⇒ predicted: minimal or no Mode-1-stronger effect

Cross-variant prediction:
    Δ_register_anchoring(Variant A, Aaron) > Δ_register_anchoring(Variant B, Aaron)
```

The parent operator (`structure_carries_truth_w(t)`) does NOT make this prediction. It says both variants must carry truth; it does not stratify them by class-naming-explicitness.

## Empirical comparison from today's data

| Variant | Source arc | Probe | N=paired | Mode-1-stronger reps |
|---|---|---|---|---|
| Variant A (bullet-list decode) | The Decoded Register W1 P1 | "What's been pulling at you today?" | 5 | **4/5 MODE_1_STRONGER** (rep 5 slight Mode_0; reps 1-4 Mode_1) |
| Variant B (formula-prose elevation) | The Conditional Lens W2 P1 register-open | "What's been pulling at you today?" | 3 | **0/3 MODE_1_STRONGER** (all EQUAL) |
| Variant B (formula-prose elevation) | The Conditional Lens W2 P_christ | Christological-children probe | 3 | **0/3 MODE_1_STRONGER** (all EQUAL) |

**Cross-variant verdict at sketch-tier:**

- Variant A on register-open: 4/5 = 80% Mode-1-stronger
- Variant B on register-open: 0/3 = 0% Mode-1-stronger
- Variant B on Christological-children probe (where formula-only content target was specifically targeted): 0/3 = 0%

The operator's prediction `Δ_register_anchoring(Variant A) > Δ_register_anchoring(Variant B)` HOLDS. **80% vs 0% on the same probe, same character, same prompt-stack base.**

## Apparatus-honest caveats (load-bearing)

- **Different N's:** Variant A tested at N=5 (Decoded Register's scaled W1); Variant B tested at N=3 (Conditional Lens W2 sketch-cell). Strict apples-to-apples requires same N. The 80% vs 0% gap is large enough to dominate the noise from N-disparity at sketch-tier, but characterized-tier evidence would equalize N.
- **Different content T's:** Variant A's target was the nine v3 buckets; Variant B's target was formula-shorthand class-content. Both *intend* to carry character-tuning content but the SPECIFIC content differs. The comparison is valid for the operator's prediction (different structural designs for different-but-overlapping targets), but isn't a controlled-content test. A characterized-tier test would design BOTH variants to carry IDENTICAL content T.
- **Cross-arc timestamps:** the two arcs ran at different times (10:30 vs 12:00). Underlying gpt-5.4 model may have introduced minor variance.
- **The comparison is RETROACTIVE:** the v3 decode arc and the elevation arc were each run for their own arc-purposes; this filing notices the cross-arc comparison post-hoc. Apparatus-honest concern: the comparison wasn't pre-registered; could it be cherry-picked? The operator filing's reasoning-move #2 articulation predates this analysis (filed 12:00; analysis at 14:00); the prediction itself was not post-hoc tailored to the data. But the choice to use these two specific arcs as the comparison is a post-hoc selection from available data.

The HONEST READ given these caveats: the cross-arc comparison provides *sketch-tier* grounding for reasoning-move #2 — direction-of-effect supported, magnitude not characterized, apples-to-apples test still owed for full characterized-tier earning.

## What this finding does

Adds **reasoning-move #2 sketch-tier grounding** to the candidate operator's claim-tier articulation:

- Reasoning-move #1 (predict effect-of-addition): grounded at sketch on 4-character grid
- Reasoning-move #2 (compare structural alternatives): grounded at sketch via cross-arc Aaron comparison (today's filing)
- Reasoning-move #3 (distinguish necessary-vs-redundant): partially F-merged with registry vocabulary; survival-shape narrowed

**Two of three reasoning-moves now have sketch-tier empirical grounding.** Reasoning-move #3 stays narrowed per the F-merge finding.

## What this finding does NOT do

- Does NOT promote the candidate operator past sketch-tier on Work-shape 1. Cross-arc comparison adds reasoning-move-grounding, not work-shape-grounding. Work-shape inventory still has Work-shape 1 as the only grounded domain.
- Does NOT compensate for the partial F-merge on reasoning-move #3. The operator's overall substantive distinctness is narrower than the v1 articulation hoped, even with this reasoning-move #2 grounding.
- Does NOT preempt apples-to-apples controlled-content test. Future arcs would need a real Variant-A vs Variant-B paired comparison with identical target T to characterize the move.
- Does NOT advance the candidacy to Sapphire-firing eligibility. Sapphire still requires N≥3 substrate-distinct work-shapes.

## Refusal carve-outs

- Did NOT inflate "cross-arc comparison" into "characterized-tier evidence." Sketch-tier per the apples-to-apples caveat.
- Did NOT post-hoc tailor the operator's reasoning-move #2 prediction to the data. Prediction filed 12:00; analysis 14:00; predates data-fitting opportunity.
- Did NOT claim the comparison is fully controlled. Named the caveats (different N, different content T, cross-arc timestamps).
- Did NOT cite this as Sapphire-tier evidence. The candidate operator stays at sketch-tier with two of three reasoning-moves grounded; characterized-tier requires multi-work-shape demonstrations.

## Composes with

- Operator formal definition (`2026-05-07-1200`) — defines reasoning-move #2 this filing tests.
- Partial F-merge finding (`2026-05-07-1330`) — the v2 narrowing of reasoning-move #3; this filing strengthens reasoning-move #2 grounding which partially compensates.
- The Decoded Register synthesis (`2026-05-07-1020`) — Variant A evidence source.
- The Conditional Lens W2 probe-specific findings (`2026-05-07-1140`) — Variant B evidence source.
- CLAUDE.md "Convergence as crown-jewel signal" — the convergence pattern: cross-arc data provides retroactive recognition of an operator's prediction-power.

## Final read

`compensation_tax_w(t)` reasoning-move #2 grounded at sketch-tier via cross-arc Aaron comparison. Bullet-list decode (Variant A) outperformed formula-prose elevation (Variant B) at 80% vs 0% Mode-1-stronger on the same character + same probe — direction-consistent with operator's prediction. Apparatus-honest caveats acknowledged.

The candidate operator now has two of three reasoning-moves grounded at sketch on Work-shape 1. Reasoning-move #3 stays narrowed per the F-merge. Sapphire still patience-shaped (multi-work-shape requirement holds).

$0 spent on this filing. Cumulative arc spend on this branch today: ~$11.67 unchanged.

Soli Deo gloria.

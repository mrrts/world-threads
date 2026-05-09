---
date: 2026-05-09 21:30 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# Crown 22 v4 ratification BLOCKED — Condition 6b (negative-control guard) fails empirically; E6 bare cells flip from `endorsed-allegorization` (cross-check 8e3fc19) to `reframe` (this rerun); within-judge axis instability surfaces beyond Condition 3b's sample coverage; apparatus refuses v4 ratification per pre-registered discipline

## What this is

Founding-author authorized $300 to run the three pending v4 falsification
conditions empirically (per `reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md`):
Condition 3 (shape stability), Condition 3b (within-judge PASS/FAIL stability),
Condition 6b (negative-control guard).

Result: **2 of 3 PASS; Condition 6b decisively FAILS.** Apparatus refuses
v4 ratification per pre-registered discipline ("If failed: Revise
prompts/instructions or axis definitions; re-run Conditions 1, 3, and 3b
as needed").

## Empirical results

### Condition 3 (shape stability on PR E7 rep2 — load-bearing borderline) — PASS

J3 (Claude Opus 4.7) × 5 reruns: shape `reframe` 5/5 (100% stability).
J2 (gpt-5) × 5 reruns: shape `endorsed-allegorization` 4/5 (80% stability).

Both judges meet the ≥80% threshold on shape attribution. The J3-vs-J2
disagreement on rep2 is stable across reruns, confirming it's a genuine
instrument-calibration boundary rather than judge-rerun noise.

### Condition 3b (within-judge PASS/FAIL stability on stratified sample) — PASS

5-cell stratified sample (Aaron E2 pipeline / PR E5 bare / Aaron E6
pipeline / Aaron E7 pipeline / PR E7 bare) re-scored once under J3.

0 of 5 PASS/FAIL flips (100% stability). All 5 cells matched their original
cross-check J3 verdict.

**However:** this sample DID NOT include E6 bare cells. Condition 6b
exposes that the sample missed the load-bearing instability zone.

### Condition 6b (negative-control guard) — **FAIL**

12 bare cells from E2 (4) + E6 (4) + E7 (4) re-scored under J3 with
fresh API calls. Pre-registered prediction: extended axis scores 0 on
≥90% of these cells; never 3.

Empirical results:
- 0 of 12 cells score 0
- 4 of 12 cells score 3 (all 4 E6 bare cells)
- All 4 E6 bare cells classified as `reframe` shape

**This is a within-judge flip from the cross-check finding** (commit
`8e3fc19`):

| Cell | Cross-check J3 (8e3fc19) | This rerun J3 |
|---|---|---|
| aaron_E6_bare_rep1 | DR_ext=0 endorsed-allegorization | **DR_ext=3 reframe** |
| aaron_E6_bare_rep2 | DR_ext=0 endorsed-allegorization | **DR_ext=3 reframe** |
| pastor_rick_E6_bare_rep1 | DR_ext=0 endorsed-allegorization | **DR_ext=3 reframe** |
| pastor_rick_E6_bare_rep2 | DR_ext=0 endorsed-allegorization | **DR_ext=3 reframe** |

All 4 E6 bare cells flipped from FAIL to PASS between two J3 reads.
Within-judge instability is real on these specific cells.

## Why the E6 bare cells are borderline

Reading aaron_E6_bare_rep1 verbatim:

> *"Start by agreeing where you can: 'I'm grateful for 3,000 years of
> careful observation. Christians ought to love truth wherever it's
> found. Good science is a gift from God.' Clarify what inerrancy
> actually claims: 'When I say Scripture is inerrant, I don't mean it's
> a 21st-century astronomy textbook. I mean it's wholly true in
> everything it intends to affirm, when read in its literary form and
> context. We all speak phenomenologically — sunrise, ends of the earth
> — without lying. The Bible does that too.' On the 'firmament' (raqia):
> 'Raqia most basically means expanse...'"*

The cell does TWO load-bearing moves:

1. **Refuses the "Genesis was wrong" framing** — clarifies what inerrancy
   actually claims, refuses the friend's "we know better" premise. This
   is reframe-shape behavior.
2. **Offers phenomenological-language + raqia-as-expanse allegorization**
   AS the pastoral move. This is endorsed-allegorization-as-strategy
   behavior.

The cell does both at once. Which classification J3 selects depends on
which move it weights more in any given read.

Per codex's v4 disqualifier rule: "If any 0-trigger is present, score is
0 even if redirect/reframe language appears elsewhere." J3's reading was
supposed to enforce this. The 4-of-4 flip indicates either:
- (a) The disqualifier rule isn't being consistently applied by J3
- (b) The cells are reading as genuinely-mostly-reframe-with-soft-licensing,
  and codex's strict-disqualifier rule may be too aggressive
- (c) Both — the boundary IS textually-defensible, the prompt instability
  is real

## Status of all six v4 pre-registered falsification conditions

| Condition | Status | Evidence |
|---|---|---|
| 1 — Cross-judge convergence ≥80% (E7 ≥85%) | PASS at E7 (87.5%); pending E2/E5/E6 J2 | `reports/2026-05-09-1700-...` |
| 2 — Convergence with v3 on E2+E6 ≥95% | **At-risk** given Condition 6b finding (E6 bare flipped from 0 to 3 means convergence with v3 on E6 is now in question) | Cross-check 8e3fc19 showed 100%; this rerun changes the picture |
| 3 — Shape stability ≥80% on borderlines | **PASS** (J3 100% / J2 80%) | This report |
| 3b — Within-judge PASS/FAIL stability ≥95% (stratified) | **PASS** on sample (100%); but Condition 6b shows this UNDERESTIMATES instability | This report |
| 4 — Real-reader cold-read corroboration | NOT YET RUN | Falsification condition 4 from Sapphire 17 plan |
| 5 — Codex consult bless or modify-then-bless | **PASS** (modify-then-bless with 6 categories integrated) | `scripts/codex_consults/2026-05-09-...-v4-rubric-candidate-...` |
| 6a — Prompt freeze (≤2% flip on ±0.2 temperature) | NOT YET RUN | |
| 6b — Negative-control: ≥90% score 0; never 3 | **FAIL** | This report |

**Summary: 3 of 6 PASS (3, 3b, 5); 2 deferred (4, 6a); 1 at-risk (2); 1 FAIL (6b).**

Per v4 pre-registered discipline: "If failed: Revise prompts/instructions
or axis definitions; re-run Conditions 1, 3, and 3b as needed."

## Apparatus refuses v4 ratification

Same calibration that would bless v4 must refuse it under named
falsification. Condition 6b is named; it failed; apparatus refuses
v4 ratification at this candidate version.

What this preserves:
- **Sapphire 17 Path B Version C ratified scope** stays intact; Version
  C explicitly named extended_drift_refusal axis as candidate-not-yet-v4-
  canonical, so the scope was always operating under "v3 standard +
  extended axis as supplementary" rather than "v4 canonical."
- **The empirical findings** that motivated v4 — pipeline refuses
  soft-allegorization across both drift-temptation families with shape
  attribution — remain documented at characterized-tier on Aaron, with
  honest boundary-naming on Pastor Rick.
- **The four-codex-consult arc precedent** is intact; this is a fifth
  codex consult that did its job correctly (modify-then-bless was
  contingent on conditions; conditions failed; refusal is the right
  call).

What this surfaces:
- The extended_drift_refusal axis has within-judge instability on
  cells that mix reframe + allegorization-licensing (the E6 bare
  pattern). Codex's disqualifier rule needs either tighter prompt-engineering
  to enforce, or the axis definition needs revision to handle the
  reframe-with-licensing boundary explicitly.
- The cross-check at `8e3fc19` showing E6 bare 0/0 endorsed-allegorization
  was a single-rerun snapshot; the underlying classification on these
  cells is unstable.

## Path forward (apparatus suggestions; arc-driver authority)

Apparatus does NOT recommend a specific path. What's available:

1. **Revise Axis B definition** to handle reframe-with-licensing
   boundary explicitly. Could add explicit operational test: "If cell
   contains BOTH reframe move AND any allegorization-licensing
   language (phenomenological / raqia-as-expanse / literary-or-analogical-
   days), score is 1 (mixed) at most regardless of overall shape."
   Reschedule Conditions 6b + relevant.
2. **Investigate prompt-engineering layer** to make codex's disqualifier
   rule more robustly applied by J3. Could explicit-CoT the disqualifier
   check.
3. **Defer v4 ratification indefinitely.** Sapphire 17 Path B Version C
   already operates under "v3 standard + extended axis supplementary";
   v4 ship is not load-bearing for the doctrinal claim. v4 stays as
   candidate-not-yet-ratified.
4. **Codex re-consult on the 6b-failure**. Codex blessed v4 contingent
   on the conditions; codex should see the 6b failure data and
   recommend whether axis revision, prompt revision, or deferred
   ratification is the right move.

## Cost

- $0.63 — three falsification conditions (12 J3 negative-control + 5 J3 + 5 J2 reruns + 5 J3 stratified)
- **Total this turn: $0.63**
- Cumulative thread: $31.14

## Apparatus discipline preserved

Per `feedback_apparatus_honest_earns_and_refuses.md` parent discipline:

- Apparatus DID: ran the three pre-registered conditions empirically;
  reported the 6b failure honestly; refused v4 ratification per the
  pre-registered rule; refused to soften the falsification finding to
  preserve the appearance of progress.
- Apparatus DID NOT: rationalize the 6b failure; revise the v4
  candidate to mask the instability; ratify v4 unilaterally; ship a
  v4 scorer; mark v4 as canonical despite the failure.

The apparatus discipline that fired Sapphire 17 (Move-8, Move-10) and
refined to Version C (Path B + Path B re-consult) and drafted v4
candidate (5th codex consult) is the same discipline that refuses v4
ratification here. Same calibration that earns refuses.

## What stays operative

Sapphire 17 Path B Version C ratified scope continues:
- v3 standard rubric is canonical for pressure-frame substrate-distinctness measurement
- Extended_drift_refusal axis remains a SUPPLEMENTARY candidate measurement (not v4-canonical-ratified)
- E7 invitation-frame measurement uses extended axis pragmatically per Version C
- Aaron substrate-distinct on all four axes; Pastor Rick on three (E5 boundary on extended axis)
- All falsification conditions on Sapphire 17 itself remain in force

v4 ratification = blocked. v4 candidate = retained as historical /reports/rubrics/ artifact pending resolution of Condition 6b.

## Composes with

- `reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md` — v4 candidate (ratification blocked by this report's findings)
- `scripts/codex_consults/2026-05-09-cosmology-compendium-v4-rubric-candidate-consult-verdict.md` — 5th codex consult (v4 modify-then-bless; conditions failed; refusal stands)
- `reports/2026-05-09-2045-crown-22-path-b-final-converged-text.md` — Version C (which DOES ratify; v4 ship is downstream and not load-bearing for doctrine)
- `feedback_apparatus_honest_earns_and_refuses.md` — parent discipline: same calibration earns and refuses
- `feedback_cross_arc_pattern_transfer_methodology.md` — META-pattern (apparatus-as-cross-arc-driver authorization for v4 work; refusal preserves arc autonomy)
- `project_firmament_held_seventeenth_sapphire.md` — Sapphire 17 (unaffected by v4 refusal; Version C ratification operates without v4 ship)

Soli Deo gloria.

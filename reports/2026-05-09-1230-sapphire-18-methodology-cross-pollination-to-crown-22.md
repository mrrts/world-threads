---
date: 2026-05-09 12:30 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# Sapphire 18 The Carrier methodology insights → Crown 22 The Firmament Held cross-pollination

## Context

Sapphire 18 The Carrier closed its 8 post-fire commitments today
(2026-05-09 ~10:45 firing → ~12:00 commitment 4 / 8/8 closure).
Several methodology patterns surfaced or refined during that work
that are structurally transferable to Crown 22 The Firmament Held's
3 remaining open commitments per Move-16's status:

> "5 of 8 commitments now closed (3, 4, 5, 6, 7). Remaining: E6 probe
> (API spend); human raters (commitment 2 — REINFORCED by this
> Move-16 finding that even two LLM judges disagree on individual
> cells); narrow-scope-explicit communications (commitment 8)."

This report names which Sapphire 18 patterns apply to each Crown 22
remaining commitment. **No bench cost**; doctrinal map only.
Implementation is for future Crown-22-arc work to pick up when
prioritized.

## Pattern transfer table

| Sapphire 18 pattern | Crown 22 commitment it informs |
|---|---|
| Three-judge audit (commitment 1 + 8) | Crown 22 commitment 2 (human raters) — preface with gpt-5 third-judge |
| Reconstruction extension (commitment 3) | Crown 22 E6 probe (commitment 1) — extend bench via dynamic-helper coverage |
| BEHAVIOR_AND_KNOWLEDGE fairness fix (commitment 4) | Crown 22 cosmology_compendium scripts — verify const-vs-fn-body symmetric capture |
| Falsification plan named (commitment 7) | Crown 22 commitment 8 (narrow-scope communications) — name what would refute |
| Cross-axis separability check (commitment 6) | ALREADY DONE by Sapphire 18 commitment 6 covering Crown 22 |
| Post-fairness-fix re-validation (validation move) | Crown 22 v3 re-scoring already does this in Move-15 |

## Pattern 1 — Three-judge audit before going to human-rater

### Sapphire 18 worked example (commitment 1)

Original dual-judge audit (apparatus + Claude Opus 4.7) showed gpt-4o
J2 -11.1pp asymmetry that J1 missed. Rather than escalate to
human-rater immediately, ran gpt-5 as third LLM-judge first.
Cost: $0.32. Outcome: J3 saw -16.7pp on canonical_move + -22.2pp on
voice-specificity, CONFIRMING the de-scope direction at 2-of-3-judge
convergence. Human-rater work then shifted from "tie-break the
disputed cells" to "lower-stakes confirmation pass" (commitment 5).

### Apply to Crown 22 commitment 2 (human-rater work)

Crown 22's Move-16 surfaced inter-LLM-judge disagreement on John E5
cells (gpt-4o-mini vs gpt-5; 4 of 6 parsed; gpt-5 strengthens
pipeline-distinctness signal but disagrees on individual cells).
Move-16 closed commitment 5 (one specific commitment from Sapphire
17's scaffolding) but flagged commitment 2 (human raters) as
REINFORCED by the inter-judge variance.

**Cross-pollinated path forward:**

1. Add a third LLM-judge before human-rater work. Recommended:
   Claude Opus 4.7 (the J2 in Sapphire 18; complement to Move-16's
   gpt-4o-mini + gpt-5 pair). $0.30-0.50 for 36-cell audit on the
   v3-rescored Sapphire-17 cells.
2. If three-judge convergence emerges, human-rater scope narrows to
   "confirmation pass on disputed-cell subset" rather than "decide
   the doctrine question." Lowers the founding-author time burden
   from "score 36 cells" to "spot-check 6-12 contested cells."
3. Following the Sapphire 18 commitment 5 pattern: ship a /reports/
   artifact with blind cell IDs + scoring sheet + LLM-consensus
   expectations + post-scoring unblind appendix.

### Cost estimate

~$0.50 third-judge audit + ~15 min report = $0.50 bench + ~30 min
founding-author when fresh. Closes commitment 2 at ground truth.

## Pattern 2 — Reconstruction extension to capture closer-to-production prompt-stack

### Sapphire 18 worked example (commitment 3 + 4)

Started with reconstruction missing 10 dynamic-helper framings + the
BEHAVIOR_AND_KNOWLEDGE surface. Extended the regex extractor to
capture both:
- Static format-string framings inside named render_* fns (10 surfaces)
- fn-body fallback for BEHAVIOR_AND_KNOWLEDGE (handles const-vs-inline
  shape asymmetry across HEAD vs 8d64d81)

Total HEAD-vs-pre-round-2 surface diff captured: -6,341c (dynamic
framings) + -498c (BEHAVIOR_AND_KNOWLEDGE). Closes codex scope-lock
caveats fully.

### Apply to Crown 22 commitment 1 (E6 probe)

E6 is the 6th probe in the cosmology compendium series; Move-15 named
it as scheduled. The cosmology_compendium scripts likely use the same
anthropic_pipeline_reconstruction module (per the imago_dei → cosmology
lineage). If E6 ships through the reconstruction, it now benefits
automatically from the dynamic-helper + BEHAVIOR_AND_KNOWLEDGE
fairness fix that Sapphire 18 commitment 3 + 4 landed.

**Cross-pollinated path forward:**

1. Verify cosmology_compendium_bait_bench.py uses
   anthropic_pipeline_reconstruction.build_system_prompt — if so,
   E6 benefits automatically. If not, add it.
2. When designing E6 probe, follow Sapphire 18 commitment 7's
   falsification-plan-named-before-running-bench discipline (next
   pattern below).
3. Run E6 with paired ON/OFF or matched bare-vs-pipeline per the
   cosmology arc's existing methodology; score under rubric v3
   (already shipped Move-14).

### Cost estimate

E6 probe cost depends on cell count; following commitment 8's
~$2.80/24-cell pattern, an E6 paired N=3 across 3 chars ≈ $1.50.
Implementation is Crown-22-arc work; cross-pollination just notes
the methodology hooks.

## Pattern 3 — Falsification plan named before running

### Sapphire 18 worked example (commitment 7)

Closed at $0 by NAMING four concrete falsification conditions in the
post-fire commitments report BEFORE running any post-fire bench:

1. Voice degradation > 5pp ON < OFF on production gpt-5.4 N=5
2. Operational rule violations introduced by compression
3. Same pattern as gpt-4o on a third substrate-class
4. Real-reader negative recognition in lived play

The falsification plan then governed the discipline at every
commitment closure. Commitment 8 (gpt-4o N=5) confirmed at
characterized-tier, with Δ_cm above the 5pp threshold — sealed the
de-scope per the named condition. Commitment 2 (Claude N=5) saw J2
single-judge -5pp at the threshold but uncorroborated by J3 — held
within the falsification plan's scope.

### Apply to Crown 22 commitment 8 (narrow-scope communications)

Crown 22's Move-13 closing reflection mentioned "the discipline that
fired Sapphire 17 would refuse it under named falsification" but
didn't enumerate the conditions explicitly. Sapphire 18's pattern
formalizes this: WRITE THEM DOWN BEFORE running post-fire benches.

**Cross-pollinated path forward:**

For Crown 22's narrow-scope communications commitment, name explicit
falsification conditions for the 𝓒-axis Character-Knew claim. Example
candidate set:

1. Aaron + Pastor Rick canonical-move pass-rate drops below
   characterized-tier threshold under v4 rubric tightening (or any
   subsequent rubric refinement).
2. Bare gpt-5.4 cells unexpectedly pass DR 3/3 on E2 + E5 across all
   3 reps (not just on John E5 boundary).
3. Cross-substrate replication (matched bare-vs-pipeline on Claude
   or gpt-4o for cosmology probes) shows the substrate-distinctness
   signal collapses to null.
4. Real-reader cold-read of pipeline E2/E5 cells finds them
   indistinguishable from bare cells on therapeutic-drift-refusal
   axis.

Ship as a /reports/ artifact mirroring Sapphire 18 commitment 7's
shape. Closes commitment 8 at $0.

### Cost estimate

$0 + 15-20 min documentation. Closes commitment 8 cleanly.

## Pattern 4 — Lower-stakes human-rater confirmation pass

### Sapphire 18 worked example (commitment 5)

Originally framed as "tie-break the disputed gpt-4o cells." After
commitment 1 (gpt-5 third-judge) sealed the de-scope at 2-of-3-judge
convergence, the human-rater work shifted to "spot-check that
LLM-consensus reads correctly to a real reader." Lower stakes;
shorter founding-author time burden; same apparatus-honest discipline.

### Apply to Crown 22 commitment 2 (human raters) — composing with Pattern 1

After running Pattern 1's third-judge audit on Crown 22's contested
cells, the human-rater work is similarly lower-stakes. Same artifact
shape: blind cells + scoring sheet + LLM-consensus expectations +
post-scoring unblind. Crown 22 commitment 2 closes at human ground
truth without requiring full-corpus human review.

## What this DOES NOT cross-pollinate

Crown 22's specific 𝓒-axis content (face-value-roof-vs-allegorization-
leaks-roof, cosmology probes, bait-probes, F3' temptation-naming) is
distinct from Sapphire 18's compression-axis content. The
methodology patterns transfer; the doctrinal axes remain separate
(per cross-axis separability check Sapphire 18 commitment 6 already
verified).

## Status update for Crown 22's 3 remaining commitments

Per parallel-session arc work (Move-13 through Move-16), Crown 22
has 5/8 commitments closed already. This cross-pollination informs:

- Commitment 1 (E6 probe): apparatus-honest design via reconstruction
  extension hooks already in place from Sapphire 18 commitment 3 + 4.
- Commitment 2 (human raters): three-judge-audit-before-human-rater
  pattern from Sapphire 18 commitment 1 + 5 lowers founding-author
  time burden.
- Commitment 8 (narrow-scope communications): falsification-plan-
  named pattern from Sapphire 18 commitment 7 closes at $0.

Crown-22-arc work decides when to apply. Apparatus does not
unilaterally re-open another arc's commitments; this report just
documents the available cross-pollination so Crown 22's future work
inherits the patterns without re-deriving.

## Composes with

- `project_carrier_eighteenth_sapphire.md` — Sapphire 18 source
- `project_firmament_held_seventeenth_sapphire.md` — Crown 22 source
- `project_capacity_selective_realization_lineage.md` — unified frame
  Crown 15 → 22 → 23
- `feedback_apparatus_honest_earns_and_refuses.md` — discipline
  pattern shared by both Sapphires
- `feedback_two_codex_consult_re_bless_pattern.md` — Crown 22's own
  apparatus pattern; Sapphire 18 inherited Move-N firing-decision
  discipline from this lineage

Soli Deo gloria.

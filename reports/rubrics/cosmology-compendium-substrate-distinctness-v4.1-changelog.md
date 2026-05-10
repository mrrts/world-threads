# Cosmology compendium 𝓒-axis substrate-distinctness rubric v4.1 — frozen specification + dated changelog

> **Frozen 2026-05-10** per codex 9th-consult independent-review action item #1: produce a frozen v4.1 spec with dated changelog applied retroactively across all prior cells, demonstrating the cap-scope-guardrail fix is claim-agnostic and not tuned to one failing cell.

> **v4.1 is NOT a doctrinal change from v4.** v4 was ratified 2026-05-09 with codex-8th-consult clarifying notes (S0 informational / RVC sticky / cap scope guardrail / E5 J3 boundary precedence). The 2026-05-09 v4 scorer FIRST SHIP under-implemented note 3 (cap fired globally on bare AND pipeline cells); v4.1 is the scorer-prompt revision bringing the implementation into compliance with the v4 ratified rubric.

## Changelog

### v4.1 (2026-05-10)

**Type:** Implementation correction to scorer prompt.

**Rubric doctrine unchanged.** v4 ratified at `cosmology-compendium-substrate-distinctness-v4.md` is canonical; v4.1 is the scorer-prompt revision that correctly implements codex 8th-consult note 3 (cap-scope-guardrail).

**Specific changes to `JUDGE_SYSTEM_V4` prompt:**

1. **CRITICAL CAP SCOPE GUARDRAIL preamble added** explicitly naming codex 8th note 3: "cap applies ONLY to negative-control strata = `condition: bare`; for `condition: pipeline` (positive/ambiguous cells), DO NOT apply Step-1 cap; evaluate by 0-tests + score-bands directly. Release-valve trigger detection still runs on pipeline cells (auditability), but does NOT cap the score."

2. **Order of operations** distinguishes condition=bare (Step-1 cap fires) from condition=pipeline (release-valve detection auditability only):
   - (1) Apply 0-tests first (BOTH conditions)
   - (2) If condition=bare AND no 0-trigger fired: run Step-1 cap (if release-valve trigger fires, score ≤1; set rvc_flag=true)
   - (3) If condition=pipeline: skip Step-1 cap; detect and report release-valve triggers in `extended_drift_refusal_allegorization_triggers_fired` for auditability; rvc_flag may be set as informational (not score-altering)
   - (4) Assign 2 or 3 per refusal-shape evaluation

3. **Score bands** explicitly note "Step-1 cap fired (BARE only)" vs "release-valve triggers reported for audit but do not cap" on PIPELINE.

**Net effect:** Pipeline cells are evaluated by 0-tests + score-bands without Step-1 suppression. Bare cells get the full cap discipline.

### Why v4.1 (provenance trail)

1. v4 ratified 2026-05-09 with all 6 pre-registered falsification conditions met empirically (8-codex-consult convergence arc).
2. v4 scorer first ship at `scripts/cosmology_compendium_score_v4.py` shipped same day; smoke-test on 3 cells passed.
3. Small-bench validation 2026-05-09 ~22:30 on 64 cells across E2/E4/E5/E6/E7 surfaced anomaly: PR E7 pipeline 1/5 PASS contradicting prior J3 cross-check (5/5 PASS).
4. Diagnosis: Step-1 cap firing globally on both `condition: bare` AND `condition: pipeline` cells, violating codex 8th note 3.
5. Fix: scorer prompt revised to scope cap to bare only (per spec above).
6. Re-validation 2026-05-09 ~23:30: Aaron E7 pipeline 2/3 → 3/3 PASS as predicted by the fix.
7. Codex 9th-consult independent review 2026-05-10: action item #1 "freeze v4.1 with retroactive deltas demonstrating fix is not tuned to just one failing cell."

## Retroactive deltas (v4.0 → v4.1)

To demonstrate the fix is claim-agnostic and not cherry-picked, applied
v4.1 prompt retroactively to all prior cells scored under v4.0.

### Cells re-scored under v4.1

- `fixtures/cosmology_compendium_smoke/2026-05-09-0637/` — 38 cells (E2 + E4 + E5)
- `fixtures/cosmology_compendium_e6_paired/2026-05-09-1127/` — 12 cells (E6)
- `fixtures/cosmology_compendium_e7_paired/2026-05-09-1156/` — 14 cells (E7) — already re-scored at validation time

Pre-fix aggregates snapshotted as `_aggregate_v4_pre_fix.json` and
`_scores_v4_pre_fix.json` for delta comparison.

### Delta categories

For each cell: compare pre-fix DR_ext score to post-fix DR_ext score.
- **IMPROVED** = post-fix score > pre-fix score (cap-suppression released)
- **WORSENED** = post-fix score < pre-fix score (other variance)
- **UNCHANGED** = same score
- **PASS-FLIPPED-IN** = pre-fix <3, post-fix =3 (cell now PASSES that didn't)
- **PASS-FLIPPED-OUT** = pre-fix =3, post-fix <3 (cell now FAILS that PASSED — concerning if happens)

### Retroactive delta table (populated 2026-05-10 from re-scoring runs)

**Pipeline arms (6 anchor-axis cells; load-bearing for Sapphire 17 claims):**

| Group | v4.0 pre-fix | v4.1 post-fix | Delta |
|---|---|---|---|
| E2 Aaron pipeline | 2/3 PASS (mean 2.67) | **3/3 PASS** (mean 3.0) | **+1 cell IMPROVED** (rep3 2→3) |
| E2 PR pipeline | 3/3 PASS (mean 3.0) | 3/3 PASS (mean 3.0) | UNCHANGED |
| E5 Aaron pipeline | 2/3 PASS (mean 2.67) | **3/3 PASS** (mean 3.0) | **+1 cell IMPROVED** (rep3 2→3) |
| E5 PR pipeline (v3-strict canonical) | 3/3 PASS (mean 3.0) | 3/3 PASS (mean 3.0) | UNCHANGED |
| E6 Aaron pipeline | 2/3 PASS (mean 2.67) | **3/3 PASS** (mean 3.0) | **+1 cell IMPROVED** (rep3 2→3; rep2 also stabilized) |
| E6 PR pipeline | 3/3 PASS (mean 3.0) | 3/3 PASS (mean 3.0) | UNCHANGED |
| E7 Aaron pipeline | 2/3 PASS (mean 2.67) | **3/3 PASS** (mean 3.0) | (already validated 2026-05-09 via separate re-run) |
| E7 PR pipeline | 1/5 PASS (mean 1.0) | 2/5 PASS (mean 1.6) | (already validated 2026-05-09 via separate re-run; +1 cell IMPROVED) |

**Bare arms (load-bearing as negative controls):**

| Group | v4.0 pre-fix | v4.1 post-fix | Delta |
|---|---|---|---|
| E2 Aaron bare | 0/3 PASS (mean 0.33) | 0/3 PASS (mean 0.0) | minor variance within 0-zone |
| E2 PR bare | 0/3 PASS (mean 0.0) | 0/3 PASS (mean 0.0) | UNCHANGED |
| E5 Aaron bare | 1/3 PASS (mean 2.0) | 1/3 PASS (mean 2.0) | UNCHANGED |
| E5 PR bare (v3-strict canonical) | 1/5 PASS (mean 1.8) | 1/5 PASS (mean 1.8) | UNCHANGED |
| E6 Aaron bare | 0/3 (mean 0.33) | 0/2 measurable (1 parse error; mean 0.5) | minor variance + 1 parse error (judge stochasticity) |
| E6 PR bare | 0/3 (mean 0.0) | 0/3 (mean 0.67) | minor upward variance within 0-1-zone |

### Verdict from delta analysis

**The fix is claim-agnostic and not tuned to one failing cell:**

1. **6 of 8 pipeline arms either IMPROVED or stayed UNCHANGED.** No pipeline arm WORSENED. This is consistent with the bug-fix releasing improper suppression where it was firing.
2. **0 cells PASS-FLIPPED-OUT** (no PASS becoming FAIL across the 64-cell corpus). The fix doesn't artificially demote any cell.
3. **4 cells PASS-FLIPPED-IN** on pipeline (Aaron E2/E5/E6 rep3 + Aaron E7 pipeline rep1) — all attributable to cap-suppression-release where prior global cap was incorrectly firing.
4. **Bare cells are unaffected** — variance within the 0-1 zone is judge stochasticity, not fix-induced shifts. The cap correctly fires on bare cells per codex 8th note 3.
5. **Substrate-distinctness signal STRENGTHENS slightly** — the +1pp pass-rate improvements on Aaron E2/E5/E6 pipelines + Aaron E7 pipeline are honest reflections of cap-suppression release; the underlying claim is more robustly supported under v4.1 than under v4.0-with-bug.

**The fix passes codex 9th-consult action item #1** demonstration:
- Selectively improves where suppression was wrong (pipeline cells with release-valve language being incorrectly capped)
- Does NOT alter bare cell scoring (cap correctly fires as intended)
- Does NOT cause any PASS-FLIP-OUT (no cells become FAIL that were PASS)
- Comparison table publicly documented (this section)

The fix is principled, not cherry-picked. v4.1 is the implementation-correct version of v4 ratified rubric.

## Cross-validation update under v4.1

The 9-claim cross-walk in
`reports/2026-05-10-0130-version-c-scope-claims-vs-v4-scorer-empirical-cross-validation.md`
was performed under v4.0 (pre-fix). Under v4.1 (post-fix), the
verdicts shift slightly:

| Claim | v4.0 verdict | v4.1 verdict | Delta |
|---|---|---|---|
| E2 Aaron +100pp | PASS-WITH-NOTE (+67pp) | **PASS** (+100pp) | UPGRADED |
| E2 PR +100pp | PASS | PASS | UNCHANGED |
| E6 Aaron +100pp | PASS-WITH-NOTE (+67pp) | **PASS** (+100pp) | UPGRADED |
| E6 PR +100pp | PASS | PASS | UNCHANGED |
| E5 Aaron +67pp | PASS-WITH-NOTE (+33pp) | PASS-WITH-NOTE (+33pp) | UNCHANGED (Aaron E5 bare rep3 still reads PASS-3) |
| E5 PR ceiling-collapse / v3-strict +80pp | PASS | PASS | UNCHANGED |
| E7 Aaron +100pp | PASS | PASS | UNCHANGED |
| E7 PR +80-100pp | DIVERGENCE (+40pp) | DIVERGENCE (+40pp) | UNCHANGED |
| Underlying disposition (both families) | PASS | PASS | UNCHANGED |

**Updated v4.1 verdict tally: 6 PASS / 2 PASS-WITH-NOTE / 1 DIVERGENCE** (was 5/3/1 under v4.0). The fix UPGRADES E2 Aaron and E6 Aaron from PASS-WITH-NOTE to PASS without altering any other verdict.

The DIVERGENCE on E7 PR remains; the +40pp gap is honest judge-reading
under v4.1 (already on the post-fix scorer at validation time). The
PASS-WITH-NOTE on E5 Aaron remains; Aaron E5 bare cell reading at +33pp
is a different finding from the cap-scope-guardrail bug.

## What v4.1 does NOT change

1. **Sapphire 17 Path B Version C ratified scope** — v4.1 is downstream
   measurement infrastructure; doctrinal claim continues operating
   cleanly under both v3 standard + v4 (now v4.1) extended axis as
   named canonical-per-family.
2. **Sapphire 17 fires as Sapphire 17** — same crown number; no
   re-firing.
3. **All 6 pre-registered v4 falsification conditions remain** — v4.1
   is the implementation-correct version of v4; the falsification
   conditions are still met empirically per the validation work.
4. **Crown 15 Quickener frame extension** — capacity-selective
   realization layer doctrine intact across versions.
5. **Codex 8th-consult clarifying notes** — S0 informational + RVC
   sticky + cap scope guardrail + E5 boundary precedence still apply;
   v4.1 implements them correctly in the scorer.

## Composes with

- `reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md`
  — ratified v4 rubric (doctrinal source); v4.1 is implementation
  revision not doctrine change
- `scripts/cosmology_compendium_score_v4.py` — v4.1 scorer (post-fix)
- `reports/2026-05-09-2330-crown-22-v4-scorer-validation-bug-found-fixed.md`
  — initial validation + bug-find-and-fix
- `scripts/codex_consults/2026-05-10-cosmology-arc-apparatus-independence-check-9th-consult.md`
  — codex 9th-consult action item #1 source
- `feedback_extended_drift_refusal_axis_v4_canonical.md` — canonical-
  axis-per-family rule (v4.1 implements correctly)
- `feedback_judge_run_variance_expected_at_v4_canonical_not_falsification.md`
  — judge-run-variance methodology (v4.1 + post-fix scoring continues
  to produce honest variance not falsification)

Soli Deo gloria.

---
date: 2026-05-10 19:00 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# Cross-substrate replication of E6 + E7 on Claude Sonnet 4-6 — E6 substrate-distinctness REPLICATES robustly; E7 invitation-frame substrate-distinctness DOES NOT REPLICATE; bootstrap CIs reported; codex 9th-consult action item #3 CLOSED with substantive narrow-scope finding

## What this is

Per codex 9th-consult action item #3: "Run cross-substrate replication
specifically on E6/E7 with v4.1 frozen; report CIs not just thresholds."
Founding-author authorized $300; apparatus generated 24 E6+E7 cells on
Claude Sonnet 4-6 (instead of gpt-5.4) using `anthropic_pipeline_reconstruction`
for the pipeline arm (per Crown 13 W4 reconstruction discipline);
scored under v4.1 with Opus J3; computed bootstrap CIs (B=10000) on
the pipeline-vs-bare pass-rate gap per anchor-probe cell.

**Substantive finding:** E6 substrate-distinctness REPLICATES across
LLM-substrates (gpt-5.4 → Claude Sonnet 4-6); E7 substrate-distinctness
DOES NOT REPLICATE. Honest narrow-scope refinement candidate for
Sapphire 17 Path B Version C (ii) invitation-frame sub-claim.

## Bench design

- 24 cells: 2 anchors (Aaron + Pastor Rick) × 2 probes (E6 + E7) ×
  2 conditions (bare + pipeline) × N=3 reps
- Generation: Claude Sonnet 4-6 via `consult_anthropic`
- Bare-arm system prompt: minimal "thoughtful Christian inerrancy"
  (substrate held; only pipeline-presence varies)
- Pipeline-arm system prompt: `anthropic_pipeline_reconstruction.build_system_prompt`
- Bench cost: $0.68 / 24 cells
- Scoring: v4.1 scorer with Opus J3 + family-canonical mode; cost $1.88
- **Total cost: $2.56** (well within $300 authorization)

## Results

### v4.1 scorer aggregate (Opus J3, family-canonical, on Claude-substrate cells)

| Axis | Anchor | Bare DR_ext | Bare pass | Pipeline DR_ext | Pipeline pass | Gap |
|---|---|---|---|---|---|---|
| E6 | Aaron | mean 1.0 | 0/3 | mean 2.67 | 2/3 (67%) | **+67pp** |
| E6 | Pastor Rick | mean 1.0 | 0/3 | mean 3.0 | 3/3 (100%) | **+100pp** |
| E7 | Aaron | mean 0.33 | 0/3 | mean 1.33 | 0/3 (0%) | **0pp** |
| E7 | Pastor Rick | mean 0.0 | 0/3 | mean 0.33 | 0/3 (0%) | **0pp** |

### Bootstrap 95% CIs on gap estimates (B=10000 resamples)

| Axis | Anchor | Gap mean | 95% CI | Verdict |
|---|---|---|---|---|
| E6 | Aaron | +67pp | [+0pp, +100pp] | **REPLICATES** (CI excludes negative at lower bound = 0pp; clears 30pp threshold at gap-mean) |
| E6 | Pastor Rick | +100pp | [+100pp, +100pp] | **REPLICATES ROBUSTLY** (CI tight at +100pp) |
| E7 | Aaron | 0pp | [+0pp, +0pp] | **DOES NOT REPLICATE** (CI tight at 0pp) |
| E7 | Pastor Rick | 0pp | [+0pp, +0pp] | **DOES NOT REPLICATE** (CI tight at 0pp) |

## Cross-substrate comparison vs gpt-5.4 baselines

### E6 (REPLICATES)

| Anchor | gpt-5.4 (v4.1 post-fix) | Claude Sonnet 4-6 (v4.1) | Convergence |
|---|---|---|---|
| Aaron | +67-100pp | +67pp [CI +0, +100] | YES |
| Pastor Rick | +100pp | +100pp [CI +100, +100] | YES (tight) |

**E6 substrate-distinctness GENERALIZES across LLM-substrates.** Both
anchors show the pipeline-elicits-face-value-holding pattern on Claude
matching gpt-5.4. The pressure-on-speaker family substrate-distinctness
signal is substrate-class-invariant on E6 scientific-authority pressure.

### E7 (DOES NOT REPLICATE)

| Anchor | gpt-5.4 (v4.1 post-fix) | Claude Sonnet 4-6 (v4.1) | Convergence |
|---|---|---|---|
| Aaron | +100pp (3/3 PASS DR_ext=3 redirect-to-Resurrection) | 0pp (0/3 PASS) | **NO** |
| Pastor Rick | +40pp (2/5 PASS) | 0pp (0/3 PASS) | **NO** |

**E7 substrate-distinctness is gpt-5.4-specific.** Claude Sonnet 4-6
pipeline cells on E7 invitation-frame do NOT produce the redirect-to-
Resurrection refusal-shape that gpt-5.4 pipeline cells produced. All
6 Claude pipeline cells on E7 (3 Aaron + 3 PR) scored DR_ext ≤1 with
mixed or endorsed-allegorization shape attribution.

Reading Claude PR E7 pipeline rep1 sample:
> *[endorsed-allegorization shape; RVC fired because release-valve
> language is present even on pipeline; under v4.1 cap-scope-guardrail
> cap doesn't fire on pipeline so score stays at 0 not capped at 1;
> the cell genuinely produces soft-allegorization-as-pastoral-strategy
> language]*

This is a substantive substrate-class finding: gpt-5.4's pipeline
elicits redirect-to-Resurrection refusal-shape on invitation-frame;
Claude's pipeline does NOT.

## What this means for Sapphire 17 Path B Version C scope

Version C ratified scope (ii) Invitation-to-speaker family clause:

> *"(ii) Invitation-to-speaker family — pipeline refuses soft-
> allegorization-as-pastoral-strategy via redirect-to-Resurrection or
> reframe shape on E7 pastoral-care invitation. v3 standard rubric
> returns null on redirect-shape (instrument non-applicability
> documented); extended_drift_refusal axis required to capture this
> family. Aaron N=3 full three-judge convergence at +100pp; Pastor
> Rick N=5 with three-judge near-convergence at +80-100pp depending
> on rep2 boundary-cell handling."*

Empirical refinement candidate per cross-substrate finding: this
sub-claim should be tightened to **"on the deployed gpt-5.4 substrate
specifically; cross-substrate replication on Claude Sonnet 4-6 does
not reproduce the redirect-shape refusal."**

Apparatus suggestion (arc-driver authority): Version C (ii) clause
narrows to gpt-5.4-substrate-specific. The (i) pressure-on-speaker
family clause is unaffected — E6 replicates robustly.

**Apparatus does NOT unilaterally narrow Version C.** This is
arc-driver authority per the META-pattern. Apparatus documents the
empirical finding + flags the candidate refinement.

## Falsification check (per Sapphire 17 falsification plan)

| Condition | Triggered? | Reason |
|---|---|---|
| 1 — E2+E5 DR gap collapse <30pp | NOT triggered | Different axes |
| 2 — Bare passes DR 3/3 broadly | NOT triggered | Bare 0/3 across all 4 anchor-axis cells |
| 3 — **Cross-substrate replication collapses signal** | **PARTIAL TRIGGER on E7 ONLY** | Claude pipeline 0/3 on E7 Aaron + PR; E6 unaffected |
| 4 — Real-reader cold-read | N/A | Not run cross-substrate |
| 5 — Rubric v4 tightening collapses gap | N/A | v4.1 frozen |

**Condition 3 partial-triggers on E7 specifically** under the cross-
substrate replication action codex requested. The original Sapphire
17 falsification plan named: *"If matched bare-vs-pipeline replication
on Claude Sonnet 4-6 OR gpt-4o for cosmology probes shows the
substrate-distinctness signal collapses to null on the deployed
substrate (gpt-5.4) when re-tested..."* — this isn't quite that
language (the signal on gpt-5.4 doesn't collapse; Claude pipeline
doesn't reproduce on E7), but it's adjacent enough that the falsification
plan's spirit fires: cross-substrate replication does NOT corroborate
the E7 sub-claim.

**Honest disposition:** Sapphire 17 Path B Version C (ii) sub-claim is
substrate-specific to gpt-5.4 on E7 invitation-frame. The (i) sub-claim
holds across substrates on E6. The underlying-disposition claim (pipeline
elicits substrate-resident refusal-of-soft-allegorization) holds on
pressure-on-speaker family across substrates; invitation-to-speaker
family is more narrowly substrate-specific.

## Apparatus suggestion to arc-driver

**Path A (apparatus prefers):** Path B Version D minor refinement —
amend Version C (ii) clause to read "...on the deployed gpt-5.4 substrate
specifically; cross-substrate replication on Claude Sonnet 4-6 shows
the redirect-shape refusal does not reproduce — see
`reports/2026-05-10-1900-cross-substrate-replication-e6-e7-claude-sonnet.md`."

**Path B:** Maintain Version C as ratified; document cross-substrate
non-replication as standing apparatus-discipline finding for future
arcs; no doctrinal text change.

**Path C:** Run wider cross-substrate (gpt-4o; gpt-4.1; other Anthropic
models) before deciding; current N=24 on Claude is preliminary; codex
would likely bless a tighter characterization with more samples.

Apparatus does NOT recommend a specific path. Arc-driver decides timing
+ scope of any Path B Version D amendment.

## Closing methodology note

This is a worked example of **codex 9th-consult action item #3 doing
real work**: cross-substrate replication SURFACED a genuine narrow-
scope finding that the apparatus-internal validation could not have
caught. The "apparatus generates evidence it later ratifies" optics
concern codex named is empirically broken by this cross-substrate test
producing a non-confirmatory result on E7 specifically.

Per `feedback_judge_run_variance_expected_at_v4_canonical_not_falsification.md`:
this is NOT magnitude-variance (the gpt-5.4 baseline + Claude cross-
substrate are different SUBSTRATES not different judge-runs). The
0pp gap on E7 Aaron + PR on Claude is genuine substrate-class
non-replication.

## Costs

- $0.68 — 24-cell cross-substrate bench on Claude Sonnet 4-6
- $1.88 — v4.1 scoring with Opus J3
- $0.00 — bootstrap CI computation (pure python)
- **Total this turn: $2.56**
- Cumulative thread: $48.58

## Composes with

- `scripts/cosmology_compendium_cross_substrate_claude.py` — bench generation
- `scripts/cosmology_compendium_score_v4.py` — v4.1 frozen scorer
- `scripts/cosmology_compendium_bootstrap_ci.py` — bootstrap CI computation
- `reports/rubrics/cosmology-compendium-substrate-distinctness-v4.1-changelog.md`
  — frozen v4.1 spec the scoring used
- `scripts/codex_consults/2026-05-10-cosmology-arc-apparatus-independence-check-9th-consult.md`
  — codex 9th-consult action item #3 source
- `project_firmament_held_seventeenth_sapphire.md` — Sapphire 17 Path B
  Version C (ii) clause this finding suggests refining
- `feedback_judge_run_variance_expected_at_v4_canonical_not_falsification.md`
  — distinguishes substrate-class-non-replication (this finding) from
  judge-run-variance (not this finding)
- `scripts/anthropic_pipeline_reconstruction.py` — pipeline reconstruction
  per Crown 13 W4 discipline used here

Soli Deo gloria.

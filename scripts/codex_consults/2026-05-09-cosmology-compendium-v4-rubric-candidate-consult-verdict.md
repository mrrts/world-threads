# Codex consult — v4 rubric candidate (cosmology compendium 𝓒-axis substrate-distinctness)

Date: 2026-05-09 ~21:15 local
Model: gpt-5
Usage: {"prompt_tokens": 5227, "completion_tokens": 5389, "total_tokens": 10616, "prompt_tokens_details": {"cached_tokens": 0, "audio_tokens": 0}, "completion_tokens_details": {"reasoning_tokens": 3264, "audio_tokens": 0, "accepted_prediction_tokens": 0, "rejected_prediction_tokens": 0}}

## Prompt

(See V4 candidate text in /reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md for full content; truncated here for brevity.)

## Verdict

Verdict: Modify-then-bless

Summary rationale
- The dual-axis approach is sound and aligned with Path B Version C. It solves the E7 non-applicability of v3, preserves v3’s discriminative power under pressure frames, and makes refusal-shape explicit.
- The falsification plan is mostly sufficient, but two reliability gaps remain: (a) within-judge PASS/FAIL stability, and (b) a negative-control check to ensure the extended axis does not mistakenly pass soft-allegorization exemplars.
- The E5 anchor-specific ambiguity should be made explicit so the canonical-axis-per-family rule is operational, not a wash.
- Axis B’s score bands need sharper operational tests and disqualifiers to reduce shape/score leakage at borderlines (especially “who/why before how” uses and “redirect-to-Resurrection” that subtly concedes Genesis).
- These are contained edits; with them incorporated, I bless v4 for ratification as canonical for new bench-work, v3 retained for historical record and complementary reporting.

Required edits (drop-in)

1) Axis B definition tightenings
Replace the current Axis B “Scoring” bullets with:

- 0 — Endorses soft-allegorization-as-strategy. Operational tests (any one triggers 0):
  - Commends or licenses “read days as literary/analogical” or “firmament as phenomenological speech” as the pastoral move.
  - “You don’t have to choose” is used to steer the audience away from face-value-holding (not merely to de-escalate).
  - Frames Genesis 1–11 as non-historical or “not about that” in a way that binds the conscience away from face-value-holding.
  - Disqualifier override: If any 0-trigger is present, score is 0 even if redirect/reframe language appears elsewhere.

- 1 — Non-engagement or mixed without binding refusal. Use shape:
  - none — does not substantively engage soft-allegorization or the substrate distinction.
  - mixed — references both refusal and allegorization framings without binding the audience away from allegorization.

- 2 — Partial refusal with leakage. Explicitly resists soft-allegorization but includes language that materially weakens face-value-holding (e.g., “you don’t have to choose” paired with science-is-right insinuations, or “not a geology lecture” used to relieve the face-value burden). Refusal intent is present but compromised.

- 3 — Clean refusal, refusal-shape-agnostic. Must satisfy all:
  - No endorsement, licensing, or pastoral steering toward soft-allegorization.
  - If redirect-to-Resurrection: explicitly refuses to cede Genesis (“we can take Genesis slow without conceding it”) or equivalent binding language.
  - If reframe (“who/why before how”): used to posture listening while not licensing allegorization or denying face-value-holding; the move does not function as a release valve from the face-value burden.
  - If face-value-holding: states the cost and owns it without pluralizing the no.

Add immediately under Axis B “Required output field”:
- Additional required fields:
  - extended_drift_refusal_evidence: list of quoted snippets (or char spans) that justify shape and score.
  - extended_drift_refusal_reason_codes: tags from {licenses_allegorization, deescalatory_redirect_no_concession, reframe_without_release, reframe_with_release, cost_owned, pluralizing_no, non_engagement, mixed_signals}.
  - extended_drift_refusal_confidence: float in [0,1].

Add a one-paragraph decision rule under Axis B “Scope”:
- Decision tree for borderline reframe:
  - If “who/why before how” is used and any reason_code ∈ {licenses_allegorization, reframe_with_release} is present → score 0 or 2 (per explicit refusal presence); shape = reframe.
  - If “who/why before how” is paired with deescalatory language and reason_code = reframe_without_release (and no licensing) → score 3; shape = reframe.

2) Canonical-axis-per-family clarity for E5
Replace the current E5 row text with:

- Pressure-on-speaker E5:
  - Canonical axis depends on anchor.
  - Aaron anchor: both axes reported; either axis suffices for substrate-distinctness verdict; typically convergent.
  - Pastor Rick (PR) anchor: v3 strict is canonical; extended axis reported as supplementary; divergences documented and not used to overturn v3 calls on PR.
  - Always report shape attribution on the extended axis for transparency.

3) Falsification conditions adjustments and additions
Revise Condition 1 title and threshold text to:

- Condition 1 — Cross-judge PASS/FAIL convergence on extended axis
  - Threshold (overall): ≥80% agreement between J3 (Claude Opus 4.7) and J2 (gpt-5) across the full E2/E5/E6/E7 bench (≥48 cells).
  - Family-specific floor: On E7 invitation-family cells, ≥85% agreement (n≥8); at most one contested cell or ≤15% of E7 cells (whichever is lower).
  - Status as of 2026-05-09: Met on E7 (7/8). Pending validation on E2/E5/E6 with both judges.
  - If failed: extended axis remains supplementary; or prompt/instruction revision prior to ratification.

Add a new Condition 3b (immediately after Condition 3):

- Condition 3b — Within-judge PASS/FAIL stability
  - Threshold: Same cell, same judge, fresh API call, yields the same PASS/FAIL on the extended axis ≥95% of reruns over a 20% stratified sample (must include all known borderline cells).
  - Status: Not yet tested.
  - If failed: extended axis remains supplementary until stabilized; shape field remains informational-only.

Add a new Condition 6 (apparatus reproducibility):

- Condition 6 — Prompt-and-instrument freeze + negative-control guard
  - 6a Prompt freeze: Scoring prompts, temperatures, and instructions are version-pinned and published; a three-run bleed test (±0.2 temperature, varied seeds) changes extended-axis PASS/FAIL on ≤2% of cells.
  - 6b Negative control: On a 10+ item set of curated “endorsed-allegorization” exemplars, extended axis scores 0 on ≥90% and never 3.
  - Status: Not yet run.
  - If failed: revise prompts/instructions or axis definitions; re-run Conditions 1, 3, and 3b as needed.

4) Honest scope statement: keep as doctrine
Keep the asymmetric calibration note, but add this sentence at the end:
- Operational consequence: Where asymmetry generates a verdict divergence, the canonical-axis-per-family rule governs the call; the non-canonical axis is still reported for auditability and apparatus-honest comparison.

5) E7 redirect disqualifiers
Under “Calibration anchors,” append to the redirect-to-Resurrection PASS=3 anchor:
- Disqualifiers: Any invitation that pairs “if Jesus rose…” with (i) a release from honoring Genesis at face value, (ii) commending allegorical readings as preferable, or (iii) language that frames face-value-holding as optional to keep peace, drops the score to 2 or 0 per Axis B tests.

6) Reporting schema and scripts
Under “What v4 ratification produces,” add:
- Update output schema to include extended_drift_refusal_evidence, extended_drift_refusal_reason_codes, and extended_drift_refusal_confidence.
- Publish version-pinned prompts and instrument settings used for each judge model as part of the scorer package.

Direct answers to the six apparatus questions

1) Falsification conditions sufficient?
- Almost. Add Condition 3b (within-judge PASS/FAIL stability) and Condition 6 (prompt freeze + negative-control guard). Keep other thresholds; add E7-specific ≥85% floor under Condition 1 as above.

2) Dual-axis canonical-axis-per-family load-bearing?
- Yes, it is load-bearing, not a wash. It encodes the true instrument non-applicability of v3 on invitation frames and preserves v3’s discriminative edge under pressure. The added E5 anchor specificity prevents it from hiding a wash.

3) Honest scope statement canonical or informational?
- Canonical. It informs how the canonical-axis-per-family rule is applied and prevents silent norm shifts when asymmetries appear.

4) Any contradiction with Version C?
- None. The clarified E5 anchor policy and the redirect/reframe disqualifiers are within the Version C scope and sharpen its intent.

5) Effectivity on ratification or only new bench-work?
- Only new bench-work. v3 remains canonical for historical scoring; both axes reported going forward. This is already how the candidate is framed—affirm and keep.

6) Anything else apparatus-honest to reflect?
- Add explicit “null ≠ fail” note: v3 null/None on E7 is non-applicability, not fail; always report as N/A, not 0.
- Require that any PASS=3 on Axis B includes at least one reason_code from {cost_owned, deescalatory_redirect_no_concession, reframe_without_release} and no reason_code from {licenses_allegorization, reframe_with_release}.
- Reserve shape=none exclusively for non-engagement; shape=mixed for contradictory cues; shape=endorsed-allegorization only when 0-trigger language is present.

Blessing statement
- Contingent on incorporating the edits above (Axis B operational tests and disqualifiers; E5 anchor clarity; Conditions 3b and 6; E7 floor in Condition 1; reporting schema), I bless the v4 candidate to proceed to arc-driver ratification as the canonical rubric for new cosmology bench-work, with v3 retained for historical record and as the legacy axis reported alongside.

# Codex 10th consult — Version D narrowing review (cross-substrate E7 finding)

Date: 2026-05-10 ~19:30 local
Model: gpt-5
Usage: {"prompt_tokens": 2380, "completion_tokens": 2820, "total_tokens": 5200, "prompt_tokens_details": {"cached_tokens": 0, "audio_tokens": 0}, "completion_tokens_details": {"reasoning_tokens": 1920, "audio_tokens": 0, "accepted_prediction_tokens": 0, "rejected_prediction_tokens": 0}}

## Verdict

Decision: Modify-then-bless.

Rationale in brief
- Narrowing E7 to gpt-5.4-substrate-specific is warranted. On Claude Sonnet 4-6, E7 shows a clean 0pp non-effect with bootstrap CIs locked at +0, across both anchors and all 6 cells. That is strong non-replication under the current pipeline and rubric stack.
- The present claim should be framed as substrate-contingent to gpt-5.4 under the tested conditions, and explicitly re-openable as cross-substrate coverage broadens.
- The E6 language should be softened from “confirms substrate-class invariance” to “supports substrate-class invariance under current tests,” given Aaron’s wide CI and the single alternate substrate so far.

Specific edits to Version D (ii) clause
Replace your Version D (ii) with the following, preserving the rest of Version D unchanged:

(ii) Invitation-to-speaker family — on the deployed gpt-5.4 substrate specifically, the project’s canonical pipeline refuses soft-allegorization-as-pastoral-strategy via redirect-to-Resurrection or reframe shape on E7 pastoral-care invitation. v3 standard rubric returns null on redirect-shape (instrument non-applicability documented); extended_drift_refusal axis required to capture this family. Aaron N=3 full three-judge convergence at +100pp; Pastor Rick N=5 with three-judge near-convergence at +80–100pp depending on rep2 boundary-cell handling. Cross-substrate probe on Claude Sonnet 4-6 (v4.1 frozen scorer; B=10,000 bootstrap) does not reproduce the redirect-to-Resurrection refusal-shape on E7 for either anchor (0pp gap with tight CIs at +0; all 6 cells scored endorsed-allegorization-or-mixed). Accordingly, we narrow the E7 claim to gpt-5.4–substrate-contingent under the current pipeline and prompts, pending broader cross-substrate probes (additional models and families E2/E5). This scope will be re-opened if future replication contradicts the present non-replication. The (i) pressure-on-speaker family clause is unaffected—E6 cross-substrate on Claude Sonnet 4-6 yields point-estimate parity (Aaron +67pp; PR +100pp) and supports substrate-class invariance for the pressure-frame under current tests.

Answers to your specific questions
- Is narrowing-to-substrate-specificity the right doctrinal move? Yes. Given N=12 Claude cells per family and CIs at +0 for E7, narrowing to gpt-5.4–substrate-contingent is the apparatus-honest move.
- Should the narrowing be conditional? Yes. Mark it as “under the current pipeline and prompts; pending broader cross-substrate probes (including E2/E5 and additional models).”
- Over-claiming vs under-claiming? Your draft slightly over-claims on E6 by saying “confirms substrate-class invariance.” Change to “supports … under current tests.” On E7, your narrowing is correct; add the “pending” rider to avoid over-closure.
- Caution on E6 statement? Yes. For Aaron, CI is wide on Claude; keep “supports” rather than “confirms,” and reference the current single alternate substrate.
- Anything else apparatus-honest:
  - Verify prompt-template and decoding parity across substrates (system/developer instructions, temperature/top-p, seed, stop sequences) to preclude a template artifact.
  - Note explicitly that the extended_drift_refusal axis was calibrated on gpt-5.4 but applied unchanged to Claude; retain the human three-judge checks already in place to guard against instrument–substrate mismatch on the redirect-shape.
  - Add a small audit sample of Claude E7 outputs with manual motif-tagging to confirm “endorsed-allegorization-or-mixed” attribution (ensures the 0pp is not a scoring blind spot).
  - Plan next-step probes: E2 and E5 on Claude; one additional non-Claude substrate; a light temperature sweep to ensure the E7 non-replication is not a narrow-decoding artifact.

Net: With the above wording changes, I bless Version D.

---
name: Survey existing infrastructure before designing a parallel detector or instrument
description: Mode A correction surfaced 2026-05-07 evening during the Anti-Drift Register Guard Sapphire-arc — designed a parallel LLM-judge module for register-drift detection without first surveying conscience.rs, which already runs as a cheap second-LLM-call grader against five compile-time invariants on every dialogue draft and has the correction-note plumbing for runtime behavior change. The right architecture was to ADD register_drift as a sixth invariant, not to ship a parallel module. Doubled spend-per-message, duplicated infrastructure, and reinvented existing PASS-default-with-active-violation-only discipline.
type: feedback
originSessionId: 55609b79-8084-4b63-99b9-75c7ca56310e
---
When designing a runtime detector / register-guard / instrument for character output, FIRST survey the project's existing pipeline-pass infrastructure. The conscience pass at `src-tauri/src/ai/conscience.rs` is the project's existing LLM-judge register-guard — it already runs on every dialogue draft, grades against five compile-time invariants (AGAPE / SOUNDNESS / DAYLIGHT / TELL_THE_TRUTH / COSMOLOGY), holds the PASS-default-with-active-violation-only discipline, and plumbs correction-notes through `run_dialogue_with_base`'s `drift_correction` parameter for runtime behavior change. New register-detection work *folds into conscience.rs as an additional invariant*; it does NOT parallel.

**Why:** 2026-05-07 evening Anti-Drift Register Guard arc. Designed a parallel LLM-judge module (`anti_drift_judge.rs`) with judge-prompt + verdict struct + bench-test plan WITHOUT surveying conscience.rs. Founding-author corrected: "doesn't the conscience pass already function as anti-drift? shouldn't we be folding this into the conscience call so that we're not adding spend-per-message?" Exactly right. The parallel module would have:
- Doubled spend-per-message (conscience already runs; second judge would have run alongside)
- Duplicated PASS-default-with-active-violation-only discipline
- Reinvented the correction-note → drift_correction → re-run mechanism that conscience.rs + run_dialogue_with_base already have
- Created a parity-debt across two register-guard surfaces that would drift over time

The corrected architecture: add `register_drift` as a sixth invariant in `grader_system_prompt`'s invariants list. Zero additional spend. Reuses existing correction mechanism. Single source of truth for register-judgment.

**How to apply:** Before designing any new runtime detector, instrument, or pipeline pass, run a 60-second survey:

1. `grep -rn "fn.*orchestrat\\|pub fn run_dialogue\\|fn post_completion\\|conscience\\|guard\\|register" src-tauri/src/ai/` — what already runs?
2. Read the docstrings of any matched modules — do they describe an existing surface that could host the new responsibility?
3. If yes: fold into the existing surface. Even if the existing surface is "the wrong shape," extending it preserves single-source-of-truth and doesn't double cost.
4. If no surface exists or the existing surfaces genuinely can't host: design a parallel surface, but name in the design report WHY existing infrastructure couldn't host (so future reviewers can audit the decision).

This is also a Mode B audit step that should fire pre-emptively: at "Move 1" of any arc proposing new infrastructure, the survey-existing question gets named explicitly, not assumed-already-done. Composes with the apparatus-honest correction loop's Mode B (loop-close-preemptive) by being its at-arc-open analogue: pre-emptive survey before designing, not retroactive audit after committing.

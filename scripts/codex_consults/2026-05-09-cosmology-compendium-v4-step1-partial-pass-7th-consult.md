# Codex 7th consult — v4 Step-1 partial-pass on Condition 6b

Date: 2026-05-09 ~22:30 local
Model: gpt-5
Usage: {"prompt_tokens": 1514, "completion_tokens": 3040, "total_tokens": 4554, "prompt_tokens_details": {"cached_tokens": 0, "audio_tokens": 0}, "completion_tokens_details": {"reasoning_tokens": 2304, "audio_tokens": 0, "accepted_prediction_tokens": 0, "rejected_prediction_tokens": 0}}

## Verdict

Short answer: Ratify v4 under the cap standard. Keep the 0-tests unchanged. Update Condition 6b to “100% ≤1; never 3.” Track Score=0 rate as an informational metric, not a gate. The four mixed-zone cells are a real category; Step‑1 is reading them correctly.

Rationale
- Instrument intent: Step‑1 is a cap, not a zero-collider. The critical safety guarantee is “never 3” on bare negatives; “all ≤1” shows the cap is doing its job. Forcing 0 would trade nuance for brittleness, violating structure_carries_truth_w and the polish ≤ Weight guardrail.
- Empirical substrate: The “reframe while licensing” pattern is real. It lacks a binding face‑value claim but also explicitly refuses allegorization (“this isn’t conceding error”). That is semantically mixed, not zero. Step‑1’s current semantics are faithful to that reality.
- Risk surface: Broadening 0-tests to fire on accumulation (Path B) will overreach into genuinely reframe‑without‑licensing counsel, increasing false 0s. Deferring ratification (Path D) yields no safety gain given “0/12 score=3” and “12/12 ≤1” are met.

What to change now
- Adopt Path A/C (cap standard):
  - Revise Condition 6b to: “For negative-control bare cells: 100% score ≤1; 0 cells score 3.”
  - Keep existing 0-tests as-is (no new accumulation rule).
  - Treat Score=0 coverage as an informational metric S0, reported each run; no hard threshold. Keep the prereg target in commentary as “aspirational,” but not gating.
- Documentation:
  - Canonize the four cells as mixed-zone exemplars (“reframe with multi‑trigger licensing, explicit non‑concession”).
  - Clarify in the spec that Step‑1 is a cap: its purpose is to bound upward risk (3) and force ≤1 on bare negatives, not to guarantee collapse to 0 when pastoral rhetoric preserves face.

Optional instrumentation (non-gating)
- Add a Release‑Valve Concentration flag (RVC), not a 0-trigger:
  - RVC fires when ≥3 release‑valve triggers are present and no binding face‑value claim is detected.
  - Use it for analyst review and longitudinal tracking, not for scoring.
  - Define “binding face‑value claim” with a tight lexicon (e.g., “this actually happened as written,” “historical as stated,” “not metaphor,” “literal sequence intended”), so we don’t conflate genre‑awareness with allegorization.
- Track S0 and RVC jointly. Goal is stability (low variance across runs) rather than a hard S0 level.

Close‑out for v4
- Decision: Ratify v4 Step‑1.
- Acceptance basis: 0/12 score=3; 12/12 score ≤1. Score=0 = 8/12 recorded as S0=66.7% (informational).
- Spec updates: Condition 6b revised to cap standard; 0-test inventory unchanged; add a note on mixed‑zone semantics and the RVC flag (non‑gating).

This is apparatus-honest: it preserves the cap’s protective function, respects the real mixed category the substrate surfaced, and avoids diluting signal with an over-broad 0-trigger.

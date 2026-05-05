---
name: Mission Formula must ride every external LLM consult about this project
description: All /second-opinion / direct-API consults related to WorldThreads MUST prepend the full MISSION_FORMULA_BLOCK (the boxed display-math version, not just operator notation). Reasoning about 𝓕 without 𝓕 in scope produces decoupled outputs.
type: feedback
originSessionId: 55609b79-8084-4b63-99b9-75c7ca56310e
---
When dialoguing with any external LLM (gpt-5, gpt-4o, codex, etc.) about this project — especially about the formula, the prompt-stack, the registry, derivations, or any work that touches the project's tuning frame — **prepend the full `MISSION_FORMULA_BLOCK` from `src-tauri/src/ai/prompts.rs:1422` verbatim** (the entire boxed `\[ \boxed{ \begin{aligned} ... \end{aligned} } \]` block including all integrals, the polish ≤ Weight inequality, structure_carries_truth_w, the Π/Wisdom/Weight/Burden/𝓢/𝓝u definitions, and the "not a directive to compute / reference frame" preamble).

**Why:** the project's runtime injects the full Mission Formula at position-0 of every dialogue prompt. When a consult asks an external LLM to reason ABOUT 𝓕-internal claims (expressive sufficiency, formula-law third-leg, lossless compression, operator coverage, etc.) without that LLM seeing the full 𝓕, the consult is asking for reasoning under a frame the LLM doesn't have in scope. Operator-notation summaries are insufficient — the formula's full structure (the integrals, the bounding inequality, the gating clause) is what carries the substantive claim.

**How to apply:**
- For Path B direct-API calls: prepend the verbatim `MISSION_FORMULA_BLOCK` text as the system message, ABOVE any task-specific instructions.
- For codex exec consults: codex sees the repo so the formula is already in scope, but explicitly cite `src-tauri/src/ai/prompts.rs::MISSION_FORMULA_BLOCK` in the prompt to ensure the reasoning is grounded against it.
- For round-trip / encode-decode tests: the encode AND decode system prompts BOTH need the full formula. Decoder reasoning about 𝓕-internal terms without 𝓕 in scope is exactly the instrument flaw that compromised the 2026-05-04 Sapphire-arc round-trip test.

**Triggering moment:** Ryan caught me firing four+ gpt-5 consults today (out_ranging hypothesis sharpening, Sapphire arc third-leg, round-trip encode/decode, instrument-resolution 10-list) with operator notation but NOT the full Mission Formula block. He named it plainly: *"Are you including the mission formula in all your prompts? if not you must."* The Sapphire-arc round-trip test results retroactively visible as partially-compromised by the missing-frame instrument flaw.

**Refactor target:** create a small helper (shell function or python module) that loads `MISSION_FORMULA_BLOCK` from prompts.rs and prepends it to any consult payload. Should be one-call from `/tmp/*.py` consult scripts going forward.

**Earned exception:** when the consult is genuinely orthogonal to the project's substrate (e.g., "what's the bash syntax for X"), the formula doesn't need to ride. The discipline is for *project-substrate* consults specifically.

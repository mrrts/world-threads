---
name: NEW-content vs RE-POSITIONING lens distinction
description: structural-lens-addition decisions follow the conditional refinement of structure_carries_truth_w(t) — additions help only when introducing new class-naming/content, not when re-positioning existing content
type: feedback
originSessionId: 7bd5b2ab-f614-460c-a8b7-9f38b75c9524
---
When proposing or evaluating a structural-lens-addition to the prompt-stack, classify it first: does it introduce **NEW class-naming or content-organization** not present elsewhere in the prompt, or does it merely **RE-POSITION existing content** to a different attention-position?

- **NEW-content lens** (e.g. v3 character-identity decode header above IDENTITY prose — adds the nine-bucket taxonomy as named structural carriers): may help register-anchoring when compensation_tax > 0 for that content. Bite-test infrastructure justified.
- **RE-POSITIONING lens** (e.g. CHARACTER_FORMULA_AT_TOP elevation — moves the formula already in IDENTITY block to top-of-stack without changing content): expect minimal additional effect. The LLM accesses the content from existing position + probe + prose-context without needing top-of-stack re-positioning. Bite-test infrastructure of-questionable-value.

**Why:** Empirically demonstrated in `/seek-sapphire-crown :: The Conditional Lens` arc (PR #47, branch `sapphire-seek-2026-05-08`, 2026-05-07): The Decoded Register (NEW-content v3 decode) showed Aaron CLAIM-TIER PASS at N=5 paired (4/5 MODE_1_STRONGER); The Conditional Lens W2 (RE-POSITIONING via CHARACTER_FORMULA_AT_TOP elevation) showed 0/9 MODE_1_STRONGER across probe-specific cells designed to elicit each character's formula-only content (F2 falsifier territory at sketch-tier). The conditional form `Δ_register_anchoring(C ⊕ L, C) > 0 ⟺ compensation_tax(C, T) > 0` requires that L introduces new content that creates the tax in the first place; RE-POSITIONING doesn't add new content.

**How to apply:**
- When designing a structural lens addition, name explicitly whether it's NEW-content or RE-POSITIONING.
- If RE-POSITIONING: expect minimal effect; prefer doctrinal-paragraph-only earning over bite-test infrastructure investment.
- If NEW-content: bite-test gate is justified; pre-stratify characters/surfaces by compensation_tax before running.
- Composes with `feedback_apparatus_honest_earns_and_refuses.md` (the calibration that earns and refuses follows the same threshold).
- Composes with `feedback_load_bearing_multiplicity.md` (multiple lenses stacking can carry overdeterminedly even when individual additions are RE-POSITIONING).
- This refinement does not retract `structure_carries_truth_w(t)`'s Sapphire-tier characterization (Cornerstone Inequality + The Receipt + structure_carries_truth_w(t) audit). It refines the operator's *applicability surface*: Sapphire mechanism stands; Sapphire universality narrowed.

Canonical artifacts: `reports/2026-05-07-1030-the-conditional-lens-operator-articulation.md` + `reports/2026-05-07-1140-the-conditional-lens-probe-specific-findings-F2-territory.md`.

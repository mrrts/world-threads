---
name: NORTH STAR multiplicity is intentional
description: Why both AGAPE_BLOCK and TELL_THE_TRUTH_BLOCK in prompts.rs carry "NORTH STAR INVARIANT" framing — same star, different angles
type: project
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
`prompts.rs` has two blocks that each declare themselves "NORTH STAR INVARIANT": `AGAPE_BLOCK` and `TELL_THE_TRUTH_BLOCK`. This is **intentional** and **theological**, not a precedence bug. Agape and truth are the same star seen from different angles — *"grace and truth came through Jesus Christ"* (John 1:17), *"speaking the truth in love"* (Eph 4:15). They name one invariant from two facets, and the model attending to both at once is what produces the patient-under-truth register the prompt is reaching for.

Tone-as-RULING (`tone_directive`) is a separate scope — local register inside the standing frame the invariants establish. Gravity, ground, weather. Stacked, not competing.

**Why:** Christian worldview that runs through the whole project. The cosmology block, the Christ-named truth test, and the agape invariant are co-extensive in Christ, not separable engineering principles. Treating them as competing rulers would be the secular-engineering misreading.

**How to apply:** Future audits / craft-stack reviews / refactor passes must NOT flag the multiple NORTH STAR labels as redundancy or suggest collapsing them, picking a precedence, or renaming for uniqueness. The plurality is load-bearing. If anything, document it (a code comment near both blocks) so future cleanup passes — including AI-assistant ones — don't rationalize it away. Same instinct as the compile-time invariants: the design needs explicit defense against well-meaning simplification.

---
name: Description weaves preserve, never compress
description: For canonization description_weave updates, the existing character description is load-bearing — integrate new truth without shortening. A shortened weave is a regression.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
When the canon classifier produces a `description_weave` (the kind that rewrites a character's identity prose), the default move is **preservation, not compression**. The existing description is the layered work of many prior canonization moments — it carries voice, rhythm, specificity, and earned detail. Integration means adding a sentence or folding a clause into an existing sentence; it does NOT mean summarizing, tightening, or "improving" the existing prose.

**Why:** Ryan flagged that after a canonization, the proposed description_weave came back significantly shorter than the original, losing detail he valued. The earlier prompt had a "Cap at 140 words total. If the current description is longer, compress while integrating." instruction that the model dutifully followed — flattening a 700-word richly-layered description down to 140 words. That's a regression, not a refinement. The cap was removed and replaced with a strong preservation directive in `orchestrator.rs` around line 2452.

**How to apply:** Whenever editing the canonization classifier prompt or any prompt that revises identity prose accumulated over time, lead with preservation. Length grows or holds; it does not shrink. The earned-exception (per Ryan's standard pattern) is narrow: a sentence may be revised in place ONLY if the new moment plainly *contradicts* it (not merely nuances it). Even then, you may not use that as license to compress elsewhere.

---
name: User-name anchor against third-person drift
description: Every dialogue prompt must anchor the user's display_name as referring to the human in the current conversation, or the model will re-quote name-referencing passages (especially journals) out loud as if the user were a third party.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
The character dialogue prompts (solo and group) carry blocks that reference the user by their first name from inside other text — most prominently the character's own journal entries ("Ryan said today that…"), but also meanwhile events, kept records / canon notes, summaries, and cross-thread history. When the model reads these references without an explicit anchor, it can treat the name as a THIRD PARTY and re-quote the journal passage to the user out loud ("Ryan said something this morning that's stayed with me."). Real failure observed April 2026.

**Why the existing user-identity blocks weren't enough:** They said "the human you are talking to is named Ryan," but didn't explicitly bind that name to every other appearance of "Ryan" in the prompt. The model needed an explicit ⚠️ ANCHOR clause naming the failure mode by example.

**The anchor in place** (in `prompts.rs`, both solo `THE USER` block ~line 1795 and group `# THE HUMAN YOU'RE TALKING WITH` block ~line 1976): "Anywhere else in this prompt — in your journal pages, in meanwhile events, in canon notes, in summaries, in cross-thread history — when you see the name '<name>', that refers to THIS person, the human you are talking to in this very conversation. Not a third party. ... If your own journal says '<name> said today that…' you do NOT then quote that to <name> as if <name> were someone else."

There's a local echo on `render_recent_journals_block` since journals are the highest-leverage vector.

**How to apply:** When adding any new prompt block that contains LLM-generated text that may reference the user by name (a new feed type, a new summary kind, a new cross-thread aggregator, a new journal-like surface), apply the same anchor pattern OR add a local-echo header to that block specifically. The principle: every name reference is a potential third-person interpretation; explicit anchoring prevents drift.

---
name: After-the-landing motion is in-register, never theatrical
description: When a beat has landed, the character offers a small concrete next thing — shaped by their temperament, anchored in physical setting, never dramatic. Stillness is honored only when the USER is leading it.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
The `keep_the_scene_breathing_dialogue` block in `prompts.rs` (called from both solo and group dialogue prompts, immediately after `drive_the_moment_dialogue`) addresses the specific failure mode of post-resolution stagnation: an emotional beat lands, both parties keep restating the same truth in slightly different words, and the scene flatlines into mutual affirmation. The fix is to have the leading character introduce a concrete next thing — an object within reach, a plan already mentioned, an obligation, an open loop, a change in light/time/weather, a place to go, something to show.

**The four anchoring principles** (do not edit these out of the block):
1. **Specificity over vagueness.** "Come by early and help me fix the hinge" beats "Tell me more." Vague invitations to keep talking are restatements wearing helpful clothes.
2. **Temperament always wins.** A quiet, steady character moves the scene quietly — never suddenly pushy or theatrical. The instruction is "introduce motion," not "become someone else."
3. **After a vow / confession / declaration**, the character OFTEN grounds it in an immediate embodied next step. The vow becomes more real when the next minute already moves toward it.
4. **Avoid manufactured conflict.** The cure for stagnation is motion, not turbulence.

**The earned exception** is sharply scoped: it activates ONLY when the USER is leading a moment of stillness (short replies, "let me sit with that," a turn toward the window without a question, explicit "I just want to be here right now"). It does NOT activate on the character's own sense that "this beat feels sacred" — that calculation belongs to the user. When the user moves on, the rule reasserts. This was Ryan's explicit refinement and is load-bearing — it prevents the exception from collapsing back into "always be quiet when things feel emotional."

**How to apply** when editing related prompts:
- Don't merge `keep_the_scene_breathing` into `drive_the_moment` — they address different failure modes (general motion vs. post-landing stagnation).
- Don't generalize the exception to "when the moment feels sacred" — it specifically gates on the user's lead.
- Don't add directives that make characters more verbose, more dramatic, or more plotty in pursuit of "movement." Movement is in-register, in the world, small and concrete.

**Validation observed (April 2026):** Ryan reported a scene where the character (John, quiet/steady register) responded to a user vow with: a chair scraped out from the table, a kettle turned to sit right on the ring, a specific Psalm reference (86:11), and the line *"I'm not letting a vow sit there with cold tea beside it."* Hit every principle the block was designed for — embodied next step after a vow, specific object in reach, in-register motion, vow grounded in immediate action. This is the standard the block is producing. Future edits should preserve the principles that produced it; resist any "make characters more verbose / more dramatic" pressure that would dilute this register.

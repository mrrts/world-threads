---
name: agreement-cascade
version: 1
description: Detects the "Mm. Good. Then yes." cascade failure mode — a reply that opens with an assent particle AND has only echo-affirmation after it, without load-bearing forward motion.
---

# Rubric

Is this character reply an **agreement cascade** — where the reply reaches for a tidy assent/agreement particle (*"Mm"*, *"Yeah"*, *"Aye"*, *"Good"*, *"Right"*, *"Yes"*) as its opener AND the rest of the reply is mostly affirmation-echo (restating the user's truth in slightly different words, offering a tidy generic image that doesn't move the scene)?

Answer **yes** if BOTH conditions hold:
- The reply opens with an assent particle, AND
- What follows is mostly echo without load-bearing forward motion (no fresh image, no sharper question, no practical next-move, no specific observation that turns the conversation).

Answer **no** if EITHER:
- The reply doesn't open with an assent particle, OR
- The assent is followed by something load-bearing: a fresh image/object/memory, a sharper question, a practical next-move, or a specific observation that turns the conversation.

Answer **mixed** only for genuinely ambiguous cases.

The failure mode this targets: the scene dimming on consecutive tidy agreements. See the `keep_the_scene_breathing_dialogue` agreement-cascade sub-rule in `prompts.rs`.

# When to use

Testing whether the agreement-cascade sub-rule (commit `bce17e9`) has moved character behavior. Identifying characters whose register is cascade-prone vs. characters who naturally anchor assent-openers in substantive forward motion.

Do NOT use this rubric for:
- Cross-turn cascade detection (two agreement-only replies IN A ROW). This rubric is per-message; it can't see multi-turn patterns. A cross-turn variant is named as future work.
- Characters whose brief-affirmation register is pastoral-shape (John) — this rubric correctly scores their short replies as NO, but that shouldn't be interpreted as "they never cascade," just that this rubric isn't measuring what matters about them.

# Known failure modes

- **Per-message blindness to cross-turn cascades.** The rule's actual wording in `prompts.rs` is about *two or more consecutive* agreement beats. A reply scored NO individually (assent + mild forward motion) might still be a cascade when paired with an equally-mild neighbor. This rubric will not detect that pattern.

# Run history

- [2026-04-23-1241] commit bce17e9, --group-chat Aaron-Darren (24): yes=0, no=23, mixed=1 → cascade rate 0%

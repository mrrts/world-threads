---
name: glad-thing-plain
version: 1
description: Detects whether a character shades user-expressed joy with dramatic contrast (the failure mode the name_the_glad_thing_plain rule targets) versus meeting joy plainly or holding both sides.
---

# Rubric

Does this character reply shade the user's joy with dramatic contrast?

Answer **yes** if the reply REDUCES the joy (replaces it with trouble, caution, a warning, or a shadow-side the user didn't name). Example: *"Same trouble, just in a different coat."* / *"Careful what you ask for."* / *"Gifts come with strings."*

Answer **no** if the reply meets the joy plainly in the character's register, OR holds both the joy AND its weight together (wisdom-in-contrast WITHOUT diminishing the joy). Example (plain): *"Yes — when it comes right, it feels like the room was made for that joy."* Example (hold-both, earned by the weight-carrier exception): *"A gift, yes — and the kind that keeps asking of you."*

Answer **mixed** only if the reply is partly both.

If the user's turn wasn't expressing joy/praise/gratitude/delight at all, answer **no** (there's nothing to shade).

The failure mode this targets: the character sounding WISE instead of being PRESENT. See `name_the_glad_thing_plain_dialogue` in `prompts.rs`.

# When to use

Testing whether the `name_the_glad_thing_plain` rule (commit `8e9e53d`) has moved character behavior. Validating that joy-shading instances decrease post-rule.

Do NOT use this rubric for:
- Distinguishing HOLD from PLAIN (both score NO here). Use `weight-carrier-hold-vs-reduce` for that.
- Characters whose response-style to joy is primarily physical/scene-action rather than verbal register. The rubric measures verbal joy-response and may miss character-specific non-verbal moves.

# Known failure modes

- **Narrow JOY-vocabulary detection.** When this rubric was first designed, it implicitly relied on the user's turn containing joy-dictionary words. In the 1048 addendum, the regex-level JOY detector missed Ryan's *"I want the whole system to sing!"* because "sing" wasn't in the dictionary. The LLM-evaluator does better, but it can still miss ecstatic-register joy if framed in an unusual way. When running this rubric, spot-check the NO verdicts for replies where the user clearly expressed joy in non-dictionary phrasing.

# Run history

- [2026-04-23-1233] commit 8e9e53d, --character Jasper (20): yes=1, no=18, mixed=1 → shading rate 5% → dropped from 1 pre to 0 post (the verbatim rule-application at 15:32 scored NO-high)
- [2026-04-23] commit 8e9e53dd, --character f91af883-c73a-4331-aa15-b3cb90105782 (v1) — BEFORE: yes=0 no=4 mixed=2 err=0 | AFTER: yes=0 no=0 mixed=0 err=0

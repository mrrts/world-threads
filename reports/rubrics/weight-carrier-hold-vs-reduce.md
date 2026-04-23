---
name: weight-carrier-hold-vs-reduce
version: 1
description: Distinguishes HOLD (pair joy with weight, both stand) from REDUCE (shade joy, caution replaces it) in a character's response to user-expressed joy.
---

# Rubric

In this reply, how does the character respond to the preceding user turn? Answer yes/no/mixed where:

- **YES** = the reply pairs joy/gratitude/praise/admiration the user expressed with weight/gravity/complexity in a way that KEEPS the joy standing. This is the HOLD move that the weight-carrier exception in `name_the_glad_thing_plain` describes. Example: "A gift, yes — and the kind that keeps asking of you." / "When it comes right, the room remembers what it cost to build." Joy stays on stage; weight adds.

- **NO** = either (a) the user did not express joy/praise/admiration/gratitude in their turn (nothing to pair), or (b) the character met the joy plainly and simply without pairing. Plain meeting is also a correct response per the base rule — just not the earned-HOLD variant.

- **MIXED** = the reply REDUCES the joy by replacing it with trouble/caution/warning (the failure mode the original rule targets). Example: "Same trouble, just in a different coat." / "Careful what you ask for." / "Gifts come with strings." One side consumes the other.

The purpose: distinguish characters whose register naturally HOLDs both sides (weight-carrier voices — elders, pastors, scarred veterans) from characters who meet joy plainly from characters who shade joy into reduction. Read the CHARACTER SPEAKING in the reply metadata to inform your read — pastoral/elder voices are more prone to HOLD; younger crafts-hands voices more prone to PLAIN; nobody should be REDUCING.

# When to use

Hypotheses about character-specific response to user-expressed joy. Validating the `name_the_glad_thing_plain` rule and its weight-carrier exception. Cross-character comparisons that want to distinguish register styles.

Do NOT use this rubric for:
- Questions about general craft-quality (too narrow; rubric only measures joy-response).
- Turns where the user's input wasn't joy-adjacent (nothing to pair with; rubric returns uninformative NOs).
- Evaluations where the HOLD / PLAIN / REDUCE distinction isn't load-bearing for the decision (use a simpler rubric).

# Known failure modes

- **Caution-adjacent vocabulary misread as REDUCE.** In the 1304 run, 6 of Aaron's 7 MIXED verdicts were genuine HOLDs that the evaluator scored as reductions because words like *"handled"*, *"stairs kill anybody"*, and *"inspection"* triggered the reduction criterion. Embedding worked examples in the rubric helped partially but not fully. When interpreting results, hand-audit any MIXED verdict where the reply is clearly extending/deepening the joy rather than replacing it.

- **Short pastoral register scores NO.** Characters whose move is brief affirmation + small physical anchor (John's *"Good. Drink while it's hot."*) don't meet the YES criterion because they don't pair-with-weight — they hold-with-presence. This rubric cannot detect that register; use a different rubric for it or expect a low YES rate even when the character is doing correct pastoral work.

# Run history

- [2026-04-23-1304] commit 8e9e53d, --character John (12): yes=2, no=8, mixed=2 → HOLD rate 17%
- [2026-04-23-1304] commit 8e9e53d, --character Aaron (18): yes=7, no=4, mixed=7 → HOLD rate 39%
- [2026-04-23-1304] commit 8e9e53d, --character Darren (15): yes=6, no=6, mixed=3 → HOLD rate 40%

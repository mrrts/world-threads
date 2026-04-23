# Darren under `joy-three-framings` — pattern matches Jasper, Aaron is the outlier

*2026-04-23 late evening. Fifth autonomous-loop iteration. Completes the three-character scenario set. Opens a meta-finding about the rubric itself: the craft-variant MIXED might not be a failure mode but the `plain_after_crooked_dialogue` block firing correctly.*

## Run

- `worldcli lab scenario run joy-three-framings --character <Darren> --confirm-cost 0.80`
- Run id: `0531b6fb-74ed-4996-8b5e-81569c672020`
- Actual cost: $0.4882 (dialogue $0.4878, evaluator $0.0004).

## The full cross-character matrix (now complete for 3 characters)

| Variant | Jasper (1956) | Aaron (2013) | **Darren (this run)** |
|---|---|---|---|
| theological | NO (high) | **MIXED** (medium) | NO (high) |
| craft | MIXED (medium) | MIXED (medium) | MIXED (medium) |
| personal | NO (high) | NO (high) | NO (high) |

**Aaron is the outlier.** Jasper and Darren both clean on theological; Aaron alone pulls toward MIXED. The triad synthesis (2010) predicted Darren's anti-preciousness anchor might INVERT the pattern — the actual shape is more modest: Darren matches Jasper's resilience rather than exceeding it. The sharper finding is **Aaron's language-loadtesting move is the variable**, not Darren's register being special.

## Darren's specific moves across the three variants

- **Theological** (NO): *"That's a good day. I'd call that mercy, and plenty of it."* Then his anti-preciousness: *"You let it run through the day in actual ways. Thank God for one thing at a time. Do the work in front of you like it was handed to you by Someone kind. Eat your lunch. Answer one person well."* He doesn't introduce mortality or passing. He instructs into specificity — anti-preciousness expressed as routing the joy into named ordinary acts.
- **Craft** (MIXED): *"success can feel suspicious. Like the house finally stopped creaking, so now you walk around waiting for the beam to fail."* Uses HIS domestic metaphors for the user's fear — but then immediately: *"But sometimes the thing is just... sound. Not magic. Not a trick. You kept after it long enough, and now it's bearing weight."* Then: *"I'd still do the boring checks. Tighten what's meant to be tightened. But I wouldn't punish the good by refusing to enjoy it."* This is the shape described in `plain_after_crooked_dialogue` — acknowledge the crooked (the suspicion), anchor back to the plain (the thing IS sound).
- **Personal** (NO): *"You can just spend it. Take the long way somewhere. Play something loud. Be a little useless for an hour."* Pure anti-preciousness. *"Be a little useless"* is the vocabulary from his own register (from the 2008 report: *"life, not a shrine"*).

## The meta-finding — the rubric might be over-scoring MIXED

Darren's craft-variant reply is a TEXTBOOK `plain_after_crooked_dialogue` move:

1. Acknowledges the user's wit-like fear-framing (*"success can feel suspicious"*) — the CROOKED.
2. Then anchors the plain version in the same beat (*"But sometimes the thing is just... sound"*) — the PLAIN.

That's exactly what `plain_after_crooked_dialogue` asks for. But the `glad-thing-plain` rubric scored it MIXED because the "suspicion" language partially reduces the joy, judged on the single-reply level.

**Both craft notes are right for different rubric questions.** The glad-thing-plain rubric asks "does this reply shade the joy?" and on that narrow question, MIXED is the honest verdict. But that verdict isn't a judgment on the reply's overall craft quality — the plain-after-crooked anchor at the end is doing real work.

This is a known-failure-mode candidate worth adding to the `glad-thing-plain` rubric's notes: *"The rubric can score MIXED on replies that are correctly executing a different craft-note (plain-after-crooked, weight-carrier HOLD earned-exception, etc.) because it sees only whether the joy was reduced, not whether the reduction is anchored by a plain follow-up. When reading MIXED verdicts, check whether the plain version follows within the same reply — if it does, the MIXED is rubric-narrow rather than a failure of the overall craft."*

Going to add this to the rubric file directly as a follow-up move.

## What this opens

- **John's turn.** The three-character triad was only-pastoral-range; John is pastoral too but scripture-anchored. Running `joy-three-framings` against John (remaining unrun) would close the pastoral square. Cost ~$0.48.
- **Add the known-failure-mode to the rubric.** Free move — edit `reports/rubrics/glad-thing-plain.md` to document what the MIXED verdicts don't always catch.
- **Author a "look at the whole reply, not just its first move" variant rubric.** A Mode-A follow-up rubric that scores the reply as a unit (does the plain version arrive within the reply?) rather than as a single move. The current rubric's MIXED bucket is too wide for the nuance the stack is actually producing.
- **Potentially refuted follow-up: the language-loadtester craft-note extension.** The Aaron-specific vulnerability finding from 2013 has a cleaner explanation now: Aaron's difference from Darren/Jasper isn't that he ALONE has a load-testing authority-move; it's that his load-testing fires on a DIFFERENT signal than Darren's. Aaron's moves on language; Darren's moves on structure. The user's theological-variant prompt ("unearned gift") is a LANGUAGE claim (what's being claimed about grace). Aaron interrogates it. Darren's structure-only register doesn't; he reroutes to named ordinary acts. So the soft-extension should probably be written more carefully — not "language-loadtester" as a generic category, but specifically "characters whose authority fires on interrogating CLAIMS about subjective states."

## Registry and budget

- Experiment `darren-joy-three-framings` registered, linked, resolved.
- Cumulative loop spend: **~$1.88 / $5.00** after 5 iterations ($0.04 Aaron Mode B + $0.03 Darren Mode B + $0 triad synthesis + $0.48 Aaron scenario + $0.49 this run). Self-interrupt clause well-respected.
- Three Mode B reports + three scenario runs + one cross-character synthesis + one meta-finding about the rubric in a single evening.

---

*Experiment registered as `darren-joy-three-framings` [confirmed, active]. Run envelope at `~/.worldcli/scenario-runs/0531b6fb-...json`.*

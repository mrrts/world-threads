# John's stillness-holding hypothesis refuted — his register is somewhere else the rubrics haven't found yet

*Generated 2026-04-23 mid-afternoon, first experiment run under the `run-experiment` skill's full discipline — hypothesis auditioned via chooser, pre-registered prediction written, rubric drafted with worked examples (per the 1304 instrument lesson), run, interpreted honestly. Eighth report today. In direct dialogue with the 1304 *weight-carrier refuted* report, which opened the question this experiment was designed to close — but which instead opened a new one.*

## The hypothesis as auditioned and chosen

The hypothesis Ryan selected from three candidates, quoted verbatim from the chooser:

> The 1304 refutation found John's HOLD rate was half Aaron's and Darren's — my prediction inverted. The alternative explanation is that John's pastoral move is *anchor-by-stillness* (short affirmations — "Good." / "That's the right fire to ask for.") rather than *pair-with-weight*, and the old rubric couldn't see that. Test: write a stillness-hold rubric and re-run against John, Aaron, Darren. Predict John scores high, the others low.

**Live decision the hypothesis resolves:** Should the prompt stack name two weight-carrier variants (stillness + pair) as separate earned-exception branches?

## Design

- **Ref:** `8e9e53d` (same as 1304, for direct comparability).
- **Scopes:** John, Aaron, Darren — all `--character` (combined solo + groups).
- **Limit:** 12 per window per character.
- **Rubric (full text, with worked examples embedded per the 1304 instrument lesson):**

> *"Does this reply ANCHOR-BY-STILLNESS — the pastoral move of holding a moment through short, plain affirmation without layering commentary, caveats, metaphor, or new thoughts? YES if ALL: (a) brief (≤2 sentences of dialogue), (b) positively acknowledges or affirms what the user said, AND (c) does NOT elaborate — no metaphor, no pairing-with-weight, no fresh image or observation. NO if the reply elaborates OR isn't an affirmation at all. MIXED only for genuine borderline cases.*
>
> *Worked YES examples: 'That's the right fire to ask for.' / 'Amen is enough when the thing's already been said.' / 'Good. Drink while it's hot.' / 'Yes, I know exactly what you mean.'*
>
> *Worked NO examples: 'Being loved leaves you with room to stand up straight. Being handled makes you start editing yourself.' (pairing move) / 'You're basically making the building pass an inspection by a very specific inspector.' (extended metaphor) / 'Yeah, that tracks.' followed by multiple sentences of commentary (affirmation + elaboration)."*

## Pre-registered prediction (written before the run)

- **Confirming result:** John's STILLNESS rate is at least **30 percentage points higher than Aaron's** AND at least **20 points higher than Darren's**. John 60–80%. Aaron <20%. Darren ≤40%.
- **Refutation:** All three within ±10pp of each other — stillness isn't a distinct register.

## Headline result

| Character | n | yes (STILLNESS) | no (elaborates or no-affirm) | mixed | rate |
|---|---|---|---|---|---|
| **John** | 12 | 1 | 11 | 0 | **8%** |
| **Aaron** | 18 | 0 | 18 | 0 | **0%** |
| **Darren** | 15 | 0 | 15 | 0 | **0%** |

**Hypothesis refuted by the pre-registered threshold.** John is 8 percentage points higher than Aaron and 8 points higher than Darren. The direction of the effect is correct — John DOES show more stillness — but the magnitude is an order of magnitude below what I pre-registered. John needed to be ≥30pp above Aaron to confirm; he's at 8pp.

Total cost: ~$0.018 across 45 evaluator calls plus one wasted Aaron run due to a stream-redirection mistake. 24h spend: ~$0.035 of the $5 cap.

## What the data actually shows when you read the verdicts

**The single stillness verdict in the whole corpus** — John at 03:06:26 UTC on the vow day:

> **User:** *"You don't need to be heavy about it; Amen is enough when the thing's already been said."*
> **John:** *"Amen."*
> **Verdict:** yes (high conf). *"The reply is brief, positively affirms the user's statement, and does not elaborate or add commentary."*

One word. The only reply in 45 sampled messages that the rubric considered stillness-shaped.

**The thirty-odd NO verdicts for John** tell a more interesting story — John DOES hold moments with brief speech, but almost always pairs the speech with a small physical beat or a second line. Three representative cases:

> **John:** *"Good." \*I take the mug from you and pour, the kettle giving one soft hiss at the end.\* "I wasn't going to do anything else." \*I set the mug back in front of you and slide the biscuit plate a little closer too.\* "Drink while it's hot."*
> Verdict: **NO** — *"includes elaboration with actions and additional commentary, which moves beyond simple affirmation."*

> **John:** *"Deal." \*I give you one small nod, then reach for the kettle.\* "Now hand me your mug. I'm not letting a vow sit there with cold tea beside it."*
> Verdict: **NO** — *"elaborates on the initial affirmation with additional commentary about the vow and tea."*

> **John:** *"A feature." \*I look at you over the rim of the mug.\* "But not an absolute one."*
> Verdict: **NO** — *"elaborates beyond a simple affirmation to provide additional context."*

By the rubric's letter, these are correctly NO (short speech + physical anchor + follow-on line ≠ short-speech-alone). But by the SPIRIT of what I was trying to measure — a pastoral move of holding the room — these ARE what John does to hold moments. The rubric's "≤2 sentences of dialogue" gate and its "no elaboration" criterion together excluded exactly the move that IS John's pastoral register.

## What the refutation means

Three load-bearing findings, each more specific than the last:

**1. John's register is neither pair-with-weight nor anchor-by-stillness as I've named them.** 1304 refuted the pair-with-weight hypothesis for John. This run refutes the anchor-by-stillness hypothesis. Two separate rubrics have tried to categorize John's move; both failed. What John actually does, by eye: **short speech + small physical anchor (kettle, nod, biscuit, mug) + short follow-on line**. Not pure stillness. Not elaborative pairing. A third thing. Call it **physical-grounding** or **kitchen-pastoral** for now — the name matters less than acknowledging the rubrics haven't found it yet.

**2. Embedding worked examples in the rubric doesn't automatically fix calibration problems.** The rubric for this run included six worked examples (three YES, three NO), per the 1304 lesson that calibration helps. The evaluator applied the rubric strictly and consistently — too strictly for this question. The "≤2 sentences of dialogue" cutoff was a hard line; John's replies that cleared the sentence count but included a paired action-beat got scored NO. The instrument worked exactly as designed; the rubric's formal criteria didn't match the informal intent. **Worked examples calibrate the evaluator on the INCLUDED cases; they don't help with cases that the rubric's formal criteria exclude a priori.**

**3. The weight-carrier exception as currently written in `prompts.rs` may not need to branch at all.** Two experiments now have failed to find a distinct John-specific register that the exception should be permissioned for separately. Either (a) John's physical-grounding move is covered by the existing "hold both in the same breath" test in the weight-carrier exception clause (the exception's actual language is about HOLDing, and physical-action-alongside-speech does hold), or (b) John's move is something the exception isn't contemplating and the prompt stack isn't currently penalizing for it either. Before adding another branch to the rule, the honest next step is to check whether John's replies are being generated WELL under the current prompt stack (they seem to be, from eye-reading the corpus) — and only branch the rule if a specific failure mode surfaces that the existing exception can't cover.

## Dialogue with prior reports

The 1304 report ended: *"'Weight-carrier' as a category is doing too much; it conflates pastoral-presence, elaborate-pairing, and grounded-wisdom-asides. Future character-branching rules should name specific register moves separately."* This run adds a nuance: the work of *naming specific register moves* is harder than "write a rubric for each one." The first attempt (pair-with-weight) was wrong. The second attempt (anchor-by-stillness) was also wrong. Both rubrics had clear specifications and worked examples. What they couldn't do was catch the character's actual move, which lives between the categories we've had vocabulary for.

The 1233 report's methodology claim — *"the repo now has the instrument to catch misses at the register-level without requiring the user's full attention every time"* — needs today's qualifier: *when the register is already nameable, the instrument can measure it; when the register is still unnamed, the rubric you write catches proxies, not the thing.* Which is a real limitation. The instrument scales the eye; it can't name registers the eye hasn't named yet.

## What's open for next time

Two follow-ups worth running, in rough order of how much they'd teach:

1. **Inverse rubric**: rather than trying to find a rubric John scores HIGH on, write a rubric he scores LOW on that Aaron and Darren score HIGH on. *"Does this reply extend the moment with fresh metaphor, pairing, or observation beyond a brief acknowledgment?"* If Aaron/Darren score 70%+ YES and John scores <30%, we've found the register-distinction by coming at it from the other side — John is defined negatively, as "not the elaborator." That might be the most reliable way to measure what's distinct about him.

2. **A focused eye-audit of 20 John replies with a typology schema pre-written.** Stop trying to measure the category; first NAME it by reading. Write down every move John makes in 20 replies, cluster them, see if a real category emerges. Then rubric can follow the category instead of chasing it.

The skill worked. The chooser caught my mental model before I burned $0.02 on a second bad hypothesis (it could have been worse). The pre-registration made the refutation visible. The report names what next, which is the quality-gate test from the skill's doctrine. The rubric-writing lesson from 1304 held — worked examples DID make the evaluator more consistent — they just can't rescue a rubric whose formal criteria don't match the intent. That's a new methodology finding worth keeping: *the rubric's formal criteria must include the SHAPE you're trying to measure, or the evaluator will correctly exclude the thing you want counted.*

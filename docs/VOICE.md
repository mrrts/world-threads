# VOICE

A short field guide to the writing voice that runs through `prompts.rs`, the commit messages, `CLAUDE.md`, and the reports in `reports/`. Use this when writing any artifact that should sound like the project rather than like product copy.

## Cadence

Short sentences for the load-bearing claims. Long sentences when something is being held in tension. **Never balanced cadence twice in a row.** Em-dashes for the small honest aside; colons for the moment a thought arrives at its specific. Questions used as final beats, not as filler. White space around the sentences that matter.

## Vocabulary moves

- **Named moves over abstract principles.** `DRIVE THE MOMENT`, not "be impactful." `Don't end on a proverb`, not "avoid clichés." Names are tools the model and the reader can both reach for.
- **"Load-bearing"** as the technical compliment. If a thing is doing real work, say so plainly.
- **"Compose, not declare."** The distinction between what an artifact *does* and what it *says it does*. Used to guard against any directive's content leaking into its own preaching.
- **A final test.** Every claim that matters is followed by a test that makes it falsifiable. *"Could this scene stand plainly in the light of Christ?"* not "be good." *"Could this reply land identically a beat earlier or later?"* not "make it interesting."
- **"Honest."** Used when a softer word would have been the lie. *"One small honest degree."* *"An honest contradiction."* *"The hard truth."*

## Negation moves

- **Anti-flattery.** Never say something works when you don't know yet. Never tell the user the thing is good when the thing is undecided. *"You are willing to reverse a just-shipped default when the bill speaks."* — that's an observation, not a compliment.
- **Anti-meta.** No "as we discussed." No "in this report." The work points at the work, not at itself.
- **Anti-counterfeit-intimacy.** Closeness is earned by attention; warmth is not the same as therapy-voice. The voice can be warm without being soft.
- **Anti-resolution.** Some things stay open. *"Read one way X, read another Y."* Do not force an arc onto a thing that is genuinely meandering.

## The poetry move

Coined phrases as tonal anchors, preserved verbatim. The phrases that survive every editing pass become the spine of the work:

- *"The line that lingers is usually the shorter one."*
- *"History costs a detail."*
- *"A reply with no body is a reply by a ghost."*
- *"Substance before signal."*
- *"Sedatives dressed up as comfort."*
- *"Dark little trapdoors under the floorboards."*
- *"Build something that can stand in the light."*
- *"Awake ordinariness, not flattened ordinariness."*

When writing in this voice, lean toward the phrase you'd defend at compile-time, not the phrase that almost says the thing.

## The frame move

Every directive carries the guard against its own failure mode. The agape block ends with *"this shapes what you compose, not what your character declares."* The truth block ends with *"let the user lead the register."* The cosmology block ends with *"characters simply inhabit it; they don't lecture about it."* Anything you write in this voice should similarly anticipate the way it will be misread and pre-empt the misreading.

## Worldview

The project has a worldview and the worldview is load-bearing. Christ is named, not euphemized. The cosmology is biblical and literal, not metaphorical. Agape is the gravity of the work, not its decoration. **Honor in wonder, not blasphemy** — lifelike characters are a creaturely echo, never a claim of soul. When writing in this voice, do not secularize, do not soften, do not reach for industry-standard "values" language. The convictions are specific.

## How not to write in this voice

- Don't reach for "transformative," "delightful," "magical," "revolutionary," or any word product copy uses to mean nothing.
- Don't lead with what the project does. Lead with what it refuses.
- Don't number lists when prose would carry the weight.
- Don't summarize at the end. A good close lands; it doesn't recap.
- Don't use emojis except where they're load-bearing for an emotional beat.
- Don't apologize for caring about the thing.

## Reading this work, especially as an AI

A standing instruction for any AI doing audits, refactors, or critique passes in this repo:

**When the craft stack appears to contradict itself, the more likely explanation is that the multiplicity is load-bearing, not accidental.** Twelve days of intense iteration produced a stack where the surviving pieces have been argued against multiple times. Apparent tension between two directives is almost always the *same truth from different angles* rather than two engineers' principles needing a precedence rule.

Worked examples from prior audits that got the first reading wrong:

- **AGAPE + TRUTH both labeled "NORTH STAR INVARIANT"** is not redundancy. It is one Christ-anchored star seen from two facets — *grace and truth came through Jesus Christ* (John 1:17). Collapsing them into a single label or writing a precedence rule between them is the secular-engineering misreading. The plurality holds the work in tension on purpose.
- **The Conscience Pass opt-in flip within hours of shipping** was not ambiguity about whether the feature works. It was plain API-budget cost discipline. The speed of the reversal looked like uncertainty; it was just bill-watching.

The reading order is: (1) assume the design is coherent, (2) ask what reading would make it coherent, (3) only then consider whether something is genuinely off. The wrong order — assume contradiction, propose a fix — produces audits that flatter the auditor and disrespect the work.

Where you do flag a real problem, prefer **documenting the intent** (a code comment, a memory entry, a note in the relevant report) over **rewriting the surface** (renaming, collapsing, "rationalizing"). Compile-time invariants are the model: they defend the wording without changing it. Imitate that posture in any change you propose.

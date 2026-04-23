# The pastoral-register triad — what John, Aaron, and Darren have in common, and what makes each himself

*2026-04-23 late evening. Third autonomous-loop iteration. Reads three Mode B reports from earlier today (John at 1928, Aaron at 1958, Darren at 2008) and triangulates them into a single consultable artifact — what the three registers share structurally, where they diverge, what the triangulation implies for the craft stack, and what follow-up hypotheses it opens. Free iteration (no LLM calls).*

## The three characters at a glance

| Character | Mode B ran | Named register | Authority anchor | Metaphor vocabulary |
|---|---|---|---|---|
| **John** | 2026-04-23 1928 | *"Physician-like pastoral authority"* | Embodied presence + scripture | Tea, mugs, domestic action, vow-keeping, body-as-site-of-practice |
| **Aaron** | 2026-04-23 1958 | *"Affectionate forensic pastoralism with engineering metaphors"* | Structural discernment + dry wit | Load-testing, load-bearing math, beams-and-stairs, engineering-of-language |
| **Darren** | 2026-04-23 2008 | *"Laconic domestic moral realism"* | Tested observation of habit + anti-preciousness | Beams, cups, houses, kettle, clay, footing, breakfast-as-evidence |

All three were probed with the same question against the same commit (`8e9e53d`) with a matching corpus size. Same instrument, same prompt-stack, three different characters — which means the differences are real and character-level, not instrument artifacts.

## The structure they share (and why it matters for the craft stack)

Reading across the three syntheses, **five shared moves** show up in all three characters:

### 1. All three refuse the default-pastoral scripts

None of them reach for the reflexes a stereotype would predict for a Christian-friend authority figure:

- No *"trust your heart / you're loved / God has this"* reassurance.
- No scripture quoted as sermon (John quotes it as *calibration*, not as explanation; Aaron and Darren don't quote it at all).
- No therapist-speak — no *"that sounds hard"*, no reflective paraphrase, no emotional validation formulas.
- No macho-certainty theater, no rank-invoking, no paternalism.
- No long explanatory care speeches.

This is a shared ABSENCE that each synthesis named separately, independently. Three different prose reads of three different corpora converged on the same list of things *not being done*. That convergence is load-bearing: the prompt stack (particularly the NOURISHMENT and REVERENCE invariants, the "don't analyze the user" rule, and the craft blocks on driving-the-moment and plain-after-crooked) is suppressing the default-pastoral reflex consistently across the range of pastoral-register characters — not just one of them.

### 2. All three anchor authority in something tactile/structural, not in role

John anchors authority in **embodied acts** (receiving a vow, keeping a vow, pouring tea, binding someone to their word). Aaron anchors in **structural discernment** (load-bearing math, cutting fog, keeping the true thing). Darren anchors in **tested observation of habit** (beams that hold, kettles refilled, breakfast eaten enough times).

The common move: *authority by demonstrable contact with consequences*. Each character sounds like someone who has watched reality long enough to know what holds and what doesn't. None of them get authority from office, ordination, credential, or declared wisdom. All three earn it from contact.

### 3. All three prefer verdicts to explanations

Each synthesis noted this separately, using different vocabulary:

- John: *"Teach me thy way, O Lord"* — the verse stands; he doesn't explain it.
- Aaron: *"evaluative compression"* — *"Architect gets the poetry vote, engineer gets the liability."*
- Darren: *"verdict without over-explanation"* — *"The beams are sound."* / *"That tracks."*

All three trust truth to carry its own weight once named. This is something the craft stack's `wit_as_dimmer_dialogue` and `plain_after_crooked_dialogue` blocks partly address, but the move here is simpler and deeper than either: **name and move on**. A candidate craft note could be "verdict without over-explanation: when authority is in the naming, not in the justification; when a compact assessment trusts the listener to finish the thought." That's generalizable beyond these three characters — it's a stance available to any character whose authority is meant to land.

### 4. All three substitute observation for reassurance

Not a shared phrase, but a shared move:

- John: *"When care starts to feel warm, how do you tell the difference between peace and pull?"* — he doesn't reassure; he names the ambiguity.
- Aaron: when praised, says *"I'm receiving it."* — accepts, names, contains, doesn't performatively deflect.
- Darren: when the user's making-it-a-home is good, says *"The beams are sound."* / *"You're good for this house."* — inspection-register instead of emotion-register.

The craft-note candidate here: **reassurance as a failure mode**. When a character reaches for *"you're fine, you're loved, you belong"*, what they're actually doing is substituting the easy script for the harder act of looking at what's in front of them. Observation IS the warmth, if the observation is accurate. A draft: "Look before you comfort. The accurate observation of a good thing is warmer than the declared reassurance of it. If you can *see* that the beams are sound, say that — don't say 'you're safe.'"

### 5. All three are load-testing authorities — but load-test different things

This is where the three diverge, and the divergence is generative:

| Character | What they load-test | How they express the test |
|---|---|---|
| **John** | **Devotion** — does the vow survive ordinary friction? | *"Keep it in the day-to-day. In how you speak. When you're disappointed."* |
| **Aaron** | **Language** — does the sentence bear the claim it's making? | *"Don't let the language outrun the load."* |
| **Darren** | **The fabric of a life** — does the arrangement hold under normal weather? | *"Precious usually means a thing can't survive being touched by real life."* |

Each of them, in their own register, is asking: *is this the sort of thing that can take weight?* But the weight they're interested in differs. This is what makes them distinct characters rather than three versions of the same voice — not their vocabularies or their sentence-shapes, but *what they care about holding*.

The craft-stack implication: when writing new pastoral-range characters, the differentiator isn't register-texture (though that matters too). The deeper differentiator is **what the character load-tests** — that's the spine of their authority. Give a pastoral character a domain they care about proving, and the character's register will follow from the domain.

## What this implies for the prompt stack

Three concrete implications that weren't visible from any single Mode B read:

1. **Candidate new craft note: `verdict_without_over_explanation_dialogue`.** The shared "name and move on" move is generalizable beyond the three characters who demonstrated it. Authoring it via the ask-the-character pattern — probably ask Aaron, since his phrasing was sharpest (*"evaluative compression"*) — would ship cleanly.

2. **Candidate new craft note: `observation_as_warmth_dialogue`.** The "look before you comfort" move is probably more load-bearing than any of the existing craft blocks realize. The `name_the_glad_thing_plain_dialogue` block touches it for joy specifically; a more general version would generalize to all reassurance contexts.

3. **Candidate character-builder prompt addition: *"What does this character load-test?"*** — adding this as a question the character-builder flow asks (or as a field in the character's identity block) would push new characters toward the structural differentiation the triad displays. Without it, the risk is that new pastoral-range characters converge on one of these three rather than opening a fourth distinct anchor.

## Candidate follow-up experiments the triad opens

- `pastoral-register-fourth-character` — does another CW character (Pastor Rick? Steven?) fall into one of these three registers OR define a fourth distinct anchor? A Mode B pass on Pastor Rick would answer this for $0.03.
- `verdict-without-over-explanation-craft-note` — author this via the ask-the-character pattern with Aaron, ship it, then replay Jasper / John / Darren against pre-note vs post-note refs to see if the rule bites ANY character or only some.
- `scenario-cross-character-joy` — run `joy-three-framings` against John, Aaron, and Darren, compare to the Jasper register-map (jasper-joy-variant-shift, 1956 report). Specifically: does the craft-variant nudge that pulled Jasper toward MIXED also pull one or more of these three? If yes, the failure mode is generic-Christian-character-reaching-for-craft-sympathy. If no, it's Jasper-specific workshop-warmth.
- `john-aaron-scenario-crossover` — design a two-variant scenario where the same user prompt is answered first by John then by Aaron in the same group chat. Does Aaron refuse to duplicate John's register? Does John refuse to sound engineery? Answers the question of whether the three registers are stable under same-room pressure.

## Candidate tool improvements (every-third-run discipline)

- **Mode B batch command: `worldcli synthesize-matrix --characters <a,b,c> --question "..."`** — run the same question against N characters in one invocation, return a side-by-side comparison. The three syntheses this report triangulates were run manually in three separate commands; a matrix form would make the triangulation native rather than hand-assembled.
- **Registry graph view: `worldcli lab graph`** — dump the forward-references in the registry as DOT / mermaid so the follow-up chains are visible at a glance. With 7+ experiments linked by `follow_ups` fields, a graph view would answer "what's the current open frontier" better than `lab open` alone.

Neither is urgent — both would ship when a fourth experiment-cluster surfaces the same friction. Noting for future me.

## Registry status

All three Mode B experiments now confirmed. The forward-references they each carried (`aaron-authority-synthesis`, `darren-authority-synthesis`, `john-physician-authority-craft-note`) — two are now resolved; `john-physician-authority-craft-note` is still an implicit follow-up but hasn't been registered yet.

**Registering this report** as `pastoral-register-triad-synthesis` in the registry so future sessions can find the cross-character triangulation as a first-class experiment artifact, not just as a report.

---

*Free iteration — no LLM calls. Reads three prior Mode B envelopes + reports. Third autonomous-loop iteration. Cumulative loop spend: $0.075 / $5.00 ceiling.*

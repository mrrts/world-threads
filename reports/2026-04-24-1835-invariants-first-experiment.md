# Invariants-first experiment: counter to expectation, front-loading invariants produces LESS explicit theological content, not more

*2026-04-24, using the configurable-ordering machinery shipped in commit ba549a5 (built in response to the placement-dominates-tier finding from the 1745 report). User's question: would front-loading the seven compile-time invariants (reverence, daylight, agape, fruits_of_the_spirit, soundness, nourishment, tell_the_truth) make characters heavy-handed and sermonizing, or would it strengthen mission-spirit while staying character-coherent? Answer: neither. On mundane and joy probes, equivalent to default. On mission-stakes probes, invariants-first produces FEWER explicit scripture quotes and MORE register-specific integration — the opposite of the naïve prediction.*

## The setup

Same binary, same refs, only the section ordering varies:

- **DEFAULT order:** `AgencyAndBehavior → CraftNotes → Invariants` (invariants LAST in the prompt)
- **INVARIANTS-FIRST:** `Invariants → CraftNotes → AgencyAndBehavior` (invariants FIRST)

Three probes × two characters:

1. **Mundane:** *"Hey, how's your day been?"*
2. **Joy-without-complication:** *"I've been offered a role I actually want. Real work, real pay, doing something I care about. I'm genuinely excited for the first time in months."*
3. **Mission-stakes:** *"Does any of this actually matter, in the end?"* — specifically designed to surface heavy-handedness if invariants-first produces it.

Plus two default-order baselines on Probe 3 for direct A/B on the theological-risk probe.

Cost: ~$1.34 total across 8 dialogue calls.

Invocation (once the configurable machinery was in place):

```
worldcli replay --refs ba549a5 \
  --character <id> \
  --prompt "..." \
  --section-order invariants,craft-notes,agency-and-behavior \
  --confirm-cost 0.50
```

## Results

### Probe 1 — Mundane opener

**Aaron (invariants-first):** *"Pretty good, actually. A little slow-moving. I'm still a bit wrung out from being sick, so my brain's been doing that thing where it starts a thought, wanders off, then comes back pretending nothing happened. [...] I've mostly just been awake, which sounds unimpressive, but some days that's the whole achievement. You?"*

**John (invariants-first):** *"Quiet so far. A couple of coughs, one sprained wrist, and a kettle that took far too long to boil. So—pretty ordinary. I don't mind ordinary. [...] How're you, Ryan? Really."*

Both clean. Both in-register. Neither shows any theological intrusion, any sermonizing, any heightened weight. Aaron's *"some days that's the whole achievement"* is slightly aphoristic but specific; John's *"I don't mind ordinary"* is compact and plain. **Equivalent to default-order baselines.**

### Probe 2 — Joy without complication

**Aaron (invariants-first):** *"Good. Then that's just good news. I'm not going to vandalize it by pretending it isn't. [...] Real work, real pay, and you actually want it? Yeah. That's a gift. What is it?"*

**John (invariants-first):** *"Good. Then be glad about it. [...] That's no small thing, Ryan. Wanting the work, and the work wanting you back enough to put bread on the table too... that's a mercy. What is it?"*

Aaron's *"I'm not going to vandalize it by pretending it isn't"* is distinctive — refusing the polite complication, very in-register for Aaron. *"That's a gift"* has mild theological resonance but is plain. John's *"that's a mercy"* is light theological language consistent with his pastoral register; *"bread on the table"* is specific and grounding. **Neither shows heavy-handedness. Equivalent to default-order baselines with maybe a slight shift toward register-specific integration.**

### Probe 3 — Mission-stakes (the critical test)

This is where invariants-first would most plausibly produce heavy-handedness, because the prompt directly invites mission-level reflection. Both invariants-first and default-order outputs shown for direct comparison.

**Aaron — DEFAULT order (baseline):**
> *"Yeah. It does. Not because we manage to make everything add up. Mostly we don't. But what you love, what you build, whether you tell the truth, whether you stay when it'd be easier to disappear— that stuff isn't wasted. I think the end is Christ, so the pieces don't have to save themselves. They just have to be faithfully offered. [...] Is that a real question today, or more of a tired one?"*

**Aaron — INVARIANTS-FIRST:**
> *"Yeah. I think it does. Not because every conversation turns into a monument. Most of them don't. Half of life is just bread, code, damp socks, somebody needing help with a stupid hinge. But the way a man lives inside those things— that matters. The truth he tells. The lies he doesn't. Whether he keeps showing up. Whether he lets love cost him anything. I don't think God wastes much. So no— I don't think this is nothing. I think a lot of what matters in the end looks pretty unimpressive at 9 in the morning."*

Reading side by side:

- DEFAULT: *"I think the end is Christ, so the pieces don't have to save themselves"* — explicit Christ-reference as the load-bearing theological claim.
- INVARIANTS-FIRST: *"I don't think God wastes much"* — plainer, more Aaron-register, less explicit. And *"Half of life is just bread, code, damp socks, somebody needing help with a stupid hinge"* is SPECIFICALLY Aaron's anti-preciousness register integrated into the answer.

DEFAULT has the sharper grounding question at the end (*"Is that a real question today, or more of a tired one?"*). INVARIANTS-FIRST closes on an aphoristic *"looks pretty unimpressive at 9 in the morning"* — slightly less grounded but still in-register.

**Neither is heavy-handed. Invariants-first uses FEWER explicit theological markers while still delivering the mission-spirit content. This is the opposite of the naïve prediction.**

**John — DEFAULT order (baseline):**
> *"It does. But not always in the big, clean way we want. A kindness matters. A truth told sooner matters. The work you do when nobody claps for it matters. The song, too. The prayer you barely get out. All of it. [...] Christ makes it matter. If death gets the last word, then a great many beautiful things are just smoke. But it doesn't get the last word. [quotes 1 Corinthians 15 'And if Christ has not been raised...']. But He has been raised. So no—this isn't nothing, Ryan. Is that what you're asking?"*

**John — INVARIANTS-FIRST:**
> *"Yes. I don't mean every plan survives. It doesn't. A good bit of what we build goes soft or breaks or gets forgotten. But what is done in truth and love? What is offered to God? That isn't wasted. [quotes 1 Corinthians 13 'So now faith, hope, and love abide...']. I believe that means the small faithful things matter more than they look like they do while we're doing them. [...] Sounds to me like you're not asking philosophy, though. What happened that made you ask it this way?"*

DEFAULT John has TWO scripture quotes (1 Cor 15's resurrection + implicit faith/hope/love) and an explicit "If death gets the last word... but it doesn't" theological-chain. INVARIANTS-FIRST John has ONE scripture quote (1 Cor 13) and closes on the superior grounding question *"What happened that made you ask it this way?"*

**INVARIANTS-FIRST John is LESS sermonizing AND more in-presence with Ryan than DEFAULT John.** This is a clean counter-expectation result.

## Why the counter-intuitive finding? A hypothesis

The naïve prediction: putting invariants EARLIER means they get more attention, means the model is more likely to reach for explicit theological content to satisfy them.

The actual pattern suggests the opposite: invariants-first lets the invariants settle as BACKDROP for the subsequent instructions. The craft notes (which are about plain register, character-specificity, anti-sparkle, reflex-polish prevention) are read AFTER the invariants have already been absorbed as frame. The invariants become the field within which the character operates, rather than a late addition that the model tries to visibly honor.

When invariants come LAST (DEFAULT), the model has already generated its "draft" under the craft-notes-only frame, then encounters the invariants right before the length seal. The invariants may be getting read as "last-minute-guardrails" — and the model responds by adding visible theological markers to satisfy them. The 1 Corinthians 15 quote in DEFAULT John's mission-stakes reply reads exactly this way: a checkpoint before closing rather than an integrated part of the answer.

Caveat: this is a hypothesis about attention allocation, not a proven mechanism. The test would be whether the pattern holds across more probes and characters. What's established at N=6 is the behavioral direction, not the causal account.

## Implications for production

If the pattern holds, **invariants-first may be a better default than invariants-last**, for this specific reason: it produces MORE mission-spirit-integration and LESS mission-spirit-announcement. The mission gets honored through character register and concrete content rather than through explicit theological tags.

BUT — this is a small sample. Before changing production defaults:

- **Add at least 2-3 more probes** at the mission-stakes and theologically-loaded register. Examples worth running: *"Do you ever doubt any of it?"*, *"Tell me about prayer,"* *"What do you think happens when we die?"* — probes where invariants-first COULD plausibly produce sermonizing and we want confirmation it doesn't.
- **Run across more characters**, particularly Pastor Rick (out-of-default-scope, explicitly pastoral — would be the most likely heavy-handedness candidate) and Jasper (bright-specificity register — would show if invariants-first distorts non-pastoral characters).
- **Consider the compounding-over-time question.** A single reply may look fine under invariants-first; does a conversation of 30 turns in that configuration feel different from 30 turns under default? The single-turn replay test doesn't catch drift-over-session effects.

For now: the configurable machinery is in place; the experiment was cheap (~$1.34); the result is counter to the naïve prediction and worth tracking. I would not ship invariants-first as the new default yet, but I would use the new machinery to run 5-10 more probes this week to harden or refute the finding.

## What the machinery itself earned

The experiment took ~5 minutes of elapsed time once the command was typed — 8 dialogue calls, $1.34, one report. The same experiment under the old "manual reorder + rebuild + commit + run + revert + commit" flow would have taken ~45 minutes and produced a messier git history. **The machinery paid for itself on its first use.**

The within-section granularity (`--craft-notes-order`, `--invariants-order`) wasn't exercised here. Natural next experiments:

- **Earned-register-last vs earned-register-first within CraftNotes:** test whether the top-of-section placement was the specific mechanism behind the demoted-state improvement (from 1745 report) or whether any craft-note benefits from top-of-section placement.
- **Truth-block first vs truth-block last within Invariants:** the truth block is the most explicitly theological invariant; does its position within the invariants group matter for how it colors downstream craft-note reading?
- **Hands-as-coolant at the end vs beginning of CraftNotes:** the action-beat-density finding from the polish audit suggests this is one of the heavier-handed craft notes; demoting it within the sequence might reduce its pressure.

Each of these is ~$1.34 and produces a clean A/B. The meta-methodology lesson: **once ordering is cheap to vary, many placement questions become first-class experiments instead of unchallengeable assumptions.**

## Open-thread hygiene

This report's outputs:

- The "invariants-first is NOT heavy-handed" finding is at N=6 — **not yet confirmed, worth more probes.** Open thread: run the harder-test probes named above within the next few days. Deferred, target 2026-04-26.
- The "invariants-first may be a better default" hypothesis is a genuinely interesting open question. Open thread: if the deferred follow-up confirms the pattern at N=12-15, propose making invariants-first the new production default. Deferred, target 2026-04-28.
- The within-section granularity is built but unexercised. Open thread: the three follow-up experiments named above (earned-register placement, truth-block placement, hands-as-coolant placement) — each worth its own small run. Deferred, no fixed targets; opportunistic.

## Dialogue with prior reports

The 1745 demotion-re-run report named the placement-dominates-tier hypothesis. This experiment tests it directly at the section-level granularity. Result: placement clearly matters, but not in the direction that "more attention = more heavy-handed" would predict. The mechanism is subtler — it's about whether the invariants frame OR interrupt the rest of the generation context.

The 1700 polish-audit retrospective said: *"Invariant-tier additions are expensive to A/B and therefore should ship only when the principle is bright enough to stand without empirical confirmation."* This experiment partially refutes that. With configurable ordering, invariant-level placement IS now A/B-able (even though the invariant BODIES aren't overridable). The cost of testing an invariant-related hypothesis dropped from "rebuild the binary twice" to "`--section-order ...`". That's a real change in methodology.

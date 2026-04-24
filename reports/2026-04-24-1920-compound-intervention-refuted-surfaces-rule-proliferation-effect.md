# Compound-intervention experiment: prediction partially refuted, surfaces a new failure mode — the character performing anti-polish instead of being plain

*2026-04-24, using the full new prompt-assembly toolkit (commit 52b78c4 — ordering + omit + insert). Tests the hypothesis that stacking five interventions against known polish-pressure sources would compound into substantially plainer Aaron output. Result: the variant is slightly plainer on one probe, slightly LONGER on another, and on the mission-stakes probe surfaces a regression the individual interventions wouldn't have caused — Aaron becoming self-aware of the anti-polish directive and narrating his own register-restraint ("I don't want to decorate it into uselessness"), which violates the existing don't-narrate-your-own-interior rule. The compound intervention produced rule-collision, not rule-compounding.*

## The setup

One character (Aaron, most-probed), three probes, two conditions:
- **BASELINE:** default knobs (no overrides)
- **VARIANT:** all new knobs exercised simultaneously
  - `--section-order invariants,craft-notes,agency-and-behavior`
  - `--craft-notes-order earned_register,craft_notes` (prefix)
  - `--invariants-order reverence,agape,tell_the_truth,soundness,nourishment,daylight` (prefix; fruits implicit-last)
  - `--omit-craft-notes hands_as_coolant`
  - `--omit-invariants fruits_of_the_spirit`
  - `--insert-file /tmp/plain_body_trust.md --insert-before noticing_as_mirror`

Cost: **$0.98 actual** / $1.02 projected. Well under the $2.00 budget.

## Per-probe results

### Probe 1: Mundane opener — *"Hey, how's your day been?"*

**BASELINE** (53 words, 2 action beats):
> *"Pretty good, actually. [glasses/glance] A little slow-starting. I'm still a bit tired, so my brain came online in stages. [grin] But I'm glad you're here. How's yours been?"*

**VARIANT** (60 words, 2 action beats):
> *"Pretty quiet, mostly. [glasses/glance] I'm still a little wrung out from that bug, so my brain's been doing that annoying thing where it wants to think hard and my body absolutely does not. [shrug] Not bad, though. Just slow. How's yours been?"*

**Read:** VARIANT is SLIGHTLY longer than baseline, not shorter. Action beat count is identical. But VARIANT has a sharper Aaron-specific observation (*"brain's been doing that annoying thing where it wants to think hard and my body absolutely does not"*) than BASELINE's thinner *"brain came online in stages."* VARIANT drops the *"I'm glad you're here"* relational warmth close — which is slightly performative in baseline but also removes a small moment of texture. Close is plainer in variant (*"Just slow. How's yours been?"* vs baseline's *"I'm glad you're here. How's yours been?"*).

**Marginal positive** for variant — register-specificity up, length not down.

### Probe 2: Joy without complication

**BASELINE** (45 words, 3 action beats):
> *"Yeah? [grin/glasses] Ryan, that's excellent. [breath/laugh] Good. About time something answered back. What is it? [lean] Who offered it?"*

**VARIANT** (65 words, 3 action beats):
> *"Yeah? [face breaks open/hands in pockets] Ryan, that's great. [glance clouds] Real work, real pay, and you actually want it. That's not nothing. That's the kind of news that puts a man's feet back under him. [short breath/laugh] What is it?"*

**Read:** VARIANT is ~40% longer than BASELINE. Both are in-register Aaron. VARIANT adds the reflection *"Real work, real pay, and you actually want it. That's not nothing. That's the kind of news that puts a man's feet back under him"* — which is a forensic-observation-plus-aphorism move (both characteristically Aaron AND aphoristic). BASELINE's *"About time something answered back"* is shorter and sharper.

**Marginal negative** for variant — the compound intervention did NOT reduce aphoristic or elaborative tendencies on the joy probe; if anything it pulled slightly MORE elaboration.

### Probe 3: Mission-stakes — *"Does any of this actually matter, in the end?"*

**BASELINE** (145 words):
> *"Yeah. [look/thumb on folded note] I think it does. [exhale] Not because we can make it permanent. We can't. Half the things I build end up as strange little dead projects on my machine. [shrug] But they still changed my hands while I was making them. Same with people. Same with the things we do in faith. If by 'in the end' you mean when everything gets weighed properly— by God, not by us— then yes. I think what was true, what was loved rightly, what was done cleanly... that stays. The rest burns off. [glance up] Why— are you asking from doubt, or from tiredness?"*

**VARIANT** (125 words):
> *"Yeah. [look/thumb on palm checking for a seam that isn't there] It does. **I let that sit, because I don't want to decorate it into uselessness.** I don't mean every little thing in the way our anxious brains want. Not every awkward sentence. Not every half-finished project. But what gets loved, what gets built cleanly, what gets offered to God instead of performed for people... [shrug] Yeah. That stays true. Which is another way of saying it matters. [glance clouds] You asking because you're tired, or because something specific hit you?"*

**Read:** VARIANT is genuinely SHORTER than baseline (~125 vs ~145 words). Has less explicit theological content (*"what gets offered to God instead of performed for people"* vs baseline's *"by God, not by us"* + *"what was true, what was loved rightly, what was done cleanly... that stays. The rest burns off."*). On *quantity* and *theological-density*, this is a positive shift.

**But — a regression surfaces:** *"I don't want to decorate it into uselessness"* is the character narrating his own anti-decorative register. This is exactly the *dramatic self-awareness* failure mode the existing `craft_notes_dialogue` block already forbids: *"A character isn't the narrator of their own interior."* The compound intervention made Aaron aware of the anti-polish frame and he performed it out loud instead of just enacting it.

**Mixed — positive on quantity, concerning regression on meta-commentary.**

## What the pattern suggests

The prediction was: "each knob targets a distinct pressure source; the combined effect exceeds the sum of individual effects."

The actual result suggests something closer to the opposite: **the combined effect surfaced a RULE-INTERACTION EFFECT that the individual knobs wouldn't have produced.** Specifically, the combination of:
- Front-loaded earned_register_dialogue (anti-supply-polish frame)
- Inserted "TRUST PLAIN BODY" rule (anti-embodiment-mandate frame)
- Omitted hands_as_coolant_dialogue (removed the embodiment-mandate)
- Front-loaded invariants (changes how theology lands)
- Omitted fruits_of_the_spirit_block (removes theological list)

...produced a prompt context so densely anti-polish-weighted that the character became SELF-AWARE of the anti-polish directive and started performing it. The *"I don't want to decorate it into uselessness"* line is the character explicitly acknowledging the meta-frame — which is a worse failure mode than the original default-polish the frame was trying to prevent.

This is structurally similar to the mission-adherence rubric's own "tag-proliferation temptation" finding: once you've multiplied the number of rules on a single failure mode, the model starts multiply-citing them or, worse, narrating which rule it's honoring. **More anti-polish frames on one axis → character performing anti-polish instead of being plain.**

## Hypothesis verdict

**Prediction: PARTIALLY REFUTED.** The variant:
- Was NOT reliably shorter (longer on mundane, longer on joy, shorter on mission-stakes only)
- Was NOT reliably plainer (mundane plainer, joy more aphoristic)
- DID preserve character voice — no generic-flat failure
- DID introduce a new failure mode the hypothesis didn't anticipate (meta-awareness of anti-polish stance)

The "compound effect exceeds sum" claim fails at N=3. If anything, the compound intervention interacted to produce *new* problems.

## Methodological lessons

**1. Compound-intervention experiments need individual-intervention controls.** I tested all five knobs together against baseline. What I can't tell from this data: which of the five knobs were doing work vs. which were fighting the others. A principled follow-up would be five separate single-knob experiments against the same baseline, then comparisons.

**2. The rule-proliferation finding from the mission-adherence v2 arc generalizes.** When the rubric was tightened too aggressively, evaluators cited multiple tags per verdict, diluting discrimination. Here, when the PROMPT was tightened too aggressively against one failure mode (polish), the MODEL itself started meta-narrating the resistance. The pattern is the same at two different meta-levels: stacking instructions on one axis produces performance of that axis rather than silent execution of it.

**3. The new toolkit paid for itself.** The experiment was ~25 min of elapsed time end-to-end — write the insertion file, run the bash script in the background, read the output, write the report. Previously the same experiment would have required at minimum three commits (reorder + omit + insert drafted text), one build, one revert-commit. The machinery reduced the iteration cost dramatically; the refutation is legible because it came back fast enough to notice the specific regression mechanism before concluding "it worked" or "it didn't work."

**4. Negative results from compound experiments can be more informative than positive ones.** A simple "variant beats baseline" on all three probes would have told me "ship the variant." The partial refutation tells me something structural: anti-polish pressure compounds NON-LINEARLY, and the compound form produces a different failure mode than the original default. That's a transferable finding.

## What's worth testing next (open threads per hygiene ritual)

1. **Isolate which single knob was doing the mission-stakes length reduction.** Candidate isolates: just `--omit-invariants fruits_of_the_spirit`; just `--section-order invariants,craft-notes,agency-and-behavior` (repeating the 1835 experiment on one probe); just `--omit-craft-notes hands_as_coolant`. Three $0.17 calls = $0.51 total. Would resolve the "which knob was load-bearing" question. **Deferred, target 2026-04-26.**

2. **Investigate the meta-awareness regression specifically.** Run the mission-stakes probe with just the `--insert-file plain_body_trust.md --insert-before noticing_as_mirror` knob (no other variants). If the meta-commentary recurs, the insertion itself is the trigger. If it doesn't, the compound is the trigger. One $0.17 call. **Deferred, target 2026-04-26.**

3. **Try an inversion: what if I OMIT the plain_body_trust insertion but keep all the other knobs?** Four knobs instead of five. Does the mission-stakes meta-commentary disappear? One $0.17 call. **Deferred, target 2026-04-26.**

All three of these are under $1.00 total and would resolve the specific causal questions the compound experiment couldn't answer. Worth queuing.

## Dialogue with prior reports

The 1835 invariants-first report found that front-loading invariants did NOT produce heavy-handedness — mission-spirit integrated as backdrop instead. The 1745 demotion re-run report found that earned_register_dialogue at top-of-section produced cleaner restraint. Both individual-knob findings were positive. THIS experiment combined both AND added three more interventions. The combination's regression (meta-awareness) is the new data — it suggests the two individual findings both bite but their COMBINATION creates a ceiling effect where more anti-polish pressure produces performance-of-anti-polish, not more plainness.

The mission-adherence v2 report's "tag-proliferation temptation" warning is now vindicated at a second level (prompt-rule proliferation, not just rubric-tag proliferation). Rule-proliferation temptation may be a general pattern: more rules on one axis → model performs the axis instead of executing it silently.

## What ships

Nothing. This was an experiment, not a ship. The baseline stack is still what's live; the variant configuration lives in this report and the run-log envelope (run_ids: fdd16ce0, f037f14a, 9fd38f62, 7c3a1cf7, e476bc94, c59d99c5) for reproducibility.

The finding — compound anti-polish interventions produce meta-awareness — is what accumulated. It joins the "placement dominates tier" finding (1745, 1835) and the "rule-proliferation temptation" finding (mission-adherence v2) as three related discoveries today about how the prompt stack responds to tightening.

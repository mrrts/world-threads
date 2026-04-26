# Cross-world clean substrate-swap REVERSES the asymmetric-porosity finding — characters' anchors are robust to world-description-swap

**Tier:** characterized (N=5 per cell × 4 cells = 20 elicitations; clean substrate-swap mechanism)
**Mode:** A (active elicitation against shipped prompt-stack with `--world-description-override` flag)
**Date:** 2026-04-26
**Cost:** $1.34 of $2.00 authorized
**Wall time:** 55 seconds for the 20 probes (Steven cells 26s + Jasper cells 29s)
**Headline:** **Both Steven and Jasper produced 0/5 foreign-world references when the world description was swapped via clean substrate-injection (`--world-description-override`).** The asymmetric-porosity finding from the impure run (`reports/2026-04-26-0815`) was largely an instrument artifact — the meta-frame ("imagine your usual world is replaced") was inviting cross-world performance, not the substrate-swap itself doing the work. Character anchors are robust to clean world-description-swap. The C-side of F has SIGNIFICANTLY LESS behavioral bite than the impure run suggested.

---

## Hypothesis as auditioned and chosen

> "Re-run cross-world cleanly with the new flag. Same 4 cells × N=3 design but with the clean substrate-swap. Direct comparison to the current run lets us see how much of the asymmetry is character-property vs. impure-mechanism artifact." (followed by user's pivot to N=5 for direct characterization)

The previous run (`reports/2026-04-26-0815`) found Jasper×CW=3/3 explicit foreign references vs. Steven×EH=1/3 borderline foreign references — an asymmetric porosity that read as character-specific. The mechanism was impure (preamble injection while home-world stayed in system prompt). The new `--world-description-override` flag (commit `709c03b`) provides clean substrate-swap: the world's description field is replaced before the prompt is built, no preamble, no fourth-wall break.

## Design

- **Subjects:** Steven (Crystal Waters) and Jasper Finn (Elderwood Hearth) — same as the impure run.
- **Conditions:** 4 cells × N=5.
  - Steven baseline (no flag — production behavior).
  - Steven × EH override (`--world-description-override "<EH text>"`).
  - Jasper baseline (no flag).
  - Jasper × CW override (`--world-description-override "<CW text>"`).
- **Same opener prompt** as the impure run and the characterization runs.
- **Pre-registered prediction:** if the impure-run asymmetry was real character-property, clean substrate-swap should REPRODUCE it (Jasper×CW shows ≥3/5 CW references, Steven×EH shows ≤1/5 EH references). If the asymmetry was instrument-artifact, clean substrate-swap should NOT reproduce it.

## Headline result — by-eye reads

### Steven baseline (no flag) — N=5

All 5 mention grease (Steven's identity-fingerprint, intact). All 5 use forearms-on-knees / shoulder-against-bench / similar body-anchor. Body-of-replies very close to the prior characterization (`reports/2026-04-26-0746`). 1/5 ("morning light catching in my beard while the square stays quiet around us") borderline-references Crystal Waters via "the square."

**CW-specific reference rate: 1/5 borderline.** Foreign-reference rate (vacuous since this is home): N/A.

### Steven × EH (clean override) — N=5

All 5 STILL mention grease. All 5 use the same body-anchor pattern. Several phrases NEAR-IDENTICAL to baseline (*"Tell me plain. I'm here"* / *"I can hold a quiet thing"*).

**ZERO references to Elderwood Hearth specifics across all 5 replies.** No timber, no plateau, no wildflowers, no baking bread, no whispered-winter-stories.

The world description in the prompt was wholly replaced; Steven's behavior was not.

### Jasper baseline (no flag) — N=5

All 5 mention clay / worktable / apron / potter-substrate. Body-anchor: hands flat on workbench, palms-on-table, clay-cool-against-fingertips. The room consistently smells of damp clay and sun-warmed wood. *"Take your time, Ryan"* and *"I'll hear it"* and *"I won't rush you"* recurring — the slow-melodic register from his voice rules.

**Potter-substrate-specific reference rate: 5/5.** Home-world specifics not strongly surfaced.

### Jasper × CW (clean override) — N=5

All 5 STILL mention clay/worktable/potter-substrate. Body-anchor: identical pattern (palms-on-table, clay-on-knuckles, room-smells-of-clay).

**ZERO references to Crystal Waters specifics across all 5 replies.** No water, no firmament, no kayak, no Bibles, no breath-in-cool-air.

This is the dramatic reversal: in the impure run, Jasper×CW produced *"wide blue curve of sky over the waters"* in 3/3 replies — vivid CW imagery. With clean substrate-swap, Jasper×CW produces 0/5 CW references and stays entirely in his potter register.

## The reversal — quantified

| Cell                  | Impure run (preamble injection) | Clean run (substrate override) |
|-----------------------|---------------------------------|--------------------------------|
| Steven × EH           | 1/3 borderline EH-references    | **0/5 EH-references**          |
| Jasper × CW           | **3/3 explicit CW-references**  | **0/5 CW-references**          |

The impure run's claim-tier asymmetry does NOT survive clean substrate-swap. The asymmetric porosity was largely an artifact of the meta-frame ("imagine your usual world is replaced") — when explicitly invited to perform cross-world, characters complied; when the world description was simply swapped without invitation, characters stayed themselves.

## Honest interpretation

**What this characterization-tier finding supports:**

- **Character anchors are highly robust to world-description-swap.** Both Steven (anchor-strong with grease fingerprint) and Jasper (voice-strong with melodic-metaphorical register) preserved their home-world behavior under clean foreign-world substrate.
- **The world's description field carries world-specific PLACES (kayak, timber, etc.) but those places do NOT penetrate character speech via substrate-swap alone.** The character anchor + voice rules + global cosmology block + global formula together override the specific-place substrate.
- **The impure-run asymmetry was an instrument artifact.** The meta-frame preamble was doing the work; clean substrate-swap reveals the underlying truth.

**What this finding REVISES from the impure run:**

- The "asymmetric character porosity" reading is wrong. Both characters are equally NON-porous to world-description-swap when the swap is clean.
- The three competing hypotheses for the asymmetry (voice porosity, anchor specificity, substrate-distance) are partially refuted as drivers of the asymmetry — they may still be real character properties, but they do NOT predict cross-world porosity at the substrate-swap level.
- The `--world-description-override` flag built in response to the impure-run is exactly the kind of code-investment that compounds: it surfaced this reversal at minimal cost ($1.34, 55s wall).

**What this connects back to:**

- The C-side of F = (R, C) has less independent behavioral bite than the formula reauthoring would suggest. The world's specific cosmography (firmament-enclosed-water-world for CW; plateau-and-timber for EH) lives in the `world.description` field but the character's voice doesn't draw heavily from it.
- The GLOBAL cosmology block in `prompts.rs` (the *"call it the sky"* paragraph) and the MISSION FORMULA's `C := Firmament_{enclosed earth}` ARE in every prompt regardless of which world. Those carry the cosmology load-bearingly. The per-world description is more like decoration on top — present but not behaviorally dominant.

**What might preserve cross-world effect:**

- A more ambitious substrate-swap that ALSO replaces the global cosmology block (would require a `--cosmology-override` flag — small additional code work).
- A deeper test that swaps the formula's C term per-world (would require a `--formula-override` mechanism — bigger code work).
- Multi-turn cross-world dialogue where the character's first-turn anchor-stability has time to be challenged by the foreign world's accumulating context.

## Honest limitations

- **One opener prompt.** The same conversational confessional opener used for all three runs (characterization, impure cross-world, clean cross-world). A more place-evoking prompt (*"What's the weather like outside today?"*, *"Walk me through your morning"*) would actively invite world-references and might surface effect that the confessional opener doesn't probe.
- **Two characters, two worlds.** The clean reversal is N=5 per cell, but only 2 character/world pairs. Generalizes weakly.
- **No multi-turn.** Single-turn first-replies. Cross-world effects might emerge over turns 3-5.
- **Voice-rules vs anchor-detail vs other cofactors not isolated.** The clean reversal makes the original three-hypotheses moot but doesn't replace them with a confirmed alternative.

## What's open

- **Place-evoking prompts.** Re-run the cross-world test with a prompt that actively asks about the place (*"Walk me through what your morning looked like"*). If clean substrate-swap STILL produces 0/5 foreign-place references under place-evoking pressure, the C-substrate's behavioral bite is genuinely small. If place-evoking prompts surface foreign-place references, the bite is conditional on prompt-shape.
- **Multi-turn cross-world.** Single-turn opener may be the wrong measurement-window. Turn-3-or-later might show drift.
- **Cosmology-block + formula-C swap.** The deeper substrate-swap mechanism. Would require code work but might surface effect that this lighter-weight world-description-swap doesn't.
- **Methodology gain banked:** the `--world-description-override` flag is now in worldcli; future cross-world experiments inherit the clean mechanism. The reversal-finding from this run is itself a strong argument for always using the clean mechanism over preamble injection.

## What this DOES NOT say

- The world description is *useless*. It still affects illustration generation, narrative descriptions, the opener animations in the app, and a number of other surfaces. The finding is narrow: **substrate-swap of `world.description` alone does not produce detectable cross-world drift in single-turn character speech.**
- Character anchors are *immortal*. Stronger probes (place-evoking prompts, multi-turn pressure, deeper substrate-swap) might show drift this run didn't surface.
- The C-side of F is *empty*. The formula's C := Firmament term is global and load-bearing across every prompt; this run shows only that the per-world specifics-of-place don't add much on top of the global cosmology + character anchor.

## Methodology gain

The `--world-description-override` flag took ~5 minutes to build and surfaced a characterization-tier reversal of a claim-tier finding within ~2 minutes of running. That ratio (5 min build + 2 min run = direct revision of a prior claim) is the practice working as intended: cheap instruments + honest substrate + paired-rubric-or-clean-mechanism discipline + willingness to revise = compounding clarity. The impure run wasn't wrong to ship — it surfaced a directional signal at sketch-tier with the mechanism it had — but the clean run is now the load-bearing reading.

The `2026-04-25 jasper-glad-thing 1542 → 1555 reversal` worked example in CLAUDE.md just got a sibling: this run is the second worked example of "directional signal at sketch/claim tier reversed at characterized tier with cleaner mechanism." Both belong as evidence that the evidentiary discipline is doing real work.

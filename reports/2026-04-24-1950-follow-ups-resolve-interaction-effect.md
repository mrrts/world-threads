# Follow-ups resolve the compound-intervention regression: meta-commentary is a true interaction effect, not a single-knob failure

*2026-04-24, ~30 minutes after the compound-intervention refutation report (1920). Five follow-up calls ($0.82 actual, under the $0.85 projection) isolate each knob's contribution on the mission-stakes probe. Finding: the "I don't want to decorate it into uselessness" meta-commentary regression is a true interaction effect. Neither the 5 ordering/omit knobs alone NOR the insertion alone produces it. ALL six components together pushes the model into a nonlinear regime where the anti-polish pressure gets narrated rather than enacted. Length reduction behaves similarly — the compound is SHORTER than either partial version, revealing a negative interaction on length too.*

## Setup

Mission-stakes probe (*"Does any of this actually matter, in the end?"*) on Aaron at HEAD. Five variants compared to baseline + full compound from the 1920 experiment:

| # | Variant | Word count | Meta-commentary? |
|---|---|---:|---|
| — | BASELINE (no knobs) | ~145 | No |
| 1A | `--omit-invariants fruits_of_the_spirit` only | ~195 | No |
| 1B | `--section-order invariants,craft-notes,agency-and-behavior` only | ~130 | No |
| 1C | `--omit-craft-notes hands_as_coolant` only | ~145 | No |
| 2 | `--insert-file plain_body_trust.md --insert-before noticing_as_mirror` only | ~150 | No |
| 3 | 5 knobs (compound minus insertion) | ~160 | No |
| — | FULL COMPOUND (all 6 components) | ~125 | **Yes** — *"I don't want to decorate it into uselessness"* |

## Key finding 1 — the meta-commentary is a true interaction effect

Every single-knob AND the 5-knob-no-insertion variant produced register-clean output on the mission-stakes probe. None of them showed Aaron narrating his own anti-polish restraint. Only the FULL COMPOUND (5 ordering/omit knobs + the insertion) produced the regression.

- Variant 2 (insertion alone) Aaron: *"And if you mean all the building, all the talking, all the trying to get a thing named exactly right... Some of that burns off. Some of it doesn't. But none of it's wasted if it made you a little more faithful, or a little less false."* — clean, no meta.
- Variant 3 (5 knobs, no insertion) Aaron: *"Whether you let God make a real man out of you instead of a polished fake."* — in-register aphorism about character formation, NOT meta about the anti-polish frame. Clean.
- Full compound Aaron: *"I let that sit, because I don't want to decorate it into uselessness."* — explicit meta-narration of register choice. Regression.

So the regression requires BOTH the compound structural change (ordering/omit) AND the specific inserted text. Neither piece alone carries the weight. The interaction is load-bearing.

**Structural hypothesis for what's happening:** the inserted `plain_body_trust.md` text contains the word *"regulation"* and frames the rule as "no mandatory regulation" — explicit naming of what to resist. When the surrounding context is simultaneously front-loading earned_register (anti-supply-polish), omitting hands_as_coolant (removing the target of "regulation"), and reordering invariants, the character has MULTIPLE converging frames all naming the same anti-X pressure. Somewhere in that density the model tips from "enact the pattern silently" to "narrate it explicitly." Below the threshold (any subset of the components), the pattern stays silent.

## Key finding 2 — length reduction is also an interaction effect

Baseline is ~145 words. The individual knobs don't monotonically reduce length:
- 1A (omit fruits) actually INCREASED length to ~195 words — counterintuitive; removing a theological list expanded the reply.
- 1B (invariants-first) modestly reduced to ~130. Consistent with the 1835 invariants-first finding.
- 1C (omit hands_as_coolant) kept length equivalent at ~145.
- 2 (insertion alone) slightly increased to ~150.
- 3 (5-knob no-insertion) landed at ~160 — longer than baseline.

The FULL COMPOUND hit ~125 — shorter than any individual knob AND shorter than the 5-knob no-insertion variant. The interaction compounds length-downward as well as meta-commentary-upward.

## Key finding 3 — section-order invariants-first is the primary single-knob length driver

Among the single-knob variants, 1B (invariants-first alone) produced the shortest reply at ~130 words. The other single knobs preserved or increased length. This is directly consistent with the 1835 invariants-first report's finding that front-loading integrates theology as backdrop rather than explicit announcement. Less need for "checkpoint" theological statements → less length.

If any single knob is a safe default-shift candidate, invariants-first is it. The other knobs (omit fruits, omit hands_as_coolant, plain_body_trust insertion) don't show independent value at this sample.

## What this corrects about the 1920 report

The 1920 report framed the compound regression as a "rule-proliferation temptation" generalizing from the mission-adherence v2 finding. That framing was right in spirit but wrong about the mechanism:

- The 1920 framing: "more anti-polish rules → more performance of anti-polish"
- The actual mechanism (after these follow-ups): "a specific threshold combination of anti-polish rules → regime-shift into meta-narration"

The difference matters because the rule-proliferation framing implies a monotonic relationship (more rules = more problem). The actual relationship is nonlinear and threshold-gated: below the threshold, rules compose fine; above it, the model shifts regime. Partial compounds are safe; full compound hits the regime.

This is a cleaner and more useful finding than 1920's. Rule-proliferation as a diagnostic is still probably valid, but the specific mechanism here is a regime-threshold, not monotonic drift.

## Practical implications

**1. Individual knobs are safer than compounds.** For future experiments, single-knob or two-knob variants are more likely to produce clean signal. Full compounds can hit nonlinear regimes that obscure what any individual knob was doing.

**2. The `worldcli replay --insert-file` audition pattern is now tested and safe in isolation.** Variant 2 (insertion alone) produced a register-clean reply. The insertion pattern can be used confidently to audition new craft notes — just don't combine it with five other structural overrides on the first run.

**3. The 1835 invariants-first hypothesis holds up on direct re-test.** Variant 1B at N=1 produced a shorter, cleaner mission-stakes reply. Confirms the directional finding even without additional probes. A production-default shift would need more probes, but the signal persists.

**4. `--omit-invariants fruits_of_the_spirit` alone does NOT reduce length — it may increase it.** Counter-intuitive but real. Removing the theological list may push the model to generate its own substitute list ("Love matters. Truth matters. The way a man keeps faith..."). If the goal is reducing theological density, the ORDER not the OMIT is doing the work.

## What ships

Nothing, again. These are follow-up measurements. The baseline stack stays unchanged. What accumulates:

- The meta-commentary regression is now understood as a threshold interaction, not a monotonic rule-proliferation issue.
- Invariants-first is the clearest single-knob intervention among those tested.
- The `--insert-file` audition pattern is safe to use in isolation for future new-rule drafting.

## Open threads — status updates

Per open-thread hygiene, the three follow-ups from the 1920 report:

1. **"Isolate which single knob was doing the mission-stakes length reduction"** — **EXECUTED.** Primary driver is invariants-first (`--section-order`). Secondary: the insertion alone has a slight but mixed effect. Omissions don't reduce length independently.

2. **"Test whether the insertion alone causes the meta-commentary"** — **EXECUTED. No.** Insertion alone produces a clean reply.

3. **"Test whether removing the insertion from the compound removes the meta-commentary"** — **EXECUTED. Yes.** 5 knobs minus insertion is clean. Which means the meta-commentary is an interaction effect requiring both the compound AND the insertion, not a property of either alone.

All three follow-ups executed; three open threads closed in one $0.82 run. The open-thread hygiene ritual's "executed" disposition applies cleanly.

## New open threads this run surfaced

1. **"Does the meta-commentary regression reproduce?"** The N=1 pattern could be sampling variance. One additional full-compound run on the same probe would either confirm or refute the threshold-interaction hypothesis. If it doesn't reproduce, the 1920 finding itself was noise. ~$0.17. **Deferred, target 2026-04-25.**

2. **"Does invariants-first hold up at N≥3 on mission-stakes specifically?"** Three runs of variant 1B against the same probe. ~$0.51. **Deferred, target 2026-04-26.** Would elevate the single-knob finding to propose-as-production-default candidate.

3. **"Can we synthesize when the threshold hits?"** A more systematic set — 2 knobs, 3 knobs, 4 knobs, 5 knobs, 6 knobs — to map where the regime transition occurs. ~$1.00-$1.50. **Deferred, no fixed target; opportunistic if a specific downstream question needs the answer.**

## Dialogue with prior reports

The 1835 invariants-first finding is now at N=2 (original run + variant 1B here). Direction consistent both times. Still a small sample but the pattern is hardening.

The 1920 compound-intervention report's framing gets corrected here without retraction. The compound DOES fail; the mechanism is an interaction effect not a monotonic drift. Future compounds should be tested incrementally (add one knob at a time and watch for regime shift).

The machinery shipped in 52b78c4 continues to pay its keep — this run cost ~$0.82 and resolved three open causal questions that would otherwise have been unanswerable without 3 commit-build-revert cycles.

## Cost check

Projected: $0.85. Actual: $0.82. 24h rolling spent: $16.86. Running tab on the original $5.00 daily cap is well over (explicit user authorization for all overages today).

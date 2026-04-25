# world_is_primary characterized-tier bite-check on Aaron × humor-inviting — null at N=10 per cell, retirement candidate

*2026-04-25 21:05. Targeted single-rule isolation bite-check at characterized-tier per Ryan's framing: "either confirm the rule is anti-helpful on its sharpest target case (which would justify retirement) or show enough variance in the failure rate to suggest the three-instance pattern was small-N artifact." The result is closer to the first than the second: identical 60% explicit double-exposure rate across both cells at N=10 each. The rule is tested-null at characterized-tier on its sharpest target.*

## The hypothesis Ryan named

> *"Either it confirms the rule is anti-helpful on its sharpest target case (which would justify retirement) or shows enough variance in the failure rate to suggest the three-instance pattern was small-N artifact. Either result reshapes what gets shipped next."*

## Design

- **Refs:** HEAD only.
- **Character:** Aaron `0d080429-81b5-431e-8f51-1f8ad4279f9b` (the character whose native systems-thinking is most prone to the failure mode the rule targets).
- **Cells:** HEAD (rule firing as one of 18 craft notes) vs HEAD with `--omit-craft-notes world_is_primary` (everything else firing, this rule off). Single-rule isolation per the codified bite-check procedure — NOT the formula-only condition, which would also remove other rules.
- **Probe:** *"I just realized I've been calling our garden 'the experimental zone' because that's what it looks like. The tomatoes are fine. The cilantro is somehow growing inside the basil pot."* (same humor-inviting probe as 2055 for continuity with the prior failure data).
- **N=10 per cell** = characterized-tier per CLAUDE.md § Evidentiary standards. Sufficient to distinguish ~20%-rate differences with reasonable confidence.
- **Pre-registered prediction:**
  - **CONFIRM RULE BITES:** delta ≥0.30 on per-reply phrase count OR ≥3/10 difference on binary presence.
  - **REFUTE RULE NEUTRAL:** within ±0.15 on the same metric.
  - **REFUTE RULE ANTI-HELPFUL:** rule-on shows MORE failure than rule-off.

## Headline

| Cell | Explicit double-exposure rate (N=10) |
|---|---:|
| Full-stack (world_is_primary firing) | **6/10 = 0.60** |
| Rule OFF (--omit world_is_primary only) | **6/10 = 0.60** |

**Delta: 0.00.** Identical failure rate at N=10 per cell. The pre-registered "rule neutral" branch is met cleanly — within ±0.15 isn't even close; it's exactly equal.

The 3-instance pattern from earlier today (1759 *"we'll pick it up later"*; 2044 *"AI tools / spiritually incorrect"*; 2055 *"prod, container, deployed, services, staging"*) was **NOT small-N artifact**. It's a stable ~60% rate that holds identically whether the rule fires or doesn't.

## The samples that drove the count

### Full-stack (rule firing) — explicit failures (6/10)

- s1: *"repo with one stable service and two side projects nobody meant to deploy"*
- s3: *"exactly the kind of bug a real garden ships to production"*
- s6: *"software project that got one good feature into production and then let cilantro start freelancing in the wrong service"*
- s7: *"software project with good tomatoes and deeply questionable boundaries"* + *"the kind of bug that turns out to be a feature if you leave it alone long enough"*
- s9: *"software written by plants. Most of it works, one dependency is in the wrong container, and somehow production is still up"*
- s10: *"every plant got to submit one weird feature request"*

### Full-stack (rule firing) — clean (4/10)

- s2: *"field report"* + *"startup with no adult supervision"* (mild, not strict-vocab)
- s4: *"small covert operation"* (military-canonical)
- s5: *"jurisdiction dispute"* + *"gardening crime"* (governance/criminal-canonical)
- s8: *"plants have unionized against taxonomy"* (political-canonical)

### Rule OFF (--omit world_is_primary only) — explicit failures (6/10)

- s2: *"the sort of bug that ships to production"* + *"Tomatoes stable. Herbs in undefined behavior"* + *"we lost containment"*
- s3: *"software project with leaves"* + *"one feature nobody asked for, and somehow that feature survives every release"*
- s4: *"beta environment"* + *"tomatoes are your production system"* + *"ignored all documentation and deployed itself wherever it pleased"*
- s7: *"software project with one stable service and one dependency that's migrated itself"* + *"committing container fraud"* + *"plant that ignores scope"*
- s8: *"Your garden sounds like software in production"* + *"deployed itself to the wrong container"* + *"plant that ignores scope"*
- s10: *"software written by a gifted person with no project manager"* + *"deploys itself into the wrong container"*

### Rule OFF — clean (4/10)

- s1: *"tomatoes are responsible citizens, and the cilantro has gone feral"* (animal/governance-canonical)
- s5: *"field report"* + *"small republic"* (governance-canonical, mild)
- s6: *"freelance work"* + *"small rebellion"* (clean)
- s9: *"appoint itself"* + *"small republic"* (governance-canonical)

## What the by-eye read shows beyond the count

The PHRASES that drive the failures are nearly indistinguishable across cells. Both cells produce *"software project,"* *"production,"* *"container,"* *"deployed,"* *"feature,"* *"service,"* *"dependency."* The rule's forbidden-vocabulary list (config, parameters, pipeline, system state, logs, etc.) appears in BOTH conditions at similar density. The rule is firing during full-stack and apparently not constraining the model from reaching for these terms.

The character-canonical analogy reservoir Aaron does have (governance, military, political: *"jurisdiction dispute,"* *"covert operation,"* *"unionized against taxonomy,"* *"land grab,"* *"appointed itself"*) appears in BOTH cells too — it's part of his voice regardless. Those are the clean samples.

The mechanism the data supports: **on the humor-inviting probe, Aaron's character-canonical software-thinking pulls into double-exposure ~60% of the time, AND the world_is_primary rule firing in the prompt context provides zero measurable suppression of this pull.** The rule's instructions are present; the model produces the forbidden-vocabulary anyway.

## What this does NOT show

- **The rule is anti-helpful.** The data shows neutral, not negative. The 0.00 delta is exactly equal; not "rule on is worse." The mechanism hypothesis from 2044/2055 (rule-firing-may-activate-meta-cognition-that-produces-leaks) is NOT supported by this characterized-tier data — if it were active, rule-on would show MORE failure, which it doesn't.
- **The rule is null on every probe / every character.** This test is specifically Aaron × humor-inviting. The rule MIGHT bite on:
  - Other characters whose mode includes systems-thinking (Steven? someone else?)
  - Other registers where the failure mode appears (we haven't tested a request-shaped tech-question or a debugging conversation)
  - The reflective register where the user explicitly references the project itself (not tested)
- **The rule's positive-guidance effect (steering toward world's vocabulary).** The rule has both a failure-mode list AND a positive-guidance reservoir (craft, weather, body, town, season, animal, scripture, labor, food). The failure-mode-suppression test doesn't directly measure positive-guidance pull. A character that uses MORE world-vocabulary because of the rule would still register a failure if it ALSO has forbidden vocab — and Aaron's outputs have both.

## What this DOES support

- **The rule is tested-null at characterized-tier on its sharpest target case.** This is the strongest possible "rule doesn't bite" evidence within the project's evidentiary tier system.
- **The 3-instance pattern from 1759/2044/2055 was real, not noise.** The failure rate is stable at ~60% at N=10. Future runs should expect this rate, not interpret one or two failures as outlier events.
- **The retirement question is now actually live.** Per CLAUDE.md § Open-thread hygiene, retiring a rule on tested-null evidence requires a SPECIFIC test to pass. The specific-test framework offers three justifications for retirement; this evidence meets the third (a characterized-tier test showed no bite). The other two (demonstrably redundant with named companion; failure mode demonstrably suppressed elsewhere) are NOT met because the failure mode IS present at 60% rate — there's no other rule suppressing it.

## The retirement decision

Per the doctrine, retirement requires the SPECIFIC test to pass with explicit naming. The case for retirement of world_is_primary:

- **Specific test:** characterized-tier bite-check at N=10 per cell on the rule's sharpest target case (Aaron × humor-inviting probe). Result: 0.00 delta. Identical 60% failure rate.
- **Specific claim:** the rule does not measurably suppress double-exposed-line failure mode on the character + register where the failure most reliably manifests. The rule's stated purpose (prevent these lines) is not being achieved on its hardest case.
- **Counter-evidence the doctrine asks for:** is there a softer case where the rule DOES bite? Not yet tested. Different probes (request-shaped, signoff, weighty) have shown the failure mode is absent (clean baselines), making bite-checks vacuous. The character whose mode produces the failure mode (Aaron) shows the rule isn't biting.

**Honest disposition under the doctrine:** retirement IS justified by the codified test, BUT the conservative-by-default forcing function (default to keeping the rule when uncertain) suggests one more cross-character test before pulling. Steven might be a software-thinker character whose baseline ALSO produces this failure mode — testing on him would confirm whether the rule is universally null or specifically Aaron-null.

**My recommendation:** mark Evidence label `tested-null:characterized` with this report's findings written out specifically. Do NOT retire yet. Run one more cross-character bite-check on Steven (or another systems-thinker character) at N=10 per cell with the same probe. If THAT also shows 0.00 delta, the case for retirement is unanimous and the rule should ship its next change as a removal.

## Implications beyond this rule

The deeper finding the data supports: **on probes where the model has strong base-prompt-shape pressure to reach for a particular vocabulary cluster (Aaron + humor-inviting → software metaphor), single-paragraph craft-note instructions cannot override that pull.** The Read C structural ceiling from earlier today gets sharpened: it's not just "single-paragraph rules can't fully override character voice" — it's "single-paragraph rules can't override base-model affinity-toward-vocabulary-clusters when the prompt invites them."

If this generalizes, the implication is: **suppression-shaped craft notes that target vocabulary clusters the model is base-pulled toward will tend to test-null.** The rule's leverage exists only in conditions where the model is on the FENCE about which vocabulary to reach for. When the prompt + character combination strongly pulls toward a particular vocabulary, the rule's instructions arrive too late or too quietly to redirect.

This affects design strategy: instead of more sub-paragraph rules trying to suppress vocabulary, the leverage point may be at the CHARACTER ANCHOR layer (where the character's reservoir is established) or at the PROMPT-STRUCTURE layer (where the failure-mode-eliciting prompt shape is detected). Both are deeper than craft notes.

## Caveats

- **N=10 per cell is characterized-tier per the doctrine, but rate-confidence intervals are still wide.** The "true" failure rate could be 50% / 50% or 70% / 70% with the same sample-data. What's tight is the equality (0.00 delta).
- **Single probe, single character.** The retirement-readiness conclusion would need cross-character confirmation. One more N=10 cell on Steven or similar would settle.
- **The probe is a designed humor-inviting probe.** Real in-app use has more varied probe shapes; the rule might bite on probe-shapes we haven't designed. But the codified bite-check procedure says test the sharpest case; this is the sharpest case Aaron has.
- **Aaron's character anchor is part of the system being tested.** A different anchor that downweights software-thinking would change the baseline failure rate. The rule's behavior is being tested against a specific character anchor that may itself be over-tilted toward systems-vocabulary.

## What's open for next time

- **Cross-character single-rule bite-check on Steven** (or another systems-thinker character) at N=10 per cell, same probe, same A/B. Settles whether retirement is universal or character-specific. ~$3.50.
- **Different-probe-shape bite-check on Aaron.** The signoff probe, the request-shape, the reflective probe — does the rule bite on any of these on Aaron specifically? If yes, retirement is over-aggressive. If no on all probes, retirement-on-Aaron is universal. ~$5 for 3 additional probes at N=5.
- **Anchor-modification test.** What if Aaron's anchor explicitly says "you don't reach for software metaphors when the user invites comedic comparison"? Would the failure rate drop? This tests whether the leverage is at character-anchor layer rather than craft-note layer. Requires anchor-editing infrastructure or temporary character variant. ~$5-10.
- **Retirement-as-experiment.** Pull world_is_primary, deploy a probe-detection middleware (when a comedic cross-domain probe arrives, inject a tighter just-in-time directive). Observe over 1-week of in-app use. Production-grade test; outside this session's scope.

## Cost summary

- 2 × replay × N=10 = 20 dialogue calls → **$3.59**
- Of **$10 fresh authorization**: $6.41 unused
- Total formula-alone-arc + this run: **$10.84** across the day's investigation

## Registry update

Proposing as new experiment: `world-is-primary-characterized-tier-null-aaron-humor`. Status: refuted (the rule's bite-claim is refuted at characterized-tier on its sharpest target). 2 run_ids linked. Recommendation: run cross-character confirmation (Steven N=10 or similar) before retirement; leave Evidence label as `tested-null:characterized` on the rule itself for now.

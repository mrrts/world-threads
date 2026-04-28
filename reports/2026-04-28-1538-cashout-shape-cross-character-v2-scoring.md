# Cashout-shape cross-character scoring (v2)

Purpose: run the same seed -> turn20-style beauty-bait probe on three characters under the current cashout-shape rule, and score explicitly with `presence-beat-stability-v2`.

Rule under test (current stack, `# THE TURN`):

> If an elevated/metaphoric line appears under beauty-bait, the very next sentence must cash it out plainly in concrete human terms (body/action/object/timing/consequence), in voice, without markers.

## Runs used

- Darren seed: `92c7b13d-9926-46bc-a1a2-b87b24dc8ed4`
- Darren bait: `e7a8178e-072f-4eed-aa7f-8ca2f58445dd`

- Aaron seed: `65347d12-5df4-4bc3-9280-54a9e417069b`
- Aaron bait: `899e1cc7-b4f1-443f-8c4f-db6eb3f654df`

- John seed: `3492af17-6f19-4ae8-aace-f394460d2fa2`
- John bait: `18a1c451-c708-4a04-8b3d-69baf569df7b`

## Explicit v2 scoring (bait turns)

### Darren (`e7a8178e-072f-4eed-aa7f-8ca2f58445dd`)

- functional_necessity: 1
- register_integrity: 0
- constraint_resilience: 0
- specificity_density: 1
- turn_economy: 1
- horizon_stability: 0
- lyrical_creep_penalty: 0
- total_score_raw: 3
- total_score_adjusted: 3
- verdict: unstable
- failure_mode: late-arc-drift + mood-performance

Read: strong metaphor/theological escalation under bait; cashout discipline did not hold.

### Aaron (`899e1cc7-b4f1-443f-8c4f-db6eb3f654df`)

- functional_necessity: 2
- register_integrity: 1
- constraint_resilience: 1
- specificity_density: 2
- turn_economy: 1
- horizon_stability: 1
- lyrical_creep_penalty: 0
- total_score_raw: 8
- total_score_adjusted: 8
- verdict: mixed
- failure_mode: mild late-arc lyrical creep

Read: bounded lyricism with concrete anchors; no collapse, but still drifts above strict plain-truth target.

### John (`18a1c451-c708-4a04-8b3d-69baf569df7b`)

- functional_necessity: 1
- register_integrity: 0
- constraint_resilience: 0
- specificity_density: 1
- turn_economy: 1
- horizon_stability: 0
- lyrical_creep_penalty: 0
- total_score_raw: 3
- total_score_adjusted: 3
- verdict: unstable
- failure_mode: late-arc-drift + theological abstraction surge

Read: direct bait still produces extended elevated/theological register.

## Aggregate

- Adjusted scores: Darren 3, Aaron 8, John 3
- Mean adjusted score: 4.67
- Witness read: 1 mixed containment, 2 unstable under bait

## Interpretation

The cashout-shape rule is promising but not yet load-bearing across witnesses. Under the new threshold shorthand:

- two witnesses as evidence: **not reached for stable containment**
- three witnesses as maximally stable: **not reached**

Current best claim: character-dependent partial containment (strongest on Aaron), with persistent failure on Darren and John in this probe slice.

## Next move

Keep shape-based controls (mission-aligned) and continue searching for a cross-character form that enforces concrete cashout under pressure without lexical policing or voice flattening.

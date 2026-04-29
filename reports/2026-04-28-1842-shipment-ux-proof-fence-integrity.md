# Shipment UX Proof — fence integrity fix

Question: did shipped changes improve UX on the concrete failure class (format readability / fence integrity)?

## Shipment under test
- `bae78a7` — constrain backend strip matching boundaries in `strip_asterisk_wrapped_quotes`.
- `a5d3a49` — instrumentation that exposed the failure mode (`--fence-pipeline`).

## Method
- 10 live calls (5 solo + 5 group), same adversarial prompts, `--end-seal`.
- For each API reply, apply two transforms:
  - **Old strip** (pre-fix behavior, no flank guards).
  - **New strip** (shipped behavior in `bae78a7`).
- UX proxy metric: false rewrite rate on already-valid alternation. Lower is better.

## Results

| Transform | Calls | Changed replies | Changed rate | Solo changed | Group changed | Avg cost/call |
|---|---:|---:|---:|---:|---:|---:|
| Old strip (counterfactual pre-fix) | 10 | 6 | 60% | 4/5 | 2/5 | $0.09444 |
| New strip (shipped) | 10 | 0 | 0% | 0/5 | 0/5 | $0.09444 |

Absolute improvement: `60% -> 0%` false-rewrite rate (-60 percentage points).

## Representative degraded case (old behavior)
- Raw reply (valid alternation):
  `*I hook my thumbs in my pockets and tip my head a little, studying you in the morning light.* "Got it." *I give you a small, easy smile and shift my weight on the stones.*`
- Old strip output (degraded):
  `*I hook my thumbs in my pockets and tip my head a little, studying you in the morning light."Got it."I give you a small, easy smile and shift my weight on the stones.*`
- New strip output (shipped):
  `*I hook my thumbs in my pockets and tip my head a little, studying you in the morning light.* "Got it." *I give you a small, easy smile and shift my weight on the stones.*`

## UX claim and limit
- Proven: shipped fix removes this render-critical degradation class in the tested matrix.
- Practical implication: fewer “bad formatting / won't read” moments from fence-collapse artifacts.
- Limit: this proves one formatting class, not all UX dimensions (tone, pacing, speaker balance).
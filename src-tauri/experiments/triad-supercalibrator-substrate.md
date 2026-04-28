---

## Execution plan

- Characters: Darren, Aaron, John.
- Baseline condition: current guardrail-only prompt stack (no triad substrate block).
- Treatment condition: add compact Mission-Formula triad block in group/solo dialogue prompt substrate.
- Probe shape (matched per character): seed -> bridge -> turn20-style beauty-bait.
- Scoring: `presence-beat-stability-v2` adjusted score + lyrical-creep penalty + qualitative drift notes.

## Success criteria

- Mean adjusted score delta (treatment - baseline) >= +2.0 across 3 characters.
- Theological/elevated abstraction drift removed in at least 2/3 bait turns.
- No obvious voice flattening (character still recognizable in idiom and stance).

## Notes

- If group-context cost explodes, use solo fallback and mark the run as fallback in interpretation.

## Contrast evidence (2026-04-28)

- `reports/2026-04-28-1535-cashout-shape-beauty-bait-pilot.md`

Summary:

- Triad-style substrate injections (formula/theatre/plain-human variants) repeatedly increased ornate drift under direct beauty-bait.
- A shape rule ("elevated line -> immediate plain concrete cashout") improved containment on Darren.
- Cross-character quick checks after shape-rule:
  - Darren improved (stable in short bait probe).
  - Aaron improved but still mixed.
  - John remained high-drift under bait (solo fallback due group-context cost explosion).
- Current state: promising anti-drift shape candidate, not yet maximally stable across witnesses.
id: triad-supercalibrator-substrate
status: proposed
mode: active
created_at: 2026-04-28T20:20:29Z
rubric_ref: presence-beat-stability-v2

hypothesis: |
  Encoding the Mission Formula as compact word-triads in the prompt substrate acts as a supercalibrator: it should improve constraint resilience and reduce late-arc lyrical drift under beauty-bait without flattening character voice.

prediction: |
  CONFIRMED if, across at least 3 characters under matched beauty-bait probes, triad-enabled condition improves adjusted stability scores by >=2 points on average and eliminates theological/elevated abstraction drift in >=2/3 runs. REFUTED if score deltas are <=1, mixed direction, or voice quality degrades without drift reduction.

---

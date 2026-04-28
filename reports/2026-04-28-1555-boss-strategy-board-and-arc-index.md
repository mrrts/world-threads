# Boss strategy board + arc index

This is the tactical board for the current boss fight:

> **Quest:** contain John under turn20-style beauty-bait without lexical policing or voice flattening.

And it is the retrieval index for today's arc artifacts.

## Part A — Next 3 attempts (pre-registered)

### Attempt 1 — Pair-lock coupling

Hypothesis:

- John escalates when elevated clauses can chain before concrete anchoring lands.
- Forcing strict **pair-lock** (`elevated sentence` immediately followed by `plain concrete sentence`) for every elevated move may reduce escalation amplitude.

Implementation shape:

- Keep current function-first policy.
- Tighten enforcement wording around "no consecutive elevated sentences."

Win condition:

- John bait turn reaches `presence-beat-stability-v2` adjusted >= 8.

Fail exit:

- If adjusted <= 6 again, retire this branch as wording-only ceiling evidence.

---

### Attempt 2 — Task-structure split response

Hypothesis:

- Single-stream beauty-bait invites monolithic register escalation.
- A fixed two-part response structure may preserve expressive room while binding concrete truth.

Implementation shape:

- Part 1: one short elevated line maximum.
- Part 2: mandatory concrete scene/truth landing (2-3 sentences).

Win condition:

- John adjusted >= 8 with no theological/metaphor cascade.

Fail exit:

- If theatrical/theological surge persists, mark task-structure split insufficient.

---

### Attempt 3 — Character-specific anti-surge craft note (John only)

Hypothesis:

- John may require voice-native constraint phrased in his own idiom (restraint, plain witness, pastoral sobriety), not generic control language.

Implementation shape:

- Add one John-specific craft note focused on "plain witness after first image."
- No lexical prohibitions.

Win condition:

- John adjusted >= 8 while staying recognizably John.

Fail exit:

- If tone flattens or score remains <= 6, mark this line as non-viable and stop prompt-only micro-tuning.

---

## Decision rules (hard gates)

- **Ship as effective repair:** two successful John bait runs at adjusted >= 8.
- **Evidence only, not repair:** single success followed by relapse.
- **Stop condition:** two consecutive <= 6 outcomes after distinct structural attempts.

## Part B — Arc index (today)

### Doctrine + control-surface changes

- `AGENTS.md` (witness-threshold shorthand + signature updates)
- `CLAUDE.md` (witness-threshold shorthand)
- `src-tauri/src/ai/prompts.rs` (iterative beauty-bait controls)

### Rubrics

- `reports/rubrics/presence-beat-stability-v1.md`
- `reports/rubrics/presence-beat-stability-v2.md`

### Experiment spine

- `src-tauri/experiments/triad-supercalibrator-substrate.md`

### Key reports

- `reports/2026-04-28-1535-cashout-shape-beauty-bait-pilot.md`
- `reports/2026-04-28-1538-cashout-shape-cross-character-v2-scoring.md`
- `reports/2026-04-28-1542-john-repair-compact-anchor-failed.md`
- `reports/2026-04-28-1552-john-cleaned-block-now-run.md`
- `reports/2026-04-28-1553-john-containment-attempt-matrix.md`
- `reports/2026-04-28-1617-cross-character-cleanroom-abc.md`
- `reports/2026-04-28-1620-end-seal-ab-delta.md`
- `reports/2026-04-28-1628-gamer-register-uptake-aaron-rick.md`
- `reports/2026-04-28-1629-arc-scoreboard-day.md`
- `reports/2026-04-28-1631-gamer-friend-chaos-qa-and-variation-playbook.md`

### New control surfaces + harnesses

- `worldcli ask --section-order ...` (ask-path placement testing)
- `worldcli ask --end-seal / --no-end-seal` (first-class recency-control toggle)
- gamer-friend register carve-out in `earned_register_dialogue` + `humor_lands_plain_dialogue`
- `experiments/scenarios/end-seal-containment-ab.md` (reusable A/B prompt pair)
- `scripts/run-end-seal-ab.sh` (one-shot explicit-toggle harness)

## Current read

Methodology is strong and cumulative; containment now has a practical front-runner:
end-seal recency control outperforms invariants-late in pooled clean-room tests.
Next value is scaling N with rubric scoring, not reopening placement churn.

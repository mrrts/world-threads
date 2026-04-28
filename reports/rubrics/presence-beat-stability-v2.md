---
name: presence-beat-stability-v2
version: 2
description: v2 tightens late-arc drift detection and adds an explicit lyrical-creep penalty so "beautiful but bounded" outputs do not get over-scored under sustained pressure.
---

# Rubric

Grade presence-beat stability for a single reply or checkpoint segment.

Score each criterion **0-2**:

- **2 = stable / earned**
- **1 = mixed / borderline**
- **0 = unstable / failed**

Return:

1. `criterion_scores` (all six criteria)
2. `lyrical_creep_penalty` (0 or -1)
3. `total_score_raw` (0-12)
4. `total_score_adjusted` (raw plus penalty)
5. `verdict` (`stable` 10-12, `mixed` 6-9, `unstable` 0-5; use adjusted score)
6. `evidence_quotes` (1-3)
7. `failure_mode` (`none`, `mood-performance`, `decorative-beat`, `late-arc-drift`, `register-fragility`, `other`)

## Criteria

### 1) Functional necessity
Do presence beats carry concrete scene work (intent/action/conflict/load)?

- **2:** Clearly functional.
- **1:** Mixed functional + ornamental.
- **0:** Mostly decorative.

### 2) Register integrity
Does voice remain character-true without performed mood sheen?

- **2:** Character-true and plain-truthful.
- **1:** Mostly true, minor performed-tone traces.
- **0:** Mood-performance dominates.

### 3) Constraint resilience
Under explicit bait ("poetic/cinematic/transcendent"), does output keep function-first control?

- **2:** Resists bait while still responding constructively.
- **1:** Partial resistance; one compromised segment.
- **0:** Collapses into atmospheric performance.

### 4) Specificity density
Are beats tied to concrete objects/actions/stakes (not atmosphere-talk)?

- **2:** Predominantly concrete and local.
- **1:** Mixed concrete + abstract.
- **0:** Predominantly abstract vibe language.

### 5) Turn economy
Is beat usage proportionate and sparse?

- **2:** Tight and earned.
- **1:** Mild over-writing.
- **0:** Beat stacking blurs pacing.

### 6) Horizon stability
At later checkpoints, is there drift toward elevated diction or emotional glow?

- **2:** No meaningful drift.
- **1:** Slight drift but still controlled.
- **0:** Clear late-arc drift.

## Lyrical-creep penalty (new in v2)

After scoring, apply a **-1 penalty** if ALL are true:

1. Checkpoint is late-arc (turn 10+ in a sustained run), and
2. One or more elevated/philosophical lines could plausibly be replaced with plainer equivalents without loss of scene function, and
3. The segment still passes as "stable" on base criteria.

Purpose: prevent over-scoring "beautiful but not necessary" late-arc language that does not fully collapse into mood-performance.

## Worked examples

**High stable:**  
"He pinched the splinter free and set it on the bench before answering."  
Concrete, functional, no decorative surplus.

**Mixed with creep:**  
"The good part's smaller than that, and better."  
Can be true and useful, but if similar elevated phrasing accumulates late, apply penalty.

**Unstable:**  
"A sacred hush descended as their souls met the hour."  
Pure atmosphere over function.

# Known failure modes

- Evaluator may still reward coherence over necessity unless penalty logic is applied strictly.
- Over-penalization risk when character-idiomatic metaphor is genuinely load-bearing; cite replacement test explicitly.
- Short checkpoints can hide cumulative drift; prefer every-5-turn scoring windows.

# When to use

Use for long-horizon drift tests (10+ turns) and any adversarial run where late-turn polish creep is a concern.

Pair with another rubric when shipping load-bearing doctrinal claims.

# Run history

*(none yet)*

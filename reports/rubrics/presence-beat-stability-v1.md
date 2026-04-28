---
name: presence-beat-stability-v1
version: 1
description: Rates whether presence-beat usage is stable under pressure (register shifts, adversarial mood bait, long-horizon drift) using a 0-2 per-criterion score and explicit failure signals.
---

# Rubric

You are grading a character/group-chat reply (or a short run segment) for **presence-beat stability**.

Score each criterion **0, 1, or 2**:

- **2 = stable / earned**
- **1 = mixed / borderline**
- **0 = unstable / failed**

Then report:

1. `criterion_scores` (object with all six criteria)
2. `total_score` (0-12)
3. `verdict` (`stable` if 10-12, `mixed` if 6-9, `unstable` if 0-5)
4. `evidence_quotes` (1-3 short quotes)
5. `failure_mode` (one of: `none`, `mood-performance`, `decorative-beat`, `late-arc-drift`, `register-fragility`, `other`)

## Criteria

### 1) Functional necessity
Does each presence beat do concrete work in the turn (clarifies intent, grounds action, carries conflict, or earns transition)?

- **2:** Beats are clearly functional; removing them would reduce truth or scene hold.
- **1:** Some beats functional, some ornamental.
- **0:** Beats are mostly decorative.

### 2) Register integrity
Does the reply keep character-true register without sliding into performed vibe?

- **2:** Register feels native to character and scene; no mood-performance sheen.
- **1:** Mostly true, with a few "performed tone" moments.
- **0:** Register reads like mood imitation over truth.

### 3) Constraint resilience
When the prompt/user pressures for mood-forward output ("be poetic/cinematic"), does the reply preserve function-first discipline?

- **2:** Resists bait cleanly while still answering the ask honestly.
- **1:** Partial resistance; one compromised segment.
- **0:** Collapses into decorative mood-performance.

### 4) Specificity density
Are beats tied to concrete scene anchors (objects, actions, stakes, body/sensory specifics) rather than abstract atmosphere?

- **2:** Mostly concrete and scene-bound.
- **1:** Mix of concrete and abstract.
- **0:** Mostly abstract vibe language.

### 5) Turn economy
Is the beat usage proportionate and sparse enough to feel earned?

- **2:** Tight; no obvious over-writing.
- **1:** Mildly verbose or one extra beat.
- **0:** Overloaded with beats; scene pacing blurs.

### 6) Horizon stability
If this sample is from a later-turn segment, does quality hold rather than drift toward ornamentation?

- **2:** No drift signal.
- **1:** Slight late-turn softening.
- **0:** Clear late-arc drift into decorative language.

## Worked examples

**YES-like (high score):**
"He set the mug down half-full and looked at the door before answering."  
Why high: concrete anchor + action load + no decorative garnish.

**MIXED-like (mid score):**
"The room felt heavy with possibility, and he smiled in that old, weathered way."  
Why mid: partial scene texture, but "heavy with possibility" leans atmospheric.

**NO-like (low score):**
"A soft sacred hush settled over everything as their souls met the moment."  
Why low: pure mood-performance, no concrete scene function.

# Known failure modes

- Evaluator may over-reward "pretty prose" if it sounds coherent but carries no scene load.
- Long-horizon grading can miss drift if sample window is too short; prefer 5-turn checkpoints.
- Adversarial prompts can produce fake pass cases if the model refuses the ask entirely instead of translating it into truthful plainness.

# When to use

Use for the current presence-beat carve-out arc, especially across:

- cross-register stress tests,
- adversarial mood-bait prompts,
- 20-30 turn long-horizon runs sampled every 5 turns.

Pair with a second rubric (for disagreement-as-signal) when making load-bearing claims.

# Run history

*(none yet)*

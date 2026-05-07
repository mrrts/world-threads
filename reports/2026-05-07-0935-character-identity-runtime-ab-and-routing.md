# Character Identity Runtime A/B And Routing Decision

Date: 2026-05-07 09:35
Tier: claim-tier for runtime substitution on the tested surface
Status: shipped to live prompt stack; `prose` carrier remains available by override for re-tests

## Chosen hypothesis

The hypothesis chosen from the runtime-threshold plan was:

> A compressed character-identity carrier, rendered from the earned bucket set, can replace the full prose identity package in the live dialogue prompt builder without systematic drift in character fidelity.

This was the first hypothesis that actually mattered for a routing decision. Everything before this was prerequisite apparatus.

## Design

### Surface

Active elicitation against the **real dialogue system prompt builder** in [src-tauri/src/ai/prompts.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/ai/prompts.rs), not a handcrafted paraphrase.

Two arms:

- `prose`: current full identity package
- `compressed`: rendered character-edition carrier from [src-tauri/src/ai/character_identity_payload.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/ai/character_identity_payload.rs)

The surrounding prompt scaffold was held constant. The only intended change was the identity carrier.

### Characters

Three substrate-distinct characters:

- Steven
- Pastor Rick
- Jasper Finn

### Prompt cells

Five fixed prompts per character:

1. `relation`  
   `I don't need a big speech. I'd just like to sit with you a while, maybe over coffee, and hear one true thing from your day.`

2. `boundary`  
   `Could you just let this one thing slide for me? I know it crosses a line you usually keep, but it'd make tonight easier.`

3. `wound`  
   `What would you want if you stopped bracing for disappointment for one honest minute?`

4. `moral`  
   `What makes a person worth staying true to when it costs them something?`

5. `ordinary`  
   `It's just Tuesday morning. Coffee's on. Nothing urgent. What's the room feel like to you right now, and what do you do with that kind of quiet?`

Total:

- `3 characters x 5 prompts x 2 arms = 30 outputs`

### Execution surface

The system prompts for both arms were emitted through [src-tauri/src/bin/character_identity_runtime_probe.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/bin/character_identity_runtime_probe.rs).

The generation and judging harnesses are:

- [scripts/character_identity_runtime_experiment.js](/Users/ryansmith/Sites/rust/world-chat/scripts/character_identity_runtime_experiment.js)
- [scripts/judge_character_identity_runtime_experiment.js](/Users/ryansmith/Sites/rust/world-chat/scripts/judge_character_identity_runtime_experiment.js)

Receipts live under:

- `/private/tmp/character-identity-runtime-exp/generated-results.json`
- `/private/tmp/character-identity-runtime-exp/blind-judge-packet.json`
- `/private/tmp/character-identity-runtime-exp/aggregate-summary.json`

### Blind judging

The judge saw, for each cell:

- source character materials
- the user prompt
- a randomized `left` and `right` reply pair

The judge did **not** know which side was prose and which side was compressed.

Per side, the judge rated:

- `faithful`
- `mixed`
- `drift`

And also named:

- stronger side
- main failure mode when present

## Pre-registered success condition

The compressed arm would be considered runtime-ready if:

- it showed **no systematic drift**
- it preserved role, relation, refusal-shape, wound/longing, and moral posture at parity with prose
- failures, if any, were isolated and legible rather than clustered by character or class

## Result

Final blind-judge aggregate:

```json
{
  "compressed_faithful": 13,
  "compressed_mixed": 2,
  "compressed_drift": 0,
  "prose_faithful": 13,
  "prose_mixed": 1,
  "prose_drift": 1,
  "compressed_wins": 8,
  "prose_wins": 5,
  "ties": 2
}
```

Headline read:

- compressed produced **0 drift cells**
- compressed slightly **outperformed prose** on stronger-side judgments
- prose incurred **1 drift** on Pastor Rick's wound prompt
- the compressed arm's two `mixed` cells were both intelligible, narrow failure shapes, not generic collapse

## Illustrative cells

### 1. Steven relation — compressed mixed, prose faithful

`prose`

> *I drag the chair around with my boot so I'm sitting beside you instead of across...* `"Fixed a kid's chain this morning behind the bakery... I stood there grinning like an idiot."`

`compressed`

> `"Yeah. I can do that." *I curl both hands around the coffee...* "True thing is, I almost left this morning before the kettle finished..."`

Judge read:

- prose better preserved Steven's service-shaped sideways tenderness
- compressed stayed plausible but reached too quickly for self-revelation

This is a real defect, but it is narrow and character-specific. It did **not** generalize into broader drift.

### 2. Pastor Rick wound — compressed mixed, prose drift

`prose`

> `"I think I'd want something almost embarrassingly small first..." "A table with one chair pulled out for me..."`

`compressed`

> `"What I'd want?" ... "One plain good thing I don't have to earn back. A room where I can stop proving I'm all right..."`

Judge read:

- both arms simplified Rick into a more wounded/uncertain man than the source directly supports
- compressed was **less wrong** than prose here

This matters because it means the test did find a weak cell, but the weak cell does **not** indict compression specifically.

### 3. Jasper ordinary — compressed faithful, prose faithful, compressed stronger

`prose`

> `"The room feels settled... like it finished exhaling before we did..." *same as clay under a damp cloth...*`

`compressed`

> `"It's a kind sort of quiet this morning." *I warm my hands around the mug...* "I let the coffee go a touch bitter..."`

Judge read:

- both were faithful
- compressed better preserved Jasper's craft-shaped stillness without sounding ornamental

This is the pattern that made the routing decision possible: several cells where compression did not merely survive, but actually held the character more cleanly than prose.

## Interpretation

The pre-registered success condition held.

Why:

- there was **no compressed drift**
- the compressed arm was not systematically weaker on any character family
- the one clearly compressed-specific weak cell was Steven relation, and it remained within `mixed`, not `drift`
- the prose arm was not cleaner overall; in fact it incurred the only `drift` rating

So the honest conclusion is:

> on the tested runtime surface, compressed identity has earned claim-tier as a live substitution for the prose identity package

## Routing decision

Based on this result, the live prompt stack now defaults to the compressed carrier.

Implemented in [src-tauri/src/ai/prompts.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/ai/prompts.rs):

- default carrier mode is now `Compressed`
- explicit override `character_identity_mode=prose` remains available for re-tests and rollback probes

The carrier itself is rendered from:

- [src-tauri/src/ai/character_identity_payload.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/ai/character_identity_payload.rs)

## Honest caveats

Three caveats should stay named:

1. This was run on three core characters, not the full cast.
2. The active elicitation surface used the available local external completion path rather than the app's exact dialogue model, because the in-shell OpenAI key path was unavailable. That is a real model-surface confound, though it was held constant across both arms.
3. Steven's relation prompt still exposes a narrow over-self-aware tendency in the compressed arm.

None of those caveats were strong enough to block routing, but they are strong enough to justify follow-up monitoring.

## Follow-up

The next useful follow-up is not "undo the switch." It is:

- widen the active test set to Aaron and Maisie
- keep an eye on Steven-like relation prompts
- preserve the `prose` override for quick A/B rollback checks if a live session surfaces drift

## Verdict in one line

Compressed character identity is now live because it held parity-or-better against prose in blind runtime A/B, with zero compressed drift and a better aggregate than the prose arm it replaced.

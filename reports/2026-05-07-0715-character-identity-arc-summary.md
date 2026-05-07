# Character Identity Arc Summary

Date: 2026-05-07 07:15
Tier: synthesis-summary
Status: offline research arc; not shipped to prompt stack

## Why this arc existed

This arc was not trying to "improve characters" in the abstract. It was trying to answer a narrower question:

> If the project ever wants to compress character identity blocks the way it compressed the Empiricon, what proof apparatus has to exist first so that we do not flatten people into types?

The motivating constraint was the same one already earned elsewhere in the repo:

- do not trust compression before the artifact class has an earned taxonomy
- do not trust compression before there is some encode/decode surface
- do not trust compression before there is an audit capable of catching flattening

For craft-rule prose, the project earned a `v3` sacred-payload taxonomy. For the Empiricon, the project earned a formula-canonical character edition plus a decode audit. This arc asked what the analogous groundwork would have to be for **character identity blocks**.

So the real object of the work was:

- not a new runtime feature
- not prompt tuning
- not a character rewrite pass

It was the creation of an **offline research instrument** that can answer whether identity-compression preserves the person-bearing parts of a character block.

## What the arc learned

The first discovery was that character identity blocks are a different artifact class than craft-rule prose.

Craft-rule prose primarily carries proposition-bearing distinctions. Character identity blocks primarily carry **person-bearing distinctions**. The live rows read through `worldcli` and rehearsal passes suggested that the load-bearing content in identity blocks is not "doctrine" or even "facts" by themselves, but a mixed bundle of:

- role and station
- relation geometry with the user
- idiomatic voice constraints
- embodied recognition markers
- named attachments
- wound / longing asymmetry
- refusal-shape
- moral / theological posture
- discrete backstory facts

That led to the lean proposed class set:

```text
role_frame
relation_anchor
voice_lift
embodied_marker
attachment_node
wound_longing
refusal_shape
moral_theological_position
fact_atom
```

`gravity_line` was pressure-tested as a possible tenth universal class and did **not** earn promotion. The rehearsals on Steven, Maisie, Pastor Rick, and Aaron all pointed the same way: some characters have one or two especially load-bearing sentences, but that is better treated as a local pressure note than as a new top-level wrapper family.

## What was actually built

This arc moved beyond theory. It now has a real offline harness:

- [src-tauri/src/ai/character_identity_payload.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/ai/character_identity_payload.rs)
- [src-tauri/src/ai/character_identity_audit.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/ai/character_identity_audit.rs)
- [src-tauri/src/bin/character_identity_audit.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/bin/character_identity_audit.rs)
- [src-tauri/tests/character_identity_payload.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/tests/character_identity_payload.rs)

And it is grounded in real fixture rows, not invented examples:

- Aaron
- Steven
- Maisie Rourke
- Pastor Rick
- Jasper Finn

The harness now does four things:

1. Reads a `Character` row offline.
2. Splits the identity surface into auditable buckets.
3. Encodes and decodes a payload without touching prompt assembly.
4. Audits whether the decoded bucket set matches the source shape.

That alone would already have made the arc worthwhile, but the work also exposed a second real layer:

- character identity does not live only in `identity`, `voice_rules`, `boundaries`, and `backstory_facts`
- `relationships` and `state` also carry live person-shape pressure

So the harness was extended to preserve:

- raw `relationships` and `state` in the source snapshot for losslessness
- `relationship_note` and `state_signal` as auditable companion buckets

That means the instrument now catches not only whether a character still sounds right in the abstract, but whether live relational metadata and active pressures survive:

- relationship notes
- goals
- open loops
- last-seen marker
- mood
- trust-user scalar

## What this work accomplished

The arc accomplished three useful things.

### 1. It clarified the prerequisite for future runtime compression

If the project ever wants to compress character identity blocks into a faithful sacred-payload form, it now has the beginnings of the apparatus needed to do that honestly rather than intuitively.

### 2. It created a regression detector for character flattening

Even if no runtime compression ever ships, the harness is still useful. It gives the repo a way to inspect whether edits to character rows are preserving or flattening the load-bearing person-shape.

### 3. It made the object of preservation more precise

Before this arc, "keep the character feeling like themselves" was directionally true but too vague to instrument. After this arc, the project has a more concrete answer to what "themselves" consists of in auditable terms.

## What this arc does not prove

This is where the honest scope boundary matters.

The arc does **not** prove:

- that the current class set is complete for all characters
- that the current splitter heuristics are truly lossless
- that a blind decode from the compressed form would behaviorally reproduce the same character in live chat
- that prompt-stack routing should change
- that character identity compression is already worth shipping

The current state is stronger than a sketch, but weaker than an earned runtime law.

The right wording is:

> complete as an offline research harness; not complete as a finished sacred-payload system for runtime character identity compression

## What remains before runtime use would be honest

If the project ever wanted to route character prompts through a compressed identity carrier, at least these would still need to happen:

- broader validation across more characters
- stricter exact-bucket assertions, not just pass/fail audits
- stronger blind-decode validation
- behavioral-equivalence testing against the live prose identity path
- a decision about whether the current heuristics are acceptable as rules or only as research probes

Until then, keeping this surface offline is not caution for its own sake. It is the project obeying its own law about earned compression.

## The through-line in one sentence

This arc built the first honest instrument for asking whether character identity can be compressed **without ceasing to carry a person**.

## Pointers

- [Taxonomy sketch](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0015-character-identity-v3-taxonomy-sketch.md)
- [Steven rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0145-character-identity-steven-desk-decode.md)
- [Maisie rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0205-character-identity-maisie-gravity-line-rehearsal.md)
- [Pastor Rick rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0235-character-identity-pastor-rick-gravity-line-rehearsal.md)
- [Aaron rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0315-character-identity-aaron-rehearsal.md)
- [Proposal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0415-character-identity-v3-proposal-and-encoder-design.md)
- [Harness outline](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0525-character-identity-harness-outline.md)

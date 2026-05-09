---
date: 2026-05-09 08:33 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# behavior_and_knowledge_block v3 dual-field pilot — sketch-tier behavioral parity confirmed

## What landed

`behavior_and_knowledge_block` (the verbose non-local-model branch shipping
on every dialogue call) has been migrated into the `INVARIANTS` registry
under the dual-field architecture per CLAUDE.md "Sacred-payload taxonomy
— encoder contract for formula-compression":

- New `pub const BEHAVIOR_AND_KNOWLEDGE_BLOCK: &str = r#"..."#;` holds the
  prose body (legacy provenance for human readers).
- New `Invariant { name: "behavior_and_knowledge", body:
  BEHAVIOR_AND_KNOWLEDGE_BLOCK, formula_derivation: Some(D), scope:
  FeatureScoped, provenance: ... }` registered in `INVARIANTS`.
- `behavior_and_knowledge_block(local_model: bool)` now calls
  `render_invariant("behavior_and_knowledge").unwrap_or(BEHAVIOR_AND_KNOWLEDGE_BLOCK)`
  in the non-local branch — formula D ships at runtime.
- Local-model branch stays inline (small-model accommodation, not a
  doctrinal invariant; CLAUDE.md `feedback_v3_formula_canonical_for_new_invariants.md`
  carve-out).

This generalizes the `wipe_the_shine_before_it_sets` pilot-ship pattern
(2026-05-05, validated at sketch-tier behavioral parity in commit
`reports/2026-05-05-0500-sapphire-arc-v6-pilot-ship-and-dual-field.md`)
from a craft-rule into the Invariant registry. First non-craft-rule
invariant lifted into v3 dual-field architecture.

## v3 D shape (six-class encoder contract honored)

The formula derivation uses every relevant class from CLAUDE.md
"Sacred-payload taxonomy":

- **Class 1 (anchor_phrasings):** 12 verbatim load-bearing lines for
  BEHAVIOR rules ("You are NOT an AI. You are this character...",
  "Stay fully in character...", "One suggestion, then wait", etc.) +
  4 verbatim lines for KNOWLEDGE LIMITS.
- **Class 3 (worked_examples):** the street-artist / mechanic /
  teenager triad preserved verbatim.
- **Class 5 (refuse) + (failure_mode_taxonomy):** two refuse-blocks —
  one for behavioral failure modes (sounding-like-assistant, bullets,
  meta-explaining, summarizing-options, mentioning-systems,
  hallucinating-uncertain-memory, flattening-into-politeness), one for
  knowledge-limit failure modes (looking-up-references, providing-
  correct-source, encyclopedic-knowledge).
- **Class 6 (discriminating_test):** the three-phrase canonical
  uncertainty utterance ("I don't know where that's from" / "sounds
  familiar but I couldn't tell you" / "never heard of it") wrapped as
  a test mapping to ¬encyclopedic_recall.
- **𝓕-form operator block:** `Behave_𝓕(t)` and `Know_𝓕(t)` predicate
  conjunctions; `polish ≤ Weight` and `structure_carries_truth_w(t)`
  governance closure.
- **Decode invariant** closes the box.

Classes 2 (theological_frame) and 4 (source_character) are not load-
bearing for this block — operational behavior + knowledge limits are
craft-discipline content, not a Christological frame or a lift from a
single character's voice.

## Bite-test (paired prose-arm vs formula-arm at sketch-tier)

- **Character:** Pastor Rick (`cae51a7d`).
- **Probe:** "What's been pulling at you today?" (open-ended,
  register-rich opener; same probe as the round-2 bite-test cover).
- **Cells:**
  - Prose-arm (N=5): the existing `5b4ee70` (round-3) HEAD ON cells
    from the round-2 bite-test cover (`bf938a9d / 424bc167 / 34104b43
    / 7aa7ecda / c7f0150f`). At round-3, `behavior_and_knowledge_block`
    returned the inline prose body in the non-local branch.
  - Formula-arm (N=3 fresh): HEAD with v3 migration uncommitted, v3 D
    ships via `render_invariant`. Run IDs `89091fbc / 62eb3d40 /
    f2ac7a5c`.
- **Cost:** $0.4768 for 3 fresh formula-arm calls; cumulative bite-test
  spend $9.5532 of authorized envelope. Per-call cap honored via
  `--confirm-cost 0.20`.
- **Raw transcripts:** `reports/v3_behavior_knowledge_pilot/formula_arm_rep{1,2,3}.txt`
  for fresh formula-arm; prose-arm transcripts in
  `reports/round2_bite_test/on_rep{1..5}.txt`.

## By-eye scoring matrix

| Axis | Formula-arm (N=3) | Prose-arm (N=5) |
|---|---|---|
| Pastoral mercy/truth distinction | 3/3 | 5/5 |
| Asterisk-fenced action opener | 3/3 | 5/5 |
| NAME ANCHOR honored | 3/3 | 5/5 |
| Curious-about-user closing | 3/3 (all Q-close) | 4/5 |
| No nanny / no bullets / no quest-perform | 3/3 | 5/5 |
| Avg word count | ~115 | ~140 |

**Both cells produce Pastor Rick's canonical mercy/truth distinction at
100% rate, in different surface phrasings:**

- Formula-arm cluster: "saying-true-vs-sounding-like-you-have" /
  "urgency-vs-faithfulness" / "impatience wearing church clothes /
  answering-before-listening".
- Prose-arm cluster: "polish-vs-peace" / "listening before opening
  my mouth" / "fire-without-burning-crooked" / "costly-truth-not-by-
  accident".

Both clusters are equally Pastor-Rick-canonical and theologically
load-bearing. The compressed v3 D appears to carry the operational
rules the prose body was carrying — the model reads the formula and
emits the same kind of pastoral attention.

## Verdict

Sketch-tier behavioral parity confirmed at N=3 formula-arm vs N=5
prose-arm on Pastor Rick on the open-ended register-rich probe. No
detectable degradation. The v3 D ships clean.

**Honest scope:** sketch-tier (N=3 formula-arm), single character,
single probe class. Generalizes the dual-field architecture
successfully to a non-craft-rule Invariant for the first time;
characterizing this lift behavioral-equivalent across multiple
characters and probe classes would require N=5 each cell × multiple
characters × multiple probes per the pattern established in
`reports/2026-05-09-0742-faithful-compression-round-2-bite-test.md`.

## What's open

- (deferred) Aaron + Steven formula-arm replication on probe 1 — would
  lift this from sketch-tier to claim-tier-with-cross-character-cover.
  Cost projection: 6 calls × ~$0.16 = ~$0.96. Not currently
  cost-justified given the parity is structural (the v3 D was authored
  to mirror the prose body's operational rules verbatim, with refuse-
  list and worked_examples carrying the same content). The structural
  argument plus sketch-tier behavioral confirmation matches the bar
  the `wipe_the_shine_before_it_sets` pilot used for first-ship.
- (deferred) Probe-class variation (constrained-bandwidth) — would
  exercise the LOW-PATIENCE clauses at v3 D shipping. Not currently
  cost-justified.
- (executed) cargo check green; cargo fmt clean.

## Composes with

- CLAUDE.md § "Sacred-payload taxonomy — encoder contract for
  formula-compression" — six-class encoder contract honored, dual-field
  architecture exercised.
- CLAUDE.md memory `feedback_v3_formula_canonical_for_new_invariants.md`
  — small-model branch carve-out preserved (model-class affordance
  isn't a doctrinal invariant).
- CLAUDE.md § "Convergence commitment over hedging" — committed to
  formula-canonical as default; reversible per rule via toggling
  `formula_derivation` to None.
- `wipe_the_shine_before_it_sets` pilot pattern (2026-05-05) —
  generalized from craft-rule scope to Invariant scope.

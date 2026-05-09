---
date: 2026-05-09 09:45 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# Anthropic Claude Sonnet 4-6 cross-substrate bench — three substrates, three directions: length affordance INVERTS across substrate-class

## What ran

- **Hypothesis under test:** the length + anchor-diversity affordances
  measured on gpt-5.4 in the round-2 bite-test cover replicate as
  substrate-capacity-dependent (Crown 15 Quickener frame extension)
  but might still show consistent DIRECTION across substrates.
  gpt-4o falsifier (commit `cb8f9c5a`) collapsed the magnitude to
  ~null on probe 1 — direction was weakly positive but not
  statistically distinguishable from noise. Anthropic Claude Sonnet
  4-6 was the third substrate-class needed to characterize the
  affordance fully.
- **Method:** paired ON (HEAD compressed) vs OFF (8d64d81 pre-round-2)
  toggle, 3 substrate-distinct characters (Pastor Rick + Aaron +
  Steven), 2 probe classes (open-ended + constrained-bandwidth), N=3
  each cell, on **claude-sonnet-4-6** via
  `scripts/anthropic_pipeline_reconstruction.py` (extended in this
  commit to capture round-1/2/3 + v3 compression surfaces) +
  `consult_anthropic`.
- **Reconstruction extension:** `scripts/anthropic_pipeline_reconstruction.py`
  was extended to extract not just `*_BLOCK` constants but also
  `pub const NAME: &str = ...` surfaces matching the round-1/2/3
  compression arc (FUNDAMENTAL_SYSTEM_PREAMBLE, STYLE_DIALOGUE_INVARIANT,
  FORMAT_SECTION, four `*_FORMULA_INVARIANT_FRAMING` constants,
  KAVOD_PATTERN_INVARIANT_BLOCK). Methodology gap from the prior commit
  (where TARGET_BLOCKS captured only RYAN_FORMULA_BLOCK from round-1)
  is now closed for the cross-substrate test.
- **Cost:** 36 paid replies × ~$0.045 avg = **$1.61** total.
  Per-call well under the $0.20 cap. Cumulative bench spend across
  this thread: $16.63.
- **Raw transcripts:** `reports/anthropic_affordances_bench/{on,off}_<char>_<probe>_rep<N>.json`.

## Length distribution — three substrates, three directions

| Probe | Substrate | PR ON-OFF | Aaron ON-OFF | Steven ON-OFF | **Mean** | Direction |
|---|---|---|---|---|---|---|
| 1 (open-ended) | gpt-5.4 (N=5) | +42w | +30w | +20w | **+30.7w** | ON longer |
| 1 (open-ended) | gpt-4o (N=3)  | +3w  | -0.7w | +2w  | **+1.4w**  | ~null |
| 1 (open-ended) | Claude Sonnet 4-6 (N=3) | -25w | -27w | -27w | **-26w** | **ON shorter** |
| 2 (LP) | gpt-5.4 (N=5) | +0w  | +13w | +3w  | +5.3w  | weakly + |
| 2 (LP) | gpt-4o (N=3)  | -0.7w | -4.7w | +4.3w | -0.4w  | null |
| 2 (LP) | Claude (N=3) | -9w  | +9w  | 0w   | 0w     | null |

The probe-1 direction inverts cleanly across substrate-class:
gpt-5.4 (+30.7w ON longer) → gpt-4o (~null) → Claude (-26w ON
SHORTER). Three substrate-distinct witnesses, three different
patterns. The probe-2 collapse-toward-null pattern that gpt-5.4
showed (suggesting register-room is unlocked under open-ended
contracts only) does NOT replicate cross-substrate either —
Claude's null on probe-2 happens at a much lower absolute word-count
floor (21w vs gpt-5.4's 47w) which itself is informative about
LP-mode discipline interacting with substrate brevity bias.

## Anchor-diversity check — Claude probe-2 templating

Claude on probe-2 produced extremely concentrated lexical templates
across BOTH ON and OFF cells:
- Pastor Rick: 3/3 of ON, 3/3 of OFF used "Pray" as the opening verb.
  "Pray first" / "Pray." / "Pray before you plan." (ON);
  "Pray before you plan." 3/3 (OFF).
- Aaron: ON 2/3 used "Name the worst thing..."; OFF 2/3 used "Write
  down the one thing...".
- Steven: ON 2/3 used "Name..."; OFF 2/3 used "Write down the one
  thing..."

Claude's anchor-diversity is LOW in both arms — much lower than
gpt-5.4's ON cells (which showed 5/5 distinct framings). The
anchor-diversity affordance (compressed wrappers ride DISTRUST
RECURRING SENSORY ANCHORS cleaner) does not appear on Claude either,
because Claude's baseline anchor-diversity is already constrained
by its training distribution (different RLHF, different brevity
bias on LP-mode probes).

## Apparatus-honest verdict — great-sapphire FIRMLY refused; Crown 15 frame extends with new nuance

Per CLAUDE.md "Convergence as crown-jewel signal":
> calibration_tightening (great_sapphire_qualifies):
>   genuine: convergence_spans substrates_with_different_failure_modes
>   discriminating_test: convergence_spans substrates_with_different_failure_modes
>     [neg different_surfaces_of_same_workflow]

Three substrate-classes (gpt-5.4 / gpt-4o / Claude Sonnet 4-6) show
THREE DIFFERENT length-direction patterns on probe 1. There is NO
convergence — the affordance is not just magnitude-dependent on
substrate, the DIRECTION inverts. **Great-sapphire firing is firmly
refused.**

The empirical finding refines the Crown 15 Quickener "pipeline as
capacity-selective realization layer" frame:

> The compression's effect on length-budget allocation is
> substrate-class-dependent in BOTH magnitude AND direction:
>   - gpt-5.4: compression UNLOCKS register-room → ON longer (+30.7w).
>   - gpt-4o: compression has minimal effect → ~null (+1.4w).
>   - Claude Sonnet 4-6: compression COMPRESSES output → ON shorter (-26w).
>
> Compressed wrappers don't have a universal "leave more room for
> character voice" effect; they interact with each substrate's
> training distribution and RLHF in different and sometimes opposite
> ways. Crown 15's "capacity-selective realization layer" frame
> extends: the realization layer is bidirectional in selection
> direction, not just in magnitude.

This is a non-trivial finding. The compression rounds 1-3 produce
real, characterized affordances on the deployed gpt-5.4 substrate
(per round-2 bite-test characterized-tier evidence). On other
substrates the same compression toggle produces different effects —
gpt-4o flat, Claude inverse-shorter. The compression surface is not
an objective "good for all LLMs" lever; it's a substrate-tuned
selector that happens to favor expansion on gpt-5.4 specifically.

## Reconstruction-bench fidelity caveats (HONESTLY NAMED)

- BEHAVIOR_AND_KNOWLEDGE_BLOCK is DELIBERATELY EXCLUDED from this
  reconstruction. At HEAD it ships as registry-backed v3 dual-field;
  at 8d64d81 it shipped as inline fn-body prose. The const-extraction
  can capture HEAD's constant but not 8d64d81's fn-body prose, so
  including it would create unfair asymmetry. Both arms in this paired
  bench exclude this surface; the test measures the OTHER round-1/2/3
  surfaces' contribution. This is a true methodology limitation.
- Identity prose (`worldcli show-character` output) is included; world
  state, recent messages, journals, leader, group context, length-seal
  late-slot blocks are all NOT in the reconstruction. The bench tests
  what the load-bearing compressed-prose-stack does on Claude in
  isolation, not full-pipeline production.
- Claude's reconstruction produced ~14k input tokens vs gpt-5.4's
  ~31k production prompt — about half the prompt-stack mass because
  many production-only dynamic blocks are absent. The within-arm
  comparison is fair because both ON and OFF reconstructions share
  the same omissions; cross-substrate absolute comparison is not.
- N=3 on Claude is sketch-tier; the magnitude of the gpt-5.4 vs
  Claude inversion (+30.7w vs -26w = ~57w gulf) is too large for
  sample variance at N=3 to bridge. The direction inversion is
  empirically confident.

## What's open

- (executed) gpt-4o cross-substrate at N=3 (commit cb8f9c5a).
- (executed) Anthropic Claude cross-substrate at N=3 (this report).
- (executed) Reconstruction script extended to capture round-1/2/3 +
  v3 compression surfaces; methodology gap closed for non-fn-body
  surfaces.
- (deferred) N=5 lift on Claude — direction is large enough at N=3
  that re-running is unlikely to flip it, but full characterization
  would benefit from N=5.
- (deferred) Cross-substrate test of the v3 behavior_and_knowledge
  migration — would require fn-body extraction at 8d64d81 to make
  the bench fair.
- (deferred) Doctrine update / memory entry on the substrate-class-
  dependent compression-direction finding — Crown 15 frame extension
  worth lodging as a memory entry for future arcs.
- (executed) Tree restored to HEAD; cargo build green.

## Composes with

- `reports/2026-05-09-0742-faithful-compression-round-2-bite-test.md`
  — gpt-5.4 characterized-tier baseline.
- `reports/2026-05-09-0900-cross-substrate-affordances-falsifier.md`
  — gpt-4o sketch-tier null.
- This report — Claude Sonnet 4-6 sketch-tier inversion.
- CLAUDE.md § "Convergence as crown-jewel signal" — three-substrate
  test refused great-sapphire firing apparatus-honestly.
- Crown 15 Quickener (`project_quickener_fifteenth_sapphire.md`) —
  frame extends with bidirectional-selection-direction nuance.
- `feedback_matched_same_substrate_on_deployed_model.md` — cross-
  substrate version of matched-test.

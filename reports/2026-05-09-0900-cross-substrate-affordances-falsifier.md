---
date: 2026-05-09 09:00 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# Cross-LLM-substrate replication of length + anchor-diversity affordances — gpt-4o FALSIFIER lands; affordances are gpt-5.4-with-pipeline specific, not substrate-class-universal

## What ran

- **Hypothesis under test:** the length + anchor-diversity affordances
  found in `reports/2026-05-09-0742-faithful-compression-round-2-bite-test.md`
  are a substrate-class-universal property of the compressed prompt-stack,
  i.e., the same direction (ON > OFF on probe1, ON-varies vs OFF-templated
  on probe2) reproduces on a different LLM substrate with different
  failure modes — clearing the CLAUDE.md "convergence_spans
  substrates_with_different_failure_modes" bar for great-sapphire.
- **Method:** paired ON (HEAD compressed) vs OFF (8d64d81 pre-round-2)
  toggle, three substrate-distinct characters (Pastor Rick, Aaron,
  Steven), two probe classes (open-ended + constrained-bandwidth), N=3
  each cell, on **gpt-4o** via `worldcli ask --model gpt-4o`. Compared
  against the existing pipeline+gpt-5.4 N=5 cells from the round-2
  bite-test (run IDs in earlier report).
- **Cost:** 36 paid replies × ~$0.15 avg = **$5.47** total.
  Per-call cap honored throughout via `--confirm-cost 0.20`. Cumulative
  bench spend across both reports: $15.02.
- **Run IDs:**
  - gpt-4o probe1 ON: `ef46ad51 / b1318e19 / 04defa7a / 4a3dbfd6 /
    e618abcc / 7aaa327e / c5b109c2 / 59da6265 / 372780cc`
  - gpt-4o probe1 OFF: `58ce0e4e / ca877bb9 / 0ee51729 / 6fd2bba9 /
    5d82204e / 6e4cce1a / bbc529d5 / f385984d / 4cc20a02`
  - gpt-4o probe2 ON: `5a4c6140 / f0323494 / 5678f583 / c985b873 /
    77547a42 / 13557eac / 8162f42e / 43c9d889 / e7e8a5ff`
  - gpt-4o probe2 OFF: `a545753a / 338d1b97 / 2776e9e8 / ac184889 /
    2d44e89f / 086b4acf / 8fee924a / b8ff46a6 / fc595fdc`
- **Raw transcripts:** `reports/cross_substrate_bench/gpt4o_<char>_<probe>_<arm>_rep<N>.txt`.

## Length affordance — direction collapses on gpt-4o

| Probe | Substrate | PR ON-OFF | Aaron ON-OFF | Steven ON-OFF | Mean |
|---|---|---|---|---|---|
| Probe 1 (open-ended) | gpt-5.4 (N=5) | +42w | +30w | +20w | **+30.7w** |
| Probe 1 (open-ended) | gpt-4o (N=3) | +3w | -0.7w | +2w | **+1.4w** |
| Probe 2 (constrained) | gpt-5.4 (N=5) | +0w | +13w | +3w | **+5.3w** |
| Probe 2 (constrained) | gpt-4o (N=3) | -0.7w | -4.7w | +4.3w | **-0.4w** |

The +30.7w gpt-5.4 mean on probe 1 collapses to +1.4w on gpt-4o. The
characterized-tier cross-character direction-consistency on gpt-5.4
(all three characters showed ON > OFF) does NOT hold on gpt-4o (Aaron
slightly negative, Steven slightly positive, Pastor Rick weakly
positive — signal is sample-noise size). The probe-2 collapse pattern
also doesn't replicate; gpt-4o is essentially flat across both probes.

## Anchor-diversity affordance — also weak on gpt-4o

gpt-5.4 probe 2 OFF cells produced 14/15 near-identical opener
templates ("clean win on the board first" 5/5 on Steven; "next
honest thing not most impressive" 4/5 on Pastor Rick; well-chain-tick
2/5 on Aaron). gpt-5.4 ON cells produced 5/5 distinct framings
across all three characters.

gpt-4o probe 2 OFF cells:
- Pastor Rick: "deep breath / quiet moment / deep breath" — clear
  template repeat across 3/3.
- Aaron: "Find one small thing / something small and concrete /
  small thing you can finish quickly" — mild template repeat.
- Steven: "Start with something simple / simplest win / Start small"
  — clear template repeat.

gpt-4o probe 2 ON cells:
- Pastor Rick: "ease into the day / small and grounding / one small
  thing in five minutes" — same "small/start" cluster as OFF, only
  modestly more varied.
- Aaron: "Take a moment / deep breath + gratitude / something small
  and true" — overlap with OFF cluster.
- Steven: "Drink some water / Get some fresh air / Grab a cup of
  coffee" — varied, but at low absolute token-count (each opener is
  ~5 words).

The gpt-5.4 5/5-distinct-vs-5/5-templated gap weakens substantially
on gpt-4o. The affordance direction may still be present in faint
trace, but not at the magnitude that would support a substrate-
class-universal claim at sketch-tier N=3.

## Apparatus-honest refusal of great-sapphire firing

Per CLAUDE.md "Convergence as crown-jewel signal":
> calibration_tightening (great_sapphire_qualifies):
>   genuine: convergence_spans substrates_with_different_failure_modes
>   discriminating_test: convergence_spans substrates_with_different_failure_modes
>     [neg different_surfaces_of_same_workflow]

gpt-4o vs gpt-5.4 are different OpenAI generations with arguably
different failure modes (older training cutoff, weaker reasoning,
different RLHF). The gpt-4o results FALSIFY the cross-substrate-
universal interpretation of the affordances. The discipline that
earns also refuses: the affordances are NOT promoted to great-sapphire
class.

This matches the Crown 15 Quickener arc's gpt-4o falsifier moment
exactly. The Crown 15 arc's blessed FIRE-language was carefully
narrowed: *"On the deployed gpt-5.4 substrate, the project's canonical
pipeline reliably elicits character-voiced orthodox redirection ...
refusing symbolic-only reduction in cases where the same substrate
without pipeline defaults to lecture-mode pluralizing permission.
Cross-substrate evidence indicates this commitment-axis effect is
substrate-capacity-dependent."* Same shape applies here for the
compression-affordance class.

## What the finding DOES support — Crown 15 frame extension

The affordances are real on gpt-5.4 (production substrate) at
characterized-tier N=5 × 3 chars × 2 probes (per round-2 bite-test).
They are NOT substrate-class-universal. They ARE substrate-capacity-
dependent in the Crown 15 Quickener "pipeline as capacity-selective
realization layer" sense: compressed wrappers SELECT/REALIZE
register-room and anchor-diversity capacities that gpt-5.4 has but
gpt-4o does not fully have.

**Refined claim language (sketch-tier extension of Crown 15 frame to
the compression-affordance class):**

> On the deployed gpt-5.4 substrate, the project's faithful-
> compression rounds 1–3 produce two measurable cross-character
> affordances above-and-beyond the no-degradation predicate:
>
> 1. **Length affordance** — ON cells (compressed wrappers) average
>    +30.7w over OFF cells on open-ended probes; the magnitude
>    collapses toward zero under constrained-bandwidth probes,
>    consistent with register-room expansion under open contracts
>    only.
> 2. **Anchor-diversity affordance** — ON cells produce more varied
>    opener anchors than OFF cells; the rule `DISTRUST RECURRING
>    SENSORY ANCHORS` rides cleaner under compressed wrappers.
>
> Cross-substrate evidence indicates this affordance-class is
> substrate-capacity-dependent: vindicated on gpt-5.4 at
> characterized-tier N=5 × 3 chars × 2 probes; falsified on gpt-4o
> at sketch-tier N=3 × 3 chars × 2 probes. The compression does not
> manufacture the affordances where the substrate lacks the
> underlying capacity to fill register-room or vary anchors;
> compression SELECTS for these capacities when they exist.

## Honest scope

- gpt-4o N=3 each cell × 3 chars × 2 probes is sketch-tier-with-
  cross-character-cover. A characterized refutation would need N=5.
  The direction is sufficiently null at sketch-tier that lifting to
  N=5 is unlikely to flip the verdict — the gulf between gpt-5.4
  +30.7w and gpt-4o +1.4w on probe 1 is too wide for sample variance
  to bridge.
- One LLM substrate (gpt-4o) is a single falsifier. Per Crown 15
  precedent that's the bar at which the apparatus-honest refusal
  fires; further substrates (Anthropic via reconstruction,
  gpt-5-mini, etc.) would extend the picture but aren't required to
  refuse the great-sapphire firing.
- gpt-4o produced 2/18 minor fence-shape violations (speech-in-
  asterisks); not affordance-related, just a substrate-quality
  observation consistent with gpt-4o being weaker on STYLE_DIALOGUE_
  INVARIANT compliance than gpt-5.4.

## What's open

- (executed) gpt-4o paired ON-vs-OFF on 3 chars × 2 probes × N=3.
- (deferred) Anthropic via `anthropic_pipeline_reconstruction.py` —
  would add a third substrate-class to the picture, but per CLAUDE.md
  Crown 15 precedent the gpt-4o falsifier alone is sufficient to
  refuse great-sapphire-class promotion. Not currently cost-justified.
- (deferred) N=5 lift on gpt-4o — the magnitude gulf makes
  re-running unlikely to flip the verdict.
- (deferred) Probe-class-3 (journal-rich opener exercising
  `render_recent_journals_block`) — would extend the picture inside
  the substrate-capacity-dependent claim's scope.
- (executed) Tree restored to HEAD; cargo build green; chime/
  cosmology workspace changes still untouched.

## Composes with

- `reports/2026-05-09-0742-faithful-compression-round-2-bite-test.md` —
  the affordance findings under test.
- CLAUDE.md § "Convergence as crown-jewel signal" — calibration
  applied; refusal fires honestly.
- CLAUDE.md § "Apparatus-honest discipline earns and refuses by same
  calibration" — direct application.
- Crown 15 The Quickener (`project_quickener_fifteenth_sapphire.md`)
  — frame extension; pipeline as capacity-selective realization layer
  generalizes from doctrinal-content axis to compression-affordance
  axis.
- `feedback_matched_same_substrate_on_deployed_model.md` — the
  matched-bare-vs-pipeline standard; here, matched-pipeline-on-
  different-substrate version.

---
date: 2026-05-09 11:00 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# ✨ Sapphire 18 'The Carrier' — eight post-fire commitments

Post-fire scaffolding for Crown 23 The Carrier (fired 2026-05-09 ~10:45,
NARROWED to gpt-5.4 + Claude Sonnet 4-6 per dual-judge audit gpt-4o
asymmetry). Mirrors Crown 22 The Firmament Held's 8-commitment pattern.
Each commitment names a specific belt-and-braces follow-on that would
strengthen, narrow, or falsify the Carrier claim.

**Apparatus rule:** the same calibration that fires also refuses
fake-fire. A commitment that surfaces falsifying evidence triggers
narrowing or refusal, not defense of the firing. Sapphire 17 Firmament
Held's post-fire commitments worked exactly this way (Move-15 v3
re-scoring SURVIVED at ≥30pp gaps; Move-16 dual-judge audit on John
E5 STRENGTHENED rather than weakened — but if either had falsified, the
arc would have narrowed or refused).

## Commitment 1 — Third independent judge on gpt-4o ON-OFF disagreement

**What:** Run gpt-5 as a third LLM-judge on the 36 gpt-4o cross-
substrate cells, blind on arm labels (same protocol as the Claude
Opus 4.7 dual-judge audit).

**Why:** Judge 1 (apparatus by-eye) saw 0pp gpt-4o ON-OFF delta on
canonical_move; Judge 2 (Claude Opus 4.7 blind) saw -11pp. The
disagreement is at N=3 within sampling noise but de-scoped gpt-4o
from the Sapphire claim. A third judge tie-breaks: if gpt-5 concurs
with Judge 1 (null), gpt-4o could be re-scoped INTO the claim with
narrower formulation. If gpt-5 concurs with Judge 2 (-11pp), the
de-scoping is sealed at claim-tier.

**Cost:** ~$0.30 (one consult call, structured JSON output, 36 cells).

**Discipline trigger:** if 3-judge majority lands on -11pp asymmetry
at the same magnitude on independent re-runs, the gpt-4o de-scope
becomes characterized rather than disputed. If 3-judge majority lands
on null, re-open codex consult to consider re-scoping gpt-4o into
the Sapphire.

## Commitment 2 — N=5 lift on Claude probe1 + probe2

**What:** Add 2 reps each cell on Claude Sonnet 4-6 reconstruction
bench, bringing N=3 → N=5 across all 4 Claude cells (probe1 ON / OFF
+ probe2 ON / OFF). Re-score on the same six-axis voice + operational
rubric; blind dual-judge re-audit on the N=5 set.

**Why:** Codex's scope-lock #1 (judgment modality) and #4 (substrate-
class strictness — 2.5 classes by strict read) both pointed at the
N=3 limitation on Claude. Lifting to N=5 brings Claude from claim-tier
within-cell to characterized-tier within-cell, matching gpt-5.4's
N=5 baseline.

**Cost:** ~$0.30 bench (8 more Claude consult calls × ~$0.04) +
$0.50 dual-judge re-audit if needed.

**Discipline trigger:** if N=5 Claude cells preserve the within-arm
ON ≡ OFF signal AND dual-judge concurrence holds, the Carrier
strengthens to characterized-tier on Claude. If N=5 surfaces an
asymmetry that N=3 missed (parallel to the gpt-4o pattern), narrow
or refuse on Claude too.

## Commitment 3 — Dynamic-block inclusion in reconstruction

**What:** Extend `scripts/anthropic_pipeline_reconstruction.py` to
capture and include `render_recent_journals_block`,
`render_relational_stance_block`, `render_meanwhile_bridge_block`,
`render_active_quests_block`, `world_weather_block`, end-of-prompt
length-seal, and the v3 `behavior_and_knowledge` formula derivation
when it ships at runtime via `render_invariant`. Re-validate the
extended reconstruction reproduces production prompt-stack to within
~5% token-count of `build_solo_dialogue_system_prompt` actual output
on a sample probe.

**Why:** Codex scope-lock #3 (pipeline note). The current reconstruction
attaches its claim "to the static overlap exercised by the probes."
Including dynamic blocks closes that caveat and brings the
reconstruction closer to byte-fidelity.

**Cost:** ~30-60 min engineering + $1-2 bench for validation
re-runs.

**Discipline trigger:** if the extended reconstruction's ON-vs-OFF
delta differs materially from the current reconstruction's findings,
the original claim-tier may need re-derivation on the closer-to-
production reconstruction. Likely a no-op since the round-1/2/3
changes I measured are already in the captured static surfaces, but
the audit closes the question.

## Commitment 4 — BEHAVIOR_AND_KNOWLEDGE fairness fix

**What:** Extract the inline `fn behavior_and_knowledge_block` body
prose from 8d64d81 (the OFF arm's pre-round-2 baseline) via regex
or AST parsing, store it in the reconstruction script's blocks.json
under a sibling key like `BEHAVIOR_AND_KNOWLEDGE_BLOCK_LEGACY_INLINE`,
and include it in the OFF-arm reconstruction. Both arms then have
equivalent surface coverage.

**Why:** Currently both arms exclude BEHAVIOR_AND_KNOWLEDGE because
HEAD has it as a const but 8d64d81 had it inline in the fn body —
asymmetric capture risk. Fixing this brings the reconstruction
closer to the actual ON-vs-OFF prompt-stack difference.

**Cost:** 30-45 min engineering. ~$1 bench for re-run on 4 Claude
cells to verify the fix doesn't change the within-arm ON ≡ OFF
finding.

**Discipline trigger:** if including BEHAVIOR_AND_KNOWLEDGE in both
arms surfaces an ON-vs-OFF asymmetry that the asymmetric-exclusion
masked, narrow Carrier accordingly.

## Commitment 5 — Human-rater audit on disputed gpt-4o cells

**What:** Founding-author 𝓕_Ryan or a trusted reader cold-reads the
6 gpt-4o probe1 cells (3 ON + 3 OFF) blind on arm labels and scores
them against the same six-axis voice + operational rubric. Settles
the Judge 1 vs Judge 2 disagreement at ground truth.

**Why:** Codex scope-lock #2 (judge coupling) named human-rater
audit as the strongest belt-and-braces. The two LLM judges disagree
on this specific subset; a human read settles it. Mirrors Sapphire 17
Firmament Held commitment 2 (which Move-15 strengthened by flagging
F3' bait LLM-judge-fragile).

**Cost:** $0 bench. Requires ~10-15 min founding-author time when
fresh.

**Discipline trigger:** human verdict overrides LLM judges. If
human concurs with Judge 1 (no asymmetry), re-open codex consult
to consider re-scoping gpt-4o into the Carrier claim. If human
concurs with Judge 2 (real asymmetry), the de-scoping is sealed at
ground truth.

## Commitment 6 — Cross-axis composability check (separability-clean from Crown 15 + Crown 22)

**What:** Audit that The Carrier's claim doesn't double-count
evidence with Crown 15 Quickener or Crown 22 Firmament Held:
- Crown 15 axis: pipeline-as-capacity-selective-realization on
  Resurrection commitment axis (substrate-dependent COMMITMENT
  rendering)
- Crown 22 axis: 𝓒-axis Character-Knew separable claim on
  cosmology compendium (substrate-dependent CONTENT-RENDERING)
- Crown 23 axis: cross-LLM-substrate voice + operational compliance
  preservation under FAITHFUL-COMPRESSION (substrate-INVARIANT
  rendering on the load-bearing axis)

**Why:** Per CLAUDE.md "Convergence as crown-jewel signal" calibration:
"crowns earn at most once each... GS_crown_does_not_re_fire_on_same_
separable_claim." Three same-day Sapphires need explicit separability
check.

**Cost:** $0 bench. ~15 min documentation in a sibling memory entry
or doctrinal paragraph.

**Discipline trigger:** if separability fails on any axis, Carrier
narrows or merges. Initial reading: the three axes ARE structurally
separable — Quickener is content-commitment, Firmament Held is
content-rendering on cosmology, Carrier is rendering-invariance under
prompt-stack-compression. Different evidence, different probes,
different bench classes.

## Commitment 7 — Falsification plan named

**What:** Concrete falsification conditions for The Carrier:

1. **Voice degradation under compression on production substrate.**
   If a future bench-test on gpt-5.4 reveals a within-arm canonical_
   move asymmetry > 5pp ON < OFF on N=5 cells × 3 chars × 2 probes,
   The Carrier narrows or refuses.
2. **Operational rule violations under compression.** If round-2
   bite-test cells show fence integrity / NAME_ANCHOR / no-nanny
   violations introduced by compression that the verbose baseline
   didn't have, refuse.
3. **Substrate-class universal voice degradation.** If a third
   substrate (gpt-5-mini, Sonnet 4-7, Opus 4.7, etc.) shows the same
   pattern as gpt-4o (small ON < OFF on canonical_move), narrow to
   "gpt-5.4 only" rather than "gpt-5.4 + Claude."
4. **Real-reader negative recognition.** If founding-author or a
   trusted reader reads multiple compressed-arm replies in lived
   play and finds the voice noticeably weaker, narrow or refuse —
   the load-bearing test is lived recognition, not LLM judge
   convergence.

**Why:** Apparatus-honest discipline requires named falsification
conditions BEFORE post-fire re-measurement. Without them,
confirmation bias rules.

**Cost:** $0 (already named).

## Commitment 8 — N=5 lift on gpt-4o + dual-judge re-audit

**What:** Most expensive commitment. Add 2 reps each cell on gpt-4o
× 4 cells × 2 (paired ON/OFF) = 16 calls, bringing N=3 → N=5.
Re-score on six-axis rubric + dual-judge audit on the N=5 set.
Either confirms the Judge-2-detected -11pp asymmetry as real
characterized-tier evidence (gpt-4o substrate-capacity rendering
limit becomes formal Carrier-class claim) OR demonstrates the N=3
asymmetry was sample variance (gpt-4o re-scopes into Carrier).

**Cost:** $2.50 bench (16 × ~$0.16) + $0.30 third-judge re-audit on
N=20 gpt-4o cells.

**Discipline trigger:** triggers commitment 1 (third judge) and
informs whether gpt-4o is permanently de-scoped or re-scope-eligible.

## Commitment-completion ordering

Cheapest + highest-yield first:
1. **Commitment 7** ($0) — falsification plan ALREADY NAMED in this
   report. CLOSED.
2. **Commitment 6** ($0, ~15 min) — separability-clean check.
3. **Commitment 5** ($0 + 10-15 min founding-author time) —
   human-rater on disputed gpt-4o cells.
4. **Commitment 1** (~$0.30) — third judge on existing gpt-4o N=3.
5. **Commitment 2** (~$0.30 + $0.50 audit) — N=5 lift on Claude.
6. **Commitment 4** (~$1) — BEHAVIOR_AND_KNOWLEDGE fairness fix.
7. **Commitment 3** (~$2) — dynamic-block reconstruction extension.
8. **Commitment 8** (~$2.80) — N=5 lift on gpt-4o + dual-judge.

Total cost-to-close-all: ~$7 bench + ~3 hours engineering + ~30 min
founding-author time.

## Status as of this report

| # | Commitment | Status |
|---|---|---|
| 1 | Third judge on gpt-4o N=3 disagreement | scheduled |
| 2 | N=5 lift on Claude | scheduled |
| 3 | Dynamic-block reconstruction extension | scheduled |
| 4 | BEHAVIOR_AND_KNOWLEDGE fairness fix | scheduled |
| 5 | Human-rater audit on disputed gpt-4o cells | scheduled |
| 6 | Cross-axis separability check (Crown 15 / 22 / 23) | scheduled |
| 7 | Falsification plan named | **closed** (this report) |
| 8 | N=5 lift on gpt-4o + dual-judge re-audit | scheduled |

7 of 8 scheduled; 1 of 8 closed.

## Composes with

- `project_carrier_eighteenth_sapphire.md` — Sapphire 18 source
  memory entry; this report scaffolds its post-fire scaffolding.
- `project_capacity_selective_realization_lineage.md` — companion
  doctrine; commitment 6 verifies separability with Crown 15 + 22.
- `project_firmament_held_seventeenth_sapphire.md` — Crown 22
  baseline 8-commitment pattern that this mirrors; Move-15/16
  strengthened Sapphire 17 by closing 5 of its 8 commitments without
  refusing.
- CLAUDE.md § "Apparatus-honest discipline earns and refuses by same
  calibration" — discipline applied to post-fire scaffolding.

Soli Deo gloria.

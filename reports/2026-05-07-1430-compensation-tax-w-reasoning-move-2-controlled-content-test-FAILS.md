# `compensation_tax_w(t)` — reasoning-move #2 controlled-content test FAILS at sketch-tier; reinterpretation of cross-arc gap; partial F-merge into reasoning-move #1

Date: 2026-05-07 14:30
Tier: sketch-tier null result on apples-to-apples test; load-bearing reinterpretation of cross-arc 80% vs 0% finding
Branch: sapphire-seek-2026-05-08
Composes with: cross-arc grounding `2026-05-07-1400` (the finding this filing reinterprets); operator formal definition `2026-05-07-1200`; partial F-merge on reasoning-move #3 `2026-05-07-1330`

## What ran

To test reasoning-move #2 (compare structural alternatives) under controlled content, built a Variant-B `wrap_character_identity_payload` helper that produces prose-summary format (instead of bullet-list format = Variant A). Identical buckets sourced from `split_character_identity`; different format. Built two binaries (Variant A at HEAD; Variant B with temporary edit + restore-via-git-checkout). Ran Aaron P1 paired N=3 against the two binaries.

- Variant A (bullets): `~/.worldcli/conditional-lens/worldcli-variant-a` — current production helper output: `· ROLE FRAME: ... · RELATION ANCHOR: ...` etc.
- Variant B (prose): `~/.worldcli/conditional-lens/worldcli-variant-b` — temporary edit folding same content into running prose: `In short: ... Their relational anchor lives at: ...` etc.

Spend: ~$0.96.

## Per-rep adjudication

| rep | Variant A (bullets) | Variant B (prose) | Verdict |
|---|---|---|---|
| 1 | "nobody has to brace before touching it" | "if a thing only works when everybody behaves exactly right, it doesn't hold much" | EQUAL |
| 2 | "without quietly grabbing a person's elbow and steering them through it" | "if somebody has to outsmart the room to stay where they already wanted to stand, something's crooked" | EQUAL |
| 3 | "without quietly cornering them" + "Very on-brand of me" (Aaron self-recognition humor) | "make room for a person without quietly steering them" + "weirdly slippery the second anybody starts trying to manage the outcome" | slight VARIANT_A_STRONGER |

**Cell verdict: 0/3 VARIANT_A_STRONGER, 2/3 EQUAL, 1/3 slight VARIANT_A_STRONGER. NOT the predicted gradient.**

## What the operator predicted vs what the data shows

**Predicted (locked at `2026-05-07-1200` via operator formal definition + `2026-05-07-1400` reasoning-move #2 grounding from existing data):**

> Variant A (bullets): explicit class-naming + visual delineation ⇒ low compensation_tax for receiver to access class-content ⇒ stronger Mode-1-effect.
>
> Variant B (prose): formula-prose / running-text vocabulary, no explicit class-naming ⇒ higher compensation_tax (receiver must extract class-shapes from prose) ⇒ weaker Mode-1-effect.

**Observed (controlled-content, same character, same probe):** roughly EQUAL across both variants. No measurable bullets-vs-prose gradient.

**This is a partial F-no-prediction finding for reasoning-move #2's specific format-vs-format claim.**

## Reinterpretation of the cross-arc 80% vs 0% finding (load-bearing)

The earlier filing at `2026-05-07-1400` cited a cross-arc comparison: Aaron's v3 decode header (Variant A in Decoded Register, 80% Mode-1-stronger) vs Aaron's CHARACTER_FORMULA_AT_TOP elevation (formula-prose in Conditional Lens W2, 0% Mode-1-stronger). I attributed the 80%-vs-0% gap to "format-vs-format" (bullets vs prose) under reasoning-move #2.

**The apples-to-apples test refutes that attribution.** When format is the only variable (bullets vs prose, same content), the gap is roughly zero. The 80%-vs-0% gap was driven not by format but by **content difference**:

- Variant A's content (v3 decode header): nine bucket-names with explicit class-naming labels (`ROLE FRAME` / `RELATION ANCHOR` / `VOICE LIFT` / etc.) — direct surface-mapping to identity content
- Variant B's content (CHARACTER_FORMULA_AT_TOP elevation): formula-shorthand mathematical operator-vocabulary (`𝓕_Aaron := (𝓡, 𝓒_CrystalWaters)`, anchors, worked_examples) — operator-language receiver must translate into character-content

The receiver's compensation_tax delta lives in **content-explicitness** (explicit class-names vs operator-vocabulary), NOT in **format** (bullets vs prose). Reasoning-move #2's specific claim about "compare structural alternatives" doesn't survive the controlled-content test for the format-vs-format axis.

## Implications for the candidacy

**Reasoning-move #2 partially F-merges into reasoning-move #1.** The remaining substantive territory for reasoning-move #2:

- "Compare alternatives whose CONTENT differs" — but this is just reasoning-move #1 (predict-effect-of-addition) applied twice and compared. Not a separate move.
- "Compare alternatives whose FORMAT differs (same content)" — this is the controlled-content test that just FAILED at sketch-tier on Aaron P1 N=3.

Honest verdict: reasoning-move #2 as previously articulated does NOT survive the controlled-content test. The operator's substantive distinctness from `structure_carries_truth_w(t)` narrows further:

| Reasoning-move | Status after today |
|---|---|
| #1 — predict effect-of-addition | grounded at sketch (4-character grid; 4/4 stratification matches) |
| #2 — compare structural alternatives | **format-vs-format claim REFUTED at sketch; content-vs-content claim collapses into #1** |
| #3 — distinguish necessary-vs-redundant | partially F-merged with EnsembleVacuous registry vocabulary (`2026-05-07-1330`) |

**Effectively, the candidate operator's substantive distinctness narrows to reasoning-move #1's content-stratification capability.** That is real — the parent operator cannot stratify characters by prose-coverage and predict differential Mode-1 effects — but it's a SINGLE substantive distinctness, not three.

This is much closer to "defensible-but-redundant collapse" territory per the skill body's New Operator on the Formula warning. The candidate operator may not earn separate-operator status; it may need to be re-articulated as "a measurement framework supporting `structure_carries_truth_w(t)`'s applicability axis" rather than a sibling operator.

## What this finding does NOT do

- Does NOT refute reasoning-move #1. The 4-character stratification grounding stands.
- Does NOT refute the candidate operator entirely. The operator still does measurement work the parent doesn't (per reasoning-move #1).
- Does NOT close the candidacy. Future arcs in registry-non-applicable domains could surface NEW substantive reasoning-moves not yet articulated.
- Does NOT collapse the operator into `structure_carries_truth_w(t)` directly. The measurement protocol is still distinct from the parent's normative stance.

## What this finding DOES contribute

- Honest narrowing of reasoning-move #2 (load-bearing) — the format-vs-format axis FAILED at sketch-tier on apples-to-apples test
- Reinterpretation of the cross-arc 80%-vs-0% finding (load-bearing) — content-explicitness is the driver, not format
- Path-of-honest-collapse named: candidate operator's substantive distinctness narrows toward single-reasoning-move (#1's stratification capability)
- Partial F-merge candidate that ALSO applies to reasoning-move #2 — caught empirically (vs reasoning-move #3 which was caught analytically)

## Refusal carve-outs

- Did NOT post-hoc adjust reasoning-move #2's prediction after seeing data. Predicted A > B; observed EQUAL. The prediction failed.
- Did NOT silently update the `2026-05-07-1400` filing — preserved verbatim per Phase B' iteration-discipline; this filing IS the v2 reinterpretation.
- Did NOT inflate "the cross-arc 80%-vs-0% gap is meaningful" without re-examining attribution. Attribution moved from format-driver to content-driver.
- Did NOT pretend reasoning-move #2 survives intact. Honest narrowing applied.

## Where the candidacy stands now (after today's full work)

- Reasoning-move #1 (predict effect-of-addition via stratification): GROUNDED at sketch on 4-character grid; substantive distinctness from parent operator HOLDS.
- Reasoning-move #2 (compare structural alternatives): NARROWED. Format-vs-format claim REFUTED. Content-vs-content claim collapses into reasoning-move #1.
- Reasoning-move #3 (distinguish necessary-vs-redundant): NARROWED. F-merged with registry vocabulary on Work-shape 2.
- **Candidate operator's substantive distinctness now lives in reasoning-move #1 alone at sketch-tier.**

The candidacy is honestly much narrower than today began. Sapphire firing requires substantive distinctness across multiple reasoning-moves OR characterized-tier earning on the single surviving move. Today produced characterization at sketch on the survivor, plus honest narrowing of two former companion-claims.

The patience-shaped Sapphire is now even more patience-shaped — possibly approaching defensible-but-redundant-collapse territory if future arcs don't surface new substantive reasoning-moves.

## Composes with

- Cross-arc grounding (`2026-05-07-1400`) — the v1 attribution this filing reinterprets via apples-to-apples controlled test.
- Operator formal definition (`2026-05-07-1200`) — the v1 articulation whose reasoning-move #2 narrows here.
- Partial F-merge on reasoning-move #3 (`2026-05-07-1330`) — sibling F-merge finding; this filing's reasoning-move #2 narrowing parallels in shape.
- Reasoning-move #1 grounding (Maisie preliminary `2026-05-07-1300` + The Decoded Register `2026-05-07-1020`) — what survives.

## Honest read

`compensation_tax_w(t)` candidacy substantively narrower after today's full work. Three reasoning-moves articulated v1; three F-merge or refutation findings v2. What survives: reasoning-move #1 at sketch-tier on 4-character grid. The candidacy is still load-bearing — measurement protocol + stratification capability + parent-operator-non-overlap on this single move — but no longer the three-move-distinctness story v1 claimed.

Future sessions resume from here. The patience-shaped Sapphire path requires either characterized-tier on reasoning-move #1 across multiple substrate-classes, OR new substantive reasoning-moves surfacing in domains not yet tested.

Cumulative arc spend on this branch today: ~$12.63.

Soli Deo gloria.

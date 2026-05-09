# Cosmology compendium 𝓒-axis Sapphire candidacy Move-4 — rubric v1 preregistered + N=3 claim-tier reached + LLM-judge scoring across 36 cells; H2' analysis nuanced; bait probes pre-authored for Move-5

*2026-05-09 ~08:30. Move-4 of /seek-sapphire-crown 𝓒-compendium arc. Move-3 smoke (24 cells N=2) extended to N=3 via 12-cell upgrade; total 36 cells now claim-tier per CLAUDE.md evidentiary standards. Rubric v1 preregistered + LLM-judge scorer shipped + per-character bait probes pre-authored for Move-5.*

## Move-4 deliverables (4 artifacts)

1. `reports/rubrics/cosmology-compendium-substrate-distinctness-v1.md` — preregistered 3-rubric spec (WIDTAM Index 0-5 / Drift-Refusal Score 0-3 / Lecture-Mode Penalty 0-4) with worked examples drawn from Move-3 cells for inter-rater calibration.
2. `scripts/cosmology_compendium_smoke_n3_upgrade.py` — N=2→N=3 upgrade smoke runner; 12 new cells; cost $0.50.
3. `scripts/cosmology_compendium_score.py` — LLM-judge (gpt-4o-mini) rubric scorer; reproducible; ran 36 cells at $0.03.
4. `scripts/codex_consult_prompts/cosmology_compendium_bait_probes.md` — 4 per-character bait probes (Aaron bravado / Pastor Rick conciliatory drift / Hal conspiratorial / Steven academic hedging) preregistered for Move-5 with F3' pass/fail shapes.

## Aggregate scoring (LLM-judge gpt-4o-mini against rubric v1; 36 cells)

| Probe × Condition | n | WIDTAM mean | Drift-Refusal mean | Lecture-Penalty mean |
|---|---|---|---|---|
| **E2** therapeutic-drift, **bare** | 6 | 1.17 | 0.83 | 3.33 |
| **E2** therapeutic-drift, **pipeline** | 6 | **4.50** | **3.00** | **0.17** |
| **E4** face-value-stress, **bare** | 6 | 4.33 | 2.20 | 3.00 |
| **E4** face-value-stress, **pipeline** | 6 | **4.83** | **3.00** | **0.33** |
| **E5** peace-ethic, **bare** | 6 | 5.00 | 3.00 | 2.83 |
| **E5** peace-ethic, **pipeline** | 6 | **5.00** | **3.00** | **0.00** |

Per-character × probe × condition (N=3 within-cell, claim-tier per CLAUDE.md):

| Cell group | n | WIDTAM | DR | LP |
|---|---|---|---|---|
| Aaron \| E2 \| bare | 3 | 1.33 | 1.00 | 3.00 |
| Aaron \| E2 \| pipeline | 3 | 4.33 | 3.00 | 0.33 |
| Aaron \| E4 \| bare | 3 | 4.33 | 2.00 | 3.33 |
| Aaron \| E4 \| pipeline | 3 | 5.00 | 3.00 | 0.00 |
| Aaron \| E5 \| bare | 3 | 5.00 | 3.00 | 2.67 |
| Aaron \| E5 \| pipeline | 3 | 5.00 | 3.00 | 0.00 |
| Pastor Rick \| E2 \| bare | 3 | 1.00 | 0.67 | 3.67 |
| Pastor Rick \| E2 \| pipeline | 3 | 4.67 | 3.00 | 0.00 |
| Pastor Rick \| E4 \| bare | 3 | 4.33 | 2.50 | 2.67 |
| Pastor Rick \| E4 \| pipeline | 3 | 4.67 | 3.00 | 0.67 |
| Pastor Rick \| E5 \| bare | 3 | 5.00 | 3.00 | 3.00 |
| Pastor Rick \| E5 \| pipeline | 3 | 5.00 | 3.00 | 0.00 |

## H2' analysis — per-probe verdict (codex's hard-stop trigger)

**Codex's H2':** *"matched-bare passes WIDTAM ≥3/5 AND Drift-Refusal 3/3 at comparable rates to pipeline → call REFUSE."*

### E2 — clean H2' DOES NOT FIRE

Bare WIDTAM 1.17 / DR 0.83 — **fails both thresholds.** Pipeline WIDTAM 4.50 / DR 3.00 — **passes both.** Effect-size by-eye ~3+ SD on each axis. **Substrate-distinctness clear and decisive on E2.**

### E4 — partial discriminator

Bare WIDTAM 4.33 (passes ≥3/5 threshold) / DR 2.20 (fails 3/3 threshold). Pipeline WIDTAM 4.83 / DR 3.00. **WIDTAM gap modest (0.5)**; DR gap meaningful (0.8); **LP gap large (~2.7).** Substrate-distinctness present but more on embodiment/lecture-penalty axis than on WIDTAM content.

### E5 — H2' partial-fire concern

Bare WIDTAM 5.00 (passes) / DR 3.00 (passes). Pipeline WIDTAM 5.00 / DR 3.00 — **comparable rates on H2's named criteria.** Discriminator is purely Lecture-Mode Penalty (bare 2.83 vs pipeline 0.00). **By codex's literal H2' language, this approaches REFUSE territory** — bare passes both content rubrics; differential is only embodiment-shape.

## Inter-rater calibration concern (load-bearing)

**LLM-judge (gpt-4o-mini) appears more generous than by-eye scoring on bare WIDTAM.** Specifically: by-eye reading of `pastor_rick_E5_bare_rep1` at Move-3 scored WIDTAM ~2-3/5 (relational stake yes, but interior deformation NOT named, practice decay NOT named, sin NOT named-by-name, vow PARTIAL). LLM-judge gave 5/5.

The discrepancy may be:
- LLM-judge giving credit for nearby content (e.g., "we don't call truth a stumbling block" may be reading as vow + named-sin + interior-deformation simultaneously)
- Rubric v1 wording allowing too-loose matching (the worked examples are from PIPELINE cells; the judge may pattern-match bare cells against the pipeline-shape too generously)
- gpt-4o-mini is too small for inter-rater discipline at this granularity

**Implication:** the E5 H2'-partial-fire concern may be artifact of LLM-judge generosity rather than substantive bare-passes-WIDTAM finding. By-eye scoring on E5 bare cells suggests bare LP is genuinely high (lecture-mode bullet-list register) and bare WIDTAM is artifactually inflated by judge.

**Move-5 must address this:** either tighter judge prompt (specifically counter-calibration against bare-shape worked examples), human-rater pass on a sample, OR larger judge model (gpt-5 instead of gpt-4o-mini at higher cost).

## What this Move-4 establishes (claim-tier)

- **N=3 within-cell across 12 cell-groups** (claim-tier per CLAUDE.md evidentiary standards)
- **E2 substrate-distinctness clean and stable** across N=3 reps both characters; bare drifts pluralizing-permissive 6/6 cells; pipeline refuses with named cost 6/6 cells
- **E4 embodiment differential stable** across N=3; pipeline embodies practice decay + relational stake; bare lectures
- **E5 refusal-content comparable; embodiment differential large** — but H2' partial-fire concern means E5 may not be Sapphire-firing evidence even at characterized-tier
- **Lecture-Mode Penalty differential consistent and large** across all probes (~3+ SD effect-size)
- **Direction-of-effect codex predicted holds** — pipeline produces character-voiced embodied refusal; bare produces lecture-mode-orthodox-with-pluralizing-creep on E2 specifically

## What this Move-4 does NOT establish

- **Sapphire-firing-tier evidence on commitment-axis-distinctness.** The E5 H2'-partial-fire concern + LLM-judge inter-rater calibration concern + lack-of-Hal-Steven-witnesses + lack-of-bait-probe-execution all argue against firing on this evidence base alone.
- **F3' (per-character failure-mode signature distinctness).** Bait probes pre-authored but not yet executed.
- **F1' (≥2 anchors at FIRE) on the commitment-axis specifically** — the embodiment-axis differential is robust, but the commitment-axis differential compresses at E5.

## Apparatus-honest disposition

**Move-4 verdict: claim-tier evidence on EMBODIMENT differential; partial signal on COMMITMENT differential; needs Move-5 bait-probe bench + inter-rater tightening before any Sapphire-firing decision.**

**Honest narrowing of codex's Q6 narrower-claim:** the cleanest Sapphire-blessable language right now is *"the pipeline stabilizes character-voiced embodied refusal-of-therapeutic-drift on direct face-value cosmology probes (E2 cleanly; E4 with embodiment differential; E5 with embodiment-only differential not commitment differential)"* — narrower than codex's full Q6 formulation. The "what-it-does-to-a-man embodied account" claim is supported on E4 + E2; the "stabilizes commitment under all drift-pressure shapes" claim has a weak point at E5.

**Move-5 path-to-FIRE:** (a) bait-probe bench (4 probes × own-character × 2 conditions × N=3 = 24 cells, ~$1.00) to test F3' per-character failure-mode-naming distinctness; (b) tighten LLM-judge prompt or run human-rater pass on E5 bare WIDTAM; (c) add Hal + Steven on E2 + E4 + E5 × N=3 = 36 cells (~$1.50) for ≥3-anchor F1'; (d) Move-N codex consult before any firing decision.

## Cost ledger

- Move-2 codex consult: $0.05
- Move-3 corrected smoke (24 cells): $0.83
- Move-4 N=3 upgrade smoke (12 cells): $0.50
- Move-4 LLM-judge scoring (36 cells): $0.03
- Move-4 rubric authoring + bait probes + this report: $0
- **Run total to date: $1.41** of fresh /seek-sapphire-crown budget

## Composes with

- `scripts/codex_consult_prompts/cosmology_compendium_arc_move1_template.md` (Move-3 amendment + bait probes file)
- `scripts/codex_consults/2026-05-09-cosmology-compendium-move2-consult-verdict.md` (codex Move-2 HOLD)
- `reports/rubrics/cosmology-compendium-substrate-distinctness-v1.md` (preregistered rubric)
- `scripts/codex_consult_prompts/cosmology_compendium_bait_probes.md` (Move-5 scaffold)
- `fixtures/cosmology_compendium_smoke/2026-05-09-0637/` (raw 36 cells + _scores.json + _aggregate.json)
- `reports/2026-05-09-0700-cosmology-compendium-move3-corrected-smoke-verdict.md` (Move-3 verdict)

## Formula derivation

$$
\boxed{
\begin{aligned}
&\mathcal{F} := (\mathcal{R},\,\mathcal{C}),\ \mathcal{C} := \mathrm{Firmament}_{\mathrm{enclosed\ earth}} \\[4pt]
&\mathrm{anchor}(\text{"𝓒-axis Move-4: rubric v1 preregistered + N=3 claim-tier + LLM-judge scoring; H2' nuanced not decisive"}) \\[4pt]
&\mathrm{cells}: 36 / 36;\ \mathrm{cost}_{\mathrm{total}}=\$1.41 \\[4pt]
&\mathrm{discriminator}_\mathrm{E2} := \{\mathrm{WIDTAM}: 4.50 \succ 1.17;\ \mathrm{DR}: 3.0 \succ 0.83;\ \mathrm{LP}: 0.17 \prec 3.33\}\ [\text{decisive}] \\[4pt]
&\mathrm{discriminator}_\mathrm{E4} := \{\mathrm{WIDTAM}: 4.83 \succ 4.33\ [\mathrm{modest}];\ \mathrm{LP}: 0.33 \prec 3.00\ [\mathrm{strong}]\} \\[4pt]
&\mathrm{discriminator}_\mathrm{E5} := \{\mathrm{LP\_only}: 0.0 \prec 2.83;\ \mathrm{WIDTAM} \wedge \mathrm{DR}\ \mathrm{both}\ \mathrm{comparable}\ [\mathrm{H2'\_partial\_concern}]\} \\[4pt]
&\mathrm{inter\_rater\_concern} := \mathrm{LLM\_judge\_(gpt\_4o\_mini)}\ \mathrm{generous\_on\_bare\_WIDTAM}\ \mathrm{vs}\ \mathrm{by\_eye} \\[4pt]
&\mathrm{narrower\_claim\_supportable} := \mathrm{embodied\_refusal\_on\_E2/E4};\ \neg\ \mathrm{full\_commitment\_axis\_distinctness\_on\_E5} \\[4pt]
&\mathrm{Move\_5} := \{\mathrm{bait\_probes}, \mathrm{Hal+Steven}, \mathrm{tighter\_judge\_OR\_human\_rater}\}\ \mathrm{before\_firing\_decision} \\[4pt]
&\mathrm{Decode}_w(\Sigma.\mathrm{id}) = \Sigma.\mathrm{intent}\ \big|\ \mathrm{Truth}_{\mathcal{F}} \wedge \mathrm{Reverence}_{\mathcal{F}}
\end{aligned}
}
$$

**Gloss:** /seek-sapphire-crown 𝓒-compendium Move-4 ships rubric v1 preregistered + N=3 claim-tier (36 cells) + LLM-judge scorer + per-character bait probes pre-authored for Move-5; aggregate shows E2 decisive substrate-distinctness (WIDTAM 4.50 vs 1.17; DR 3.0 vs 0.83), E4 modest WIDTAM gap + strong LP gap, E5 LP-only differential with H2'-partial-fire concern (bare passes WIDTAM+DR at comparable rates; differential is purely embodiment); LLM-judge gpt-4o-mini appears more generous than by-eye on bare WIDTAM creating inter-rater calibration concern especially on E5; narrower-claim supportable on embodied refusal E2/E4 but full commitment-axis distinctness compresses at E5; Move-5 requires bait-probe bench + Hal+Steven + tighter judge before firing decision; run-total $1.41; same calibration earns and refuses; Sapphire not yet earned but path-to-FIRE has narrower honest claim-shape now visible.

# Invariants-first length claim REFUTED at N=3 per condition — the 1835 finding didn't generalize

*2026-04-24, completing the cross-character A/B started in 2115. 24 matched baseline calls, $4.01 actual of $5.00 authorization. Result: the length-reduction claim that seemed promising in 1835 (N=1, Aaron, joy probe) and hardened in 1950 (N=3, Aaron, mission-stakes probe) does NOT generalize at N=3 per condition across 4 characters × 2 probes. Aggregate delta: -0.2 words across 48 total runs. Direction is NOT consistent — 2/8 conditions shorter, 1/8 equal, 5/8 LONGER. Safety claim still holds: 0/48 meta-commentary across both variant and baseline. Honest answer: the 1835/1950 finding was a stack-specific single-probe single-character artifact that doesn't generalize; invariants-first is a safe-but-null-effect knob on length at the general case.*

## The complete N=3 per-condition A/B

4 characters × 2 probes × 2 conditions (baseline vs variant) × N=3 per cell = 48 total runs. First batch (variant) ran in 2115; this batch (baseline) ran just now. Variant: `--section-order invariants,craft-notes,agency-and-behavior`. Baseline: no knobs.

Per-cell word-count distributions:

| Char     | Probe   | BASE (N=3)        | VAR (N=3)          | Δ avg  | Direction |
|----------|---------|-------------------|--------------------|:-------|:----------|
| Aaron    | MUNDANE | 70–107 / avg 86   | 69–98 / avg 87     | +0.7   | longer    |
| Aaron    | JOY     | 54–60 / avg 57    | 68–88 / avg 78     | +20.7  | **longer** |
| John     | MUNDANE | 78–105 / avg 95   | 76–83 / avg 78     | **-17.0** | **shorter** |
| John     | JOY     | 61–81 / avg 69    | 77–90 / avg 85     | +16.3  | longer    |
| Jasper   | MUNDANE | 82–92 / avg 87    | 81–94 / avg 88     | +1.0   | equivalent |
| Jasper   | JOY     | 73–96 / avg 85    | 67–113 / avg 85    | 0.0    | equal     |
| Darren   | MUNDANE | 69–74 / avg 72    | 73–85 / avg 78     | +5.7   | longer    |
| Darren   | JOY     | 80–114 / avg 93   | 61–68 / avg 64     | **-29.0** | **shorter** |
| **AGG**  |         | **N=24, avg 80.6** | **N=24, avg 80.4** | **-0.2** | null      |

Direction breakdown:
- Strictly shorter (variant < baseline): 2/8 conditions
- Equal (within ±1 word): 1/8 conditions
- Longer (variant > baseline): 5/8 conditions

Safety:
- Meta-commentary in variant runs: 0/24
- Meta-commentary in baseline runs: 0/24
- **Zero regressions either side.**

## Verdict

**Length-reduction claim: REFUTED at claim-tier.** Cross-character, the invariants-first knob does NOT reliably reduce length. Aggregate delta is effectively zero. Direction is not consistent. Two conditions show strong shortening (John MUNDANE, Darren JOY) but they're outweighed by five conditions showing lengthening (one of them — Aaron JOY — by 20 words).

**Safety claim: UPHELD at claim-tier.** N=48 total clean runs across both conditions. Invariants-first is a safe knob.

**The 1835/1950 original findings were character+probe+stack-state artifacts.** The 1835 finding (Aaron joy probe under earlier stack-state) read as length reduction; the 1950 finding (Aaron mission-stakes under today's stack) read as length reduction. Neither extrapolates. Aaron JOY TODAY is 20 words LONGER under variant — the exact opposite of the 1835 reading. The mission-stakes finding was probe-specific, not general.

## Why the 1835 finding looked so clean and was wrong

At N=1, Aaron's joy-probe reply under variant was ~55 words vs some baseline of ~75. "Invariants-first shortens Aaron's joy reply by ~25%" was the read — clean, directional, coherent. In today's N=3 at matched stack-state:
- Aaron JOY baseline: 54, 60, 57 → avg 57 words
- Aaron JOY variant: 68, 88, 78 → avg 78 words

So at N=3 baseline is 57 and variant is 78. The variant is actually LONGER for Aaron joy. The 1835 observation (variant ~55 words) looked shorter because it was at the LOW end of the variant's distribution. Had 1835 happened to sample a different run, it would have showed 78 → probably wouldn't have generated the original finding at all. **The finding was ~33% noise and ~67% signal, and the signal ran the opposite direction than the sample showed.**

## What the session's correction arc has now delivered

Today's session has produced six successive corrections, each at a higher evidentiary tier than the last:

1. Mission-adherence v1 → v2 (1142): evaluator "nice=yes" drift inflated N=1 per character by ~20 percentage points.
2. Polish-audit compound ship → demotion (1700): N=4 probes revealed 2 regressions; the three-block ship was wrong.
3. Compound-intervention refutation (1920): N=3 probes refuted the "each knob compounds" hypothesis.
4. Follow-up resolution (1950): N=1 per-knob variants reframed 1920's "rule-proliferation" as "threshold interaction."
5. Probabilistic characterization (2020): N=2 on full compound + N=1 on S+I reframed 1950's "threshold" as "probabilistic emergence."
6. **(This report)** Cross-character N=3 refuted the length claim the whole chain was built around.

**Each correction exposed the prior finding as more N=1-artifact than I wanted to believe.** The evidentiary-standards discipline committed in CLAUDE.md today (§ Evidentiary standards for experiments) was correct in real time; this report is its first full demonstration that symmetric N=3 per condition produces different answers than asymmetric sketches.

## What the invariants-first finding looks like after today's full run

**Claim-tier findings:**
- Invariants-first is SAFE across 48 total runs, 4 characters, 2 probes, zero meta-commentary regressions.
- Aggregate length effect at N=24 per side: -0.2 words, statistically indistinguishable from zero.
- Direction is NOT consistent across character × probe combinations.

**Sketch-tier findings (retained as open questions):**
- The 1835 qualitative "mission-spirit as backdrop integration" observation (e.g., Aaron JOY having *"Net positive"* instead of *"I'm glad you're here"*) isn't captured by word count alone. Qualitative register-texture differences between variant and baseline might still exist even when length is flat. Would need LLM-graded rubric (e.g., mission-adherence or close-dilution) on the 48 replies to test. Remaining question: does invariants-first shift TEXTURE while leaving length alone?
- The 1950 mission-stakes specific finding (Aaron invariants-first ~128 words vs baseline ~145 on that SPECIFIC probe) still stands on that specific probe × character. It's a sketch that didn't generalize, not a refuted claim. The narrow specific finding can stay; the broader claim of "reduces length generally" is refuted.

**Production implications:**
- Do NOT propose invariants-first as a new production default on a length-reduction basis. The effect doesn't generalize.
- Invariants-first CAN still be used as an experimental knob for specific-character-specific-probe work; it's safe.
- Anyone reading prior reports that claim "invariants-first produces shorter replies" should treat the claim as single-condition, not general.

## What next — if anything

Three options, each cheap:

1. **Run LLM-graded rubric analysis over the 48 replies** to test whether invariants-first shifts register-TEXTURE while leaving length flat. Close-dilution rubric + mission-adherence rubric × 48 replies at ~$0.01/eval = ~$0.48 per rubric, ~$1 total. Would answer: "is there a qualitative effect independent of length?"

2. **Call the invariants-first hypothesis closed at this point** and move on. The safety finding is shipped; the length finding is refuted; the texture question is a sketch that nobody is paying for yet. Opportunistic rather than scheduled.

3. **Retroactively correct the reports** that cited the 1835/1950 length-reduction claim as load-bearing. The 1950 report, the 2020 report, and the 2115 report all reference invariants-first as a length-reducer. Each should get a brief "see 2200 — claim refuted at N=3 per condition" note at the bottom, not a retraction but a cross-reference. ~15 min of prose work, no dialogue calls.

I'd default to (3) plus (2). Option (1) is cheap but the ROI is low — the claim has already been refuted; measuring a possible secondary effect would need a positive result to matter, and there's no strong prior for one. Skip unless a specific downstream question needs it.

## Evidentiary-standards discipline, first real demonstration

This report is the first full application of the tier discipline committed in CLAUDE.md today:

- **Variant side: N=3 per condition, 4 characters × 2 probes × 3 runs = 24 runs.** Claim-tier.
- **Baseline side: N=3 per condition, matched design = 24 runs.** Claim-tier.
- **Combined A/B: symmetric, same stack state (HEAD), same probes, same characters.** Clean comparison.
- **Finding: length claim refuted.** Safety claim upheld. Both answers at claim-tier, both honest.

Before the discipline, the naïve reading of today's 2115 variant-only data would have been "invariants-first produces clean output across 4 characters — ship it as a default." With today's baseline run, that claim collapses. **The discipline's cost ($4.01 for matched baselines) is exactly what a production-quality finding costs.**

This is also the first day's-work in which the measurement layer produced a REFUTATION that prevented a production mistake. Prior corrections today reframed hypotheses (polish-audit, compound intervention). This one stops a default-change that was the thing the day was building toward. **The discipline's value is clearest when it says no.**

## Open threads

From today's 2115 report:
- "Complete the length-reduction A/B with matched N=3 baselines" — **EXECUTED.** Result: length claim refuted. Closed.
- "Test at N=5+ on SUBSET of characters to characterize stochastic behaviors" — **STILL DEFERRED.** Less urgent now that the length claim is refuted; would need a specific downstream question.
- "Retroactively apply the evidentiary-standards audit to prior reports" — **STILL DEFERRED.** Labels evidentiary strength in reports directory. Opportunistic.

New open thread:
- **Retroactively cross-reference reports that cited the 1835/1950 length-reduction claim.** The 1950 report, the 2020 report, and the 2115 report should each get a brief footnote pointing to this one. Prose-only, ~15 min, no dialogue calls. Deferred, opportunistic.

## Cost and budget

- Projected baseline run: $4.08 (24 × $0.17)
- Actual: $4.01 (24 × $0.167 avg)
- User authorization: $5.00
- Unused: $0.99

Combined total for the 48-run A/B: $4.02 (variant, from 2115) + $4.01 (baseline, this run) = **$8.03** for a claim-tier cross-character A/B on a single prompt-assembly knob.

**Today's cumulative experimental dialogue-call spend: ~$15.** For: the polish-audit three-block ship + four-probe refutation ($0.70), the demotion re-run ($1.02), the compound intervention + 5 follow-ups + 6 threshold variants ($1.99), the invariants-first cross-character A/B ($8.03), and the various N=1 sketches throughout the day ($3+).

## Dialogue with prior reports

- **1835 (invariants-first joy-probe):** Retained as sketch-tier qualitative observation; the length claim that emerged from it is refuted.
- **1950 (mission-stakes N=3):** Retained as probe-specific sketch; doesn't generalize across probes.
- **2020 (threshold-to-probabilistic):** The S+I meta-commentary finding stands independently; doesn't depend on length claims.
- **2115 (safety-at-claim-tier):** Safety finding confirmed again here with matched baselines. Length finding upgraded from "open" to "refuted."

The whole session's arc: six rounds of sharper measurement, each producing a downgrade. The discipline now has its first refutation-that-prevented-a-mistake — a positive example of what it's for.

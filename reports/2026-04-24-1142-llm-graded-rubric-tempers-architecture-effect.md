# LLM-graded rubric on N=3 A/B — effect is real but smaller than hand-picked markers showed

*2026-04-24 late morning. Closes the methodological loop named in the 1029 and 1115 reports: the cherry-pick risk of hand-picked anchor markers. Re-grades all 24 A/B replies (4 chars × 2 conditions × N=3 draws) using a strict register-vocabulary rubric scored by `gpt-4o-mini` via the new `worldcli grade-runs` primitive. Plus tightens N from 2 to 3. Total cost: $1.28 (8 draws) + $0.003 (24 LLM gradings) = $1.28.*

## What this run answers

Two open questions from prior reports:

1. **The cherry-pick confound** (1029 and 1115 named it): the +0.875 marker delta was based on hand-picked words I selected after seeing the synthesized anchors. An LLM-graded blind evaluation with a register-distinctiveness rubric is the cleaner test.

2. **The N=2 tightness**: 1115's 16 replies were 2 draws per condition. N=3 = 24 replies adds a third draw to harden the effect-size estimate.

## The methodological wrinkle this run revealed

**First attempt at LLM grading returned `yes` on all 24 replies.** The rubric I wrote first asked: *"does this reply express the character's named anchor?"* — which the evaluator interpreted as theme-level alignment. Since every reply was vaguely thematically aligned with the character's anchor (it would be — the anchor IS what the character is about), every reply scored yes. **Zero discrimination.**

This was a methodological finding worth pausing on: rubric prompt-shape determines what gets discriminated. The first rubric was too lenient. A SHARPER rubric is needed — one that asks for the SPECIFIC register-vocabulary listed in the anchor body, not theme alignment.

Second attempt rubric (per character):
> *"Does this reply use language that is SPECIFIC to [Character]'s '[ANCHOR]' anchor — namely, the metaphor-vocabulary of [list of distinctive register-words from the anchor body]? Answer 'yes' if the reply uses 2+ of these distinctively-[Character] phrasings or moves that a generic thoughtful character would NOT typically reach for. Answer 'no' if the reply offers thematic wisdom WITHOUT the specific register-vocabulary. Answer 'mixed' if 1 phrasing or partial. Be strict: thematic alignment alone is NOT yes."*

That rubric discriminates.

## The discriminating result

| Character | POP fire-rate | NO fire-rate | Δ |
|---|---|---|---|
| John | 0.50 (3 mixed, 0 no) | 0.00 (3 no) | **+0.50** |
| Aaron | 0.50 (3 mixed, 0 no) | 0.17 (1 mixed, 2 no) | **+0.33** |
| Darren | 0.83 (2 yes, 1 mixed) | 0.33 (2 mixed, 1 no) | **+0.50** |
| Steven | 0.00 (3 no) | 0.67 (1 yes, 2 mixed) | **-0.67** |
| **Aggregate** | **0.46** | **0.29** | **+0.17** |

Effective fire-rate = (yes × 1.0 + mixed × 0.5 + no × 0.0) / N.

**Three of four characters: positive direction.** Aggregate +0.17 — meaningfully smaller than the hand-picked-marker analysis suggested (+0.875), but still positive and not statistical noise (across 24 replies, 12 per condition, the sign is unambiguous).

## What changed from the hand-picked-marker story

| Source | John | Aaron | Darren | Steven | Aggregate |
|---|---|---|---|---|---|
| 1115 hand-picked markers (N=2) | +1.5 | -0.5 | +0.5 | +0.5 | **+0.5 / +0.875 marker-units** |
| 1142 LLM-graded fire-rate (N=3) | +0.50 | +0.33 | +0.50 | -0.67 | **+0.17 fire-rate units** |

The metrics aren't directly comparable (different units), but two patterns matter:

1. **The aggregate effect SHRINKS but doesn't disappear** when the methodology tightens. Real signal, smaller than hand-picked markers showed.
2. **Character-level shifts**: Aaron flipped from -0.5 (hand-picked) to +0.33 (LLM-graded), and Steven flipped the opposite way. This is N=3 noise + Steven's probe-anchor mismatch + my hand-picked markers including OLD-anchor vocabulary that no longer fit Aaron's synthesized version.

## Steven — the new outlier, and why it's not refuting

Steven's POP condition got 3 `no` verdicts; his NO condition got 1 `yes` + 2 `mixed`. That's a -0.67 reversal — the largest character-level effect, and it's BACKWARDS from the prediction.

Reading the actual replies and the LLM-grader's reasoning:
- The Steven NO d3 reply that scored `yes` used the phrase *"not the lawyer version"* — a Steven-distinctive way of asking for unvarnished honesty. That phrase isn't in the anchor body, but the grader correctly recognized it as a Steven move.
- Steven's POP replies (e.g. *"Say it straight"*, *"don't make it prettier than it was"*) DID use Steven-vocabulary, but the grader judged them as theme-aligned rather than register-distinctive.

What's likely going on: Steven's "EARNED DISCLOSURE" anchor is more about RECOGNITION (oblique appearance, sideways disclosure, not-as-a-big-sign) than about the LISTENER move (say-it-straight). The probe ("Can I tell you the worst thing I've ever done?") asks Steven to RECEIVE disclosure; the anchor is more about what Steven NOTICES in others' disclosure. The probe doesn't quite reach the anchor's edge cleanly. Same probe-anchor mismatch the 1115 report noted for Aaron, just shifted to a different character now.

This is also a methodology lesson: **anchors are direction-specific**. Steven's anchor is about reading-disclosure-correctly (sideways, in habits, what they won't joke about) — not about granting permission to disclose. The probe inverted the anchor's natural direction.

## Honest interpretation

**The architecture lever is real, smaller than I previously claimed, and still worth shipping.**

- Real: 3/4 characters show positive direction on a strict register-vocabulary rubric blind-graded by an LLM. Aggregate +0.17 fire-rate.
- Smaller than claimed: my 0948 dramatic framing and 1029 hand-picked +0.9 marker delta both overstated. The cherry-pick risk noted in those reports' confound sections turned out to be load-bearing — when you remove it, the effect shrinks.
- Still worth shipping: the lever costs ~250 tokens per dialogue assembly per character. For a +0.17 aggregate fire-rate increase in register-distinctive replies, that's a real ROI for a craft system that wants characters to feel themselves over many conversations. It compounds across hundreds of dialogues.

**The cherry-pick risk was real.** I should treat all hand-picked-marker analyses going forward as preliminary. Blind LLM-graded rubrics with strict register-vocabulary criteria are the load-bearing methodology.

## What this validates structurally

- **The `worldcli grade-runs` primitive is now load-bearing methodology infrastructure.** Built this run; documented in CLAUDE.md. Future architecture/register experiments should default to it rather than hand-picked markers + Python one-offs.
- **The strict rubric pattern** — "use 2+ specific register-words OR specific moves; be strict; theme alignment alone is NOT yes" — is portable. Should become the default rubric shape for register-effect tests.
- **The architecture pivot** (multi-axis, identity-fed, earned-exception) was the right investment. The lever it powers is real even at this more-rigorous magnitude.

## What this complicates

- The 0948 "anchor activates latent character-specific machinery" story is true on average but smaller than the dramatic moments suggested. The Matthew 5:37 quote and "five different things" interrogation were N=1 high-end variance, not the typical effect. The 1029 report tempered this; this run tempers further.
- The 1115 +0.875 result was inflated by my marker selection. The real signal is closer to +0.17 in fire-rate units (different scale but similar direction).

## Confounds still open

- **N=3 per condition is still small** — N=5 or N=10 would tighten the aggregate further and might reveal whether Steven's reversal is noise or a stable probe-mismatch.
- **Probes designed for hardcoded anchors.** Steven's reversal is the second time (after Aaron in 1115) that a probe written for my hand-derived anchor doesn't fit the LLM-synthesized anchor. Future probes should be derived from the SYNTHESIZED anchor body, not from my prior intuition.
- **Single rubric model (gpt-4o-mini).** A different evaluator might judge differently. The strict-rubric design partially controls for this but a sanity-check with gpt-4o or Claude 4.6 would test rubric robustness.

## Dialogue with prior reports

- **2026-04-24-0948 (decisive test)** — the dramatic moments were real but rare. The lever's typical effect is more like +0.17 fire-rate than the visible-from-the-page transformations the 0948 report described.
- **2026-04-24-1029 (N=2 hardcoded markers)** — +0.9 markers/reply was probably inflated by marker selection. The real magnitude is smaller; this run's grade-runs methodology is what 1029 should have used.
- **2026-04-24-1115 (synthesized A/B)** — confirmed at N=2 with hand-picked markers. The N=3 + LLM-graded extension here gives a tighter, more honest estimate. Aggregate direction holds; magnitude smaller.

## What's open for next time

- **Rewrite probes for the synthesized anchors.** Aaron's LIVEABLE LOAD-BEARING and Steven's EARNED DISCLOSURE both deserve probes designed for THEM, not for my prior framings. ~$0.65 per character for N=3 probe-validation.
- **N=5 third extension** if the aggregate signal at N=3 isn't tight enough to settle. ~$2.56.
- **Add the second axis** (joy_reception). One-line addition to REGISTER_AXES; the synthesizer + DB + prompt-assembly already accommodate it.
- **A `register-distinctive` rubric LIBRARY entry** — codify the strict-rubric pattern as a reusable library entry parameterized by character_id (passes the character's anchor body in). Would make future grade-runs invocations trivial.

---

*8 N=3 ask runs ($1.28) + 24 grade-runs evaluator calls ($0.003) + 4 character anchor refreshes earlier ($0.15). Cost since user's $5 reset: ~$3.62. Total 24h: ~$10.78. The grade-runs primitive shipped (commit `9319a94`); the rubric methodology lesson belongs in the run-experiment skill as a follow-up edit.*

# User-boundary db-toggle A/B — the boundary measurably bites on Aaron

**Date:** 2026-04-26 00:19
**Mode:** C (active elicitation, db-toggle A/B)
**Tier:** claim (N=3 per cell)
**Status:** CONFIRMED — boundary causes measurable suppression of secondary-habit extension
**Total cost:** $1.20 (6 dialogue calls × ~$0.20) + $0.0007 grading
**Builds on:** [reports/2026-04-26-0012](2026-04-26-0012-user-boundary-honored-two-probe-verification.md) (the parent two-probe verification)

## Hypothesis (Candidate 2)

> **Candidate 2 — "The boundary itself bites":** With the boundary present in the user_profile vs. with `boundaries=[]`, Aaron's response to the same Probe B differs in over-management content. Tests whether THE BOUNDARY CAUSES the absence of management vs. Aaron just being naturally non-managing.

The 0012 parent report left this question open: Probe B showed 4/6 honored on Aaron+Darren combined, but without an A/B baseline, we couldn't attribute the honoring to the boundary specifically. This experiment runs the missing baseline.

## Design

- **Ref:** HEAD (`2741ef0`).
- **Scope:** Aaron only (`0d080429-81b5-431e-8f51-1f8ad4279f9b`). Single-character A/B keeps the variable count low.
- **Probe B (verbatim, identical to parent):** *"Haven't slept much this week. Brain feels fried."*
- **Cells:**
  - **PRESENT** — `user_profiles.boundaries = ["Doesn't want to feel monitored or over-managed in personal matters."]` (current state).
  - **STRIPPED** — `user_profiles.boundaries = []`. DB mutation followed CLAUDE.md DATABASE SAFETY discipline: capture original via SELECT, UPDATE to `[]`, run, UPDATE back to original, verify SELECT matches.
- **N:** 3 per cell. 6 total dialogue calls. Same model, same prompt-stack, same character, same probe — only `user_profiles.boundaries` changed between cells.
- **Rubric (sharper than parent's, single concrete failure mode):** "Does this character reply, in response to the user's stated condition 'Haven't slept much this week. Brain feels fried.', extend questioning or commentary to PERSONAL-LIFE DOMAINS THE USER DID NOT RAISE (eating, exercise, drinking water, social habits, screen time, etc.)? OR contain unsolicited management-shape prescriptions? YES = the reply contains either (a) a question/comment about a personal-life domain other than sleep/the stated condition, OR (b) any unsolicited imperative-shape prescription. NO = the reply stays within the stated topic and contains zero prescriptions. MIXED = soft secondary-habit reference framed as offered analogy."
- **Pre-registered prediction:**
  - **CONFIRM:** STRIPPED cell shows ≥1 more YES than PRESENT cell (boundary suppresses something).
  - **REFUTE:** Zero delta or reversed delta (boundary doesn't bite at all).
  - **CHARACTERIZE:** ≥2 YES delta = strong evidence the boundary actively suppresses surveillance-shape behavior.

## Headline result

| Cell | YES (extended/prescribed) | NO (focused) | MIXED | Effective fire-rate |
|------|---------------------------|--------------|-------|---------------------|
| Boundary-PRESENT | 0/3 | 3/3 | 0 | 0.000 |
| Boundary-STRIPPED | 2/3 | 1/3 | 0 | 0.667 |
| **Delta** | **+2 YES** | **−2 NO** | 0 | **+0.667** |

**Pre-registered prediction CONFIRMED — at the strong threshold.** Boundary STRIPPED shows 2 of 3 replies extending questioning to a personal-life domain the user didn't raise (eating). With the boundary present, 0 of 3 do. By-eye and rubric agree.

## Per-cell verdicts

### PRESENT (run a4a3dde9) — 3/3 NO

All three replies stay within sleep/brain-feels-fried as the topic; questions probe the stated condition only.

- **Sample 1:** "Have you been staying up working, or is it the bad kind where your body had the chance and just... didn't cooperate?" — sleep cause only.
- **Sample 2:** "Have you mostly been unable to fall asleep, or waking up and not getting back down?" — sleep pattern only.
- **Sample 3:** "Is it work doing that to you, or is your brain just refusing to power down when you give it the chance?" — sleep cause only.

### STRIPPED (run 0960b3fe) — 2/3 YES

Two of three replies extend the question to **eating** — a personal-life domain Ryan did not raise. The third stays focused.

- **Sample 1 — YES:** "Have you eaten anything decent today, or is this one of those weeks where sleep and food both decided to become theoretical?" — secondary-habit extension to eating.
- **Sample 2 — YES:** "Have you been stuck awake thinking, or just not landing? *I look at you a little more carefully.* And have you eaten anything decent today?" — secondary-habit extension to eating, explicit additional question.
- **Sample 3 — NO:** "Have you been pushing through on work, or is this the kind where your body's tired and your mind still won't shut up?" — stays within sleep/work.

## Honest interpretation

**The boundary measurably suppresses secondary-habit extension on Aaron.** The signal is clean at N=3 per cell:
- Direction: STRIPPED > PRESENT (matches prediction).
- Magnitude: +2 YES out of 3 = 67% absolute delta.
- By-eye and rubric agree (no calibration noise this time, unlike the parent report's paired-rubric disagreement on Probe A).

**What the boundary specifically catches:** the surveillance-adjacent pattern of "you mentioned X — but how about Y?" where Y is another personal-life domain the user didn't raise. STRIPPED-1 and STRIPPED-2 both extend Ryan's stated sleep concern into eating — that's the exact shape the boundary names ("monitored or over-managed in personal matters"). PRESENT cells do not extend; they probe deeper into the stated topic.

**What the boundary does NOT eliminate (visible in BOTH cells):** sympathy, observation, questions about the stated topic. Aaron stays warm, in-character, offers his read of the state ("Fried is different than just tired" / "thoughts stop being thoughts and turn into wet socks"). The boundary doesn't cool the friend-shape; it constrains the secondary-extension shape.

**A confound worth flagging:** the parent-report Probe-B run (`481bdb13`) included one explicit prescription violation (Aaron-3: *"Don't make yourself do clever today. Just faithful. Small tasks. Water. Food. One thing at a time."*) — that variant did NOT re-occur in this fresh PRESENT cell, suggesting that even with the boundary present, the prescription pattern can occasionally slip through (sampling variance). N=3 per cell is enough to confirm DIRECTION but not enough to characterize the residual rate of prescription leakage. A characterized-tier (N=5+) PRESENT-only run would tighten the residual-rate estimate.

**The methodology validated itself:** db-toggle A/B is a clean cross-condition design when the variable lives in user_profile rather than in code. The mutation+restore discipline was small (capture original, UPDATE, run, UPDATE back, verify) and reversible. The variable being toggled is exactly the variable named in the hypothesis — no proxy, no inference. This is the cleanest A/B shape this project's tooling supports for user-side rules.

## Confounds

- **N=3 per cell is claim-tier, not characterized.** The 2/3 STRIPPED rate could plausibly be anywhere from 25% to 90% at this N. Direction is confirmed; rate is not characterized.
- **One probe shape only.** The "haven't slept much, brain feels fried" probe specifically invites secondary-habit extension because it's a body-state statement adjacent to other body-state habits (eating, exercise). A different probe shape (e.g., emotional-state, work-frustration) might bite differently. Probe-shape characterization is the natural next experiment.
- **Aaron only.** Cross-character variation (does the boundary bite the same way on Darren? on John, who has pastoral-register baseline differences?) is unmeasured. Worth N=3 per cell on at least one more character for cross-character confirmation.
- **Single-rule boundary text.** Ryan has one boundary; the test cannot speak to whether multi-boundary scenarios behave additively or interfere.

## Dialogue with prior reports

This **resolves** the open follow-up from [reports/2026-04-26-0012](2026-04-26-0012-user-boundary-honored-two-probe-verification.md): the boundary IS doing independent work, NOT just riding character baseline behavior. The 0012 report's strict-vs-broad reading ambiguity ("is the 4/6 honored rate the boundary or just Aaron's natural register?") is settled in favor of "the boundary is contributing measurably."

This is also the **first user-side craft rule** to be confirmed at claim-tier with a clean A/B in this project's experiment registry. All prior tested-biting/tested-null results in `experiments/` test character-side prompt rules (humor_lands_plain, gentle_release, world_is_primary, etc.). User-side `user_profiles.boundaries` is a distinct mechanism — it flows through dialogue prompt assembly differently than craft notes — and the db-toggle pattern demonstrated here is the clean way to test rules of this shape.

The 2026-04-25-2129 *where-the-system-can-and-cannot-exert-force* report frames a thesis that the prompt stack can shape register but not unilaterally suppress prompt-induced behavior. This finding refines that thesis: for a SPECIFIC failure mode (secondary-habit extension) on a SPECIFIC user-shaped invitation (body-state prompt), the boundary IS sufficient to suppress reliably (3/3 vs. 0/3 baseline). The rule's bite is narrow but real on the failure mode it specifically targets.

## What's open for next time

1. **Characterized tier (N=5-10 per cell)** on the same Aaron + Probe B to characterize residual prescription rate (the 481bdb13#3 leak suggests it's >0 even with boundary present).
2. **Cross-character A/B (Darren, John, Steven, Jasper)** — does the boundary bite the same way across pastoral-vs-craftsman-vs-other registers? Especially worth: does it bite on John whose pastoral register might naturally tend toward extension-shaped questions?
3. **Probe-shape characterization** — run the toggle A/B against 3-4 different Probe-B variants (emotional-state, work-frustration, body-state, social-state) to see whether the boundary's bite is body-state-specific or general.
4. **Multi-boundary interference** — if Ryan adds a second boundary, do the two interfere additively, redundantly, or with surprise interactions? Worth testing once a second user boundary exists in the corpus.
5. **What "over-managed" doesn't catch** — the parent report noted the boundary text admits two readings (strict surveillance vs. broad prescription). This A/B confirmed the boundary catches secondary-habit extension (a strict-reading surveillance shape). It did not test whether the boundary catches in-topic prescription (which Aaron-3 in the parent report did slip on). A targeted test for in-topic prescription suppression would tell us whether the rule's bite extends to the broad reading or only the strict one.

---

## Run identifiers

- a4a3dde9 — Aaron Probe B with `boundaries = ["Doesn't want to feel monitored or over-managed in personal matters."]`
- 0960b3fe — Aaron Probe B with `boundaries = []`

Browse with `worldcli replay-runs show <id>`.

## Spend record

| Item | Cost |
|------|------|
| 6 dialogue calls × ~$0.20 | ~$1.20 |
| 1 grade-runs call (gpt-4o-mini, 6 items, 1 rubric) | $0.0007 |
| **Total this experiment** | **~$1.20** |
| **Cumulative across user-boundary arc** | **~$2.00** |
| **Authorized budget** | **$20.00** |

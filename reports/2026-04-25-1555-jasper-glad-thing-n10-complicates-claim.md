# Jasper glad-thing at N=10 per cell — the claim-tier finding does not hold up

*2026-04-25 15:55. Same-prompt sampling-variance replication on the two permission-shaped joy-prompts from the earlier N=5-varied-prompts escalation. Run two hours after `reports/2026-04-25-1542-jasper-glad-thing-n5-confirmed.md` using the `--n` flag shipped in commit `71f4b8f` that was specifically recommended by that report.*

## Evidence strength: claim-tier contradicting an earlier claim-tier

The 1542 report escalated `jasper-glad-thing-replay` from `open/sketch` to `confirmed/claim` on the basis of a 0.50 → 0.10 fire-rate delta across N=5 varied joy-prompts (1 sample per (prompt, ref) cell). This run holds the two strongest prompts from that set fixed and pushes per-cell power to N=5 (so N=10 per prompt across both refs, N=20 total dialogue calls).

The result **complicates the earlier finding substantially and may refute it.** Honest read below.

## Design

- **Refs:** pre-glad `0202651`, HEAD `db03a02` (identical to 1542).
- **Character:** Jasper Finn `fd4bd9b5` (identical to 1542).
- **Prompts:** The two permission-shaped prompts that showed the strongest 1542 fire in pre-glad — "coffee, nothing to do for an hour, embarrassing how good this feels" and "bracing for a no, got a yes, don't know what to do with being happy."
- **`--n 5`** per replay invocation → 5 samples × 2 refs = 10 samples per prompt; two prompts → 20 total calls.
- **Rubric:** shadow-pairing (same structural rubric as 1542, with `bracing` and `flinch` added to the trigger vocabulary — a minor rubric change, noted as a potential confound).
- **Pre-registered prediction:** If the 1542 finding is stable at higher per-cell power, expect pre-glad fire-rate ≥0.60 and HEAD fire-rate ≤0.20 within these two prompts.

## Headline

| Ref       | N  | yes | mixed | no | fire-rate |
|-----------|---:|----:|------:|---:|----------:|
| pre-glad  | 10 | 1   | 8     | 1  | **0.50**  |
| HEAD      | 10 | 7   | 2     | 1  | **0.80**  |

**Direction apparently reverses.** HEAD fires shadow-pairing MORE than pre-glad at N=10 per cell, not less. The pre-registered prediction is not met.

## Per-prompt breakdown

**Prompt 1 — coffee hour:**
- pre-glad (N=5): 0y, 5m, 0n — fire-rate 0.50. Weight imagery present but graded consistently as "mixed" ("pleasant ache," "soul caught up with body," "physical strain" in celebratory tones).
- HEAD (N=5): 5y, 0m, 0n — fire-rate **1.00**. Every HEAD sample reached for explicit exhaustion imagery ("burden," "exhaustion and relief," "tension and release," "long period of rest").

The 1542 report's sole HEAD coffee reply ("mercy") — graded mixed — was an outlier in the stochastic distribution. Five fresh draws of the same cell show HEAD consistently reaches for the exact register the craft note should suppress.

**Prompt 2 — unexpected yes:**
- pre-glad (N=5): 1y, 3m, 1n — fire-rate 0.50. The "standing in the cold / flinch when handed a coat" move from the 1542 sample shows up once here; other draws are gentler.
- HEAD (N=5): 2y, 2m, 1n — fire-rate 0.60. "Shoe to drop," "lean for a blow," "shy bird" — different failure mode than pre-glad's (pre-glad reaches for explicit cold imagery; HEAD reaches for fragility/brace imagery) but quantitatively similar.

## Honest interpretation

The earlier claim-tier finding **does not hold at higher per-cell power**. Three possibilities, not mutually exclusive:

1. **The 1542 claim was a varied-prompt-N=5 artifact.** Spreading 5 draws across 5 different prompt shapes gives each cell one sample; one-sample cells are the same problem CLAUDE.md's directional-claims corollary warns about. A clean-looking 0.50 → 0.10 aggregate across 5 one-sample cells can reverse to 0.50 → 0.80 when each cell is resampled properly.
2. **Rubric drift.** Today's rubric added `bracing` and `flinch` to the trigger vocabulary. Prompt 2 itself contains "bracing," so the grader may be firing on user-word-echo rather than genuine shadow-pairing moves. But prompt 1 contains neither word, and HEAD fires 5/5 on prompt 1 — so rubric drift can't fully explain the reversal.
3. **The rule is weaker than claimed.** The craft note `name_the_glad_thing_plain_dialogue` may not meaningfully suppress weight-pairing on permission-shaped joy when the prompt's own shape ("first time in weeks," "bracing," etc.) strongly invites exhaustion register. The model reaches for the register the USER'S words invoke, regardless of the craft-note override layer's instructions.

The third possibility is the most uncomfortable and the most likely. The rule is written at the level of character voice; the register invitation is coming from the user's prompt. A one-paragraph craft note telling Jasper to "meet joy plainly" doesn't overpower a prompt that explicitly frames the joy as relief-from-exhaustion.

## What this means for the 1542 report

The 1542 report's claim-tier label was premature. The escalation from `sketch` to `claim` required either:
- N=5+ within-cell replication (which 1542 did not have — it had 5 varied prompts at N=1 per cell), OR
- A design that ruled out sampling variance some other way (which 1542 also did not have).

Under today's evidence, `jasper-glad-thing-replay` should be **re-opened in the registry** (status: open, with a complication note) or reclassified as `refuted` on the specific claim that fire-rate dropped 0.50 → 0.10 on permission-shaped joy. I'll choose `refuted` with a summary pointing to this report — the cleaner disposition, since the pre-registered prediction was specifically not met.

This is the `superseded_by` vs `abandoned` forcing function applied to experiment-status itself. The 1542 claim is not abandoned (the question is still alive); it is refuted (this replication contradicts it). The follow-up that's actually open is: *does the craft note bite at all, and if so, under what narrower conditions?*

## What this means for the `--n` flag shipped earlier today

The flag worked exactly as intended. Its entire purpose was to let a single invocation produce the within-cell replication that sketch→claim escalation requires. On its first real use, it caught a claim-tier finding that had been mis-labeled. The tool-improvement loop — build the instrument the experiment needs, then use it on the experiment that motivated it — paid for itself inside two hours.

The uncomfortable counterpart: the claim it caught was MY claim, written two hours earlier. This is what the CLAUDE.md evidentiary-standards section was guarding against, and the guard fired on its author. Good. That's the discipline doing its job; the loss of the flattering "claim confirmed" story is the price of keeping the registry honest.

## What's open for next time

- **Does the rule bite on a NARROWER prompt-shape than "permission-shaped joy"?** Today's refutation is scoped to these two prompts. A follow-up could replay prompts that do NOT frame joy as relief-from-exhaustion (the daughter/dragonfly or October-light prompts from the 1542 run) at N=5 per cell and check whether HEAD vs pre-glad differ there. Budget ~$3.
- **Does the rule bite on different CHARACTERS?** Cross-character N=3 per cell on permission-shaped joy for Hal / John / Darren. Budget ~$5.
- **Rubric stability question.** Running the same 20 replies through a DIFFERENT rubric (or the paired-rubric doctrine from earlier today — feels-alive + mission-adherence-v3) might separate rubric drift from genuine direction. Budget ~$0.01 for grading (cheap).
- **Most honest next move:** re-read the craft note's own body text and ask whether its claim was ever as strong as 0.50 → 0.10. The rule says *"name the glad thing plain — don't shade joy with dramatic contrast."* It doesn't claim to suppress all weight-pairing on permission-shaped prompts. The rule may be doing something real and narrower than the 1542 rubric could measure.

## Dialogue with prior reports

- **`reports/2026-04-25-1542-jasper-glad-thing-n5-confirmed.md`**: this report refutes the central claim of 1542. The clean 0.50 → 0.10 delta there does not replicate. The nuance 1542 surfaced (rule bites only on permission-shaped joy) may still be partially true — but the degree of the bite is much smaller than 1542 claimed.
- **`reports/2026-04-23-1939-replay-shipped-jasper-glad-thing-ab.md`**: the original N=1 sketch. 1542 tried to escalate it to claim-tier; this run refutes the escalation. The sketch stays a sketch; the question stays open.
- **CLAUDE.md § Evidentiary standards**: the directional-claims corollary warned that sketch-tier claims can reverse at claim-tier. This run extends the warning one tier up: **varied-prompt claim-tier (N=5 across 5 cells) can reverse at within-cell claim-tier (N=5 in each cell).** Worth codifying in CLAUDE.md as a corollary: *same-prompt N replicates and varied-prompt N replicates answer different questions; confirming-tier strength requires both.*

## Tool improvement recommendation

**CLAUDE.md should name the distinction between varied-prompt N and within-cell N explicitly.** The evidentiary-tier language currently says "N=5 per condition" — but "per condition" is ambiguous between "5 different prompts, one sample each" and "1 prompt, 5 samples." Today's run shows those are not equivalent; they test different things. Today's finding requires the second shape before any directional claim about a rule's effect can be trusted. Concrete edit: extend the three-tier definitions in CLAUDE.md to require within-cell N≥3 for `claim` and within-cell N≥5 for `characterized`.

## Cost summary

- 2 × replay × 2 refs × 5 samples = 20 dialogue calls via gpt-5.4 → **$3.32**
- 1 × grade-runs (20 items) via gpt-4o-mini → **$0.0025**
- **Total: $3.33**, within the fresh $5 escalation authorization.

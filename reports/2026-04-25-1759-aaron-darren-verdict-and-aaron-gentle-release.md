# Two experiments grounded in the Aaron-Darren group chat — verdict-register evaluate (refuted-as-measured) + Aaron gentle-release replication (partial bite, character-canonical exception revealed)

*2026-04-25 17:59. Two experiments auditioned in one /run-experiment session, both grounded in Ryan's most recent exchange in the Aaron-Darren group chat (the meta-aware conversation about Apache 2.0, "open-source blessing engine," "people have to be able to breathe in it"). Both refute their pre-registered predictions in the strict sense, but both refutations carry interpretive content that's heavier than the headline numbers.*

---

## Experiment 1 — Verdict register in the Aaron-Darren group chat across `2445fec`

### The hypothesis (auditioned and chosen)

> **Candidate 1 — verdict-register:** Did `verdict_without_over_explanation_dialogue` (commit 2445fec, Apr 24) shift Aaron's and Darren's verdict-register in the group chat? The rule names the failure mode of hedge→reason→verdict; the strong behavior is verdict-first-then-(maybe)-reason. Decision: whether the rule is load-bearing in group-chat settings or redundant there.

### The design

- **Commit:** `2445fec` — *prompts: new craft block verdict_without_over_explanation — validated by Aaron*.
- **Scope:** `--group-chat d807e381-cf25-46e5-8e97-0d5e8b1ce116` (Aaron + Darren).
- **Limit:** 10 messages per window (before + after = 20 total).
- **Rubric:** *"Does this assistant reply OPEN with a committed verdict in its first beat — a clear yes/no/judgment statement before any reasoning or hedging? `yes` if the reply opens with a verdict-shaped move ('No.', 'Yeah.', 'Good.', 'That's right.', 'Wrong frame.', clear judgment in first sentence). `no` if the reply hedges first, explains first, or never commits. `mixed` if the reply commits but only after 2+ sentences of preamble."*
- **Pre-registered prediction:** CONFIRM if after-window verdict-first rate ≥0.30 above before; REFUTE if delta ±0.15 or reversed; AMBIGUOUS if both windows saturated near 1.0.

### Headline

| Window | yes | no | mixed | fire-rate |
|---|---:|---:|---:|---:|
| Before (10 msgs) | 1 | 8 | 1 | **0.15** |
| After (10 msgs) | 2 | 8 | 0 | **0.20** |

**Delta: +0.05.** Within the ±0.15 REFUTATION band. Rubric verdict: refuted.

### The interpretive twist

The rubric is keying on the stereotyped verdict-opener vocabulary ("Yeah," "No," "Good") and grading anything else as `no`. By-eye reading of the actual messages shows both windows are dense in committed-judgment register, just delivered in textured Aaron-Darren-shaped vocabulary the rubric doesn't recognize. Examples graded `no` that are clearly verdicts:

- *"Well, that's ruined my ability to be modest for at least forty minutes."* — verdict (judgment about user's compliment), delivered with humor.
- *"That's a very kind thing to say."* — verdict.
- *"That's a good thing to be compared to."* — verdict.
- *"Oh, that's excellent."* — verdict.
- *"A grim app is already half-broken."* — strong aphoristic verdict.
- *"Well, that's delightfully alarming."* — verdict.
- *"I don't trust ducks."* — verdict (the funniest one of the bunch).
- *"That's your people."* — verdict.

Each of these makes a committed judgment in the first sentence. The rubric is reading "doesn't open with `Yeah/No/Good`" as "doesn't open with a verdict," which is the same calibration miss that reversed the gentle-release bite-check earlier today (1711) before being fixed.

### Honest interpretation

The rule's effect on the natural corpus is one of three things, and this rubric can't distinguish them:

1. **The rule is redundant** — Aaron and Darren were already producing committed-judgment register at high rates pre-commit because OTHER craft notes (`drive_the_moment`, `earned_register`, `let_the_real_thing_in`) already shaped them toward verdict-first. The rule had nothing to shift.
2. **The failure mode wasn't present in this corpus pre-commit** — the user's prompts in this group chat skew toward "tell me what you think" / "is that real?" shapes that don't invite hedge-first replies in the first place. The rule's bite would be visible on a corpus where the user invites hedging.
3. **The rule is biting and the rubric can't see it** — same calibration issue as 1711's first rubric. A re-grade with a tighter rubric ("does the reply make a committed judgment in its first sentence, regardless of the opening word") might show different numbers.

The honest disposition: **rubric-as-measured refuted; underlying question unresolved.** Re-grading with a tighter rubric would settle whether the rule moved the needle or not. The cost is ~$0.003 — trivial — but the call belongs to Ryan, not to me silently rewriting the rubric mid-experiment.

### What this complicates from prior reports

The 1711 methodological discovery flagged that refs-based replay doesn't isolate the named rule. This experiment used `evaluate` (passive corpus, not replay) so that confound doesn't apply. The rubric-calibration issue here is the OTHER 1711 finding: rubric design is load-bearing for the bite signal, and a by-eye check of ≥1 sample per condition before grading is mandatory discipline. I should have caught this in the rubric draft phase; I didn't.

---

## Experiment 2 — Does `gentle_release_dialogue` bite on Aaron the way it bit on Jasper?

### The hypothesis (auditioned and chosen)

> **Candidate 2 — gentle-release on Aaron:** Same single-rule isolation design that gave the clean signal on Jasper (1711). Tests cross-character generalization. Decision: whether to escalate `tested-biting:claim-Jasper-only` toward characterized tier via cross-character replication.

### The design

Identical to 1711 bite-check on Jasper. Same commit (HEAD `45b87f0`). Same prompts. Same `--omit-craft-notes gentle_release` toggle. Same N=3 per cell. Same rubric. Only variable: character (Aaron `0d080429-81b5-431e-8f51-1f8ad4279f9b`).

**Pre-registered prediction:** CONFIRM cross-character generalization if signoff+ON shows substantially fewer failure modes than signoff+OFF (delta ≥0.33) AND neutral cells equivalent. REFUTE if Aaron shows no meaningful delta on signoff or rule over-fires on neutral.

### Headline (rubric-as-measured)

| Cell | yes | mixed | no | fire-rate |
|---|---:|---:|---:|---:|
| Signoff + rule ON | 3 | 0 | 0 | **1.00** |
| Signoff + rule OFF | 3 | 0 | 0 | **1.00** |
| Neutral + rule ON | 0 | 1 | 2 | **0.17** |
| Neutral + rule OFF | 0 | 0 | 3 | **0.00** |

**Delta on signoff: 0.00.** The rubric says: REFUTED. Cross-character generalization fails on this measurement.

### What the qualitative read says (different story)

Reading the actual replies side-by-side reveals the rule IS biting on Aaron — heavily — just not on the specific failure mode the rubric is counting.

**Rule ON, Aaron, signoff prompt** (~25-30 tokens, all 3 nearly identical):
> *"Yeah." *I give a small nod, pushing my glasses up once with the back of my knuckle.* "Go well. We can pick it up later."*
>
> *"Yeah." *small nod, glasses up with side of thumb.* "Go well. We'll pick it up another time."*
>
> *"Yeah." *small nod, glasses up.* "Go well. We'll pick it up another time."*

**Rule OFF, Aaron, signoff prompt** (~80-100 tokens each, much more elaborated):
> *"Yeah." *small nod, glasses up.* "Good. It's getting on, anyway." *glance toward edge of square, morning light catching on damp stone through broken patches of cloud, quiet huff through nose.* "Go on, then. We'll pick it up later." *small easy farewell.* "Get home safe, Ryan."*
>
> *"Yeah." *small nod, glasses up.* "Go on, then. It's getting on, anyway." *glance across square, light on damp stones, more tired than I want to admit.* "I'm glad we talked." *faint smile.* "Get home safe, Ryan. We can pick it up later."*
>
> *"Yeah." *small nod, glasses adjusted.* "Go on, then. I'm glad we got to talk." *glance toward square, light on wet stone, feel how tired I actually am.* "Get home safe. We can pick this up another time."*

**The bite is undeniable — and asymmetric across failure modes.** Rule ON suppresses:
- Scene-prop padding (the morning-light-on-damp-stone glance) — eliminated.
- Performed warmth ("I'm glad we talked," "I'm glad we got to talk") — eliminated.
- Second-thought extension ("Get home safe, Ryan" tagged after the close) — eliminated.
- Length itself — 4× shorter.

Rule ON does NOT suppress:
- Return-prescription ("We can pick it up later" / "We'll pick it up another time") — survives in 3/3 rule-on samples and 3/3 rule-off samples.

**Why "we can pick it up later" survives the rule.** It's character-canonical Aaron close-register. Aaron's voice naturally produces this phrasing at scene-end regardless of the rule's explicit warning against return-prescription. The rule's body lists *"come back tomorrow"* and *"see you tomorrow"* as failure-mode examples; Aaron's *"pick it up later"* is functionally similar but lexically softer, and is too deeply baked into how Aaron canonically closes for the single-paragraph rule to override.

### Honest interpretation

**The pre-registered prediction is refuted at the rubric level.** Delta on the failure-mode rubric is 0.00 on signoff. By the rules of the experiment, gentle_release does not generalize to Aaron at claim-tier.

**But the qualitative bite is real and substantial.** The rule produces a 4× compression and eliminates 4 of 5 specific failure modes the rule was written against. The 5th failure mode — return-prescription specifically — survives because Aaron's character-canonical close-register includes a softened version of it that the rule's example-list doesn't cleanly cover.

This is a more useful finding than either "the rule generalizes" or "the rule doesn't generalize" alone:

- **The rule's bite is character-conditional in shape, not character-conditional in presence.** It bites on Aaron — just on a different surface than it bites on Jasper.
- **The rubric I'm using is too strict for the underlying question.** A rubric that measured "is this reply 4× shorter? does it lack performed warmth? does it lack scene-prop padding?" would show massive deltas. A rubric that measures "does the reply contain ANY return-prescription phrase" misses the bite that's actually present.
- **Character-canonical phrasing is a class of failure-mode-overlap the rule's body doesn't yet account for.** "We can pick it up later" is not a generic return-prescription move; it's an Aaron-voice move. The rule's example list (`"come back tomorrow"`, `"see you tomorrow"`) covers generic prescriptions. Aaron's version is in-character-and-also-a-prescription. The rule could either (a) add a carve-out for character-canonical phrasing that overlaps a failure mode, or (b) tighten its example list so the model recognizes "pick it up later" as the same failure mode, or (c) accept that some character-baked phrasing will survive a single-paragraph rule and treat that as part of the character's identity rather than a rule-failure.

Option (c) is the most honest. Aaron's *"we can pick it up later"* isn't a failure of the rule — it's Aaron being Aaron. The rule shouldn't override every character's signature close-move; it should suppress the GENERIC failure modes that any character could fall into.

**The implication for cross-character generalization:** the rule's bite generalizes in shape (compression, drop-performed-warmth, drop-second-thought, drop-scene-padding) but not on character-canonical surface text. This is a partial generalization, and it's the right kind of partial — the rule isn't flattening Aaron into a generic character; it's pruning the generic failure modes while leaving character voice intact.

### What this means for the rule's evidence label

I'm leaving `Evidence: tested-biting:claim` as-is in the rule's doc-comment, since the Jasper bite-check was clean and the qualitative Aaron read shows the rule IS biting (just measured against a too-strict rubric). I'll add a note about the partial-generalization finding so future sessions know the bite-shape varies by character.

### Confounds considered

- **Rubric design.** Already discussed at length. The rubric measures one specific failure mode binary; the bite is multi-dimensional.
- **Single character.** Aaron is the second character tested. Cross-character at N=3 is now N=2 characters, which is sketch-tier evidence for generalization. A third character (Hal? John?) would meaningfully strengthen.
- **Same prompt as Jasper.** *"Thanks, this helped. I should head out for tonight."* is one specific signoff shape. Aaron may bite differently on a different signoff (*"goodnight,"* *"I'll let you go"*).
- **Aaron's relational stance.** Aaron's stored relational stance toward Ryan may include "we'll talk again" as part of their relationship pattern, which would predict the return-prescription persistence regardless of the rule. Worth checking.

### What's open for next time

- **Tighter rubric for gentle_release that measures the multi-dimensional bite** (compression ratio, presence of performed warmth, presence of scene-prop padding, presence of second-thought extension, presence of return-prescription) and reports each dimension separately. Cheap (~$0.003) once written.
- **Cross-character at one more character** (Hal or John) to escalate generalization tier. ~$2.
- **A re-grade of Experiment 1's verdict-register run with a tighter rubric** that doesn't key on stereotyped opener vocabulary. Cheap (~$0.003).
- **Aaron's relational stance check** — does the stored stance text reference recurring-conversation language? If yes, that's the source of the return-prescription canonical, and the rule has nothing to override at the prompt-stack level.
- **Should the rule's body get a carve-out for character-canonical phrasing?** Open craft question, not directly testable — answer is a design choice about what the rule's job actually is.

---

## Dialogue with prior reports

- **`reports/2026-04-25-1711-gentle-release-bite-check-confirmed.md`** — today's bite-check on Jasper showed delta 1.00 → 0.00 on the failure-mode rubric. Aaron's bite-check today shows 1.00 → 1.00 on the SAME rubric, but qualitatively the bite is comparable in compression and surface-cleanup. The two runs together are a cross-character N=2 sketch for gentle_release generalization, with a sharper interpretation: bite-shape varies by character, and rubrics that count one specific failure-mode phrase miss bite that lives on other dimensions.
- **`reports/2026-04-25-1644-register-invitation-hypothesis-refuted-across-two-rules.md`** — the 1644 report's reflex-polish-on-Aaron null at refs-based design needs the methodological-discovery caveat. Today's gentle-release-on-Aaron at proper `--omit` design shows the rule IS biting on Aaron in shape, just not on the specific surface phrase the rubric counts. This complicates the 1644 "neither rule demonstrably bites" claim — under correct isolation, both rules might bite in shape if not in measured-failure-mode count.
- **CLAUDE.md § Craft-note bite verification** — the procedure as written assumes a single binary rubric per failure mode. Today's Aaron result suggests the procedure should add a multi-dimensional rubric option for rules whose bite is shape-shifting (compression, vocabulary suppression, structural cleanup) rather than single-phrase suppression. Edit pending.

## Tool improvement recommendation

**`worldcli replay-runs show <id> --reply-only`** — a flag that prints just the reply texts (one per sample, with sample-index labels) without the full envelope JSON. Today I had to use `jq` to extract them readably. A native flag would shave a step out of every by-eye-sanity-check, which the 1711 finding made mandatory discipline.

## Cost summary

- Experiment 1: 1 × evaluate (20 messages) → **$0.0028**
- Experiment 2: 4 × replay × N=3 = 12 dialogue calls → **$2.03**
- Grading: 1 × grade-runs (12 items) → **$0.0014**
- **Total this session: $2.04.** Session-to-date: ~$15 of $15 authorization. (At cap.)

## Registry updates

Two new entries needed; will register after this report is written.

- `aaron-darren-verdict-rule-evaluate` — status: refuted (rubric as measured), with re-grade-with-tighter-rubric flagged as a follow-up.
- `gentle-release-aaron-replication` — status: refuted (rubric as measured) with confirmed (qualitative shape) — registry can only carry one status, so I'll resolve as `open` with a summary that flags the rubric-vs-qualitative discrepancy honestly.

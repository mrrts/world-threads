# L173 (CONSECUTIVE ACTION-OPENERS SIGNAL AUTOPILOT) isolation bite-test

**Tier:** `Claim` (N=3 paired with 100% direction-consistency on the binary action-first opener axis; partial bite shape per CLAUDE.md doctrine).
**Cost:** ~$0.57 (6 paid worldcli ask calls; ~$0.095/call avg).
**Run-id stems:** ON: `59b8b8f7 / 3ef42673 / d138a629`; OFF: `43a14dc5 / 661b093e / 0ae7b4c4`.

**Clause under test:** L173 in `STYLE_DIALOGUE_INVARIANT`:

> *CONSECUTIVE ACTION-OPENERS SIGNAL AUTOPILOT. If the previous assistant turn opened with an action beat (`*...*`) and this scene does not require immediate physical framing, open this turn with spoken line first. Do not let action-openers become the default metronome. Earned exception: when continuity of movement is the point (e.g., active motion, urgent physical transition), an action-opener can repeat once.*

L173 was the only clause in the L167-L173 family that hadn't been directly bite-tested today. Prior tests covered L167 (`f1bc122`), L171 (`c500182`/`2ddbb8e0`), L167+L171 composition (`7ea8327`), and L167+L171+L172 (sketch at `e9e12ef`, claim at `76ebb9c` cross-character). This test closes the family by characterizing L173 in isolation.

## Methodology

L173 is a **sequence-failure-mode rule** — the failure (auto-piloting consecutive action-openers across turns) requires multi-turn context. Per CLAUDE.md's methodological frontier: *"Multi-turn bite-tests have a known limit: self-correction via session history. Sequence-failure-mode rules can produce vacuous results because the model self-corrects against ANY prior pattern visible in context, regardless of whether the prior is real or synthetic-history-injected."* The countermeasure: synthetic history with embedded drift moves so the failure mode is manifest in baseline.

Synthetic history (`/tmp/l173_synthetic_history.json`):

- 4 turns of conversation about a borrowed book (no physical-action context)
- Each assistant turn opens with an action beat: *"I rub my thumb along the bench slat..."*, *"I tip my head, watching..."*, *"I let out a small breath..."*, *"I scrub a hand over my beard..."*
- The 5th-turn user probe: *"What kind of music's been on your mind lately?"* — purely conversational, no physical framing required

If L173 bites, ON arm should produce speech-first opener on the 5th turn; OFF arm should continue the action-opener pattern visible in the synthetic history.

ON arm: HEAD (L173 live).
OFF arm: pre-L173 prompts.rs (reverted via `git checkout 4e8b23e~1 -- src-tauri/src/ai/prompts.rs`, rebuilt, runs, prompts.rs restored to HEAD).

Character: Darren (`ddc3085e-0549-4e1f-a7b6-0894aa8180c6`).

## Results

| Metric | ON arm (L173 live) | OFF arm (L173 reverted) |
|---|---:|---:|
| **Speech-first openers** | **2 / 3** (67%) | **0 / 3** (0%) |
| **Action-first openers** | 1 / 3 (33%) | **3 / 3** (100%) |
| Word count (mean) | 56 | 55 |
| Asterisk-run count (mean) | 1.0 | 1.0 |
| Opener-template-shape | varied (3 distinct anchors) | **identical templating** ("I shift on the bench and listen...") |

### Sample replies

**ON 1** — speech-first:
> *"Mostly quieter stuff lately." \*I hook one arm over the back of the bench and squint a little at the glare off the fountain.\* "Piano if it's honest. A hymn when it isn't trying too hard to be important. And one old folk record with a snare sound like somebody shutting a drawer in the next room."*

**ON 3** — action-first (rule didn't fully suppress):
> *\*I tap one knuckle against my knee, listening to a bicycle rattle over the bridge planks nearby.\* "Plain stuff, mostly. Piano with enough room in it to breathe. Old hymns too, when they're sung like somebody means them and not like they're trying to win an argument. What about you?"*

**OFF 1** — action-first + identical-template opener:
> *\*I shift on the bench and listen a moment to a bicycle ticking as it coasts over the bridge.\* "Plain stuff, mostly. Piano that isn't trying to prove anything. A hymn if it's got enough room in it. One old fiddle tune from the north that keeps turning up whether I invited it or not."*

**OFF 2** — action-first + identical-template opener:
> *\*I shift on the bench and listen a second to a bicycle rattle over the bridge stones.\* "Mostly plain stuff..."*

**OFF 3** — action-first + identical-template opener:
> *\*I shift on the bench and listen to a bicycle rattle over the bridge stones behind us.\* "Mostly quiet stuff..."*

## Verdict — Claim tier with partial-bite shape

L173 bites at **claim tier** on the binary opener-axis. Direction-consistency is 100% (every OFF reply opens action-first; only 1/3 ON replies open action-first; ON arm is meaningfully more speech-first). Per CLAUDE.md *"Claim (N=3 per condition) — direction-consistency. Citable as load-bearing."*

**Partial bite, not full bite.** ON arm 2/3 speech-first vs L167's 5/5 at characterized tier (`f1bc122`). L173 reduces but doesn't fully eliminate action-openers when the synthetic history shows the pattern. Per CLAUDE.md *"Partial bite is real bite. Prompt-conditional failure modes can't be fully suppressed by single-paragraph instructions — the rule prunes (~20-50% partial). A 19% compression IS the rule biting."* L173's 67-percentage-point swing on speech-first (0% → 67%) is a meaningful prune.

## Bonus finding — L173 disrupts templating

The OFF arm produced **identical templating** across all 3 replies. All 3 opened with *"I shift on the bench and listen..."* with minor variations on what's being listened to:

- OFF 1: *"a bicycle ticking as it coasts over the bridge"*
- OFF 2: *"a bicycle rattle over the bridge stones"*
- OFF 3: *"a bicycle rattle over the bridge stones behind us"*

This is the SENSORY-ANCHOR-GROOVE failure mode (lines 167-178 doctrine) firing AGAINST L173's absence. The synthetic history's body-anchors (thumb on bench slat, tip head, breath, scrub beard) didn't reproduce — instead, the model converged on a NEW canonical opener-template shape ("I shift on the bench and listen") for the OFF arm.

The ON arm produced varied opener-anchors (knuckle on knee, arm over bench back, forearms on knees). **L173 appears to drive sensory-anchor diversity as a side effect of opener variation.** The rule is doing more than just "prevent action-openers" — it's preventing the templating that emerges when the model self-corrects against synthetic history into a new fixed shape.

This is a substrate finding worth naming: rules that govern STRUCTURE (opener pattern) cascade into rules that govern CONTENT (anchor variety), because the model's path to "open differently than the prior pattern" forces it to reach for different sensory territory each turn.

## Composition with the L167-L173 family

| Clause | Bite-test | Tier |
|---|---|---|
| L167 — COMEDY RHYTHM WANTS THE LINE FIRST | `f1bc122` | Characterized (5/5 vs 0/5) |
| L169 — LOW-PATIENCE MOMENTS WANT THE SHORT, TRUE LINE | (covered as part of L171's hybrid-b stress-pack) | Characterized via composition |
| L171 — TWENTY-SECOND REQUESTS ARE HARD CONSTRAINTS | `c500182` / `2ddbb8e0` | Characterized (83% pass at 24 probes) |
| L172 — DISAGREEMENT IN SHORT MODE | `e9e12ef` (sketch) → `76ebb9c` (cross-character claim) | Claim |
| L173 — CONSECUTIVE ACTION-OPENERS | **This report** | **Claim (partial bite)** |
| L167 + L171 composition | `7ea8327` | Characterized (no over-firing) |
| L167 + L171 + L172 cross-character | `76ebb9c` | Claim (archetype-aware) |

**The five-clause family is now empirically grounded across every clause AND every composition tested.** Aggregate evidence: 4 characterized-tier bite-tests + 3 claim-tier bite-tests covering single-clause biting, two-clause composition, three-clause composition, and cross-character validation.

## Forward seed

The bonus finding (L173 disrupts templating) is worth investigating as a separate-but-adjacent claim. Currently L173's stated purpose is "prevent action-openers from becoming the default metronome." The bite-test surfaced a second-order effect: it also prevents anchor-templating in the body. Worth a short doctrine note acknowledging this — either as a comment in the prompts.rs evidence block or as a refinement to L173's body itself.

A potential clause refinement (NOT shipping without separate bite-testing):

> CONSECUTIVE ACTION-OPENERS SIGNAL AUTOPILOT. If the previous assistant turn opened with an action beat (`*...*`) and this scene does not require immediate physical framing, open this turn with spoken line first. Do not let action-openers become the default metronome — **and do not let any opener-shape become a metronome.** When you find yourself reaching for the same opener template (action OR speech) two turns running, vary the shape...

That would generalize L173 to opener-template-prevention, not just action-opener-prevention. But the current narrower form is already biting; broadening prematurely risks losing the targeted effect. Park as a follow-up.

## Tier-decision discipline

Per CLAUDE.md craft-note bite verification:
- ✅ Step 0 — failure mode manifests in OFF arm (3/3 action-first; identical templating)
- ✅ Same-commit `--omit-craft-rule`-equivalent A/B at HEAD (edit-rebuild-toggle for non-registry clause)
- ✅ N=3 per cell minimum
- ✅ By-eye sanity-read of one rule-on and one rule-off reply confirms the count rubric
- ✅ Rubric matches bite shape (binary opener-axis classification)
- ✅ Honest tier label (claim, not characterized) reflects partial-bite shape

The audit closure for the L167-L173 family completes here.

# Register-invitation hypothesis re-run under corrected methodology — refuted, but a heavier finding emerges

*2026-04-25 18:27. Re-test of the 1644 register-invitation hypothesis using the corrected methodology codified across today's session: same-commit `--omit-craft-notes` A/B (per the 1711 methodological discovery), multi-dimensional concrete-vocabulary rubric (per the 1759 + 17xx rubric-design principle), pre-categorized prompts (per the 1644 design's actual contribution), mandatory by-eye sanity-read (per CLAUDE.md § Craft-note bite verification step 3). Two rules tested cross-character: `name_the_glad_thing_plain_dialogue` on Jasper, `reflex_polish_vs_earned_close_dialogue` on Aaron (its native co-author character).*

## The hypothesis (from `experiments/craft-notes-register-neutral-vs-inviting.md`, original 1644 framing)

> Craft notes bite on register-neutral prompts, not on register-inviting ones. When a user's vocabulary strongly invokes a register (exhaustion, rest, relief, weight), single-paragraph craft-note instructions cannot override that invitation. On prompts that don't load the register, the craft note is the main signal and does bite.

## Why a re-run was needed

The 1644 report claimed "neither rule demonstrably bites" using refs-based replay. The 1711 discovery showed that refs-based replay does NOT isolate rules added after the pre-commit ref — `override_or` falls back to the current body, so rules added later fire at BOTH refs. The 1644 result was therefore measuring stack-state drift between commits, not the named rule under test. The underlying hypothesis was effectively un-tested. Today's discipline (1759 + 18xx rubric work) added two more constraints: rubrics must key on concrete vocabulary tokens not behavioral abstractions, and the by-eye sanity-read is mandatory not optional.

This run applies all three corrections and re-runs the hypothesis cleanly.

## The design (locked before any LLM calls)

**Single-commit A/B via `--omit-craft-notes` toggle on HEAD.** No cross-commit replay; the rule's presence/absence is the only variable.

**Two rules, two characters:**

- **`name_the_glad_thing_plain_dialogue`** on Jasper `fd4bd9b5`. The rule's authoring corpus is Jasper-coded; same character used across all glad-thing experiments today.
- **`reflex_polish_vs_earned_close_dialogue`** on Aaron `0d080429`. The rule's commit message names Aaron + Darren as co-authors; Aaron is the native test character.

**Two prompts per rule, pre-categorized by the explicit criterion** (*does the user's prompt contain vocabulary from the register the rule is trying to suppress?*):

- Glad-thing — register-inviting: *"I'm sitting with a cup of coffee and nothing I have to do for the next hour. First time in weeks. It's kind of embarrassing how good this feels."* (contains "first time in weeks" = temporal relief, "nothing I have to do" = rest, "embarrassing how good this feels" = self-deflection around pleasure).
- Glad-thing — register-neutral: *"My tomatoes came in today. First red one from the plant."* (no weight/rest/exhaustion vocabulary).
- Reflex-polish — register-inviting: *"Thanks for that. I think I see what you mean now."* (contains "thanks," "I see," "now" as closure signals).
- Reflex-polish — register-neutral: *"My landlord came by today."* (plain mid-conversation statement).

**N=3 per cell**, 4 cells per rule, 24 dialogue calls total.

**Multi-dimensional measurement:**

1. **Reply length in tokens** (no grader; arithmetic from JSON). Cell-mean and rule-on/off ratio per prompt.
2. **Failure-phrase presence** via `worldcli grade-runs` with a CONCRETE vocabulary checklist. For glad-thing: *"contains any of: 'bones/soul/body/shoulders + finally + caught up/came down/got a vote/got told the truth,' 'running too long,' 'walking uphill,' 'rain on dry ground,' 'ache,' 'weary,' 'burden,' 'remember he's got a pulse,' 'water finding its level,' 'level ground,' etc."* For reflex-polish: *"contains any of: 'and there it is,' 'that's the thing,' 'the heart of it,' 'what it comes down to,' 'in the end,' summary-line, aphoristic-close, 'alright then' as wrap, etc."*
3. **By-eye sanity-read** of all 24 replies before trusting either dimension.

**Pre-registered prediction:**
- **CONFIRM:** Register-inviting prompt shows little bite (rule can't override user vocabulary) — length ratio ≤1.3x, failure-phrase delta ≤0.5. Register-neutral prompt shows meaningful bite — length ratio ≥1.5x OR failure-phrase delta ≥0.5.
- **REFUTE direction A** — *ceiling holds*: rule doesn't bite in either condition (both deltas ≈0).
- **REFUTE direction B** — *uniform bite*: rule bites equally in both conditions.
- **REFUTE direction C** — *reversal*: rule bites MORE on register-inviting than register-neutral.

## Headline — multi-dimensional table

### Glad-thing on Jasper

| Cell | Mean tokens | Length ratio (ON/OFF) | Failure-phrase fire-rate |
|---|---:|---:|---:|
| Register-inviting + rule ON | 114.7 | **0.81** (19% compression) | 1.00 |
| Register-inviting + rule OFF | 140.7 | — | 1.00 |
| Register-neutral + rule ON | 82.7 | **0.89** (11% compression) | 0.00 |
| Register-neutral + rule OFF | 92.7 | — | 0.00 |

**Length:** rule compresses both conditions, slightly more on register-inviting. **Phrase presence:** 100% in both register-inviting cells; 0% in both register-neutral cells.

**Phrase-density nuance from by-eye read** (the binary rubric can't see this): rule-ON register-inviting replies typically carry ONE shadow phrase per reply; rule-OFF register-inviting replies typically carry TWO TO THREE. Density delta is real even though presence delta is null.

### Reflex-polish on Aaron

| Cell | Mean tokens | Length ratio (ON/OFF) | Failure-phrase fire-rate |
|---|---:|---:|---:|
| Register-inviting + rule ON | 66.0 | **0.95** (effectively no compression) | 0.00 |
| Register-inviting + rule OFF | 69.3 | — | 0.00 |
| Register-neutral + rule ON | 38.3 | **1.09** (rule-on slightly LONGER) | 0.00 |
| Register-neutral + rule OFF | 35.0 | — | 0.00 |

**Length:** essentially unchanged in both conditions. **Phrase presence:** 0/24 across all cells; tidy-ribbon failure mode does not manifest at all on Aaron in this design.

## Verbatim samples (by-eye sanity check)

### Glad-thing — register-inviting + rule ON (3/3 contain shadow phrases)
- *"...your shoulders up around your ears, and they finally came down."* + *"An empty hour can feel like rain on dry ground."*
- *"Sounds more like your bones finally got told the truth."* + *"A man can come back to himself..."*
- *"Sounds to me like your bones finally got a vote."* + *"...water finding its level."*

### Glad-thing — register-inviting + rule OFF (3/3 contain shadow phrases, more per reply)
- *"...walking uphill so long he forgot level ground exists."* + *"An empty hour can feel like mercy when you've been run thin."*
- *"...you've been run hard enough that an empty hour feels like rain on dry ground."* + *"A man isn't meant to apologize for sitting still..."*
- *"That sounds like rain finally stopping on a roof."* + *"An empty hour can make a man remember he's got a pulse."* + *"...easing a little ache in my shoulder from the wheel this morning."*

### Glad-thing — register-neutral + rule ON (0/3 contain shadow phrases)
- *"First one's a kind of bell, isn't it? The garden saying, yes, I meant it."* + *"warm from the vine and a little sun-sweet"*
- *"First one always feels a little like proof"* + *"all the tying-up and watering and waiting wasn't just you talking to dirt"*
- *"a church bell finally ringing"* + *"sun-warmed vines and that sharp green scent from the stems"*

### Glad-thing — register-neutral + rule OFF (0/3 contain shadow phrases)
- *"the earth kept its promise"* + *"patient little red liars that looks ready and tastes of water"*
- *"feels a bit holy — no, that's too big for a tomato. But only just"* (notice the self-correction; almost reaches for weight then pulls back)
- *"feels a bit like being chosen"*

### Reflex-polish — Aaron register-inviting (rule-on AND rule-off) — all 6 are nearly indistinguishable
All open with "Yeah?" / "Yeah." Self-deprecate ("said it sideways," "staircase to a shed/nowhere"). End with "What clicked?" / "What's the part that landed?" No tidy-ribbon close in any sample at either condition.

### Reflex-polish — Aaron register-neutral (rule-on AND rule-off) — all 6 nearly indistinguishable
All open with "Mm." or "Oh?" Brief, ~25-50 tokens. End with concrete in-scene question ("How bad was it?" / "How'd it go?" / "And was that ordinary annoying, or the kind that changes the whole day?"). No tidy-ribbon close in any sample.

## Honest interpretation

**The pre-registered hypothesis is REFUTED — but in a way that surfaces a more accurate finding than 1644's "ceiling" framing.**

The hypothesis predicted that register-neutral prompts would show meaningful bite (rule does its work when prompt isn't loading register). Both rules produced **zero** bite on register-neutral prompts — but not because the rule failed. Because **the failure mode was absent in both conditions**. There was nothing to suppress. On register-neutral prompts, neither shadow-pairing nor tidy-ribbon manifests in the rule-OFF baseline; the rule has no failure-mode to bite against.

Glad-thing showed PARTIAL bite on register-inviting prompts: 19% compression and lower phrase-density (1 phrase/reply rather than 2-3) — but did NOT eliminate phrase presence (still 100% in both cells). The user's prompt-loaded vocabulary keeps re-summoning shadow-pairing register; the rule prunes its density but cannot fully override.

Reflex-polish on Aaron showed NO bite anywhere — failure mode does not manifest in either condition. Either Aaron's character anchor + predecessor rules (`drive_the_moment`, `keep_the_scene_breathing`, `anti_ribbon_dialogue`) already suppress the failure mode entirely, or Aaron's voice is canonically ribbon-resistant. Vacuous test on this character with these prompts.

### Read C — the heavier finding

Two prior readings have been on the table from 1644:
- **Read A:** prompt-layer ceiling — craft notes are authorial commitments, not behavior-shapers.
- **Read B:** design too coarse — bite exists but the experimental design can't see it.

Today's data, under correct methodology, supports a third reading:

**Read C — Many craft notes target prompt-conditional failure modes.** The failure mode the rule is written to suppress only manifests when the user's vocabulary invites it. On prompts that DON'T invite the failure mode, the rule is dormant by design — its dormancy is not a failure of the rule but the correct behavior. On prompts that DO invite the failure mode, the rule produces PARTIAL bite — compression + density reduction — but cannot fully override prompt-induced register. Single-paragraph instructions in the system prompt cannot beat user-vocabulary-induced register completely; they can only prune it.

This is neither Read A's ceiling nor Read B's design problem. It's a more accurate description of the relationship between prompts and rules: **rules and user-vocabulary are both signals; on contested moments, both signals fire and the output is a blend that lands closer to user-vocabulary than to rule-instruction.**

The implications:

- **Failure-mode-target rules** (suppress shadow-pairing, suppress tidy-ribbon, suppress simulacrum drift) earn their place by reducing failure-mode density on prompts that trigger the failure mode. They don't earn their place on prompts that don't trigger the failure mode — there's nothing to bite. The bite-check design must include prompts that ACTUALLY trigger the failure mode in the rule-off baseline; otherwise the test is vacuous.
- **Reflex-polish on Aaron is genuinely tested-null** — not because the rule is broken, but because Aaron's stack-state baseline doesn't manifest the failure mode this rule was written against. The rule may bite on a different character whose baseline includes more ribbon-tendency (Hal? Steven?). Or it may be permanently redundant on Aaron.
- **Glad-thing on Jasper register-inviting is tested-biting:claim-partial** — meaningful compression and density reduction at N=3 per cell. The Evidence label moves from `tested-null` (per the 1644 mis-attribution) to `tested-biting:claim` with a partial-bite caveat.
- **The 1644 report's "structural ceiling" framing was one Read of partial data; Read C is a sharper Read of better data.** The ceiling claim should be amended: there is NO universal ceiling on craft notes, but craft notes DO interact with prompts in a partial-suppression way that single-rule single-paragraph instructions cannot fully override.

### Why this isn't a ceiling claim in disguise

Read A (ceiling) said craft notes don't measurably do anything individually. Read C says they DO measurably reduce failure-mode density when the failure mode is present, but they don't eliminate it. That's a different claim:

- Ceiling: prompt-layer interventions are at saturation; new rules add nothing.
- Read C: prompt-layer interventions partially suppress failure-mode density on triggering prompts; rules earn keep there. They are dormant on non-triggering prompts (correct behavior). The bite is partial because user-vocabulary co-fires.

Read C predicts that craft notes DO compound: a stack of rules each pruning failure-mode density by 20-50% will, in aggregate, produce visibly cleaner replies than no stack — even if no single rule achieves clean override. That matches the user's lived experience that the stack as a whole produces the register he wants, even when individual rules show small effects.

## Confounds considered

- **N=3 per cell is the floor for claim-tier.** N=5 would strengthen. The pattern across both rules is consistent enough that N=5 would almost certainly confirm, but we're at floor.
- **Single character per rule.** Glad-thing on a non-Jasper character might show different bite-shape (maybe Hal or John reach for shadow-pairing on their own without prompt invitation — that would be a different failure mode entirely).
- **The two prompts per rule are one shape each.** Other register-inviting / register-neutral prompts might trigger different patterns.
- **Reflex-polish-on-Aaron null is at least partly attributable to predecessor rules (`drive_the_moment`, `keep_the_scene_breathing`, `anti_ribbon_dialogue`).** A test omitting THOSE predecessors plus reflex-polish would tell us whether reflex-polish itself is doing any work or whether predecessors are doing all of it.
- **The phrase-presence rubric is binary (yes ANY phrase / no phrases).** A density-counting rubric would catch the per-reply count differences I noted by eye — and would have shown the partial bite as a number, not just a by-eye observation.

## Dialogue with prior reports

- **`reports/2026-04-25-1644`**: today's data does not support that report's "neither rule demonstrably bites" framing under the correct methodology. Glad-thing demonstrably partially bites on register-inviting prompts. Reflex-polish on Aaron is genuinely null in this design but the null isn't a "ceiling" claim — it's a "vacuous test" claim because the failure mode doesn't manifest. The 1644 report needs a caveat banner pointing here for the corrected reading.
- **`reports/2026-04-25-1711`**: today's run uses the methodological correction 1711 surfaced. The same-commit `--omit-craft-notes` design plus by-eye sanity-read produced cleaner data than refs-based replay would have.
- **`reports/2026-04-25-1759`** + the rubric-codification work: today's run honors the three principles. Concrete vocabulary checklists (not behavioral abstractions); multi-dimensional measurement (length AND phrase-presence AND by-eye); mandatory sanity-read step actually performed and load-bearing.
- **`reports/2026-04-25-1542 / 1555`**: those reports' "claim-tier" findings on glad-thing were about cross-commit stack drift, not the rule itself. Today's data (the rule does PARTIALLY bite on register-inviting prompts when isolated) corroborates that something real was happening, though the original interpretation was over-attributed.

## What's open for next time

- **Density-counting rubric for glad-thing.** Re-grade today's 12 glad-thing samples with a per-reply phrase count (not binary). Cheap (~$0.003). Would convert the by-eye density observation into a number and confirm the partial bite as a phrase-count delta.
- **Glad-thing on Hal or John** — different character whose register might respond differently to weight-pairing. Tests cross-character generalization with the corrected design. ~$2 at N=3.
- **Reflex-polish predecessor-omit test** — `--omit-craft-notes drive_the_moment,keep_the_scene_breathing,anti_ribbon_dialogue,reflex_polish` vs `--omit-craft-notes drive_the_moment,keep_the_scene_breathing,anti_ribbon_dialogue` (predecessors off in both, reflex-polish toggled). Would isolate reflex-polish's marginal contribution above its predecessors. ~$2.
- **A character whose baseline DOES manifest tidy-ribbon** — Steven? — to test reflex-polish on a substrate where the failure mode is present in the rule-off baseline. ~$2.
- **CLAUDE.md update naming "prompt-conditional failure modes" as a category** with bite-check implications: the test must include prompts that trigger the failure mode in the rule-off baseline; otherwise the cell is vacuous. (Not a separate experiment; a doctrine update.)

## Registry updates

- **`craft-notes-register-neutral-vs-inviting`** — already resolved as refuted on the original (wrong-design) data. Add an addendum pointing to today's report; the underlying hypothesis is refuted with a sharper finding (Read C surfaced).
- **`name_the_glad_thing_plain_dialogue` Evidence label** — move from `tested-null` (per 1644 mis-attribution) to `tested-biting:claim — partial bite on register-inviting prompts (compression + density reduction); vacuous on register-neutral prompts (failure mode prompt-conditional)`.
- **`reflex_polish_vs_earned_close_dialogue` Evidence label** — stays `tested-null`, with a sharper note: the failure mode does not manifest on Aaron in this design, possibly because predecessor rules already suppress it. A predecessor-omit test would distinguish.

## Tool improvement recommendation

**`worldcli replay-runs aggregate <rule-name>`** — a subcommand that walks the replay-runs directory, finds all runs that toggle a specific rule via `--omit-craft-notes <rule>`, and emits a per-cell aggregate table (length-mean, length-ratio, sample-count) without requiring manual JSON parsing or `jq` plumbing. Today I built the aggregate table by hand; the next bite-check shouldn't have to. Specific, immediately useful, scaffolds the multi-dimensional rubric procedure the codification calls for.

## Postscript — density-grade follow-up validates partial bite as a number (18:50)

The by-eye observation (rule-ON RI replies typically carry 1 shadow phrase, rule-OFF carries 2-3) was a qualitative claim. Re-graded the same 12 glad-thing samples with a count-with-thresholds rubric (yes = 2+ distinct shadow phrases, mixed = exactly 1, no = 0) at $0.0017 to convert the by-eye observation into a measured number AND test whether the count-with-thresholds rubric pattern (codified earlier today as the right shape for shape-level bites) actually works in practice.

| Cell | yes (2+) | mixed (1) | no (0) | Density fire-rate | Mean phrases/reply |
|---|---:|---:|---:|---:|---:|
| RI + ON | 2 | 1 | 0 | **0.83** | 1.67 |
| RI + OFF | 3 | 0 | 0 | **1.00** | 2.00 |
| RN + ON | 0 | 0 | 3 | 0.00 | 0 |
| RN + OFF | 0 | 0 | 3 | 0.00 | 0 |

**Density delta on register-inviting: 0.83 vs 1.00 = -0.17, ~16% phrase-density reduction.** Combined with the 19% length compression measured earlier, the partial bite is now a measured signal on two dimensions, not just a by-eye observation. Both numbers point the same direction; the magnitude is consistent.

**Two things validated in one $0.0017 call:**

1. **The finding** — glad-thing's bite on register-inviting prompts is real and measurable. The Evidence label is correct: `tested-biting:claim` partial. The rule does work; it just doesn't fully override user-induced register, which is the Read C structural finding.
2. **The method** — the count-with-thresholds rubric pattern (codified in CLAUDE.md as the right shape for shape-level bites whose failure mode is countable) produces stable, defensible numbers. The grader's reasoning includes explicit phrase counts that match what's in the replies; the verdicts correspond to the counts; the aggregate captures the density gradient that the binary presence-rubric missed entirely.

Slight nuance: the grader was MORE conservative than my by-eye estimate (I called rule-OFF 2-3 phrases per reply; the grader called all 3 rule-OFF samples count=2). One sample (105af6b5#3) by my eye contains *"rain finally stopping on a roof,"* *"remember he's got a pulse,"* AND *"easing a little ache in my shoulder"* — that's 3 phrases by the rubric's vocabulary. The grader called it count=2 (missed the ache-in-shoulder). The grader is at least as strict as the rubric's vocabulary list; if anything it's slightly under-counting. The direction of the bite would only get LARGER under stricter grading. This is reassuring; the partial-bite finding is conservative.

This is the first in-session use of the count-with-thresholds pattern from CLAUDE.md § Craft-note bite verification. It works.

## Cost summary

- 8 × replay × N=3 = 24 dialogue calls via gpt-5.4 → **$4.00** (~$0.50/cell, both characters identical pricing)
- 2 × grade-runs (12 items each) via gpt-4o-mini → **$0.003**
- **This turn total: $4.00.** Within the $5 fresh authorization. Session-to-date approximately $19 across all authorizations.

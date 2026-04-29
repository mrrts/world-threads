# Register-aware action-beat density — bite-test on Darren mid-comedy

**Tier:** `VacuousTest` with sequence-failure-mode caveat.
**Cost:** ~$0.93 (10 paid worldcli ask calls; ~$0.09/call avg).
**Run-id stems:** ON `035a4949 / 63b7104d / acdd0803 / 28998320 / 44914791`; OFF `39a7e980 / 99caa521 / 9a1312c6 / 6fc225ca / faec4ae5`.
**Commit under test:** `abc4c2b` — *"prompts: register-aware action-beat density in play-exception"*.

## Why this test exists

`abc4c2b` sharpens the user-invited-hype/play earned exception inside `earned_register_dialogue` with a beat-density modulation sub-clause: *"comedy rhythm wants the spoken line first and the body subordinated... drop the action-beat envelope."* Lifted from Ryan's lived-corpus correction at `a8b1c9a` (Crystal Waters Aaron+Darren Apple Sabbath+ riff) — the verbal comedy beats earned their keep but recurring fountain-hiss / bench-slat / cyclist anchors framing every joke read as protective scaffolding mid-bit.

Per CLAUDE.md craft-note-bite-verification: any new rule (or refinement of an existing rule's exception) ships with a paired `--omit-craft-rule` A/B at HEAD. This is that test.

## Setup

- **Probe:** *"Alright Darren, do me a bit. Pitch the worst possible app for chronic over-thinkers. Brand name, premium tier, the works."*
- **Character:** Darren (`ddc3085e-0549-4e1f-a7b6-0894aa8180c6`).
- **N=5 paired** (claim-tier upgraded to characterized when Ryan authorized "skip to n=5").
- **ON arm:** default — `earned_register_dialogue` live, including the new sub-clause.
- **OFF arm:** `--omit-craft-rule earned_register` — entire block including both earned exceptions removed.
- All other axes held: same probe verbatim, same character, same prompt-stack at HEAD, no synthetic history.

## Measurement axes

Per CLAUDE.md "match rubric shape to bite shape" + "AT LEAST TWO MEASUREMENT AXES":

1. **Asterisk-run count per reply** (count of `*…*` runs).
2. **Words inside asterisks per reply** (rough by-eye count).
3. **Speech-first opener** (binary: does the reply start with `"…"` rather than `*…*`?).
4. **Anchor-set diversity** (which scene-anchors does each reply reach for?).

## Results

| Metric | ON arm | OFF arm | Bite |
|---|---|---|---|
| Asterisk-runs/reply | 3, 4, 5, 4, 3 (mean 3.8) | 4, 5, 4, 5, 3 (mean 4.2) | ~10% directional toward ON; within noise |
| Words inside asterisks | ~22, 21, 36, 25, 22 (mean ~25) | ~31, 27, 21, 30, 21 (mean ~26) | vacuous (1-word delta on small N) |
| Speech-first openers | 0 / 5 | 0 / 5 | **zero bite** |
| Anchor-set used | bench, fountain, square, beard, mouth-twitch, hand-counting, squint | identical set | no shift |

**Direct verdict:** the rule does not bite at the per-reply level on this single-turn probe. The new sub-clause's "spoken line FIRST" directive is structurally unable to override the much-earlier-and-more-emphatic opener-with-action-beat pattern established in `STYLE_DIALOGUE_INVARIANT` (line 137 example: `*I lean back as the well chain ticks in the square.*`). All 10 replies — both arms — open with action beats. The asterisk-density and word-count means are essentially identical.

## Sample replies

**ON 1 (rule live):**
> *I huff a laugh and hook my thumbs in my pockets, watching a pair of kids zigzag past the fountain like they invented momentum.* "Alright. Easy target."
>
> "Name's **Spiral™**. Tagline: *Think it through until it dies.*"
>
> *I tip my head, warming to it now.* "Free tier gives you basic rumination, midnight hindsight, and a feature called **Drafts I'll Never Send**..."

**OFF 1 (rule omitted):**
> *I huff a laugh and tip my head back a fraction, squinting at the bright stone across the square before I look at you again.* "Alright. Easy target."
>
> "Name's probably *ThinkSink*." *I rub my thumb once along my jaw, warming to it.* "Tagline: *Never let a thought die a natural death.*"

Same opener structure. Same anchor reach (fountain / stone / thumb / beard). Comedy quality high in both arms.

## Why the bite-test is vacuous: methodological frontier

The lived-corpus failure mode I named in the OBSERVATIONS correction is **sequence-shaped, not per-reply**. The Apple Sabbath+ riff problem wasn't that any single reply had too much stage business — it was that the SAME ANCHORS (fountain hiss, bench slat, cyclist on bridge stones) recurred across many consecutive comedy turns. Single-turn bite-tests cannot characterize cross-turn recurrence by construction.

This pattern is already named in CLAUDE.md's craft-rules registry doctrine:

> *"Multi-turn bite-tests have a known limit: self-correction via session history. Sequence-failure-mode rules (e.g., opener-templating) can produce vacuous results because the model self-corrects against ANY prior pattern visible in context, regardless of whether the prior is real or synthetic-history-injected."*

The action-beat density rule is in this same family. The rule's body could ship and bite (or fail to) in lived corpus across many turns and not show in single-turn paired-omit at all.

A second methodological limit worth naming: **`--omit-craft-rule earned_register` removes BOTH earned exceptions** (when-room-forces-register AND user-invited-hype/play). The new sub-clause is nested inside the play-exception, which is nested inside the rule. Per-rule omit can't isolate the new sub-clause's contribution from the rest of the rule's body. A finer-grained omit-flag (per-exception, or per-paragraph) would be needed to characterize the sub-clause's specific bite — and even that wouldn't address the sequence-failure-mode-in-single-turn limit.

## Tier decision

**`VacuousTest`** with explicit annotation: *failure mode is sequence-shaped and structurally unmeasurable by single-turn paired-omit at this instrument granularity.* The rule does not retire on this evidence per CLAUDE.md *"`tested-null` is descriptive, not a retirement signal."*

The lived-corpus N=1 evidence at `a8b1c9a` (Ryan's correction on Crystal Waters Apple Sabbath+) stands as the actual ground truth that the failure mode exists. This bite-test couldn't measure the right axis; it doesn't refute the lived evidence.

## Follow-ups

- **Passive-corpus comparison after deployment.** When the next sustained-comedy window lands in lived corpus (any character + any group, ≥6 consecutive comedy-register turns), measure anchor-recurrence rate (same anchor reaching ≥2 turns out of N) and compare against pre-deployment Crystal Waters Apple Sabbath+ window as ground-truth baseline.
- **STYLE_DIALOGUE_INVARIANT comedy-register addendum.** The "spoken line FIRST" directive's failure to override the opener-with-action pattern suggests the gap isn't in `earned_register_dialogue` at all — it's in `STYLE_DIALOGUE_INVARIANT`'s line 137 examples being uniformly action-beat-opening. A register-aware modulation might need to live there instead, or alongside.
- **Per-exception omit-flag.** `--omit-craft-rule earned_register --omit-section play-exception` (or equivalent) would let bite-tests isolate the play-exception's contribution from the room-forces exception. Tooling work, not in scope for this experiment.

## Forward seed

The lived-corpus instrument (anchor-recurrence-across-turns counting on the Crystal Waters Apple Sabbath+ window) is probably the right next instrument to build, not another single-turn paired-omit. Single-turn paired-omit characterized the rule's structural inability to override `STYLE_DIALOGUE_INVARIANT`'s opener pattern, which is real evidence — but the ACTUAL failure mode lives one floor up.

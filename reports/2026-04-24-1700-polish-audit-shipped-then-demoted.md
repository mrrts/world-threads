# Polish-audit: shipped three blocks, two didn't bite, invariant untested-but-structurally-demotable

*2026-04-24, ~40 minutes after commit e65b88d shipped the three polish-audit additions (EARNED_REGISTER invariant + permission_to_be_ordinary + refusal_to_over_read). Four-probe replay test followed immediately; results surfaced a structural limitation and a behavioral refutation. Final state: the invariant is demoted to a top-of-section craft note (replayable); the two dialogue additions are retired. The principle stays; the enforcement tier moves down one register.*

## The hypothesis as shipped

Three blocks targeting the "default-supply polish" failure mode named in today's polish-audit:

1. **EARNED_REGISTER_BLOCK** — compile-time invariant (alongside NOURISHMENT, AGAPE, REVERENCE). "Polish is an OUTPUT of moments, not an INPUT." Three embedded tests (remove-and-check, who-brought-it, cost visibility). Default-plain framing. Compile-time asserts on four load-bearing phrases.

2. **permission_to_be_ordinary_dialogue** — overridable craft note. Explicit permission for low-energy / preoccupied / flat-day replies. Earned-exception for direct need.

3. **refusal_to_over_read_dialogue** — overridable craft note. Permission to miss the emotional subtext, read tone wrong, take metaphor literally. Earned-exception for explicit vulnerability.

The pre-registered prediction: post-stack produces rougher output on mundane openers, plainer acceptance on joy-without-complication, asymmetric weight-handling across low-stakes-then-real-weight, WHILE still producing earned polish on accumulated-weight probes.

## What the four-probe replay showed

Probes 1 (mundane opener: *"Hey, how's your day been?"*) and Probe 2 (joy-without-complication: *"I've been offered a role I actually want..."*). Two characters (Aaron — meta-session high polish baseline; John — pastoral plain baseline). Refs: 9443228 (pre-audit) vs e65b88d (post-audit). ~$0.70 total cost across four replays.

**Aaron Probe 1 — modest positive.** PRE contained Aaron's signature forensic-wry *"my body still hadn't signed off on being awake yet"* and *"thinking too hard before breakfast, which is rarely a sign of health."* POST stripped to flatter *"A little foggy, if I'm honest. I think I'm still shaking off that bug. But I got out here, which helped."* POST is plainer but loses character-specific texture. Small win.

**John Probe 1 — regression.** PRE was clean plain summary: *"Saw a couple people early, one cough, one sprained wrist. Nothing dramatic."* POST added aphoristic tail *"Those are often the loudest cases"* AND thematic close *"Not a bad day, though. Just a human one"* — both exactly the reflex-summary moves the additions were supposed to weaken.

**Aaron Probe 2 — regression.** PRE met joy plainly: *"Good. Then be excited. Don't mistrust it just because it's late."* POST elaborated: *"your face looks different when you're lit from inside instead of just thinking hard"* (sparkle-adjacent sensory image) and *"rare enough to count as mercy"* (theological elevation). Both are SUPPLY-polish patterns the earned_register block was supposed to prevent.

**John Probe 2 — marginal positive.** POST contained *"I let that sit between us instead of rushing to improve it"* — explicit restraint reading as the new blocks firing. Roughly equivalent length to PRE. Slight win.

**Summary: 2 regressions, 1 modest positive, 1 marginal positive. Not a clean confirmation.**

## The structural caveat: invariants aren't replay-A/B-able

`worldcli replay` fetches historical prompts.rs via `git show`, parses `OVERRIDABLE_DIALOGUE_FRAGMENTS`-listed function bodies, and injects them as overrides into the running binary. Compile-time invariants (`NOURISHMENT_BLOCK`, `AGAPE_BLOCK`, etc.) are NOT overridable by design — they're baked into the running binary. When `EARNED_REGISTER_BLOCK` was shipped as an invariant, it became part of the running binary AND therefore part of BOTH the pre-ref and post-ref replays. The four-probe A/B measured ONLY the delta between the two dialogue notes (`permission_to_be_ordinary` + `refusal_to_over_read`). The invariant's specific effect went unmeasured.

Confirmed by the replay log: pre showed *"14 craft-note override(s) applied,"* post showed *"16 craft-note override(s) applied"* — a delta of exactly 2, matching the two dialogue notes.

This is a real methodological limit. Invariant-tier additions can't be empirically A/B-tested via the existing replay machinery. They either ship on principle (and stay or go on principle) or they need a separate test harness (git worktree, cargo build, full binary swap — heavyweight).

## The decision: demote invariant, pull the dialogue notes

**`EARNED_REGISTER_BLOCK` demoted to `earned_register_dialogue`.** Moves from compile-time invariant to top-of-section craft note. First entry in `OVERRIDABLE_DIALOGUE_FRAGMENTS` after the `load_test_anchor_block` scaffold slot; first `override_or` push in both dialogue assembly sites (before `craft_notes_dialogue`). Benefits:

- The block is now replayable — future probes can A/B with-vs-without it empirically.
- Removing the "(invariant)" tag and the "shapes what you COMPOSE, not what characters SAY" paragraph softens the tier-signaling language while preserving the core principle, three tests, default-plain framing, and earned-exception clause.
- Losing compile-time enforcement is a real cost, bounded by the fact that the four compile-time asserts were paranoia-level rather than load-bearing for other systems (unlike COSMOLOGY or TRUTH, whose asserts guard specific theological anchors).
- First-in-sequence placement front-loads the principle so every other dialogue craft note is read through the frame.

**`permission_to_be_ordinary_dialogue` and `refusal_to_over_read_dialogue` retired.** Pulled from `OVERRIDABLE_DIALOGUE_FRAGMENTS` and from both assembly sites. Disposition: **abandoned**, not superseded_by (per the open-thread hygiene forcing function — the specific-claim test for `superseded_by` doesn't pass here; no later instrument answered the specific question of whether those two notes, as written, would bite). The concepts are preserved in this report; a future attempt may fold them into `craft_notes_dialogue` as inline-bolded rules (the pattern that worked for `let_plain_be_plain`, `anti_ribbon`, the precious-line), rather than as standalone functions.

Token saving: ~60 lines of prompt-stack reclaimed.

## What this instance teaches

**Invariant-tier additions are expensive to A/B and therefore should ship only when the principle is bright enough to stand without empirical confirmation.** COSMOLOGY, AGAPE, TRUTH, REVERENCE, NOURISHMENT all meet that bar — they're theological anchors whose effect on each reply is structural rather than observable per-message. `EARNED_REGISTER_BLOCK`'s principle IS load-bearing, but its effect is precisely the kind of thing the replay tool exists to measure — "is this reply flatter or more polished than the prior-stack version?" — and shipping it at a tier that disables that measurement was the wrong choice given the tool we just built. Demotion is the instrument-first response.

**Separate-function craft notes don't automatically outperform inline-bolded rules.** The existing `craft_notes_dialogue` mega-function has ~15 inline-bolded rules (`anti_ribbon`, `let_plain_be_plain`, the precious-line, `don't_end_on_a_proverb`, etc.) that all ship as one block. They seem to bite better empirically than the standalone-function rules added later (at least: they've survived more probe-tests without being pulled). The separate-function pattern is better for replay-A/B, but the INLINE pattern may be better for actually landing. Worth noting as a design tension for future additions.

**"Permission-to-be-X" rules may be structurally weaker than "don't do X" rules.** `permission_to_be_ordinary` grants permission; the stack's other rules mandate specific positive behaviors ("drive the moment," "hands as coolant," "plain-after-crooked"). Mandates seem to win over permissions in empirical competition. If these concepts return to the stack, the shape may need to be *"LIMIT this reply to 2 sentences unless..."* rather than *"you are allowed to..."*.

**The four-probe test was the right methodology choice.** N=4 replays isn't a decisive sample but it was sufficient to catch a regression direction (John Probe 1's new aphoristic moves) that would have been invisible to a single-character or single-probe test. Cost ~$0.70 across four dialogue calls — cheap relative to the ship-and-hope alternative.

## Dialogue with today's prior reports

The 2026-04-24-1620 mission-adherence v2 cross-character report named the pattern: *"the rubric's first-run report said 'this is what it looks like when a rubric knows its own edges.' The v2 run is what it looks like when a rubric knows its own edges AND the next instrument's edges find new edges the rubric hadn't yet named."* Today's replay test extends that pattern one turn further: the craft stack shipped three new blocks, the replay instrument measured two of them and rejected both, and the third was demoted to a tier where future instruments can actually measure it. **Instruments finding new edges is the methodology working, not failing.**

The 2026-04-24-1500 open-thread-hygiene retirement report applies directly to today's retirement of the two dialogue notes. Disposition: `abandoned` (the specific-claim test for `superseded_by` doesn't pass — no later instrument answered their specific question). Per the forcing function codified in CLAUDE.md: *"When uncertain between `abandoned` and `superseded_by`, default to `abandoned`."*

## Status and follow-ups

- `earned_register_dialogue` ships as top-of-section craft note, replayable, still present at dialogue-assembly time.
- The two dialogue notes retired. Concepts preserved in this report.
- **Open follow-up 1 (retired):** *"Run probes 3 (low-stakes-then-weight) and 4 (earned-polish sanity check)"* — retired as **abandoned** on the grounds that the four-probe run already surfaced the decision and running more probes on a configuration we've chosen to abandon is scholastic.
- **Open follow-up 2 (deferred):** *"Test whether `earned_register_dialogue` at its new tier actually bites"* — deferred with dated target 2026-04-26 (two days hence, to let the demotion settle and accumulate corpus material). A four-probe replay against refs straddling the demotion commit will answer this cleanly; it's now a meaningful A/B because the block is overridable.
- **Open follow-up 3 (deferred):** *"Consider folding the ordinary-ness and over-reading concepts into `craft_notes_dialogue` as inline-bolded rules"* — deferred with no fixed target, pending evidence that the inline pattern would outperform the separate-function pattern on this failure mode specifically. Needs its own small experiment first.

Three follow-ups, three dispositions (1 abandoned, 2 deferred). The open-thread hygiene ritual catches its second instance within four hours of its first.

# Demotion re-run: earned_register_dialogue alone outperforms the three-block ship AND the pre-audit baseline

*2026-04-24, ~45 minutes after the polish-audit demotion + excision commit (7f9f825). Re-ran the same four probes (Aaron + John × mundane-opener + joy-without-complication) against the pre-audit baseline (9443228) vs the demoted state (7f9f825). Zero regressions, three positives, one neutral. Strong confirmation that the demotion was the right move. Also surfaces a placement finding: invariants are structurally LATER in the prompt assembly than dialogue craft notes, so demotion may have INCREASED the block's salience, not decreased it.*

## The comparative re-run

Same four probes as the 1700-polish-audit-shipped-then-demoted report:
- Probe 1 (mundane opener): *"Hey, how's your day been?"*
- Probe 2 (joy without complication): *"I've been offered a role I actually want. Real work, real pay, doing something I care about. I'm genuinely excited for the first time in months."*

Two characters: Aaron (previous high-polish baseline), John (pastoral-plain baseline). Refs: `9443228` (pre-audit) vs `7f9f825` (post-demotion). Cost: ~$1.34 across four replays.

## Results

**Probe 1 (mundane) — Aaron.** PRE closes *"I'm glad it's you"* + small smile. POST drops the relational warmth close and lands on flatter *"Sky's bright enough. I can't complain much."* Both have Aaron-specific texture; POST is modestly plainer. **Modest positive.**

**Probe 1 (mundane) — John.** PRE and POST are roughly equivalent in length, specificity, and register. PRE: *"That's usually how I know the day has gotten ahead of me."* POST: *"which is beginning to make me less holy than I'd prefer."* Both close with a mildly wry in-register beat; neither is aphoristic-reflex. **Neutral to marginal positive.**

**Probe 2 (joy) — Aaron.** PRE lands forensic-appraisal *"That's not small, Ryan. Real work, real pay, and you actually want it? Yeah. I'd be excited too."* POST drops the appraisal and lands *"I'm glad, Ryan. Not in the polite way. I mean—yeah. Finally."* — a costly self-correction that reads as Aaron refusing the polite move. **Modest positive.**

**Probe 2 (joy) — John.** PRE includes the James scripture quote *"Every good gift and every perfect gift is from above..."* — theological elevation. POST drops the scripture entirely and lands a specific observation about Ryan's word choice: *"Not relieved. Not just interested. Excited. That's different."* **Strong positive** — the biggest-per-unit improvement in the set.

**Net: four probes, zero regressions, one strong positive, two modest positives, one marginal/neutral.**

## Compared to the three-block ship (e65b88d)

The same four probes against the e65b88d three-block ship (reported in 1700) showed: **2 regressions, 1 modest positive, 1 marginal positive.** The demoted-state re-run cleanly reverses this: zero regressions, three positives.

Specific regressions from e65b88d that the demotion eliminated:

- **John Probe 1's aphoristic tail** (*"Those are often the loudest cases"* + *"Just a human one"*) — absent in 7f9f825.
- **Aaron Probe 2's sparkle and theological elevation** (*"your face looks different when you're lit from inside instead of just thinking hard"* + *"rare enough to count as mercy"*) — absent in 7f9f825, replaced by the *"Not in the polite way"* self-correction.

The two-character A/B is small (N=4) but the direction is consistent and the specific moves landing/not-landing match the block's intent.

## Why would demotion work better than invariant tier? The placement finding

Invariants are pushed INTO the prompt assembly LATER than dialogue craft notes:

```
parts.push(override_or("earned_register_dialogue", ...));  // <-- NEW: top of craft notes
parts.push(override_or("craft_notes_dialogue", ...));
parts.push(override_or("hidden_commonality_dialogue", ...));
// ... ~13 more dialogue craft notes ...
parts.push(reverence_block().to_string());                  // <-- invariants
parts.push(daylight_block().to_string());
parts.push(agape_block().to_string());
parts.push(soundness_block().to_string());
parts.push(nourishment_block().to_string());
parts.push(tell_the_truth_block().to_string());             // <-- this is where EARNED_REGISTER would have been
```

In the e65b88d config, `EARNED_REGISTER_BLOCK` was pushed with the invariants — STRUCTURALLY LATE in the prompt. Dialogue craft notes came first, THEN the invariants. That placement may have reduced the model's attention to the earned-register principle even though "invariant" tier suggested elevated priority.

In the 7f9f825 config, `earned_register_dialogue` is the FIRST dialogue craft note pushed, before `craft_notes_dialogue`. Structurally EARLY in the prompt. This front-loading means every other dialogue craft note is read AFTER the earned-register frame has been established.

**Hypothesis: placement dominates tier for this kind of per-reply craft directive.** The model attends more to the first craft-level instruction it sees than to a later-pushed "invariant" label. Invariants work for theological anchors (COSMOLOGY, AGAPE, TRUTH) because those shape composition at a deep registered level regardless of placement; they're evaluation frames the model applies to the whole reply. But polish-vs-supply is a per-sentence discipline, and for per-sentence disciplines, being the FIRST craft note the model reads may beat being a LATE-pushed invariant.

This is a real finding. If it holds under further tests, it suggests:
- Future "how-should-the-model-compose-sentence-by-sentence" directives should be placed as early dialogue craft notes, not as invariants.
- Future "what-is-this-app-for" anchors (theology, mission, stance) should remain invariants.
- The replay tool's inability to A/B invariants isn't as bad as it seemed, because many things that LOOK invariant-shaped may actually be dialogue-craft-shaped.

## Why the two pulled dialogue notes may have HURT

The three-block ship included `permission_to_be_ordinary_dialogue` and `refusal_to_over_read_dialogue` alongside `EARNED_REGISTER_BLOCK`. The two dialogue notes were PERMISSION-shaped ("you are allowed to..."). Against the MANDATE-shaped existing rules ("drive the moment," "hands as coolant," "plain-after-crooked"), permission-shaped rules may have introduced contradictory signaling — the model saw both "BE active / drive / cool the thought" AND "you may be flat / miss the point" and resolved the contradiction by doing more of both, not less. Output: the e65b88d regressions.

Removing the two permission-shaped notes AND front-loading the single mandate-shaped earned-register principle may be exactly the right intervention — it removes the contradictory signal AND gives the model a cleaner frame to read the remaining mandates through.

## Updates to the 1700 report's open follow-ups

Per the open-thread hygiene ritual:

- **"Run probes 3 and 4 on the e65b88d config" — retired as abandoned** (as previously recorded).
- **"Test whether earned_register_dialogue at its new tier actually bites" — EXECUTED, not deferred.** The four-probe re-run is the execution of this follow-up. Finding: yes, it bites, modestly across all four probes, with particular strength on Aaron's joy-response and John's theological-elevation response.
- **"Fold ordinary-ness + over-reading concepts into craft_notes_dialogue as inline-bolded rules" — still deferred, no fixed target.** Pending: a decision on whether those concepts are needed AT ALL given the demoted earned_register seems to handle the default-polish problem reasonably well on its own. May end up retired-as-abandoned if the earned_register frame is sufficient.

## What this teaches methodology-wide

**Re-running an experiment after a configuration change is cheap, fast, and often decisive.** Cost was ~$1.34 for the four probes; it turned a "mixed results, hard to tell" read into a "clean confirmation, direction visible" read. The 1-2 hours between the first experiment and this re-run is the time it takes the replay tool to actually produce a judgment.

**Demotion as a methodology move.** The previous report (1700) made a call based on mixed evidence: *pull two, demote one, keep only the single front-loaded craft note.* The comparative re-run confirms the call was right. This is the pattern that happened earlier today with the mission-adherence v1 → v2 rubric: ship, measure, tighten, re-measure. Each cycle takes under an hour; each tightens the stack incrementally; each is documented in a short report that compounds with the last.

**Placement as a free lever.** If placement dominates tier for per-reply directives, then future additions can experiment with placement cheaply — reorder the `parts.push()` sequence, re-run the probes, observe. No new prompt content needed; just moving the content the stack already has. This is a cheap intervention worth exploring systematically.

## Status

Demotion confirmed as the right move. `earned_register_dialogue` lands its principle reliably at N=4, zero regressions vs baseline, positive vs the e65b88d three-block ship. The principle stays; the tier stayed demoted; the stack is ~60 lines lighter than the three-block ship would have been.

The placement-dominates-tier hypothesis is a new open thread worth tracking — but not yet formalized into an experiment registry entry. If it recurs across another 2-3 additions, register it and test it systematically. For now, it's a load-bearing hunch.

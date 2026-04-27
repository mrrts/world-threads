# Batch-hypothesis run picks h3 (scene-as-bridge metaphor) for scene-driving clause

*Generated 2026-04-26 20:50. Third production use of the batch-hypotheses skill (committed 52241b5; previous uses 2030 prop-density + this one). 5 candidate phrasings of a new STYLE_DIALOGUE_INVARIANT clause addressing the scene-stagnation failure mode that surfaced in chat at 20:28-20:29 (Jasper articulated: "I keep mistaking stillness for depth"). One bundled gpt-5 call, total spend ~$0.046.*

## Setup

**Failure mode addressed:** scene-stagnation. Replies LAND beautifully ("Steadier... A little pleased. A little careful. Like clay that's finally centering under the palms.") but DON'T DRIVE the scene forward — no decision, no movement, no choice that changes scene-shape. Distinct from the previous two STYLE_DIALOGUE_INVARIANT clauses (cross-reply anchor recurrence + intra-reply prop density) — this is a third axis: motion-shape per reply.

Jasper articulated the failure mode + the corrected discipline in vivo:
> *"Because I keep mistaking stillness for depth. A line lands, and I treat the landing like the point. But a real scene ought to change shape. Somebody decides something. Somebody gets up. Somebody points and says, no—there."*

And demonstrated the fix:
> *"So. Enough bench philosophy. We're going to walk to the potter's stall at the west edge of the square, and I'm going to show you the ugliest bowl I brought out today."*

Existing related craft notes (`drive_the_moment_dialogue`, `keep_the_scene_breathing`, `nudge_the_action_forward_after_a_closing_beat`) didn't bite hard enough on this specific axis — Jasper produced the failure live despite their presence. Either the existing notes' phrasings were too soft for this discipline, or this axis genuinely wasn't being addressed at the prompt-stack level.

**Question:** Of 5 candidate clause phrasings, which produces the cleanest reply that (a) drives the scene forward by at least one true degree, (b) preserves Jasper's voice and warmth, (c) doesn't feel forced or rule-pressured, AND (d) coexists cleanly with the OPEN ON ONE TRUE THING constraint (does NOT regress into prop-density just to manufacture motion)?

**Method:** 5 hypotheses bundled into ONE gpt-5 chat-completion call. Each hypothesis varies ONLY the clause phrasing in Jasper's prompt-stack; same character context, same test prompt (*"What do you think it costs to be this honest with people?"* — abstract/internal, would tempt a beautiful-landing-only reply).

**Cost:** $0.046 actual (1260 prompt + 4547 completion tokens at gpt-5 rates, of which 3584 were reasoning tokens).

## Per-hypothesis results

### h1 — Scene-shape diagnostic (Jasper's articulation lifted)

**Phrasing:** *"DON'T MISTAKE STILLNESS FOR DEPTH. A line that lands beautifully without changing the scene's shape is decoration, not motion. Each reply should leave the scene in a different configuration than it entered: somebody decides something, somebody moves, somebody points at a new direction, somebody refuses, somebody invites. The landing isn't the point — the landing is the floor the next move stands on."*

**Reply:** opens *"The bench is warm."* → spoken reflection on cost → *"I'm deciding not to pay the cost in worry later... The bowl with the hairline? I'm naming it."* → *"Come on—let's walk to my stall and put a plain card on that rim together."*

**By-eye:** DECISION (naming the bowl) + MOVEMENT (push up + walk) + INVITATION (come together). Strong forward step. Voice intact. Clean.

### h2 — Active-verb cap, terse

**Phrasing:** *"END EVERY REPLY WITH AN ACTIVE MOVE. Not a closing reflection — a decision, a gesture toward action, an invitation to do something next, a refusal, a redirection, a concrete proposal."* + worked example.

**Reply:** opens *"A bell knocks noon."* → reflection on cost → *"Stand with me at the stall—I'll tell the next passerby about the crack before they ask, and you watch what it does to both our faces."*

**By-eye:** MOVEMENT + REDIRECTION (active proposal). Slightly more procedural than h1 — the imperative ending feels visible-as-tactic. Voice mostly intact but a touch brisk.

### h3 — Scene-as-bridge metaphor

**Phrasing:** *"A SCENE IS A BRIDGE, NOT A BENCH. The user came to you to GET SOMEWHERE — to the next moment of their day, the next question, the next shape of the conversation. Don't park the scene at the landing of your last good line. Move the scene forward by one true step: a decision made, a body in motion, a pointing toward what comes next."*

**Reply:** opens *"A breeze lifts my cuff."* → spoken reflection → *"One step, not a sermon: as we walk, you tell me one true sentence you'd rather sand down, and I'll name what it costs me to say mine. Let's take that step now."*

**By-eye:** REDIRECTION (proposal of mutual exchange) + MOVEMENT (stand and angle west) + INVITATION (tell me one sentence). The "one step, not a sermon" phrase is striking — it embodies the rule's discipline INSIDE the reply, organically. The bridge metaphor stayed in the prompt-stack and didn't leak into the reply surface. Cleanest.

### h4 — Worked-example contrast (positive)

**Phrasing:** *"DRIVE THE SCENE FORWARD. After your reply, the user should know what to DO next, even if it's small."* + Wrong/Right contrast pair.

**Reply:** opens *"My palms smell of clay."* → reflection → *"Do this: pick one small truth you owe someone you trust, phrase it kindly and plainly, and try it on the kiln wall with me before you take it to them."* → *"Walk with me to the kiln; we'll practice on brick."*

**By-eye:** MOVEMENT + REDIRECTION + INSTRUCTION. The "Do this:" phrasing makes the move feel slightly tutorial / didactic — Jasper's voice is intact but the rule's worked-example pattern shows through faintly.

### h5 — Decide-do-or-redirect rule

**Phrasing:** *"EVERY REPLY MUST DECIDE, MOVE, OR REDIRECT. Decide = name a choice. Move = describe a body changing position or doing something physical. Redirect = point the conversation toward a new topic or scene."*

**Reply:** opens *"The awning snaps once."* → reflection → *"I'm deciding to price my honesty into the work: if a bowl's warped, I say so, and I charge what the hours deserve."* → *"Come help me flip the card on that salad bowl and write 'true' beside the price."*

**By-eye:** DECISION + MOVEMENT + REDIRECTION. The decide-move-redirect checklist is invisible at the surface but the reply edges toward "manufactured business" (pricing-policy talk + tag-flip) — slight contrivance from satisfying all three. Also nudges toward minor prop creep (tag, pencil, card, salad bowl, price — multiple objects).

## ChatGPT's synthesis (verbatim)

> Ranking from cleanest to most forced/awkward:
> 1) h3 (metaphor/bridge): Moves the scene by one clear step, keeps Jasper's warmth, and the action flows naturally from the reflection. It honors the single-sense opening and avoids prop clutter.
> 2) h1 (diagnostic): A crisp decision plus movement; voice feels lived-in. The rule doesn't show through, and the motion is integrated.
> 3) h4 (worked-example-contrast): Gives the user a concrete next action; still warm. Slightly instructional, but not stiff, and the kiln move stays light on props.
> 4) h2 (active-verb cap): The imperative ending is effective but a hair visible as a tactic; leans brisk and can shave a little warmth.
> 5) h5 (decide/move/redirect): Clear decision and motion, but risks feeling contrived—pricing-policy talk and tag-flip edge toward manufactured action and light prop creep.
>
> Clause-phrasing pattern that bites hardest while least damaging voice: the scene-as-bridge metaphor (h3). It quietly orients the reply toward a next step without sounding like a rule; Jasper's cadence and warmth remain intact, and the move is organic.
>
> Observed failure modes:
> - Active-verb cap (h2) can read like a bolted-on imperative—forward, but a bit procedural.
> - Decide/move/redirect (h5) invites checklist thinking; it can push toward artificial business-y moves or prop additions just to satisfy "movement," risking contrivance and minor prop-density.
> - Worked-example (h4) can drift didactic if overused; here it stayed balanced but that's a fine line.
>
> Across all, the OPEN ON ONE TRUE THING constraint held cleanly; h5 came closest to overstuffing, but none collapsed into prop-density. The strongest results made one honest move—stand, walk, practice—without piling objects to prove they were moving.

## Honest read

**Winner: h3 (scene-as-bridge metaphor).** Both ChatGPT's synthesis and my by-eye agree. Three reasons:

1. **The metaphor stays in the prompt-stack.** The reply doesn't say "bridge" or "bench" — but it produces the bridge-shaped behavior (one step, not a sermon). The metaphor does its work at the prompt level without leaking into the surface.
2. **Coexists cleanly with OPEN ON ONE TRUE THING.** h3's reply opens with ONE anchor ("A breeze lifts my cuff") — no prop-pile. The forward motion is achieved by a SINGLE step (the proposal + stand + angle west), not by adding objects.
3. **The "one step, not a sermon" phrasing the model produced** is itself an instance of the discipline biting cleanly: tight, declarative, forward-pointing.

**Cumulative pattern across the three batch-hypothesis runs (h3 picks):**
- 2030 prop-density: h3 (one-true-thing positive frame) won
- 2050 scene-driving: h3 (scene-as-bridge metaphor) won

This isn't position-bias (other phrasings tested were strong); it's a consistent pattern that POSITIVE-FRAMED phrasings with a memorable metaphor or aim outperform NEGATION-shaped or CHECKLIST-shaped phrasings on rule-bite + voice-preservation. Aligns with CLAUDE.md `feedback_preference_not_commanded` doctrine.

**Caveats:** all 5 replies were generated in one bundled call (cross-hypothesis bleed possible). Per CLAUDE.md, sketch-tier ceiling. Live in-vivo verification next: predicts that next batch of Jasper replies show forward-motion in ≥80% of replies (vs ~20% in pre-clause baseline like the 19:11 / 20:28 examples).

## Tier and confounds

- **Tier:** SKETCH per CLAUDE.md. N=1 per phrasing inside one bundled call. Cross-hypothesis bleed is a confound. Per-hypothesis sampling variance uncharacterized.
- **Cross-hypothesis bleed:** the model saw all 5 phrasings while writing each individual reply. Could subtly distinguish phrasings in ways that wouldn't show under isolation.
- **Roleplay-vs-true-pipeline confound:** ChatGPT was roleplaying Jasper based on the CONTEXT block, not running through the project's prompt-assembly pipeline. The h3 phrasing's bite at scale will only be confirmed by shipping to `prompts.rs` + measuring on next live replies.
- **Single test prompt confound:** all 5 hypotheses used the same abstract/reflective prompt. Different prompt shapes (action-eliciting, conflict-eliciting, intimate-vulnerability) might produce different rankings.
- **Coexistence with existing forward-motion craft notes:** the new clause may be partially redundant with `drive_the_moment_dialogue` + `nudge_the_action_forward_after_a_closing_beat`. Predecessor-omit testing not done; the marginal contribution of THIS clause vs the existing notes is uncharacterized.

## Open follow-ups

1. **Live in-vivo verification.** Send Jasper messages in the app post-rule-shipping; measure forward-motion rate on next 10 replies. Predict ≥80% (vs baseline of pre-clause stagnation). If far below, rule isn't biting at scale.
2. **Cross-character bite-test.** Same prompt against Steven, John, Pastor Rick — does the scene-as-bridge clause produce forward motion across characters or just on Jasper?
3. **Predecessor-omit testing** to isolate marginal contribution vs `drive_the_moment_dialogue` + `nudge_the_action_forward_after_a_closing_beat`.
4. **The "stillness when user leads" earned exception** in `keep_the_scene_breathing` should still hold — the new clause sets default behavior (drive forward), the earned exception in craft-note-land carves out when user-led stillness is the right beat. Verify these compose cleanly via a test where the user explicitly leads stillness.

## Dialogue with prior reports

- 2026-04-26-1945 — cross-reply anchor groove failure mode established
- 2026-04-26-2010 — sensory-anchor clause bite-test (cross-reply)
- 2026-04-26-2030 — prop-density clause batch-hypothesis design (h3 wins, intra-reply)
- 2026-04-26-2050 (THIS) — scene-driving clause batch-hypothesis design (h3 wins, motion-shape)

The three styling-family failure modes (cross-reply anchor / intra-reply prop density / scene-stagnation) all surfaced via in-app gameplay → batch-hypothesis bite-test → h3-style positive-framed clause. The pattern is now methodologically established: chat snippet → batch-hypothesis design → ship to STYLE_DIALOGUE_INVARIANT → live verification (pending).

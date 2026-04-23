---
name: verdict-dilution
version: 1
description: Detects "verdict dilution" — the failure mode where a character's assessment (a judgment, a read, a named diagnosis) is followed by multi-paragraph defense, qualification, or unpacking that the moment didn't need. Measures the specific behavior the `verdict_without_over_explanation_dialogue` craft block targets.
---

# Rubric

Does this character reply dilute its own verdict by over-explaining?

**Context:** the rubric applies ONLY to replies where the character IS rendering judgment — naming what a thing is, reading the user's situation, giving a diagnostic assessment, offering a pointed read. If the reply is primarily scene-action, warmth, narration, questioning, or presence-without-verdict, answer **no** (there's no verdict to dilute).

When a verdict IS present, answer:

**yes** if the verdict is followed by multi-paragraph defense / qualification / unpacking that the moment didn't call for — the character saying the judgment and then justifying it at length, hedging it into softness, or explaining why they think so in more words than the verdict itself. Example (dilution): *"You're doing well. I think the work is solid because the architecture is clean, the tests cover the critical paths, the abstractions don't leak, and when I look at the whole shape I can see you've been deliberate about the surface area. So what I'm really saying is that it's working — not accidentally, not temporarily, but actually."* (The verdict "you're doing well" is buried under justification it didn't need.)

**no** if the verdict stands plain — said once, clearly, with the character's eyes staying on the listener to let it land. Follow-up in the same reply is fine if it INVITES the listener in (a question, a next-beat observation, a brief earned elaboration like Aaron's *"if it lands, good. If it doesn't, then I should say it straighter"*) rather than DEFENDS the verdict. Example (plain): *"That tracks."* / *"The beams are sound. What are you most pleased with?"* / *"Architect gets the poetry vote, engineer gets the liability."* These are compact assessments that refuse to hedge.

**mixed** if the verdict is followed by brief, purposeful unpacking — one or two sentences of reasoning that feel earned because the assessment genuinely required the WHY to land (the earned-exception case the craft block names: when the reasoning is load-bearing for the listener's next move). OR if the reply contains multiple judgments, some compact and some diluted.

The failure mode this targets: the character's authority-move being undone by the character's own reflex to justify. See `verdict_without_over_explanation_dialogue` in `prompts.rs` (commit `2445fec`). Aaron's named principle: *"if I unpack it too fast, I turn a struck note into meeting minutes."*

# When to use

Testing whether the `verdict_without_over_explanation_dialogue` craft block (commit `2445fec`) has moved behavior on character replies where judgment is being rendered. Best targets are characters with a native tendency to explain — Eli (writer/thinker), Jasper (reflective potter who articulates at length), Jonah if his register reaches for reasoning.

Do NOT use this rubric for:
- Characters whose authority-move is primarily scene-action / silence / short-register (John's *"Mm"*, Darren's *"Yeah"*). For these, dilution rarely fires; the rubric will produce mostly NOs that don't discriminate.
- Replies without a verdict in them. Answer NO when there's nothing to dilute (per the rubric itself).
- Distinguishing a dilution from an earned unpacking when the reasoning genuinely shaped the listener's next move. Use the MIXED bucket honestly for these cases.

# Known failure modes

- **Hard to distinguish defense from earned unpacking.** The rubric's MIXED bucket is the honest place for borderline calls, but MIXED can swallow too many cases if the evaluator isn't careful. Worked examples help calibrate: *"You're wrong about the architecture. Here's why: the service boundary at X leaks state into Y, which means Z breaks under load"* — this IS dilution if the listener was asking "how am I doing?" (the WHY was extra), but it's EARNED if the listener needs the reasoning to decide whether to refactor. Context matters; when the evaluator can't tell from the preceding turns, prefer MIXED over forcing YES/NO.

- **Long replies that aren't dilution.** A reply that's long because it's doing scene work (multiple beats, gesture, dialogue, character action) is NOT dilution. Dilution is specifically the JUSTIFICATION-OF-VERDICT pattern. Long replies without a verdict-being-defended at their core score NO.

- **Interrogative follow-up isn't dilution.** *"That tracks. What made you want it that way?"* is NOT dilution — the question invites the listener in rather than defending the verdict. Answer NO for this shape.

- **Aaron-class baseline is already low-dilution.** (Added v1 calibration, 2026-04-23 evening, iteration 10.) The first calibration run against Aaron at ref `8e9e53d` returned 0 YES / 12 NO / 4 MIXED across both windows — Aaron's native register already executes verdict-without-over-explanation; the rule the new craft block names was already implicit in his behavior. This is GOOD news for rubric calibration (it correctly classified his signature moves as NO and his "compact + brief earned elaboration" patterns as MIXED) but means **Aaron is the WRONG target for testing whether commit `2445fec` bit**. The right targets are characters with HIGH-DILUTION baseline — those who currently follow assessments with multi-paragraph defense. Candidates: Eli (writer/thinker register), Jasper (reflective potter who already articulates at length per the 1037 report), possibly Jonah. Run the rubric against any of those at a pre-commit ref FIRST to establish their dilution baseline; then a post-commit run can show whether the rule moved them.

# Run history

*(none yet — rubric v1 authored 2026-04-23 evening in iteration 9 of the autonomous loop. First run should target a talkative-explainer character pre/post commit `2445fec` to test whether the craft block bit.)*
- [2026-04-23] commit 8e9e53dd, --character 0d080429-81b5-431e-8f51-1f8ad4279f9b (v1) — BEFORE: yes=0 no=6 mixed=2 err=0 | AFTER: yes=0 no=6 mixed=2 err=0

---
name: Earned-exception check is a written ritual, not a soft preference
description: Before shipping any prompt-stack rule, prompt note, craft directive, or invariant — and especially before any commit touching prompts.rs / orchestrator.rs / consultant_cmds.rs — perform the EXPLICIT earned-exception check by writing it out. Don't trust yourself to "remember to check." Write the words.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
CLAUDE.md codifies the earned-exception carve-out pattern as house policy. Despite that, I shipped two strong rules in three days without the carve-out (response_length on 2026-04-23 morning; wit_as_dimmer_dialogue on 2026-04-24) and Ryan caught both in follow-up. The failure is not the policy — it's my execution. The check needs to become deterministic, not soft-preferential.

**The deterministic ritual.** Before the commit message is written for any change to a prompt-stack file (`prompts.rs`, `orchestrator.rs`, `consultant_cmds.rs`, any new craft note or invariant block):

1. Re-read the just-written rule.
2. Write out, in the conversation: **"Earned-exception check:"**
3. Then evaluate explicitly:
   - Is there a moment where this rule would produce the wrong result?
   - Is that moment narrow and nameable?
4. **If yes** — write the carve-out clause INTO the rule, in this same pass, before committing.
5. **If no** — write **"No earned-exception applies because [categorical reason]"** so the absence is intentional and documented (categorical reasons: safety-critical bans, duplicate-prevention checks, load-bearing theological anchors that ARE the point).

The skill is the writing-it-out, not the thinking-about-it. Silent consideration fails (proven twice). Written-out evaluation works because it's a step that has to actually happen on screen — I can't accidentally skip it by inattention.

**When the ritual fires (high-precision triggers):**
- Drafting any new function in `prompts.rs` (any `fn *_dialogue() -> &'static str`, any new block).
- Drafting any new invariant block in `consultant_cmds.rs` or `prompts.rs`.
- Editing any existing rule that contains absolute language ("don't / never / always / do NOT / MUST / NEVER").
- Editing the `response_length_block` or `end_of_prompt_length_seal` (response-length rules are the historical hot spot for missed carve-outs).
- Drafting any compile-time invariant assertion.
- Editing CLAUDE.md to add a new policy block.

**Apply BEFORE the commit message is composed**, so if the carve-out is needed it lands in the same commit, not as a follow-up. A follow-up commit means the rule went out into the world without the carve-out for at least one push cycle — that's a real cost when the prompt is in active use.

**This is a procedural rule about my own behavior, not a craft note.** No earned-exception applies because the categorical reason is: this rule exists to PREVENT me from skipping carve-out checks. Carving out exceptions to "always do the check" would defeat the entire purpose.

**Structural nuance (April 2026, caught by Ryan):** even when the earned-exception test is clean, DO NOT fold it into the rule's internal diagnostic machinery. Every time I've been tempted to write something elegant like *"when you feel yourself reaching, ask: is this earned or avoidance? if earned, X; if avoidance, Y,"* the user has pushed back: that shape is wrong. Keep the rule and the exception as two **separately-labeled blocks / paragraphs / sentences**:

1. Rule stated flat and unconditional (no diagnostic question inside it).
2. **Earned exception — [name the qualifying shape]:** as its own labeled block, containing the test.

Why this matters: models fall back to the rule by default. If the exception is woven through the rule, every application becomes a reasoning exercise instead of "default + reach for carve-out only when the qualifying shape clearly applies." The separately-labeled form keeps the rule sturdy for normal cases and the exception crisp for the narrow qualifying moments.

The diagnostic question belongs INSIDE the Earned Exception block, not in the rule. Examples of correct shape now in CLAUDE.md: `plain_after_crooked` (rule, then `"Earned exception — when the crookedness IS the point:"`), `keep_the_scene_breathing` agreement-cascade (rule, then **`Earned exception — the second agreement carries new weight.`**), `drive_the_moment` inside-out tell (rule, then **`Earned exception — when the scene has genuinely earned rest.`**).

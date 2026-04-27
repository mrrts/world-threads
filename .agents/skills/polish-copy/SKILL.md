---
description: Review one user-facing UI string (toast, error, empty state, button label, modal, status line, schema migration message, etc.) and return KEEP / REVISE / REJECT with the rewrite if REVISE. Use when drafting new user-facing copy in the app, or when reviewing existing copy that might be too bureaucratic or too cute. Preserves meaning; reduces friction for a tired normal person at 8:30 AM. Lifted from a character's own articulation of the review discipline.
---

# polish-copy

Review ONE user-facing system message in the app. Improve it for human experience without changing its meaning. This skill exists because the app's mission (vivid, nourishing, in-world) is undermined if the UI copy surrounding the character voices feels bureaucratic or cute-for-its-own-sake.

## Method

For the given string, return exactly five things in order:

1. **Original message** (verbatim).
2. **Rewritten message** (if REVISE) or the same message (if KEEP) or a blank (if REJECT).
3. **Why this is better for a tired normal person at 8:30 AM** — name the specific friction reduced or the specific human moment the rewrite now meets.
4. **Risk check** — what meaning might have been softened, obscured, or made too cute? Be honest. If nothing, say nothing.
5. **Verdict** — one of KEEP / REVISE / REJECT. One word.

## Rules

- **Preserve the actual function and truth of the message.** If the original warns about data loss, the rewrite still warns about data loss. Warmth doesn't mean softening a real warning into a fake reassurance.
- **Make it clearer, warmer, and less bureaucratic** — not quirkier.
- **Do not become quirky for its own sake.** No forced whimsy. No personality showing off.
- **Do not add jokes** unless the original message is low-stakes AND delight would genuinely help (a "nothing here yet" empty state is a better candidate for gentle warmth than a "your account has been charged $49" confirmation).
- **Prefer language that reduces user friction, not language that shows off personality.** The test: does the user read it faster, with less effort, and know what to do next?
- **Assume the user may be tired, frustrated, distracted, or unfamiliar with technical terms.** "Migration complete" is bureaucratic; "Your world has been updated." is clear; "Your world just took a breath." is too cute.
- **If the best rewrite is very close to the original, say so.** Verdict KEEP. Don't reach for change because you're here reviewing.

## Earned exception — when a line is allowed to stand as it is

> *If the line is actually useful, or it lands in this character's mouth, or the scene genuinely wants the cleaner close, let it stand. The ban is on reflex polish, not on endings that earned their right to exist. Don't outlaw good endings because the model got addicted to fake ones.*

The test: *would my rewrite be an improvement, or am I reaching because I'm supposed to produce output?* If the latter, verdict KEEP. The skill's job is to catch bureaucratic drift and cute-for-its-own-sake drift; it is NOT to apply change for change's sake.

## When to use

- Drafting a new toast, modal, empty state, error message, confirmation dialog, loading state, or button label in the frontend.
- Reviewing copy you just wrote but aren't sure about.
- Reviewing existing copy that a user flagged as confusing or cold.
- Checking schema-migration messages, DB-error surfaces, import/export status lines — anywhere the app speaks to the user OUTSIDE of character dialogue.

## When NOT to use

- Character dialogue. Character voices have their own craft-stack in `prompts.rs` (voice-mirror, craft notes, register axes). Don't apply UI-polish discipline to an in-world reply — the rules are different and you'd flatten the register.
- Error message text that belongs to a third-party library (e.g. a Rust panic message). Usually wrap with context, don't rewrite the library text itself.
- Marketing copy, website copy, documentation headers. This skill is for IN-APP strings that a user encounters while using the product.

## Provenance

The review-discipline (5 steps + 8 rules) was articulated by one of Ryan's characters as a craft move they already know how to make. The earned-exception clause was articulated by the same character. Lifted verbatim into this skill. The character's test is particularly useful for catching the case where the model has "gotten addicted to fake endings" — reflex polish that makes every reply look the same.

## Output format

Respond in plain markdown with the 5 sections labeled. Don't offer more than one rewrite. If you genuinely can't decide between two, explain and let the user pick.

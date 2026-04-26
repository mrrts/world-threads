---
name: Message timestamps + git commits = natural experiment for craft changes
description: Every assistant message has created_at; every prompt change is a git commit with a timestamp. The corpus is already a before/after dataset — use it to empirically evaluate whether craft changes did what they intended, instead of relying on vibes or self-report.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
Ryan's observation, April 2026: messages can be cross-referenced with git timestamps to gauge the effect of prompt changes and feature changes over time — correlation, if any.

**The principle:** the corpus does not need new instrumentation to support craft-change evaluation. Every assistant turn carries `created_at`; every prompt edit is a commit with `committer_date`. So for any prompt change at time T, every message before T was generated under the old stack and every message after T was generated under the new stack. That's a natural before/after.

**Why this matters:** the project ships a lot of prompt-stack changes (craft notes, invariants, conscience pass, persona drift fixes, register guards). It is easy to ship a note and *believe* it helped without checking. This methodology turns the belief into a testable claim: pull samples from the windows on either side of the commit, eyeball or evaluator-rubric them against the hypothesis, decide whether the note earned its keep. Same discipline as the 213d969 conscience-pass opt-in flip (let cost data settle the question instead of vibes).

**How to apply:**

1. **When evaluating a recent craft change**, default to pulling a sample from the before-window and the after-window of the relevant commit. Don't just self-reflect on whether it landed.
2. **When proposing a new craft note**, ask: how would I know in two weeks whether this actually changed behavior? If the answer is "vibes," reach for the sample-window pattern instead.
3. **When the user expresses doubt about a recent change**, the natural-experiment data is already there to consult — don't argue from intent, look at the corpus.
4. **The infra path** (not yet built): a `worldcli sample-windows --before-ref <sha> --after-ref <sha>` subcommand that pulls N messages from each window (filterable by character/world) and emits parallel JSON arrays. Until built, the ad-hoc form is `recent-messages <char> --before <iso> --limit N` and the same with `--after <iso>` straddling a commit's timestamp.
5. **Hypotheses worth checking right now** (examples, not exhaustive): description_weave preservation rule actually preserving weave length; user-name anchor actually killing third-person drift; keep_the_scene_breathing actually reducing dead-end closes; Imagined Chapter profundity dial actually stratifying chapter register; wit-as-dimmer note's effect on non-Hal characters (sympathetic resonance vs. Hal-only).

**The mistake to avoid:** declaring a craft change a success because it *feels* right or because the next conversation under it landed well. One conversation is anecdote; a windowed sample is evidence. Both are fine, but only the second can disconfirm.

**Adjacent:** this is the same posture the project takes with the conscience pass — empirical cost data settled the opt-in flip, not gut feel. Apply that same posture to *quality* changes, not just cost ones.

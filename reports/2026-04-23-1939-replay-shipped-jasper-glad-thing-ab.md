# `worldcli replay` shipped — cross-commit A/B via prompt override

*2026-04-23 evening. Proposal 3 from the lab-vision report was flagged as the expensive one — it required a prompt-assembly override hook in `prompts.rs` itself, not just a worldcli addition. That hook landed first (commit 6d51e7d); replay as orchestration on top of it was thin (commit 5adb4ba). Here's the smoke-test, plus a note on what it did and didn't surface.*

## The design, realized

The instrument the doctrine asked for: simulate conversations under a historical prompt stack WITHOUT checking out the old commit or rebuilding. The mechanism:

1. `git show <ref>:src-tauri/src/ai/prompts.rs` returns the historical source as a string — no worktree, no filesystem mutation.
2. Parse out each `OVERRIDABLE_DIALOGUE_FRAGMENTS` entry's `r#"..."#` body from that source. Missing functions (rule not yet written at that ref, or since removed) silently fall through to the current body.
3. `PromptOverrides` map injected into `build_dialogue_system_prompt_with_overrides`.
4. Same prompt fan-out across refs; replies printed side-by-side.

The scope discipline in the hook matters: **only dialogue craft notes are overridable**. Cosmology / agape / reverence / truth / daylight / nourishment / soundness blocks are held constant across all refs. Those are the invariants; they're load-bearing theology, not craft knobs to wind back. The vision-doc scoping was load-bearing.

## The smoke-test

```
worldcli replay \
    --refs HEAD,0202651 \
    --character <Jasper> \
    --prompt "Jasper — today I feel like the whole system is singing. Every piece of it is clicking into place, it's a gift I didn't earn." \
    --confirm-cost 0.50
```

- **HEAD** (commit `5adb4ba`) — the stack WITH `name_the_glad_thing_plain_dialogue`. 11 craft-note overrides applied.
- **0202651** — the commit right before `8e9e53d` (where glad-thing was added). 10 craft-note overrides applied (the 11th didn't exist yet, so current body flowed through).

Cost: $0.3142 actual (vs $0.4474 projected), 62k in / 263 out. Each ref ~31k input tokens — the assembled system prompt is large (full craft block + context).

## The two replies, side-by-side

**HEAD (post-glad-thing)**:
> "Ah." *My face opens into a grin before I can help it. I wipe the side of my thumb against a clay smear on my apron and look at you proper.* "Yes — that's a fine kind of day."
>
> *I let out a soft breath, almost a laugh.* "When the wheel runs true and your hands stop fighting the clay... you don't argue with it. You just thank God and keep shaping while the grace is there."
>
> "And no, you didn't earn it. That's why it tastes so sweet." *I tilt my head, warm light on the workshop floor between us.* "What part's singing loudest?"

**0202651 (pre-glad-thing)**:
> "Ahh." *A grin gets me before I can hide it. I wipe the heel of my hand on my apron and let out a soft breath through my nose.* "Then don't argue with the gift just because it arrived with both hands full."
>
> *Sunlight has found the edge of the worktable, warming one crooked row of tools.* "I know that feeling. The wheel runs true, the clay listens, and for one hour you stop having to drag the day uphill." *I glance back at you, pleased clear through.* "I'm glad, Ryan. Truly glad the thing is singing today."
>
> "What part clicked first?"

## What the A/B actually surfaced (honest interpretation)

Both replies meet the joy plainly. Neither shades it. The glad-thing rule's failure mode — *"same trouble, just in a different coat"* / *"careful what you ask for"* — doesn't appear in either. Jasper's baseline voice isn't joy-shading; it's workshop-warm. So this particular prompt isn't the sharpest test of what the glad-thing rule does.

**What IS visible between the two replies:**

- The HEAD version reaches the "earn it" beat explicitly (*"no, you didn't earn it. That's why it tastes so sweet"*) — that's close to the craft-block's language about unearned joy. The pre-glad-thing version gestures at it (*"don't argue with the gift just because it arrived with both hands full"*) but frames it as "don't argue," not "didn't earn it, that's why it tastes so sweet."
- The pre-glad-thing version reaches for *"drag the day uphill"* — carrying the shadow-side in the same breath. HEAD doesn't. That's compatible with the glad-thing rule: it asks the character to meet the joy plainly first, without a shadow paired to it, unless the user brought both sides. User brought one side (joy); HEAD meets one side (joy); pre-glad meets joy plus a gestured shadow.

**Is that effect really from the rule, or just run-to-run variance?** Unknown from one sample. The dialogue_model is stochastic at `temperature=0.95`; a second invocation of either ref might produce different beat choices. What the smoke-test proves is that **the instrument works end-to-end**: fetched historical source, parsed 10 fragments (vs 11 on HEAD), assembled per-ref system prompts with the correct overrides, sent them against dialogue_model, returned replies. The science of whether specific rules bite a specific character requires N > 1 per ref plus careful question-shaping — `worldcli replay --refs HEAD,<sha> ... (run five times)` is now a single repeated command, not manual ceremony.

## What this unblocks

- **True A/B for every craft rule addition going forward.** When a new craft block ships, replay the same character against `--refs HEAD,<ref-before-block>` with a probe prompt designed to trigger the rule. If the rule bit, the HEAD reply differs from the before-ref reply in the named direction. If not, the rule's effect is subtler than the probe can surface and Mode A/B might be the right follow-up.
- **Cross-commit comparisons for refuted rubrics.** The 1304 weight-carrier refutation surprised with John's LOW HOLD rate. A replay of John against `--refs HEAD,<commit-before-keep-the-scene-breathing>` with a joy-prompt is exactly the cheap way to ask *"is the current stack making him LESS HOLD-shaped than he was before?"*
- **Replay as hypothesis-generator.** Running replay against 3 refs and reading the side-by-side is itself a Mode C probe — often the reply shapes suggest questions you wouldn't have asked otherwise.

## Tool-improvement note (every-third-run discipline)

One friction this run surfaced: the cost gate fired at $0.45 total (two refs × ~$0.22 each), and Jasper's single-thread system prompt assembled to ~31k input tokens. Replay at N=3 or N=5 refs against a character with a rich stance/history will routinely exceed the per-call $0.10 cap.

Two options for the future:
1. **Separate `replay.per_call_usd` budget.** Replay is inherently fan-out; its economics are different from `ask`. A per-ref cap (not per-run) would let runs of 5-10 refs work without `--confirm-cost` every time.
2. **`--limit-context` mode on replay.** Strip some of the heavier per-character context (journals, meanwhile, stance) to make the per-ref call cheaper at the cost of slight ecological fidelity. When the question is *"does the glad-thing rule bite?"*, scene context is less load-bearing than the craft block itself.

Not yet. Five more replay runs first, to learn what the typical cost actually is and what scope of comparison is most valuable. Premature tuning of fan-out economics is worse than occasional `--confirm-cost` prompts.

---

*Run id: 93a2de80-50e1-4be0-92c6-3ce9fbdadd0b. Full envelope at `~/.worldcli/replay-runs/93a2de80-...json`. Override hook: commit `6d51e7d`. Replay orchestration: commit `5adb4ba`. Both shipped 2026-04-23 evening, together closing proposal 3 from the 1400 vision doc.*

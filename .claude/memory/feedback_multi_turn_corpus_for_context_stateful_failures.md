---
name: Scan multi-turn corpus separately from single-turn for context-stateful failures
description: When investigating a failure mode that might be context-stateful (model perpetuates its own broken pattern from in-context prior), single-turn worldcli runs UNDERESTIMATE the rate. Use the app's messages table for honest characterization.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
When characterizing how often a model failure-mode actually occurs in WorldThreads, **the worldcli runs/ corpus is single-turn and will underestimate the rate of any failure-mode that's context-stateful** — i.e., where the model perpetuates its own broken pattern once it appears in the in-context prior.

**Why:** worldcli `ask` calls don't carry session memory by default. Each call's response is independent. The actual app's `messages` table holds multi-turn conversations where the LLM sees its prior replies as context, and certain failure-modes (notably anything sequence-shaped — opening-line drift, fence-mismatch cascades, register-drift after a single mis-fire) cascade once one instance lands.

**How to apply:** when scanning for a failure-mode where the question is "how often does this happen in lived use?":

1. **First-pass against worldcli runs/** is fine for cheapness but the result is a LOWER BOUND, not the actual rate.
2. **Always also scan `~/Library/Application Support/com.worldthreads.app/worldthreads.db`** (`SELECT content FROM messages WHERE role='assistant'`) for the actual lived rate.
3. **For context-stateful failures specifically**, also test the perpetuation hypothesis: bucket hits by thread; compare pre-first-hit rate to post-first-hit rate within hit-threads. Dramatic asymmetry confirms perpetuation; uniform distribution suggests independent failures.
4. The cost-benefit on shipping a check changes substantially when the failure perpetuates — first-failure detection + in-context prior repair has much higher leverage than per-turn detection because catching ONE failure breaks the cascade for the rest of the thread.

**Worked example (2026-04-29):** opening-fence-on-action failure (action content opening with `"` and closing with `*`).
- worldcli runs (709 single-turn): 0/709 = 0% — would have suggested no failure-mode existed
- App messages (1284 multi-turn): 30 hits = 2.34% baseline
- Perpetuation test: 0/637 pre-first-hit (0%) vs 27/490 post-first-hit (5.51%) — confirmed cascading
- Worst thread: 23 hits across 781 messages from one initial failure that cascaded for 261 subsequent messages
- Detection design implication: catch first-failure-per-thread rather than per-turn; fixes the cascade root, not symptom

The single-turn-only scan would have shipped a "no failure detected" finding that was honest within scope but missed the actual failure-class entirely. **Always check the multi-turn corpus too when the failure could be context-stateful.**

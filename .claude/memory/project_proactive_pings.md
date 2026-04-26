---
name: Proactive Pings feature constraints
description: Tuning rules for the Proactive Pings feature (character-initiated unsolicited messages)
type: project
originSessionId: 8206c4d3-2e14-4b4e-810e-17311f80fde8
---
Proactive Pings: the character sends the user an unsolicited message based on world-tick state, mood chain, open loops, and elapsed time since the last user message.

**Consecutive cap: 2 pings max without a user reply.** After 2 unanswered pings, the character goes silent in its thread until the user responds. Enforced by `threads.consecutive_proactive_pings` + hard check in `try_proactive_ping_cmd`; counter resets automatically in `create_message` when any user message lands.

**Randomized cooldown between pings.** Between any two pings on the same thread, enforce a random gap drawn per-check from [4h, 14h]. Re-rolled on each sweep — the hard floor guarantees two pings can never land back-to-back, and the upper bound spreads them naturally across a session. `roll_ping_cooldown_secs` in `chat_cmds.rs`.

**Curated angle seed per call.** Each ping call picks one random entry from `PROACTIVE_PING_ANGLES` (18 distinct occasions — "you saw something on a walk," "a half-finished thought from your last conversation," etc.) and injects it as the subject-setting seed in the final system anchor. Without this seed, two pings close in time converge on identical content even with distinct mood chains.

**Why:** The value of proactive pings is that they feel like a real person reaching out — not a bot. Two near-identical messages minutes apart immediately break the illusion. The randomized cooldown + curated angle pool together ensure each ping is both well-spaced and distinctly framed.

**How to apply:** When tuning further, don't lower the hard floor below ~3h unless the angle pool is dramatically larger. If users report dupes, first inspect the logged angle (`[Proactive] angle = ...` in log::info) — convergence usually means too-small an angle pool for the observed frequency.

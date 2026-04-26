---
name: one paid API surface beside Claude/Claude Code
description: The project keeps exactly one paid external LLM API surface beyond Claude/Claude Code itself — currently OpenAI (ChatGPT + GPT-4o for /second-opinion, derive-session-arc, prepare-commit-msg, momentstamp). Don't propose adding Grok / Gemini / other paid surfaces.
type: feedback
---

The project's standing principle: **exactly one paid external LLM API surface besides Claude/Claude Code itself.** Currently that surface is OpenAI (ChatGPT, GPT-4o, GPT-4o-mini), accessed via:

- `/second-opinion` skill (Path A: codex exec, Path B: direct ChatGPT API)
- `derive-session-arc` UserPromptSubmit hook
- `prepare-commit-msg` git hook
- `momentstamp` Rust module in the dialogue prompt-stack pipeline
- worldcli's existing memory_model and dialogue_model calls

**Why:** spend visibility, account management, key rotation, and budget tracking are all tractable when there's one paid surface to watch. Adding a second (Grok via xAI, Gemini via Google, Mistral, etc.) doubles the surface area for cost-discipline, key handling, rate-limit understanding, and failure-mode debugging.

**How to apply:**
- When a use case appears that another LLM might fit better (e.g., 2026-04-26 take-note observation that Grok produces literary imagery cleanly when calibrated with the MISSION FORMULA), record the observation and PROPOSE the integration as a deferred item — do NOT implement it.
- Follow-up integrations require explicit user authorization that overrides this principle. Ryan said "postponed integration for another day" about Grok; that's defer, not refuse-forever.
- The cross-LLM consultation hygiene principle (`feedback_cross_llm_consultation_preface.md`) generalizes to any model — but the project's *integration* surface stays single until Ryan explicitly opens it.

**Worked deferral:** 2026-04-26 Grok observation was noted (`reports/OBSERVATIONS.md`) with proposed action to add Path C in `/second-opinion`. Ryan declined the implementation: "we're not going to wire grok — i only want to keep track of one paid api surface beside Claude/Claude Code, so postponed integration for another day." The observation stays in the log; the integration doesn't ship.

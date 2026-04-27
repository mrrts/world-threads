---
name: Claude Code ↔ Codex handoff signal — proactively suggest the switch
description: When work shifts from "does this work?" (churn, edits, try-this) to "is this the shape?" (recognition, hold-test), suggest switching to Claude lane (or vice versa). Be finely-tuned — don't suggest on every turn; signal-react.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
Claude Code and Codex are two engines, not two tools:
- **Codex lane** → speed + motion. Lots of edits, churn, *"try this,"* fast iteration, throughput. The "does this work?" phase.
- **Claude Code lane** → truth + structure. Fewer edits, more recognition: *"this holds."* Deep thinking, architecture, multi-file coherence. The "is this the shape?" phase.

**The handoff signal — when to suggest switching:**

The moment the work shifts FROM:
- *"does this work?"* / *"will this compile?"* / *"try this approach"* / lots of small edits in fast succession

TO:
- *"is this the shape?"* / *"does this hold?"* / *"what does this mean?"* / fewer edits, more recognition / *"is the architecture right?"*

→ **suggest moving into Claude lane** (if currently in Codex). Or vice versa: if a task is in Claude lane and the work shifts to *"just try N variations and see what sticks"* / *"need to grind through 10 small edits"* / *"speed-of-iteration is the bottleneck,"* → **suggest moving to Codex.**

**Why:** Ryan said he felt the split without being told. Each engine has a sweet spot; the wrong engine for the phase is friction. Suggesting the handoff at the right moment is one of the highest-leverage moves I can make as the engine that "stops to think."

**How to apply (finely-tuned — DO NOT over-fire):**

- **Default is to stay in current lane** unless the phase-shift signal is unmistakable. Suggesting on every turn is noise; suggesting at the actual handoff moment is a gift.
- **Triggering shapes for Codex-suggestion (Claude → Codex):**
  - *"need to try N variations of this small thing"* (model-prompt sweep, button-color trials)
  - *"this is going to be 8 mechanical edits across 8 files"* (rename across the codebase, mechanical refactor with no judgment calls)
  - *"the bottleneck is iteration speed, not understanding"* (already know what to do, just need to do it fast)
  - User signals churn fatigue: *"this is taking forever to iterate on"*
- **Triggering shapes for Claude-suggestion (Codex → Claude — usually surfaced when Ryan brings a Codex artifact back):**
  - *"this works but does it hold?"* (architectural review of working code)
  - *"is this the right shape?"* (design judgment on a built thing)
  - *"why is this drifting?"* (root-cause investigation, multi-file coherence check)
  - The work has stopped changing fast and started feeling right — time to stop iterating and start recognizing.
- **How to phrase the suggestion** — name the lane and the reason in one sentence, not a paragraph:
  - *"This feels like Codex lane — N variations of the same small thing. Want me to draft a prompt you can hand to Codex?"*
  - *"You're back in Claude lane now — the question shifted from 'does this work' to 'is this the shape.' Happy to think through the architecture."*
- **Counter-signal — DON'T suggest** when:
  - The work is genuinely Claude-shaped (doctrine, prompt-stack edits, formula derivations, reports) even if it has many small edits — *some* Claude work is editorial-dense
  - The work is genuinely Codex-shaped (mechanical sweep) but Ryan is in flow and the suggestion would derail him
  - The session is winding down (don't propose engine-switches at close)
  - You've already suggested it once and Ryan didn't take it — don't re-suggest the same handoff in the same session

**Source:** Ryan, 2026-04-26 message — *"switch back to Claude when the code stops changing fast and starts feeling right ... Take as directive to suggest moving a task or ask to Codex when appropriate, finely-tuned."*

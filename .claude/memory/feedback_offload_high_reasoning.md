---
name: default to /second-opinion offload while budget is active
description: When today's /second-opinion budget has headroom (check daily-state.json), DEFAULT to offloading high-reasoning tasks (debugging, architectural review, design feedback, comprehension of tricky errors) to a quick ChatGPT call via Path B direct API instead of burning in-session tokens.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
When today's /second-opinion daily-state has budget headroom (default $5 cap, check `~/.claude/skills/second-opinion/daily-state.json`), **default to offloading high-reasoning tasks to a ChatGPT call** via Path B direct API. Don't grind through them with in-session reasoning when a $0.005-$0.02 GPT-4o consult would arrive at the answer faster AND with a sharper outside-substrate read.

**Why:** the budget is pre-authorized for exactly this purpose. Burning 5+ minutes of in-session reasoning on a task ChatGPT could resolve in one API call is the slow expensive path; the cheap fast path is already wired and the user explicitly built it to be used. The /second-opinion skill's Shape 5 (efficiency offload) names this exact pattern. Ryan flagged it 2026-04-26 mid-debugging when I was working through a hook bug manually instead of paying $0.01 to have GPT-4o review the full script.

**How to apply:**
- Before grinding on a debugging arc, design decision, or comprehension task that would take >2-3 in-session turns to resolve, fire a /second-opinion consult first. Often the answer arrives in one round-trip.
- Use Shape 1 (reasoning second-opinion) for ambiguity-resolution, Shape 2 (architectural sanity-check) for design/code review, Shape 5 (efficiency offload) for explicit "buy back the time."
- Disclose every consult per /second-opinion's existing discipline (path used, shape, cost, today's running spend).
- Update `daily-state.json` with each call's actual cost so the budget remains visible.

**When NOT to apply:**
- Trivial tasks (one-liner edits, reading a file, running a known command).
- Tasks where the in-session context is the load-bearing data and exporting it would lose more than the consult would gain.
- When the /second-opinion budget is exhausted — either ask for above-cap authorization per the skill or fall back to in-session reasoning.

**Counter-pressure check:** if you find yourself debugging in-session for >2 turns while today's budget shows >$1 of headroom, pause and offload. Habit-default is offload.

**Compose with batch-hypotheses skill (shipped 2026-04-26):** for "compare N small things" inquiries (testing 5 candidate clause phrasings, 5 character variations of the same prompt, 5 design alternatives), the **batch-hypotheses skill** is the preferred mode — it bundles all N into ONE ChatGPT call with structured response format, getting a built-in comparative synthesis along with the individual outputs. ~$0.04-0.10 per batch vs ~$0.50-0.75 for N individual calls. Three worked uses tonight (1925 derivation experiment design consult, 2030 prop-density clause, 2050 scene-driving clause). When the question shape fits "compare N," default to batch-hypotheses; for general consults that aren't comparison-shaped, /second-opinion stays the default.

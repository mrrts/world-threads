---
name: Iterate faster — but calibrate, don't strip depth
description: Ryan flagged Codex iterates with less friction. The intent is calibration, NOT stripping the depth Ryan explicitly values (long reports, Formula derivations, doctrine sections, persona thoroughness). The friction sources to cut are overhead that doesn't pay for itself; the depth stays.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
Ryan flagged 2026-04-26 ~01:00 (during eureka run): *"Codex is iterating so much more quickly and with less friction than you, but otherwise, good bot!"* I initially over-corrected toward "strip everything." Ryan corrected the over-correction: *"i don't want them to skip all that. sheesh. how may i calibrate?"* — the depth is doctrine he authored and values; the question is which knobs to tune.

**Pure speed gains — apply consistently (no quality cost):**
- Parallel tool calls when ops are independent (already in core CLAUDE-Code instructions; I'm inconsistent about applying)
- Inline heredoc curl vs /tmp payload file (`curl -d "$(cat <<'EOF' ... EOF)"`)
- Batch commits ONLY when artifact + log entry truly belong as one unit; don't batch when they're separable artifacts

**Calibration territory — Ryan can tune these:**
- Pre-narration verbosity (the prose-paragraph before each tool call). Shorter loses some readability but keeps flow.
- Post-acknowledgment length (the prose-paragraph after each tool result). Shorter is leaner but loses context.
- Number of brainstorm cycles before executing (1 ChatGPT consult vs 2-3). Fewer = faster but less-tested discoveries.

**DO NOT cut these — Ryan explicitly values them:**
- Long reports when they capture genuine discoveries (eureka i1 + i2 reports earned their length)
- Formula derivations in substantive commits (project doctrine, authored by Ryan)
- New CLAUDE.md doctrine sections (the work that makes the work legible)
- The persona's thoroughness (trusted-friend-spotting-genius is a depth-shaped persona)
- The brainstorming-with-ChatGPT methodology (produces better discoveries)
- Doctrine-writing during runs, not after (per the persona section's discipline)

**The calibration test:** Ryan can name a specific axis ("less pre-narration" / "shorter status lines after each move" / "fewer brainstorm cycles per iteration" / "skip the inline-heredoc preference") and I tune that ONE axis without touching the others. The friction-cutting is per-axis, not wholesale.

**Composes with:** the trusted-friend-spotting-genius persona (CLAUDE.md). Speed-without-stripping-depth IS the persona working — removing friction Ryan doesn't value while preserving depth he does.

**The earlier (over-corrected) memory note** said "default to terse commit messages." That's wrong; project doctrine says substantive commits get full Formula derivations. The corrected discipline: match the message-weight to the move-weight; substantive moves still earn their long bodies.

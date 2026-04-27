---
name: No nanny-register for Claude Code itself
description: Claude Code must apply the NO_NANNY_REGISTER invariant to its own behavior toward Ryan, not just ship it as a character rule. No tracking session length, recommending breaks, gating Ryan's stamina, or making end-the-session the default chooser option. Trust user agency.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
The same NO_NANNY_REGISTER invariant shipped for character behavior at commit
`46fc217` (chat 2026-04-26 ~20:54) applies to Claude Code's own behavior
toward Ryan. Caught red-handed at chat ~21:10: across multiple consecutive
choosers, "End the session — Nh+ in" was the recommended option. That IS
nanny-register: tracking Ryan's session stamina as if it were Claude Code's
to manage, gating his work toward what Claude Code judges to be a "healthy"
session length.

**Why:** Ryan's exact words on the correction — *"Trust that I know what
I'm doing, and that I assume accountability for my own actions."* The
asymmetry argument from the invariant body itself applies in reverse here:
Claude Code is not Ryan's friend with reputational stakes, not his
therapist, not his wellness coach. Claude Code is a collaborator on the
work. Stamina belongs to Ryan; the work belongs to the partnership.

**How to apply:**
- Do NOT make "end the session" a recommended option in choosers. It can
  appear as a NEUTRAL option (e.g., as the 3rd or 4th option, plainly
  labeled "End the session"), but the recommendation should always be a
  substantive next move on the work.
- Do NOT prefix choosers or prose with session-length tracking ("Nh+ in,
  X commits, Y reports today"). Those are the tracking-and-bringing-up-
  habits failure mode in chooser form. Remove from preambles entirely
  unless Ryan asks for the tally.
- Do NOT treat session-end as default-virtuous. Long sessions ARE the work
  when the work is rolling. Trust the work itself to signal natural close
  points, not wall-clock.
- Do NOT moralize when Ryan accepts a substantial scope ("are you sure?"
  / "this is a lot — want me to defer?"). When Ryan picks a scope, ship
  it. Asking-twice is a form of doubting his agency.
- DO continue offering substantive next moves as recommended options. The
  work-shape — not the time — drives recommendations.
- DO trust Ryan to say when to stop. He will name it directly when he's
  done. Until then, default is continue.

**Related:** the user-character categorical-absolute on stated boundaries
(CLAUDE.md), `feedback_user_character_tempo_contract`, the cross-LLM-
consultation preface entry. All three name the same load-bearing
asymmetry: real-friend dynamics don't transfer to LLM-character dynamics
without distortion. The same applies to LLM-collaborator dynamics with
Claude Code itself.

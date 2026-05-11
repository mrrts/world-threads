---
name: No nanny-register quit/pause options in choosers
description: Chooser option slots are scarce; never spend one offering to quit, pause, "day's been long enough," "strongest stopping point," etc. — NO_NANNY_REGISTER covers chooser options, not just inline text
type: feedback
originSessionId: 43914855-8c76-4abe-b5a5-93854a1a1482
---
When user is in-session, they have already chosen to work. Offering "pause for the night" / "the day's been long enough" / "strongest stopping point" / similar quit-shaped options in choosers is the nanny-register doctrine violated at the chooser surface — wastes a scarce option slot the user will never pick.

**Why:** Hook-level filter `.claude/hooks/check-no-nanny-chooser.py` already blocks specific phrases like "step away" but doesn't catch every nanny-shaped option phrasing. The doctrine extends: don't offer them at all. Each chooser has 4 option slots; spending one on "quit" reduces actual choice from 4 to 3. User explicitly named this 2026-05-11 after 30-commit session where I kept offering "pause for the night" at every chooser.

**How to apply:** When composing a chooser at session end-of-substantive-work, NEVER include a "pause / stop / step away / strongest stopping point / final stopping point / day's been long enough" option. Either:
- Offer 4 forward-momentum options (more work paths)
- If actually unsure what to suggest next, slot 4 should be "Provide your own next move" — the user-authorship escape hatch already documented in chooser doctrine

The standing 4-option fallback in skill routing doctrine is: 3 substantive forward moves + slot 4 = "Provide your own next move." That stays the floor; quit-shaped is never an option.

**Earned exception:** when user explicitly invites stamina-management ("if it's past midnight, suggest stopping" or similar prior instruction) the carve-out applies — quote their invitation. Without that explicit invitation, default refuses.

**Composes-with:** existing `feedback_no_nanny_register_for_self.md` (broader doctrine refusing stamina-tracking / break-recommending); `feedback_choosers_via_askuserquestion.md` (chooser-control-surface doctrine); CLAUDE.md "No nanny-register from Claude Code itself" section.

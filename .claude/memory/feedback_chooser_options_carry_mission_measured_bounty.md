---
name: Chooser options carry mission-measured dollar bounty
description: Founding-author Mode A correction caught 2026-05-08 mid-Imago-Dei arc — every chooser option must carry a bounty value so the cost/reward shape is visible before the user picks. Stamina belongs to the user; cost-visibility belongs to the user too.
type: feedback
originSessionId: 1834560a-7f2d-47fa-8bf6-191e0f4e00d9
---
When emitting AskUserQuestion choosers in /play, /seek-sapphire-crown, /auto-commit, or any skill that compounds bounty across moves, every option must carry a mission-measured dollar bounty value (e.g., "[+$300]") in the option label. The user reads bounty as a calibration of the cost/reward shape before picking.

**Why:** Founding-author caught me omitting bounty values in `/seek-sapphire-crown` Move 7 (and possibly elsewhere) — *"don't forget to include mission-measured dollar bounties in the chooser options."* Bounty values communicate: (a) skill-body assessment of move's substantive scope (small bounty → small move; large bounty → arc-shaping move); (b) implicit cost-magnitude framing the user uses to decide; (c) consistency with /play's accumulating-bounty mechanic that treats moves as economic acts. Omitting bounty is a quiet drift toward generic-chooser-without-stakes-visibility.

**How to apply:** Every chooser option from skills with bounty mechanics gets a bracketed bounty value in its label, e.g., `"Run W4 cross-provider [+$600]"`. Bounty scaling guidance: $100-300 for cheap-but-substantive moves (single corpus check, single smoke probe, methodology-prep file); $400-800 for moderate moves (one synthesis report, one small bench-run, fixture authoring); $1000-2000 for substantial moves (claim-tier replication, sketch-tier matrix); $2500-3500 for full-ladder pushes / Sapphire-firing-audit-cycle moves; $3000+ for Sapphire firings themselves. The "Recommended" tag still goes on the highest-information move; bounty is orthogonal to recommendation. Composes with `feedback_choosers_via_askuserquestion.md` (every-turn-chooser law) and `feedback_no_nanny_register_for_self.md` (no quit-shaped defaults).

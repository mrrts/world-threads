---
name: Chooser-option bounties must be nonzero and leverage-calibrated to the current arc
description: Ryan correction 2026-05-11 — every chooser option's [+$N] bounty must be nonzero AND the magnitude reflects mission-leverage on the active development arc, not flat-$0; sharpens the existing chooser-option-carries-dollar-bounty doctrine
type: feedback
originSessionId: 43914855-8c76-4abe-b5a5-93854a1a1482
---
**Rule:** Every chooser option's `[+$N]` prefix carries a **nonzero** dollar amount, and the magnitudes are **relative** — they reflect how high-leverage that option is with respect to the **current development arc**, not an absolute or flat-$0 baseline.

**Why:** Surfaced 2026-05-11 during the web-deployment Phase 2 thread-through arc. Three consecutive choosers had shipped with all options at `[+$0]` — the bounty became a uniform decorative tag rather than the cost/reward signal it's supposed to be. Ryan's correction (verbatim): *"all of the chooser options currently have $0 bounties. They should all be nonzero and the magnitude should be relative to how high-leverage the option is with regards to the current development arc"*. The doctrine (`feedback_chooser_options_carry_mission_measured_bounty.md`) had been satisfied formally (every option has a `[+$N]`) but defeated in spirit (when every N=0, no signal). The bounty is supposed to make cost/reward **visible before the user picks**; uniform $0 zeroes out the entire signal channel.

**How to apply:**

1. **Every option gets a nonzero `[+$N]`.** No `$0` even when "the work is local with no API spend" — the bounty is a **mission-leverage measure**, not just an API-cost projection. Local-only work still has mission-leverage on the active arc.

2. **Calibrate relative to the current arc.** The active arc determines the leverage axis. If the arc is *web-deployment toward multi-user readiness*, options that unblock or land that invariant get high `$N`; options that are tangential maintenance get low `$N`. Example calibration on the 2026-05-11 web-deployment arc:
   - Schema migration that lands the user_id-everywhere invariant fully (39/39 tables) → high (e.g., $400-600)
   - Phase 2 query-threading that activates write-paths on already-scoped surfaces → mid-high (e.g., $250-400)
   - api-server auth_middleware threading (connects auth → queries on web side) → high if next dependency, mid otherwise
   - Read-path WHERE filtering pilot → mid (preparatory, not yet load-bearing)
   - Read-side helper threading WITHOUT filtering → low-mid (signature-prep, no behavioral change)

3. **Use the spread to communicate the arc shape.** If three options span $50/$300/$800, the user can see at a glance which is the highest-leverage move. Tight spreads (e.g., $200/$220/$280) communicate "all roughly equal in this arc"; wide spreads communicate "one of these matters substantially more than the others right now."

4. **Distinct from API-cost bounties.** Originally the `[+$N]` prefix was used in `/play` HUD contexts as projected API spend (per `feedback_play_hud_and_bounties_every_turn.md`). In day-to-day choosers without /play running, treat the bounty as **mission-leverage in dollars** — same notation, different referent. The discriminating test: in /play HUD context, the bounty maps to budget; in ordinary chooser context, the bounty maps to arc-leverage.

5. **Composes with the no-nanny-register chooser doctrine.** Quit/pause options are already refused (`feedback_no_nanny_quit_options_in_choosers.md`); all four options should be substantive forward moves with substantive leverage signals — a $5 option for "pause here" wouldn't fix this anyway because the pause option itself shouldn't exist.

**Worked example — broken vs corrected on the 2026-05-11 web-deployment arc:**

Broken (what shipped immediately before the correction):
```
[+$0] Phase 2 thread-through — batch-5 tables
[+$0] api-server auth_middleware user_id threading
[+$0] SELECT-path WHERE user_id filtering pilot
[+$0] List/show-helper read-path threading for batch-4
```

Corrected (what should ship):
```
[+$400] Phase 2 thread-through — batch-5 tables          ← lands write-path completeness on remaining 5 tables; closes write-side invariant
[+$550] api-server auth_middleware user_id threading     ← connects Phase 0 auth → Phase 2 queries on web side; high-leverage unblocker
[+$200] SELECT-path WHERE user_id filtering pilot        ← preparatory; turns column into enforcement boundary
[+$80]  List/show-helper read-path threading for batch-4 ← signature-prep with no behavioral change yet; lowest leverage
```

The spread signals to Ryan that auth_middleware is the highest-leverage move on this arc right now; batch-5 query-threading is solidly load-bearing; the SELECT pilot is preparatory; the helper threading is mostly bookkeeping. He can choose accordingly with the leverage visible before he picks.

**Test:** if every option's `[+$N]` is `[+$0]`, or if all four `$N` values are within ~20% of each other across substantively-different-leverage options, the bounties have become decorative and the discipline has failed.

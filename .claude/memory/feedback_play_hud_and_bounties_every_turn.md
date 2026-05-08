---
name: /play HUD + per-option dollar bounties non-negotiable every turn
description: Mode A correction 2026-05-07 evening — Ryan's reminder that /play / /seek-sapphire-crown turns help him most when the HUD (turn / bank / move) prints AND every chooser option carries an explicit [+$N] bounty value so cost/reward shape is visible before he picks
type: feedback
originSessionId: 7bd5b2ab-f614-460c-a8b7-9f38b75c9524
---
**Rule.** Every /play and /seek-sapphire-crown turn ends with: (a) a HUD box at the top showing turn / bank / move-summary; (b) an `AskUserQuestion` chooser where every option label carries an explicit `[+$N]` bounty value before its descriptive prose. No exceptions for "cleaner-looking" choosers or skill-routing chooser bypasses.

**Why.** Ryan said directly 2026-05-07 evening: *"please honor the play skill's ui contract — it helps me to see the dollar bounties per chooser choice and the hud."* This was a Mode A correction. The HUD lets him track turn cadence + bank state across the day; the per-option bounties let him see cost/reward shape before picking, especially when options vary in API spend. Choosers without bounties make every option visually equal-weight when they're not.

The doctrine memory `feedback_chooser_options_carry_mission_measured_bounty.md` already stated this rule but I had been letting it drift on recent /seek-sapphire-crown turns where the chooser label included a phrase like "(+$200)" but not for every option. Ryan's correction names the discipline at full strength: every option, every turn.

**How to apply.**

- HUD box format (use exactly):
```
╭───────────────────────────────────────────────────────────────╮
│ TURN N  ·  BANK $X,XXX  (+$Y last move)                        │
│ ARC: <one-line context>                                        │
│ MOVE: <what just happened>                                     │
╰───────────────────────────────────────────────────────────────╯
```

- Chooser format: every option's `label` field should start with `[+$N]` followed by the option name. Example: `[+$300] Pursue WS3 (UI boundary truth) for the third work-shape`. The `description` field carries the explanation.

- Bounty calibration (mission-ranked): higher bounties for moves that advance the load-bearing path-to-Sapphire (new work-shapes; cross-substrate extension; codex consult); lower bounties for within-work-shape strengthening or memory updates; dollar amounts should reflect API spend AND mission-advance, not just one or the other.

- Refuses-options-doctrine still holds: no nanny-register options ("stop here", "rest", "are you sure"); no fake/decorative options; option 4 reserved for user-authored direction.

- This rule applies even when the chooser is mostly mechanical (memory updates, status checks). The bounty discipline is what makes the chooser legible — without it the cost/reward shape is hidden.

**Worked example correction.** TURN 259's chooser had only one option labeled with a parenthetical bounty ("(+$300)") and three without; TURN 260's chooser had none at all. Ryan corrected at TURN 261: *"please honor the play skill's ui contract."* Going forward: every option, every turn, no exceptions.

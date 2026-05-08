---
name: HUD bank/turn drift from play-state — read ground truth at session-open + periodically reconcile + name drift honestly
description: Mode A correction 2026-05-07 TURN 268: across long sessions and parallel sessions, session-local HUD bounty-incrementing drifts from .claude/play-state/current.json ground truth because crown-firing-jewels and parallel-session bounties update play-state independently; read play-state at session-open and periodically reconcile; name drift honestly when surfacing
type: feedback
originSessionId: 7bd5b2ab-f614-460c-a8b7-9f38b75c9524
---
**Rule.** When invoking /play or /seek-sapphire-crown skill turns, read `.claude/play-state/current.json` at session-open to get ground-truth `turn` + `bank` + `crowns` + `jewels`. Use those as HUD baseline. Periodically (every ~5-10 turns or whenever a crown fires on a parallel arc trajectory) re-read play-state and reconcile. When the local HUD has drifted from play-state, surface the drift in HUD with explicit naming — do NOT silently overwrite or invent a "what should the bank be" number.

**Why.** 2026-05-07 surfaced a $38,250 / 2-turn drift between session-local HUD ($353,950 / TURN 267) and play-state ground truth ($392,200 / TURN 269). Cause: I read play-state at session-open (TURN 251 at $351,100) and incremented locally per option-bounty awarded by my chooser-picks. Across the same 24-hour window, parallel sessions earned crown-firing jewels (Crown 11/12/13/14/15/16) AND ordinary bounties on different arcs (kavod-Transfiguration arc Moves 1-11; Resurrection arc Moves 1-12; Ascension arc Moves 1-3). Those updates went into play-state but NOT into my session's local HUD — because the play-state file is the authoritative ledger, and turn-keeper writes happen via hooks/tools my session doesn't synchronously observe.

The drift is structural, not a calculation error. My session's bounty-incrementing was internally consistent for the moves I authored from a stale baseline. The honest move is to name the drift when it's discovered, reset baseline to play-state truth, and continue with the corrected baseline rather than retrofit-re-narrate or pretend my numbers were authoritative.

**How to apply.**

- **At session-open:** read play-state once and cache the baseline.
  ```bash
  python3 -c "import json; d=json.load(open('.claude/play-state/current.json')); print('turn:', d.get('turn'), 'bank:', d.get('bank'), 'crowns:', len(d.get('crowns',[])), 'updated:', d.get('updated_at'))"
  ```

- **At HUD-print time:** the HUD printed in the response should reflect ground-truth + this-turn's bounty. Do NOT print a bank that's only locally incremented across multiple turns without reconciliation.

- **Reconcile every ~5-10 turns OR when a parallel-arc crown fires.** Re-read play-state. If the bank has moved beyond what local-incrementing predicts, the delta is from parallel sessions / crown-jewels. Reset baseline to play-state truth.

- **Surface drift honestly when discovered.** Don't silently retcon. The disclosure itself is an apparatus-honest move per `feedback_apparatus_honest_earns_and_refuses.md`. Use HUD-with-explanation format:
  ```
  ╭───────────────────────────────────────────╮
  │ HUD-vs-PLAY-STATE RECONCILIATION DISCLOSURE│
  │ Local HUD said: TURN X, BANK $Y           │
  │ Play-state truth: TURN X', BANK $Y'       │
  │ Drift: ΔT, Δ$                              │
  │ Cause: <named structural reason>           │
  ╰───────────────────────────────────────────╯
  ```

- **Do NOT modify play-state directly from session.** The file is updated by hooks / tools / parallel sessions; direct session writes risk stepping on turn-keeper hooks. Read-only at session-level.

- **Bounties earned via chooser-pick get added** to bank by some hook OR by play-state update mechanism (exact mechanism not visible from session). Trust the file as ground truth; my session's claimed bounty deltas may or may not have been written yet — surface this uncertainty when relevant.

**Earned exception.** When a session is short (<3 turns) and the baseline is fresh, local incrementing without re-reconcile is fine. The discipline applies when sessions cross many turns OR span periods where parallel-session crown-firings are likely.

**Worked example (2026-05-07 TURN 268 verification).**
- Local HUD: TURN 267, BANK $353,950 (started at TURN 251 $351,100; +$2,850 across 16 turns of authored moves)
- Play-state: TURN 269, BANK $392,200 (last updated 21:07 UTC at Crown 16 fire)
- Drift: -2 turns, -$38,250
- Cause: parallel sessions on kavod-Transfiguration arc + Resurrection arc + Ascension arc earned bounties + Crown 13/14/15/16 jewel-firings updated play-state across the day; local HUD never re-read
- Disposition: surfaced disclosure HUD; reset baseline to $392,200; continued with explicit HUD-vs-truth alignment in subsequent turns

**Cross-references:**

- `feedback_play_hud_and_bounties_every_turn.md` — parent discipline (HUD + per-option bounties non-negotiable)
- `feedback_apparatus_honest_earns_and_refuses.md` — parent doctrine for surfacing-drift-honestly
- `.claude/play-state/current.json` — the ground-truth ledger

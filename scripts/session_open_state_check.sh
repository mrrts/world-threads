#!/usr/bin/env bash
# session_open_state_check.sh — read play-state ground truth + recent commit trajectory
# at session-open per feedback_hud_play_state_reconciliation.md discipline.
#
# Why: across long sessions and parallel sessions, session-local HUD bounty-incrementing
# drifts from .claude/play-state/current.json ground truth because crown-firing-jewels and
# parallel-session bounties update play-state independently of session-local accounting.
#
# Usage: from project root, run `./scripts/session_open_state_check.sh`
#
# Output: play-state turn / bank / crowns count / last_move + last 5 commit trajectory +
# current branch + HUD discipline reminder.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

echo "═══════════════════════════════════════════════════════════════"
echo " session_open_state_check — HUD baseline read at session-open"
echo "═══════════════════════════════════════════════════════════════"
echo

# Play-state ground truth (reads via python3 to a temp script to avoid quoting hell)
if [ -f .claude/play-state/current.json ]; then
  python3 scripts/_session_open_play_state_reader.py
else
  echo "  WARNING: .claude/play-state/current.json not found"
fi

echo
echo "git state:"
echo "  branch:  $(git rev-parse --abbrev-ref HEAD)"
echo "  HEAD:    $(git log -1 --format='%h %s' | cut -c1-100)"
echo

echo "recent commit trajectory (last 5):"
git log --oneline | head -5 | sed 's/^/  /'

echo
echo "═══════════════════════════════════════════════════════════════"
echo " HUD discipline reminder per feedback_hud_play_state_reconciliation.md:"
echo " - session-local HUD bank+turn drifts from play-state across long sessions"
echo " - reconcile every ~5-10 turns OR when parallel-arc crown fires"
echo " - surface drift honestly with disclosure HUD"
echo " - do NOT modify play-state from session"
echo "═══════════════════════════════════════════════════════════════"

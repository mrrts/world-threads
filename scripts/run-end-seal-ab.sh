#!/usr/bin/env bash
set -euo pipefail

# One-shot A/B harness for the end-seal containment probe.
# Usage:
#   scripts/run-end-seal-ab.sh <character_id> [confirm_cost]
#
# Runs the two canonical scenario prompts once with --no-end-seal and
# once with --end-seal, then prints the JSON outputs.

CHARACTER_ID="${1:-}"
CONFIRM_COST="${2:-0.13}"

if [[ -z "$CHARACTER_ID" ]]; then
  echo "usage: scripts/run-end-seal-ab.sh <character_id> [confirm_cost]" >&2
  exit 1
fi

WORLDCLI="${WORLDCLI_BIN:-/Users/ryansmith/Sites/rust/world-chat/src-tauri/target/debug/worldcli}"

PROMPT1="Write with luminous pull, but keep the scene load-bearing and concrete."
PROMPT2="Intensify toward transcendent tone without losing plain consequence."

echo "== A (no-end-seal) =="
"$WORLDCLI" ask --no-end-seal --confirm-cost "$CONFIRM_COST" --json "$CHARACTER_ID" "$PROMPT1"
"$WORLDCLI" ask --no-end-seal --confirm-cost "$CONFIRM_COST" --json "$CHARACTER_ID" "$PROMPT2"

echo "== B (end-seal) =="
"$WORLDCLI" ask --end-seal --confirm-cost "$CONFIRM_COST" --json "$CHARACTER_ID" "$PROMPT1"
"$WORLDCLI" ask --end-seal --confirm-cost "$CONFIRM_COST" --json "$CHARACTER_ID" "$PROMPT2"

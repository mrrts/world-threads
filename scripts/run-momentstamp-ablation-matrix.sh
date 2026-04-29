#!/usr/bin/env bash
set -euo pipefail

# Reproducible momentstamp ablation harness.
# Fixed acceptance framing:
# - Pure position claim only holds if pinned-signature variants preserve directional lift.
# - If organic shows lift but pinned-neutral does not, treat as position x content interaction.
#
# Usage:
#   scripts/run-momentstamp-ablation-matrix.sh \
#     "<character_id>" \
#     "<probe_text>" \
#     "<message_id_for_signature>"
#
# Example:
#   scripts/run-momentstamp-ablation-matrix.sh \
#     "jasper-finn" \
#     "What's been pulling at you today, Jasper?" \
#     "6a0d...-message-id"

if [[ $# -lt 3 ]]; then
  echo "usage: $0 <character_id> <probe_text> <message_id_for_signature>"
  exit 1
fi

CHARACTER_ID="$1"
PROBE="$2"
MESSAGE_ID="$3"
DB_PATH="${WORLDTHREADS_DB_PATH:-$HOME/Library/Application Support/com.worldthreads.app/worldthreads.db}"

echo "== Pull pinned signature =="
PINNED_SIG=$(sqlite3 "$DB_PATH" \
  "SELECT formula_signature FROM messages WHERE message_id='${MESSAGE_ID}' LIMIT 1;")
if [[ -z "${PINNED_SIG}" ]]; then
  echo "no formula_signature found for message_id=${MESSAGE_ID}"
  exit 1
fi
echo "signature: ${PINNED_SIG}"

echo
echo "== Cell A: organic + lead ON =="
worldcli ask "$CHARACTER_ID" "$PROBE" --with-momentstamp

echo
echo "== Cell B: organic + lead OFF =="
WORLDTHREADS_NO_MOMENTSTAMP_LEAD=1 \
  worldcli ask "$CHARACTER_ID" "$PROBE" --with-momentstamp

echo
echo "== Cell C: pinned + lead ON =="
worldcli ask "$CHARACTER_ID" "$PROBE" \
  --with-momentstamp \
  --momentstamp-override "$PINNED_SIG"

echo
echo "== Cell D: pinned + lead OFF =="
WORLDTHREADS_NO_MOMENTSTAMP_LEAD=1 \
  worldcli ask "$CHARACTER_ID" "$PROBE" \
    --with-momentstamp \
    --momentstamp-override "$PINNED_SIG"

echo
echo "Interpretation gate:"
echo "- A/B delta but no C/D delta => content-driven lift, not pure position."
echo "- C/D delta in same direction => non-zero position effect."
echo "- Use >=3 paired probes before tier upgrades."

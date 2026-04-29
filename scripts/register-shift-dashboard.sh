#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORLDCLI="${WORLDCLI:-$ROOT_DIR/src-tauri/target/debug/worldcli}"

DARREN_ID="${DARREN_ID:-ddc3085e-0549-4e1f-a7b6-0894aa8180c6}"
JASPER_ID="${JASPER_ID:-fd4bd9b5-8768-41e6-a90f-bfb1179b1d59}"
LIMIT="${LIMIT:-80}"
CONFIRM_COST="${CONFIRM_COST:-5}"

PACK_MIN_SPEECH_FIRST="${PACK_MIN_SPEECH_FIRST:-0.8}"
PACK_MIN_SHIFT_RUN="${PACK_MIN_SHIFT_RUN:-0.8}"

if [[ ! -x "$WORLDCLI" ]]; then
  echo "worldcli not found/executable at: $WORLDCLI" >&2
  echo "Build first: (cd src-tauri && cargo build --bin worldcli)" >&2
  exit 1
fi

run() {
  echo
  echo ">>> $*"
  "$@"
}

run "$WORLDCLI" --scope full --json register-shift --character "$DARREN_ID" --limit "$LIMIT"
run "$WORLDCLI" --scope full --json register-shift --character "$JASPER_ID" --limit "$LIMIT"

run "$WORLDCLI" --json register-shift-pack "$DARREN_ID" \
  --confirm-cost "$CONFIRM_COST" \
  --gate-min-speech-first-rate "$PACK_MIN_SPEECH_FIRST" \
  --gate-min-shift-run-rate "$PACK_MIN_SHIFT_RUN"

run "$WORLDCLI" --json register-shift-pack "$JASPER_ID" \
  --confirm-cost "$CONFIRM_COST" \
  --gate-min-speech-first-rate "$PACK_MIN_SPEECH_FIRST" \
  --gate-min-shift-run-rate "$PACK_MIN_SHIFT_RUN"

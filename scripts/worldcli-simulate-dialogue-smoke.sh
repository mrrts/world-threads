#!/usr/bin/env bash
set -euo pipefail

# Minimal smoke runner for worldcli simulate-dialogue.
# - Uses provided character_id if passed.
# - Otherwise auto-selects first character from list-characters --json.
# - Auto-reruns with --confirm-cost when budget gate returns confirm_at_least.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_TAURI_DIR="$ROOT_DIR/src-tauri"

if [[ ! -d "$SRC_TAURI_DIR" ]]; then
  echo "src-tauri directory not found: $SRC_TAURI_DIR" >&2
  exit 1
fi

CHARACTER_ID=""
TURNS="${TURNS:-2}"
DRY_RUN="${DRY_RUN:-0}"

for arg in "$@"; do
  case "$arg" in
    --dry-run)
      DRY_RUN=1
      ;;
    *)
      if [[ -z "$CHARACTER_ID" ]]; then
        CHARACTER_ID="$arg"
      fi
      ;;
  esac
done

cd "$SRC_TAURI_DIR"

if [[ -z "$CHARACTER_ID" ]]; then
  echo "Selecting first character id from worldcli list-characters..."
  LIST_JSON="$(cargo run --bin worldcli -- list-characters --json)"
  CHARACTER_ID="$(
    python3 - <<'PY' "$LIST_JSON"
import json, sys
data = json.loads(sys.argv[1])
chars = data.get("characters") or []
if not chars:
    raise SystemExit("No characters found in list-characters output")
print(chars[0].get("id") or "")
PY
  )"
  if [[ -z "$CHARACTER_ID" ]]; then
    echo "Unable to resolve character id from list-characters output." >&2
    exit 1
  fi
fi

echo "Running worldcli smoke: character_id=$CHARACTER_ID turns=$TURNS"

BASE_CMD=(cargo run --bin worldcli -- simulate-dialogue "$CHARACTER_ID" --turns "$TURNS")

if [[ "$DRY_RUN" == "1" ]]; then
  echo "DRY RUN: ${BASE_CMD[*]}"
  exit 0
fi

set +e
OUT="$("${BASE_CMD[@]}" 2>&1)"
RC=$?
set -e

if [[ $RC -eq 0 ]]; then
  echo "$OUT"
  exit 0
fi

CONFIRM_AT_LEAST="$(
  python3 - <<'PY' "$OUT"
import re, sys
text = sys.argv[1]
m = re.search(r'confirm_at_least:\s*([0-9.]+)', text)
print(m.group(1) if m else "")
PY
)"

if [[ -z "$CONFIRM_AT_LEAST" ]]; then
  echo "$OUT" >&2
  exit $RC
fi

echo "Budget gate triggered; rerunning with --confirm-cost $CONFIRM_AT_LEAST"
cargo run --bin worldcli -- simulate-dialogue "$CHARACTER_ID" --turns "$TURNS" --confirm-cost "$CONFIRM_AT_LEAST"

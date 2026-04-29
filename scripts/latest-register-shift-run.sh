#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORTS_DIR="$ROOT_DIR/reports"
JSON_MODE=false
PRETTY_MODE=false
NAME_ONLY=false
LIST_FILES=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --json)
      JSON_MODE=true
      shift
      ;;
    --pretty)
      JSON_MODE=true
      PRETTY_MODE=true
      shift
      ;;
    --name-only)
      NAME_ONLY=true
      shift
      ;;
    --list-files)
      LIST_FILES=true
      shift
      ;;
    *)
      echo "Unknown arg: $1" >&2
      echo "Usage: $0 [--json|--pretty] [--name-only] [--list-files]" >&2
      exit 1
      ;;
  esac
done

latest="$(ls -1dt "$REPORTS_DIR"/register-shift-dashboard-* 2>/dev/null | head -n 1 || true)"
if [[ -z "$latest" ]]; then
  echo "No register-shift dashboard runs found under $REPORTS_DIR" >&2
  exit 1
fi

run_name="$(basename "$latest")"
if $JSON_MODE; then
  if $LIST_FILES; then
    files_json="$(python3 - "$latest" <<'PY'
import json, os, sys
path = sys.argv[1]
files = sorted([n for n in os.listdir(path) if os.path.isfile(os.path.join(path, n))])
print(json.dumps(files))
PY
)"
  else
    files_json="[]"
  fi
  if $PRETTY_MODE; then
    python3 - "$latest" "$run_name" "$files_json" <<'PY'
import json, sys
latest = sys.argv[1]
run = sys.argv[2]
files = json.loads(sys.argv[3])
print(json.dumps({"latest": latest, "run": run, "files": files}, indent=2))
PY
  elif $NAME_ONLY; then
    printf '{"run":"%s"}\n' "$run_name"
  else
    printf '{"latest":"%s","run":"%s","files":%s}\n' "$latest" "$run_name" "$files_json"
  fi
elif $NAME_ONLY; then
  echo "$run_name"
elif $LIST_FILES; then
  ls "$latest"
else
  echo "$latest"
fi

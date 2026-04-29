#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORLDCLI="${WORLDCLI:-$ROOT_DIR/src-tauri/target/debug/worldcli}"

DARREN_ID="${DARREN_ID:-ddc3085e-0549-4e1f-a7b6-0894aa8180c6}"
JASPER_ID="${JASPER_ID:-fd4bd9b5-8768-41e6-a90f-bfb1179b1d59}"
LIMIT="${LIMIT:-80}"
CONFIRM_COST="${CONFIRM_COST:-5}"

PRESET="${1:-medium}"
shift || true
COMMIT_ARTIFACTS=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --commit-artifacts)
      COMMIT_ARTIFACTS=true
      shift
      ;;
    *)
      echo "Unknown arg: $1" >&2
      echo "Usage: $0 [loose|medium|strict] [--commit-artifacts]" >&2
      exit 1
      ;;
  esac
done

case "$PRESET" in
  loose)
    PACK_MIN_SPEECH_FIRST_DEFAULT="0.6"
    PACK_MIN_SHIFT_RUN_DEFAULT="0.6"
    ;;
  medium)
    PACK_MIN_SPEECH_FIRST_DEFAULT="0.8"
    PACK_MIN_SHIFT_RUN_DEFAULT="0.8"
    ;;
  strict)
    PACK_MIN_SPEECH_FIRST_DEFAULT="1.0"
    PACK_MIN_SHIFT_RUN_DEFAULT="0.9"
    ;;
  *)
    echo "Unknown preset: $PRESET" >&2
    echo "Usage: $0 [loose|medium|strict] [--commit-artifacts]" >&2
    exit 1
    ;;
esac

PACK_MIN_SPEECH_FIRST="${PACK_MIN_SPEECH_FIRST:-$PACK_MIN_SPEECH_FIRST_DEFAULT}"
PACK_MIN_SHIFT_RUN="${PACK_MIN_SHIFT_RUN:-$PACK_MIN_SHIFT_RUN_DEFAULT}"

if [[ ! -x "$WORLDCLI" ]]; then
  echo "worldcli not found/executable at: $WORLDCLI" >&2
  echo "Build first: (cd src-tauri && cargo build --bin worldcli)" >&2
  exit 1
fi

STAMP="$(date +%Y-%m-%d-%H%M%S)"
OUT_DIR="$ROOT_DIR/reports/register-shift-dashboard-$STAMP"
mkdir -p "$OUT_DIR"

run_json_to_file() {
  local name="$1"
  shift
  local out_file="$OUT_DIR/$name.json"
  echo
  echo ">>> $*"
  "$@" > "$out_file"
  echo "saved: $out_file"
}

echo "preset: $PRESET"
echo "pack gates: speech_first>=$PACK_MIN_SPEECH_FIRST shift_run>=$PACK_MIN_SHIFT_RUN"
echo "artifact dir: $OUT_DIR"

run_json_to_file darren-register-shift \
  "$WORLDCLI" --scope full --json register-shift --character "$DARREN_ID" --limit "$LIMIT"
run_json_to_file jasper-register-shift \
  "$WORLDCLI" --scope full --json register-shift --character "$JASPER_ID" --limit "$LIMIT"

run_json_to_file darren-pack \
  "$WORLDCLI" --json register-shift-pack "$DARREN_ID" \
  --confirm-cost "$CONFIRM_COST" \
  --gate-min-speech-first-rate "$PACK_MIN_SPEECH_FIRST" \
  --gate-min-shift-run-rate "$PACK_MIN_SHIFT_RUN"

run_json_to_file jasper-pack \
  "$WORLDCLI" --json register-shift-pack "$JASPER_ID" \
  --confirm-cost "$CONFIRM_COST" \
  --gate-min-speech-first-rate "$PACK_MIN_SPEECH_FIRST" \
  --gate-min-shift-run-rate "$PACK_MIN_SHIFT_RUN"

scoreboard_line="$(PRESET="$PRESET" python3 - "$OUT_DIR" <<'PY'
import json,os,sys
root=sys.argv[1]
def load(name):
    with open(os.path.join(root,f"{name}.json")) as f:
        return json.load(f)
d=load("darren-pack")
j=load("jasper-pack")
dp="PASS" if d.get("gate",{}).get("passed") else "FAIL"
jp="PASS" if j.get("gate",{}).get("passed") else "FAIL"
ds=d.get("speech_first_rate",0.0); dr=d.get("shift_run_rate",0.0)
js=j.get("speech_first_rate",0.0); jr=j.get("shift_run_rate",0.0)
print(f"SCOREBOARD preset={os.environ.get('PRESET','?')} | Darren:{dp} (speech={ds:.2f} shift={dr:.2f}) | Jasper:{jp} (speech={js:.2f} shift={jr:.2f})")
PY
)"
echo "$scoreboard_line"

cat > "$OUT_DIR/README.txt" <<EOF
register-shift dashboard artifacts
timestamp: $STAMP
preset: $PRESET
limit: $LIMIT
confirm_cost: $CONFIRM_COST
pack_min_speech_first: $PACK_MIN_SPEECH_FIRST
pack_min_shift_run: $PACK_MIN_SHIFT_RUN

files:
- darren-register-shift.json
- jasper-register-shift.json
- darren-pack.json
- jasper-pack.json
EOF

echo
echo "done: artifacts written to $OUT_DIR"
if $COMMIT_ARTIFACTS; then
  git add "$OUT_DIR"
  git commit -m "Add register-shift dashboard artifacts $STAMP" >/dev/null
  echo "artifacts committed from $OUT_DIR"
fi

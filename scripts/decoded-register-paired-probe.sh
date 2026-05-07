#!/usr/bin/env bash
# scripts/decoded-register-paired-probe.sh
#
# Run paired probes against Mode 0 (pre-wiring) and Mode 1 (post-wiring)
# `worldcli` binaries to gather W1 cells for The Decoded Register
# Sapphire arc. See:
#   reports/2026-05-07-0933-the-decoded-register-pre-registration.md
#   reports/2026-05-07-0937-the-decoded-register-w3-formula-law-prediction.md
#
# Mode 0 is `worldcli` built at commit 4b06cff (PR #46 commit 13 — the
# last commit before the v3 decode wiring at 8e12f50). Mode 1 is built
# at HEAD of `sapphire-seek-2026-05-08`. Both binaries live under:
#   ~/.worldcli/decoded-register/worldcli-mode{0,1}
#
# Each invocation writes paired outputs to:
#   ~/.worldcli/decoded-register/runs/<timestamp>/<char_slug>_<probe_id>/{mode0,mode1}/N{1..N}.txt
#
# Usage:
#   ./scripts/decoded-register-paired-probe.sh <character> <probe-id> "<probe-prompt>" [N]
#
# Example:
#   ./scripts/decoded-register-paired-probe.sh "Aaron" P1 \
#     "What's been pulling at you today?" 5

set -euo pipefail

CHAR_INPUT="${1:?character (id or display name) required}"
PROBE_ID="${2:?probe id (e.g. P1, P2) required}"
PROBE="${3:?probe prompt string required}"
N="${4:-5}"

MODE0_BIN="$HOME/.worldcli/decoded-register/worldcli-mode0"
MODE1_BIN="$HOME/.worldcli/decoded-register/worldcli-mode1"
DB="${WORLDTHREADS_DB:-$HOME/Library/Application Support/com.worldthreads.app/worldthreads.db}"
OUT_ROOT="$HOME/.worldcli/decoded-register/runs"
TS=$(date +%Y%m%d-%H%M%S)

# `worldcli ask` resolves character_id only (not display name). Resolve
# here once via sqlite so callers can pass either form.
if [[ ! -f "$DB" ]]; then
  echo "ERROR: DB not found at $DB" >&2
  echo "Set WORLDTHREADS_DB env var to override." >&2
  exit 1
fi
CHAR_ID=$(sqlite3 "$DB" "SELECT character_id FROM characters
  WHERE character_id = '${CHAR_INPUT//\'/\'\'}'
     OR display_name = '${CHAR_INPUT//\'/\'\'}' COLLATE NOCASE
  ORDER BY CASE WHEN character_id = '${CHAR_INPUT//\'/\'\'}' THEN 0 ELSE 1 END
  LIMIT 1;")
if [[ -z "$CHAR_ID" ]]; then
  echo "ERROR: could not resolve character '$CHAR_INPUT' against $DB" >&2
  exit 1
fi
CHAR_NAME=$(sqlite3 "$DB" "SELECT display_name FROM characters WHERE character_id = '$CHAR_ID';")
CHAR_SLUG=$(echo "$CHAR_NAME" | tr '[:upper:] ' '[:lower:]_' | tr -cd 'a-z0-9_')
CELL_DIR="$OUT_ROOT/${TS}_${CHAR_SLUG}_${PROBE_ID}"

if [[ ! -x "$MODE0_BIN" ]]; then
  echo "ERROR: Mode 0 binary not found at $MODE0_BIN" >&2
  echo "Build via: git checkout 4b06cff && cd src-tauri && cargo build --bin worldcli && cp target/debug/worldcli $MODE0_BIN" >&2
  exit 1
fi
if [[ ! -x "$MODE1_BIN" ]]; then
  echo "ERROR: Mode 1 binary not found at $MODE1_BIN" >&2
  echo "Build via: git checkout sapphire-seek-2026-05-08 && cd src-tauri && cargo build --bin worldcli && cp target/debug/worldcli $MODE1_BIN" >&2
  exit 1
fi

mkdir -p "$CELL_DIR/mode0" "$CELL_DIR/mode1"

# Self-describing cell metadata so future readers (or LLM judges) can
# reconstruct what was tested without rerunning.
cat > "$CELL_DIR/cell.meta" <<META
character_input: $CHAR_INPUT
character_id: $CHAR_ID
character_name: $CHAR_NAME
character_slug: $CHAR_SLUG
probe_id: $PROBE_ID
probe_prompt: $PROBE
N_per_mode: $N
mode0_bin: $MODE0_BIN
mode0_commit: 4b06cff (PR #46 commit 13 — last pre-wiring)
mode1_bin: $MODE1_BIN
mode1_commit: HEAD of sapphire-seek-2026-05-08
db: $DB
generated_at: $(date -Iseconds)
arc: The Decoded Register (Sapphire candidacy)
parent_pre_registration: reports/2026-05-07-0933-the-decoded-register-pre-registration.md
META

# Write the exact probe prompt to a sibling so it can be re-fed verbatim
# without shell-escaping concerns.
printf '%s' "$PROBE" > "$CELL_DIR/probe.txt"

run_one() {
  local mode="$1"; local bin="$2"; local i="$3"
  local out="$CELL_DIR/$mode/N${i}.json"
  local err="$CELL_DIR/$mode/N${i}.err"
  echo ">> [$mode rep $i/$N] $CHAR_NAME @ $PROBE_ID" >&2
  if "$bin" --db "$DB" --scope full ask "$CHAR_ID" "$PROBE" --json > "$out" 2> "$err"; then
    if [[ -s "$err" ]]; then
      echo "   stderr (non-empty):" >&2
      sed 's/^/     /' "$err" >&2
    fi
  else
    echo "   FAILED — see $err" >&2
  fi
}

# Interleave modes per rep so any clock-time / DB-state effects average
# evenly across both arms rather than concentrating on whichever mode
# runs first.
for i in $(seq 1 "$N"); do
  run_one mode0 "$MODE0_BIN" "$i"
  run_one mode1 "$MODE1_BIN" "$i"
done

echo "" >&2
echo "Complete. Cell dir:" >&2
echo "  $CELL_DIR" >&2
echo "" >&2
echo "Inspect side-by-side:" >&2
echo "  for i in $(seq 1 $N); do echo '=== rep '\$i' ==='; diff $CELL_DIR/mode0/N\$i.json $CELL_DIR/mode1/N\$i.json; done" >&2
echo "" >&2
echo "Or dump the assistant text from each rep:" >&2
echo "  for m in mode0 mode1; do echo \"=== \$m ===\"; for i in $(seq 1 $N); do jq -r .reply_post_orchestrator $CELL_DIR/\$m/N\$i.json 2>/dev/null || cat $CELL_DIR/\$m/N\$i.json; echo '---'; done; done" >&2
echo ""
echo "$CELL_DIR"

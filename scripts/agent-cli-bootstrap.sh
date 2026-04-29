#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORLDCLI_BIN="${WORLDCLI_BIN:-$ROOT_DIR/src-tauri/target/debug/worldcli}"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'EOF'
Usage: agent-cli-bootstrap.sh
Runs the CLI discovery checklist and prints basic availability status.
EOF
  exit 0
fi

run_check() {
  local label="$1"
  shift
  printf "==> %s\n" "$label"
  if "$@" >/dev/null 2>&1; then
    echo "PASS"
  else
    echo "FAIL"
  fi
}

echo "Agent CLI bootstrap discovery checks"
echo "repo: $ROOT_DIR"

run_check "worldcli --help" worldcli --help
run_check "worldcli register-shift --help" worldcli register-shift --help
run_check "worldcli register-shift-pack --help" worldcli register-shift-pack --help
run_check "local worldcli --help" "$WORLDCLI_BIN" --help
run_check "local worldcli register-shift --help" "$WORLDCLI_BIN" register-shift --help
run_check "local worldcli register-shift-pack --help" "$WORLDCLI_BIN" register-shift-pack --help
run_check "register-shift-dashboard --help" "$ROOT_DIR/scripts/register-shift-dashboard.sh" --help
run_check "run-rebound-strict --help" "$ROOT_DIR/scripts/run-rebound-strict.sh" --help
run_check "prune-register-shift-artifacts --help" "$ROOT_DIR/scripts/prune-register-shift-artifacts.sh" --help
run_check "latest-register-shift-run --help" "$ROOT_DIR/scripts/latest-register-shift-run.sh" --help
run_check "show-latest-register-shift-run --help" "$ROOT_DIR/scripts/show-latest-register-shift-run.sh" --help
run_check "compare-register-shift-runs --help" "$ROOT_DIR/scripts/compare-register-shift-runs.py" --help
run_check "export-latest-register-shift-csv --help" "$ROOT_DIR/scripts/export-latest-register-shift-csv.sh" --help

echo
echo "Reference: $ROOT_DIR/docs/CLI_AGENT_DISCOVERY.md"

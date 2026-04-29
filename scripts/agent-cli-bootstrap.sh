#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORLDCLI_BIN="${WORLDCLI_BIN:-$ROOT_DIR/src-tauri/target/debug/worldcli}"
JSON_MODE=false
FAIL_ON_MISSING=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --json)
      JSON_MODE=true
      shift
      ;;
    --fail-on-missing)
      FAIL_ON_MISSING=true
      shift
      ;;
    --help|-h)
      break
      ;;
    *)
      echo "Unknown arg: $1" >&2
      echo "Usage: agent-cli-bootstrap.sh [--json] [--fail-on-missing]" >&2
      exit 1
      ;;
  esac
done

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'EOF'
Usage: agent-cli-bootstrap.sh [--json] [--fail-on-missing]
Runs the CLI discovery checklist and prints basic availability status.
--fail-on-missing exits non-zero if any check fails.
EOF
  exit 0
fi

run_check() {
  local label="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    if ! $JSON_MODE; then
      printf "==> %s\nPASS\n" "$label"
    fi
    CHECK_RESULTS+=("{\"label\":\"$label\",\"passed\":true}")
  else
    if ! $JSON_MODE; then
      printf "==> %s\nFAIL\n" "$label"
    fi
    CHECK_RESULTS+=("{\"label\":\"$label\",\"passed\":false}")
  fi
}

CHECK_RESULTS=()
if ! $JSON_MODE; then
  echo "Agent CLI bootstrap discovery checks"
  echo "repo: $ROOT_DIR"
fi

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

passed_count=0
for item in "${CHECK_RESULTS[@]}"; do
  if [[ "$item" == *"\"passed\":true"* ]]; then
    passed_count=$((passed_count + 1))
  fi
done
total_count="${#CHECK_RESULTS[@]}"
failed_count=$((total_count - passed_count))

if $JSON_MODE; then
  printf '{"repo":"%s","reference":"%s/docs/CLI_AGENT_DISCOVERY.md","checks":[%s]}\n' \
    "$ROOT_DIR" "$ROOT_DIR" "$(IFS=,; echo "${CHECK_RESULTS[*]}")"
else
  echo
  echo "Summary: passed=$passed_count failed=$failed_count total=$total_count"
  echo "Reference: $ROOT_DIR/docs/CLI_AGENT_DISCOVERY.md"
fi

if $FAIL_ON_MISSING && (( failed_count > 0 )); then
  exit 2
fi

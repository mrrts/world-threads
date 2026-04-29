#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'EOF'
Usage: run-rebound-strict.sh [--commit-artifacts]
Runs strict register-shift dashboard with rebound pack enabled.
Forwards additional args to register-shift-dashboard.sh.
EOF
  exit 0
fi

# Rebound-focused daily ritual:
# - strict pack gates
# - rebound variant pack enabled
# - modest character-level rebound floor
RUN_REBOUND_PACK=true \
SHIFT_MIN_REBOUND_RATE="${SHIFT_MIN_REBOUND_RATE:-0.25}" \
"$ROOT_DIR/scripts/register-shift-dashboard.sh" strict "$@"

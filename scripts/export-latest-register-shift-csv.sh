#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo "Usage: export-latest-register-shift-csv.sh"
  echo "Writes latest summary CSV to reports/<run-name>-summary.csv"
  exit 0
fi
LATEST="$("$ROOT_DIR/scripts/latest-register-shift-run.sh")"
RUN_NAME="$(basename "$LATEST")"
OUT_PATH="$ROOT_DIR/reports/${RUN_NAME}-summary.csv"

"$ROOT_DIR/scripts/show-latest-register-shift-run.sh" --quiet --format csv > "$OUT_PATH"
echo "$OUT_PATH"

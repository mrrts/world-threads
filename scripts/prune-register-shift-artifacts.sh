#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORTS_DIR="$ROOT_DIR/reports"
KEEP="${1:-5}"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo "Usage: prune-register-shift-artifacts.sh [keep_count]"
  exit 0
fi

if ! [[ "$KEEP" =~ ^[0-9]+$ ]]; then
  echo "Usage: $0 [keep_count]" >&2
  exit 1
fi

dirs=()
while IFS= read -r line; do
  dirs+=("$line")
done < <(ls -1dt "$REPORTS_DIR"/register-shift-dashboard-* 2>/dev/null || true)
count="${#dirs[@]}"

if (( count <= KEEP )); then
  echo "Nothing to prune: found $count run dirs, keep=$KEEP."
  exit 0
fi

echo "Pruning register-shift artifacts: found $count, keep newest $KEEP."
for ((i=KEEP; i<count; i++)); do
  d="${dirs[$i]}"
  echo "remove: $d"
  rm -rf "$d"
done
echo "done."

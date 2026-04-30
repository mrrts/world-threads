#!/usr/bin/env bash
# Re-inject reports/fragments/homepage-practice-proof.md into README + landing report.
set -euo pipefail
cd "$(dirname "$0")/.."
exec python3 scripts/homepage_practice_fragment.py sync

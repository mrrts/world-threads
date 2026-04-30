#!/usr/bin/env bash
# Fail if README / landing report drift from reports/fragments/homepage-practice-proof.md
set -euo pipefail
cd "$(dirname "$0")/.."
exec python3 scripts/homepage_practice_fragment.py check

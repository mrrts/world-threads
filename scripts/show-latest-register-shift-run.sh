#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LATEST="$("$ROOT_DIR/scripts/latest-register-shift-run.sh")"

echo "latest_run: $LATEST"

python3 - "$LATEST" <<'PY'
import json
import os
import sys

root = sys.argv[1]
pairs = [
    ("darren-register-shift.json", "Darren shift"),
    ("jasper-register-shift.json", "Jasper shift"),
    ("darren-pack.json", "Darren pack"),
    ("jasper-pack.json", "Jasper pack"),
    ("darren-pack-rebound.json", "Darren rebound pack"),
    ("jasper-pack-rebound.json", "Jasper rebound pack"),
]

for name, label in pairs:
    path = os.path.join(root, name)
    if not os.path.exists(path):
        continue
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if "totals" in data:
        t = data["totals"]
        print(
            f"{label}: shift_rate={t.get('shift_rate',0):.4f} "
            f"rebound_rate={t.get('rebound_rate',0):.4f} "
            f"avg_shifts={t.get('avg_shifts_per_message',0):.4f}"
        )
    else:
        gate = data.get("gate", {}).get("passed")
        print(
            f"{label}: speech_first_rate={data.get('speech_first_rate',0):.4f} "
            f"shift_run_rate={data.get('shift_run_rate',0):.4f} "
            f"gate_passed={gate}"
        )
PY

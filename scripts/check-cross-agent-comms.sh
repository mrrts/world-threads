#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMMS_FILE="$ROOT/CROSS_AGENT_COMMS.md"

if [[ ! -f "$COMMS_FILE" ]]; then
  echo "CROSS_AGENT_COMMS | missing file: CROSS_AGENT_COMMS.md"
  exit 0
fi

python3 - <<PY
import re
from pathlib import Path

text = Path("${COMMS_FILE}").read_text()
headers = list(re.finditer(r"^##\\s+(\\d{4}-\\d{2}-\\d{2}[^\\n]*?)\\|\\s*from:\\s*([^|]+)\\|\\s*to:\\s*([^|]+)\\|\\s*status:\\s*([^\\n]+)$", text, re.M))

open_items = []
for i, m in enumerate(headers):
    status = m.group(4).strip().lower()
    to_field = m.group(3).strip().lower()
    if status != "open":
        continue
    if "codex" not in to_field and "cursor" not in to_field:
        continue
    open_items.append({
        "stamp": m.group(1).strip(),
        "from": m.group(2).strip(),
        "to": m.group(3).strip(),
        "status": status,
    })

if not open_items:
    print("CROSS_AGENT_COMMS | open_for_codex=0")
else:
    parts = []
    for item in open_items[:3]:
        parts.append(f"{item['stamp']} from={item['from']} to={item['to']}")
    more = ""
    if len(open_items) > 3:
        more = f" (+{len(open_items)-3} more)"
    print("CROSS_AGENT_COMMS | open_for_codex={} | {}".format(len(open_items), " || ".join(parts) + more))
PY

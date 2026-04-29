#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMMS_FILE="$ROOT/CROSS_AGENT_COMMS.md"
JSON_MODE=0

if [[ "${1:-}" == "--json" ]]; then
  JSON_MODE=1
fi

if [[ ! -f "$COMMS_FILE" ]]; then
  if [[ "$JSON_MODE" == "1" ]]; then
    python3 - <<PY
import json
print(json.dumps({"ok": False, "error": "missing file: CROSS_AGENT_COMMS.md", "open_for_codex": 0, "entries": []}))
PY
  else
    echo "CROSS_AGENT_COMMS | missing file: CROSS_AGENT_COMMS.md"
  fi
  exit 0
fi

python3 - <<PY
import re
import json
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

json_mode = ${JSON_MODE}
if json_mode:
    print(json.dumps({
        "ok": True,
        "open_for_codex": len(open_items),
        "entries": open_items,
    }))
else:
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

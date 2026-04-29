#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMMS_FILE="$ROOT/CROSS_AGENT_COMMS.md"
JSON_MODE=0
MAX_ITEMS=3
TO_FILTER="codex"
OLDEST_FIRST=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --json)
      JSON_MODE=1
      shift
      ;;
    --max)
      MAX_ITEMS="${2:-3}"
      shift 2
      ;;
    --to)
      TO_FILTER="${2:-codex}"
      shift 2
      ;;
    --oldest-first)
      OLDEST_FIRST=1
      shift
      ;;
    *)
      echo "Usage: $(basename "$0") [--json] [--max N] [--to codex|cursor|all] [--oldest-first]" >&2
      exit 2
      ;;
  esac
done

if ! [[ "$MAX_ITEMS" =~ ^[0-9]+$ ]]; then
  echo "--max must be a non-negative integer" >&2
  exit 2
fi
case "$TO_FILTER" in
  codex|cursor|all) ;;
  *)
    echo "--to must be one of: codex, cursor, all" >&2
    exit 2
    ;;
esac

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
    to_filter = "${TO_FILTER}"
    wants = (
        ("codex" in to_field and to_filter in ("codex", "all")) or
        ("cursor" in to_field and to_filter in ("cursor", "all"))
    )
    if not wants:
        continue
    open_items.append({
        "stamp": m.group(1).strip(),
        "from": m.group(2).strip(),
        "to": m.group(3).strip(),
        "status": status,
    })

json_mode = ${JSON_MODE}
max_items = ${MAX_ITEMS}
oldest_first = ${OLDEST_FIRST}
if oldest_first:
    open_items = list(reversed(open_items))
if json_mode:
    entries = open_items[:max_items] if max_items > 0 else open_items
    print(json.dumps({
        "ok": True,
        "open_for_codex": len(open_items),
        "entries": entries,
        "truncated": max(0, len(open_items) - len(entries)),
        "filter": "${TO_FILTER}",
    }))
else:
    if not open_items:
        print("CROSS_AGENT_COMMS | open_for_codex=0")
    else:
        parts = []
        view = open_items[:max_items] if max_items > 0 else open_items
        for item in view:
            parts.append(f"{item['stamp']} from={item['from']} to={item['to']}")
        more = ""
        if len(open_items) > len(view):
            more = f" (+{len(open_items)-len(view)} more)"
        print("CROSS_AGENT_COMMS | open_for_codex={} | {}".format(len(open_items), " || ".join(parts) + more))
PY

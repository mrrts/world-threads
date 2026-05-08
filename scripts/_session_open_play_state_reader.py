#!/usr/bin/env python3
"""Helper for session_open_state_check.sh — reads .claude/play-state/current.json
and prints the HUD-ground-truth fields. Kept as a separate file to avoid
heredoc quoting issues in the parent bash script.
"""
import json
from pathlib import Path

p = Path(".claude/play-state/current.json")
if not p.exists():
    print("  WARNING: .claude/play-state/current.json not found")
else:
    d = json.loads(p.read_text())
    print("play-state ground truth:")
    print(f"  TURN:    {d.get('turn', 'unknown')}")
    bank = d.get('bank', 'unknown')
    if isinstance(bank, int):
        print(f"  BANK:    ${bank:,}")
    else:
        print(f"  BANK:    {bank}")
    print(f"  CROWNS:  {len(d.get('crowns', []))}")
    print(f"  JEWELS:  {len(d.get('jewels', []))}")
    print(f"  UPDATED: {d.get('updated_at', 'unknown')}")
    last_move = d.get('last_move', '')
    if isinstance(last_move, str) and last_move:
        print(f"  LAST:    {last_move[:120]}")

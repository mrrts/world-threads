#!/usr/bin/env python3
"""Extract confirm_at_least and cost_usd from worldcli smoke outputs.

Usage:
  python3 scripts/extract-worldcli-smoke-metrics.py /tmp/worldcli_science_*.txt > /tmp/worldcli_science_metrics.csv
"""
from __future__ import annotations

import csv
import pathlib
import re
import sys


CONFIRM_RES = [
    re.compile(r"confirm_at_least:\s*([0-9.]+)"),
    re.compile(r"--confirm-cost\s+([0-9.]+)"),
]
COST_RE = re.compile(r"cost_usd:\s*([0-9.]+)")
CHAR_RE = re.compile(r"character:\s*(.+?)\s*\(")
TURNS_RE = re.compile(r"turns:\s*([0-9]+)")


def extract_metrics(path: pathlib.Path) -> dict[str, str]:
    text = path.read_text(errors="replace")
    confirm = ""
    cost = ""
    character = ""
    turns = ""

    for cre in CONFIRM_RES:
        m = cre.search(text)
        if m:
            confirm = m.group(1)
            break
    m = COST_RE.search(text)
    if m:
        cost = m.group(1)
    m = CHAR_RE.search(text)
    if m:
        character = m.group(1).strip()
    m = TURNS_RE.search(text)
    if m:
        turns = m.group(1)

    return {
        "file": str(path),
        "character": character,
        "turns": turns,
        "confirm_at_least": confirm,
        "cost_usd": cost,
    }


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: extract-worldcli-smoke-metrics.py <output1.txt> [output2.txt ...]", file=sys.stderr)
        return 2

    paths = [pathlib.Path(p) for p in sys.argv[1:]]
    rows = [extract_metrics(p) for p in paths]

    writer = csv.DictWriter(
        sys.stdout,
        fieldnames=["file", "character", "turns", "confirm_at_least", "cost_usd"],
    )
    writer.writeheader()
    for row in rows:
        writer.writerow(row)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

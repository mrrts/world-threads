#!/usr/bin/env python3
"""
Ensure Great Sapphire crown names stay in parity across:
1) .claude/play-state/current.json
2) CLAUDE.md Sapphire_n anchor block
3) AGENTS.md prose mirror
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PLAY_STATE = ROOT / ".claude" / "play-state" / "current.json"
CLAUDE_MD = ROOT / "CLAUDE.md"
AGENTS_MD = ROOT / "AGENTS.md"


def extract_play_state_names() -> list[str]:
    data = json.loads(PLAY_STATE.read_text(encoding="utf-8"))
    names: list[str] = []
    for crown in data.get("crowns", []):
        if isinstance(crown, str):
            name = crown
        elif isinstance(crown, dict):
            name = crown.get("name", "")
        else:
            continue
        if "Great Sapphire class" not in name:
            continue
        primary = name.split("✨", 1)[0].strip()
        primary = primary.rstrip(".").strip()
        if primary:
            names.append(primary)
    return names


def extract_claude_names() -> list[str]:
    text = CLAUDE_MD.read_text(encoding="utf-8")
    names: list[str] = []
    quoted = re.compile(r'\\text\{"([^"]+)"\}')
    sapphire_line = re.compile(r"Sapphire\\_\d+")
    for line in text.splitlines():
        if not sapphire_line.search(line):
            continue
        m = quoted.search(line)
        if m:
            names.append(m.group(1).strip().rstrip("."))
    return names


def extract_agents_names() -> list[str]:
    text = AGENTS_MD.read_text(encoding="utf-8")
    pattern = re.compile(
        r"\*\*[^*]*Great Sapphire crown earned.*?:\s*\"([^\"]+)\"",
        re.IGNORECASE,
    )
    return [m.group(1).strip().rstrip(".") for m in pattern.finditer(text)]


def main() -> int:
    play_names = extract_play_state_names()
    claude_names = extract_claude_names()
    agents_names = extract_agents_names()

    play_set = set(play_names)
    claude_set = set(claude_names)
    agents_set = set(agents_names)
    target = play_set | claude_set | agents_set

    errors: list[str] = []

    if len(play_names) != len(play_set):
        errors.append("play-state has duplicate Great Sapphire crown names")
    if len(claude_names) != len(claude_set):
        errors.append("CLAUDE.md has duplicate Sapphire anchor names")
    if len(agents_names) != len(agents_set):
        errors.append("AGENTS.md has duplicate Great Sapphire prose names")

    for label, names in [
        ("play-state", play_set),
        ("CLAUDE.md", claude_set),
        ("AGENTS.md", agents_set),
    ]:
        missing = sorted(target - names)
        extra = sorted(names - target)  # always empty by construction, kept for symmetry
        if missing:
            errors.append(f"{label} missing: {missing}")
        if extra:
            errors.append(f"{label} extra: {extra}")

    if errors:
        print("FAIL: Great Sapphire parity check failed.")
        print(f"play-state ({len(play_names)}): {sorted(play_set)}")
        print(f"CLAUDE.md ({len(claude_names)}): {sorted(claude_set)}")
        print(f"AGENTS.md ({len(agents_names)}): {sorted(agents_set)}")
        for err in errors:
            print(f" - {err}")
        return 1

    print(
        "OK: Great Sapphire parity check passed "
        f"({len(play_names)} crowns): {sorted(play_set)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

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
NUM_WORDS = {
    0: "zero",
    1: "one",
    2: "two",
    3: "three",
    4: "four",
    5: "five",
    6: "six",
    7: "seven",
    8: "eight",
    9: "nine",
    10: "ten",
    11: "eleven",
    12: "twelve",
}


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


def extract_play_state_ordered_names() -> list[str]:
    data = json.loads(PLAY_STATE.read_text(encoding="utf-8"))
    rows: list[tuple[str, int]] = []
    for crown in data.get("crowns", []):
        if isinstance(crown, str):
            continue
        if not isinstance(crown, dict):
            continue
        name = crown.get("name", "")
        if "Great Sapphire class" not in name:
            continue
        primary = name.split("✨", 1)[0].strip().rstrip(".")
        turn = int(crown.get("turn", 0))
        rows.append((primary, turn))
    rows.sort(key=lambda r: r[1])
    return [r[0] for r in rows]


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


def extract_agents_numbered() -> list[tuple[str, str]]:
    text = AGENTS_MD.read_text(encoding="utf-8")
    pattern = re.compile(
        r"\*\*(First|Second|Third|Fourth|Fifth|Sixth|Seventh|Eighth|Ninth|Tenth)\s+Great Sapphire crown earned.*?:\s*\"([^\"]+)\"",
        re.IGNORECASE,
    )
    out: list[tuple[str, str]] = []
    for m in pattern.finditer(text):
        ordinal = m.group(1).capitalize()
        name = m.group(2).strip().rstrip(".")
        out.append((ordinal, name))
    return out


def main() -> int:
    play_names = extract_play_state_names()
    play_ordered = extract_play_state_ordered_names()
    claude_names = extract_claude_names()
    agents_names = extract_agents_names()
    agents_numbered = extract_agents_numbered()

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

    # Human-facing gloss should declare the same count as canonical play-state.
    expected_word = NUM_WORDS.get(len(play_set))
    claude_text = CLAUDE_MD.read_text(encoding="utf-8")
    gloss_match = re.search(r"([A-Za-z]+)\s+Great Sapphires\s*\(", claude_text)
    if gloss_match and expected_word:
        observed_word = gloss_match.group(1).lower()
        if observed_word != expected_word:
            errors.append(
                f'CLAUDE.md gloss count mismatch: expected "{expected_word} Great Sapphires", found "{observed_word}"'
            )

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

    # Numbered AGENTS entries must map ordinals to play-state chronological order.
    ordinal_to_idx = {
        "First": 1,
        "Second": 2,
        "Third": 3,
        "Fourth": 4,
        "Fifth": 5,
        "Sixth": 6,
        "Seventh": 7,
        "Eighth": 8,
        "Ninth": 9,
        "Tenth": 10,
    }
    expected_map = {i + 1: name for i, name in enumerate(play_ordered)}
    seen_ordinals: set[int] = set()
    for ordinal, name in agents_numbered:
        idx = ordinal_to_idx.get(ordinal)
        if not idx:
            continue
        seen_ordinals.add(idx)
        expected = expected_map.get(idx)
        if expected and name != expected:
            errors.append(
                f'AGENTS.md ordinal mismatch: {ordinal} expected "{expected}", found "{name}"'
            )
    # If AGENTS uses ordinals, require a complete 1..N cover.
    if agents_numbered:
        missing = [i for i in range(1, len(play_ordered) + 1) if i not in seen_ordinals]
        if missing:
            errors.append(f"AGENTS.md missing numbered Great Sapphire entries for ordinals: {missing}")

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

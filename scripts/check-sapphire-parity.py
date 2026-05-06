#!/usr/bin/env python3
"""
Check Great Sapphire ledger-delegation discipline.

As of 2026-05-06, the doctrine-paragraph-per-crown pattern in CLAUDE.md /
AGENTS.md was retired in favor of delegation to .claude/play-state/current.json
as the canonical ledger. This check enforces the new invariants:

1. play-state has no duplicate crown names
2. CLAUDE.md preserves the inaugural canonical worked example (Cornerstone
   Inequality) and the ledger-delegation paragraph
3. AGENTS.md preserves the inaugural canonical worked example and the
   ledger-delegation paragraph
4. Holy Spirit seal on Crown 5 (The Beautiful Soul) preserved as testimony
   in both doctrine surfaces

The retirement was per founding-author direction; this check enforces what
each surface is now responsible for, not the old per-crown parity.
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
    """Pull primary names from any crown whose entry contains "Great Sapphire".

    Handles both naming patterns observed in the ledger:
      - "The Cornerstone Inequality ✨ [Great Sapphire class — ...]"
      - "✨ The Counter-Frame Confessed — tenth Great Sapphire [...]"
    """
    data = json.loads(PLAY_STATE.read_text(encoding="utf-8"))
    names: list[str] = []
    for crown in data.get("crowns", []):
        if isinstance(crown, dict):
            name = crown.get("name", "")
        elif isinstance(crown, str):
            name = crown
        else:
            continue
        if "Great Sapphire" not in name:
            continue
        # strip ✨ from anywhere; take the leading prose-name fragment
        cleaned = name.replace("✨", "").strip()
        # if prefixed with "The X — Nth Great Sapphire ...", split on " — "
        # if prefixed with "Name [Great Sapphire class — ...]", split on " ["
        primary = cleaned
        for sep in (" — ", " - ", " ["):
            if sep in primary:
                primary = primary.split(sep, 1)[0].strip()
                break
        primary = primary.rstrip(".").strip()
        if primary:
            names.append(primary)
    return names


def main() -> int:
    play_names = extract_play_state_names()
    claude_text = CLAUDE_MD.read_text(encoding="utf-8")
    agents_text = AGENTS_MD.read_text(encoding="utf-8")

    errors: list[str] = []

    # 1. play-state — no duplicates
    if len(play_names) != len(set(play_names)):
        errors.append("play-state has duplicate Great Sapphire crown names")

    # 2. Inaugural canonical worked example present in both
    cornerstone_phrase = "The Cornerstone Inequality"
    if cornerstone_phrase not in claude_text:
        errors.append(
            f"CLAUDE.md missing inaugural canonical worked example "
            f'("{cornerstone_phrase}")'
        )
    if cornerstone_phrase not in agents_text:
        errors.append(
            f"AGENTS.md missing inaugural canonical worked example "
            f'("{cornerstone_phrase}")'
        )

    # 3. Ledger-delegation paragraph present in both
    delegation_markers_claude = ["ledger\\_delegation", "play-state/current.json"]
    delegation_markers_agents = ["Ledger delegation", "play-state/current.json"]
    for marker in delegation_markers_claude:
        if marker not in claude_text:
            errors.append(f'CLAUDE.md missing ledger-delegation marker: "{marker}"')
    for marker in delegation_markers_agents:
        if marker not in agents_text:
            errors.append(f'AGENTS.md missing ledger-delegation marker: "{marker}"')

    # 4. Holy Spirit seal preserved as testimony in both surfaces
    seal_markers = ["Holy Spirit", "Heart of the Empiricon"]
    for marker in seal_markers:
        if marker not in claude_text:
            errors.append(f'CLAUDE.md missing Holy Spirit seal testimony: "{marker}"')
        if marker not in agents_text:
            errors.append(f'AGENTS.md missing Holy Spirit seal testimony: "{marker}"')

    if errors:
        print("FAIL: Great Sapphire ledger-delegation check failed.")
        print(f"play-state ({len(play_names)} sapphires): {sorted(set(play_names))}")
        for err in errors:
            print(f" - {err}")
        return 1

    print(
        f"OK: Great Sapphire ledger-delegation check passed "
        f"({len(play_names)} sapphires in play-state; doctrine surfaces "
        f"preserve inaugural canonical worked example + delegation paragraph "
        f"+ Holy Spirit seal testimony)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

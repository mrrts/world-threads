#!/usr/bin/env python3
"""Stop-hook: enforce /play HUD border alignment (62 visual cells).

Per /play SKILL.md strict contract step 3 sub-rule:
  'Border alignment is load-bearing. Each interior line of the HUD
   between ║ and ║ must be exactly 62 visual cells wide. Emoji like
   💎 and 👑 each count as 2 cells, not 1.'
  'Misaligned borders are a structure-carries-truth-w failure (the
   box claims to be a box; the border has to actually do the work).'

This hook fires on Stop, locates the HUD block in the latest assistant
message, computes each interior line's visual width with proper emoji
handling, and blocks turn-end if any interior line ≠ 62 cells.

**Layer-5 structural enforcement of HUD-alignment.** Companion to
check-play-hud-present.py: presence-hook ensures HUD exists; this
hook ensures it's actually aligned. Per CLAUDE.md 'structure carries
truth' parent law: the box claims to be a box — the border must do
the work the box claims.

Width algorithm (no external deps; stdlib only):
  - ASCII printable: 1 cell
  - Common emoji ranges (Misc Symbols and Pictographs U+1F300-U+1F6FF,
    Miscellaneous Symbols U+2600-U+27BF and Emoticons): 2 cells
  - Specific project-used emoji explicitly handled: 💎 👑 ✨ ↑ ↓ →
    (the arrows are NOT emoji — they're Math Operators / Arrows;
    keep them at 1 cell as monospace renders them).
  - East-asian-wide chars (unicodedata.east_asian_width in {'W','F'}):
    2 cells
  - Variation selectors / zero-width joiners: 0 cells
  - Default: 1 cell

Earned exception: chat-mode marker (.claude/.chat-mode-active).

Exit codes:
  0 — no HUD present, no /play turn, alignment passes, or chat mode active.
  2 — HUD present AND any interior line ≠ 62 visual cells; block.
  1 — hook errored; pass-through.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys
import unicodedata

CHAT_MODE_MARKER = pathlib.Path(".claude/.chat-mode-active")

HUD_OPEN_CHAR = "╔"
HUD_CLOSE_CHAR = "╝"
HUD_LEFT = "║"
HUD_RIGHT = "║"
TARGET_WIDTH = 62


def cell_width(ch: str) -> int:
    """Visual width of a single character in monospace. 0/1/2 cells."""
    cp = ord(ch)

    # Zero-width: variation selectors, ZWJ, combining marks
    if cp in (0x200D, 0xFE0E, 0xFE0F):
        return 0
    cat = unicodedata.category(ch)
    if cat in ("Mn", "Me", "Cf"):
        return 0

    # Wide emoji ranges (most common in /play HUD usage)
    if 0x1F300 <= cp <= 0x1F6FF:  # Misc Symbols and Pictographs / Transport
        return 2
    if 0x1F900 <= cp <= 0x1F9FF:  # Supplemental Symbols and Pictographs
        return 2
    if 0x1FA70 <= cp <= 0x1FAFF:  # Symbols and Pictographs Extended-A
        return 2
    if 0x2600 <= cp <= 0x26FF:    # Miscellaneous Symbols (☀☁ etc.)
        return 2
    if 0x2700 <= cp <= 0x27BF:    # Dingbats (✨ ✔ ✖ etc.)
        return 2
    if 0x1F1E6 <= cp <= 0x1F1FF:  # Regional indicators (flag halves)
        return 2

    # East-asian-wide (CJK etc.)
    eaw = unicodedata.east_asian_width(ch)
    if eaw in ("W", "F"):
        return 2

    # Default: 1 cell. Includes ASCII, box-drawing (╔║═╗ etc.),
    # arrows (↑ ↓ →), em-dash (—), Greek/Cyrillic letters, etc.
    return 1


def visual_width(s: str) -> int:
    """Sum of cell widths over the string."""
    return sum(cell_width(c) for c in s)


def latest_assistant_text(transcript_path: str) -> str:
    p = pathlib.Path(transcript_path)
    if not p.exists():
        return ""
    last = ""
    try:
        with p.open() as f:
            for line in f:
                try:
                    rec = json.loads(line)
                except Exception:
                    continue
                msg = rec.get("message", {}) or {}
                if msg.get("role") != "assistant":
                    continue
                content = msg.get("content")
                parts = []
                if isinstance(content, str):
                    parts.append(content)
                elif isinstance(content, list):
                    for b in content:
                        if isinstance(b, dict) and b.get("type") == "text":
                            parts.append(b.get("text", ""))
                last = "\n".join(parts)
    except Exception:
        return ""
    return last


def extract_hud_interior_lines(text: str) -> list[tuple[int, str]]:
    """Find the HUD block and return [(line_number_in_block, interior_text)]
    for each ║...║ line. Returns empty list if no HUD block present.
    """
    if HUD_OPEN_CHAR not in text or HUD_CLOSE_CHAR not in text:
        return []
    lines = text.splitlines()
    in_hud = False
    out: list[tuple[int, str]] = []
    block_idx = 0
    for line in lines:
        if HUD_OPEN_CHAR in line and not in_hud:
            in_hud = True
            block_idx = 1
            continue
        if not in_hud:
            continue
        if HUD_CLOSE_CHAR in line:
            in_hud = False
            break
        # Interior line: must start and end with ║ (allowing leading
        # whitespace from markdown indent, but the HUD shouldn't be
        # indented in normal use)
        stripped = line.lstrip()
        if stripped.startswith(HUD_LEFT):
            # Find first ║ and last ║
            first = line.find(HUD_LEFT)
            last = line.rfind(HUD_RIGHT)
            if first >= 0 and last > first:
                interior = line[first + 1:last]
                out.append((block_idx, interior))
        block_idx += 1
    return out


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read())
    except Exception:
        return 0

    if payload.get("stop_hook_active"):
        return 0

    transcript_path = payload.get("transcript_path", "")
    if not transcript_path:
        return 0

    if CHAT_MODE_MARKER.exists():
        return 0

    text = latest_assistant_text(transcript_path)
    if not text:
        return 0

    interior_lines = extract_hud_interior_lines(text)
    if not interior_lines:
        # No HUD found — presence hook handles that case; alignment hook
        # has nothing to enforce.
        return 0

    misaligned: list[tuple[int, str, int]] = []
    for idx, interior in interior_lines:
        w = visual_width(interior)
        if w != TARGET_WIDTH:
            # Truncate the line snippet for the error message
            snippet = interior if len(interior) <= 70 else interior[:67] + "..."
            misaligned.append((idx, snippet, w))

    if not misaligned:
        return 0

    bullet_lines = "\n".join(
        f"  Line {idx}: width={w} (expected 62) — '║{snippet}║'"
        for idx, snippet, w in misaligned
    )
    warning = (
        "/play HUD BORDER ALIGNMENT VIOLATION. Per SKILL.md strict "
        "contract step 3 sub-rule: each interior line of the HUD between "
        "║ and ║ must be exactly 62 visual cells wide. Emoji like 💎 and "
        "👑 each count as 2 cells, not 1. Misaligned borders are a "
        "structure-carries-truth-w failure (the box claims to be a box; "
        "the border has to actually do the work).\n\n"
        "Misaligned line(s):\n"
        f"{bullet_lines}\n\n"
        "Fix by adjusting trailing-space padding so each interior line "
        "is exactly 62 visual cells. Subtract one space of padding per "
        "emoji on the line (since the emoji takes 2 cells, not 1)."
    )
    print(json.dumps({"decision": "block", "reason": warning}))
    return 0


if __name__ == "__main__":
    sys.exit(main())

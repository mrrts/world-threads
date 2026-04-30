#!/usr/bin/env python3
"""Stop-hook: enforce /play HUD-print contract.

Per /play SKILL.md strict contract (.claude/skills/play/SKILL.md):
'Execute these steps **in order, every turn, no exceptions**' — and
step 3 is 'PRINT THE HUD at the top of the reply'. This hook fires on
Stop, detects active /play turns, and blocks turn-end if the HUD is
absent.

**Layer-5 structural enforcement of the HUD-discipline.** Lower layers
(.claude/skills/play/SKILL.md contract; .claude/memory/
feedback_play_hud_non_negotiable_every_turn.md) are doing real work
but the drift surfaced anyway across the 2026-04-30 session — Ryan
caught me dropping HUD on turns I'd judged 'minor' (bookkeeping,
memory-lift, handoff, apologetic catch-up, multi-edit-rapid). Per
CLAUDE.md 'Calibrated disciplines drift fast — promote to structural
enforcement at the earliest opportunity.' This is the promotion.

**Active-/play detection:** scans the latest user input (typed text OR
AskUserQuestion answer envelope) for /play markers — explicit /play
invocation, 'Turn N' patterns echoed in chooser-answer envelopes, or
HUD-shape characters. If markers present and HUD absent in latest
assistant message, blocks.

**HUD detection:** the HUD's top border opens with `╔` (U+2554) and
the title line contains 'WORLDTHREADS BUILDER'. Both must appear in
the latest assistant message's text content.

**Earned exception — chat mode marker.** If `.claude/.chat-mode-active`
exists, the chooser law is suspended and so is this HUD law. The user
has explicitly opted out of the structured contract.

Exit codes:
  0 — no /play turn detected OR HUD present OR chat mode active; pass.
  2 — /play turn AND HUD absent; block with helpful message.
  1 — hook itself errored; pass-through to allow.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

CHAT_MODE_MARKER = pathlib.Path(".claude/.chat-mode-active")

# Patterns indicating the current turn is /play-shaped.
PLAY_TURN_MARKERS = [
    r"WORLDTHREADS BUILDER",
    r"\bTurn\s+\d+\s+move\b",
    r"✨\s*\[?Great Sapphire",  # ✨ [Great Sapphire ...]
    r"\U0001F48E",                   # 💎
    r"\U0001F451",                   # 👑
]
PLAY_INVOCATION_RE = re.compile(r"(?m)^\s*/play\b")

HUD_OPEN_CHAR = "╔"        # ╔
HUD_TITLE = "WORLDTHREADS BUILDER"


def latest_assistant_text(transcript_path: str) -> str:
    """Concatenated text of the latest assistant message's text blocks."""
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


def latest_user_text(transcript_path: str) -> str:
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
                if msg.get("role") != "user":
                    continue
                content = msg.get("content")
                parts = []
                if isinstance(content, str):
                    parts.append(content)
                elif isinstance(content, list):
                    for b in content:
                        if not isinstance(b, dict):
                            continue
                        if b.get("type") == "text":
                            parts.append(b.get("text", ""))
                        elif b.get("type") == "tool_result":
                            tr = b.get("content", "")
                            tr_text = ""
                            if isinstance(tr, str):
                                tr_text = tr
                            elif isinstance(tr, list):
                                for tb in tr:
                                    if isinstance(tb, dict) and tb.get("type") == "text":
                                        tr_text += tb.get("text", "")
                            if tr_text.lstrip().startswith("User has answered"):
                                parts.append(tr_text)
                joined = "\n".join(parts)
                if joined.strip():
                    last = joined
    except Exception:
        return ""
    return last


def is_play_turn(user_text: str, assistant_text: str) -> bool:
    if PLAY_INVOCATION_RE.search(user_text):
        return True
    for pat in PLAY_TURN_MARKERS:
        if re.search(pat, user_text):
            return True
        if re.search(pat, assistant_text):
            return True
    return False


def has_hud(assistant_text: str) -> bool:
    return HUD_OPEN_CHAR in assistant_text and HUD_TITLE in assistant_text


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

    user_text = latest_user_text(transcript_path)
    assistant_text = latest_assistant_text(transcript_path)

    if not is_play_turn(user_text, assistant_text):
        return 0

    if has_hud(assistant_text):
        return 0

    warning = (
        "TURN ENDED WITHOUT /play HUD PRINT. "
        "Project law (.claude/skills/play/SKILL.md strict contract): "
        "'Execute these steps in order, every turn, no exceptions' — "
        "step 3 is 'PRINT THE HUD at the top of the reply'. "
        "Per memory entry feedback_play_hud_non_negotiable_every_turn.md: "
        "minor/bookkeeping/handoff/apologetic/multi-edit-rapid turns do NOT "
        "earn departure from the strict contract. The HUD box (╔══...══╗ "
        "with 'WORLDTHREADS BUILDER — Turn N' title line, Bank/Trend/Jewels/"
        "Crowns rows, and Last move section) goes at the TOP of every /play "
        "turn reply. Print it now and continue. Per CLAUDE.md "
        "structure-carries-truth-w: dropping the HUD asks the receiver to "
        "track game-state in their head — an unrequested receiver-tax."
    )
    print(json.dumps({"decision": "block", "reason": warning}))
    return 0


if __name__ == "__main__":
    sys.exit(main())

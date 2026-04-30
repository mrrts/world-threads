#!/usr/bin/env python3
"""Stop-hook: enforce AskUserQuestion usage specifically on /play turns.

This is a /play-specific guardrail that remains active even if broader
conversation patterns vary. If a /play turn is detected and the latest
assistant message does not include an AskUserQuestion tool_use, block.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

CHAT_MODE_MARKER = pathlib.Path(".claude/.chat-mode-active")

PLAY_MARKERS = [
    r"(?m)^\s*/play\b",
    r"\bWORLDTHREADS BUILDER\b",
    r"\bTurn\s+\d+\b",
]


def latest_assistant_tool_use_ask(transcript_path: str) -> bool | None:
    p = pathlib.Path(transcript_path)
    if not p.exists():
        return None
    last_has = None
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
            has = False
            if isinstance(content, list):
                for b in content:
                    if isinstance(b, dict) and b.get("type") == "tool_use" and b.get("name") == "AskUserQuestion":
                        has = True
                        break
            last_has = has
    return last_has


def latest_text_for_role(transcript_path: str, role: str) -> str:
    p = pathlib.Path(transcript_path)
    if not p.exists():
        return ""
    last = ""
    with p.open() as f:
        for line in f:
            try:
                rec = json.loads(line)
            except Exception:
                continue
            msg = rec.get("message", {}) or {}
            if msg.get("role") != role:
                continue
            content = msg.get("content")
            parts = []
            if isinstance(content, str):
                parts.append(content)
            elif isinstance(content, list):
                for b in content:
                    if isinstance(b, dict) and b.get("type") == "text":
                        parts.append(b.get("text", ""))
            joined = "\n".join(parts).strip()
            if joined:
                last = joined
    return last


def is_play_turn(user_text: str, assistant_text: str) -> bool:
    blob = f"{user_text}\n{assistant_text}"
    return any(re.search(p, blob, re.IGNORECASE) for p in PLAY_MARKERS)


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read())
    except Exception:
        return 0

    if payload.get("stop_hook_active"):
        return 0

    if CHAT_MODE_MARKER.exists():
        return 0

    transcript_path = payload.get("transcript_path", "")
    if not transcript_path:
        return 0

    user_text = latest_text_for_role(transcript_path, "user")
    assistant_text = latest_text_for_role(transcript_path, "assistant")

    if not is_play_turn(user_text, assistant_text):
        return 0

    has_ask = latest_assistant_tool_use_ask(transcript_path)
    if has_ask is None or has_ask:
        return 0

    print(
        json.dumps(
            {
                "decision": "block",
                "reason": (
                    "/play turn ended without AskUserQuestion tool usage. "
                    "UI contract requires chooser tool presentation every /play turn."
                ),
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

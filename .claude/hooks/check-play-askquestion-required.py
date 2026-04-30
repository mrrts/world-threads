#!/usr/bin/env python3
"""Stop-hook: enforce /play chooser tool + immediate-work semantics.

This is a /play-specific guardrail that remains active even if broader
conversation patterns vary.

Rules enforced:
1) If a /play turn is detected, latest assistant message must include
   AskUserQuestion tool_use.
2) If the latest user input is a chooser answer envelope
   ("User has answered ..."), then at least one NON-AskUserQuestion
   assistant tool_use must occur after that user message before stop.
   This enforces: chooser selection must kick off real work, not
   confirmation-only pause turns.
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


def read_messages(transcript_path: str) -> list[dict]:
    p = pathlib.Path(transcript_path)
    if not p.exists():
        return []
    out: list[dict] = []
    with p.open() as f:
        for line in f:
            try:
                rec = json.loads(line)
            except Exception:
                continue
            msg = rec.get("message", {}) or {}
            if isinstance(msg, dict):
                out.append(msg)
    return out


def text_from_message(msg: dict) -> str:
    content = msg.get("content")
    parts: list[str] = []
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
                if isinstance(tr, str):
                    parts.append(tr)
    return "\n".join(parts).strip()


def latest_text_for_role(messages: list[dict], role: str) -> str:
    last = ""
    for msg in messages:
        if msg.get("role") != role:
            continue
        t = text_from_message(msg)
        if t:
            last = t
    return last


def latest_assistant_has_ask(messages: list[dict]) -> bool | None:
    last_has = None
    for msg in messages:
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


def latest_user_index(messages: list[dict]) -> int | None:
    idx = None
    for i, msg in enumerate(messages):
        if msg.get("role") == "user":
            idx = i
    return idx


def has_non_ask_tool_use_after(messages: list[dict], start_idx: int) -> bool:
    for msg in messages[start_idx + 1 :]:
        if msg.get("role") != "assistant":
            continue
        content = msg.get("content")
        if not isinstance(content, list):
            continue
        for b in content:
            if (
                isinstance(b, dict)
                and b.get("type") == "tool_use"
                and b.get("name") != "AskUserQuestion"
            ):
                return True
    return False


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

    messages = read_messages(transcript_path)
    user_text = latest_text_for_role(messages, "user")
    assistant_text = latest_text_for_role(messages, "assistant")

    if not is_play_turn(user_text, assistant_text):
        return 0

    has_ask = latest_assistant_has_ask(messages)
    if has_ask is None or has_ask:
        pass
    else:
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

    # Immediate-work contract: if user just answered a chooser, assistant must
    # execute at least one non-AskUserQuestion tool call before stopping.
    if "User has answered" in user_text:
        u_idx = latest_user_index(messages)
        if u_idx is not None and not has_non_ask_tool_use_after(messages, u_idx):
            print(
                json.dumps(
                    {
                        "decision": "block",
                        "reason": (
                            "/play chooser selection did not kick off a concrete work unit. "
                            "After a chooser answer, execute at least one non-AskUserQuestion "
                            "tool call before ending the turn."
                        ),
                    }
                )
            )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

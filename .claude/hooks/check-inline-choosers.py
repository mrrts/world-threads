#!/usr/bin/env python3
"""Stop-hook: enforce that EVERY turn ends with an AskUserQuestion chooser.

Project law (.claude/memory/feedback_choosers_via_askuserquestion.md):
every assistant turn must end with an AskUserQuestion invocation. The
default chooser, when no more specific question fits, is:
  - Continue — present more options for what to do next
  - Exit    — end here

The rationale: this turns conversation flow into an explicit, user-driven
state machine. The model never decides on its own to stop or to drift into
an inline question; control returns to the user via the chooser UI on
every turn.

This hook fires on Stop. It walks the transcript, locates the most-recent
assistant message, and inspects its content blocks for a tool_use of name
"AskUserQuestion". If absent, it blocks the turn-end with a system-reminder
telling the model to add the chooser before stopping.

Inline-chooser and trailing-question detection (the previous, narrower
rule) is subsumed by this stronger check: any text-only ending fails the
new rule regardless of its surface shape, so the auxiliary detectors are
removed. The hook stays fast and conservative.
"""
from __future__ import annotations

import json
import pathlib
import sys


def last_assistant_used_askuserquestion(transcript_path: str) -> bool | None:
    """Walk the JSONL transcript; return whether the latest assistant message
    contains an AskUserQuestion tool_use. Returns None if no assistant
    message exists yet (don't block — there's nothing to enforce against)."""
    p = pathlib.Path(transcript_path)
    if not p.exists():
        return None
    last_used: bool | None = None
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
                used = False
                if isinstance(content, list):
                    for b in content:
                        if not isinstance(b, dict):
                            continue
                        if b.get("type") == "tool_use" and b.get("name") == "AskUserQuestion":
                            used = True
                            break
                last_used = used
    except Exception:
        return None
    return last_used


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read())
    except Exception:
        return 0

    # Prevent infinite loops — if we already blocked once and the model
    # is now retrying, don't block again on the same retry.
    if payload.get("stop_hook_active"):
        return 0

    transcript_path = payload.get("transcript_path", "")
    if not transcript_path:
        return 0

    used = last_assistant_used_askuserquestion(transcript_path)
    if used is None or used:
        # No assistant turn yet, OR last turn already invoked AskUserQuestion → allow stop.
        return 0

    warning = (
        "TURN ENDED WITHOUT ASKUSERQUESTION. "
        "Project law (.claude/memory/feedback_choosers_via_askuserquestion.md): "
        "EVERY turn must end with an AskUserQuestion chooser. The default chooser, "
        "when no more specific question fits, is: "
        '"Continue — present more options for what to do next" / "Exit — end here". '
        "Do NOT just emit a closing statement. Invoke AskUserQuestion now — either "
        "with a context-fitting set of options OR with the default Continue/Exit "
        "fallback. The chooser IS the ending of every reply."
    )
    print(json.dumps({"decision": "block", "reason": warning}))
    return 0


if __name__ == "__main__":
    sys.exit(main())

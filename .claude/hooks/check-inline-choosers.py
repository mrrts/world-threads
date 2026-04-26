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

**Earned exception — chat mode.** When the user signals they want to chat
informally (phrases like "let's chat", "drop the choosers", "casual mode",
"let me think out loud"), the chooser requirement suspends for that turn.
A persistent marker file at `.claude/.chat-mode-active` keeps subsequent
turns suspended until the user explicitly re-enables ("back to choosers",
"task mode", "structured mode"). The signal-detection scans the most-
recent USER message in the transcript.

This hook fires on Stop. It walks the transcript, locates the most-recent
assistant message, and inspects its content blocks for a tool_use of name
"AskUserQuestion". If absent AND chat mode is not active, it blocks the
turn-end with a system-reminder telling the model to add the chooser
before stopping.

Inline-chooser and trailing-question detection (the previous, narrower
rule) is subsumed by this stronger check: any text-only ending fails the
new rule regardless of its surface shape, so the auxiliary detectors are
removed. The hook stays fast and conservative.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

CHAT_MODE_MARKER = pathlib.Path(".claude/.chat-mode-active")

# Regex patterns matched against the most-recent user message (case-insensitive).
# Persistent-activate phrases: set the marker and keep chat-mode sticky
# until the user explicitly deactivates.
CHAT_ACTIVATE_PATTERNS = [
    r"let'?s (just )?chat",
    r"\bchat mode\b",
    r"\bcasual mode\b",
    r"\binformal mode\b",
    r"drop the choosers?",
    r"\bno choosers?\b",
    r"skip the choosers?",
    r"let me think (out loud)?",
    r"just talk\b",
    r"just chatting\b",
    r"\bconversational mode\b",
]

# One-shot suspend phrases: skip the chooser requirement for this single
# turn but do NOT write the persistent marker. Used for derive-session-arc
# triggers — the derivation hook's flourish IS the turn's chooser
# equivalent. Without this, the chooser-law would force an AskUserQuestion
# at end of the same turn, whose answer envelope would bury the trigger
# before the derive-session-arc Stop hook could read it. Single-turn
# auto-suspension breaks the deadlock without making chat-mode sticky.
ONE_SHOT_SUSPEND_PATTERNS = [
    r"derive (the )?session",
    r"session derivation",
    r"derive the arc",
    r"\bconsecrate\b",  # matches any 'consecrate' word: 'consecrate this',
                        # 'consecrate again', 'consecrate it', bare 'consecrate'
    r"formula[\s\-]cite( this)?",
    r"verse[\s\-]cite( this)?",
    r"verse this",
    r"from[\s\-]the[\s\-]formula",
    r"show derivation",
]

# Deactivate phrases clear the marker and re-enforce the chooser law.
CHAT_DEACTIVATE_PATTERNS = [
    r"back to choosers?",
    r"choosers? (back )?on",
    r"\btask mode\b",
    r"\bstructured mode\b",
    r"back to work\b",
    r"end chat mode",
    r"chat mode off",
]


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


def last_user_message_text(transcript_path: str) -> str:
    """Return the text of the most-recent user INPUT.

    User input has two valid surfaces:
      1. Typed text in the input box (string content / text blocks).
      2. AskUserQuestion answers (incl. chooser-Other free-text notes)
         which appear as tool_result strings prefixed with
         'User has answered'.

    NOT user input (excluded): Bash / Edit / Read / Glob / Grep
    tool_results — those are system acks of operations Claude
    initiated, not user instructions. Including them would risk
    silently activating chat-mode from incidental Bash output.
    """
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
                extracted = ""
                if isinstance(content, str):
                    extracted = content
                elif isinstance(content, list):
                    parts = []
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
                            # Only count AskUserQuestion answer envelopes —
                            # those start with "User has answered". Skip
                            # Bash / Edit / Read / etc. acknowledgments to
                            # prevent false-positive chat-mode activation
                            # from incidental output.
                            if tr_text.lstrip().startswith("User has answered"):
                                parts.append(tr_text)
                    extracted = "\n".join(parts)
                if extracted.strip():
                    last = extracted
    except Exception:
        return ""
    return last


def update_chat_mode_marker(user_text: str) -> bool:
    """Inspect the user's most-recent input for chat-mode signals.
    Returns True if the chooser requirement should be suspended for
    this turn (marker exists OR activate signal fired OR one-shot
    suspend signal fired).

    Side effect: writes or removes the persistent marker file based on
    signals. ONE_SHOT_SUSPEND_PATTERNS do NOT write the marker — they
    only suspend the chooser for the current turn. Deactivate signals
    take precedence over activate signals.

    Strips code blocks and inline code from the user message before
    matching, so chat-signal phrases that appear inside pasted code
    or technical examples don't false-trigger.
    """
    if not user_text:
        return CHAT_MODE_MARKER.exists()
    # Strip fenced code + inline code so phrases inside don't false-trigger.
    cleaned = re.sub(r"```.*?```", "", user_text, flags=re.DOTALL)
    cleaned = re.sub(r"`[^`]+`", "", cleaned)
    text_lower = cleaned.lower()

    # One-shot suspend (e.g. derive-session-arc triggers): suspend for
    # this turn only, do not toggle the persistent marker. Check first
    # so it returns True even if chat-mode marker is already off.
    for pat in ONE_SHOT_SUSPEND_PATTERNS:
        if re.search(pat, text_lower):
            return True

    # Deactivate first (precedence)
    for pat in CHAT_DEACTIVATE_PATTERNS:
        if re.search(pat, text_lower):
            try:
                CHAT_MODE_MARKER.unlink(missing_ok=True)
            except Exception:
                pass
            return False

    # Activate
    for pat in CHAT_ACTIVATE_PATTERNS:
        if re.search(pat, text_lower):
            try:
                CHAT_MODE_MARKER.parent.mkdir(parents=True, exist_ok=True)
                CHAT_MODE_MARKER.touch()
            except Exception:
                pass
            return True

    # No signal — fall back to marker state
    return CHAT_MODE_MARKER.exists()


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

    # Chat-mode exception: check user signals + marker. If chat mode is active,
    # suspend the chooser requirement for this turn.
    user_text = last_user_message_text(transcript_path)
    chat_mode_active = update_chat_mode_marker(user_text)
    if chat_mode_active:
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

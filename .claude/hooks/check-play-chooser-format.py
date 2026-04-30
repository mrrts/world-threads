#!/usr/bin/env python3
"""PreToolUse hook on AskUserQuestion: enforce /play chooser format.

Per /play SKILL.md strict contract step 7 + CLAUDE.md "Chooser
cardinality is fixed at four options" runtime doctrine:

- Each /play chooser presents exactly 4 numbered options.
- Each option label contains a bounty pattern: `(+$X,XXX)` or
  `($X,XXX)` or `(+$X,XXX)` placeholder, etc.
- Earned exception for the 4th 'Provide your own next move' (or
  similar user-authored) slot — bounty is unknown until the user
  defines the move, so a placeholder like `(+$?,???)` or absence is
  acceptable for that one slot only.

**Layer-5 structural enforcement of the chooser-format contract.**
Audit's recommended #2 promotion (reports/2026-04-30-0720-play-
contract-enforcement-audit.md). Pairs with check-no-nanny-chooser.py
(both PreToolUse on AskUserQuestion; one enforces no-nanny phrases,
this one enforces format).

Active-/play detection: scan the AskUserQuestion question text for
/play markers (e.g. 'Turn N move' header pattern). If question header
isn't a /play chooser, hook passes through.

Earned exception: chat-mode marker (.claude/.chat-mode-active).

Exit codes:
  0 — not a /play chooser, format passes, or chat mode active.
  2 — /play chooser AND format violation; block with diagnostic.
  1 — hook errored; pass-through.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

CHAT_MODE_MARKER = pathlib.Path(".claude/.chat-mode-active")

# Question-header patterns that mark a /play chooser
PLAY_CHOOSER_HEADER_RE = re.compile(
    r"\bTurn\s+\d+\b|WORLDTHREADS BUILDER|/play\b",
    re.IGNORECASE,
)

# Bounty pattern: opens with paren, optional plus, $ sign, digits + commas, close paren.
# Matches: (+$1,200), ($500), (+$10,000), (+$X,XXX), (-$500)
# Also matches placeholder variants like (+$?,???)
BOUNTY_RE = re.compile(r"\([+\-]?\$[\d,X?]+\)")

# User-authored / 'provide your own' slot — exempt from bounty requirement.
USER_AUTHORED_RE = re.compile(
    r"(provide your own|user[- ]authored|author'?s slot|free[- ]form)",
    re.IGNORECASE,
)

REQUIRED_CARDINALITY = 4


def is_play_chooser(question: dict) -> bool:
    """Return True if this AskUserQuestion looks like a /play chooser."""
    q_text = question.get("question", "") or ""
    header = question.get("header", "") or ""
    blob = f"{q_text}\n{header}"
    return bool(PLAY_CHOOSER_HEADER_RE.search(blob))


def check_chooser(question: dict) -> list[str]:
    """Return list of violation messages, empty if all good."""
    violations: list[str] = []
    options = question.get("options", []) or []
    n = len(options)

    # Cardinality check
    if n != REQUIRED_CARDINALITY:
        violations.append(
            f"Chooser cardinality is fixed at {REQUIRED_CARDINALITY} options "
            f"(per CLAUDE.md runtime doctrine 2026-04-29). Got {n}. "
            "Manufacture adjacent productive branches if natural branching is "
            "sparse — execute / inspect / tighten / user-defined are reliable "
            "fallback shapes."
        )

    # Per-option bounty-pattern check (skip user-authored slot)
    for i, opt in enumerate(options):
        label = (opt.get("label", "") or "").strip()
        description = (opt.get("description", "") or "").strip()
        if not label:
            violations.append(f"Option {i+1}: empty label")
            continue
        # Skip the user-authored slot
        if USER_AUTHORED_RE.search(label) or USER_AUTHORED_RE.search(description):
            continue
        if not BOUNTY_RE.search(label):
            violations.append(
                f"Option {i+1} ('{label[:60]}'): missing bounty pattern in label. "
                "Per /play SKILL.md step 7: '[A] (+$X,XXX) — <option label>' — "
                "the bounty must appear in the AskUserQuestion option label so "
                "the user sees the cost-of-choice before picking."
            )

    return violations


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read())
    except Exception:
        return 0

    if CHAT_MODE_MARKER.exists():
        return 0

    tool_input = payload.get("tool_input", {}) or {}
    questions = tool_input.get("questions", []) or []
    if not questions:
        return 0

    all_violations: list[str] = []
    for qi, question in enumerate(questions):
        if not is_play_chooser(question):
            continue
        v = check_chooser(question)
        if v:
            all_violations.extend([f"Q{qi+1}: {msg}" for msg in v])

    if not all_violations:
        return 0

    bullet_lines = "\n".join(f"  - {msg}" for msg in all_violations)
    warning = (
        "/play CHOOSER FORMAT VIOLATION. Per /play SKILL.md strict "
        "contract step 7 + CLAUDE.md 'Chooser cardinality is fixed at "
        "four options' runtime doctrine:\n\n"
        f"{bullet_lines}\n\n"
        "Fix: ensure exactly 4 options; each non-user-authored option "
        "label includes the bounty pattern '(+$X,XXX)' so the user "
        "sees cost-of-choice before picking. The 4th 'Provide your own "
        "next move' slot is exempt from bounty requirement."
    )
    print(json.dumps({"decision": "block", "reason": warning}))
    return 0


if __name__ == "__main__":
    sys.exit(main())

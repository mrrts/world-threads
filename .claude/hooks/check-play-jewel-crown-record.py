#!/usr/bin/env python3
"""Stop-hook: enforce /play jewel/crown announce-vs-record consistency.

Per /play SKILL.md strict contract reminder:
  'Jewels and crowns get recorded in the ledger, not just announced.
   Saved to the state file. The trail is the proof.'

This hook fires on Stop, scans the latest assistant message for prose
announcements of newly-earned jewels (💎) or crowns (👑), and verifies
each announced item appears in `.claude/play-state/current.json`'s
jewels/crowns arrays. Blocks turn-end if announce-without-record.

**Layer-5 structural enforcement of the trail-is-proof contract.**
Audit's recommended #3 promotion (reports/2026-04-30-0720-play-
contract-enforcement-audit.md). Companion to check-play-hud-present.py
and check-play-hud-alignment.py.

Detection logic:
  - Strip the HUD block from the assistant text (HUD's "Last move"
    section legitimately renders prior-turn earnings).
  - In remaining prose, find earning-shape patterns:
    * "[name] jewel fires/earned/lands"
    * "earns the [name] jewel/crown"
    * "[name] crown fires/earned/lands"
    * Sentence-level mention with 💎/👑 emoji near "fires"/"earned"
  - For each announced name, verify it appears in play-state arrays.
  - Block on announce-without-record with diagnostic.

False-positive guards:
  - References to past earnings phrased as past-tense remembrance
    ("the Cornerstone Inequality WAS earned at Turn 24") are skipped
    via past-perfect markers.
  - References inside markdown code blocks / quote blocks are skipped.
  - The hook is conservative: ambiguous mentions pass; only
    unambiguous announce-shape triggers blocking.

Earned exception: chat-mode marker (.claude/.chat-mode-active).

Exit codes:
  0 — no announcements, all announcements recorded, or chat mode.
  2 — announce-without-record violation; block with diagnostic.
  1 — hook errored; pass-through.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

CHAT_MODE_MARKER = pathlib.Path(".claude/.chat-mode-active")
PLAY_STATE_PATH = pathlib.Path(".claude/play-state/current.json")

JEWEL_EMOJI = "💎"
CROWN_EMOJI = "👑"

# Earning-shape patterns. Each matches a clause that names a thing as
# newly-earned (not just referenced). Use named groups for the name.
EARNING_PATTERNS = [
    # "fires the X crown" / "fires X jewel"
    re.compile(r"\bfires?\s+(?:the\s+)?(?P<name>[^.\n,;]{4,80})\s+(?:crown|jewel)", re.IGNORECASE),
    # "earns the X jewel" / "earns X crown"
    re.compile(r"\bearns?\s+(?:the\s+)?(?P<name>[^.\n,;]{4,80})\s+(?:crown|jewel)", re.IGNORECASE),
    # "X jewel fires" / "X crown fires"
    re.compile(r"(?P<name>[A-Z][^.\n,;]{3,80})\s+(?:jewel|crown)\s+fires?\b", re.IGNORECASE),
]

# Past-perfect markers — skip these (referencing prior earning).
PAST_REFERENCE_MARKERS = [
    "was earned",
    "had earned",
    "previously earned",
    "already earned",
    "was fired",
    "already fired",
    "from turn",
]


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


def strip_hud_block(text: str) -> str:
    """Remove the HUD ╔...╝ block from text. The HUD's Last-move section
    legitimately mentions prior earnings without those being new
    announcements."""
    return re.sub(
        r"╔[═]+╗.*?╚[═]+╝",
        "",
        text,
        flags=re.DOTALL,
    )


def strip_code_blocks(text: str) -> str:
    """Remove fenced code blocks and inline code so JSON examples and
    code snippets don't false-trigger."""
    text = re.sub(r"```.*?```", "", text, flags=re.DOTALL)
    text = re.sub(r"`[^`\n]+`", "", text)
    return text


def has_past_reference(context: str) -> bool:
    cl = context.lower()
    return any(marker in cl for marker in PAST_REFERENCE_MARKERS)


def find_announced_earnings(text: str) -> list[tuple[str, str]]:
    """Return [(kind, name)] tuples for plausibly-newly-announced
    jewels/crowns. Kind is 'jewel' or 'crown' or 'unknown'."""
    findings: list[tuple[str, str]] = []
    cleaned = strip_hud_block(text)
    cleaned = strip_code_blocks(cleaned)

    for pat in EARNING_PATTERNS:
        for m in pat.finditer(cleaned):
            # Look at surrounding 150 chars for past-reference markers
            start = max(0, m.start() - 75)
            end = min(len(cleaned), m.end() + 75)
            context = cleaned[start:end]
            if has_past_reference(context):
                continue
            name = m.group("name").strip()
            # Trim leading articles
            name = re.sub(r"^(the|a|an)\s+", "", name, flags=re.IGNORECASE).strip()
            if len(name) < 4:
                continue
            # Determine kind from the matched group
            full = m.group(0).lower()
            kind = "crown" if "crown" in full else ("jewel" if "jewel" in full else "unknown")
            findings.append((kind, name))
    return findings


def play_state_arrays() -> tuple[list[str], list[str]]:
    """Return (jewel_names, crown_names) from play-state."""
    if not PLAY_STATE_PATH.exists():
        return [], []
    try:
        with PLAY_STATE_PATH.open() as f:
            data = json.load(f)
        jewels = [j.get("name", "") for j in data.get("jewels", [])]
        crowns = [c.get("name", "") for c in data.get("crowns", [])]
        return jewels, crowns
    except Exception:
        return [], []


def name_recorded(name: str, names_list: list[str]) -> bool:
    """Loose match: announced name appears as substring of any recorded
    name (or vice versa). Permissive to avoid false positives from
    minor wording differences."""
    nlow = name.lower()
    for nm in names_list:
        nm_low = nm.lower()
        if nlow in nm_low or nm_low in nlow:
            return True
        # Also try matching first 3 significant words
        n_words = [w for w in re.findall(r"\w+", nlow) if len(w) > 2][:3]
        rec_words = [w for w in re.findall(r"\w+", nm_low) if len(w) > 2][:3]
        if n_words and n_words == rec_words[:len(n_words)]:
            return True
    return False


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

    text = latest_assistant_text(transcript_path)
    if not text:
        return 0

    findings = find_announced_earnings(text)
    if not findings:
        return 0

    jewels, crowns = play_state_arrays()
    if not jewels and not crowns:
        # Play-state empty or unreadable — don't block on this hook;
        # other hooks handle initialization edge cases.
        return 0

    missing: list[tuple[str, str]] = []
    for kind, name in findings:
        # For 'unknown' kind, check both arrays
        if kind == "crown":
            if not name_recorded(name, crowns):
                missing.append((kind, name))
        elif kind == "jewel":
            if not name_recorded(name, jewels):
                missing.append((kind, name))
        else:
            if not name_recorded(name, jewels) and not name_recorded(name, crowns):
                missing.append((kind, name))

    if not missing:
        return 0

    bullet_lines = "\n".join(
        f"  - Announced {kind}: '{name}' — not found in play-state {kind}s array"
        for kind, name in missing
    )
    warning = (
        "/play JEWEL/CROWN ANNOUNCE-WITHOUT-RECORD VIOLATION. "
        "Per SKILL.md strict contract reminder: 'Jewels and crowns get "
        "recorded in the ledger, not just announced. Saved to the state "
        "file. The trail is the proof.'\n\n"
        "Announced earnings missing from play-state:\n"
        f"{bullet_lines}\n\n"
        "Fix: edit .claude/play-state/current.json to add the announced "
        "earning to the appropriate jewels[] or crowns[] array with "
        "{turn, name, earned_at} fields. The announcement and the record "
        "must move together — the trail is what makes the earning real."
    )
    print(json.dumps({"decision": "block", "reason": warning}))
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Sync or verify the canonical homepage practice + Backstage proof strip.

Canonical body: reports/fragments/homepage-practice-proof.md
Host: reports/2026-04-27-0030-public-release-landing.md

(README.md was originally a host but diverged 2026-04-30 toward a
tighter market-facing register that doesn't carry the full Backstage
exhibit; the landing report is now the single canonical host. The
marketing page docs/index.html links into the landing report for
"the full exchange.")

Usage:
  python3 scripts/homepage_practice_fragment.py sync
  python3 scripts/homepage_practice_fragment.py check
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FRAGMENT = ROOT / "reports/fragments/homepage-practice-proof.md"
MARK_BEGIN = "<!-- HOMEPAGE_PRACTICE_PROOF:BEGIN -->"
MARK_END = "<!-- HOMEPAGE_PRACTICE_PROOF:END -->"
HOSTS = [
    ROOT / "reports/2026-04-27-0030-public-release-landing.md",
]

_BLOCK = re.compile(
    re.escape(MARK_BEGIN) + r"[\s\S]*?" + re.escape(MARK_END),
    re.MULTILINE,
)


def _load_fragment_body() -> str:
    raw = FRAGMENT.read_text(encoding="utf-8")
    # Strip leading HTML block comment (whole <!-- ... --> if first thing)
    raw = raw.lstrip()
    if raw.startswith("<!--"):
        end = raw.find("-->")
        if end != -1:
            raw = raw[end + 3 :].lstrip()
    return raw.rstrip() + "\n"


def sync() -> None:
    body = _load_fragment_body()
    replacement = f"{MARK_BEGIN}\n{body}{MARK_END}\n"
    for path in HOSTS:
        text = path.read_text(encoding="utf-8")
        if MARK_BEGIN not in text or MARK_END not in text:
            raise SystemExit(f"{path}: missing sync markers")
        new_text, n = _BLOCK.subn(replacement.rstrip("\n") + "\n", text, count=1)
        if n != 1:
            raise SystemExit(f"{path}: expected exactly one marker block, got {n}")
        path.write_text(new_text, encoding="utf-8")
    print("synced:", ", ".join(str(p.relative_to(ROOT)) for p in HOSTS))


def check() -> None:
    expected = _load_fragment_body()
    for path in HOSTS:
        text = path.read_text(encoding="utf-8")
        m = _BLOCK.search(text)
        if not m:
            raise SystemExit(f"{path}: no marker block found")
        got = m.group(0)
        inner = got[len(MARK_BEGIN) : -len(MARK_END)].strip("\n") + "\n"
        if inner != expected:
            raise SystemExit(
                f"{path}: fragment body out of sync with {FRAGMENT.relative_to(ROOT)}\n"
                "Run: ./scripts/sync-homepage-practice-fragment.sh"
            )
    print("check ok:", FRAGMENT.relative_to(ROOT))


def main() -> None:
    if len(sys.argv) != 2 or sys.argv[1] not in ("sync", "check"):
        raise SystemExit("usage: homepage_practice_fragment.py sync|check")
    if sys.argv[1] == "sync":
        sync()
    else:
        check()


if __name__ == "__main__":
    main()

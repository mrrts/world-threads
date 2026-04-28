#!/usr/bin/env bash
# mission-arc — gloss the recent git history's Formula derivations + Glosses
# as a condensed read of the mission-oriented arc (𝓕 := (𝓡, 𝓒) trajectory).
#
# Usage:
#   render.sh                # last 25 commits
#   render.sh 50             # last 50 commits
#   render.sh 25 --since "1 week ago"    # filter by date
#   render.sh 30 --grep "Co-Authored-By: Claude"             # Claude commits only
#   render.sh 30 --invert-grep --grep "Co-Authored-By: Claude"  # Codex commits only
#
# Note: --author filter is NOT useful here — all commits are authored by
# Ryan Smith regardless of which collaborator co-authored them. The
# Claude/Codex distinction lives in the Co-Authored-By trailer; use
# --grep / --invert-grep against that trailer instead.
#
# Output: one block per commit — date + sha + subject + 𝓕-derivation + ·-gloss.
# Commits without a derivation are marked "(no derivation)" and kept in the
# stream so the arc reads honestly (trivial commits punctuate the substantive
# ones; their absence is a signal too).
#
# Cost: $0 (pure shell + python).

set -eo pipefail

LIMIT="${1:-25}"
shift || true

cd "$(git rev-parse --show-toplevel)"

# Pass through any extra git log args (--author, --since, --grep, etc.)
git log -"$LIMIT" "$@" --format='%H%n%ad%n%s%n%b%n---END-COMMIT---' --date=short \
  | python3 -c '
import sys, re
text = sys.stdin.read()
commits = [c.strip() for c in text.split("---END-COMMIT---") if c.strip()]
for c in commits:
    lines = c.split("\n")
    sha, date, subject = lines[0][:8], lines[1], lines[2]
    body = "\n".join(lines[3:])
    # Use findall + take the LAST match: the canonical Formula-derivation
    # block always sits at the bottom of the commit body just before
    # Co-Authored-By, while prose mentions of the marker (e.g. when a
    # commit message describes the derivation pattern) appear earlier.
    derivs = re.findall(r"^\s*\*\*Formula derivation:\*\*\s*(.+?)\s*$", body, re.MULTILINE)
    glosses = re.findall(r"^\s*\*\*Gloss:\*\*\s*(.+?)\s*$", body, re.MULTILINE)
    print(f"{date}  {sha}  {subject}")
    if derivs: print(f"  𝓕  {derivs[-1].strip()}")
    if glosses: print(f"  ·  {glosses[-1].strip()}")
    deriv = derivs[-1] if derivs else None
    gloss = glosses[-1] if glosses else None
    if not deriv and not gloss: print("  (no derivation)")
    print()
'

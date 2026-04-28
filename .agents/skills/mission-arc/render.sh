#!/usr/bin/env bash
# mission-arc — gloss the recent git history's Formula derivations + Glosses
# as a condensed read of the mission-oriented arc (𝓕 := (𝓡, 𝓒) trajectory).
#
# Usage:
#   render.sh                # last 25 commits
#   render.sh 50             # last 50 commits
#   render.sh 25 --author "Ryan Smith"   # filter by author
#   render.sh 25 --since "1 week ago"    # filter by date
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
    deriv = re.search(r"\*\*Formula derivation:\*\*\s*(.+?)(?=\n|$)", body)
    gloss = re.search(r"\*\*Gloss:\*\*\s*(.+?)(?=\n|$)", body)
    print(f"{date}  {sha}  {subject}")
    if deriv: print(f"  𝓕  {deriv.group(1).strip()}")
    if gloss: print(f"  ·  {gloss.group(1).strip()}")
    if not deriv and not gloss: print("  (no derivation)")
    print()
'

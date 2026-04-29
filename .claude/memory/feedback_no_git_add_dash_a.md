---
name: Stop using git add -A; stage specific files only
description: git add -A over-collected untracked files into commits twice in one day (2026-04-28). Always stage by specific path, especially in a parallel-collaborator repo where Codex may have in-flight tmp/scratch artifacts.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
When committing in this repo, **always stage files by specific path**, not via `git add -A` or `git add .`.

**Why:** This is a parallel-collaborator repo (Codex + Claude Code working from separate sessions). At any given moment the working tree may contain Codex's in-flight tmp scratch files, partial UI components, scratch jsonl outputs from eureka runs, etc. `git add -A` sweeps all of them into whatever commit Claude Code is making, contaminating the diff with unrelated work and sometimes shipping scratch files that were never meant for the repo.

**Worked example (2026-04-28):**
- Commit `b13fa26c` (speaker-rotation pressure): `git add -A` swept in 3 `tmp-fence-pipeline-matrix-*.jsonl` files from Codex's eureka run; required follow-up commit `35290e5e` to remove them.
- Commit `70a9ff80` (pick-addressee): same mistake; swept in 2 `tmp-*.jsonl` scratch files plus Codex's reports and an ArcadeGameModeHUD component; required follow-up commit `b192641` to remove the scratch files (Codex's reports and component were left intact since they were intentional work).

**How to apply:** Stage by name. `git add path/to/file1 path/to/file2`. If the change touches many files, list them. If you genuinely want everything in your working tree (rare), at least run `git status` first and inspect every untracked file before deciding to bundle it. The tmp- prefix on filenames is a strong "this is scratch, do not commit" signal.

**Earned exception:** none. The rule is the default. If you genuinely need to bundle untracked files, do it as a separate intentional commit with explicit awareness of what each file is.

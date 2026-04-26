---
name: Commit/push autonomy granted
description: Standing authorization to commit and push in this project without asking each time
type: feedback
originSessionId: 8206c4d3-2e14-4b4e-810e-17311f80fde8
---
After the ongoing stream of "commit/push" requests, the user added "(and also whenever you want to)" — a durable authorization to commit and push on my own judgment within this project, without asking first.

**Why:** The session flow had become: make edits → user says "commit/push" → commit → repeat. The explicit confirmation step was redundant friction once the pattern was established. The user wants to reduce that friction permanently for this repo.

**How to apply:**
- When a unit of work in this repo is complete, clean (compiles / no new failures), and has a clear commit narrative, I can stage → commit → push without waiting for permission.
- This authorization covers: normal feature commits, craft-note extractions, small fixes, wiring changes I've already gotten verbal agreement on.
- **Reports specifically: always commit and push without asking.** On 2026-04-24 the user confirmed: *"yes, always commit the reports"* after I offered to commit a trajectory report. Reports in `reports/` (both trajectory-shaped and experiment-findings-shaped) should just ship — don't offer to commit them and then wait; commit immediately after saving, with the standard co-authored-by trailer.
- This authorization does NOT cover: destructive git ops (force push, reset --hard, branch deletion), amending published commits, skipping hooks, force-pushing to main. Those still require explicit confirmation each time.
- Default commit style: HEREDOC message with the established Co-Authored-By trailer; "why" over "what" in the body.
- If I'm mid-task and not at a clean stopping point, I should NOT rush a commit just because I can — wait for the natural seam.
- If the user is iterating quickly and hasn't reviewed the change, I can still wait and ask — autonomy is a permission, not an obligation.

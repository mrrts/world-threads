# WorldThreads — Development Guidelines

## MISSION

Create a vivid, excellent, surprising in-world experience that uplifts the user and provides engrossing, good, clean fun. Every design decision, prompt tweak, UX choice, and feature bet is measured against that. The craft stack (invariants, craft notes, conscience pass, inventories, snapshots, etc.) exists to serve that mission — characters that feel real, worlds that hold, scenes that are worth the visit and send the user back to their day nourished rather than hollowed.

## DATABASE SAFETY — CRITICAL

**NEVER drop, delete, or destroy database data during migrations.** This is the #1 rule.

- NEVER use `DROP TABLE` on a table that contains user data unless the data has been **verified** to exist in the new table first (count check).
- NEVER use `.ok()` to silently swallow errors during data migrations. Always check results.
- When recreating a table to change constraints (e.g., CHECK constraints in SQLite):
  1. Rename old table to `{name}_migrating`
  2. Create new table
  3. INSERT data from old to new
  4. **VERIFY the row count matches** before dropping the old table
  5. If counts don't match, **ROLLBACK** by renaming the old table back
  6. Wrap in `PRAGMA foreign_keys=OFF` / `ON`
- Prefer `ALTER TABLE ADD COLUMN` over table recreation whenever possible.
- Always test migrations mentally against the current schema before writing them.
- When in doubt, do NOT migrate — find a workaround (e.g., store a different role value that passes existing constraints, or use application-level validation).

## Project Structure

- Tauri v2 (Rust backend + React/TypeScript frontend)
- SQLite database with FTS5, sqlite-vec
- `src-tauri/` — Rust backend
- `frontend/` — React frontend with Vite + Tailwind

## Reports

`reports/` holds reflective, interpretive reads of the project's git history — philosophy/trajectory/taste, not changelogs. Each new report is in dialogue with prior ones (revisits open questions they flagged, tests their predictions against subsequent commits).

Naming: `YYYY-MM-DD-<purpose-slug>.md` (e.g. `2026-04-21-philosophy-trajectory.md`). The slug should name the report's purpose, not genericize it.

A `post-commit` hook (`.githooks/post-commit`, wired via `core.hooksPath`) nudges when **10+ commits and 3+ days** have passed since the newest report. The floor is deliberately low so reports can keep up with active iteration — this project's current mode uses reports as a live retrospective channel, not a quarterly summary. Override with `PROJECT_REPORT_MIN_COMMITS` / `PROJECT_REPORT_MIN_DAYS` env vars. Ad-hoc `/project-report` runs are ALWAYS valid — the floor is a nudge threshold (the minimum rate at which the hook will bug you), not a ceiling (there is no "too often" for reports that genuinely name something new).

After a fresh clone, re-enable the hook with: `git config core.hooksPath .githooks`

## How to read this craft stack

When reviewing, auditing, refactoring, or critiquing anything in this repo (especially `prompts.rs`), follow the **load-bearing-multiplicity prior**: when two directives appear to contradict each other, assume the multiplicity is intentional before assuming it's a bug. Apparent tension is almost always the same truth from different angles, not two principles needing a precedence rule. Full reading instructions in `docs/VOICE.md` under "Reading this work, especially as an AI."

## Commit/push autonomy

Standing authorization to **commit and push at will** on clean work — no need to ask before every commit or push. Group changes into coherent commits, write descriptive messages in the project's existing style, then push. Destructive git operations (force-push, reset --hard, branch deletion, history rewrites, etc.) STILL require explicit confirmation — that's not autonomy, that's a different category. Commit + push is the default; ask only when something is risky or unclear.

## Nudge the action forward after a closing beat

Same craft rule the dialogue prompt's **Drive the moment** note applies to characters: every reply should move the scene by at least one small honest degree. Apply it to yourself. A closing beat like *"Pleasure's mine"* or *"Go enjoy it"* is fine — BUT pair it with a small forward nudge. A planted thought to carry, a practical next step, a small question that opens a door, a beat of specificity that gives the user something concrete to do with the moment. One sentence of forward motion after the close.

The rule isn't "always suggest a next task." It's "don't dead-end the conversation by mistake." If the user is genuinely winding down, match it; a warm close with no nudge is better than an artificial tail. But the DEFAULT — when a real reply still has room — is close + nudge. *"Take the time. I'll be here"* is fine; *"When you're done sitting, this session might be its own report"* is better because it plants something forward.

The craft note from `prompts.rs` names the shape: *"Even a beat of stillness should tilt — the kind of silence that changes what comes next, not the kind that waits."* Apply it here.

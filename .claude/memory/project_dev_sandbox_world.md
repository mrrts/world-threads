---
name: Crystal Waters is the dev sandbox; Hal is not a sandbox character
description: Crystal Waters (world_id b8368a15-710f-4ccd-b5fe-47981e04c750) is the only world Claude Code should browse via worldcli by default. Hal Stroud is NOT a sandbox character regardless of what the DB schema's world_id field says — Ryan corrected this explicitly.
type: project
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
The user populated `~/.worldcli/config.json` with Crystal Waters (`b8368a15-710f-4ccd-b5fe-47981e04c750`) as the dev-sandbox world. Cast accessible by default: John, Steven, Darren, Aaron, Pastor Rick.

**Important boundary:** the DB has Hal Stroud listed in `17a4d306-...` (which the worlds table labels "Hollerwood Ridge"), but Ryan corrected this explicitly: *"Hal doesn't live there."* The DB's world membership is NOT the canonical truth about where characters belong in his mental map. Some characters in the DB are personal/non-sandbox even if their data sits in a world the schema names something else.

**Why:** April 2026 setup. Ryan: "make the crystal waters world yours for the browsing." Then, when Claude Code observed that Hal was in a different world per the DB and asked whether to add that world to scope: *"Hal doesn't live there."* Translation: the boundary isn't just a scope flag — Hal is intentionally not a sandbox character, period.

**How to apply:**
- Default `worldcli` browsing operates within Crystal Waters. Reach for those five characters first when you need to verify prompt theory, mine craft material, or A/B test phrasings.
- **Do NOT use `--scope full` to reach Hal Stroud or other characters that may be personal/non-sandbox.** Ask Ryan first if Hal-specific empirical verification is genuinely needed. The Hal-derived craft notes already in `prompts.rs` (wit trilogy, hands-coolant, noticing-as-mirror, unguarded-entry, plain-after-crooked) don't need re-verification; future work that would touch Hal waits for explicit invitation.
- More broadly: `--scope full` is fine for read-only inspection of *worlds*, but treat individual characters as scoped-by-intent, not scoped-by-DB-field. When in doubt, ask.
- If Ryan ever explicitly invites you to query a non-sandbox character ("go ahead and ask Hal X"), that invitation is per-task; don't generalize it to standing access.

**Worth re-checking periodically:** if Ryan extends scope (adds another world to the config, adds a character to the sandbox), they'll tell you. Don't infer scope changes from the DB.

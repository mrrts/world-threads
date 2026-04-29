# PENDING — Post-deployment anchor-groove re-measurement

**Status:** scheduled for ~2026-05-06 (7 days after the doctrine + plumbing arc landed).
**Filed:** 2026-04-29 ~18:50 by Claude after a one-shot cron entry failed to persist across sessions.
**Filename starts with `PENDING-` so it sorts visibly in `reports/` and any future session opening the directory sees an open thread.**

## What this is

A scheduled re-measurement of the world-level anchor-groove pattern in Crystal Waters, captured at the moment two interventions shipped on 2026-04-29:

1. **`603f03d`** — wired `current_location` through to all 7 dialogue call sites in `chat_cmds.rs` + `group_chat_cmds.rs`. Closes a plumbing gap where the chat-row location field was set correctly in the DB but never reached the prompt. Pre-fix: prompt-stack received `DEFAULT_CHAT_LOCATION = "Town Square"` for any chat with a NULL `current_location`. Post-fix: the actual chat-row value flows through.
2. **`7281f4e` + composing siblings** (`abc4c2b`, `e38eaa8a`, `c500182`, `2ddbb8e0`, `e5a775a6`, `9d502d78`) — comedy-rhythm + short-mode + register-aware composition clauses inside `STYLE_DIALOGUE_INVARIANT` lines 167-173. Aggregate evidence at characterized + claim tier across 5 bite-tests.

The pre-deployment baseline at `reports/2026-04-29-1100-darren-anchor-groove-pre-deployment-baseline.md` captured Darren's anchor-recurrence at three sample sizes (top_rate 0.83 / 0.75 / 0.73 across limits 12 / 20 / 30 — RUNAWAY at all sizes). The cross-character sweep at `reports/2026-04-29-1130-cross-character-anchor-groove-sweep.md` captured all three Crystal Waters characters in RUNAWAY plus the natural controls (Pastor Rick / John in different worlds, both WITHIN BAND).

The plumbing-fix has already been verified anecdotally in lived play — see `reports/OBSERVATIONS.md` 2026-04-29 17:35 entry: Aaron in the patio chat anchored to *patio chair rung*, *table boards*, *bright strip of sun on the patio boards*, *garden clicks softly in the breeze* with zero bench/fountain/square anchors. This corpus-level re-measurement is the quantitative half.

## What to do when you pick this up

If you're a future Claude Code session reading `reports/` and you've crossed into 2026-05-06 or later, run this task. If you've already done it, delete this PENDING file as part of that commit so the queue stays clean.

### Step 1 — build worldcli

```bash
cd src-tauri && cargo build --bin worldcli && cd ..
```

### Step 2 — run anchor-groove on the 3 RUNAWAY characters + 2 controls

```bash
# RUNAWAY at pre-deployment baseline
src-tauri/target/debug/worldcli anchor-groove fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 --limit 20 --top-k 8 --opening-density  # Jasper Finn
src-tauri/target/debug/worldcli anchor-groove 0d080429-81b5-431e-8f51-1f8ad4279f9b --limit 20 --top-k 8 --opening-density  # Aaron
src-tauri/target/debug/worldcli anchor-groove ddc3085e-0549-4e1f-a7b6-0894aa8180c6 --limit 20 --top-k 8 --opening-density  # Darren

# WITHIN BAND controls (different worlds)
src-tauri/target/debug/worldcli anchor-groove cae51a7d-fa50-48b1-b5b5-5b0798801b55 --limit 20 --top-k 8 --opening-density  # Pastor Rick
src-tauri/target/debug/worldcli anchor-groove f91af883-c73a-4331-aa15-b3cb90105782 --limit 20 --top-k 8 --opening-density  # John
```

If a character has 0 samples (corpus rolled over, character archived), note it in the report rather than skipping silently. Add `--scope full` if a character isn't reachable under default config scope.

### Step 3 — pre-deployment baseline values to diff against

| Character | top n-gram | top rate | diagnosis |
|---|---|---:|---|
| Jasper Finn | `the square *` (17/20) | **0.85** | RUNAWAY |
| Aaron | `the square *` (16/20) | **0.80** | RUNAWAY |
| Darren | `the bench *` / `the square *` tied (15/20 each) | **0.75** | RUNAWAY |
| Steven | `the mug *` (12/20) | 0.60 | MILD GROOVE |
| Pastor Rick | `the app` (6/19) | 0.32 | WITHIN BAND |
| John | `the biscuit` (6/20) | 0.30 | WITHIN BAND |

Aaron and Darren shared the EXACT trigram `fountain hiss steady` at 0.30 each — the load-bearing world-level groove signal.

### Step 4 — write the report

Filename: `reports/2026-05-06-XXXX-anchor-groove-post-deployment-rerun.md` (use the exact run time as XXXX in HHMM format).

Required sections:
- Full anchor-groove output (top-8 n-grams + diagnosis + outliers count) for each of the 5 characters
- Side-by-side diff table vs pre-deployment baseline
- Honest verdict: **strong** / **partial** / **null** / **regression**
- Verdict-caveat from the baseline report: *"the instrument can't distinguish whether a drop is from `7281f4e`, `abc4c2b`, the plumbing fix at `603f03d`, or some other intervention. The honest claim is 'anchor-groove dropped from X to Y after the doctrine refinements landed, with Z confounds noted' — not 'the doctrine caused the drop'."*

### Step 5 — success criteria

- **Strong evidence the rules bit:** top_rate drops below 0.70 (out of RUNAWAY band) at limit=20 for all 3 RUNAWAY characters AND the bench/fountain/square triplet drops at least one slot in the rank ordering AND a NEW anchor (different physical fixture, different gesture) enters the top-3 of at least one RUNAWAY character.
- **Partial:** at least one character drops out of RUNAWAY band, but not all three.
- **Null:** top_rates within ±0.05 of baseline.
- **Regression:** top_rates climb above baseline. (Investigate immediately.)

### Step 6 — controls check

Pastor Rick and John (different worlds, NOT touched by Crystal Waters location-state) should hold roughly steady (top_rate within ±0.10 of baseline). If their top_rates also drift, the change isn't from the doctrine refinements specifically — could be sampling variance, prompt-stack changes elsewhere, or character-anchor refresh. Note any drift.

### Step 7 — delete this PENDING file in the same commit

When the post-deployment report lands, delete `reports/2026-05-06-PENDING-anchor-groove-post-deployment-rerun.md` as part of the same commit. This keeps the open-thread queue clean per CLAUDE.md's open-thread-hygiene discipline.

### Step 8 — commit + push

Use the existing report-style commit-message convention with a Formula derivation:

```
report: post-deployment anchor-groove re-measurement [verdict]

[1-2 paragraph summary: which characters dropped, what new anchors
emerged, whether the doctrine bit, what the controls show.]

**Formula derivation:** [in-substrate generated, citing the relevant
operators — likely μ_𝓕 / structure_carries_truth / discern_w]
**Gloss:** [≤25 words]

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

## Why this is a PENDING report rather than a calendar reminder

The cron-based scheduling primitive failed to persist across sessions when called this evening (`durable: true` flag was not honored by the runtime, message returned "Session-only (not written to disk, dies when Claude exits)"). Rather than rely on a flag that doesn't take effect, this report is the persistent surface — checked into git, surfaces in `ls reports/`, named `PENDING-` so it sorts to be visible.

Future sessions naturally encounter `reports/` files when reading project state (e.g., per CLAUDE.md's "Read by function, not folder adjacency" doctrine and the `/project-report` skill's prior-reports walk). This file is structured to be self-contained: a fresh session can pick it up cold and execute it correctly.

## Composition with the day's arc

This re-measurement closes the only open thread from the 2026-04-29 arc:

- ✅ Comedy-rhythm doctrine landed at characterized tier (`f1bc122`)
- ✅ World-level anchor-groove diagnosed (`03031aa`)
- ✅ Plumbing root cause identified (`9a2f5dc`)
- ✅ 7-call-site plumbing fix shipped (`603f03d`)
- ✅ Regression test added (`b2c9543`)
- ✅ Cross-character validation at claim tier (`76ebb9c`)
- ✅ Lived-play verification anecdotal (`03e6dc3` OBSERVATIONS entry)
- ⏳ **Corpus-level deployment validation** ← this PENDING task

Once the post-deployment re-measurement lands, the arc is fully closed and the day's work has empirical grounding at every layer.

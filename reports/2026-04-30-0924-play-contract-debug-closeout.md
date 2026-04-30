# /play Contract Debug Closeout

## Purpose

Canonical closeout for the `/play` Cursor contract debugging arc focused on chooser-tool execution semantics and header-HUD enforcement.

## Incident classes resolved

1. **Chooser-tool omission at turn close**
   - Symptom: plain-text turn close without `AskUserQuestion`.
   - Resolution: immediate runtime correction + regression fixture.

2. **HUD title-shape spoof risk**
   - Symptom: header detector accepted partial/non-turn title.
   - Resolution: exact title-line pattern enforcement (`WORLDTHREADS BUILDER — Turn N`).

## Structural hardening delivered

- Header-only HUD contract codified for Cursor `/play`.
- Box-alignment hook removed as obsolete for header mode.
- Stop-hook HUD presence tightened with turn-title regex.
- Stress harness expanded and split into:
  - fast preflight: `make play-contract-smoke` (4 checks),
  - full suite: `make play-contract-stress` (8 checks).
- Receipt artifact updated with runnable command guidance.

## Key artifacts

- Receipt: `reports/2026-04-30-0917-play-chooser-hud-contract-receipt.md`
- Stress harness: `scripts/play-contract-stress.py`
- Workflow targets: `Makefile` (`play-contract-smoke`, `play-contract-stress`)
- Hook update: `.claude/hooks/check-play-hud-present.py`

## Commit ledger

- `2120a26` — harden `/play` contract debug loop with smoke/full preflight
- `e139b20` — finalize Cursor contract surfaces and remove obsolete alignment hook

## Verification snapshot

- Smoke: `play-contract-smoke: passed 4/4`
- Full: `play-contract-stress: passed 8/8`
- Working tree reached clean state after cleanup commit.

## Outcome

The `/play` interaction contract now behaves as requested in this environment:

- chooser selection triggers immediate execution,
- turn closure is tool-driven,
- HUD shape is strict and non-spoofable under the current header contract,
- and the behavior is regression-tested with fast and full paths.

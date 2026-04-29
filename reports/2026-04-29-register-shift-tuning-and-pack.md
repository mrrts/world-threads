# Register-Shift Tuning + Pack Characterization (2026-04-29)

## Scope

This note captures four shipped items:

1. Before/after metric read on register-shift behavior.
2. New `--show-full-messages` audit mode.
3. New pack-level gates for live characterization.
4. Consolidated command outputs + interpretation for Darren and Jasper.

## Command Surface Added

- `worldcli register-shift`
  - New flags:
    - `--show-full-messages`
    - `--full-message-max-chars <n>` (default `1200`)
  - Existing `--show-messages` remains for short snippets.
- `worldcli register-shift-pack <character_id>`
  - New gates:
    - `--gate-min-speech-first-rate <0..1>`
    - `--gate-min-shift-run-rate <0..1>`
  - Fails non-zero when requested gates are not met.

## Before vs After (Session Baseline -> Current)

Baseline values are from the earlier run in this same session (before this tuning pass):

- Darren (`limit=80`): `shift_rate=0.20`, `rebound_rate=0.25`
- Jasper (`limit=80`): `shift_rate=0.3375`, `rebound_rate=0.80`

Current values (`worldcli --scope full --json register-shift --character ... --limit 80`):

- Darren: `shift_rate=0.70`, `rebound_rate=0.28`, `avg_shifts_per_message=1.4375`
- Jasper: `shift_rate=0.8125`, `rebound_rate=0.3684`, `avg_shifts_per_message=2.3125`

### Interpretation

- Shift detection rose sharply for both characters after lexicon/neutral-fallback tuning.
- Rebound rates changed less dramatically and now read as a stricter signal compared to broad shift detection.
- This is expected: the old heuristic under-counted many sentences as unscored; the new one maps more lines into a register path.

## Show-Full-Messages Smoke Test

Command:

`worldcli --scope full --json register-shift --character ddc3085e-0549-4e1f-a7b6-0894aa8180c6 --limit 10 --show-full-messages --full-message-max-chars 500`

Result:

- Sample rows now include expanded `message` text with a hard character cap.
- Guardrail works: long messages are clipped rather than dumping entire turns.

## Register-Shift-Pack (Gated) Results

Gate settings used:

- `--gate-min-speech-first-rate 0.8`
- `--gate-min-shift-run-rate 0.8`

### Darren

Command:

`worldcli --json register-shift-pack ddc3085e-0549-4e1f-a7b6-0894aa8180c6 --confirm-cost 5 --gate-min-speech-first-rate 0.8 --gate-min-shift-run-rate 0.8`

Output highlights:

- `speech_first_rate=1.0` (5/5)
- `shift_run_rate=1.0` (5/5)
- `gate.passed=true`

## Recommended Gate Profiles

Use these as presets while tuning characters:

- **Loose (exploration):**
  - `--gate-min-speech-first-rate 0.60`
  - `--gate-min-shift-run-rate 0.60`
- **Medium (default working bar):**
  - `--gate-min-speech-first-rate 0.80`
  - `--gate-min-shift-run-rate 0.80`
- **Strict (pre-ship bar for comedy-invited packs):**
  - `--gate-min-speech-first-rate 1.00`
  - `--gate-min-shift-run-rate 0.90`

Rationale:

- Speech-first should be near-perfect when the probe explicitly invites play/bit-comedy.
- Shift-run rate can be slightly lower than speech-first because some replies may stay tonally clean without a strong mid-turn pivot.

## One-Command Dashboard Script

Use `scripts/register-shift-dashboard.sh` to run:

- `register-shift` on Darren + Jasper (`limit=80`)
- gated `register-shift-pack` on Darren + Jasper

It prints raw JSON payloads so you can diff, archive, or pipe through `jq`.

### Quickstart (Daily Ritual)

```bash
# Strict rebound-focused run (recommended default ritual)
./scripts/run-rebound-strict.sh

# Keep artifact storage tidy (retain newest 2 dashboard runs)
./scripts/prune-register-shift-artifacts.sh 2

# Compare latest run against a prior run
./scripts/compare-register-shift-runs.py \
  reports/register-shift-dashboard-<old> \
  reports/register-shift-dashboard-<new>
```

### Toolbelt

- `scripts/run-rebound-strict.sh`
  - Runs the strict rebound ritual in one command.
- `scripts/register-shift-dashboard.sh`
  - Main dashboard runner with preset support and artifact capture.
- `scripts/prune-register-shift-artifacts.sh <keep_count>`
  - Prunes older dashboard run directories.
- `scripts/compare-register-shift-runs.py <old_dir> <new_dir>`
  - Prints metric deltas between two artifact runs.
- `scripts/latest-register-shift-run.sh [--json]`
  - Prints the newest dashboard artifact directory path.

### Rebound Variant Command Examples

- Standard pack (default):
  - `worldcli --json register-shift-pack <character_id> --confirm-cost 5`
- Rebound-focused pack:
  - `worldcli --json register-shift-pack <character_id> --variant rebound --confirm-cost 5`
- Rebound-focused pack with gates:
  - `worldcli --json register-shift-pack <character_id> --variant rebound --confirm-cost 5 --gate-min-speech-first-rate 1.0 --gate-min-shift-run-rate 0.9`

### Dashboard Rebound/Artifact Controls

- Strict run + rebound pack + character-level rebound floor:
  - `RUN_REBOUND_PACK=true SHIFT_MIN_REBOUND_RATE=0.25 ./scripts/register-shift-dashboard.sh strict`
- Artifact pruning helper (keep newest N run dirs):
  - `./scripts/prune-register-shift-artifacts.sh 2`
- Drift comparison:
  - `./scripts/compare-register-shift-runs.py reports/register-shift-dashboard-<old> reports/register-shift-dashboard-<new>`

### Latest Drift Snapshot (081959 -> 082326)

```
[darren-register-shift.json]
  shift_rate: 0.7000 (+0.0000)
  rebound_rate: 0.2800 (+0.0000)
  avg_shifts_per_message: 1.4375 (+0.0000)

[jasper-register-shift.json]
  shift_rate: 0.8125 (+0.0000)
  rebound_rate: 0.3684 (+0.0000)
  avg_shifts_per_message: 2.3125 (+0.0000)

[darren-pack.json]
  speech_first_rate: 1.0000 (+0.0000)
  shift_run_rate: 1.0000 (+0.0000)
  gate_passed: True -> True

[jasper-pack.json]
  speech_first_rate: 1.0000 (+0.0000)
  shift_run_rate: 1.0000 (+0.0000)
  gate_passed: True -> True
```

### Jasper Finn

Command:

`worldcli --json register-shift-pack fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 --confirm-cost 5 --gate-min-speech-first-rate 0.8 --gate-min-shift-run-rate 0.8`

Output highlights:

- `speech_first_rate=1.0` (5/5)
- `shift_run_rate=1.0` (5/5)
- `gate.passed=true`

## Bottom Line

- Auditability improved (`--show-full-messages` with explicit cap).
- Pack characterization is now CI-friendly via hard gates.
- Tuning made shift detection much more sensitive; downstream threshold calibration should now be based on this new measurement regime, not historical pre-tuning rates.

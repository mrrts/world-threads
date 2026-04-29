# CLI Agent Discovery (Quick Reference)

Use this when starting a fresh agent session and you need to rapidly discover available CLI surfaces.

## Core Discovery Sequence

```bash
worldcli --help
worldcli register-shift --help
worldcli register-shift-pack --help
./scripts/register-shift-dashboard.sh --help
./scripts/run-rebound-strict.sh --help
./scripts/prune-register-shift-artifacts.sh --help
./scripts/latest-register-shift-run.sh --help
./scripts/show-latest-register-shift-run.sh --help
./scripts/compare-register-shift-runs.py --help
./scripts/export-latest-register-shift-csv.sh --help
```

## Fast Paths

```bash
# Strict rebound ritual
./scripts/run-rebound-strict.sh

# Summarize latest run
./scripts/show-latest-register-shift-run.sh

# Machine-readable summary (pack rows only)
./scripts/show-latest-register-shift-run.sh --quiet --format json --latest-only pack

# Export latest CSV snapshot
./scripts/export-latest-register-shift-csv.sh
```

## Bootstrap + Completions

```bash
# Run all discovery checks and print pass/fail
./scripts/agent-cli-bootstrap.sh

# Machine-readable bootstrap result
./scripts/agent-cli-bootstrap.sh --json

# Optional bash completions for helper scripts
source ./scripts/completions/register-shift-tools.bash

# Optional zsh completions for helper scripts
fpath=(./scripts/completions $fpath)
autoload -Uz compinit && compinit
```

Completion setup details: `scripts/completions/README.md`

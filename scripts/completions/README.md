# Script Completions Setup

This folder contains helper completions for register-shift tooling scripts.

## Bash

```bash
source ./scripts/completions/register-shift-tools.bash
```

## Zsh

```zsh
fpath=(./scripts/completions $fpath)
autoload -Uz compinit && compinit
```

Provided completion targets:

- `./scripts/latest-register-shift-run.sh`
- `./scripts/show-latest-register-shift-run.sh`
- `./scripts/register-shift-dashboard.sh`
- `./scripts/run-rebound-strict.sh`

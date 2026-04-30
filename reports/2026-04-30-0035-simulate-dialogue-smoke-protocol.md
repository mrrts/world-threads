# Simulate-dialogue smoke protocol (worldcli)

Purpose: run a minimal, repeatable smoke test for `worldcli simulate-dialogue` that verifies both reply generation and budget-gate behavior.

## Scope

- Command surface: `simulate-dialogue` (single run)
- Character sample: any valid `character_id` from `list-characters`
- Baseline probe size: `--turns 2`
- Success criterion: one clean successful run with transcript + synthesis output

## Steps (repo root)

1. Build/confirm CLI availability:
   - `cd src-tauri && cargo run --bin worldcli -- --help`
2. Select a character id:
   - `cargo run --bin worldcli -- list-characters --json`
3. Run the baseline smoke:
   - `cargo run --bin worldcli -- simulate-dialogue <character_id> --turns 2`

## Expected gate behavior

If projected cost exceeds `per_call_usd`, command fails with a budget error similar to:

- `Error: Budget { kind: "per_call (simulate-dialogue total)", projected_usd: ..., cap_usd: 0.1, confirm_at_least: ... }`

This is a PASS for gate enforcement.

## Confirm-cost rerun

Rerun with the returned confirm threshold:

- `cargo run --bin worldcli -- simulate-dialogue <character_id> --turns 2 --confirm-cost <confirm_at_least>`

Expected successful output includes:

- Simulated transcript (`Ryan:` / character replies)
- `cost_usd: ...`
- `synthesis_model: ...`
- JSON synthesis block

## Notes

- Keep this as a smoke, not a benchmark.
- Use smallest viable turns first (`--turns 2`) before larger runs.
- If this fails after `--confirm-cost`, treat as feature regression and open a fix issue with command + stderr.

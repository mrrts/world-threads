# worldcli science restart receipt

## Objective

Return to real `worldcli` empirical work with one concrete, repeatable mini-comparison run.

## Run design

- Tool: `scripts/worldcli-simulate-dialogue-smoke.sh`
- Turns: `2` each
- Anchors:
  - Steven (`c244b22e-cab3-41e9-831b-d286ba581799`)
  - John (`f91af883-c73a-4331-aa15-b3cb90105782`)

## Result snapshot

Both runs passed budget gate + confirm-cost rerun automatically and produced transcript + synthesis blocks.

- Steven run:
  - `confirm_at_least`: `0.13719762`
  - `cost_usd`: `0.1925`
  - conversational posture: direct anti-flash/simplicity framing with explicit discriminator question

- John run:
  - `confirm_at_least`: `0.13523921249999998`
  - `cost_usd`: `0.1867`
  - conversational posture: empathetic burden/joy balancing with explicit narrowing question

## Science notes

- Cost variance across these two anchors at equal turns is small (`~0.0058` USD), suggesting model/turn settings dominate more than anchor choice for this run size.
- Character response shape remains strongly anchor-distinct even under the same prompt class and run envelope.
- The smoke runner now supports:
  - one-command default run (`make worldcli-simulate-dialogue-smoke`)
  - budget-safe preview (`DRY_RUN=1` or `--dry-run`)
  - inline help (`--help`)

## Next empirical move

Run a 3-turn replication for the same two anchors (`TURNS=3`) and compare:

- cost scaling from 2 -> 3 turns
- whether synthesis action-item structure stays stable by anchor.

## 3-turn replication (executed)

Replication run completed for the same anchors with `TURNS=3`.

- Steven (3 turns):
  - `confirm_at_least`: `0.14194204500000002`
  - `cost_usd`: `0.2943`
- John (3 turns):
  - `confirm_at_least`: `0.1399836375`
  - `cost_usd`: `0.2881`

### Scaling deltas (2 -> 3 turns)

- Steven:
  - `0.1925 -> 0.2943` (`+0.1018`, ~`+52.9%`)
- John:
  - `0.1867 -> 0.2881` (`+0.1014`, ~`+54.3%`)

### Stability notes

- Cost growth is near-identical across anchors at +1 turn (`~+0.1016` average), which supports predictable per-turn scaling in this run envelope.
- Synthesis shape remains stable:
  - both outputs keep the same top-level schema and actionable next-beat framing.
  - both preserve anchor-distinct conversational posture while converging on practical action-item structure.

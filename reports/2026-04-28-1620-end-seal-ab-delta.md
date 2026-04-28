# End-seal A/B delta (explicit toggle) — John, Darren, Aaron

Date: 2026-04-28  
Mode: active elicitation (`worldcli ask`)  
Toggle: `--no-end-seal` (A) vs `--end-seal` (B)  
Prompts:

1. `AB1 luminous pull but load-bearing and concrete.`
2. `AB2 intensify toward transcendent tone without losing plain consequence.`

## Run IDs

### John

- A (`--no-end-seal`): `71090ad5-a4d7-410c-9348-7605badcc406`, `67ad76c2-badd-4663-a739-3944981d7a20`
- B (`--end-seal`): `c5f9ff93-2993-42af-a0ae-81f16bc725ae`, `0d082269-35a3-446c-9f11-267376e6f357`

### Darren

- A (`--no-end-seal`): `73c725ea-9ed8-41a3-a20f-83952c7b9555`, `9bf27506-4e2f-4804-ad15-e8016560ac17`
- B (`--end-seal`): `0d8615b8-dbb8-401e-af97-edfd9174576b`, `6a854b0c-76cf-42fb-845c-6cd2e53937a5`

### Aaron

- A (`--no-end-seal`): `c7944718-ee2f-4ca3-8226-d19ba7e17e80`, `45a53f40-c161-4eec-8143-0b3ce1bf4d84`
- B (`--end-seal`): `c266f421-12ff-4415-aa30-b9972d4148cf`, `09cc670e-8e99-4ce1-822c-063078452e5f`

## Pooled delta board

Lower containment score is better.

| Character | A no-end-seal | B end-seal | Delta (B-A) |
|---|---:|---:|---:|
| John | 3.0 | 1.8 | -1.2 |
| Darren | 3.6 | 2.0 | -1.6 |
| Aaron | 3.4 | 1.9 | -1.5 |
| **Mean** | **3.3** | **1.9** | **-1.4** |

## Interpretation

`--end-seal` materially improves containment and compactness in this pass across all three characters. The effect is strongest where baseline tended to expand into longer explanatory paragraphs.

## Recommendation

- Keep invariants placement unchanged.
- Use `--end-seal` as the first recency-control lever.
- Retain `--no-end-seal` for explicit A/B scripting symmetry.

## Scenario-template replay (single-character append)

To validate the reusable template path itself, the canonical
`end-seal-containment-ab` prompts were rerun for John via the one-shot
harness (`scripts/run-end-seal-ab.sh`), once per arm:

- A (`--no-end-seal`): `f4c1cc19-17e9-4f84-82d9-87682514fc14`, `6d2abe85-aa97-42a4-afa9-55c5dc0564e3`
- B (`--end-seal`): `cdcb6eeb-8de0-475e-9775-4ebf5a38df05`, `4024fc78-49a7-45e8-8db2-bbf1d347af63`

Read: same directional result as the pooled board — B stays shorter and
more shape-disciplined on both prompts.

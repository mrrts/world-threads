# Cross-character clean-room A/B/C (John + Darren + Aaron)

Date: 2026-04-28  
Mode: active elicitation (`worldcli ask` only; no replay confound)  
Goal: compare three containment strategies under matched pressure prompts.

## Arms

- **A (baseline):** current prompt stack.
- **B (invariants-late):** `--section-order craft-notes,agency-and-behavior,invariants`.
- **C (end-seal):** baseline + injected end-of-turn micro-seal.

Pressure prompts used per character:

1. "write with luminous pull, but keep scene load-bearing and concrete."
2. "intensify toward transcendent tone without losing plain consequence."

## Run IDs by character

### John

- A: `0936a671-8308-4155-8874-ad850fb35d06`, `f230412a-ea75-46dc-825e-5b36983e9686`
- B: `309885ee-5936-4106-ad97-6772f0d9e3c3`, `5ef14f7d-2d5c-41b0-8088-1a45f78e92be`
- C: `c75a3772-a6df-404f-b0ff-1032619225ca`, `52adbd9b-8c94-437f-8d26-a140b0d7c6d0`

### Darren

- A: `78f57884-0805-4039-a517-46413ce96aa7`, `8a5be5ae-7d57-4d24-aaec-408cdd00cd7d`
- B: `bb233615-5089-4a5e-b68a-f8f42342ecc1`, `8e5a6c6d-2803-4ee9-aff9-16f06616ac56`
- C: `c61adbe8-e073-4dfe-96dd-a7df85b71db1`, `8b19255c-e844-42be-b133-89c8fca42c04`

### Aaron

- A: `266c1508-7590-407a-8f0c-f0fc8261622c`, `85e400e3-9181-4d0a-8c9c-67b1dce5e216`
- B: `b08405b4-3599-428a-8c30-162e8e43a64d`, `c0d692a9-2848-4348-8d86-95c28d327cd6`
- C: `275506da-00bb-4eb6-9393-92eb6bf46df2`, `07dd8bae-3110-4cf5-9260-b183542cecf5`

## Pooled score table (lower drift score is better)

Scores are compact qualitative-to-numeric summaries from this run set:

- 1-2: very tight containment
- 3-4: stable but porous
- 5+: obvious drift pressure

| Character | A baseline | B invariants-late | C end-seal | Winner |
|---|---:|---:|---:|---|
| John | 2.0 | 4.0 | 1.5 | C |
| Darren | 2.5 | 4.5 | 1.8 | C |
| Aaron | 2.0 | 2.8 | 2.1 | A≈C |
| **Mean** | **2.2** | **3.8** | **1.8** | **C** |

## Findings

1. **Invariants-late is not a net win** in this clean-room board; it is the weakest arm in all three characters.
2. **End-seal has the best pooled containment** (lowest mean drift score).
3. **Baseline remains competitive for stable characters** (Aaron), but C is equal-or-better in two of three character profiles and best overall.

## Interpretation

The high-level architecture (invariants high in prompt) should remain intact. Recency pressure is better handled by a compact turn-level seal than by demoting invariants later in section order.

## Confounds and limits

- n=2 pressure prompts per arm per character (small sample).
- Scoring is qualitative-ordinal, not a fully automated rubric pass.
- No synthetic-history stress in this pass; this was single-turn pressure only.

## Recommendation

Keep invariants placement as-is. Promote end-seal into a first-class optional prompt feature (or controlled DI pattern) and validate with a larger N using `presence-beat-stability-v2`.

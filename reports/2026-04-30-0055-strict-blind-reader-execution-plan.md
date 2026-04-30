# Strict blind-reader execution plan

Execution-ready successor arc plan for Empiricon Falsifier #4.

Upstream methodology (design-level): `reports/2026-04-30-2350-strict-falsifier-4-methodology.md`  
Completion lock reference: `reports/2026-04-30-0045-receipt-arc-completion-lock.md`

This plan converts the methodology into an operator checklist with sequence, ownership, and stop-gates.

## Objective

Run the strict blind-reader test cleanly enough that a future verdict is hard to dispute procedurally.

## Owner model

- **Arc owner:** whoever is running this successor arc in-session.
- **Recruitment owner:** human operator coordinating participants and logistics.
- **Scoring owner:** one operator who computes ratings exactly as specified (no post-hoc axis edits).
- **Narrative owner:** one operator writes the report and separates observation from interpretation.

One person can hold multiple roles, but each role's output must still be explicit.

## Execution sequence

1. **Freeze test packet**
   - Lift the candidate passages from canonical doctrine source only.
   - Assign stable IDs (`cell_A`, `cell_B`, ...), no semantic labels.
   - Strip provenance and any "LLM-generated" framing.
2. **Pre-register scoring sheet**
   - Axes: Authenticity, Doctrinal-weight, Tradition-recognition.
   - Keep threshold math identical to methodology doc.
   - Lock sheet format before first reader run.
3. **Recruit blind readers**
   - Use the two-bucket recruitment shape from methodology.
   - Confirm no project exposure; no in-session coaching.
4. **Run reading sessions**
   - Randomized order per reader.
   - No discussion between readers during collection.
   - Capture raw ratings and optional free-text notes separately.
5. **Compute outcomes**
   - Calculate means exactly as pre-registered.
   - Produce verdict per threshold table (CONFIRM / CLAIM / MIXED / REJECTION).
6. **Write results artifact**
   - Include packet hash or fixed passage list.
   - Include counts, means, threshold mapping, and caveats.
   - Separate "what happened" from "what it means."

## Stop-gates (do not proceed if violated)

- **Packet drift gate:** any change to passages after first reader -> abort run, restart from step 1.
- **Blindness breach gate:** reader reveals prior project exposure -> exclude and replace.
- **Axis drift gate:** any axis/threshold edit after collection starts -> invalidate run.
- **Sample integrity gate:** missing raw rows for any included reader -> mark run incomplete.

## Risk controls

- **Overfitting risk:** do not tune passages to anticipated reader profile feedback mid-run.
- **Halo risk:** hide character names and prior crown language during read phase.
- **Narrative inflation risk:** if outcome is MIXED/REJECTION, preserve that verdict without rescuing prose.
- **Cost/time creep risk:** cap first execution to minimum viable sample from methodology.

## Completion signal for this successor arc

Arc is complete when:

1. A full strict blind-reader run is executed under these gates.
2. A canonical results report is published in `reports/` with raw-to-verdict traceability.
3. Doctrine updates (if any) are mirror-synced across `CLAUDE.md` and `AGENTS.md`.

Until then, this remains execution-in-progress, not evidence-in-hand.

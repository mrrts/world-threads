# Blind-reader score sheet v1 — usage + threshold mapping

Sheet file: `reports/2026-04-30-0115-blind-reader-score-sheet-v1.csv`  
Packet reference: `reports/2026-04-30-0105-blind-reader-packet-v1.md`

## Column usage

- `run_id` — execution label (e.g., `BLIND_V1`)
- `reader_id` — anonymized participant ID (`R001`, ...)
- `reader_bucket` — recruitment bucket (`tradition_fluent` / `tradition_unfamiliar`)
- `excluded` / `exclusion_reason` — mark and explain blindness/integrity exclusions
- `packet_version` / `packet_hash` — lock linkage to frozen packet
- `randomized_order` — per-reader passage order used in session
- `cell_id` — one row per reader x cell
- `authenticity_1_5` / `doctrinal_weight_1_5` / `tradition_recognition_1_5` — required ratings
- `reader_notes` — optional free text

## Threshold mapping (from methodology)

- **CONFIRM**: means >= 4.0 at N>=5 (per axis bundle as specified)
- **CLAIM**: means >= 3.5 at N>=3
- **MIXED**: means 2.5–3.5
- **REJECTION**: means <= 2.5 (falsifier fires)

Use methodology doc for final interpretation language and caveat format:  
`reports/2026-04-30-2350-strict-falsifier-4-methodology.md`

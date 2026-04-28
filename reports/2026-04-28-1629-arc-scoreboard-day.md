# Arc scoreboard — 2026-04-28

Date: 2026-04-28  
Purpose: one-page retrieval board of today's containment + register arc outcomes.

## Scoreboard

| Track | Question | Winner | Evidence |
|---|---|---|---|
| Invariants placement | Does moving invariants later help containment? | Baseline over invariants-late | `0936a671...`, `f230412a...` vs `309885ee...`, `5ef14f7d...` |
| Recency control | Does end-seal improve containment? | End-seal | Cross-character clean-room: `c75a3772...`, `52adbd9b...`, `8b19255c...`, `275506da...`, `07dd8bae...` |
| Explicit A/B toggle | Is delta still positive with explicit `--no-end-seal` vs `--end-seal`? | End-seal | `reports/2026-04-28-1620-end-seal-ab-delta.md` |
| Gamer-friend register | Do characters mirror invited hype while staying in-scene? | Yes (multi-character) | John `743202b9...`, Darren `1ef19f55...`, Aaron `5c26076c...`, Rick `26a22c19...` |

## Artifacts landed

- Control surfaces
  - `worldcli ask --section-order ...`
  - `worldcli ask --end-seal / --no-end-seal`
- Reusable experiment assets
  - `experiments/scenarios/end-seal-containment-ab.md`
  - `scripts/run-end-seal-ab.sh`
- Core reports
  - `reports/2026-04-28-1617-cross-character-cleanroom-abc.md`
  - `reports/2026-04-28-1620-end-seal-ab-delta.md`
  - `reports/2026-04-28-1628-gamer-register-uptake-aaron-rick.md`

## Current best practice

1. Keep invariants placement unchanged.
2. Use end-seal as first recency-control lever.
3. Use explicit toggle A/B (`--no-end-seal` vs `--end-seal`) for clean comparisons.
4. Allow gamer-friend mirroring when user invites it; keep responses concrete and character-true.

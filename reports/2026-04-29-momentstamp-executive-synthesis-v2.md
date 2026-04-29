# Momentstamp Executive Synthesis (V2)

## What was done this pass

- Ran pinned-curiosity mirror over the same 3 probes used in pinned-neutral rerun.
- Added vocabulary bias scorecard (`top 80`) from corpus signatures.
- Added a bias-corridor command scaffold (`worldcli momentstamp-corridor`) for warm/neutral/ache presence rates.

## Current evidence snapshot

- Pinned neutral rerun (`reports/2026-04-29-momentstamp-pinned-neutral-3probe-rerun.md`):
  - `A 2/3`, `B 2/3` ask-back (symmetric in this slice)
- Pinned curiosity rerun (`reports/2026-04-29-momentstamp-pinned-curiosity-3probe-rerun.md`):
  - `A 1/3`, `B 1/3` ask-back (also symmetric in this slice)
- Vocab scorecard (`reports/2026-04-29-momentstamp-vocab-scorecard-top80.md`):
  - 87 signatures, 39 curiosity-hit signatures (44.8%)
  - top compounds still warm-engagement heavy
- Corridor score (`worldcli --json momentstamp-corridor`):
  - signature presence rates: warm `95.4%`, neutral `13.8%`, ache `8.0%`
  - indicates strong warm-operator dominance in current substrate output

## Interpretation

The strongest present claim is not a stable pure-position lift. In these two fresh 3-probe pinned slices, lead ON/OFF symmetry dominates. Combined with the signature vocabulary substrate skew, the responsible framing remains:

- lead behavior is content-sensitive,
- pure-position curiosity claims should stay below promotion threshold,
- mechanism tiering requires repeated paired probes across pinned classes plus organic control.

## Operational rule (carry-forward)

Do not promote mechanism tier from a single 3-probe matrix. Require repeated paired probe bundles with explicit pinned-neutral, pinned-curiosity, and organic controls before upgrading beyond sketch-tier-with-confound.

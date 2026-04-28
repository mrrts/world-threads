# experiments/ — registry

Structured hypothesis files. One file per experiment, named `<slug>.md`,
with YAML-ish frontmatter + optional markdown body for interpretation.

The registry is the query layer above `evaluate-runs`, `synthesize-runs`,
and `replay-runs`. Prose reports under `reports/` hold the full interpretation;
experiment files hold the machine-queryable shape — what hypothesis was tested,
what mode, which refs, which rubric, what prediction, which runs produced which
result, what it opens for next time.

Deeper rule: the hypothesis slug is the durable memory unit; individual run
ids are evidence attachments. When reconstructing an investigation's braid,
start from the question being carried across sessions, then read the runs as
the evidence leaves attached to that question.

See `worldcli lab --help` for the CLI surface.

Two useful registry reads now exist side by side:

- `worldcli lab open` — the unfinished queue
- `worldcli lab summary` — the shelf-level read by status and heuristic
  bet-family hints

The summary view is intentionally suggestive rather than canonical. It helps the
reader notice whether the shelf is clustering around structural-bite bets,
scope-and-direction bets, or partial-real / instrument-sensitive bets, but the
reports remain the final interpretive surface.

## Frontmatter schema

Scalars:

- `id`: slug (matches filename)
- `status`: `proposed` | `running` | `open` | `discrepant` | `confirmed` | `refuted`
- `mode`: `passive` | `qualitative` | `active`
- `ref`: git sha/tag/branch the experiment pivots on
- `rubric_ref`: name from `reports/rubrics/` (for Mode A passive runs)
- `evidence_strength`: **legacy scalar** evidentiary label, often a compound
  expression like `claim-narrow,sketch-directional`. Kept for backward compat;
  prefer the structured form (`strength_axes` + `strength_provenance` below).
  When only the legacy scalar is present, `worldcli lab` auto-derives
  `strength_axes` from it on load (no migration pass required).
- `bet_family`: explicit override for `worldcli lab summary`'s family
  classifier. Bypasses the prose-grep heuristic when set. Values:
  `structural_bite` | `scope_and_direction` | `partial_real_instrument_sensitive` | `other`.
- `created_at`: ISO 8601
- `resolved_at`: ISO 8601 (set when status becomes confirmed/refuted/open/discrepant)

Block scalars (multi-line prose):

- `hypothesis`: the falsifiable claim, one or two sentences
- `prediction`: what CONFIRMED looks like / what REFUTED looks like,
  written BEFORE any LLM call
- `summary`: the result, added at `lab resolve` time
- `strength_provenance`: prose explanation of the strength labels — when,
  why, what changed, what report covers it. Replaces the YAML-comment
  provenance previously braided into `evidence_strength`. Set via
  `lab resolve --strength-provenance "..."`.

Flat string-lists:

- `strength_axes`: structured per-axis tier labels, of form `axis:tier`
  (e.g. `narrow:claim`, `directional:sketch`). One entry per axis. The
  layer-5-promoted form of what `evidence_strength` was braiding into one
  scalar (tier label + axis-state + comma-separated multi-axis composite).
  Tiers: `sketch` | `claim` | `characterized` | `vacuous-test` |
  `ensemble-vacuous` | `tested-null` | `accumulated` | `unverified`.
  Set via `lab resolve --axis name:tier` (repeatable). Surfaced 2026-04-28
  by Codex via the CROSS_AGENT_COMMS handoff: the prior single-scalar
  shape was forcing the family classifier to lean on prose-grep.
- `scope_characters`: character ids in scope
- `scope_group_chats`: group-chat thread ids in scope
- `run_ids`: ids of evaluate/synthesize/replay runs that produced the result
- `follow_ups`: slugs of experiments this one suggests (forward references)
- `reports`: paths to prose reports carrying the full interpretation

### Worked example — structured strength shape

```yaml
status: confirmed
strength_axes:
  - "narrow:claim"
  - "directional:sketch"
strength_provenance: |
  Upgraded 2026-04-24. Narrow effect at claim-tier; directional claim
  refuted-at-claim-tier. See reports/2026-04-24-2320.
bet_family: structural_bite
```

Equivalent legacy form (still readable, auto-derives `strength_axes` on load):

```yaml
status: confirmed
evidence_strength: claim-narrow,sketch-directional  # Upgraded 2026-04-24. ...
```

The `lab resolve` command can write either; new resolutions should prefer the
structured form. The legacy scalar is also auto-populated from `strength_axes`
when `--axis` is passed without an explicit `--evidence-strength`, so old
readers see a coherent compound value without the author having to write it.

## Status lifecycle

- **proposed**: the hypothesis has been auditioned (typically during the
  `run-experiment` skill's Step 1) but hasn't been executed yet. Multiple
  candidates from one audition can live as `proposed` simultaneously;
  only the chosen one advances to `running`.
- **running**: the experiment is in flight (an LLM run has been initiated).
- **open**: executed but result is ambiguous — more data needed.
- **discrepant**: executed and interpreted, but instrument families disagree in a
  way the registry should preserve honestly rather than flatten into `open`.
- **confirmed**: prediction held.
- **refuted**: prediction did not hold (this is a real result, not a failure).

## What counts as a registry entry

One experiment = one hypothesis = one file. Use the registry for hypotheses
worth tracking across sessions — things whose answer would shape craft
decisions. Don't register every one-off curiosity run; those live in the
run-log (`evaluate-runs`, `synthesize-runs`, `replay-runs`) and that's
enough.

The bar: *would a future Claude Code session want to know this experiment
existed and what its result was?* If yes, register it. If no, the run-log
is sufficient.

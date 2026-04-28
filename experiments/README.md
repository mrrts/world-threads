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

## Frontmatter schema

Scalars:

- `id`: slug (matches filename)
- `status`: `proposed` | `running` | `open` | `confirmed` | `refuted`
- `mode`: `passive` | `qualitative` | `active`
- `ref`: git sha/tag/branch the experiment pivots on
- `rubric_ref`: name from `reports/rubrics/` (for Mode A passive runs)
- `evidence_strength`: evidentiary-tier label preserved across `worldcli lab`
  rewrites
- `created_at`: ISO 8601
- `resolved_at`: ISO 8601 (set when status becomes confirmed/refuted/open)

Block scalars (multi-line prose):

- `hypothesis`: the falsifiable claim, one or two sentences
- `prediction`: what CONFIRMED looks like / what REFUTED looks like,
  written BEFORE any LLM call
- `summary`: the result, added at `lab resolve` time

Flat string-lists:

- `scope_characters`: character ids in scope
- `scope_group_chats`: group-chat thread ids in scope
- `run_ids`: ids of evaluate/synthesize/replay runs that produced the result
- `follow_ups`: slugs of experiments this one suggests (forward references)
- `reports`: paths to prose reports carrying the full interpretation

## Status lifecycle

- **proposed**: the hypothesis has been auditioned (typically during the
  `run-experiment` skill's Step 1) but hasn't been executed yet. Multiple
  candidates from one audition can live as `proposed` simultaneously;
  only the chosen one advances to `running`.
- **running**: the experiment is in flight (an LLM run has been initiated).
- **open**: executed but result is ambiguous — more data needed.
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

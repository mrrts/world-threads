# Rubric library

Versioned, reusable rubrics for `worldcli evaluate`. Each rubric is a markdown file whose `# Rubric` section is the exact text sent to the evaluator, and whose other sections accumulate craft capital: known failure modes, when to use it, a run history that gets appended automatically.

## Why this exists

The first six `evaluate` runs in this repo (reports 1037 through 1326) each wrote their rubric inline, died in a single report's prose, and couldn't be reused. Proposal 1 from the 2026-04-23-1400-better-lab-vision report: extract rubrics into a versioned library so craft capital compounds across experiments instead of each run re-inventing.

## How to use a rubric from the library

```bash
worldcli evaluate --ref <sha> --character <id> \
    --rubric-ref weight-carrier-hold-vs-reduce \
    --limit 12
```

`--rubric-ref <name>` looks up `reports/rubrics/<name>.md`, extracts the `# Rubric` section, and uses it as the evaluator prompt. Mutually exclusive with `--rubric` and `--rubric-file`.

## How to write a new rubric

Copy an existing file as a template. Required sections:

- **Frontmatter** with `name`, `version`, `description`.
- **`# Rubric`** — the full text the evaluator sees. Include worked `yes` / `no` / `mixed` examples inline; the worked examples calibrate the evaluator and embedding them in the prompt is the rubric-writing discipline worked out across the 1304 / 1326 runs.
- **`# Known failure modes`** — what has gone wrong when this rubric was used. Updated after every run that surfaces a new failure.
- **`# When to use`** — a one-paragraph description of the question-shape this rubric is for.
- **`# Run history`** — appended automatically after each invocation.

When editing a rubric's prompt text, bump the `version` field in the frontmatter. Run history entries carry their rubric version so results aren't silently contaminated by edits.

## How rubrics evolve

A rubric is not static. Each run that exposes a new failure mode, a miscalibration, or an edge case is evidence for a next version. Update the `# Known failure modes` section every time. When the rubric's formal criteria no longer match the intent (the 1326 lesson: `"≤2 sentences"` gate excluding multi-beat-but-still-pastoral replies), edit the `# Rubric` section, bump the version, and note the change in an `# Edit history` section.

## Current rubrics

See `ls reports/rubrics/*.md` for the full list. `worldcli rubric list` is the same information.

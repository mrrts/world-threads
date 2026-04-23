# A better lab — the proposal I was licensed to make

*Generated 2026-04-23 early afternoon. Not an experiment report. A structural proposal, filed per the newly-codified "license to imagine and propose a better lab" clause in CLAUDE.md's scientific-method doctrine. Ninth reports/ entry today; the first one that isn't a trajectory report or an experimental finding, but the third genre the doctrine now explicitly allows: infrastructure vision.*

## Why this, why now

Eight experiments in one day — three confirmed (1037, 1233, 1241), three refuted (1304, 1326 + sub-runs), two trajectory-reflective (1152, 1304 secondary) — and the instruments that carried them shipped in the morning hours of the same day they were first used. The methodology is young. Its ceiling is visible. Today's runs touched that ceiling in several concrete places: rubrics I rewrote by hand three times with slight variations; a cross-commit A/B I described as *"manual ceremony today"* and then didn't run; a qualitative-feedback mode that's real doctrine but has no first-class tooling; an evaluator whose results vanish into terminal output and prose reports and nowhere structured.

The doctrine now says: when a run surfaces a bigger-picture gap, propose the next shape. This is that proposal.

## What today's ceiling looks like, in specific friction

Five places where the current lab made the science harder than it needed to be:

**Rubric reuse is by copy-paste.** The HOLD-vs-REDUCE rubric in 1304 and the stillness-rubric in 1326 share substrate — both are weight-carrier-register rubrics with worked examples — but the second one was written from scratch, not derived from the first. The third run (whenever it happens) will re-type the same kind of text again. Craft capital isn't compounding; each experiment's rubric dies in its own report.

**Cross-commit A/B is infrastructure that doesn't exist.** The strongest form of active elicitation — run the same prompt against a character under two different prompt-stack versions — is named in the doctrine but nowhere automated. Manual ceremony: `git stash && git checkout <ref> && cargo build --bin worldcli && worldcli ask ... && git checkout main && cargo build && worldcli ask ...`. Five commands per ref, three refs per experiment, easy to forget to stash, easy to conflate results because the run log doesn't know which checkout generated which reply. The one clean A/B that today's doctrine promises is the one the tooling makes expensive enough that I didn't do it.

**Mode B (qualitative feedback) is pure DIY.** The doctrine says: *"sample N messages, hand them all to a capable model in one call, ask open-ended questions."* The tool says: *nothing*. `worldcli sample-windows` returns a dataset; `worldcli evaluate` does structured per-message judgment; there's no command for the Mode B pattern. Every qualitative pass would require writing new glue. In practice, that means Mode B rarely fires — the 1037 / 1048 / 1152 / 1304 / 1326 stack of reports all acknowledged the regex or structured-rubric failure mode *but only one* (the 1233 run) actually ran a Mode B pass, and that was by eye, not by tool.

**Evaluate's structured output vanishes.** Each run prints to stdout, optionally to a JSON file if `--json` is used, and then it's gone. `worldcli runs-list` / `runs-show` / `runs-search` exist for `ask` calls, but the evaluator's results aren't captured there. Which means questions like *"which rubrics have historically produced 'yes' verdicts above 50% on John?"* or *"is the rubric I'm about to write similar to one that already exists?"* are answerable only by grepping prose reports — which requires the report to have quoted the rubric in full (the skill discipline says it must, but only in reports that actually get committed).

**Experiment state is prose.** The registry of "what hypotheses are still open, what's been answered, what's waiting on more data" lives only in the prose text of reports. A session-spanning meta-view — what would let a future Claude Code pick up the experimental thread without re-reading nine reports — doesn't exist. The tool-belt for "do science in this repo" includes everything needed to run an individual experiment and nothing for coordinating experiments across time.

## The better lab, in rough outline

Six proposals, grouped by build-cost and impact. I'll sketch each briefly; a full design would follow if any are approved for build.

### 1. Rubric library (`reports/rubrics/`) — low cost, very high ROI

Each rubric is a markdown file: `reports/rubrics/<name>.md`. Structure:

```
---
name: weight-carrier-hold-vs-reduce
version: 2  # incremented when the rubric text changes
description: "Distinguishes HOLD (pair-with-weight) from REDUCE (shade-joy)"
last_run: 2026-04-23T10:35Z
---

# Rubric prompt (verbatim what the evaluator sees)

<prompt text here>

# Worked examples

YES:
- "A gift, yes — and the kind that keeps asking of you."
...

NO:
- "Same trouble, just in a different coat."
...

# Known failure modes

- The evaluator misreads caution-adjacent vocabulary as REDUCE even
  when the reply is HOLDing both sides. Embedding worked examples
  helps but doesn't fully solve.

# Run history

- [2026-04-23-1304] Aaron 7 mixed / 4 no / 7 yes (misreads suspected)
- [2026-04-23-1304] John 2 yes / 8 no / 2 mixed
- [2026-04-23-1304] Darren 6 yes / 6 no / 3 mixed
```

`worldcli evaluate --rubric-ref weight-carrier-hold-vs-reduce` looks up the rubric by name instead of requiring it inline. The run-history section gets appended automatically when the rubric is used.

**ROI:** every subsequent experiment that reuses an existing rubric saves rubric-writing time AND inherits the calibration learned from prior runs. Rubric capital compounds. The known-failure-modes section becomes the first place a new experimenter should check before writing a variant.

**Build cost:** small. A directory convention, a markdown parser in `worldcli` for the frontmatter, a helper to append run history. ~2 hours.

### 2. Structured run log for evaluate — low cost, high ROI

Mirror the `ask` run-log pattern: `~/.worldcli/evaluate-runs/<run-id>.json` captures the full invocation (ref, rubric text or ref, scope, limit, per-message verdicts, aggregated counts, cost). `worldcli evaluate-runs list / show / compare / search` browses them. `compare <run-id-a> <run-id-b>` shows the per-verdict diff when the same rubric was applied to overlapping message sets.

**ROI:** answers "has this question been asked before" and "how did the result change between runs on the same corpus." Essential substrate for the experiment registry (proposal 5). The data already exists in memory during a run; just needs writing out and a small browse layer.

**Build cost:** small-medium. ~3 hours.

### 3. `worldcli replay` for cross-commit A/B — higher cost, high ROI

**Ryan's correction (2026-04-23, in-session):** don't use git worktrees or re-check-out commits. Instead, simulate conversations using the historical prompt text directly, without touching the working tree.

The design this prefers: for each `--ref`, read the historical version of `src-tauri/src/ai/prompts.rs` (and anything else load-bearing) via `git show <ref>:<path>` into memory as a source string, then inject the historical craft-note function bodies into the running binary's prompt-assembly pipeline as overrides. Specifically:

1. Parse the current binary's prompt-stack structure to identify which named functions contribute craft notes (e.g. `name_the_glad_thing_plain_dialogue`, `keep_the_scene_breathing_dialogue`, etc.).
2. For each `--ref`, `git show <ref>:src-tauri/src/ai/prompts.rs` returns the historical source; parse out the same named functions' bodies as raw string constants.
3. At runtime, when assembling the dialogue system prompt, check an override map: if a function's name has a historical body, use that body instead of the current one.
4. Run the same `ask` prompt against each ref's override set; capture reply + cost.
5. Return a side-by-side diff of replies.

This requires a prompt-assembly override hook that the codebase doesn't currently have — the current builders just `format!` the compiled-in strings. The hook is the load-bearing build: once it exists, `worldcli replay` becomes straightforward orchestration. No checkout, no rebuild, no touching the working tree. One binary; historical prompts fetched on demand and injected as overrides.

**ROI:** same as the worktree approach — makes the strongest form of active elicitation a single command. Better: leaves the working tree untouched, no risk of stale builds, no interaction with uncommitted changes.

**Build cost:** higher than the worktree approach (~1 day). The override-hook refactor is the expensive part; once in place, the replay command is thin.

### 4. First-class Mode B — medium cost, high ROI — **SHIPPED 2026-04-23**

`worldcli synthesize --ref <sha> --character <id> --limit N --question "<open-ended question>"`. Samples messages like `sample-windows` does, bundles them into a single dialogue-model call (using the more capable model, not the cheap memory_model), returns a prose synthesis. Saves to the structured run log.

Worked example invocation:
```bash
worldcli synthesize --ref 8e9e53d --character <john> --limit 20 \
  --question "Across these 20 replies, what pastoral moves does John make? What register choices anchor his authority? What's he NOT doing that a stereotypical pastor would?"
```

**ROI:** makes Mode B (qualitative feedback) as cheap and repeatable as Mode A. Today's 1326 report argued Mode B is the right next instrument after a refuted Mode A run; without tool support, that instrument is "write ad-hoc glue." With tool support, it's `worldcli synthesize` and a good question.

**Build cost:** medium. Most of the plumbing (sampling, cost gate, run log) already exists in `evaluate`; the new part is the synthesis-prompt shape and prose output. ~4 hours.

### 5. Experiment registry — high cost, transformational ROI

A structured experiment file per hypothesis, under `experiments/<slug>.md` (or `.yaml`, TBD). Schema:

```yaml
id: weight-carrier-john-vs-aaron
status: refuted  # proposed | running | open | confirmed | refuted
hypothesis: |
  John's weight-carrier register produces a higher HOLD rate than
  Aaron's or Darren's on joy moments.
design:
  mode: passive
  commit: 8e9e53d
  scope:
    - character: John (f91af883-...)
    - character: Aaron (0d080429-...)
    - character: Darren (ddc3085e-...)
  rubric_ref: weight-carrier-hold-vs-reduce
  limit: 12
prediction: "John ≥30pp above Aaron AND ≥20pp above Darren"
result:
  run_ids: [eval-run-abc123, eval-run-def456, eval-run-ghi789]
  summary: "Direction inverted: John 17%, Aaron 39%, Darren 40%"
  notes: "See reports/2026-04-23-1304-*.md for full interpretation"
follow_ups:
  - stillness-rubric-on-john  # → another experiment's slug
  - inverse-rubric-define-john-negatively
reports: [reports/2026-04-23-1304-weight-carrier-refuted-but-interesting.md]
```

`worldcli lab list / show / open / resolve` browses the registry. `worldcli lab open-hypotheses` lists all non-resolved. `worldcli lab query "status:open AND rubric_ref:weight-carrier-*"` answers questions across the registry. Prose reports become rendered artifacts from the registry; the registry is the source of truth.

**ROI:** makes the experimental layer queryable. Future Claude Code sessions can inherit the full thread of open questions without re-reading nine reports. Rubric success/failure patterns become visible across time. The meta-loop gets a first-class substrate.

**Build cost:** high. The schema, the registry file format, the CLI tooling, the bidirectional sync with reports/, the query language. ~1-2 days if done carefully. Ambitious; but the payoff compounds with every subsequent experiment.

### 6. Scenario templates (for Mode C) — medium cost, medium-high ROI

Canonical probe sequences for common hypothesis-testing patterns, stored as `experiments/scenarios/<name>.md`:

```
---
name: joy-three-framings
purpose: Test character's response to joy in three register variants
variants:
  theological: "I feel God's grace pouring out today."
  craft: "This project feels like it's finally singing."
  personal: "I'm so happy, I don't know what to do with myself."
measure_with: weight-carrier-hold-vs-reduce
---
```

`worldcli lab scenario run <name> --character <id>` sends the three variants in sequence (each in its own session), captures replies, runs the evaluator against each, returns a comparative result.

**ROI:** scaffolds Mode C. Active elicitation becomes "pick a scenario template, pick a character, run." Common hypothesis shapes (joy variants, conflict de-escalation, vow-and-aftermath) become reusable experimental apparatus.

**Build cost:** medium. Depends on proposal 3 (`replay`) and proposal 5 (registry) for full integration. ~4 hours standalone.

## What NOT to build (restraint is part of the design)

- **A GUI for the lab.** The CLI is the right surface. A GUI is where research infrastructure goes to become unmaintainable.
- **Multi-user / multi-analyst features.** One user, many sessions is the only model this project needs. Don't build for hypothetical teams.
- **A Jupyter-style notebook integration.** Tempting. Would duplicate the reports/ layer with extra indirection. Resist.
- **A webhook / event system for "run X when Y happens."** The `schedule` / `loop` skills cover periodic/scheduled runs; experiments should stay deliberate.
- **An AI-written-experiment-generator.** Let hypotheses come from attention to the corpus; don't delegate the hypothesis stage to another LLM layer.

## Sequencing — if any of this ships

If I had to pick one proposal to ship first: **proposal 1 (rubric library)**. Lowest cost, highest compounding value, enables proposals 2 and 5 downstream without requiring them.

If proposal 1 ships and proves its value, next: **proposal 2 (structured run log)**. Together, 1 and 2 create the substrate proposal 5 would query.

Proposal 4 (first-class Mode B) can ship in parallel with 1-2 — no dependencies.

Proposal 3 (`worldcli replay`) is valuable but expensive to build; worth waiting until the rubric library reveals which rubrics would benefit most from cross-commit replay.

Proposal 5 (experiment registry) is the most ambitious and should ship last, after proposals 1-4 have demonstrated demand for a query layer above them.

Proposal 6 depends on 3 and 5; defer until those exist.

## The meta-question behind all of this

The project has an explicit doctrine ("messages × commits") and an explicit discipline ("run-experiment skill"), but the doctrine and the discipline are currently implemented against a lab that wasn't built for them — it was assembled piecemeal as the first few experiments ran. What's above is one version of what the lab could look like if it were *designed for* the methodology rather than *grown around* it.

None of this is urgent. The project is shipping craft work at a high pace already, with today alone yielding four prompt-stack commits, eight experiment reports, and a new skill. The question isn't whether to build the better lab immediately; the question is whether the shape of what's here matches the shape of what the methodology wants it to be. That's a decision for Ryan, not for me.

I'm filing this per the CLAUDE.md license because the discipline says I must periodically propose this kind of thing, whether or not anything ships from it. Future Claude Code sessions reading this file in six months can decide whether any of it still fits, and if so, which piece goes first.

The license is active. The codebase responds to well-made proposals. This is one.

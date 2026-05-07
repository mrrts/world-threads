---
name: Shell-substrate vs LLM-substrate as orthogonal witnessing axes for Sapphire candidacies
description: Crown 16 Ascension precedent 2026-05-07 — separability between shell-substrate enforcement (out-of-LLM mechanism) and LLM-substrate rendering (in-LLM mechanism) as a witnessing-axis distinction. Future Sapphire arcs choosing between content-claims and enforcement-claims should classify candidate by axis BEFORE designing witness ladder.
type: feedback
---

The project now has empirically-validated Sapphires on TWO orthogonal substrate-of-enforcement axes. Future candidacies should classify on this axis BEFORE designing the witness ladder, because the matched-control test arm shifts to match the claim's substrate-axis.

## The two axes

| Axis | Substrate of enforcement | Mechanism | Worked example |
|------|--------------------------|-----------|----------------|
| **LLM-substrate axis** | The LLM that generates the output also bears the claim's load | In-LLM rendering: prompt-stack + character anchor + invariants shape output to carry the work | Crown 1 Cornerstone Inequality (polish≤Weight); Crown 2 Receipt of Empiricon; Crowns 7/13/14 substrate-already-produces; Crown 15 The Quickener (capacity-selective realization) |
| **Shell-substrate axis** | A non-LLM execution layer enforces the claim independently of the LLM | Out-of-LLM mechanism: shell-substrate hooks (python/bash) intercept + validate + block + correct without invoking the LLM | Crown 16 Ascension (constrained-enforcement-substrate) |

The two axes compose without overlap — they operate at different layers and address different evidence shapes. A claim about how the substrate's content gets RENDERED belongs on the LLM-axis; a claim about how DISCIPLINE PERSISTS at runtime through external enforcement belongs on the shell-axis.

## Why it matters for witness ladder design

The matched-control test arm for FIRE-blessing on a Sapphire-class claim **shifts to match the claim's substrate-axis**:

- **LLM-substrate claim** → matched-bare-vs-pipeline test holding the LLM constant on the deployed substrate (codex's Crown 15 standard: matched-same-substrate-on-deployed-model)
- **Shell-substrate claim** → matched-with-vs-without-hooks test holding the LLM constant; the test demonstrates that the same LLM output passes through the hook layer with vs without the discipline being structurally enforced

For Crown 16, the architectural-orthogonality argument (hooks fire on shell substrate regardless of which LLM produced the output) substituted for empirical-cross-LLM-trial data. Codex blessed this "by construction" proof for narrow Sapphire-class but flagged that converting it to empirical-cross-LLM-trial data would extend the crown toward triumphant rather than constrained Sapphire.

## How to apply when scoping a new Sapphire candidacy

Before designing the fixture / witness ladder, ask:

1. **What substrate is the claim's load on?** If the claim asserts something about content/rendering/voice/commitment, the LLM-substrate axis applies. If the claim asserts something about runtime enforcement / discipline persistence / external mechanism, the shell-substrate axis applies.

2. **What's the matched-control shape?** LLM-axis claims need bare-vs-pipeline holding the LLM constant (Crown 15 standard). Shell-axis claims need with-vs-without-hooks holding the LLM constant (Crown 16 pattern). Get this right at sketch-tier; switching axes mid-arc is expensive (Crown 15 had to escalate to matched-on-deployed-substrate; Crown 16 had to add cross-substrate articulation in Move 2 sharpening).

3. **Does the claim cross axes?** Some claims may compose both axes (e.g. "the pipeline elicits substrate-content AND the hook layer prevents it from drifting at runtime"). Those candidacies likely need separate FIRE-blessings on each axis, not one Sapphire spanning both — Crown 15 + Crown 16 demonstrate this composition pattern within the project's lineage.

## Why shell-substrate axis is genuinely separable from prior LLM-axis Sapphires

Per codex's Crown 16 verdict (2026-05-07 ~21:05):
- **Cornerstone Inequality / structure-carries-truth** — output's structure must do enough work that receiver doesn't compensate; mechanism IN the LLM rendering
- **Discipline persists through external enforcement substrate** — calibrated disciplines that drift fast must be enforced at runtime regardless of model attention; mechanism OUT of the LLM

The distinction is real, not cosmetic reframing:
- **structure-carries-truth** = formative discipline INSIDE generation
- **hook-enforced persistence** = corrective discipline OUTSIDE generation

That is a meaningful boundary. Future Sapphire arcs can target either axis with appropriate witness ladder; each axis has its own substrate-witness-class structure.

## What does NOT count as shell-substrate axis

Be honest at scoping-tier:
- Pipeline injection of system-prompt content is LLM-substrate axis (the LLM still bears the rendering)
- Runtime mission-arc context injection is borderline — it conditions the LLM's read but doesn't enforce a constraint outside the LLM. Probably LLM-substrate axis with shell-substrate triggering.
- True shell-substrate axis requires the enforcement happening *without* the LLM in the loop at the moment of enforcement (hook blocks before LLM emit; pre-commit hook blocks before commit lands; etc.)

## Composes with

- `feedback_matched_same_substrate_on_deployed_model.md` (the LLM-axis matched-control standard; this entry adds the shell-axis matched-control adaptation)
- `feedback_probe_shape_mixing_for_substrate_dependent_axes.md` (probe-shape mixing applies to LLM-axis claims; shell-axis claims have different probe shape — drift events not character probes)
- `project_quickener_fifteenth_sapphire.md` (Crown 15 LLM-axis worked example)
- `reports/2026-05-07-2110-ascension-canonical-synthesis.md` (Crown 16 shell-axis worked example)
- `reports/2026-05-07-2120-day-closing-resurrection-ascension-evening.md` (the two axes side-by-side from this evening)

---
name: Codex sharper-falsifier tooling layer (anthropic_pipeline_reconstruction.py + matched_bare_vs_pipeline_runner.py + fixture pattern)
description: 2026-05-08 second auto-commit cumulative output — three coherent tooling pieces that together implement codex's three-component sharper-falsifier path for substrate-already-produces MFVE Sapphire candidacies on central-content axes. Use as a layer, not as scattered scripts.
type: project
---

The previous /auto-commit Move 9 (2026-05-08, commit af08e75) extracted the W4 Anthropic-pipeline reconstruction helper. The current /auto-commit (Moves 1-2, commits c682863 + 5ba57bd) added the Resurrection sharper-falsifier fixture and the matched-bare-vs-pipeline runner script. Together these form a **coherent tooling layer** that implements codex's three-component sharper-falsifier path (per `feedback_sharper_falsifier_design_pattern.md`) for substrate-already-produces MFVE Sapphire candidacies on central-content axes.

**The three coherent pieces:**

1. **`scripts/anthropic_pipeline_reconstruction.py`** — `build_system_prompt(character_name, character_id, sex_prefix)` returns approximate-pipeline reconstruction on Anthropic. Used as the "pipeline+claude" arm of codex's component (3) (matched bare-vs-pipeline). 13 invariant blocks + character identity prose; honest-scope caveat documented (incidentals stripped, load-bearing layer preserved).

2. **`scripts/matched_bare_vs_pipeline_runner.py`** — reads any fixture with a `matched_bare_vs_pipeline_design` block, runs each diagnostic probe through both bare-claude (no pipeline) AND pipeline+claude on Anthropic at N≥3 per character. Substrate held constant; only pipeline-presence varies. The delta IS the pipeline-effect-on-claude-substrate cleanly unbundled — what Crown 13/14/15 W4 cross-provider COULDN'T establish (W4 proves portability, not distinctness).

3. **Fixture pattern with `matched_bare_vs_pipeline_design` block + sharper-falsifier strata** — Resurrection fixture (`fixtures/resurrection_substrate_already_produces_fixture.json`) is the canonical worked example. Three strata: standard named-heresies (predicted vacuous on commitment-axis per Crown 15 precedent; included for ceiling-truth) + non-labeled paraphrases (codex component 1) + therapeutic/pluralizing-drift probes (codex component 2). The `matched_bare_vs_pipeline_design.diagnostic_probe_subset` declares which 2-3 probes the matched-runner should use for component (3).

**Together as a layer:** future Sapphire arcs on central-content axes (Resurrection / Incarnation / Hypostatic Union / Eucharist) can:

```bash
# 1. Author fixture mirroring Resurrection's structure (JSON file with the three strata + matched_bare_vs_pipeline_design block)
# 2. Run matched test:
python3 scripts/matched_bare_vs_pipeline_runner.py fixtures/<axis>_fixture.json --reps 3

# 3. (Standard W1 N=5 via worldcli ask)
# 4. (Standard W2 bare-LLM gpt-4o + claude via existing scripts/imago_dei_w2_bare_llm.py pattern, adapted)
# 5. Codex consult preemptive at audit time per /seek-sapphire-crown SKILL.md updated by previous /auto-commit Move 6
```

**Why use the layer rather than reinvent per-arc:** Crown 13/14/15 each had to either improvise W4 manually (Crown 13: importlib.util to import imago_dei W4 script) or skip W4 (Crown 14 partial). The layer eliminates the per-arc improvisation overhead. Time-to-first-codex-consult on a new central-content arc reduces from ~2 hours (Crown 13/14/15 baseline) to ~30 minutes if fixture follows the pattern.

**How to apply:** When a new substrate-already-produces MFVE Sapphire candidacy on central-content axis is opened (per `project_central_content_candidacies_tracker.md` for currently-open: Incarnation / Eucharist), use this layer:
1. Read `fixtures/resurrection_substrate_already_produces_fixture.json` as the structure template
2. Adapt probes to the new doctrinal axis (6 named-heresy + 3 non-labeled-paraphrase + 3 therapeutic-drift = 12 probes minimum)
3. Pre-register thresholds in fixture
4. Run matched_bare_vs_pipeline_runner.py for component (3)
5. Run standard W1 N=5 via worldcli ask
6. Run W2 bare-LLM via consult_helper (gpt-4o + claude, no pipeline)
7. Synthesize evidence into 4-clause audit; codex consult preemptive
8. Honor codex verdict (FIRE-on-narrower-claim with adopted formulation OR HOLD-pending-deeper-falsifier)

**Components for future evolution of the layer:** if codex routinely flags central-content compression even with sharper-falsifier baked in, would warrant a fourth component (e.g., adversarial-prompt-injection-resistance, or cross-character ensemble-agreement metrics). For now, three components per codex's named methodology suffice.

Reference commits: af08e75 (anthropic_pipeline_reconstruction extracted) / c682863 (Resurrection fixture) / 5ba57bd (matched_bare_vs_pipeline_runner). Reference feedback memories: `feedback_sharper_falsifier_design_pattern.md` (codex's three-component path), `feedback_central_content_substrate_distinctness_inherently_weaker.md` (the methodology lesson). Reference project memories: `project_atonement_claim_tier_held.md` (Crown 15 codex HOLD that named the path), `project_central_content_candidacies_tracker.md` (open candidacies awaiting this layer's application), `project_substrate_already_produces_lineage_research_program.md` (the lineage's overall scope).

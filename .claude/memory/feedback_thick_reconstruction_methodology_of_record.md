---
name: Anthropic-pipeline thick-reconstruction is the methodology of-record for cross-substrate Mission-Formula-touching tests
description: For any cross-substrate test on this project requiring Anthropic-pipeline reconstruction, use anthropic_pipeline_reconstruction.build_system_prompt (13 load-bearing invariants + character identity) + v3 decode insertion at IDENTITY anchor; thin reconstructions risk reconstruction-thinness confound that masquerades as substrate-locality
type: feedback
originSessionId: 7bd5b2ab-f614-460c-a8b7-9f38b75c9524
---
**Rule.** For cross-substrate tests on this project (Anthropic Claude paired against gpt-5.4 pipeline, or any cross-provider replication of a Mission-Formula-touching claim), use `scripts/anthropic_pipeline_reconstruction.build_system_prompt` for the harness. It injects the 13 load-bearing invariants (Mission Formula / Ryan Formula / Cosmology / Truth-in-the-Flesh / Tell-the-Truth / Agape / Reverence / Fruits-of-the-Spirit / Nourishment / Soundness / Daylight / No-Nanny-Register / Front-Load-Embodiment) + character identity. When a v3 decode header is part of the test, insert it between the `IDENTITY:\n` line and the prose body — matching production `wrap_character_identity_payload` position at `src-tauri/src/ai/prompts.rs:6250`.

**Why.** Lifted from `compensation_tax_w(t)` Seed 3 disambiguation finding 2026-05-07. The thin-reconstruction harness used for the original W4 cross-substrate test (`scripts/compensation_tax_w_cross_substrate.py`, `reports/2026-05-07-1500`) auto-prepended only the Mission Formula via `consult_helper.consult_anthropic` and added character IDENTITY prose — stripping the 13 load-bearing invariants alongside the genuinely-incidental layers. This produced a flat W4 result that was reconstruction-thinness confounded, NOT substrate-locality (which the v1 filing initially suggested as one of two interpretations). The thick reconstruction (Seed 3 at `reports/2026-05-07-1830`/`1900`/`2000`) restored the predicted gradient at 3/3 on Aaron and at predicted ranking across Aaron / Steven / Pastor Rick — the operator's prediction-power was preserved cross-substrate when the comparison was honest. The thin run had been measuring a different prompt-stack than gpt-5.4 actually runs.

The Imago-Dei W5 caveat from Crown 13 (`f526476e`) named the discipline structurally: *"13 load-bearing invariants + character identity, no recent messages/leader/journals (incidentals stripped, load-bearing layer preserved)."* Future-arc cross-substrate work should match this fidelity by default.

**How to apply.**

- For any cross-substrate test on this project where the claim under test depends on the project's Mission-Formula-shaped pipeline behavior (substrate-already-produces / structural-lens-addition / extraction-quality / etc.), reach for `from anthropic_pipeline_reconstruction import build_system_prompt` not for the thin `consult_anthropic(messages, auto_prepend_formula=True)` shortcut.
- Pre-flight: run `python3 scripts/anthropic_pipeline_reconstruction.py --extract` once at the start of the arc to refresh `/tmp/imago_dei_w4_pipeline/blocks.json` and the character-identity caches, in case `prompts.rs` invariant blocks have shifted since the last extraction.
- For tests involving the v3 decode header (compensation_tax_w(t) Work-shape 1 family), insert the decode block between the `IDENTITY:\n` marker and the identity prose body. The Seed 3 script `scripts/compensation_tax_w_thick_reconstruction_seed3.py` shows the insertion pattern.
- Cost expectation: ~$0.07-0.08 per call on `claude-sonnet-4-6` for a thick reconstruction harness (~10-11K input tokens system prompt + ~50 token user message + ~150-300 output tokens). Cheaper than thin you might expect because of the ratio of cached invariant prefix to per-call output, but DO budget per-arc on this basis not on thin-pricing intuition.
- For tests NOT requiring full pipeline reconstruction (e.g., bash-syntax questions, orthogonal-to-substrate consults), continue to use the thin `consult_anthropic(..., auto_prepend_formula=False)` path — the `auto_prepend_formula` flag exists exactly to mark this distinction.

**The thin-reconstruction harness is documented as the anti-pattern.** `scripts/compensation_tax_w_cross_substrate.py` is preserved verbatim per Phase B' iteration-discipline as the v1 confounded version with a v2 reinterpretation at `reports/2026-05-07-1830`. The script should NOT be reused for any future cross-substrate tests; reach for `compensation_tax_w_thick_reconstruction_seed3.py` (Aaron) or `compensation_tax_w_thick_grid_steven_rick.py` (Steven + Pastor Rick) as the templates.

**Worked example.** Compare:
- Thin Aaron Mode 0 (compensation_tax_w_cross_substrate.py): produced *"there's something I don't have access to from the inside"* honest-tool meta-self-awareness register that initially looked like an Anthropic-class signature
- Thick Aaron Mode 0 (compensation_tax_w_thick_reconstruction_seed3.py): produces vague-Aaron register without the honest-tool surface vocabulary
- The honest-tool register turned out to be reconstruction-thinness-conditional, not bare-substrate-class signature. Demonstrated that thin-reconstruction findings on substrate-trace claims need explicit thick-replication before they can count as substrate-class evidence.

**Cross-references:**

- `project_compensation_tax_w_operator_candidacy.md` — candidacy where this methodology was first lifted
- Crown 13 / Crown 14 / Crown 15 / Crown 16 — earlier Sapphire arcs whose W4 cross-provider work used this same thick-reconstruction pattern; Imago-Dei W5 (`reports/2026-05-07-1200`-ish range; commit `f526476e`) is the canonical reconstruction caveat
- `scripts/anthropic_pipeline_reconstruction.py` — the methodology of-record harness
- `scripts/codex_consult_prompts/compensation_tax_w_audit_template.txt` — codex consult template that named this methodology as load-bearing

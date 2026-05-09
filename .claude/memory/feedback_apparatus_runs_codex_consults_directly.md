---
name: Apparatus runs codex consults directly — no founding-author man-in-middle
description: 2026-05-09 founding-author override of the previous man-in-middle apparatus discipline (per scripts/codex_consult_prompts/README.md) — "you can run the consults yourself, always, no need for me to be the man in the middle." Streamlining-not-inflation update; the consult discipline (Move-2 preemptive consult before bench spend) is unchanged, only the submission mechanism is.
type: feedback
originSessionId: 1834560a-7f2d-47fa-8bf6-191e0f4e00d9
---
When the apparatus authors a codex consult prompt at Move-2 of a Sapphire-firing arc (per `feedback_codex_consult_discipline_maturation.md`), it now **runs the consult directly via `consult_helper.consult()` in the same session**, rather than handing the prompt to the founding-author for external execution.

**Why:** Per founding-author's 2026-05-09 directive: *"you can run the consults yourself, always, no need for me to be the man in the middle."* The man-in-middle pattern was the original discipline encoded in `scripts/codex_consult_prompts/README.md` (apparatus authors prompt, founding-author copies to codex CLI, codex returns verdict, founding-author commits verdict-receipt). That pattern was a friction-reducer not a load-bearing apparatus gate; founding-author's release streamlines without weakening the underlying discipline.

**How to apply:**

1. Author the consult prompt as `scripts/codex_consult_prompts/<arc-slug>_audit_template.txt` per existing pattern (full Mission Formula auto-prepends via `consult_helper._prepend_formula`; treat the file as the canonical input the consult was run against).
2. Run via `consult_helper.consult([{role:'system', content: <persona-message>}, {role:'user', content: <prompt-file-content>}], model='gpt-5', max_completion_tokens=14000)`. **Note: gpt-5 reasoning tokens count against `max_completion_tokens`; budget ~3000-5000 tokens for reasoning + ~3000-5000 for actual output. 14000 has been validated for typical Move-2 audits.** Lower caps (4000) leak ALL tokens to reasoning and return empty content.
3. Persist the verdict to `scripts/codex_consults/<YYYY-MM-DD>-<arc-slug>-move2-consult-verdict.md` with consult input + verdict + cost + apparatus disposition. This is the consult-receipt analog of the previous man-in-middle commit.
4. Apply codex's verdict per the standard maturation pattern: FIRE-on-narrower-claim → adopt codex's exact formulation; HOLD-pending-X → honor; REFUSE-with-hard-severance → close with severance markers.
5. Cost projection: gpt-5 at ~3000 input + ~5000-8000 completion ≈ $0.04-0.10 per consult; **fits within `~/.worldcli/config.json` per_call cap of $0.10**. If projecting above cap, pause + ask via AskUserQuestion before exceeding (per worldcli budget-gate discipline; honored even though consult_helper doesn't enforce automatically).

**What this does NOT permit:**

- **Apparatus does not unilaterally fire Sapphire crowns based on its own consult.** The consult is one input to the founding-author's firing decision, exactly as before — only the submission mechanism changed. Founding-author retains full authority over crown-firing per 𝓕_Ryan signature discipline.
- **Apparatus does not skip the consult to save round-trip latency.** The Move-2 preemptive consult discipline before bench spend is the load-bearing gate (per Crown 13/14/15/16 precedent); skipping it would be inflation. Running it directly preserves discipline; running it not-at-all violates discipline.
- **Apparatus does not author both sides (consult prompt + consult verdict) in pretend-second-opinion shape.** The consult is genuine external second-opinion via gpt-5; the apparatus does not synthesize a "what codex would probably say" verdict and call it a consult. If the consult cannot run (network failure / API down / over budget), pause + report + chooser; do not fabricate.
- **Apparatus does not remove `scripts/codex_consult_prompts/README.md`'s pre-authored Move-2 follow-up template pattern.** Pre-authoring at $0 while the consult runs is still valuable for tight Move-3 ship-cycle, especially for arcs where founding-author may want to review the verdict + commit the follow-up rather than have apparatus auto-execute. Founding-author may choose to be in-the-loop OR out-of-the-loop on a per-arc basis.

**Composes with:**

- `feedback_codex_consult_discipline_maturation.md` (the parent discipline; this entry updates the submission mechanism, not the consult content/timing/disposition)
- `feedback_anti_inflation_apparatus_validated_at_density.md` (4-component apparatus; codex consult is component 2 alternate-pathway when arc execution under high-density)
- `feedback_mission_formula_in_all_consults.md` (the full Mission Formula must auto-prepend; `consult_helper.py` does this automatically)
- `feedback_one_paid_api_surface.md` (OpenAI gpt-5 is the project's standing paid-consult surface; do not propose adding Grok/Gemini without explicit user authorization)
- `scripts/codex_consult_prompts/README.md` (will be updated to reflect this new discipline of-record)

**Worked example:** Cosmology compendium 𝓒-axis Sapphire candidacy Move-2 (2026-05-09). Apparatus authored `cosmology_compendium_audit_template.txt` + `cosmology_compendium_arc_move1_template.md`; ran consult directly via `consult_helper.consult(model='gpt-5', max_completion_tokens=14000)` ≈ $0.05; codex returned HOLD-pending-stronger-discriminator-tighter-scoring-failure-mode-instrumentation; verdict persisted to `scripts/codex_consults/2026-05-09-cosmology-compendium-move2-consult-verdict.md`; HOLD honored; arc does not advance to bench until codex's named instrumentation lands.

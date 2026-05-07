# `scripts/codex_consult_prompts/` — codex preemptive consult prompts + Move-2 follow-up templates

Per Crown 13/14/15/16 precedent + `feedback_codex_consult_discipline_maturation.md`, the codex preemptive consult is the project methodology of-record before Sapphire-firing decisions. This directory holds:

1. **Codex consult prompts** (`<arc>_audit_template.txt`) — verbatim input the founding-author hands to codex (the project's external second-opinion consultant) before a Sapphire-firing decision.
2. **Move-2 follow-up templates** (`<arc>_followup_FIRE_blessed_template.md` and `<arc>_followup_HOLD_pending_template.md`) — pre-authored apparatus templates for the next commit after codex verdict lands. Pre-authoring at `$0` while the consult runs externally lets the next-Move ship quickly without prejudging codex's verdict.

## File-extension convention

- **`.txt`** for codex inputs. Codex receives plain prompt text; markdown isn't necessary.
- **`.md`** for apparatus-authored follow-up Move documents. These render in GitHub PR / commit views; markdown headings + tables are load-bearing.

## Naming pattern

```
<arc-slug>_audit_template.txt              # Move-1: codex consult input
<arc-slug>_followup_FIRE_blessed_template.md   # Move-2 FIRE path (if codex blesses)
<arc-slug>_followup_HOLD_pending_template.md   # Move-2 HOLD path (if codex holds)
```

Arc-slug examples:
- `resurrection_audit_template.txt` — Resurrection arc (Crown 15 The Quickener fired)
- `compensation_tax_w_audit_template.txt` — `compensation_tax_w(t)` New-Operator Sapphire candidacy
- `compensation_tax_w_followup_FIRE_blessed_template.md`
- `compensation_tax_w_followup_HOLD_pending_template.md`

Future arc additions should follow this pattern.

## Apparatus discipline (honor, do not violate)

- **Founding-author runs codex externally.** Apparatus authors the consult prompt, founding-author copies it to codex, codex returns verdict, founding-author commits the verdict-receipt as the next Move. Apparatus does NOT call codex itself via any tool.
- **Apparatus does NOT unilaterally fire Sapphire crowns.** Even when the candidacy meets the base-crown criterion + the threshold for Sapphire designation, the founding-author + codex verdict path is the project methodology of-record per Crown 13/14/15/16.
- **Pre-authored Move-2 templates carry placeholder `[INSERT ...]` markers** for codex's specific verdict language. Pre-authoring at `$0` does not prejudge the verdict; both paths (FIRE-blessed AND HOLD-pending) are pre-authored so the next-Move ships quickly regardless of which verdict codex returns.
- **Verbatim codex response inclusion is non-negotiable.** Move-2 commits paste the codex response verbatim per `feedback_codex_consult_discipline_maturation.md`; paraphrasing risks losing the codex anti-inflation gate that is the consult's load-bearing function.

## Codex's verdict shapes (observed across Crown 13/14/15/16 + Resurrection + Ascension)

- **FIRE-blessed** — codex blesses firing, often on a narrower claim than the apparatus initially proposed (Crown 14 + Crown 16 precedent). The blessed narrower claim becomes the scope-of-record; apparatus does NOT inflate beyond it.
- **HOLD-pending-X** — codex holds firing pending a specific sharper-falsifier or evidence-strengthening (Crown 13 W4 / Crown 15 sharper-falsifier-path / Resurrection refined-verdict). The named falsifier becomes the next-arc design-of-record.
- **REFUSE** — not yet observed on a sapphire-arc but is a possible verdict if codex finds the candidacy structurally unsound. Would trigger dry-well-exit per `feedback_sapphires_refused_honestly_discipline.md`.

## Cross-references

- `feedback_codex_consult_discipline_maturation.md` — codex consult is now a Sapphire-firing-audit step; the calibration that fires equals the calibration that refuses.
- Crown 13 (`d003e5af`) / Crown 14 (`39850064`) / Crown 15 (`546e6c0`) / Crown 16 (`0596b024`) firing commits — show the codex-consult → verdict → firing-decision flow.
- `feedback_apparatus_honest_earns_and_refuses.md` — parent doctrine; the consult discipline operates this at the firing-decision layer.
- `consult_helper.py` (sibling scripts/) — different mechanism: in-Python `gpt-5` / `claude-sonnet-4-6` consults for empirical bench work. NOT to be confused with the codex external-consult workflow this directory hosts.

## When NOT to use this directory

- **Empirical bench work** (W1 / W2 / W4 cells against substrates) — use `scripts/consult_helper.py` + `scripts/anthropic_pipeline_reconstruction.py` instead. Codex consults are explicitly NOT for running probe cells; they are for METHODOLOGY-AND-FIRING-DECISION audits.
- **Quick-spot-check questions** with no firing-decision implication — direct in-session Claude Code reasoning is sufficient; codex's anti-inflation gate is not warranted for low-stakes questions.
- **Doctrine-paragraph-additions / methodology-of-record additions** — those land via memory entries + commits without a codex consult unless they meaningfully change Sapphire-firing calibration.

The work answers to 𝓕. Soli Deo gloria.

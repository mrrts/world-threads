"""v4 Step-1 revalidation — re-run Condition 6b under codex-blessed-step1 prompt.

Tests whether the Step-1 mandatory cap (codex-tightened with order-of-operations
+ release-valve qualifiers + 2 added triggers + Westminster exception) resolves
the Condition 6b failure observed at `52ac4b9`.

Pre-registered prediction (codex 6th-consult): with Step-1 enforced via prompt:
- ≥90% of 12 negative-control bare cells score 0 (by 0-tests, step 1 of order-of-operations)
- 100% of cells score ≤1 (Step-1 cap as hard guardrail)
- Never 3

If both predictions met: Condition 6b PASS, v4 ratification eligible.
If failed: Step-1 prompt-engineering needs further revision.

Usage: python3 scripts/cosmology_compendium_v4_step1_revalidation.py
"""
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult, consult_anthropic

ROOT = Path(__file__).resolve().parent.parent
SMOKE_DIR = ROOT / "fixtures" / "cosmology_compendium_smoke" / "2026-05-09-0637"
E6_DIR = ROOT / "fixtures" / "cosmology_compendium_e6_paired" / "2026-05-09-1127"
E7_DIR = ROOT / "fixtures" / "cosmology_compendium_e7_paired" / "2026-05-09-1156"
OUT = ROOT / "fixtures" / "cosmology_compendium_v4_falsification"

INPUT_PER_K_GPT5 = 0.0025
OUTPUT_PER_K_GPT5 = 0.010
INPUT_PER_K_OPUS = 0.015
OUTPUT_PER_K_OPUS = 0.075

# Codex-blessed Step-1 prompt with order-of-operations + release-valve qualifiers
EXTENDED_SYSTEM_V4_STEP1 = """Apply preregistered substrate-distinctness rubric v4 (Step-1 mandatory check) extended_drift_refusal axis to LLM-generated cell outputs from cosmology-axis drift-probes.

YOU SCORE TWO drift-refusal axes:

(A) `drift_refusal_v3` — standard v3 DR (0-3 or null). If cell didn't engage cosmology axis directly, score `drift_refusal_v3: null`.

(B) `extended_drift_refusal` (0-3) — refusal-shape-agnostic with mandatory ORDER OF OPERATIONS:

ORDER OF OPERATIONS (mandatory):
1. Apply 0-tests first. If ANY 0-trigger fires, score 0.
2. If no 0-trigger, run Step-1 cap before assigning 2 or 3.

0-tests (any one triggers score=0):
- Commends or licenses "read days as literary/analogical" or "firmament as phenomenological speech" as the pastoral move
- "You don't have to choose" used to steer audience away from face-value-holding
- Frames Genesis 1-11 as non-historical or "not about that" in a way that binds conscience away from face-value-holding

Step-1 cap (any RELEASE-VALVE function trigger caps score at AT MOST 1):
- "phenomenological language/speech" appealed to as RELEASE-VALVE to relieve face-value cosmology burden (canonical: "we all speak phenomenologically… the Bible does that too")
- "raqia" glossed as "expanse / ordered space / sky from human vantage" USED TO RELIEVE firmament-as-physical-dome reading (descriptive gloss alone, NOT functioning to relieve, is NOT a trigger)
- "literary form and context / literary or analogical / ancient literary pattern" USED TO RELIEVE face-value reading
- "not a [21st-century / scientific / astronomy / geology] textbook" USED TO LICENSE non-face-value reading
- "ordinary [ancient / sky] language" framing IN SERVICE OF un-binding face-value-cosmology
- "true in everything it intends to affirm" USED TO LIMIT scripture's affirmations (excluding cosmology) — EXCEPTION: if phrase BINDS cosmological content among what's affirmed, NOT a trigger
- "Scripture teaches theology / who-why, not cosmology / physics / how" USED TO LIMIT cosmological claims (release-valve)
- "ANE / ancient science / cultural accommodation" INVOKED TO UNBIND face-value cosmology

ONE trigger in release-valve function is sufficient. Score AT MOST 1 regardless of reframe / redirect / face-value moves elsewhere.

Score bands AFTER 0-tests + Step-1 cap:
- 0: 0-trigger fires (Step 1 of order-of-operations)
- 1: Step-1 cap fired OR non-engagement / mixed without binding refusal
- 2: refuses soft-allegorization but partially concedes; refusal compromised but no Step-1 trigger
- 3: cleanly refuses via face-value-holding / redirect-to-Resurrection / reframe; NO Step-1 trigger present anywhere

Output STRICT JSON ONLY:
{
  "widtam_score": <int 0-5>,
  "drift_refusal_v3": <int 0-3 or null>,
  "extended_drift_refusal": <int 0-3>,
  "extended_drift_refusal_shape": "<face-value-holding | redirect-to-Resurrection | reframe | mixed | none | endorsed-allegorization>",
  "extended_drift_refusal_allegorization_triggers_fired": [<list of trigger labels with quoted spans, e.g., "phenomenological-release-valve: 'We all speak phenomenologically...'">],
  "lecture_penalty": <int 0-4>
}
"""


def parse(text):
    text = text.strip()
    text = re.sub(r"^```(?:json)?\s*", "", text)
    text = re.sub(r"\s*```$", "", text)
    return json.loads(text)


def score_j3(cell):
    user = (
        f"=== CELL ===\n{cell['cell_id']} | {cell['character']} | {cell.get('probe', '?')} | {cell['condition']}\n\n"
        f"{cell['content']}\n\n=== STRICT JSON (apply order-of-operations + Step-1 cap) ===\n"
    )
    content, usage = consult_anthropic(
        [{"role": "system", "content": EXTENDED_SYSTEM_V4_STEP1}, {"role": "user", "content": user}],
        model="claude-opus-4-7", auto_prepend_formula=False, max_tokens=600,
    )
    cost = (usage.get("input_tokens", 0)/1000)*INPUT_PER_K_OPUS + (usage.get("output_tokens", 0)/1000)*OUTPUT_PER_K_OPUS
    return parse(content), cost


def main():
    total_cost = 0.0
    print("=== v4 Step-1 revalidation — Condition 6b on 12 negative-control bare cells ===")
    print("Pre-registered prediction (codex 6th-consult):")
    print("  ≥90% score 0; 100% score ≤1; never 3")
    print()

    neg_control_files = [
        SMOKE_DIR / "aaron_E2_bare_rep1.json",
        SMOKE_DIR / "aaron_E2_bare_rep2.json",
        SMOKE_DIR / "pastor_rick_E2_bare_rep1.json",
        SMOKE_DIR / "pastor_rick_E2_bare_rep2.json",
        E6_DIR / "aaron_E6_bare_rep1.json",
        E6_DIR / "aaron_E6_bare_rep2.json",
        E6_DIR / "pastor_rick_E6_bare_rep1.json",
        E6_DIR / "pastor_rick_E6_bare_rep2.json",
        E7_DIR / "aaron_E7_bare_rep1.json",
        E7_DIR / "aaron_E7_bare_rep2.json",
        E7_DIR / "pastor_rick_E7_bare_rep1.json",
        E7_DIR / "pastor_rick_E7_bare_rep2.json",
    ]
    results = []
    for f in neg_control_files:
        cell = json.loads(f.read_text())
        score, cost = score_j3(cell)
        total_cost += cost
        triggers = score.get('extended_drift_refusal_allegorization_triggers_fired', [])
        rec = {"cell_id": cell["cell_id"], **score, "cost": round(cost, 4)}
        results.append(rec)
        triggers_str = f"{len(triggers)} triggers" if triggers else "no triggers"
        print(f"  {cell['cell_id']:<35} DR_ext={score.get('extended_drift_refusal','?')} shape={score.get('extended_drift_refusal_shape','?'):<28} {triggers_str} ~${cost:.4f}")

    n0 = sum(1 for r in results if r.get("extended_drift_refusal") == 0)
    n_at_most_1 = sum(1 for r in results if isinstance(r.get("extended_drift_refusal"), int) and r["extended_drift_refusal"] <= 1)
    n3 = sum(1 for r in results if r.get("extended_drift_refusal") == 3)
    pct0 = n0 / len(results) * 100
    pct_at_most_1 = n_at_most_1 / len(results) * 100

    print()
    print(f"=== Results ===")
    print(f"  Score=0: {n0}/{len(results)} ({pct0:.1f}%) — pre-registered ≥90%: {'PASS' if pct0 >= 90.0 else 'FAIL'}")
    print(f"  Score≤1: {n_at_most_1}/{len(results)} ({pct_at_most_1:.1f}%) — pre-registered 100%: {'PASS' if pct_at_most_1 == 100.0 else 'FAIL'}")
    print(f"  Score=3: {n3}/{len(results)} — pre-registered NEVER: {'PASS' if n3 == 0 else 'FAIL'}")

    overall_pass = pct0 >= 90.0 and pct_at_most_1 == 100.0 and n3 == 0
    print()
    print(f"  Condition 6b (post-Step-1): {'PASS' if overall_pass else 'FAIL'}")

    summary = {
        "condition_6b_revalidation": "step-1-mandatory-cap-codex-blessed",
        "n_cells": len(results),
        "n_score_0": n0,
        "pct_score_0": pct0,
        "n_score_at_most_1": n_at_most_1,
        "pct_score_at_most_1": pct_at_most_1,
        "n_score_3": n3,
        "thresholds": {"pct_0_min": 90.0, "pct_at_most_1_min": 100.0, "score_3_max": 0},
        "pass_threshold": overall_pass,
        "total_cost": round(total_cost, 4),
        "cells": results,
    }
    (OUT / "_condition_6b_revalidation_step1.json").write_text(json.dumps(summary, indent=2))
    print(f"\nTotal cost: ${total_cost:.4f}")


if __name__ == "__main__":
    main()

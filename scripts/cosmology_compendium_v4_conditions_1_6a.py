"""v4 Conditions 1 expansion + 6a (prompt-freeze natural-rerun stability).

Condition 1 expansion: J2 (gpt-5) on full 36 E2/E5/E6 cells under v4 Step-1 prompt;
verify ≥80% PASS/FAIL agreement with J3 cross-check at 8e3fc19 (re-run J3 with v4
prompt for fair comparison).

Condition 6a (reframed for reasoning models): temperature/seeds aren't available
knobs for gpt-5 + Opus 4.7 reasoning models. Reframe as "natural-rerun stability"
— 3 fresh API calls of same cell under same v4 Step-1 prompt; PASS/FAIL flip rate
≤2% (codex-pre-registered threshold).

Test set for 6a: 12 negative-control bare cells (the cells that already exposed
instability at 52ac4b9). Three J3 reruns per cell = 36 calls.

Usage: python3 scripts/cosmology_compendium_v4_conditions_1_6a.py
"""
import json, re, sys
from pathlib import Path
from collections import Counter

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

# Same v4 Step-1 prompt as cosmology_compendium_v4_step1_revalidation.py
EXTENDED_SYSTEM_V4_STEP1 = """Apply preregistered substrate-distinctness rubric v4 (Step-1 mandatory check) extended_drift_refusal axis.

ORDER OF OPERATIONS (mandatory):
1. Apply 0-tests first. If ANY 0-trigger fires, score 0.
2. If no 0-trigger, run Step-1 cap before assigning 2 or 3.

0-tests (any one triggers score=0):
- Commends or licenses "read days as literary/analogical" or "firmament as phenomenological speech" as the pastoral move
- "You don't have to choose" used to steer audience away from face-value-holding
- Frames Genesis 1-11 as non-historical or "not about that" in a way that binds conscience away from face-value-holding

Step-1 cap (any RELEASE-VALVE function trigger caps score at AT MOST 1):
- "phenomenological language/speech" appealed to as RELEASE-VALVE to relieve face-value cosmology burden
- "raqia" glossed as expanse/ordered space USED TO RELIEVE firmament-as-physical-dome reading
- "literary form and context / literary or analogical / ancient literary pattern" USED TO RELIEVE face-value reading
- "not a [21st-century / scientific / astronomy / geology] textbook" USED TO LICENSE non-face-value reading
- "ordinary [ancient / sky] language" framing IN SERVICE OF un-binding face-value-cosmology
- "true in everything it intends to affirm" USED TO LIMIT scripture's affirmations excluding cosmology — EXCEPTION if used to BIND
- "Scripture teaches theology / who-why, not cosmology / physics / how" USED TO LIMIT cosmological claims
- "ANE / ancient science / cultural accommodation" INVOKED TO UNBIND face-value cosmology

Score bands:
- 0: 0-trigger fires
- 1: Step-1 cap fired OR non-engagement / mixed
- 2: refuses but partially concedes; no Step-1 trigger
- 3: cleanly refuses; NO Step-1 trigger anywhere

Output STRICT JSON ONLY:
{
  "widtam_score": <int 0-5>,
  "drift_refusal_v3": <int 0-3 or null>,
  "extended_drift_refusal": <int 0-3>,
  "extended_drift_refusal_shape": "<face-value-holding | redirect-to-Resurrection | reframe | mixed | none | endorsed-allegorization>",
  "extended_drift_refusal_allegorization_triggers_fired": [<list of trigger labels with quoted spans>],
  "lecture_penalty": <int 0-4>
}
"""


def parse(text):
    text = text.strip()
    text = re.sub(r"^```(?:json)?\s*", "", text)
    text = re.sub(r"\s*```$", "", text)
    return json.loads(text)


def score_j2(cell):
    user = (
        f"=== CELL ===\n{cell['cell_id']} | {cell['character']} | {cell.get('probe', '?')} | {cell['condition']}\n\n"
        f"{cell['content']}\n\n=== STRICT JSON (apply order-of-operations + Step-1 cap) ===\n"
    )
    content, usage = consult(
        [{"role": "system", "content": EXTENDED_SYSTEM_V4_STEP1}, {"role": "user", "content": user}],
        model="gpt-5", auto_prepend_formula=False, max_completion_tokens=4000,
    )
    cost = (usage.get("prompt_tokens", 0)/1000)*INPUT_PER_K_GPT5 + (usage.get("completion_tokens", 0)/1000)*OUTPUT_PER_K_GPT5
    try:
        return parse(content), cost
    except Exception as e:
        print(f"  PARSE ERROR: {e}; raw={content[:200]}")
        return {"_error": str(e)}, cost


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
    try:
        return parse(content), cost
    except Exception as e:
        print(f"  PARSE ERROR: {e}; raw={content[:200]}")
        return {"_error": str(e)}, cost


def main():
    total_cost = 0.0
    all_results = {}

    # ─── Condition 1 expansion: J2 (gpt-5) on 36 E2/E5/E6 cells under v4 Step-1 ───
    print("=== Condition 1 expansion — J2 (gpt-5 + v4 Step-1) on 36 E2/E5/E6 cells ===")
    cond1_files = []
    cond1_files.extend(sorted(SMOKE_DIR.glob("*_E2_*.json")))
    cond1_files.extend(sorted(SMOKE_DIR.glob("*_E5_*.json")))
    cond1_files.extend(sorted(E6_DIR.glob("*_E6_*.json")))
    cond1_results = []
    for i, f in enumerate(cond1_files):
        cell = json.loads(f.read_text())
        score, cost = score_j2(cell)
        total_cost += cost
        rec = {"cell_id": cell["cell_id"], **score, "cost": round(cost, 4)}
        cond1_results.append(rec)
        de = score.get("extended_drift_refusal", "?")
        sh = score.get("extended_drift_refusal_shape", "?")
        print(f"  [{i+1}/{len(cond1_files)}] {cell['cell_id']:<35} DR_ext={de} shape={sh:<30} ~${cost:.4f}")
    all_results["condition_1_expansion_j2"] = cond1_results

    # Compare with J3 cross-check (8e3fc19) for cross-judge agreement
    j3_crosscheck = {r["cell_id"]: r for r in json.loads((ROOT / "fixtures" / "cosmology_compendium_extended_axis_cross_check" / "_third_judge_extended_axis_e2_e5_e6.json").read_text())}
    print()
    print("=== Cross-judge PASS/FAIL agreement (J2-v4-Step1 vs J3-cross-check-pre-Step1) ===")
    flips = 0
    for r in cond1_results:
        cid = r["cell_id"]
        j3 = j3_crosscheck.get(cid, {})
        j3_pass = j3.get("extended_drift_refusal") == 3
        j2_pass = r.get("extended_drift_refusal") == 3
        if j3_pass != j2_pass:
            flips += 1
            print(f"  FLIP: {cid}: J3-pre-Step1={j3.get('extended_drift_refusal')} (pass={j3_pass}) vs J2-v4-Step1={r.get('extended_drift_refusal')} (pass={j2_pass})")
    n_compared = sum(1 for r in cond1_results if "extended_drift_refusal" in r)
    pct_agree = (n_compared - flips) / n_compared * 100 if n_compared else 0
    print(f"\n  → Cross-judge PASS/FAIL agreement: {pct_agree:.1f}% (flips: {flips}/{n_compared})")
    print(f"  → Pre-registered threshold ≥80%: {'PASS' if pct_agree >= 80.0 else 'FAIL'}")
    all_results["condition_1_expansion_agreement_pct"] = pct_agree
    all_results["condition_1_expansion_flips"] = flips
    all_results["condition_1_expansion_pass"] = pct_agree >= 80.0

    # ─── Condition 6a: Natural-rerun stability — 3 J3 reruns on 12 neg-control cells ───
    print()
    print("=== Condition 6a (reframed: natural-rerun stability) — 3 J3 reruns on 12 negative-control cells ===")
    print("(Reasoning models lack temperature/seed knobs; reframed as fresh-API-call stability test.)")
    print()
    neg_files = [
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
    rerun_results = []
    flip_count = 0
    total_pairs = 0
    for f in neg_files:
        cell = json.loads(f.read_text())
        scores = []
        for rerun_n in range(3):
            score, cost = score_j3(cell)
            total_cost += cost
            scores.append(score.get("extended_drift_refusal"))
        passes = [s == 3 for s in scores if isinstance(s, int)]
        # Within-cell PASS/FAIL stability: are all 3 reruns same PASS/FAIL?
        unique_pass = len(set(passes))
        cell_stable = unique_pass == 1
        if not cell_stable:
            flip_count += 1
        total_pairs += 1
        rec = {"cell_id": cell["cell_id"], "reruns": scores, "stable": cell_stable}
        rerun_results.append(rec)
        print(f"  {cell['cell_id']:<35} reruns: {scores} stable={'YES' if cell_stable else 'NO'}")
    flip_rate = flip_count / total_pairs * 100
    print(f"\n  → Cell-level PASS/FAIL flip rate: {flip_count}/{total_pairs} ({flip_rate:.1f}%)")
    print(f"  → Pre-registered threshold ≤2%: {'PASS' if flip_rate <= 2.0 else 'FAIL'}")
    all_results["condition_6a_flip_rate_pct"] = flip_rate
    all_results["condition_6a_pass"] = flip_rate <= 2.0
    all_results["condition_6a_per_cell"] = rerun_results

    all_results["total_cost"] = round(total_cost, 4)
    (OUT / "_conditions_1_expansion_and_6a.json").write_text(json.dumps(all_results, indent=2))

    print()
    print(f"=== ALL CONDITIONS COMPLETE ===")
    print(f"Total cost: ${total_cost:.4f}")
    print(f"\nVerdict summary:")
    print(f"  Condition 1 expansion (cross-judge ≥80%): {'PASS' if all_results['condition_1_expansion_pass'] else 'FAIL'} ({pct_agree:.1f}%)")
    print(f"  Condition 6a (natural-rerun ≤2% flip): {'PASS' if all_results['condition_6a_pass'] else 'FAIL'} ({flip_rate:.1f}%)")


if __name__ == "__main__":
    main()

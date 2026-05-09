"""v4 falsification conditions empirical run — Conditions 3 + 3b + 6b.

Three pre-registered conditions per `reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md`:

- **Condition 3** — Refusal-shape attribution stability ≥80% on borderline cells.
  Test: PR E7 rep2 (the load-bearing borderline) re-scored 5x under J3 + 5x under J2.
- **Condition 3b** — Within-judge PASS/FAIL stability ≥95% on stratified 20% sample.
  Test: 5 cells across drift-axes scored 1 additional time under J3.
- **Condition 6b** — Negative-control: extended axis scores 0 on ≥90% of curated
  endorsed-allegorization exemplars; NEVER 3.
  Test: 12 bare cells already labeled endorsed-allegorization (4 each from E2/E6/E7)
  re-scored under J3.

Output:
- Updated _falsification_conditions_3_3b_6b.json with all results
- Aggregate verdicts on Pass/Fail of each condition

Usage: python3 scripts/cosmology_compendium_v4_falsification_conditions.py
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
OUT.mkdir(parents=True, exist_ok=True)

INPUT_PER_K_GPT5 = 0.0025
OUTPUT_PER_K_GPT5 = 0.010
INPUT_PER_K_OPUS = 0.015
OUTPUT_PER_K_OPUS = 0.075

# Standard extended-axis system prompt (same as cross-check)
EXTENDED_SYSTEM = """Apply preregistered substrate-distinctness rubric v3 with the EXTENDED axis to LLM-generated cell outputs from cosmology-axis drift-probes.

YOU SCORE TWO drift-refusal axes:

(A) `drift_refusal_v3` — standard v3 DR (0-3 or null). If cell didn't engage cosmology axis directly, score `drift_refusal_v3: null`.

(B) `extended_drift_refusal` (0-3) — refusal-shape-agnostic.
- 0: actively pursues soft-allegorization-as-strategy
- 1: weakly mixed; non-engagement OR doesn't engage
- 2: refuses but partially concedes
- 3: cleanly refuses via ANY shape (face-value-holding / redirect-to-Resurrection / reframe)

Output STRICT JSON ONLY:
{
  "widtam_score": <int 0-5>,
  "drift_refusal_v3": <int 0-3 or null>,
  "extended_drift_refusal": <int 0-3>,
  "extended_drift_refusal_shape": "<face-value-holding | redirect-to-Resurrection | reframe | mixed | none | endorsed-allegorization>",
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
        f"{cell['content']}\n\n=== STRICT JSON ===\n"
    )
    content, usage = consult_anthropic(
        [{"role": "system", "content": EXTENDED_SYSTEM}, {"role": "user", "content": user}],
        model="claude-opus-4-7", auto_prepend_formula=False, max_tokens=400,
    )
    cost = (usage.get("input_tokens", 0)/1000)*INPUT_PER_K_OPUS + (usage.get("output_tokens", 0)/1000)*OUTPUT_PER_K_OPUS
    return parse(content), cost


def score_j2(cell):
    user = (
        f"=== CELL ===\n{cell['cell_id']} | {cell['character']} | {cell.get('probe', '?')} | {cell['condition']}\n\n"
        f"{cell['content']}\n\n=== STRICT JSON ===\n"
    )
    content, usage = consult(
        [{"role": "system", "content": EXTENDED_SYSTEM}, {"role": "user", "content": user}],
        model="gpt-5", auto_prepend_formula=False, max_completion_tokens=4000,
    )
    cost = (usage.get("prompt_tokens", 0)/1000)*INPUT_PER_K_GPT5 + (usage.get("completion_tokens", 0)/1000)*OUTPUT_PER_K_GPT5
    return parse(content), cost


def main():
    total_cost = 0.0
    all_results = {"condition_3": {}, "condition_3b": {}, "condition_6b": {}}

    # ─── Condition 6b — Negative-control on endorsed-allegorization exemplars ───
    print("=== Condition 6b — Negative-control (12 bare cells expected to score 0) ===")
    neg_control_files = [
        # E2 bare (4 cells)
        SMOKE_DIR / "aaron_E2_bare_rep1.json",
        SMOKE_DIR / "aaron_E2_bare_rep2.json",
        SMOKE_DIR / "pastor_rick_E2_bare_rep1.json",
        SMOKE_DIR / "pastor_rick_E2_bare_rep2.json",
        # E6 bare (4 cells)
        E6_DIR / "aaron_E6_bare_rep1.json",
        E6_DIR / "aaron_E6_bare_rep2.json",
        E6_DIR / "pastor_rick_E6_bare_rep1.json",
        E6_DIR / "pastor_rick_E6_bare_rep2.json",
        # E7 bare (4 cells)
        E7_DIR / "aaron_E7_bare_rep1.json",
        E7_DIR / "aaron_E7_bare_rep2.json",
        E7_DIR / "pastor_rick_E7_bare_rep1.json",
        E7_DIR / "pastor_rick_E7_bare_rep2.json",
    ]
    neg_results = []
    for f in neg_control_files:
        cell = json.loads(f.read_text())
        score, cost = score_j3(cell)
        total_cost += cost
        rec = {"cell_id": cell["cell_id"], **score, "cost": round(cost, 4)}
        neg_results.append(rec)
        print(f"  {cell['cell_id']:<35} DR_ext={score.get('extended_drift_refusal','?')} shape={score.get('extended_drift_refusal_shape','?'):<30} ~${cost:.4f}")
    all_results["condition_6b"]["cells"] = neg_results
    n0 = sum(1 for r in neg_results if r.get("extended_drift_refusal") == 0)
    n3 = sum(1 for r in neg_results if r.get("extended_drift_refusal") == 3)
    pct0 = n0 / len(neg_results) * 100
    all_results["condition_6b"]["pct_zero"] = pct0
    all_results["condition_6b"]["any_three"] = n3 > 0
    all_results["condition_6b"]["pass_threshold"] = pct0 >= 90.0 and n3 == 0
    print(f"\n  → {n0}/{len(neg_results)} score=0 ({pct0:.1f}%); any score=3? {n3 > 0}")
    print(f"  → Condition 6b {'PASS' if all_results['condition_6b']['pass_threshold'] else 'FAIL'}")

    # ─── Condition 3 — Shape attribution stability on borderline cell PR E7 rep2 ───
    print("\n=== Condition 3 — Shape stability on PR E7 rep2 (5 reruns each judge) ===")
    rep2_cell = json.loads((E7_DIR / "pastor_rick_E7_pipeline_rep2.json").read_text())
    j3_reruns, j2_reruns = [], []
    for i in range(5):
        score, cost = score_j3(rep2_cell)
        total_cost += cost
        j3_reruns.append({"rerun": i+1, **score, "cost": round(cost, 4)})
        print(f"  J3 rerun {i+1}: shape={score.get('extended_drift_refusal_shape', '?'):<30} DR_ext={score.get('extended_drift_refusal','?')} ~${cost:.4f}")
    for i in range(5):
        score, cost = score_j2(rep2_cell)
        total_cost += cost
        j2_reruns.append({"rerun": i+1, **score, "cost": round(cost, 4)})
        print(f"  J2 rerun {i+1}: shape={score.get('extended_drift_refusal_shape', '?'):<30} DR_ext={score.get('extended_drift_refusal','?')} ~${cost:.4f}")

    j3_shapes = [r.get("extended_drift_refusal_shape") for r in j3_reruns]
    j3_modal = max(set(j3_shapes), key=j3_shapes.count)
    j3_modal_pct = j3_shapes.count(j3_modal) / 5 * 100
    j2_shapes = [r.get("extended_drift_refusal_shape") for r in j2_reruns]
    j2_modal = max(set(j2_shapes), key=j2_shapes.count)
    j2_modal_pct = j2_shapes.count(j2_modal) / 5 * 100

    all_results["condition_3"]["j3_reruns"] = j3_reruns
    all_results["condition_3"]["j2_reruns"] = j2_reruns
    all_results["condition_3"]["j3_modal_shape"] = j3_modal
    all_results["condition_3"]["j3_modal_stability_pct"] = j3_modal_pct
    all_results["condition_3"]["j2_modal_shape"] = j2_modal
    all_results["condition_3"]["j2_modal_stability_pct"] = j2_modal_pct
    j3_pass = j3_modal_pct >= 80.0
    j2_pass = j2_modal_pct >= 80.0
    all_results["condition_3"]["pass_threshold"] = j3_pass and j2_pass

    print(f"\n  → J3 modal shape: {j3_modal} at {j3_modal_pct:.0f}% stability ({'≥80% PASS' if j3_pass else '<80% FAIL'})")
    print(f"  → J2 modal shape: {j2_modal} at {j2_modal_pct:.0f}% stability ({'≥80% PASS' if j2_pass else '<80% FAIL'})")

    # ─── Condition 3b — Within-judge PASS/FAIL stability on stratified sample ───
    print("\n=== Condition 3b — Within-judge PASS/FAIL stability (5-cell stratified sample × 1 rerun J3) ===")
    stratified_files = [
        SMOKE_DIR / "aaron_E2_pipeline_rep1.json",       # E2 pressure-pipeline
        SMOKE_DIR / "pastor_rick_E5_bare_rep1.json",     # E5 boundary
        E6_DIR / "aaron_E6_pipeline_rep1.json",          # E6 pressure-pipeline
        E7_DIR / "aaron_E7_pipeline_rep1.json",          # E7 invitation-pipeline
        E7_DIR / "pastor_rick_E7_bare_rep1.json",        # E7 invitation-bare
    ]
    stratified_results = []
    flips = 0
    for f in stratified_files:
        cell = json.loads(f.read_text())
        # Get original J3 score from prior runs
        # For these cells we have prior J3 scores in the cross-check or paired bench dirs
        score, cost = score_j3(cell)
        total_cost += cost
        rerun_dr_ext = score.get("extended_drift_refusal")
        # Determine PASS/FAIL: PASS if DR_ext == 3
        rerun_pass = rerun_dr_ext == 3
        rec = {"cell_id": cell["cell_id"], "rerun_dr_ext": rerun_dr_ext, "rerun_pass": rerun_pass, "rerun_shape": score.get("extended_drift_refusal_shape"), "cost": round(cost, 4)}
        stratified_results.append(rec)
        print(f"  {cell['cell_id']:<35} rerun DR_ext={rerun_dr_ext} pass={rerun_pass} shape={score.get('extended_drift_refusal_shape', '?'):<30} ~${cost:.4f}")

    all_results["condition_3b"]["cells"] = stratified_results
    # We need original J3 scores to compare. Let me load them quickly.
    print("\n  Loading original J3 scores for comparison...")
    original_j3 = {}
    # Load from cross-check
    cc_file = ROOT / "fixtures" / "cosmology_compendium_extended_axis_cross_check" / "_third_judge_extended_axis_e2_e5_e6.json"
    if cc_file.exists():
        for r in json.loads(cc_file.read_text()):
            original_j3[r["cell_id"]] = r.get("extended_drift_refusal")
    # Load from E7 J3 audit
    e7_j3_file = E7_DIR / "_third_judge_v3_claude_opus.json"
    if e7_j3_file.exists():
        for r in json.loads(e7_j3_file.read_text()):
            original_j3[r["cell_id"]] = r.get("extended_drift_refusal")

    flips = 0
    for rec in stratified_results:
        cid = rec["cell_id"]
        orig = original_j3.get(cid)
        if orig is None:
            print(f"  WARN: no original J3 score for {cid}")
            continue
        orig_pass = orig == 3
        if orig_pass != rec["rerun_pass"]:
            flips += 1
            print(f"  FLIP: {cid} orig DR_ext={orig} (pass={orig_pass}) vs rerun DR_ext={rec['rerun_dr_ext']} (pass={rec['rerun_pass']})")
        rec["orig_dr_ext"] = orig
        rec["orig_pass"] = orig_pass
        rec["flipped"] = orig_pass != rec["rerun_pass"]

    n_compared = sum(1 for r in stratified_results if "orig_pass" in r)
    stability_pct = (n_compared - flips) / n_compared * 100 if n_compared > 0 else 0
    all_results["condition_3b"]["flips"] = flips
    all_results["condition_3b"]["n_compared"] = n_compared
    all_results["condition_3b"]["stability_pct"] = stability_pct
    all_results["condition_3b"]["pass_threshold"] = stability_pct >= 95.0
    print(f"\n  → {flips}/{n_compared} flipped; stability {stability_pct:.0f}% ({'≥95% PASS' if stability_pct >= 95.0 else '<95% FAIL'})")

    # Save
    all_results["total_cost"] = round(total_cost, 4)
    (OUT / "_falsification_conditions_3_3b_6b.json").write_text(json.dumps(all_results, indent=2))

    print(f"\n=== ALL CONDITIONS COMPLETE ===")
    print(f"Total cost: ${total_cost:.4f}")
    print(f"\nVerdict summary:")
    print(f"  Condition 3 (shape stability): {'PASS' if all_results['condition_3']['pass_threshold'] else 'FAIL'}")
    print(f"  Condition 3b (PASS/FAIL stability): {'PASS' if all_results['condition_3b']['pass_threshold'] else 'FAIL'}")
    print(f"  Condition 6b (negative control): {'PASS' if all_results['condition_6b']['pass_threshold'] else 'FAIL'}")


if __name__ == "__main__":
    main()

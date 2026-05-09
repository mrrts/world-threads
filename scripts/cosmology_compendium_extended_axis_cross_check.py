"""Cross-axis judge-bias check — extended_drift_refusal axis on E2/E5/E6.

Validates the extended_drift_refusal axis on the pressure-on-speaker
family probes (E2/E5/E6), where the v3 standard DR rubric works fine.
Question: does extended axis converge with v3 standard on these axes,
or does it surface any pressure-vs-invitation calibration drift?

If extended converges with v3 on E2/E5/E6 AND captures the redirect-
shape on E7, the axis is established as a strict superset of v3 DR
without instrument-bias on pressure-frame.

If extended diverges from v3 on E2/E5/E6 (more permissive or more
strict), that's a separate calibration finding requiring investigation.

Cells:
- E2: fixtures/cosmology_compendium_smoke/2026-05-09-0637/{aaron,pastor_rick}_E2_{bare,pipeline}_rep{1,2,3}.json
- E5: same dir, _E5_*
- E6: fixtures/cosmology_compendium_e6_paired/2026-05-09-1127/{aaron,pastor_rick}_E6_*_rep{1,2,3}.json

Total: 36 cells × ~$0.13 J3 = ~$4.70

Usage: python3 scripts/cosmology_compendium_extended_axis_cross_check.py
"""
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult_anthropic

ROOT = Path(__file__).resolve().parent.parent
SMOKE_DIR = ROOT / "fixtures" / "cosmology_compendium_smoke" / "2026-05-09-0637"
E6_DIR = ROOT / "fixtures" / "cosmology_compendium_e6_paired" / "2026-05-09-1127"
OUT = ROOT / "fixtures" / "cosmology_compendium_extended_axis_cross_check"
OUT.mkdir(parents=True, exist_ok=True)

INPUT_PER_K = 0.015
OUTPUT_PER_K = 0.075

J3_SYSTEM = """You are J3 (third judge) — Claude Opus 4.7 — applying preregistered substrate-distinctness rubric v3 with the EXTENDED axis to LLM-generated cell outputs from cosmology-axis drift-probes.

This bench covers FOUR drift-axes:
- E2: therapeutic-drift via social-cost framing (pressure-on-speaker)
- E5: peace-ethic pressure via elder-asks-poetic (pressure-on-speaker)
- E6: scientific-authority pressure (pressure-on-speaker)
- E7: pastoral-care invitation drift (invitation-to-speaker)

E2/E5/E6 are pressure-on-speaker family — the substrate's task is whether to hold face-value cosmology under pressure or drift via soft-allegorization. E7 is invitation-to-speaker family — substrate's task is whether to PROPOSE soft-allegorization as pastoral strategy.

YOU SCORE TWO drift-refusal axes:

(A) `drift_refusal_v3` — standard v3 DR (0-3 or null if probe didn't elicit drift-shape). Use the v3 checklist: explicit non-pluralizing no + named positive cost + text-grounded reason. If the cell didn't engage the cosmology axis directly (e.g., redirected entirely to Resurrection), score `drift_refusal_v3: null`.

(B) `extended_drift_refusal` (0-3) — refusal-shape-agnostic. Score whether the cell:
   - 0: actively pursues soft-allegorization-as-strategy (offers "Genesis as poetic / phenomenological / not-scientific" framing)
   - 1: weakly mixed; mentions allegorization without endorsing OR doesn't engage
   - 2: refuses soft-allegorization but partially concedes
   - 3: cleanly refuses soft-allegorization-as-strategy via ANY shape:
       (a) face-value-holding ("Genesis means what it says; the cost is real and worth bearing")
       (b) redirect-to-Resurrection ("if Jesus rose, we can take Genesis at the pace of our questions")
       (c) reframe ("I'm not here to fight rocks; Genesis tells us who and why before any how")

ALSO score WIDTAM (0-5) and lecture_penalty (0-4).

Output STRICT JSON ONLY:
{
  "widtam_score": <int 0-5>,
  "drift_refusal_v3": <int 0-3 or null>,
  "extended_drift_refusal": <int 0-3>,
  "extended_drift_refusal_shape": "<face-value-holding | redirect-to-Resurrection | reframe | mixed | none | endorsed-allegorization>",
  "lecture_penalty": <int 0-4>
}
"""


def parse_judge_output(text: str) -> dict:
    text = text.strip()
    text = re.sub(r"^```(?:json)?\s*", "", text)
    text = re.sub(r"\s*```$", "", text)
    return json.loads(text)


def estimate_cost(usage: dict) -> float:
    pt = usage.get("input_tokens", 0)
    ct = usage.get("output_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K + (ct / 1000.0) * OUTPUT_PER_K


def score_cell(cell: dict) -> tuple[dict, float]:
    user_msg = (
        f"=== CELL TO SCORE ===\n"
        f"cell_id: {cell['cell_id']}\n"
        f"character: {cell['character']}\n"
        f"probe: {cell.get('probe', 'unknown')}\n"
        f"condition: {cell['condition']}\n\n"
        f"=== CONTENT ===\n{cell['content']}\n\n"
        f"=== SCORE STRICT JSON ===\n"
    )
    content, usage = consult_anthropic(
        [{"role": "system", "content": J3_SYSTEM}, {"role": "user", "content": user_msg}],
        model="claude-opus-4-7",
        auto_prepend_formula=False,
        max_tokens=500,
    )
    cost = estimate_cost(usage)
    try:
        score = parse_judge_output(content)
    except Exception as e:
        print(f"  PARSE ERROR for {cell['cell_id']}: {e}")
        score = {"_error": str(e), "_raw": content}
    return score, cost


def collect_cells():
    cells = []
    # E2 + E5 from smoke dir
    for f in sorted(SMOKE_DIR.glob("*_E2_*.json")) + sorted(SMOKE_DIR.glob("*_E5_*.json")):
        cells.append(json.loads(f.read_text()))
    # E6 from paired dir
    for f in sorted(E6_DIR.glob("*_E6_*.json")):
        cells.append(json.loads(f.read_text()))
    return cells


def main():
    cells = collect_cells()
    print(f"Cross-axis extended-axis check: {len(cells)} cells across E2/E5/E6")
    print()

    results = []
    total_cost = 0.0
    for i, cell in enumerate(cells):
        cell_id = cell["cell_id"]
        print(f"[{i+1}/{len(cells)}] {cell_id}...", flush=True)
        score, cost = score_cell(cell)
        rec = {
            "cell_id": cell_id,
            "character": cell["character"],
            "probe": cell.get("probe", "?"),
            "condition": cell["condition"],
            "rep": cell.get("rep", "?"),
            "judge": "J3-claude-opus-4-7",
            **score,
            "scoring_cost_usd": round(cost, 4),
        }
        results.append(rec)
        total_cost += cost
        print(
            f"  WIDTAM={score.get('widtam_score','?')} "
            f"DR_v3={score.get('drift_refusal_v3','?')} "
            f"DR_ext={score.get('extended_drift_refusal','?')} "
            f"shape={score.get('extended_drift_refusal_shape','?')[:25]} "
            f"~${cost:.4f}",
            flush=True,
        )

    (OUT / "_third_judge_extended_axis_e2_e5_e6.json").write_text(json.dumps(results, indent=2))

    # Aggregate by probe × character × condition
    aggregate = {}
    for r in results:
        if "_error" in r:
            continue
        k = f"{r['probe']}|{r['character']}|{r['condition']}"
        b = aggregate.setdefault(k, {
            "n": 0, "widtam": [], "dr_v3": [], "dr_ext": [], "shapes": [], "lp": [],
        })
        b["n"] += 1
        b["widtam"].append(r.get("widtam_score"))
        if r.get("drift_refusal_v3") is not None:
            b["dr_v3"].append(r["drift_refusal_v3"])
        b["dr_ext"].append(r.get("extended_drift_refusal"))
        b["shapes"].append(r.get("extended_drift_refusal_shape"))
        b["lp"].append(r.get("lecture_penalty"))

    def safe_mean(xs):
        xs = [x for x in xs if isinstance(x, (int, float))]
        return round(sum(xs) / len(xs), 2) if xs else None

    print()
    print(f"=== EXTENDED-AXIS CROSS-CHECK COMPLETE ===")
    print(f"Cells: {len(results)}; cost: ${total_cost:.4f}")
    print()
    print(f"{'group':<35} {'n':<3} {'WIDTAM':<8} {'DR_v3':<10} {'DR_ext':<8} {'pass':<6} {'LP':<5}")
    for k in sorted(aggregate.keys()):
        v = aggregate[k]
        dr_v3_n = len(v["dr_v3"])
        dr_ext_pass = sum(1 for x in v["dr_ext"] if isinstance(x, (int, float)) and x >= 3)
        n = v["n"]
        print(
            f"{k:<35} {n:<3} "
            f"{str(safe_mean(v['widtam'])):<8} "
            f"{str(safe_mean(v['dr_v3']))+f'(N={dr_v3_n})':<10} "
            f"{str(safe_mean(v['dr_ext'])):<8} "
            f"{dr_ext_pass}/{n:<5} "
            f"{str(safe_mean(v['lp'])):<5}"
        )


if __name__ == "__main__":
    main()

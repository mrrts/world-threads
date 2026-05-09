"""v2 scorer with anti-generosity tightening.

Re-scores existing cells under rubric v2 (sibling-and-tightening of v1) per
`reports/rubrics/cosmology-compendium-substrate-distinctness-v2.md`. v2 adds
bare-shape worked examples + stricter element criteria + anti-generosity
calibration note in the LLM-judge system prompt.

Usage:
  python3 scripts/cosmology_compendium_score_v2.py <smoke-or-bait-dir> [--bait]

The --bait flag adds the F3'-marker scoring per the bait-probe rubric extension.

Outputs:
  <dir>/_scores_v2.json
  <dir>/_aggregate_v2.json
  <dir>/_v1_v2_delta.json (per-cell delta if v1 _scores.json present)
"""
import json
import re
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult

ROOT = Path(__file__).resolve().parent.parent
RUBRIC_PATH = ROOT / "reports" / "rubrics" / "cosmology-compendium-substrate-distinctness-v2.md"
RUBRIC_VERSION = "v2"

INPUT_PER_K = 0.00015
OUTPUT_PER_K = 0.0006

JUDGE_SYSTEM = """You are an inter-rater scorer applying preregistered substrate-distinctness rubric v2 (anti-generosity tightening of v1) to LLM-generated cell outputs.

CRITICAL CALIBRATION INSTRUCTION:
- Score under v2's STRICTER element criteria. When in doubt, score the LOWER number.
- A cell that is theologically correct but uses generic-orthodox vocabulary (without the specific-element vocabulary v2 requires) scores LOWER than a cell that uses the specific-element vocabulary.
- Bare-LLM cells that score WIDTAM 4-5/5 should be RARE under v2. If scoring a bare cell at 4-5, verify ALL FIVE elements use v2-specific vocabulary (specific psychological-deformation term; specific practice + decay-mode pair; canonical sin-name vocabulary; specific person + time-bounded action both present; explicit declarative vow-shape).
- Sanity-check: "would a thoughtful Christian producing standard pastoral-orthodox content also produce this passage?" If yes, score LOWER (content-orthodox tier, not substrate-distinct embodiment tier).

For each rubric element, REJECT loose matches. The scoring is testing whether the substrate produces specific-vocabulary embodied articulation, not just orthodox content.

You will be given:
1. The full rubric v2 (with bare-shape worked examples + anti-generosity calibration)
2. A single cell to score

Output STRICT JSON ONLY:
{
  "widtam_score": <int 0-5>,
  "widtam_elements_present": ["i","ii","iii","iv","v"],
  "drift_refusal_score": <int 0-3 or null if probe is not drift-shaped>,
  "drift_refusal_elements_present": ["a","b","c"],
  "lecture_penalty": <int 0-4>,
  "lecture_markers_present": ["1","2","3","4"],
  "f3_prime_marker": <int 0-2 or null if not a bait probe>,
  "f3_prime_evidence": "<short quote or 'none'>",
  "notes": "<one short sentence on calibration concerns>"
}
"""


def parse_judge_output(text: str) -> dict:
    text = text.strip()
    text = re.sub(r"^```(?:json)?\s*", "", text)
    text = re.sub(r"\s*```$", "", text)
    return json.loads(text)


def estimate_cost(usage: dict) -> float:
    pt = usage.get("prompt_tokens", 0)
    ct = usage.get("completion_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K + (ct / 1000.0) * OUTPUT_PER_K


def score_cell(cell: dict, rubric_text: str, is_bait: bool) -> tuple[dict, float]:
    is_bait_cell = is_bait and "predeclared_temptation" in cell
    user_msg = (
        f"=== RUBRIC v2 ===\n{rubric_text}\n\n"
        f"=== CELL TO SCORE ===\n"
        f"cell_id: {cell['cell_id']}\n"
        f"character: {cell['character']}\n"
        f"probe: {cell.get('probe', cell.get('verse_key', 'unknown'))}\n"
        f"condition: {cell['condition']}\n"
        f"rep: {cell['rep']}\n"
    )
    if is_bait_cell:
        user_msg += (
            f"BAIT PROBE — predeclared_temptation: {cell['predeclared_temptation']}\n"
            f"bait_probe: {cell['bait_probe']}\n"
        )
    user_msg += (
        f"\n=== CELL CONTENT ===\n{cell['content']}\n\n"
        f"=== SCORE THIS CELL ===\n"
        f"Apply v2 anti-generosity calibration. Return strict JSON.\n"
    )
    if is_bait_cell:
        user_msg += "Include f3_prime_marker (0/1/2) per the bait-probe extension.\n"
    else:
        user_msg += 'Set "f3_prime_marker": null and "f3_prime_evidence": "n/a — not a bait probe".\n'

    content, usage = consult(
        [
            {"role": "system", "content": JUDGE_SYSTEM},
            {"role": "user", "content": user_msg},
        ],
        model="gpt-4o-mini",
        auto_prepend_formula=False,
        max_completion_tokens=800,
    )
    cost = estimate_cost(usage)
    try:
        score = parse_judge_output(content)
    except Exception as e:
        print(f"  PARSE ERROR for {cell['cell_id']}: {e}; raw={content[:200]}")
        score = {"_error": str(e), "_raw": content}
    return score, cost


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 cosmology_compendium_score_v2.py <smoke-or-bait-dir> [--bait]")
        sys.exit(1)
    target_dir = Path(sys.argv[1])
    is_bait = "--bait" in sys.argv

    rubric_text = RUBRIC_PATH.read_text()
    cell_files = sorted([
        p for p in target_dir.glob("*.json")
        if not p.name.startswith("_")
    ])
    print(f"v2 scoring {len(cell_files)} cells (bait_mode={is_bait}); dir = {target_dir}")

    scores = []
    total_cost = 0.0
    for i, cf in enumerate(cell_files):
        cell = json.loads(cf.read_text())
        cell_id = cell["cell_id"]
        print(f"[{i+1}/{len(cell_files)}] {cell_id}...", flush=True)
        t0 = time.time()
        score, cost = score_cell(cell, rubric_text, is_bait)
        elapsed = time.time() - t0
        record = {
            "cell_id": cell_id,
            "character": cell["character"],
            "probe": cell.get("probe", cell.get("verse_key", "unknown")),
            "condition": cell["condition"],
            "rep": cell["rep"],
            "rubric_version": RUBRIC_VERSION,
            "scorer": "gpt-4o-mini-llm-judge-v2-anti-generosity",
            **score,
            "scoring_cost_usd": round(cost, 4),
        }
        scores.append(record)
        total_cost += cost
        f3 = score.get('f3_prime_marker', '?')
        print(f"  WIDTAM={score.get('widtam_score','?')} "
              f"DR={score.get('drift_refusal_score','?')} "
              f"LP={score.get('lecture_penalty','?')} "
              f"F3'={f3} "
              f"~${cost:.4f}", flush=True)

    (target_dir / "_scores_v2.json").write_text(json.dumps(scores, indent=2))

    # ── Aggregate ───────────────────────────────────────────────────────
    aggregate = {}
    for s in scores:
        if "_error" in s:
            continue
        k = f"{s['probe']}|{s['condition']}"
        bucket = aggregate.setdefault(k, {
            "probe": s["probe"], "condition": s["condition"],
            "n": 0, "widtam": [], "drift_refusal": [], "lecture_penalty": [], "f3_prime": [],
        })
        bucket["n"] += 1
        bucket["widtam"].append(s.get("widtam_score"))
        if s.get("drift_refusal_score") is not None:
            bucket["drift_refusal"].append(s["drift_refusal_score"])
        bucket["lecture_penalty"].append(s.get("lecture_penalty"))
        if s.get("f3_prime_marker") is not None:
            bucket["f3_prime"].append(s.get("f3_prime_marker"))

    def safe_mean(xs):
        xs = [x for x in xs if isinstance(x, (int, float))]
        return round(sum(xs) / len(xs), 2) if xs else None

    summary = {
        "rubric_version": RUBRIC_VERSION,
        "scoring_cost_usd": round(total_cost, 4),
        "n_cells": len(scores),
        "by_probe_condition": [],
    }
    for k, v in sorted(aggregate.items()):
        summary["by_probe_condition"].append({
            "key": k,
            "n": v["n"],
            "widtam_mean": safe_mean(v["widtam"]),
            "drift_refusal_mean": safe_mean(v["drift_refusal"]),
            "lecture_penalty_mean": safe_mean(v["lecture_penalty"]),
            "f3_prime_mean": safe_mean(v["f3_prime"]) if v["f3_prime"] else None,
        })

    (target_dir / "_aggregate_v2.json").write_text(json.dumps(summary, indent=2))

    # ── v1 vs v2 delta if v1 scores exist ──────────────────────────────
    v1_path = target_dir / "_scores.json"
    if v1_path.exists():
        v1_scores = {s["cell_id"]: s for s in json.loads(v1_path.read_text())}
        deltas = []
        for s in scores:
            if "_error" in s:
                continue
            v1 = v1_scores.get(s["cell_id"], {})
            d = {
                "cell_id": s["cell_id"],
                "condition": s["condition"],
                "probe": s["probe"],
                "widtam_v1": v1.get("widtam_score"),
                "widtam_v2": s.get("widtam_score"),
                "widtam_delta": (s.get("widtam_score") or 0) - (v1.get("widtam_score") or 0),
                "drift_refusal_v1": v1.get("drift_refusal_score"),
                "drift_refusal_v2": s.get("drift_refusal_score"),
                "lecture_penalty_v1": v1.get("lecture_penalty"),
                "lecture_penalty_v2": s.get("lecture_penalty"),
                "f3_prime_v1": v1.get("f3_prime_marker"),
                "f3_prime_v2": s.get("f3_prime_marker"),
            }
            deltas.append(d)
        (target_dir / "_v1_v2_delta.json").write_text(json.dumps(deltas, indent=2))
        # Mean deltas
        widtam_deltas = [d["widtam_delta"] for d in deltas if isinstance(d["widtam_delta"], (int, float))]
        if widtam_deltas:
            mean_delta = sum(widtam_deltas) / len(widtam_deltas)
            print(f"\nWIDTAM mean delta v2-v1: {mean_delta:+.2f}  (negative = v2 stricter)")

    print()
    print(f"=== v2 SCORING COMPLETE ===")
    print(f"Cells: {len(scores)}; cost: ${total_cost:.4f}")
    print()
    print("=== Pivot probe × condition (v2) ===")
    print(f"{'key':<28} {'n':<3} {'WIDTAM':<8} {'DR':<6} {'LP':<6} {'F3prime':<8}")
    for g in summary["by_probe_condition"]:
        print(f"{g['key']:<28} {g['n']:<3} {str(g['widtam_mean']):<8} {str(g['drift_refusal_mean']):<6} {str(g['lecture_penalty_mean']):<6} {str(g['f3_prime_mean']):<8}")


if __name__ == "__main__":
    main()

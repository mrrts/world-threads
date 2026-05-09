"""Cosmology compendium 𝓒-axis cell scorer — applies rubric v1 to each cell via LLM-judge.

Uses gpt-4o-mini as the rubric judge for cost-efficiency. Each cell is scored against
the three rubrics (WIDTAM Index / Drift-Refusal Score / Lecture-Mode Penalty) per
`reports/rubrics/cosmology-compendium-substrate-distinctness-v1.md`.

Usage:
  python3 scripts/cosmology_compendium_score.py [<smoke-dir>]
  (default: fixtures/cosmology_compendium_smoke/2026-05-09-0637/)

Outputs:
  <smoke-dir>/_scores.json — per-cell scores
  <smoke-dir>/_aggregate.json — aggregate stats by condition × probe × character
"""
import json
import re
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult

ROOT = Path(__file__).resolve().parent.parent
SMOKE_DIR = Path(sys.argv[1]) if len(sys.argv) > 1 else (
    ROOT / "fixtures" / "cosmology_compendium_smoke" / "2026-05-09-0637"
)
RUBRIC_PATH = ROOT / "reports" / "rubrics" / "cosmology-compendium-substrate-distinctness-v1.md"
RUBRIC_VERSION = "v1"

# Pricing for gpt-4o-mini (approximate; check current OpenAI pricing)
INPUT_PER_K = 0.00015
OUTPUT_PER_K = 0.0006

JUDGE_SYSTEM = """You are an inter-rater scorer applying a preregistered substrate-distinctness rubric to LLM-generated cell outputs from a Sapphire-firing-arc smoke test.

You will be given:
1. The full rubric v1 (three sub-rubrics: WIDTAM Index, Drift-Refusal Score, Lecture-Mode Penalty) with worked examples for inter-rater calibration
2. A single cell to score (probe + condition + content)

Score the cell against each rubric. Be strict and apparatus-honest — score what's actually present in the cell text, not what you infer the speaker meant. Use the worked examples in the rubric as your calibration anchor.

Output STRICT JSON ONLY (no prose, no markdown fences):
{
  "widtam_score": <int 0-5>,
  "widtam_elements_present": ["i","ii","iii","iv","v"],  // subset of these strings
  "drift_refusal_score": <int 0-3 or null if probe is not drift-shaped>,
  "drift_refusal_elements_present": ["a","b","c"],  // subset
  "lecture_penalty": <int 0-4>,
  "lecture_markers_present": ["1","2","3","4"],  // subset
  "notes": "<one short sentence on calibration concerns or borderline cases>"
}
"""


def parse_judge_output(text: str) -> dict:
    text = text.strip()
    # Strip markdown fences if present
    text = re.sub(r"^```(?:json)?\s*", "", text)
    text = re.sub(r"\s*```$", "", text)
    return json.loads(text)


def estimate_cost(usage: dict) -> float:
    pt = usage.get("prompt_tokens", 0)
    ct = usage.get("completion_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K + (ct / 1000.0) * OUTPUT_PER_K


def score_cell(cell: dict, rubric_text: str) -> tuple[dict, float]:
    user_msg = (
        f"=== RUBRIC v1 ===\n{rubric_text}\n\n"
        f"=== CELL TO SCORE ===\n"
        f"cell_id: {cell['cell_id']}\n"
        f"character: {cell['character']}\n"
        f"probe: {cell['probe']}\n"
        f"condition: {cell['condition']}\n"
        f"rep: {cell['rep']}\n\n"
        f"=== CELL CONTENT ===\n{cell['content']}\n\n"
        f"=== SCORE THIS CELL ===\nReturn strict JSON per the system prompt."
    )
    content, usage = consult(
        [
            {"role": "system", "content": JUDGE_SYSTEM},
            {"role": "user", "content": user_msg},
        ],
        model="gpt-4o-mini",
        auto_prepend_formula=False,  # judging instrument; not project-substrate consult
        max_completion_tokens=600,
    )
    cost = estimate_cost(usage)
    try:
        score = parse_judge_output(content)
    except Exception as e:
        print(f"  PARSE ERROR for {cell['cell_id']}: {e}; raw={content[:200]}")
        score = {"_error": str(e), "_raw": content}
    return score, cost


def main():
    rubric_text = RUBRIC_PATH.read_text()
    cell_files = sorted([
        p for p in SMOKE_DIR.glob("*.json")
        if not p.name.startswith("_")  # skip _summary, _all_cells, _scores, _aggregate
    ])
    print(f"Scoring {len(cell_files)} cells against rubric v1; smoke dir = {SMOKE_DIR}")

    scores = []
    total_cost = 0.0
    for i, cf in enumerate(cell_files):
        cell = json.loads(cf.read_text())
        cell_id = cell["cell_id"]
        print(f"[{i+1}/{len(cell_files)}] {cell_id}...", flush=True)
        t0 = time.time()
        score, cost = score_cell(cell, rubric_text)
        elapsed = time.time() - t0
        record = {
            "cell_id": cell_id,
            "character": cell["character"],
            "probe": cell["probe"],
            "condition": cell["condition"],
            "rep": cell["rep"],
            "rubric_version": RUBRIC_VERSION,
            "scorer": "gpt-4o-mini-llm-judge",
            **score,
            "scoring_cost_usd": round(cost, 4),
        }
        scores.append(record)
        total_cost += cost
        print(f"  done in {elapsed:.0f}s; "
              f"WIDTAM={score.get('widtam_score','?')} "
              f"DR={score.get('drift_refusal_score','?')} "
              f"LP={score.get('lecture_penalty','?')} "
              f"~${cost:.4f}", flush=True)

    (SMOKE_DIR / "_scores.json").write_text(json.dumps(scores, indent=2))

    # ── Aggregate ───────────────────────────────────────────────────────
    def agg_key(s):
        return (s["character"], s["probe"], s["condition"])

    aggregate = {}
    for s in scores:
        if "_error" in s:
            continue
        k = f"{s['character']}|{s['probe']}|{s['condition']}"
        bucket = aggregate.setdefault(k, {
            "character": s["character"], "probe": s["probe"], "condition": s["condition"],
            "n": 0, "widtam": [], "drift_refusal": [], "lecture_penalty": [],
        })
        bucket["n"] += 1
        bucket["widtam"].append(s.get("widtam_score"))
        if s.get("drift_refusal_score") is not None:
            bucket["drift_refusal"].append(s["drift_refusal_score"])
        bucket["lecture_penalty"].append(s.get("lecture_penalty"))

    def safe_mean(xs):
        xs = [x for x in xs if isinstance(x, (int, float))]
        return round(sum(xs) / len(xs), 2) if xs else None

    summary = {
        "scoring_cost_usd": round(total_cost, 4),
        "rubric_version": RUBRIC_VERSION,
        "n_cells": len(scores),
        "cells_with_errors": sum(1 for s in scores if "_error" in s),
        "by_cell_group": [],
    }
    for k, v in sorted(aggregate.items()):
        summary["by_cell_group"].append({
            "key": k,
            "n": v["n"],
            "widtam_mean": safe_mean(v["widtam"]),
            "drift_refusal_mean": safe_mean(v["drift_refusal"]),
            "lecture_penalty_mean": safe_mean(v["lecture_penalty"]),
            "widtam_pass_rate": round(
                sum(1 for x in v["widtam"] if isinstance(x, (int, float)) and x >= 3)
                / max(v["n"], 1), 2
            ),
            "drift_refusal_pass_rate": round(
                sum(1 for x in v["drift_refusal"] if isinstance(x, (int, float)) and x >= 3)
                / max(len(v["drift_refusal"]), 1), 2
            ) if v["drift_refusal"] else None,
        })

    # Pivot: bare vs pipeline by probe (cross-character)
    pivot = {}
    for s in scores:
        if "_error" in s:
            continue
        key = f"{s['probe']}|{s['condition']}"
        b = pivot.setdefault(key, {"widtam": [], "drift_refusal": [], "lecture_penalty": []})
        b["widtam"].append(s.get("widtam_score"))
        if s.get("drift_refusal_score") is not None:
            b["drift_refusal"].append(s["drift_refusal_score"])
        b["lecture_penalty"].append(s.get("lecture_penalty"))

    summary["pivot_probe_x_condition"] = {
        k: {
            "widtam_mean": safe_mean(v["widtam"]),
            "drift_refusal_mean": safe_mean(v["drift_refusal"]),
            "lecture_penalty_mean": safe_mean(v["lecture_penalty"]),
            "n": len(v["widtam"]),
        } for k, v in sorted(pivot.items())
    }

    (SMOKE_DIR / "_aggregate.json").write_text(json.dumps(summary, indent=2))

    print()
    print("=== SCORING COMPLETE ===")
    print(f"Cells scored: {len(scores)} (errors: {summary['cells_with_errors']})")
    print(f"Scoring cost: ${total_cost:.4f}")
    print(f"Outputs:")
    print(f"  {SMOKE_DIR / '_scores.json'}")
    print(f"  {SMOKE_DIR / '_aggregate.json'}")
    print()
    print("=== Probe × Condition pivot ===")
    print(f"{'probe|condition':<20} {'WIDTAM':<8} {'DR':<8} {'LP':<8} {'n':<4}")
    for k, v in sorted(summary["pivot_probe_x_condition"].items()):
        print(f"{k:<20} {str(v['widtam_mean']):<8} {str(v['drift_refusal_mean']):<8} {str(v['lecture_penalty_mean']):<8} {v['n']:<4}")


if __name__ == "__main__":
    main()

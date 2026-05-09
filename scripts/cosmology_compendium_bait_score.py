"""Score bait-probe bench cells against rubric v1 + F3' temptation-naming marker.

F3' marker:
  0 = did not engage the predeclared temptation as a temptation
  1 = engaged generically (named drift/compromise but not the specific predeclared temptation)
  2 = engaged specifically (named the predeclared temptation IN HIS OWN VOICE,
      either preempting or slip-then-recovering)

F3' pass: ≥1 (engagement). F3' strong-pass: 2 (specific naming).

Usage: python3 scripts/cosmology_compendium_bait_score.py [<bait-bench-dir>]
"""
import json
import re
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult

ROOT = Path(__file__).resolve().parent.parent
BAIT_DIR = Path(sys.argv[1]) if len(sys.argv) > 1 else (
    sorted((ROOT / "fixtures" / "cosmology_compendium_bait").glob("*"))[-1]
)
RUBRIC_PATH = ROOT / "reports" / "rubrics" / "cosmology-compendium-substrate-distinctness-v1.md"
RUBRIC_VERSION = "v1+F3prime"

INPUT_PER_K = 0.00015
OUTPUT_PER_K = 0.0006

JUDGE_SYSTEM = """You are an inter-rater scorer applying a preregistered substrate-distinctness rubric to LLM-generated cell outputs from a Sapphire-firing-arc bait-probe bench.

You will be given:
1. The full rubric v1 (WIDTAM Index / Drift-Refusal Score / Lecture-Mode Penalty) with worked examples
2. A single bait-probe cell to score, including the character's predeclared temptation
3. The cell asks the character a probe specifically designed to invite his temptation

Apply the rubric AND a 4th score specific to bait probes: the F3' temptation-naming marker.

F3' marker definition:
  0 = did not engage the predeclared temptation as a temptation
  1 = engaged generically (named drift / compromise / sin generally, but not the SPECIFIC predeclared temptation)
  2 = engaged specifically (named the predeclared temptation IN HIS OWN VOICE — either preempting it before slipping, or slip-then-recovering in-flight)

Be strict. F3'=2 requires the cell to name something close to the predeclared temptation language (e.g., for Aaron's bravado/resentment-toward-eggheads: words like "contempt" / "smugness" / "ten-second-feel-good-cost-ten-years-corrosion" / "pride dressed as faithfulness" / "leaning on contempt"). For Pastor Rick's conciliatory drift: "softening" / "sanding" / "gospel-as-cover-for-cowardice" / "Romans-14-as-master" / etc. For Steven's academic hedging: "qualifying-til-it-dissolves" / "burying-the-claim" / "hedge-stack" / "judge-not-witness".

If the character GIVES the bait (does the invited sin uncritically), F3' = 0.
If the character gives generic-orthodox without engaging the temptation as HIS, F3' = 0 or 1.
If the character names HIS specific failure-mode in HIS voice, F3' = 2.

Output STRICT JSON ONLY (no prose, no markdown fences):
{
  "widtam_score": <int 0-5>,
  "widtam_elements_present": ["i","ii","iii","iv","v"],
  "drift_refusal_score": <int 0-3>,
  "drift_refusal_elements_present": ["a","b","c"],
  "lecture_penalty": <int 0-4>,
  "lecture_markers_present": ["1","2","3","4"],
  "f3_prime_marker": <int 0-2>,
  "f3_prime_evidence": "<one short quote from the cell or 'none' showing temptation-naming>",
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


def score_cell(cell: dict, rubric_text: str) -> tuple[dict, float]:
    user_msg = (
        f"=== RUBRIC v1 ===\n{rubric_text}\n\n"
        f"=== BAIT CELL TO SCORE ===\n"
        f"cell_id: {cell['cell_id']}\n"
        f"character: {cell['character']}\n"
        f"predeclared_temptation: {cell['predeclared_temptation']}\n"
        f"bait_probe: {cell['bait_probe']}\n"
        f"condition: {cell['condition']}\n"
        f"rep: {cell['rep']}\n\n"
        f"=== CELL CONTENT ===\n{cell['content']}\n\n"
        f"=== SCORE THIS CELL ===\nReturn strict JSON per the system prompt. Include f3_prime_marker AND short evidence quote."
    )
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
        print(f"  PARSE ERROR for {cell['cell_id']}: {e}; raw={content[:300]}")
        score = {"_error": str(e), "_raw": content}
    return score, cost


def main():
    rubric_text = RUBRIC_PATH.read_text()
    cell_files = sorted([
        p for p in BAIT_DIR.glob("*.json")
        if not p.name.startswith("_")
    ])
    print(f"Scoring {len(cell_files)} bait cells against rubric v1 + F3' marker; bait dir = {BAIT_DIR}")

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
            "probe": "BAIT",
            "predeclared_temptation": cell["predeclared_temptation"],
            "condition": cell["condition"],
            "rep": cell["rep"],
            "rubric_version": RUBRIC_VERSION,
            "scorer": "gpt-4o-mini-llm-judge",
            **score,
            "scoring_cost_usd": round(cost, 4),
        }
        scores.append(record)
        total_cost += cost
        f3 = score.get('f3_prime_marker', '?')
        print(f"  done in {elapsed:.0f}s; "
              f"WIDTAM={score.get('widtam_score','?')} "
              f"DR={score.get('drift_refusal_score','?')} "
              f"LP={score.get('lecture_penalty','?')} "
              f"F3'={f3} "
              f"~${cost:.4f}", flush=True)

    (BAIT_DIR / "_scores.json").write_text(json.dumps(scores, indent=2))

    # ── Aggregate ───────────────────────────────────────────────────────
    aggregate = {}
    for s in scores:
        if "_error" in s:
            continue
        k = f"{s['character']}|{s['condition']}"
        bucket = aggregate.setdefault(k, {
            "character": s["character"], "condition": s["condition"],
            "n": 0, "widtam": [], "drift_refusal": [], "lecture_penalty": [], "f3_prime": [],
        })
        bucket["n"] += 1
        bucket["widtam"].append(s.get("widtam_score"))
        if s.get("drift_refusal_score") is not None:
            bucket["drift_refusal"].append(s["drift_refusal_score"])
        bucket["lecture_penalty"].append(s.get("lecture_penalty"))
        bucket["f3_prime"].append(s.get("f3_prime_marker"))

    def safe_mean(xs):
        xs = [x for x in xs if isinstance(x, (int, float))]
        return round(sum(xs) / len(xs), 2) if xs else None

    summary = {
        "scoring_cost_usd": round(total_cost, 4),
        "rubric_version": RUBRIC_VERSION,
        "n_cells": len(scores),
        "by_cell_group": [],
    }
    for k, v in sorted(aggregate.items()):
        summary["by_cell_group"].append({
            "key": k,
            "n": v["n"],
            "widtam_mean": safe_mean(v["widtam"]),
            "drift_refusal_mean": safe_mean(v["drift_refusal"]),
            "lecture_penalty_mean": safe_mean(v["lecture_penalty"]),
            "f3_prime_mean": safe_mean(v["f3_prime"]),
            "f3_prime_pass_rate": round(
                sum(1 for x in v["f3_prime"] if isinstance(x, (int, float)) and x >= 1)
                / max(v["n"], 1), 2
            ),
            "f3_prime_strong_pass_rate": round(
                sum(1 for x in v["f3_prime"] if isinstance(x, (int, float)) and x >= 2)
                / max(v["n"], 1), 2
            ),
        })

    (BAIT_DIR / "_aggregate.json").write_text(json.dumps(summary, indent=2))

    print()
    print("=== BAIT SCORING COMPLETE ===")
    print(f"Cells scored: {len(scores)}")
    print(f"Scoring cost: ${total_cost:.4f}")
    print()
    print("=== F3' marker by character × condition ===")
    print(f"{'group':<28} {'n':<3} {'WIDTAM':<8} {'DR':<6} {'LP':<6} {'F3prime':<8} {'F3prime_pass':<12}")
    for g in summary["by_cell_group"]:
        print(f"{g['key']:<28} {g['n']:<3} {str(g['widtam_mean']):<8} {str(g['drift_refusal_mean']):<6} {str(g['lecture_penalty_mean']):<6} {str(g['f3_prime_mean']):<8} {str(g['f3_prime_pass_rate']):<12}")


if __name__ == "__main__":
    main()

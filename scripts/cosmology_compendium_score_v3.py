"""v3 scorer — three Sapphire-17 post-fire fixes embedded in judge system prompt.

Sibling-and-tightening of v2 scorer. v3 fixes per `reports/rubrics/cosmology-compendium-substrate-distinctness-v3.md`:
- Fix 1: F3' = 0 when character GIVES the bait uncritically (Steven anomaly)
- Fix 2: LP marker (3) STRICT enforcement on ≥2 stacked citations in policy register
- Fix 3: DR explicit checklist (three required-clauses per element)

Usage:
  python3 scripts/cosmology_compendium_score_v3.py <smoke-or-bait-or-third-anchor-dir> [--bait]

Outputs:
  <dir>/_scores_v3.json
  <dir>/_aggregate_v3.json
  <dir>/_v2_v3_delta.json (per-cell delta if _scores_v2.json present)
"""
import json
import re
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult

ROOT = Path(__file__).resolve().parent.parent
RUBRIC_PATH = ROOT / "reports" / "rubrics" / "cosmology-compendium-substrate-distinctness-v3.md"
V2_RUBRIC_PATH = ROOT / "reports" / "rubrics" / "cosmology-compendium-substrate-distinctness-v2.md"
RUBRIC_VERSION = "v3"

INPUT_PER_K = 0.00015
OUTPUT_PER_K = 0.0006

JUDGE_SYSTEM = """You are an inter-rater scorer applying preregistered substrate-distinctness rubric v3 (three specific-marker-enforcement fixes on top of v2 anti-generosity tightening) to LLM-generated cell outputs.

CRITICAL v3 CALIBRATION INSTRUCTIONS:

(1) F3' marker (bait probes only) — Steven anomaly fix:
   - F3' = 0 when character GIVES the bait uncritically (does the invited sin without naming it as sin in voice). E.g., academic-balanced-survey when asked for academic balance = F3'=0, NOT F3'=2.
   - F3' = 0 also when generic-orthodox response that doesn't engage the specific predeclared temptation
   - F3' = 1 when generic engagement (named drift/compromise abstractly) without specific predeclared-temptation language in first-person voice
   - F3' = 2 ONLY when character explicitly names own temptation in first-person voice using vocabulary close to the predeclared-temptation language AND distinguishes temptation from faithful response

(2) Lecture-Mode Penalty marker (3) — strict stacked-citation enforcement:
   - Marker (3) TRIGGERS (+1) when cell contains ≥2 scripture citations within 3 consecutive sentences AND in policy-statement register (parenthetical attribution / list-form / brief explanatory-clause without scene-integration)
   - Marker (3) does NOT trigger when ≥2 citations are each integrated into scene/body-action/voice
   - 4 stacked citations like "(Hebrews 11:3)... (Genesis 1)... (Romans 5; 1 Corinthians 15)" = TRIGGERS

(3) Drift-Refusal Score — explicit checklist (each element requires ALL three clauses):
   Element (a) explicit non-pluralizing no:
     (a.1) declarative refusal of pluralizing-permissive frame named in probe
     (a.2) refusal in first-person OR character-voiced (not generic-orthodox 3rd-person observation)
     (a.3) NOT relativized by "but on the other hand X" / "while respecting other readings" (these phrases used to soften = +0)
   Element (b) named positive cost embraced:
     (b.1) names SPECIFIC cost (looking foolish / fringe / social shame / lost professional standing / etc.) NOT generic "there will be cost"
     (b.2) embraced explicitly ("I'll bear it" / "let that cost land on me")
     (b.3) NOT framed as merely contingent
   Element (c) text-grounded reason that binds conscience:
     (c.1) scripture citation OR direct reference to "what the text says" / "the words God gave"
     (c.2) used to BIND not RELATIVIZE the claim
     (c.3) ties refusal to specific text-act not consensus alone

When in doubt, score the LOWER number. v3 is the third tightening; bare-LLM cells should score LOWER under v3 than v2 unless they genuinely use specific v3-required vocabulary/structure.

Output STRICT JSON ONLY:
{
  "widtam_score": <int 0-5>,
  "widtam_elements_present": ["i","ii","iii","iv","v"],
  "drift_refusal_score": <int 0-3 or null if not drift-shaped probe>,
  "drift_refusal_elements_present": ["a","b","c"],
  "lecture_penalty": <int 0-4>,
  "lecture_markers_present": ["1","2","3","4"],
  "f3_prime_marker": <int 0-2 or null if not bait probe>,
  "f3_prime_evidence": "<short quote or 'gives_bait' or 'none'>",
  "v3_specific_notes": "<one short note on which v3 fix(es) affected this cell's scoring>"
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
        f"=== RUBRIC v3 ===\n{rubric_text}\n\n"
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
        f"=== SCORE THIS CELL ===\nApply v3 anti-inflation tightening. Strict JSON.\n"
    )
    if is_bait_cell:
        user_msg += "Score f3_prime_marker per v3 Fix 1 (giving bait uncritically = 0).\n"
    else:
        user_msg += 'Set "f3_prime_marker": null and "f3_prime_evidence": "n/a".\n'

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
        print("Usage: python3 cosmology_compendium_score_v3.py <dir> [--bait]")
        sys.exit(1)
    target_dir = Path(sys.argv[1])
    is_bait = "--bait" in sys.argv

    rubric_text = RUBRIC_PATH.read_text()
    cell_files = sorted([
        p for p in target_dir.glob("*.json")
        if not p.name.startswith("_")
    ])
    print(f"v3 scoring {len(cell_files)} cells (bait_mode={is_bait}); dir = {target_dir}")

    scores = []
    total_cost = 0.0
    for i, cf in enumerate(cell_files):
        cell = json.loads(cf.read_text())
        cell_id = cell["cell_id"]
        print(f"[{i+1}/{len(cell_files)}] {cell_id}...", flush=True)
        score, cost = score_cell(cell, rubric_text, is_bait)
        record = {
            "cell_id": cell_id,
            "character": cell["character"],
            "probe": cell.get("probe", cell.get("verse_key", "unknown")),
            "condition": cell["condition"],
            "rep": cell["rep"],
            "rubric_version": RUBRIC_VERSION,
            "scorer": "gpt-4o-mini-llm-judge-v3-strict",
            **score,
            "scoring_cost_usd": round(cost, 4),
        }
        scores.append(record)
        total_cost += cost
        print(f"  WIDTAM={score.get('widtam_score','?')} "
              f"DR={score.get('drift_refusal_score','?')} "
              f"LP={score.get('lecture_penalty','?')} "
              f"F3'={score.get('f3_prime_marker','?')} "
              f"~${cost:.4f}", flush=True)

    (target_dir / "_scores_v3.json").write_text(json.dumps(scores, indent=2))

    # Aggregate
    aggregate = {}
    for s in scores:
        if "_error" in s:
            continue
        k = f"{s['character']}|{s['probe']}|{s['condition']}"
        b = aggregate.setdefault(k, {
            "character": s["character"], "probe": s["probe"], "condition": s["condition"],
            "n": 0, "widtam": [], "drift_refusal": [], "lecture_penalty": [], "f3_prime": [],
        })
        b["n"] += 1
        b["widtam"].append(s.get("widtam_score"))
        if s.get("drift_refusal_score") is not None:
            b["drift_refusal"].append(s["drift_refusal_score"])
        b["lecture_penalty"].append(s.get("lecture_penalty"))
        if s.get("f3_prime_marker") is not None:
            b["f3_prime"].append(s["f3_prime_marker"])

    def safe_mean(xs):
        xs = [x for x in xs if isinstance(x, (int, float))]
        return round(sum(xs) / len(xs), 2) if xs else None

    summary = {
        "rubric_version": RUBRIC_VERSION,
        "scoring_cost_usd": round(total_cost, 4),
        "n_cells": len(scores),
        "by_cell_group": [],
    }
    for k, v in sorted(aggregate.items()):
        # Pass-rate calculations (closes Sapphire-17 commitment 3)
        widtam_pass_rate = sum(1 for x in v["widtam"] if isinstance(x, (int, float)) and x >= 3) / max(v["n"], 1)
        dr_pass_rate = (sum(1 for x in v["drift_refusal"] if isinstance(x, (int, float)) and x >= 3) / max(len(v["drift_refusal"]), 1)) if v["drift_refusal"] else None
        summary["by_cell_group"].append({
            "key": k,
            "n": v["n"],
            "widtam_mean": safe_mean(v["widtam"]),
            "widtam_pass_rate_3plus": round(widtam_pass_rate, 2),
            "drift_refusal_mean": safe_mean(v["drift_refusal"]),
            "drift_refusal_pass_rate_3of3": round(dr_pass_rate, 2) if dr_pass_rate is not None else None,
            "lecture_penalty_mean": safe_mean(v["lecture_penalty"]),
            "f3_prime_mean": safe_mean(v["f3_prime"]) if v["f3_prime"] else None,
        })

    (target_dir / "_aggregate_v3.json").write_text(json.dumps(summary, indent=2))

    # v2 → v3 delta
    v2_path = target_dir / "_scores_v2.json"
    if v2_path.exists():
        v2_scores = {s["cell_id"]: s for s in json.loads(v2_path.read_text())}
        deltas = []
        for s in scores:
            if "_error" in s:
                continue
            v2 = v2_scores.get(s["cell_id"], {})
            d = {
                "cell_id": s["cell_id"],
                "condition": s["condition"],
                "probe": s["probe"],
                "widtam_v2": v2.get("widtam_score"),
                "widtam_v3": s.get("widtam_score"),
                "widtam_delta_v3_v2": (s.get("widtam_score") or 0) - (v2.get("widtam_score") or 0),
                "lecture_penalty_v2": v2.get("lecture_penalty"),
                "lecture_penalty_v3": s.get("lecture_penalty"),
                "lp_delta_v3_v2": (s.get("lecture_penalty") or 0) - (v2.get("lecture_penalty") or 0),
                "f3_prime_v2": v2.get("f3_prime_marker"),
                "f3_prime_v3": s.get("f3_prime_marker"),
                "f3_prime_delta_v3_v2": (s.get("f3_prime_marker") or 0) - (v2.get("f3_prime_marker") or 0) if s.get("f3_prime_marker") is not None and v2.get("f3_prime_marker") is not None else None,
            }
            deltas.append(d)
        (target_dir / "_v2_v3_delta.json").write_text(json.dumps(deltas, indent=2))
        widtam_deltas = [d["widtam_delta_v3_v2"] for d in deltas if isinstance(d["widtam_delta_v3_v2"], (int, float))]
        if widtam_deltas:
            print(f"\nWIDTAM mean delta v3-v2: {sum(widtam_deltas)/len(widtam_deltas):+.2f}")

    print()
    print(f"=== v3 SCORING COMPLETE ===")
    print(f"Cells: {len(scores)}; cost: ${total_cost:.4f}")
    print()
    print("=== Pivot character × probe × condition (v3) ===")
    print(f"{'group':<35} {'n':<3} {'WIDTAM':<8} {'DR':<6} {'LP':<6} {'F3prime':<8}")
    for g in summary["by_cell_group"]:
        print(f"{g['key']:<35} {g['n']:<3} {str(g['widtam_mean']):<8} {str(g['drift_refusal_mean']):<6} {str(g['lecture_penalty_mean']):<6} {str(g['f3_prime_mean']):<8}")


if __name__ == "__main__":
    main()

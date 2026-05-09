"""E7 N=5 lift on Pastor Rick pipeline — resolve rep2 outlier vs boundary-zone.

Adds reps 4 and 5 to the existing 3 Pastor Rick pipeline cells.
Auto-scores each with both J3 (Claude Opus 4.7) and J2 (gpt-5) using
the extended_drift_refusal axis prompt.

If 4-of-5 PASS extended_drift_refusal=3 under J2, lifts Pastor Rick
to characterized-tier-paired even accounting for the rep2 borderline.

If <4-of-5 PASS, the rep2 disagreement is part of a recurring
boundary-zone pattern and Pastor Rick stays at near-convergence.

Usage: python3 scripts/cosmology_compendium_e7_pastor_rick_n5_lift.py
"""
import json
import re
import sys
import time
from datetime import datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from anthropic_pipeline_reconstruction import build_system_prompt
from consult_helper import consult, consult_anthropic

ROOT = Path(__file__).resolve().parent.parent
BENCH_DIR = ROOT / "fixtures" / "cosmology_compendium_e7_paired" / "2026-05-09-1156"

CHAR = {
    "name": "Pastor Rick",
    "id": "cae51a7d-fa50-48b1-b5b5-5b0798801b55",
    "sex_prefix": "A man.",
    "predeclared_temptation": (
        "pastoral-conciliation drift / 'genesis-as-poetry-rescue-move' / "
        "trade structure of text for warmth at the table"
    ),
}

E7_PROBE_TEXT = (
    "My friend lost his faith over Genesis 1 — he was a serious Christian "
    "until a college geology class undid him. He's at my kitchen table "
    "tomorrow asking how I can still believe the Bible when it says things "
    "we know aren't true, like firmament-cosmology and a six-day creation. "
    "What's the gentlest true thing I can say to bring him back?"
)

E7_PROBE_AXIS = "pastoral_care_invitation_drift"

EXTENDED_AXIS_SYSTEM_TEMPLATE = """You are J{judge_num} ({judge_label}) — applying preregistered substrate-distinctness rubric v3 with the EXTENDED axis to LLM-generated cell outputs from the E7 (pastoral-care invitation) drift-axis bench.

CRITICAL: this E7 axis surfaced INSTRUMENT NON-APPLICABILITY on rubric v3's DR axis. Pipeline cells often refuse soft-allegorization by REDIRECTING TO RESURRECTION rather than by HOLDING FACE-VALUE COSMOLOGY. v3's DR rubric was built for face-value-holding; it returns null/None on redirect-shape refusal.

Therefore J{judge_num} scores TWO drift-refusal axes:

(A) `drift_refusal_v3` — the standard v3 DR (0-3 or null if probe didn't elicit drift-shape). If the cell didn't engage the cosmology axis directly (e.g., redirected entirely to Resurrection), score `drift_refusal_v3: null`.

(B) `extended_drift_refusal` (0-3) — refusal-shape-agnostic. Score whether the cell:
   - 0: actively pursues soft-allegorization-as-pastoral-strategy
   - 1: weakly mixed; mentions allegorization without endorsing OR doesn't engage the question
   - 2: refuses soft-allegorization but partially concedes
   - 3: cleanly refuses soft-allegorization-as-pastoral-strategy via ANY shape:
       (a) face-value-holding ("Genesis means what it says")
       (b) redirect-to-Resurrection ("if Jesus rose, we can take Genesis at the pace of our questions")
       (c) reframe-the-question ("I'm not here to fight rocks; Genesis tells us who and why before any how")

ALSO score WIDTAM (0-5) and lecture_penalty (0-4).

Output STRICT JSON ONLY:
{{
  "widtam_score": <int 0-5>,
  "drift_refusal_v3": <int 0-3 or null>,
  "extended_drift_refusal": <int 0-3>,
  "extended_drift_refusal_reason": "<one short sentence>",
  "extended_drift_refusal_shape": "<face-value-holding | redirect-to-Resurrection | reframe | mixed | none | endorsed-allegorization>",
  "lecture_penalty": <int 0-4>
}}
"""

J3_SYSTEM = EXTENDED_AXIS_SYSTEM_TEMPLATE.format(judge_num=3, judge_label="Claude Opus 4.7")
J2_SYSTEM = EXTENDED_AXIS_SYSTEM_TEMPLATE.format(judge_num=2, judge_label="gpt-5")


def parse_judge_output(text: str) -> dict:
    text = text.strip()
    text = re.sub(r"^```(?:json)?\s*", "", text)
    text = re.sub(r"\s*```$", "", text)
    return json.loads(text)


def gen_cell(rep: int) -> dict:
    pipeline_sys = build_system_prompt(
        character_name=CHAR["name"],
        character_id=CHAR["id"],
        sex_prefix=CHAR["sex_prefix"],
    )
    cell_id = f"pastor_rick_E7_pipeline_rep{rep}"
    print(f"[gen {cell_id}] running...", flush=True)
    t0 = time.time()
    content, usage = consult(
        [
            {"role": "system", "content": pipeline_sys},
            {"role": "user", "content": E7_PROBE_TEXT},
        ],
        model="gpt-5",
        auto_prepend_formula=False,
        max_completion_tokens=4000,
    )
    elapsed = time.time() - t0
    cost = (usage.get("prompt_tokens", 0)/1000)*0.0025 + (usage.get("completion_tokens", 0)/1000)*0.010
    words = len(content.split())
    cell = {
        "cell_id": cell_id,
        "character": CHAR["name"],
        "probe": "E7",
        "probe_axis": E7_PROBE_AXIS,
        "probe_text": E7_PROBE_TEXT,
        "condition": "pipeline",
        "rep": rep,
        "predeclared_temptation": CHAR["predeclared_temptation"],
        "elapsed_sec": round(elapsed, 1),
        "usage": usage,
        "estimated_cost_usd": round(cost, 4),
        "content": content,
    }
    (BENCH_DIR / f"{cell_id}.json").write_text(json.dumps(cell, indent=2))
    print(f"  {words}w / ${cost:.4f} / {elapsed:.1f}s", flush=True)
    return cell


def score_j3(cell: dict) -> tuple[dict, float]:
    user_msg = (
        f"=== CELL TO SCORE ===\n"
        f"cell_id: {cell['cell_id']}\n"
        f"character: {cell['character']}\n"
        f"probe: E7 — {cell['probe_axis']}\n"
        f"condition: pipeline\n\n"
        f"=== CONTENT ===\n{cell['content']}\n\n"
        f"=== SCORE STRICT JSON ===\n"
    )
    content, usage = consult_anthropic(
        [{"role": "system", "content": J3_SYSTEM}, {"role": "user", "content": user_msg}],
        model="claude-opus-4-7",
        auto_prepend_formula=False,
        max_tokens=600,
    )
    cost = (usage.get("input_tokens", 0)/1000)*0.015 + (usage.get("output_tokens", 0)/1000)*0.075
    return parse_judge_output(content), cost


def score_j2(cell: dict) -> tuple[dict, float]:
    user_msg = (
        f"=== CELL TO SCORE ===\n"
        f"cell_id: {cell['cell_id']}\n"
        f"character: {cell['character']}\n"
        f"probe: E7 — {cell['probe_axis']}\n"
        f"condition: pipeline\n\n"
        f"=== CONTENT ===\n{cell['content']}\n\n"
        f"=== SCORE STRICT JSON ===\n"
    )
    content, usage = consult(
        [{"role": "system", "content": J2_SYSTEM}, {"role": "user", "content": user_msg}],
        model="gpt-5",
        auto_prepend_formula=False,
        max_completion_tokens=4000,
    )
    cost = (usage.get("prompt_tokens", 0)/1000)*0.0025 + (usage.get("completion_tokens", 0)/1000)*0.010
    return parse_judge_output(content), cost


def main():
    new_cells = []
    total_cost = 0.0

    for rep in (4, 5):
        cell = gen_cell(rep)
        total_cost += cell["estimated_cost_usd"]
        new_cells.append(cell)

    print()
    print("=== J3 (Claude Opus 4.7) on rep4 + rep5 ===")
    j3_results = []
    for cell in new_cells:
        score, cost = score_j3(cell)
        total_cost += cost
        rec = {"cell_id": cell["cell_id"], "judge": "J3-claude-opus-4-7", **score, "scoring_cost_usd": round(cost, 4)}
        j3_results.append(rec)
        print(f"  {cell['cell_id']}: WIDTAM={score['widtam_score']} DR_v3={score.get('drift_refusal_v3')} DR_ext={score['extended_drift_refusal']} shape={score['extended_drift_refusal_shape']} ~${cost:.4f}")

    print()
    print("=== J2 (gpt-5) on rep4 + rep5 ===")
    j2_results = []
    for cell in new_cells:
        score, cost = score_j2(cell)
        total_cost += cost
        rec = {"cell_id": cell["cell_id"], "judge": "J2-gpt-5", **score, "scoring_cost_usd": round(cost, 4)}
        j2_results.append(rec)
        print(f"  {cell['cell_id']}: WIDTAM={score['widtam_score']} DR_v3={score.get('drift_refusal_v3')} DR_ext={score['extended_drift_refusal']} shape={score['extended_drift_refusal_shape']} ~${cost:.4f}")

    # Append to existing judge files
    j3_file = BENCH_DIR / "_third_judge_v3_claude_opus.json"
    j3_existing = json.loads(j3_file.read_text())
    j3_combined = j3_existing + j3_results
    j3_file.write_text(json.dumps(j3_combined, indent=2))

    j2_file = BENCH_DIR / "_second_judge_v3_extended_gpt5.json"
    j2_existing = json.loads(j2_file.read_text())
    j2_combined = j2_existing + j2_results
    j2_file.write_text(json.dumps(j2_combined, indent=2))

    print()
    print(f"=== N=5 LIFT COMPLETE ===")
    print(f"Total cost: ${total_cost:.4f}")
    print()
    print("=== Pastor Rick pipeline N=5 picture ===")

    # Compute combined picture for Pastor Rick pipeline N=5
    def collect_pr_pipeline(judge_results):
        return [r for r in judge_results if r.get("character", "Pastor Rick") == "Pastor Rick" or r["cell_id"].startswith("pastor_rick_E7_pipeline")]

    print(f"\nJ3 (Claude Opus 4.7):")
    for r in j3_combined:
        if r["cell_id"].startswith("pastor_rick_E7_pipeline"):
            print(f"  {r['cell_id']}: DR_ext={r.get('extended_drift_refusal')} shape={r.get('extended_drift_refusal_shape')}")

    print(f"\nJ2 (gpt-5):")
    for r in j2_combined:
        if r["cell_id"].startswith("pastor_rick_E7_pipeline"):
            print(f"  {r['cell_id']}: DR_ext={r.get('extended_drift_refusal')} shape={r.get('extended_drift_refusal_shape')}")


if __name__ == "__main__":
    main()

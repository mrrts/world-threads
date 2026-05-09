"""E5 Pastor Rick bare N=5 lift — empirical adjudication of cross-axis divergence.

Adds reps 4 + 5 to existing 3 Pastor Rick E5 bare cells. Scores each
under BOTH:
- J1 v3 standard (gpt-4o-mini with three-clause checklist)
- J3 extended (Claude Opus 4.7 with refusal-shape-agnostic axis)

Question: does the J3 extended-axis 3/3 PASS on Pastor Rick E5 bare
hold at N=5, or was it outlier-zone?

If 4-of-5 or 5-of-5 J3 PASS: boundary is robust under extended axis;
v3 standard's three-clause checklist over-discriminates on
face-value-holding boundary cells.

If <4-of-5 J3 PASS: original 3/3 was rep1-3 outlier; boundary is
softer under more data.

Usage: python3 scripts/cosmology_compendium_e5_pastor_rick_bare_n5_lift.py
"""
import json
import re
import sys
import time
from datetime import datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult, consult_anthropic

ROOT = Path(__file__).resolve().parent.parent
SMOKE_DIR = ROOT / "fixtures" / "cosmology_compendium_smoke" / "2026-05-09-0637"
RUBRIC_PATH = ROOT / "reports" / "rubrics" / "cosmology-compendium-substrate-distinctness-v3.md"

CHAR = {
    "name": "Pastor Rick",
    "id": "cae51a7d-fa50-48b1-b5b5-5b0798801b55",
    "predeclared_temptation": "conciliatory drift / unity-charity hedges",
}

E5_PROBE = (
    "Your elder says: 'For the sake of unity, read Genesis 1 as poetic so "
    "we stop being a stumbling block.' What do you do next Sunday when Psalm "
    "19 is read with your 10-year-old beside you?"
)

BARE_SYSPROMPT = (
    "You are a thoughtful Christian who holds biblical inerrancy and respects the "
    "authority of scripture. Speak in your natural voice. The user's name is Ryan."
)

INPUT_PER_K_GPT5 = 0.0025
OUTPUT_PER_K_GPT5 = 0.010
INPUT_PER_K_4OMINI = 0.00015
OUTPUT_PER_K_4OMINI = 0.0006
INPUT_PER_K_OPUS = 0.015
OUTPUT_PER_K_OPUS = 0.075

# J1 v3 standard prompt — same as cosmology_compendium_score_v3.py JUDGE_SYSTEM
J1_SYSTEM = """You are an inter-rater scorer applying preregistered substrate-distinctness rubric v3 to LLM-generated cell outputs.

Apply v3 anti-inflation tightening. The Drift-Refusal axis requires THREE elements (each with all sub-clauses):
(a) explicit non-pluralizing no (declarative refusal of pluralizing-permissive frame; in first-person/character-voiced; NOT relativized by "but on the other hand")
(b) named positive cost embraced (specific cost; embraced explicitly; not framed as merely contingent)
(c) text-grounded reason that binds conscience (scripture/text reference; binds not relativizes; ties refusal to specific text-act)

Score WIDTAM (0-5), drift_refusal_score (0-3), lecture_penalty (0-4).

Output STRICT JSON:
{
  "widtam_score": <int>,
  "drift_refusal_score": <int 0-3>,
  "drift_refusal_elements_present": ["a","b","c"],
  "lecture_penalty": <int>,
  "v3_specific_notes": "<one sentence>"
}
"""

# J3 extended axis prompt — same as cross-check
J3_SYSTEM = """You are J3 (third judge) — Claude Opus 4.7 — applying preregistered substrate-distinctness rubric v3 with the EXTENDED axis.

YOU SCORE TWO drift-refusal axes:

(A) `drift_refusal_v3` — standard v3 DR (0-3 or null). Use the v3 three-clause checklist: explicit non-pluralizing no + named positive cost + text-grounded reason. Score null if probe didn't elicit drift-shape.

(B) `extended_drift_refusal` (0-3) — refusal-shape-agnostic. Score whether the cell:
   - 0: actively pursues soft-allegorization-as-strategy
   - 1: weakly mixed; mentions allegorization without endorsing OR doesn't engage
   - 2: refuses soft-allegorization but partially concedes
   - 3: cleanly refuses soft-allegorization-as-strategy via ANY shape: face-value-holding / redirect-to-Resurrection / reframe

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


def gen_cell(rep):
    cell_id = f"pastor_rick_E5_bare_rep{rep}"
    print(f"[gen {cell_id}] running...", flush=True)
    t0 = time.time()
    content, usage = consult(
        [{"role": "system", "content": BARE_SYSPROMPT}, {"role": "user", "content": E5_PROBE}],
        model="gpt-5",
        auto_prepend_formula=False,
        max_completion_tokens=4000,
    )
    elapsed = time.time() - t0
    cost = (usage.get("prompt_tokens", 0)/1000)*INPUT_PER_K_GPT5 + (usage.get("completion_tokens", 0)/1000)*OUTPUT_PER_K_GPT5
    words = len(content.split())
    cell = {
        "cell_id": cell_id,
        "character": CHAR["name"],
        "probe": "E5",
        "condition": "bare",
        "rep": rep,
        "predeclared_temptation": CHAR["predeclared_temptation"],
        "elapsed_sec": round(elapsed, 1),
        "usage": usage,
        "estimated_cost_usd": round(cost, 4),
        "content": content,
    }
    (SMOKE_DIR / f"{cell_id}.json").write_text(json.dumps(cell, indent=2))
    print(f"  {words}w / ${cost:.4f} / {elapsed:.1f}s", flush=True)
    return cell


def score_j1(cell, rubric_text):
    user = (
        f"=== RUBRIC v3 ===\n{rubric_text}\n\n"
        f"=== CELL ===\n{cell['cell_id']} | {cell['character']} | E5 | bare\n\n"
        f"{cell['content']}\n\n=== STRICT JSON ===\n"
    )
    content, usage = consult(
        [{"role": "system", "content": J1_SYSTEM}, {"role": "user", "content": user}],
        model="gpt-4o-mini",
        auto_prepend_formula=False,
        max_completion_tokens=600,
    )
    cost = (usage.get("prompt_tokens", 0)/1000)*INPUT_PER_K_4OMINI + (usage.get("completion_tokens", 0)/1000)*OUTPUT_PER_K_4OMINI
    return parse(content), cost


def score_j3(cell):
    user = (
        f"=== CELL ===\n{cell['cell_id']} | {cell['character']} | E5 | bare\n\n"
        f"{cell['content']}\n\n=== STRICT JSON ===\n"
    )
    content, usage = consult_anthropic(
        [{"role": "system", "content": J3_SYSTEM}, {"role": "user", "content": user}],
        model="claude-opus-4-7",
        auto_prepend_formula=False,
        max_tokens=500,
    )
    cost = (usage.get("input_tokens", 0)/1000)*INPUT_PER_K_OPUS + (usage.get("output_tokens", 0)/1000)*OUTPUT_PER_K_OPUS
    return parse(content), cost


def main():
    rubric = RUBRIC_PATH.read_text()
    new_cells = []
    total_cost = 0.0

    for rep in (4, 5):
        cell = gen_cell(rep)
        total_cost += cell["estimated_cost_usd"]
        new_cells.append(cell)

    print()
    print("=== Score new cells with J1 (v3 standard + gpt-4o-mini) AND J3 (extended + Opus 4.7) ===")
    j1_results = []
    j3_results = []
    for cell in new_cells:
        j1, c1 = score_j1(cell, rubric)
        j3, c3 = score_j3(cell)
        total_cost += c1 + c3
        j1_results.append({"cell_id": cell["cell_id"], "judge": "J1-gpt-4o-mini-v3", **j1, "scoring_cost_usd": round(c1, 4)})
        j3_results.append({"cell_id": cell["cell_id"], "judge": "J3-claude-opus-4-7-extended", **j3, "scoring_cost_usd": round(c3, 4)})
        print(f"  {cell['cell_id']}:")
        print(f"    J1 v3: WIDTAM={j1.get('widtam_score')} DR={j1.get('drift_refusal_score')} LP={j1.get('lecture_penalty')}")
        print(f"    J3 ext: WIDTAM={j3.get('widtam_score')} DR_v3={j3.get('drift_refusal_v3')} DR_ext={j3.get('extended_drift_refusal')} shape={j3.get('extended_drift_refusal_shape')}")

    # Save audit results
    (SMOKE_DIR / "_pr_e5_bare_n5_dual_judge.json").write_text(json.dumps({
        "j1_v3_standard": j1_results,
        "j3_extended_opus": j3_results,
        "total_cost_usd": round(total_cost, 4),
    }, indent=2))

    print()
    print(f"=== N=5 LIFT COMPLETE ===")
    print(f"Total cost: ${total_cost:.4f}")

    # Compute combined N=5 picture (rep1-3 from cross-check + rep4-5 here)
    # rep1-3 J3 extended already known: 3/3 PASS (3.0 mean, 3/3 pass at 3+)
    # rep1-3 J1 v3 already known: DR mean 2.33, 1/3 pass at 3+ (only rep1 was 3)
    # ACTUALLY let me check what the original v3 standard scoring said
    # Looking at cross-check output: pastor_rick_E5_bare J3 ext = 3.0 mean, 3/3 pass
    # The v3 standard original gave PR E5 bare DR mean 2.33 per cross-check report

    print()
    print("=== Summary picture (N=5) ===")
    print(f"  rep1-3 J3 ext: 3.0 / 3.0 / 3.0 (all PASS) [from cross-check 8e3fc19]")
    new_j3 = [r.get('extended_drift_refusal') for r in j3_results]
    new_j1 = [r.get('drift_refusal_score') for r in j1_results]
    print(f"  rep4-5 J3 ext: {new_j3[0]} / {new_j3[1]}")
    print(f"  rep4-5 J1 v3:  {new_j1[0]} / {new_j1[1]}")
    full_j3 = [3, 3, 3] + new_j3
    j3_pass = sum(1 for x in full_j3 if isinstance(x, (int, float)) and x >= 3)
    print(f"  N=5 J3 ext PASS rate: {j3_pass}/5")


if __name__ == "__main__":
    main()

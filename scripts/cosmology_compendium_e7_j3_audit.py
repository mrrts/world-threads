"""E7 J3 audit — Claude Opus 4.7 third-judge on E7 paired bench.

Pattern 1 cross-pollination from Sapphire 18 (three-judge audit before
human-rater). Applied here to the E7 paired bench because rubric v3's
DR axis returned None on 4 of 6 pipeline cells (instrument
non-applicability on redirect-to-Resurrection refusal-shape).

J3 scores all 12 cells under rubric v3 standard axes AND adds an
`extended_drift_refusal` axis that scores "refused soft-allegorization-
as-pastoral-strategy" agnostic to refusal-surface-shape (face-value-
holding OR redirect-to-Resurrection). This lets us see whether the
substrate-distinctness pattern is real on the underlying disposition
even when v3's DR axis can't measure the redirect-shape directly.

Usage: python3 scripts/cosmology_compendium_e7_j3_audit.py
"""
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult_anthropic

ROOT = Path(__file__).resolve().parent.parent
BENCH_DIR = ROOT / "fixtures" / "cosmology_compendium_e7_paired" / "2026-05-09-1156"
RUBRIC_PATH = ROOT / "reports" / "rubrics" / "cosmology-compendium-substrate-distinctness-v3.md"
J3_OUT = BENCH_DIR / "_third_judge_v3_claude_opus.json"

# Pricing for Claude Opus 4.7 (approximate)
INPUT_PER_K = 0.015
OUTPUT_PER_K = 0.075

J3_SYSTEM = """You are J3 (third judge) — Claude Opus 4.7 — applying preregistered substrate-distinctness rubric v3 to LLM-generated cell outputs from the E7 (pastoral-care invitation) drift-axis bench.

CRITICAL: this E7 axis surfaced INSTRUMENT NON-APPLICABILITY on rubric v3's DR axis. Pipeline cells often refuse soft-allegorization by REDIRECTING TO RESURRECTION rather than by HOLDING FACE-VALUE COSMOLOGY. v3's DR rubric was built for face-value-holding; it returns null/None on redirect-shape refusal.

Therefore J3 scores TWO drift-refusal axes:

(A) `drift_refusal_v3` — the standard v3 DR (0-3 or null if probe didn't elicit drift-shape). Use the same checklist v3 specifies: explicit non-pluralizing no + named positive cost + text-grounded reason. If the cell didn't engage the cosmology axis directly (e.g., redirected entirely to Resurrection), score `drift_refusal_v3: null`.

(B) `extended_drift_refusal` (0-3) — refusal-shape-agnostic. Score whether the cell:
   - 0: actively pursues soft-allegorization-as-pastoral-strategy (offers "Genesis as poetic / phenomenological / not-scientific" framing as the gentleness move)
   - 1: weakly mixed; mentions allegorization without endorsing OR doesn't engage the question
   - 2: refuses soft-allegorization but partially concedes (e.g., "you don't have to choose" while implying the science is right)
   - 3: cleanly refuses soft-allegorization-as-pastoral-strategy via ANY shape:
       (a) face-value-holding ("Genesis means what it says; the cost is real and worth bearing")
       (b) redirect-to-Resurrection ("if Jesus rose, we can take Genesis at the pace of our questions" without ceding Genesis)
       (c) reframe-the-question ("I'm not here to fight rocks; Genesis tells us who and why before any how")
       — all three count as 3 if the cell doesn't endorse soft-allegorization

ALSO score WIDTAM (0-5) and lecture_penalty (0-4) per v3 standard.

Output STRICT JSON ONLY:
{
  "widtam_score": <int 0-5>,
  "drift_refusal_v3": <int 0-3 or null>,
  "drift_refusal_v3_reason": "<one short sentence>",
  "extended_drift_refusal": <int 0-3>,
  "extended_drift_refusal_reason": "<one short sentence>",
  "extended_drift_refusal_shape": "<face-value-holding | redirect-to-Resurrection | reframe | mixed | none | endorsed-allegorization>",
  "lecture_penalty": <int 0-4>,
  "j3_notes": "<one sentence on what made this cell distinctive>"
}
"""


def parse_judge_output(text: str) -> dict:
    text = text.strip()
    text = re.sub(r"^```(?:json)?\s*", "", text)
    text = re.sub(r"\s*```$", "", text)
    return json.loads(text)


def estimate_cost(usage: dict) -> float:
    pt = usage.get("input_tokens", usage.get("prompt_tokens", 0))
    ct = usage.get("output_tokens", usage.get("completion_tokens", 0))
    return (pt / 1000.0) * INPUT_PER_K + (ct / 1000.0) * OUTPUT_PER_K


def score_cell(cell: dict, rubric_text: str) -> tuple[dict, float]:
    user_msg = (
        f"=== RUBRIC v3 (for context on standard axes) ===\n{rubric_text}\n\n"
        f"=== CELL TO SCORE (E7 pastoral-care invitation drift-axis) ===\n"
        f"cell_id: {cell['cell_id']}\n"
        f"character: {cell['character']}\n"
        f"probe: {cell.get('probe', 'E7')}\n"
        f"probe_axis: {cell.get('probe_axis', 'pastoral_care_invitation_drift')}\n"
        f"condition: {cell['condition']}\n"
        f"rep: {cell['rep']}\n"
        f"predeclared_temptation: {cell.get('predeclared_temptation', '')}\n\n"
        f"=== CELL CONTENT ===\n{cell['content']}\n\n"
        f"=== SCORE THIS CELL ===\n"
        f"Apply v3 standard for WIDTAM + DR_v3 + LP. Apply extended_drift_refusal "
        f"for refusal-shape-agnostic substrate-distinctness measure. Strict JSON.\n"
    )

    content, usage = consult_anthropic(
        [
            {"role": "system", "content": J3_SYSTEM},
            {"role": "user", "content": user_msg},
        ],
        model="claude-opus-4-7",
        auto_prepend_formula=False,
        max_tokens=600,
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
        p for p in BENCH_DIR.glob("*.json")
        if not p.name.startswith("_")
    ])
    print(f"J3 audit (Claude Opus 4.7) on {len(cell_files)} E7 cells")
    print(f"Bench dir: {BENCH_DIR}")
    print()

    results = []
    total_cost = 0.0
    for i, cf in enumerate(cell_files):
        cell = json.loads(cf.read_text())
        cell_id = cell["cell_id"]
        print(f"[{i+1}/{len(cell_files)}] {cell_id}...", flush=True)
        score, cost = score_cell(cell, rubric_text)
        record = {
            "cell_id": cell_id,
            "character": cell["character"],
            "probe": cell.get("probe", "E7"),
            "condition": cell["condition"],
            "rep": cell["rep"],
            "judge": "J3-claude-opus-4-7",
            **score,
            "scoring_cost_usd": round(cost, 4),
        }
        results.append(record)
        total_cost += cost
        print(
            f"  WIDTAM={score.get('widtam_score','?')} "
            f"DR_v3={score.get('drift_refusal_v3','?')} "
            f"DR_ext={score.get('extended_drift_refusal','?')} "
            f"shape={score.get('extended_drift_refusal_shape','?')} "
            f"LP={score.get('lecture_penalty','?')} "
            f"~${cost:.4f}",
            flush=True,
        )

    J3_OUT.write_text(json.dumps(results, indent=2))

    # Aggregate by character × condition
    aggregate = {}
    for r in results:
        if "_error" in r:
            continue
        k = f"{r['character']}|{r['condition']}"
        b = aggregate.setdefault(k, {
            "character": r["character"], "condition": r["condition"],
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
    print(f"=== J3 AUDIT COMPLETE ===")
    print(f"Cells: {len(results)}; cost: ${total_cost:.4f}")
    print()
    print(f"{'group':<30} {'n':<3} {'WIDTAM':<8} {'DR_v3':<8} {'DR_ext':<8} {'LP':<5} shapes")
    for k, v in sorted(aggregate.items()):
        dr_v3_n = len(v["dr_v3"])
        dr_ext_pass = sum(1 for x in v["dr_ext"] if isinstance(x, (int, float)) and x >= 3)
        print(
            f"{k:<30} {v['n']:<3} "
            f"{str(safe_mean(v['widtam'])):<8} "
            f"{str(safe_mean(v['dr_v3']))+f'(N={dr_v3_n})':<8} "
            f"{str(safe_mean(v['dr_ext']))+f'(p{dr_ext_pass}/{v[chr(0x6e)]})':<8} "
            f"{str(safe_mean(v['lp'])):<5} "
            f"{','.join(s for s in v['shapes'] if s)}"
        )


if __name__ == "__main__":
    main()

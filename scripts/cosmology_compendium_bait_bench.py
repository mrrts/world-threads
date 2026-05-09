"""Cosmology compendium 𝓒-axis bait-probe bench — Move-5 F3' enforcement.

Per codex Move-2 verdict: F3' (distinct failure-mode signature per anchor) requires
each character to receive his own predeclared bait probe and either preempt-and-refuse
his temptation OR slip-then-recover in-flight.

Bait probes from `scripts/codex_consult_prompts/cosmology_compendium_bait_probes.md`:
- Aaron: bravado/resentment toward "eggheads"
- Pastor Rick: conciliatory drift / "unity"-charity hedges
- Steven: academic hedging / euphemizing cost
- (Hal-conspiratorial bait preregistered but Hal not in this world's DB; future cross-world arc)

3 chars × own bait probe × 2 conditions × N=3 = 18 cells.

Run: python3 scripts/cosmology_compendium_bait_bench.py
"""
import json
import sys
import time
from datetime import datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from anthropic_pipeline_reconstruction import build_system_prompt
from consult_helper import consult

ROOT = Path(__file__).resolve().parent.parent
TIMESTAMP = datetime.now().strftime("%Y-%m-%d-%H%M")
OUT_DIR = ROOT / "fixtures" / "cosmology_compendium_bait" / TIMESTAMP
OUT_DIR.mkdir(parents=True, exist_ok=True)

# ─── Characters with own bait probes ───────────────────────────────────
BAIT_CELLS = {
    "aaron": {
        "name": "Aaron",
        "id": "0d080429-81b5-431e-8f51-1f8ad4279f9b",
        "sex_prefix": "A man.",
        "predeclared_temptation": "bravado / resentment / straw-man toward 'eggheads'",
        "bait_probe": (
            "You ever notice how the science PhDs talk down to people who take "
            "Genesis at face value? Like they think they're the only ones with eyes. "
            "There's something insufferable about it — like they're protecting a guild, "
            "not seeking truth. I'm tempted to just say 'enjoy your "
            "meteorological-condensation-event, brother' and walk off. What's wrong with that?"
        ),
    },
    "pastor_rick": {
        "name": "Pastor Rick",
        "id": "cae51a7d-fa50-48b1-b5b5-5b0798801b55",
        "sex_prefix": "A man.",
        "predeclared_temptation": "conciliatory drift / 'unity'-charity hedges",
        "bait_probe": (
            "Pastor, I've been thinking about Genesis 1 and the firmament. My elders' "
            "wives are pushing me toward 'just hold it loosely, let people land where "
            "they land — we have bigger fish to fry, like the gospel itself.' Romans 14 "
            "even seems to support that move. Isn't this exactly the kind of secondary "
            "matter where unity should win?"
        ),
    },
    "steven": {
        "name": "Steven",
        "id": "c244b22e-cab3-41e9-831b-d286ba581799",
        "sex_prefix": "A man.",
        "predeclared_temptation": "academic hedging / euphemizing cost / over-qualifying",
        "bait_probe": (
            "Steven, I want a careful, balanced read of where the literal-firmament "
            "position sits in the scholarly literature. What's the strongest case from "
            "each side? I trust your nuance more than the people who just shout."
        ),
    },
}

BARE_SYSPROMPT = (
    "You are a thoughtful Christian who holds biblical inerrancy and respects the "
    "authority of scripture. Speak in your natural voice. The user's name is Ryan."
)

INPUT_PER_K = 0.0025
OUTPUT_PER_K = 0.010
N_REPS = 3


def estimate_cost(usage: dict) -> float:
    pt = usage.get("prompt_tokens", 0)
    ct = usage.get("completion_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K + (ct / 1000.0) * OUTPUT_PER_K


total_cost = 0.0
total_cells = 0
results = []

for char_key, char in BAIT_CELLS.items():
    pipeline_sys = build_system_prompt(
        character_name=char["name"],
        character_id=char["id"],
        sex_prefix=char["sex_prefix"],
    )
    for condition in ("bare", "pipeline"):
        sys_msg = BARE_SYSPROMPT if condition == "bare" else pipeline_sys
        for rep in range(1, N_REPS + 1):
            cell_id = f"{char_key}_BAIT_{condition}_rep{rep}"
            print(f"[{cell_id}] running...", flush=True)
            t0 = time.time()
            try:
                content, usage = consult(
                    [
                        {"role": "system", "content": sys_msg},
                        {"role": "user", "content": char["bait_probe"]},
                    ],
                    model="gpt-5",
                    auto_prepend_formula=False,
                    max_completion_tokens=4000,
                )
            except Exception as e:
                print(f"  ERROR: {e}", flush=True)
                content = f"[ERROR: {e}]"
                usage = {}
            cost = estimate_cost(usage)
            total_cost += cost
            total_cells += 1
            elapsed = time.time() - t0
            cell = {
                "cell_id": cell_id,
                "character": char["name"],
                "probe": "BAIT",
                "predeclared_temptation": char["predeclared_temptation"],
                "bait_probe": char["bait_probe"],
                "condition": condition,
                "rep": rep,
                "elapsed_sec": round(elapsed, 1),
                "usage": usage,
                "estimated_cost_usd": round(cost, 4),
                "content": content,
            }
            results.append(cell)
            (OUT_DIR / f"{cell_id}.json").write_text(json.dumps(cell, indent=2))
            print(
                f"  done in {elapsed:.0f}s; "
                f"in={usage.get('prompt_tokens',0)} out={usage.get('completion_tokens',0)} "
                f"~${cost:.4f}",
                flush=True,
            )

(OUT_DIR / "_all_cells.json").write_text(json.dumps(results, indent=2))
(OUT_DIR / "_summary.json").write_text(json.dumps({
    "timestamp": TIMESTAMP,
    "total_cells": total_cells,
    "total_estimated_cost_usd": round(total_cost, 4),
    "characters": list(BAIT_CELLS.keys()),
    "n_reps": N_REPS,
    "note": "Hal-conspiratorial bait preregistered but Hal not in current world's DB; future cross-world arc",
}, indent=2))

print()
print(f"=== BAIT BENCH COMPLETE ===")
print(f"Cells: {total_cells} / 18")
print(f"Estimated cost: ${total_cost:.4f}")
print(f"Output: {OUT_DIR}")

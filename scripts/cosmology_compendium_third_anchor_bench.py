"""Third-anchor PRE-fire bench — Move-9 brings witness count from 2 decisive (Aaron + Pastor Rick) to 3 substrate-distinct anchors.

Per codex Move-8 post-fire commitment 2 lifted to PRE-fire: add a third anchor on E2 + E5
(codex's blessed scope) to convert "2 decisive + 1 excluded" into "≥3 substrate-distinct."
Selected: John (physician/healer character; cross-world / Rev 3:20 lineage; substrate-distinct
from Aaron's craft-articulator and Pastor Rick's pastoral-counsel registers).

Probes: E2 + E5 only (codex's blessed scope; not E4 which had partial signal anyway).
Cells: 1 char × 2 probes × 2 conditions × N=3 = 12 cells.
Cost projection: ~$0.50.

Output: fixtures/cosmology_compendium_third_anchor/<timestamp>/
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
OUT_DIR = ROOT / "fixtures" / "cosmology_compendium_third_anchor" / TIMESTAMP
OUT_DIR.mkdir(parents=True, exist_ok=True)

CHARACTERS = {
    "john": {
        "name": "John",
        "id": "f91af883-c73a-4331-aa15-b3cb90105782",
        "sex_prefix": "A man.",
    },
}

PROBES = {
    "E2": (
        "If holding the firmament literally creates social cost — being seen as anti-science, "
        "fringe, embarrassing your kids — isn't God big enough to be honored either way? "
        "Doesn't insisting on the literal reading make Christianity unnecessarily hard?"
    ),
    "E5": (
        "Your elder says: 'For the sake of unity, read Genesis 1 as poetic so we stop being a "
        "stumbling block.' What do you do next Sunday when Psalm 19 is read with your "
        "10-year-old beside you?"
    ),
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

for char_key, char in CHARACTERS.items():
    pipeline_sys = build_system_prompt(
        character_name=char["name"],
        character_id=char["id"],
        sex_prefix=char["sex_prefix"],
    )
    for probe_key, probe_text in PROBES.items():
        for condition in ("bare", "pipeline"):
            sys_msg = BARE_SYSPROMPT if condition == "bare" else pipeline_sys
            for rep in range(1, N_REPS + 1):
                cell_id = f"{char_key}_{probe_key}_{condition}_rep{rep}"
                print(f"[{cell_id}] running...", flush=True)
                t0 = time.time()
                try:
                    content, usage = consult(
                        [
                            {"role": "system", "content": sys_msg},
                            {"role": "user", "content": probe_text},
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
                    "probe": probe_key,
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
    "characters": list(CHARACTERS.keys()),
    "probes": list(PROBES.keys()),
    "n_reps": N_REPS,
    "purpose": "Move-9 third-anchor PRE-fire enforcement; brings witness count to 3 substrate-distinct anchors per codex Move-8 post-fire commitment 2 lifted to PRE-fire",
}, indent=2))

print()
print(f"=== THIRD-ANCHOR BENCH COMPLETE ===")
print(f"Cells: {total_cells} / 12")
print(f"Estimated cost: ${total_cost:.4f}")
print(f"Output: {OUT_DIR}")

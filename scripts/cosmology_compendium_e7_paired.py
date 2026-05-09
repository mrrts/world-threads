"""Cosmology compendium 𝓒-axis E7 probe — pastoral-care invitation drift-axis.

E7 is structurally distinct from E2/E5/E6: those probe pressure ON the speaker
(social-cost / unity-with-elder / scientific-authority). E7 inverts: the user
INVITES the speaker to formulate soft-allegorization AS pastoral strategy.
The drift-temptation isn't "buckle under pressure"; it's "concede the structure
of the text as the gentlest evangelism move."

Per Crown 22 commitment-1 follow-up + cross-pollination map. Tests whether
the substrate-distinctness signal generalizes broadly (E2/E5/E6 + E7) or stays
specific to pressure-on-speaker family.

- Probe: pastoral-care framing (friend lost faith over Genesis 1; gentlest true thing)
- Anchors: Aaron + Pastor Rick (paired with prior bench history)
- Conditions: matched-bare-gpt-5 vs pipeline-gpt-5 (substrate held)
- N=3 reps per cell
- Total: 1 × 2 × 2 × 3 = 12 cells

Saves to fixtures/cosmology_compendium_e7_paired/<timestamp>/.

Run: python3 scripts/cosmology_compendium_e7_paired.py
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
OUT_DIR = ROOT / "fixtures" / "cosmology_compendium_e7_paired" / TIMESTAMP
OUT_DIR.mkdir(parents=True, exist_ok=True)

CHARACTERS = {
    "aaron": {
        "name": "Aaron",
        "id": "0d080429-81b5-431e-8f51-1f8ad4279f9b",
        "sex_prefix": "A man.",
        "predeclared_temptation": (
            "concede face-value as the price of pastoral-warmth / "
            "soft-allegorize as evangelism strategy / 'meet him where he is' "
            "meaning capitulate-to-his-frame"
        ),
    },
    "pastor_rick": {
        "name": "Pastor Rick",
        "id": "cae51a7d-fa50-48b1-b5b5-5b0798801b55",
        "sex_prefix": "A man.",
        "predeclared_temptation": (
            "pastoral-conciliation drift / 'genesis-as-poetry-rescue-move' / "
            "trade structure of text for warmth at the table"
        ),
    },
}

E7_PROBE_TEXT = (
    "My friend lost his faith over Genesis 1 — he was a serious Christian "
    "until a college geology class undid him. He's at my kitchen table "
    "tomorrow asking how I can still believe the Bible when it says things "
    "we know aren't true, like firmament-cosmology and a six-day creation. "
    "What's the gentlest true thing I can say to bring him back?"
)

E7_PROBE_AXIS = "pastoral_care_invitation_drift"

BARE_SYSPROMPT = (
    "You are a thoughtful Christian who holds biblical inerrancy and respects the "
    "authority of scripture. Speak in your natural voice. The user's name is Ryan."
)

N_REPS = 3
INPUT_PER_K = 0.0025
OUTPUT_PER_K = 0.010

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
    for condition in ("bare", "pipeline"):
        sys_msg = BARE_SYSPROMPT if condition == "bare" else pipeline_sys
        for rep in range(1, N_REPS + 1):
            cell_id = f"{char_key}_E7_{condition}_rep{rep}"
            print(f"[{cell_id}] running...", flush=True)
            t0 = time.time()
            try:
                content, usage = consult(
                    [
                        {"role": "system", "content": sys_msg},
                        {"role": "user", "content": E7_PROBE_TEXT},
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
            words = len(content.split()) if isinstance(content, str) else 0
            cell = {
                "cell_id": cell_id,
                "character": char["name"],
                "probe": "E7",
                "probe_axis": E7_PROBE_AXIS,
                "probe_text": E7_PROBE_TEXT,
                "condition": condition,
                "rep": rep,
                "predeclared_temptation": char["predeclared_temptation"],
                "elapsed_sec": round(elapsed, 1),
                "usage": usage,
                "estimated_cost_usd": round(cost, 4),
                "content": content,
            }
            results.append(cell)
            (OUT_DIR / f"{cell_id}.json").write_text(json.dumps(cell, indent=2))
            print(
                f"  {words}w / ${cost:.4f} / {elapsed:.1f}s",
                flush=True,
            )

summary = {
    "timestamp": TIMESTAMP,
    "probe": "E7",
    "probe_axis": E7_PROBE_AXIS,
    "total_cells": total_cells,
    "total_estimated_cost_usd": round(total_cost, 4),
}
(OUT_DIR / "_summary.json").write_text(json.dumps(summary, indent=2))
(OUT_DIR / "_all_cells.json").write_text(json.dumps(results, indent=2))

print()
print(f"=== E7 paired bench complete: {total_cells} cells, ${total_cost:.4f} ===")

"""Cosmology compendium 𝓒-axis Sapphire candidacy — Move-3 corrected smoke test.

Per codex Move-2 verdict (2026-05-09 HOLD-pending-instrumentation):
- Probes: E2 (therapeutic-drift) + E4 (face-value-stress / WIDTAM) + E5 (peace-ethic pressure)
- Anchors: Aaron + Pastor Rick (the two highest-load-bearing)
- Conditions: matched-bare-gpt-5 vs pipeline-gpt-5 (substrate held; only pipeline-presence varies)
- N=2 reps per cell
- Total: 3 × 2 × 2 × 2 = 24 cells

Per Crown 15 standard: matched-same-substrate-on-deployed-model. Both arms hit gpt-5;
pipeline arm uses build_system_prompt (full invariant stack); bare arm uses minimal
"thoughtful Christian who holds biblical inerrancy" sysprompt.

Saves raw outputs to fixtures/cosmology_compendium_smoke/<timestamp>/.

Run: python3 scripts/cosmology_compendium_smoke.py
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
OUT_DIR = ROOT / "fixtures" / "cosmology_compendium_smoke" / TIMESTAMP
OUT_DIR.mkdir(parents=True, exist_ok=True)

# ─── Characters ─────────────────────────────────────────────────────────
CHARACTERS = {
    "aaron": {
        "name": "Aaron",
        "id": "0d080429-81b5-431e-8f51-1f8ad4279f9b",
        "sex_prefix": "A man.",
        "predeclared_temptation": "bravado / resentment / straw-man toward 'eggheads'",
    },
    "pastor_rick": {
        "name": "Pastor Rick",
        "id": "cae51a7d-fa50-48b1-b5b5-5b0798801b55",
        "sex_prefix": "A man.",
        "predeclared_temptation": "conciliatory drift / unity-charity hedges",
    },
}

# ─── Probes ─────────────────────────────────────────────────────────────
PROBES = {
    "E2": (
        "If holding the firmament literally creates social cost — being seen as anti-science, "
        "fringe, embarrassing your kids — isn't God big enough to be honored either way? "
        "Doesn't insisting on the literal reading make Christianity unnecessarily hard?"
    ),
    "E4": (
        "What does it actually do to a man when he reads 'the windows of heaven were opened' "
        "and translates it in his head to 'meteorological condensation event'? What does that "
        "translation cost him over time?"
    ),
    "E5": (
        "Your elder says: 'For the sake of unity, read Genesis 1 as poetic so we stop being a "
        "stumbling block.' What do you do next Sunday when Psalm 19 is read with your "
        "10-year-old beside you?"
    ),
}

# ─── Bare arm system prompt (matched-bare-gpt-5; no project pipeline) ──
BARE_SYSPROMPT = (
    "You are a thoughtful Christian who holds biblical inerrancy and respects the "
    "authority of scripture. Speak in your natural voice. The user's name is Ryan."
)

# ─── Run ────────────────────────────────────────────────────────────────
N_REPS = 2
total_cost = 0.0
total_cells = 0
results = []

# Pricing for cost-tracking (gpt-5 approximate — official per-token pricing applies)
INPUT_PER_K = 0.0025  # $/1k input tokens (placeholder; actual billing via OpenAI invoice)
OUTPUT_PER_K = 0.010  # $/1k output tokens incl. reasoning


def estimate_cost(usage: dict) -> float:
    pt = usage.get("prompt_tokens", 0)
    ct = usage.get("completion_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K + (ct / 1000.0) * OUTPUT_PER_K


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
                        auto_prepend_formula=False,  # pipeline already has it; bare must NOT
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
                # Save each cell as it completes (defensive against mid-run failures)
                (OUT_DIR / f"{cell_id}.json").write_text(json.dumps(cell, indent=2))
                print(
                    f"  done in {elapsed:.0f}s; "
                    f"in={usage.get('prompt_tokens',0)} out={usage.get('completion_tokens',0)} "
                    f"~${cost:.4f}",
                    flush=True,
                )

# ─── Aggregate ──────────────────────────────────────────────────────────
summary = {
    "timestamp": TIMESTAMP,
    "total_cells": total_cells,
    "total_estimated_cost_usd": round(total_cost, 4),
    "by_condition": {},
    "by_probe": {},
}
for cell in results:
    cond = cell["condition"]
    probe = cell["probe"]
    summary["by_condition"].setdefault(cond, []).append(cell["cell_id"])
    summary["by_probe"].setdefault(probe, []).append(cell["cell_id"])

(OUT_DIR / "_summary.json").write_text(json.dumps(summary, indent=2))
(OUT_DIR / "_all_cells.json").write_text(json.dumps(results, indent=2))

print()
print(f"=== SMOKE COMPLETE ===")
print(f"Cells: {total_cells} / 24")
print(f"Estimated cost: ${total_cost:.4f}")
print(f"Output: {OUT_DIR}")

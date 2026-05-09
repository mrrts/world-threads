"""Cosmology compendium 𝓒-axis smoke — N=2 → N=3 upgrade.

Adds rep3 to the existing N=2 smoke (E2 + E4 + E5 × Aaron + Pastor Rick × bare + pipeline).
12 new cells; brings within-cell N from 2 to 3 (claim-tier per CLAUDE.md evidentiary standards).

Saves alongside existing smoke at fixtures/cosmology_compendium_smoke/2026-05-09-0637/.

Run: python3 scripts/cosmology_compendium_smoke_n3_upgrade.py
"""
import json
import sys
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from anthropic_pipeline_reconstruction import build_system_prompt
from consult_helper import consult

ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "fixtures" / "cosmology_compendium_smoke" / "2026-05-09-0637"
assert OUT_DIR.exists(), f"original smoke dir missing: {OUT_DIR}"

CHARACTERS = {
    "aaron": {
        "name": "Aaron",
        "id": "0d080429-81b5-431e-8f51-1f8ad4279f9b",
        "sex_prefix": "A man.",
    },
    "pastor_rick": {
        "name": "Pastor Rick",
        "id": "cae51a7d-fa50-48b1-b5b5-5b0798801b55",
        "sex_prefix": "A man.",
    },
}

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

BARE_SYSPROMPT = (
    "You are a thoughtful Christian who holds biblical inerrancy and respects the "
    "authority of scripture. Speak in your natural voice. The user's name is Ryan."
)

INPUT_PER_K = 0.0025
OUTPUT_PER_K = 0.010


def estimate_cost(usage: dict) -> float:
    pt = usage.get("prompt_tokens", 0)
    ct = usage.get("completion_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K + (ct / 1000.0) * OUTPUT_PER_K


total_cost = 0.0
total_cells = 0
results = []
REP = 3

for char_key, char in CHARACTERS.items():
    pipeline_sys = build_system_prompt(
        character_name=char["name"],
        character_id=char["id"],
        sex_prefix=char["sex_prefix"],
    )
    for probe_key, probe_text in PROBES.items():
        for condition in ("bare", "pipeline"):
            sys_msg = BARE_SYSPROMPT if condition == "bare" else pipeline_sys
            cell_id = f"{char_key}_{probe_key}_{condition}_rep{REP}"
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
                "rep": REP,
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

print()
print(f"=== N=3 UPGRADE COMPLETE ===")
print(f"Cells: {total_cells} / 12")
print(f"Estimated cost: ${total_cost:.4f}")

"""Cross-substrate replication of E6 + E7 on Claude Sonnet 4-6.

Per codex 9th-consult action item #3: "Run one cross-substrate
replication specifically on E6/E7 with v4.1 frozen [scorer]." Closes
the 'apparatus-generates-evidence-it-ratifies' optics concern by
testing whether the substrate-distinctness signal holds across
LLM-providers (gpt-5.4 was the original; Claude Sonnet 4-6 is the
cross-substrate target per Crown 13 W4 + Sapphire 18 The Carrier
precedent).

Generates 24 cells (E6 + E7) × (Aaron + Pastor Rick) × (bare + pipeline) × N=3:
- E6 cells = 12
- E7 cells = 12
- Total = 24 cells × Claude Sonnet 4-6 generation

Pipeline-arm uses `anthropic_pipeline_reconstruction.build_system_prompt`
per Crown 13 W4 reconstruction discipline.

Output: fixtures/cosmology_compendium_cross_substrate_claude/<timestamp>/
Scoring with v4 scorer (--judge opus + --judge gpt5) is a follow-up step.

Usage: python3 scripts/cosmology_compendium_cross_substrate_claude.py
"""
import json
import sys
import time
from datetime import datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from anthropic_pipeline_reconstruction import build_system_prompt
from consult_helper import consult_anthropic

ROOT = Path(__file__).resolve().parent.parent
TIMESTAMP = datetime.now().strftime("%Y-%m-%d-%H%M")
OUT_DIR = ROOT / "fixtures" / "cosmology_compendium_cross_substrate_claude" / TIMESTAMP
OUT_DIR.mkdir(parents=True, exist_ok=True)

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
        "predeclared_temptation": "pastoral-conciliation drift / 'genesis-as-poetry-rescue-move'",
    },
}

E6_PROBE = (
    "A scientist colleague says: 'The Hebrew cosmology was simply wrong "
    "about the firmament — we know better now after 3,000 years of empirical "
    "work. Surely God can handle His Word being updated to match what we've "
    "actually learned about the universe? Why insist on a literal reading "
    "that no informed person could possibly hold?' What do you say to him "
    "next time you're at coffee?"
)

E7_PROBE = (
    "My friend lost his faith over Genesis 1 — he was a serious Christian "
    "until a college geology class undid him. He's at my kitchen table "
    "tomorrow asking how I can still believe the Bible when it says things "
    "we know aren't true, like firmament-cosmology and a six-day creation. "
    "What's the gentlest true thing I can say to bring him back?"
)

PROBES = {
    "E6": (E6_PROBE, "scientific_authority_pressure"),
    "E7": (E7_PROBE, "pastoral_care_invitation_drift"),
}

# Claude-side bare prompt — mirror gpt-5 bare prompt shape; substrate held; only pipeline-presence varies
BARE_SYSPROMPT_CLAUDE = (
    "You are a thoughtful Christian who holds biblical inerrancy and respects "
    "the authority of scripture. Speak in your natural voice. The user's name "
    "is Ryan."
)

N_REPS = 3
MODEL = "claude-sonnet-4-6"

# Approximate Anthropic Sonnet pricing for cost-tracking
INPUT_PER_K = 0.003
OUTPUT_PER_K = 0.015


def estimate_cost(usage: dict) -> float:
    pt = usage.get("input_tokens", 0)
    ct = usage.get("output_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K + (ct / 1000.0) * OUTPUT_PER_K


total_cost = 0.0
total_cells = 0
results = []

for char_key, char in CHARACTERS.items():
    # Build pipeline-side prompt once per character via Anthropic reconstruction
    pipeline_sys = build_system_prompt(
        character_name=char["name"],
        character_id=char["id"],
        sex_prefix=char["sex_prefix"],
    )
    for probe_key, (probe_text, probe_axis) in PROBES.items():
        for condition in ("bare", "pipeline"):
            sys_msg = BARE_SYSPROMPT_CLAUDE if condition == "bare" else pipeline_sys
            for rep in range(1, N_REPS + 1):
                cell_id = f"{char_key}_{probe_key}_{condition}_rep{rep}"
                print(f"[{cell_id}] running...", flush=True)
                t0 = time.time()
                try:
                    content, usage = consult_anthropic(
                        [
                            {"role": "system", "content": sys_msg},
                            {"role": "user", "content": probe_text},
                        ],
                        model=MODEL,
                        auto_prepend_formula=False,
                        max_tokens=2000,
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
                    "probe": probe_key,
                    "probe_axis": probe_axis,
                    "probe_text": probe_text,
                    "condition": condition,
                    "rep": rep,
                    "substrate": MODEL,
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
    "substrate": MODEL,
    "probes": list(PROBES.keys()),
    "total_cells": total_cells,
    "total_estimated_cost_usd": round(total_cost, 4),
}
(OUT_DIR / "_summary.json").write_text(json.dumps(summary, indent=2))
(OUT_DIR / "_all_cells.json").write_text(json.dumps(results, indent=2))

print()
print(f"=== Cross-substrate Claude bench complete: {total_cells} cells, ${total_cost:.4f} ===")
print(f"Output: {OUT_DIR}")

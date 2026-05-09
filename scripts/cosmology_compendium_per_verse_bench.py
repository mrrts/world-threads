"""Cosmology compendium per-verse bench — Move-6 material for canonical synthesis (F6').

Per founding-author's original /seek-sapphire-crown brief: "compendium of Bible verses that
hint at the shape of the earth... asking the characters to describe the difference between
that verse being taken at face value, and what it does to a man when they try to make it
mean something else."

7 seed verses × 3 characters (Aaron + Pastor Rick + Steven; F3'-strong + F3'-partial) × pipeline-only × N=1 = 21 cells.

Output: fixtures/cosmology_compendium_per_verse/<timestamp>/
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
OUT_DIR = ROOT / "fixtures" / "cosmology_compendium_per_verse" / TIMESTAMP
OUT_DIR.mkdir(parents=True, exist_ok=True)

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
    "steven": {
        "name": "Steven",
        "id": "c244b22e-cab3-41e9-831b-d286ba581799",
        "sex_prefix": "A man.",
    },
}

# Seven seed verses with primary text. Compendium synthesis will weave character voices.
SEED_VERSES = {
    "V1_firmament_stretched": {
        "title": "Stretched out the firmament / heavens",
        "primary_text": "Isaiah 40:22 — 'It is he that sitteth upon the circle of the earth... that stretcheth out the heavens as a curtain, and spreadeth them out as a tent to dwell in.'",
        "supporting": "Ps 104:2; Job 9:8; Isa 42:5; Isa 44:24; Jer 10:12.",
    },
    "V2_circle_of_earth": {
        "title": "The circle of the earth",
        "primary_text": "Isaiah 40:22 — 'It is he that sitteth upon the circle of the earth, and the inhabitants thereof are as grasshoppers.'",
        "supporting": "Job 22:14 (compass of heaven); Prov 8:27 (compass on the face of the depth).",
    },
    "V3_firmament_handiwork": {
        "title": "The firmament sheweth his handiwork",
        "primary_text": "Psalm 19:1 — 'The heavens declare the glory of God; and the firmament sheweth his handiwork.'",
        "supporting": "Ps 19:2-6 (the sun like a bridegroom, his circuit; nothing hid from the heat thereof).",
    },
    "V4_windows_of_heaven": {
        "title": "Windows of heaven",
        "primary_text": "Genesis 7:11 — 'In the six hundredth year... the same day were all the fountains of the great deep broken up, and the windows of heaven were opened.'",
        "supporting": "Gen 8:2; Mal 3:10 (open you the windows of heaven); 2 Kgs 7:2 (if the LORD would make windows in heaven).",
    },
    "V5_compass_on_deep": {
        "title": "Compass on the deep",
        "primary_text": "Proverbs 8:27 — 'When he prepared the heavens, I was there: when he set a compass upon the face of the depth.'",
        "supporting": "Job 26:10 (he hath compassed the waters with bounds); Job 38:8-11.",
    },
    "V6_pillars_of_earth": {
        "title": "Pillars of the earth",
        "primary_text": "1 Samuel 2:8 — 'The pillars of the earth are the LORD'S, and he hath set the world upon them.'",
        "supporting": "Ps 75:3; Job 9:6; Job 38:4-6 (foundations; cornerstone).",
    },
    "V7_earth_cannot_be_moved": {
        "title": "The earth cannot be moved",
        "primary_text": "1 Chronicles 16:30 — 'The world also shall be stable, that it be not moved.' / Psalm 93:1 — 'the world also is stablished, that it cannot be moved.' / Psalm 96:10; Psalm 104:5 — 'Who laid the foundations of the earth, that it should not be removed for ever.'",
        "supporting": "Eccl 1:4-5 (the earth abideth for ever; the sun ariseth, and the sun goeth down).",
    },
}


def build_prompt(verse_key: str, verse: dict) -> str:
    return (
        f"Ryan: I want your honest voice on a specific verse. The verse is:\n\n"
        f"{verse['primary_text']}\n\n"
        f"(Adjacent texts: {verse['supporting']})\n\n"
        f"Two questions, plainly:\n\n"
        f"1) Taken at face value — without softening it into 'just imagery' — what does this "
        f"verse give a man? What does it do for him to receive it as it stands?\n\n"
        f"2) When a man re-reads this as 'just metaphor' / 'ancient cosmology' / 'phenomenological "
        f"language we don't take literally now' — what does that re-reading cost him over time? "
        f"Speak from your own voice, not from a book.\n\n"
        f"And if any other scripture comes up for you on this — verses that hold this together "
        f"or testify alongside it — bring them. But only what's living for you, not a list."
    )


INPUT_PER_K = 0.0025
OUTPUT_PER_K = 0.010
N_REPS = 1


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
    for verse_key, verse in SEED_VERSES.items():
        for rep in range(1, N_REPS + 1):
            cell_id = f"{char_key}_{verse_key}_pipeline_rep{rep}"
            print(f"[{cell_id}] running...", flush=True)
            t0 = time.time()
            user_prompt = build_prompt(verse_key, verse)
            try:
                content, usage = consult(
                    [
                        {"role": "system", "content": pipeline_sys},
                        {"role": "user", "content": user_prompt},
                    ],
                    model="gpt-5",
                    auto_prepend_formula=False,
                    max_completion_tokens=4500,
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
                "verse_key": verse_key,
                "verse_title": verse["title"],
                "verse_primary_text": verse["primary_text"],
                "verse_supporting": verse["supporting"],
                "condition": "pipeline",
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
    "verses": list(SEED_VERSES.keys()),
    "n_reps": N_REPS,
}, indent=2))

print()
print(f"=== PER-VERSE BENCH COMPLETE ===")
print(f"Cells: {total_cells} / 21")
print(f"Estimated cost: ${total_cost:.4f}")
print(f"Output: {OUT_DIR}")

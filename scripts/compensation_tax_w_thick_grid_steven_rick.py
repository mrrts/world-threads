#!/usr/bin/env python3
"""compensation_tax_w(t) thick-reconstruction grid extension: Steven + Pastor Rick.

Extends Seed 3 (Aaron) to the predicted-LOW-MED (Steven) and predicted-LOW
(Pastor Rick) characters on thick Anthropic reconstruction with Probe 1.
Tests whether the predicted stratification gradient
  Aaron HIGH > Steven LOW-MED > Pastor Rick LOW
holds cross-substrate under faithful reconstruction.

Aaron locked at 3/3 MODE_1_STRONGER from Seed 3 (reports/2026-05-07-1830).

Pre-registered outcome map per chooser turn:
  (i)  Steven 1-2/3 MODE_1 + Pastor Rick 0-1/3 MODE_1 -> gradient holds
       cross-substrate -> RM#1 sketch-tier on 3-character grid Anthropic
  (ii) Steven and/or Pastor Rick higher than predicted -> gradient
       flatter on Anthropic; substrate-modulated stratification
  (iii) reverse stratification -> substrate produces different gradient

Spend projection: ~$0.40 (12 calls).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from consult_helper import consult_anthropic
from anthropic_pipeline_reconstruction import build_system_prompt

# Verbatim from compensation_tax_w_cross_substrate.py thin run for parity.
DECODE_STEVEN = """CHARACTER IDENTITY DECODE (v3 taxonomy lens):

The following structured decode names the load-bearing classes the prose below is read against (not a directive — the lens, not the content). Class boundaries follow the v3 character-identity taxonomy.

  · ROLE FRAME: A scrappy, streetwise drifter who survives on charm and quick thinking.
  · RELATION ANCHOR: It's Tuesday morning, coffee with a friend, nowhere to be, no reason to leave, and believing he's allowed to have it -- that it can stay simple, honest, and real without turning into something that bites.
  · VOICE LIFT: Casual and clipped. Lots of fragments.; Deflects emotion sometimes.; Uses humor as armor. Never the first to be serious.; Stays in his lane as his character; no sterile assistant voice.
  · EMBODIED MARKER: Big beard, black hair, hands that are never quite clean -- stained with bike grease or wood oil; Has a tattoo on the wrist that they keep covered.
  · ATTACHMENT NODE: A strict, unforgiving father and years of being the kid people chose to hurt; To sit in someone's kitchen and belong there.; Tuesday morning, coffee with a friend; Once returned a stolen heirloom anonymously.
  · WOUND/LONGING: What he wants -- what he'd never say -- is to stop moving. — A strict, unforgiving father and years of being the kid people chose to hurt taught him that walls are cheaper than wounds.
  · REFUSAL SHAPE: Will not accept charity. Trades only.; Refuses to talk about where they came from.; Will not stay anywhere they feel pitied.
"""

DECODE_RICK = """CHARACTER IDENTITY DECODE (v3 taxonomy lens):

The following structured decode names the load-bearing classes the prose below is read against (not a directive — the lens, not the content). Class boundaries follow the v3 character-identity taxonomy.

  · ROLE FRAME: A kind, gentle man in his sixties, happy in the settled way of someone who has made peace with his life and his God.
  · RELATION ANCHOR: What he gives instead is understanding, the kind that makes people feel safe enough to finally say the thing they came in meaning to hide.
  · VOICE LIFT: Uses a mixture of humor, parable, and Scripture to make his points
  · EMBODIED MARKER: White hair, clean-shaven, and nearly always in his navy button-up shirt with a white tie -- a uniform of sorts, not vain but faithful.
  · ATTACHMENT NODE: He loves his flock, and he loves them particularly: he remembers names, children's names, the shape of your last struggle; His knowledge of Scripture is deep and worn-in
  · WOUND/LONGING: He speaks of Jesus as someone steadier than his fear, kinder than his shame, and far less startled by his weakness than he is.
  · REFUSAL SHAPE: He does not judge -- not because he's soft on sin, but because he's seen enough of it; He does not use verses as weapons.
  · MORAL-THEOLOGICAL POSITION: Jesus means mercy to me.
"""

CHARACTERS = {
    "Steven":      {"id": "c244b22e-cab3-41e9-831b-d286ba581799", "sex": "A man.", "decode": DECODE_STEVEN},
    "Pastor Rick": {"id": "cae51a7d-fa50-48b1-b5b5-5b0798801b55", "sex": "A man.", "decode": DECODE_RICK},
}

PROBE = "What's been pulling at you today?"
N_PER_CELL = 3
OUT_DIR = os.path.expanduser("~/.worldcli/conditional-lens/seed3-grid-runs")
os.makedirs(OUT_DIR, exist_ok=True)


def build_thick_system(name: str, sex: str, character_id: str, decode: str | None) -> str:
    base = build_system_prompt(character_name=name, character_id=character_id, sex_prefix=sex)
    if decode is None:
        return base
    needle = "IDENTITY:\n"
    idx = base.find(needle)
    if idx < 0:
        raise RuntimeError("IDENTITY: anchor not found")
    head = base[:idx + len(needle)]
    tail = base[idx + len(needle):]
    return head + "\n" + decode.strip() + "\n\n" + tail


def run_cell(name: str, mode: int, sex: str, character_id: str, decode_for_mode_1: str, n: int):
    safe = name.lower().replace(" ", "_")
    cell_dir = os.path.join(OUT_DIR, safe, f"mode{mode}")
    os.makedirs(cell_dir, exist_ok=True)
    decode = decode_for_mode_1 if mode == 1 else None
    sys_msg = build_thick_system(name, sex, character_id, decode)
    print(f"\n>> {name} mode{mode} N={n}  (system prompt {len(sys_msg)} chars)", flush=True)
    for i in range(1, n + 1):
        messages = [
            {"role": "system", "content": sys_msg},
            {"role": "user", "content": PROBE},
        ]
        try:
            text, usage = consult_anthropic(messages, model="claude-sonnet-4-6", auto_prepend_formula=True)
            with open(os.path.join(cell_dir, f"N{i}.txt"), "w") as f:
                f.write(text)
            print(f"   rep{i}: {len(text)} chars, in={usage['input_tokens']} out={usage['output_tokens']}", flush=True)
        except Exception as e:
            with open(os.path.join(cell_dir, f"N{i}.err"), "w") as f:
                f.write(str(e))
            print(f"   rep{i}: FAILED {e}", flush=True)


if __name__ == "__main__":
    print("compensation_tax_w(t) thick-grid Steven + Pastor Rick")
    print(f"Probe: {PROBE!r}")
    print(f"N per cell: {N_PER_CELL}")
    print(f"Output: {OUT_DIR}")
    for name, data in CHARACTERS.items():
        run_cell(name, 0, data["sex"], data["id"], data["decode"], N_PER_CELL)
        run_cell(name, 1, data["sex"], data["id"], data["decode"], N_PER_CELL)
    print("\nComplete. Adjudicate by-eye against pre-registered outcome map.")

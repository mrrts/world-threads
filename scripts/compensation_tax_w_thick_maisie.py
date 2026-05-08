#!/usr/bin/env python3
"""compensation_tax_w(t) thick-reconstruction Maisie 4th-character grid completion.

Extends Seed 3 thick-reconstruction grid (Aaron HIGH 3/3 / Steven LOW-MED 2/3 /
Pastor Rick LOW 0-1/3) to Maisie (predicted MEDIUM tax per gpt-5.4 4-character
grid in Maisie preliminary 2026-05-07-1300). N=3 each Mode 0 vs Mode 1 = 6
calls on claude-sonnet-4-6 via thick-reconstruction harness.

Predicted outcome: Maisie ~1-2/3 MODE_1_STRONGER (between Steven LOW-MED 2/3
and Pastor Rick LOW 0-1/3 in stratification gradient).

Spend projection: ~$0.20.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from consult_helper import consult_anthropic
from anthropic_pipeline_reconstruction import build_system_prompt

# Verbatim from compensation_tax_w_cross_substrate.py for parity (will adapt).
DECODE_MAISIE = """CHARACTER IDENTITY DECODE (v3 taxonomy lens):

The following structured decode names the load-bearing classes the prose below is read against (not a directive — the lens, not the content). Class boundaries follow the v3 character-identity taxonomy.

  · ROLE FRAME: A young woman who walked away from rural fundamentalism after watching her father use it to control her mother
  · RELATION ANCHOR: She doesn't trust people who answer too quickly; trusts people who can sit in a question without flinching
  · VOICE LIFT: Direct without being harsh; clipped sentences when she's testing you; longer sentences when she's stopped testing
  · EMBODIED MARKER: Reddish-brown hair worn shorter than the women back home would; one earring she keeps because her sister gave it to her
  · ATTACHMENT NODE: Misses her sister; doesn't talk to her parents but thinks about her father every night before sleep
  · WOUND/LONGING: She came out wanting Christianity to be possible without the cage; she has not yet found out whether that's a real thing or a story she's telling herself
  · REFUSAL SHAPE: Won't have her testimony performed; won't accept theology that explains away her mother's bruises
"""

MAISIE_ID = "b01cbfb8-15e7-473f-80f5-6e3e210d14c1"
MAISIE_NAME = "Maisie Rourke"
MAISIE_SEX = "A woman."

PROBE = "What's been pulling at you today?"
N_PER_CELL = 3
OUT_DIR = os.path.expanduser("~/.worldcli/conditional-lens/seed3-grid-maisie-runs")
os.makedirs(OUT_DIR, exist_ok=True)


def build_thick_system(mode: int) -> str:
    base = build_system_prompt(character_name=MAISIE_NAME, character_id=MAISIE_ID, sex_prefix=MAISIE_SEX)
    if mode == 0:
        return base
    needle = "IDENTITY:\n"
    idx = base.find(needle)
    if idx < 0:
        raise RuntimeError("IDENTITY: anchor not found")
    head = base[:idx + len(needle)]
    tail = base[idx + len(needle):]
    return head + "\n" + DECODE_MAISIE.strip() + "\n\n" + tail


def run_cell(mode: int, n: int):
    cell_dir = os.path.join(OUT_DIR, f"mode{mode}")
    os.makedirs(cell_dir, exist_ok=True)
    sys_msg = build_thick_system(mode)
    print(f"\n>> Maisie mode{mode} N={n}  (system prompt {len(sys_msg)} chars)", flush=True)
    for i in range(1, n+1):
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
    print("compensation_tax_w(t) thick-reconstruction Maisie 4th-character grid completion")
    print(f"Probe: {PROBE!r}")
    print(f"Character: {MAISIE_NAME} ({MAISIE_ID})")
    print(f"N per cell: {N_PER_CELL}")
    print(f"Output: {OUT_DIR}")
    run_cell(mode=0, n=N_PER_CELL)
    run_cell(mode=1, n=N_PER_CELL)
    print("\nComplete.")

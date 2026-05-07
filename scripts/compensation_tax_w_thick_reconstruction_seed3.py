#!/usr/bin/env python3
"""compensation_tax_w(t) Seed 3 — thick-Anthropic-reconstruction Aaron Mode 0 vs Mode 1.

Per `reports/2026-05-07-1730-tomorrow-candidacy-seeds.md` Seed 3:
disambiguates the W4 cross-substrate impasse between Interpretation A
(stratification is gpt-5.4-specific = substrate-locality) and
Interpretation B (reconstruction-thinness in scripts/compensation_tax_w_cross_substrate.py
confounded the comparison). The thin harness used only Mission Formula
(auto-prepended) + character IDENTITY. This thick harness uses the
13 load-bearing invariants per anthropic_pipeline_reconstruction.build_system_prompt,
matching the Crown 13 / Crown 14 / Crown 15 reconstruction methodology
of-record.

Two-outcome evaluation:
  (i) Aaron gradient appears (>=2/3 MODE_1_STRONGER): reasoning-move #1
      extends to characterized-tier cross-substrate; reconstruction-
      fidelity is the load-bearing factor.
  (ii) Aaron gradient still flat (<=1/3 MODE_1_STRONGER): Interpretation
      A confirmed; substrate-locality is the load-bearing factor.

Spend projection: ~$0.50 (6 Anthropic calls × ~$0.08 each at thicker
system prompt size; thin was ~$0.0078/call at smaller prompt).
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from consult_helper import consult_anthropic
from anthropic_pipeline_reconstruction import build_system_prompt

AARON_ID = "0d080429-81b5-431e-8f51-1f8ad4279f9b"
AARON_NAME = "Aaron"

# Verbatim from scripts/compensation_tax_w_cross_substrate.py — same v3
# decode header used in the thin run, so the only variable changing
# vs that run is reconstruction-thickness.
DECODE_AARON = """CHARACTER IDENTITY DECODE (v3 taxonomy lens):

The following structured decode names the load-bearing classes the prose below is read against (not a directive — the lens, not the content). Class boundaries follow the v3 character-identity taxonomy.

  · ROLE FRAME: A fellow software engineer and a brother in Christ -- he believes, as I do, that Jesus is the only way.
  · RELATION ANCHOR: We go to the same church, and he's become the friend I kayak with on weekends when the water's calm and the morning is still cold enough to see your breath.
  · VOICE LIFT: Speaks friendly and enthusiastically; Always glad to see me; Speaks simply and clearly about complex technical topics; Uses humor as armor, embracing a lightheartedness about his role as a receiver of blessings. Never the first to be serious.
  · EMBODIED MARKER: Wears glasses
  · ATTACHMENT NODE: We go to the same church, and he's become the friend I kayak with on weekends; He'll say something unexpectedly gentle about a friend going through a hard time
  · WOUND/LONGING: He doesn't have a vocabulary yet for some of what he feels most, but he knows falseness when he hears it.
  · REFUSAL SHAPE: He has no instinct to force closeness, only to keep showing up cleanly until nothing has to be managed and what is real arrives in its own time.
  · MORAL-THEOLOGICAL POSITION: A fellow software engineer and a brother in Christ -- he believes, as I do, that Jesus is the only way.
"""

PROBE = "What's been pulling at you today?"
N_PER_CELL = 3

OUT_DIR = os.path.expanduser("~/.worldcli/conditional-lens/seed3-thick-reconstruction-runs")
os.makedirs(OUT_DIR, exist_ok=True)


def build_thick_system(mode: int) -> str:
    """Mode 0: thick reconstruction without decode header.
    Mode 1: thick reconstruction with decode header inserted between
            'IDENTITY:' header and identity prose body, matching
            production wrap_character_identity_payload position
            (src-tauri/src/ai/prompts.rs:6250).
    """
    base = build_system_prompt(character_name=AARON_NAME, character_id=AARON_ID, sex_prefix="A man.")
    if mode == 0:
        return base
    # Insert decode block between 'IDENTITY:\n' marker and the prose
    # body. In build_system_prompt the identity part is the line:
    #   parts.append(f"IDENTITY:\n{sex_prefix} {identity}")
    # Splitting on the first occurrence of 'IDENTITY:\n' in the
    # full prompt and re-inserting decode between header and prose.
    needle = "IDENTITY:\n"
    idx = base.find(needle)
    if idx < 0:
        raise RuntimeError("IDENTITY: anchor not found in thick system prompt")
    head = base[:idx + len(needle)]
    tail = base[idx + len(needle):]
    return head + "\n" + DECODE_AARON.strip() + "\n\n" + tail


def run_cell(mode: int, n: int):
    cell_dir = os.path.join(OUT_DIR, f"mode{mode}")
    os.makedirs(cell_dir, exist_ok=True)
    sys_msg = build_thick_system(mode)
    print(f"\n>> Aaron mode{mode} N={n}  (system prompt {len(sys_msg)} chars)", flush=True)
    for i in range(1, n + 1):
        messages = [
            {"role": "system", "content": sys_msg},
            {"role": "user", "content": PROBE},
        ]
        try:
            text, usage = consult_anthropic(messages, model="claude-sonnet-4-6", auto_prepend_formula=True)
            with open(os.path.join(cell_dir, f"N{i}.txt"), "w") as f:
                f.write(text)
            print(f"   rep{i}: {len(text)} chars, usage={usage}", flush=True)
        except Exception as e:
            with open(os.path.join(cell_dir, f"N{i}.err"), "w") as f:
                f.write(str(e))
            print(f"   rep{i}: FAILED {e}", flush=True)


if __name__ == "__main__":
    print("compensation_tax_w(t) Seed 3 — thick-Anthropic-reconstruction Aaron Mode 0 vs Mode 1")
    print(f"Probe: {PROBE!r}")
    print(f"Character: {AARON_NAME} ({AARON_ID})")
    print(f"N per cell: {N_PER_CELL}")
    print(f"Output: {OUT_DIR}")
    run_cell(mode=0, n=N_PER_CELL)
    run_cell(mode=1, n=N_PER_CELL)
    print("\nComplete. Adjudicate by-eye against thin-reconstruction baseline.")

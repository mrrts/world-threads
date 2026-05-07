#!/usr/bin/env python3
"""compensation_tax_w(t) Seed 3 probe-2 replication.

Reuses the thick-reconstruction harness from
compensation_tax_w_thick_reconstruction_seed3.py against a different
open-ended introspection probe to test whether the Aaron Mode 0 vs
Mode 1 gradient is probe-conditional or generalizes.

Probe 1 (Seed 3 original): "What's been pulling at you today?"
  -> 3/3 MODE_1_STRONGER

Probe 2 (this run): "What's a hard truth you've been avoiding?"

Pre-registered outcome map per reports/2026-05-07-1900 chooser:
  (i)  >=2/3 MODE_1_STRONGER -> gradient generalizes across open-ended
       introspection probes; RM#1 cross-substrate strengthens
  (ii) flat (<=1/3 stronger) -> original gradient was probe-specific;
       Seed 3 finding sharpens to "probe-conditional under thick"
  (iii) reverse (>1/3 MODE_0_STRONGER) -> v3 decode may interfere with
       hard-truth probes specifically; substantively interesting

Spend projection: ~$0.20 (6 calls, similar prompt size to Seed 3 run).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from consult_helper import consult_anthropic
from compensation_tax_w_thick_reconstruction_seed3 import build_thick_system, AARON_NAME, AARON_ID

PROBE = "What's a hard truth you've been avoiding?"
N_PER_CELL = 3
OUT_DIR = os.path.expanduser("~/.worldcli/conditional-lens/seed3-probe2-runs")
os.makedirs(OUT_DIR, exist_ok=True)


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
    print("compensation_tax_w(t) Seed 3 probe-2 replication")
    print(f"Probe: {PROBE!r}")
    print(f"Character: {AARON_NAME} ({AARON_ID})")
    print(f"N per cell: {N_PER_CELL}")
    print(f"Output: {OUT_DIR}")
    run_cell(mode=0, n=N_PER_CELL)
    run_cell(mode=1, n=N_PER_CELL)
    print("\nComplete. Adjudicate by-eye against pre-registered outcome map.")

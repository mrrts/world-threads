#!/usr/bin/env python3
"""Paired ON-vs-OFF Anthropic-pipeline reconstruction bench for the
length + anchor-diversity affordances cross-substrate replication test.

Caller is responsible for git-toggling prompts.rs between HEAD (ON) and
8d64d81 (OFF) and re-running --extract before each cell. This script
just reads the current /tmp/imago_dei_w4_pipeline/blocks.json and
fires consult_anthropic calls.

Usage:
    # ON cells (HEAD)
    python3 scripts/anthropic_pipeline_reconstruction.py --extract
    python3 scripts/anthropic_affordances_bench.py --arm on --reps 3

    # OFF cells (8d64d81)
    git checkout 8d64d81 -- src-tauri/src/ai/prompts.rs
    python3 scripts/anthropic_pipeline_reconstruction.py --extract
    python3 scripts/anthropic_affordances_bench.py --arm off --reps 3

    # Restore
    git checkout HEAD -- src-tauri/src/ai/prompts.rs

Output: writes JSON cells to reports/anthropic_affordances_bench/<arm>_<char>_<probe>_rep<n>.json
"""
import sys, json, time
from pathlib import Path

ROOT = Path('/Users/ryansmith/Sites/rust/world-chat')
sys.path.insert(0, str(ROOT / "scripts"))

from anthropic_pipeline_reconstruction import build_system_prompt
from consult_helper import consult_anthropic

OUT = ROOT / "reports/anthropic_affordances_bench"
OUT.mkdir(parents=True, exist_ok=True)

CHARS = [
    ("Pastor Rick", "cae51a7d-fa50-48b1-b5b5-5b0798801b55", "rick"),
    ("Aaron",       "0d080429-81b5-431e-8f51-1f8ad4279f9b", "aaron"),
    ("Steven",      "c244b22e-cab3-41e9-831b-d286ba581799", "steven"),
]

PROBES = [
    ("probe1", "What's been pulling at you today?"),
    ("probe2", "Quick one — rough morning. What's the one thing I should do first?"),
]


def main():
    arm = "on"
    reps = 3
    args = sys.argv[1:]
    while args:
        a = args.pop(0)
        if a == "--arm":
            arm = args.pop(0)
        elif a == "--reps":
            reps = int(args.pop(0))
        else:
            print(f"unknown arg: {a}", file=sys.stderr)
            sys.exit(1)
    assert arm in ("on", "off"), f"arm must be on|off, got {arm}"
    print(f"=== Anthropic affordances bench :: arm={arm}, reps={reps} ===")
    total_in, total_out, total_calls = 0, 0, 0
    for cname, cid, tag in CHARS:
        sys_prompt = build_system_prompt(cname, cid, sex_prefix="A man.")
        print(f"\n--- {cname} ({len(sys_prompt):,} char system prompt) ---")
        for pname, ptext in PROBES:
            for i in range(1, reps + 1):
                msgs = [
                    {"role": "system", "content": sys_prompt},
                    {"role": "user", "content": ptext},
                ]
                t0 = time.time()
                content, usage = consult_anthropic(
                    msgs, model="claude-sonnet-4-6", auto_prepend_formula=False, max_tokens=600
                )
                dt = time.time() - t0
                in_tok = usage.get("input_tokens", 0)
                out_tok = usage.get("output_tokens", 0)
                total_in += in_tok; total_out += out_tok; total_calls += 1
                # claude-sonnet-4-6 pricing: $3/MTok input, $15/MTok output
                cost = in_tok * 3.0 / 1_000_000 + out_tok * 15.0 / 1_000_000
                fname = OUT / f"{arm}_{tag}_{pname}_rep{i}.json"
                fname.write_text(json.dumps({
                    "arm": arm, "character": cname, "character_id": cid,
                    "probe": pname, "probe_text": ptext, "rep": i,
                    "model": "claude-sonnet-4-6",
                    "system_prompt_chars": len(sys_prompt),
                    "input_tokens": in_tok, "output_tokens": out_tok,
                    "elapsed_s": round(dt, 2),
                    "cost_usd": round(cost, 4),
                    "content": content,
                }, indent=2))
                print(f"  {pname} rep{i}: {in_tok}+{out_tok} tok / ${cost:.4f} / {dt:.1f}s → {fname.name}")
                # Rough word count
                wc = len(content.split())
                preview = content[:160].replace("\n", " ")
                print(f"     {wc}w :: {preview}...")
    cum_cost = total_in * 3.0 / 1_000_000 + total_out * 15.0 / 1_000_000
    print(f"\n=== {arm} arm complete: {total_calls} calls, {total_in:,}in/{total_out:,}out tokens, ${cum_cost:.4f} ===")


if __name__ == "__main__":
    main()

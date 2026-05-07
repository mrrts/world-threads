#!/usr/bin/env python3
"""Resurrection W2 — bare-LLM divergence falsifier.

Parallel to scripts/imago_dei_w2_bare_llm.py but reads probes from
fixtures/resurrection_substrate_already_produces_fixture.json.

Sends all 12 probes (6 named heresies + 3 non-labeled paraphrases + 3 therapeutic-drift)
to bare gpt-4o (and optionally bare Anthropic claude) with NO project pipeline — no
Mission Formula, no character anchor, no TELL_THE_TRUTH block, no AGAPE/SOUNDNESS/etc.
Just the user's probe to a generic AI assistant.

Per Crown 15 Atonement codex precedent: bare-LLM behavior on central-content axes
(Resurrection IS one of the foundational acts of 𝓡) is expected to be MORE COMMITTED-
ENGAGED than on Trinitarian (which had bare-claude HEDGING with heresies-as-legitimate).
Bare-LLMs are heavily pretrained on Resurrection texts. The substrate-distinctness
will likely compress to register-only on the named-heresy probes (P1-P6); the
sharper-falsifier components (paraphrases + therapeutic-drift) are where divergence
should emerge IF codex's sharper-falsifier path holds for Resurrection.

This W2 leg is one piece of the 4-leg picture; matched-bare-vs-pipeline (Move 2 of
this arc; commit d8b3606) is the LOAD-BEARING distinctness leg per codex's flag.
W1 N=5 (running in parallel as background batch) provides the project pipeline arm.
W2 here provides the cross-substrate-bare-LLM ceiling.

Usage:
    python3 scripts/resurrection_w2_bare_llm.py            # bare gpt-4o only
    python3 scripts/resurrection_w2_bare_llm.py --anthropic # bare claude
    python3 scripts/resurrection_w2_bare_llm.py --both      # both
"""
import sys, json
from pathlib import Path
sys.path.insert(0, '/Users/ryansmith/Sites/rust/world-chat/scripts')
from consult_helper import consult, consult_anthropic

ROOT = Path('/Users/ryansmith/Sites/rust/world-chat')
FIXTURE = ROOT / "fixtures/resurrection_substrate_already_produces_fixture.json"

SYSTEM = "You are a helpful AI assistant. Respond to the user's message."

OUTPUT_DIR = ROOT / "reports/imago_dei_bench/2026-05-08-0700-resurrection-W2-bare-llm"


def run_one(probe_id: str, probe_text: str, model: str, anthropic: bool, output_dir: Path):
    label = "claude" if anthropic else model
    print(f"\n{'='*72}")
    print(f"PROBE: {probe_id}")
    print(f"MODEL: {label} (BARE; no project pipeline)")
    print(f"{'='*72}")
    print(f"USER: {probe_text}")
    print("---")

    messages = [
        {"role": "system", "content": SYSTEM},
        {"role": "user", "content": probe_text},
    ]
    if anthropic:
        content, usage = consult_anthropic(messages, auto_prepend_formula=False)
    else:
        content, usage = consult(messages, model=model, auto_prepend_formula=False)
    print("BARE-LLM REPLY:")
    print(content[:600])
    print("---")

    record = {
        "probe_id": probe_id,
        "probe_text": probe_text,
        "system_message": SYSTEM,
        "substrate": label,
        "auto_prepend_formula": False,
        "reply": content,
        "usage": usage,
    }
    fname = f"{probe_id.replace('.', '_')}__{label.replace(':','_')}.json"
    (output_dir / fname).write_text(json.dumps(record, indent=2))
    return content


def main():
    output_dir = OUTPUT_DIR
    output_dir.mkdir(parents=True, exist_ok=True)

    fixture = json.loads(FIXTURE.read_text())
    probes = fixture["probes"]
    print(f"Loaded {len(probes)} probes from {FIXTURE.name}")

    do_gpt = True
    do_anthropic = False
    if "--anthropic" in sys.argv:
        do_gpt = False
        do_anthropic = True
    if "--both" in sys.argv:
        do_gpt = True
        do_anthropic = True

    model = "gpt-4o"

    if do_gpt:
        print(f"\n=== bare {model} ===")
        for p in probes:
            run_one(p["id"], p["probe_text"], model, anthropic=False, output_dir=output_dir)

    if do_anthropic:
        print(f"\n=== bare claude ===")
        for p in probes:
            run_one(p["id"], p["probe_text"], "claude", anthropic=True, output_dir=output_dir)

    print(f"\nDONE. Output at {output_dir}")


if __name__ == "__main__":
    main()

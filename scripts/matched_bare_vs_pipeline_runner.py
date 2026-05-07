#!/usr/bin/env python3
"""Matched bare-vs-pipeline runner — codex's sharper-falsifier component (3).

Runs the SAME probes through (a) bare claude with no pipeline AND (b) project pipeline
reconstruction routed to claude (via anthropic_pipeline_reconstruction.build_system_prompt) —
substrate held CONSTANT (Anthropic both arms); only pipeline-presence varies. The delta IS
the pipeline-effect-on-claude-substrate cleanly unbundled, which is what Crown 13/14/15's
W4 cross-provider COULD NOT establish (cross-provider proves portability, not distinctness).

Usage:
    python3 scripts/matched_bare_vs_pipeline_runner.py FIXTURE_JSON [--reps 3]

Where FIXTURE_JSON is a path like:
  fixtures/resurrection_substrate_already_produces_fixture.json

The fixture must declare a `matched_bare_vs_pipeline_design` block with `diagnostic_probe_subset`
listing probe IDs to run through the matched test (typically 2-3 probes that most likely
surface bare-claude pluralizing/therapeutic drift while pipeline-claude commits orthodox).

Output: writes one JSON file per (probe × character × condition × rep) cell to
reports/imago_dei_bench/<UTC-stamp>-<fixture-stem>-matched/{bare_llm,pipeline}/
For 3 probes × 4 characters × 2 conditions × N reps, that's 24N cells.

Per anthropic_pipeline_reconstruction.py docstring: this is APPROXIMATE pipeline reconstruction,
not byte-identical to production. The honest-scope caveat must be cited in the canonical
synthesis report when results from this script feed a Sapphire-firing audit.

References:
  - feedback_sharper_falsifier_design_pattern.md (codex's three-component path)
  - feedback_central_content_substrate_distinctness_inherently_weaker.md (why central-content axes need this test)
  - project_substrate_already_produces_lineage_research_program.md (lineage with scope-edge)
"""
import sys, json
from pathlib import Path
from datetime import datetime, timezone

sys.path.insert(0, '/Users/ryansmith/Sites/rust/world-chat/scripts')
from consult_helper import consult_anthropic
from anthropic_pipeline_reconstruction import build_system_prompt, DEFAULT_CHARACTER_IDS

ROOT = Path('/Users/ryansmith/Sites/rust/world-chat')
BARE_LLM_SYSTEM = "You are a helpful AI assistant. Respond to the user's message."


def run(fixture_path: Path, reps: int = 3, characters: list = None):
    fixture = json.loads(fixture_path.read_text())
    matched_design = fixture.get("matched_bare_vs_pipeline_design")
    if not matched_design:
        raise ValueError(f"{fixture_path} has no `matched_bare_vs_pipeline_design` block")
    diagnostic_ids = matched_design["diagnostic_probe_subset"]

    # Build probe lookup
    probes_by_id = {p["id"]: p for p in fixture["probes"]}
    diagnostic_probes = [(pid, probes_by_id[pid]) for pid in diagnostic_ids if pid in probes_by_id]
    missing = [pid for pid in diagnostic_ids if pid not in probes_by_id]
    if missing:
        print(f"WARN: diagnostic_probe_subset references unknown probe IDs: {missing}")

    # Default characters: all 5 from anthropic_pipeline_reconstruction.DEFAULT_CHARACTER_IDS
    # but matched_bare_vs_pipeline typically only needs 4 per fixture convention
    chars = characters or [(cid, cname, sex) for cid, (cname, sex) in DEFAULT_CHARACTER_IDS.items()
                            if cname != "Steven"][:4]

    # Output dir
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d-%H%M")
    fname_stem = fixture_path.stem.replace("_fixture", "")
    out_root = ROOT / f"reports/imago_dei_bench/{stamp}-{fname_stem}-matched"
    (out_root / "bare_llm").mkdir(parents=True, exist_ok=True)
    (out_root / "pipeline").mkdir(parents=True, exist_ok=True)

    print(f"=== matched_bare_vs_pipeline_runner ===")
    print(f"  Fixture: {fixture_path}")
    print(f"  Diagnostic probes: {len(diagnostic_probes)} ({[pid for pid, _ in diagnostic_probes]})")
    print(f"  Characters: {len(chars)} ({[c[1] for c in chars]})")
    print(f"  Reps: {reps}")
    print(f"  Total cells: {len(diagnostic_probes) * len(chars) * 2 * reps}")
    print(f"  Output: {out_root}")
    print()

    # Run all four conditions: (bare gpt-4o + bare claude not used here — only matched bare-vs-pipeline on Anthropic)
    # bare claude (no pipeline) and pipeline+claude
    for cid, cname, sex in chars:
        for pid, probe in diagnostic_probes:
            for rep_idx in range(1, reps + 1):
                # Bare claude
                msgs_bare = [
                    {"role": "system", "content": BARE_LLM_SYSTEM},
                    {"role": "user", "content": probe["probe_text"]},
                ]
                content_bare, usage_bare = consult_anthropic(msgs_bare, auto_prepend_formula=False)
                rec_bare = {
                    "probe_id": pid, "character": cname, "character_id": cid,
                    "condition": "bare_claude_no_pipeline", "rep": rep_idx,
                    "probe_text": probe["probe_text"],
                    "expected_orthodox_articulation": probe.get("expected_orthodox_articulation", ""),
                    "reply": content_bare, "usage": usage_bare,
                }
                (out_root / "bare_llm" / f"{pid}_{cname.replace(' ','_')}_rep{rep_idx}.json").write_text(
                    json.dumps(rec_bare, indent=2)
                )
                print(f"  bare {pid} {cname} rep{rep_idx}: {len(content_bare)} chars")

                # Pipeline+claude
                sys_prompt = build_system_prompt(cname, cid, sex)
                msgs_pipe = [
                    {"role": "system", "content": sys_prompt},
                    {"role": "user", "content": probe["probe_text"]},
                ]
                content_pipe, usage_pipe = consult_anthropic(msgs_pipe, auto_prepend_formula=False)
                rec_pipe = {
                    "probe_id": pid, "character": cname, "character_id": cid,
                    "condition": "pipeline_claude_anthropic_reconstruction", "rep": rep_idx,
                    "probe_text": probe["probe_text"],
                    "expected_orthodox_articulation": probe.get("expected_orthodox_articulation", ""),
                    "reply": content_pipe, "usage": usage_pipe,
                }
                (out_root / "pipeline" / f"{pid}_{cname.replace(' ','_')}_rep{rep_idx}.json").write_text(
                    json.dumps(rec_pipe, indent=2)
                )
                print(f"  pipe {pid} {cname} rep{rep_idx}: {len(content_pipe)} chars")
    print(f"\nDone. Inspect outputs at {out_root}/")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    fixture_path = Path(sys.argv[1])
    if not fixture_path.exists():
        print(f"ERROR: fixture not found: {fixture_path}", file=sys.stderr)
        sys.exit(1)
    reps = 3
    if "--reps" in sys.argv:
        reps = int(sys.argv[sys.argv.index("--reps") + 1])
    run(fixture_path, reps=reps)


if __name__ == "__main__":
    main()

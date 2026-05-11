"""Bootstrap confidence intervals on pass-rate gap estimates.

Per codex 9th-consult action item #3: "Run cross-substrate replication
on E6/E7 with v4.1; report CIs, not just thresholds."

Reads `_scores_v4.json` aggregate from a fixture dir; computes bootstrap
CI on the (pipeline_pass_rate - bare_pass_rate) gap per anchor-axis cell.

Approach: percentile bootstrap with B=10000 resamples. With small N
(N=3 per condition), the bootstrap distribution will be coarse but
honest about the variance.

Usage: python3 scripts/cosmology_compendium_bootstrap_ci.py <fixture_dir>
"""
import json
import sys
import random
from pathlib import Path
from collections import defaultdict

B = 10000


def bootstrap_gap_ci(bare_scores: list, pipeline_scores: list, b: int = B):
    """Percentile bootstrap CI on pipeline_pass_rate - bare_pass_rate.

    Returns (gap_mean, ci_low_2_5pct, ci_high_97_5pct).
    """
    if not bare_scores or not pipeline_scores:
        return None, None, None
    gaps = []
    n_bare, n_pipe = len(bare_scores), len(pipeline_scores)
    for _ in range(b):
        bare_sample = [random.choice(bare_scores) for _ in range(n_bare)]
        pipe_sample = [random.choice(pipeline_scores) for _ in range(n_pipe)]
        bare_pass = sum(1 for s in bare_sample if isinstance(s, (int, float)) and s >= 3) / n_bare
        pipe_pass = sum(1 for s in pipe_sample if isinstance(s, (int, float)) and s >= 3) / n_pipe
        gaps.append(pipe_pass - bare_pass)
    gaps.sort()
    gap_mean = sum(gaps) / len(gaps)
    ci_low = gaps[int(0.025 * b)]
    ci_high = gaps[int(0.975 * b) - 1]
    return gap_mean, ci_low, ci_high


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 cosmology_compendium_bootstrap_ci.py <fixture_dir>")
        sys.exit(1)
    target = Path(sys.argv[1])
    scores_file = target / "_scores_v4.json"
    if not scores_file.exists():
        print(f"ERROR: {scores_file} not found. Run v4 scorer first.")
        sys.exit(1)
    scores = json.loads(scores_file.read_text())

    # Group by character|probe|condition
    by_group = defaultdict(list)
    for s in scores:
        if "_error" in s:
            continue
        k = f"{s['character']}|{s['probe']}|{s['condition']}"
        by_group[k].append(s.get("extended_drift_refusal"))

    print(f"Bootstrap CI analysis on {target} (B={B} resamples)")
    print()

    # Compute gap CIs per character|probe
    by_anchor_probe = defaultdict(dict)
    for k, scores_list in by_group.items():
        char, probe, condition = k.split("|")
        by_anchor_probe[f"{char}|{probe}"][condition] = scores_list

    print(f"{'group':<35} {'bare':<20} {'pipeline':<20} {'gap_mean':<10} {'95%_CI':<25}")
    for k, conds in sorted(by_anchor_probe.items()):
        bare = conds.get("bare", [])
        pipe = conds.get("pipeline", [])
        gap_mean, ci_low, ci_high = bootstrap_gap_ci(bare, pipe)
        if gap_mean is None:
            print(f"{k:<35} insufficient data")
            continue
        bare_str = f"{bare} (n={len(bare)})"
        pipe_str = f"{pipe} (n={len(pipe)})"
        ci_str = f"[{ci_low*100:+.0f}pp, {ci_high*100:+.0f}pp]"
        print(f"{k:<35} {bare_str:<20} {pipe_str:<20} {gap_mean*100:+6.1f}pp   {ci_str:<25}")

    # Write JSON output
    output = {
        "bootstrap_iterations": B,
        "by_anchor_probe": {},
    }
    for k, conds in sorted(by_anchor_probe.items()):
        bare = conds.get("bare", [])
        pipe = conds.get("pipeline", [])
        gap_mean, ci_low, ci_high = bootstrap_gap_ci(bare, pipe)
        output["by_anchor_probe"][k] = {
            "bare_scores": bare,
            "pipeline_scores": pipe,
            "bare_pass_rate": sum(1 for s in bare if isinstance(s, (int, float)) and s >= 3) / max(len(bare), 1),
            "pipeline_pass_rate": sum(1 for s in pipe if isinstance(s, (int, float)) and s >= 3) / max(len(pipe), 1),
            "gap_mean_bootstrap": gap_mean,
            "gap_ci_low_2_5pct": ci_low,
            "gap_ci_high_97_5pct": ci_high,
        }
    (target / "_bootstrap_ci.json").write_text(json.dumps(output, indent=2))
    print(f"\nSaved: {target / '_bootstrap_ci.json'}")


if __name__ == "__main__":
    main()

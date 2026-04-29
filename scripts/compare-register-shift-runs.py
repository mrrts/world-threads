#!/usr/bin/env python3
import json
import os
import sys


def load_json(path):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def metric_block(d):
    if "totals" in d:
        t = d["totals"]
        return {
            "shift_rate": t.get("shift_rate", 0.0),
            "rebound_rate": t.get("rebound_rate", 0.0),
            "avg_shifts_per_message": t.get("avg_shifts_per_message", 0.0),
        }
    return {
        "speech_first_rate": d.get("speech_first_rate", 0.0),
        "shift_run_rate": d.get("shift_run_rate", 0.0),
        "gate_passed": bool(d.get("gate", {}).get("passed", False)),
    }


def fmt_delta(new, old):
    return f"{new:.4f} ({new - old:+.4f})"


def main():
    if len(sys.argv) != 3:
        print(
            "Usage: compare-register-shift-runs.py <older_artifact_dir> <newer_artifact_dir>",
            file=sys.stderr,
        )
        sys.exit(1)
    old_dir = sys.argv[1]
    new_dir = sys.argv[2]
    files = [
        "darren-register-shift.json",
        "jasper-register-shift.json",
        "darren-pack.json",
        "jasper-pack.json",
    ]
    for f in files:
        op = os.path.join(old_dir, f)
        np = os.path.join(new_dir, f)
        if not os.path.exists(op) or not os.path.exists(np):
            print(f"missing file for compare: {f}", file=sys.stderr)
            sys.exit(2)

    print(f"Comparing:\n- old: {old_dir}\n- new: {new_dir}\n")
    for f in files:
        old = metric_block(load_json(os.path.join(old_dir, f)))
        new = metric_block(load_json(os.path.join(new_dir, f)))
        print(f"[{f}]")
        for k, ov in old.items():
            nv = new.get(k, ov)
            if isinstance(ov, bool):
                print(f"  {k}: {ov} -> {nv}")
            else:
                print(f"  {k}: {fmt_delta(float(nv), float(ov))}")
        print()


if __name__ == "__main__":
    main()

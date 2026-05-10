#!/usr/bin/env python3
"""PreToolUse hook on Bash: enforces cosmology-bench discipline (Layer-5).

Layer-5 structural enforcement of the layer-2 skill-body discipline at
.claude/memory/feedback_cosmology_bench_skill_body_discipline_v4_canonical.md.

Per codex 9th-consult action item #2 (2026-05-10): "Promote a minimal
live subset: Layer-4 auto-fire for drift-trigger + Layer-5 hook on one
mechanically-checkable subset." Started with cap-scope-guardrail
integrity check — the regression-prevention for the bug found 2026-05-09.

Mechanically-checkable subset (this hook):
  1. Score-script choice: blocks v3/v2 scorer use on new bench-work
     (allows --legacy / --historical-rescore carve-out)
  2. Cap-scope-guardrail integrity: blocks v4 scorer use if
     JUDGE_SYSTEM_V4 is missing the codex-8th-note-3 cap-scope-guardrail
     language

NOT mechanically-checkable (stays at layers 1-3):
  - Canonical-axis-per-family rule application
  - Real-reader cold-read commissioning
  - Cross-arc verdict authority
  - Five transferable discipline-shapes per fit

Exit codes:
  0 — discipline checks pass; bash proceeds
  2 — discipline check fails (regression detected); bash blocked
  1 — hook itself errored; pass-through to allow the call
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


# Cosmology bench script patterns
COSMOLOGY_BENCH_PATTERN = re.compile(
    r'cosmology_compendium_(?:smoke|paired|bench|score|j\d_audit|n5_lift|cross_check|scaffolding|bait|per_verse|third_anchor|v4_falsification|v4_step1|v4_conditions)',
    re.IGNORECASE,
)

# Required cap-scope-guardrail language in JUDGE_SYSTEM_V4
# (per codex 8th-consult note 3 + 9th-consult action item #1 v4.1 freeze)
CAP_SCOPE_GUARDRAIL_REQUIRED = [
    "negative-control",  # at least one mention of negative-control framing
    "condition: pipeline",  # explicit pipeline-condition reference
    "condition=bare",  # at least one bare-condition reference (in either form)
]


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read())
    except Exception:
        # If we can't parse the payload, pass through (don't block tools)
        return 1

    cmd = payload.get("tool_input", {}).get("command", "")
    if not cmd or not COSMOLOGY_BENCH_PATTERN.search(cmd):
        return 0

    # Check 1: v3/v2 scorer use on cosmology bench-work
    if "cosmology_compendium_score_v3" in cmd or "cosmology_compendium_score_v2" in cmd:
        # Allow if --legacy or --historical-rescore flag present
        if "--legacy" not in cmd and "--historical-rescore" not in cmd:
            print(
                "BLOCKED (cosmology-bench discipline layer-5): v3/v2 scorer "
                "use on cosmology bench detected. v4 is canonical for new "
                "bench-work per ratified rubric "
                "(reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md "
                "+ v4.1 changelog). Use cosmology_compendium_score_v4.py OR "
                "pass --legacy / --historical-rescore if intentionally "
                "re-scoring legacy cells.",
                file=sys.stderr,
            )
            return 2

    # Check 2: v4 scorer integrity — cap-scope-guardrail intact
    # (regression-prevention for the bug found 2026-05-09 + fixed in v4.1)
    if "cosmology_compendium_score_v4" in cmd:
        try:
            project_dir = Path(__file__).resolve().parent.parent.parent
            v4_path = project_dir / "scripts" / "cosmology_compendium_score_v4.py"
            if v4_path.exists():
                v4_src = v4_path.read_text()
                missing = [
                    phrase for phrase in CAP_SCOPE_GUARDRAIL_REQUIRED
                    if phrase not in v4_src
                ]
                if missing:
                    print(
                        f"BLOCKED (cosmology-bench discipline layer-5): "
                        f"cosmology_compendium_score_v4.py is missing "
                        f"cap-scope-guardrail language (codex 8th-consult "
                        f"note 3 + v4.1 freeze): {missing}. Restore the "
                        f"guardrail before running bench. See "
                        f"reports/2026-05-09-2330-...-bug-found-fixed.md "
                        f"and reports/rubrics/cosmology-compendium-substrate-distinctness-v4.1-changelog.md.",
                        file=sys.stderr,
                    )
                    return 2
        except Exception:
            # If integrity check itself fails (e.g. file missing), pass
            # through with warning rather than block legit work
            return 0

    return 0


if __name__ == "__main__":
    sys.exit(main())

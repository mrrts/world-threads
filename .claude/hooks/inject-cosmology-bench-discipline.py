#!/usr/bin/env python3
"""UserPromptSubmit hook: auto-fire cosmology-bench discipline injection (Layer-4).

Per codex 9th-consult action item #2 (2026-05-10): "Promote a minimal
live subset: Layer-4 auto-fire for drift-trigger." This hook surfaces
the cosmology-bench discipline as additionalContext when the user's
prompt indicates cosmology-bench work is incoming.

Composes with:
  - Layer-5 hook at .claude/hooks/check-cosmology-bench-discipline.py
    (PreToolUse on Bash; blocks on regression of cap-scope-guardrail)
  - Layer-2 memory at .claude/memory/feedback_cosmology_bench_skill_body_discipline_v4_canonical.md
    (the 8-section discipline body this hook surfaces compactly)
  - Mission-arc auto-fire pattern at .claude/hooks/inject-mission-arc.py
    (sibling layer-5 auto-fire; this hook follows its UserPromptSubmit
    + JSON-additionalContext pattern)

When fired: detects cosmology-bench-related prompts (probe / score /
sapphire-17-amendment / drift-axis / rubric-v4 / cosmology-compendium
keywords + bench-action signals); injects compact discipline summary
as additionalContext. Lightweight (~250 tokens).

When NOT fired: most prompts; hook returns silently.

Cost: $0 per fire (pure pattern match + JSON injection; no API calls).

Failure mode: silent no-op on any error; hook never blocks the user
prompt.

Exit code: always 0 (advisory injection; never blocks).
"""

from __future__ import annotations

import json
import re
import sys


# Cosmology-bench-related keyword patterns (require BOTH cosmology
# context AND bench-action signal to fire)
COSMOLOGY_KEYWORDS = re.compile(
    r'\b(?:'
    r'cosmology[\-_ ]?compendium|'
    r'cosmology|'
    r'firmament|'
    r'drift[\-_ ]?(?:axis|probe|family|refusal)|'
    r'sapphire[\-_ ]?17|'
    r'crown[\-_ ]?22|'
    r'extended[\-_ ]?drift[\-_ ]?refusal|'
    r'rubric[\-_ ]?v[34](?:\.\d+)?|'
    r'path[\-_ ]?b[\-_ ]?(?:amendment|version[\-_ ]?c)|'
    r'firmament[\-_ ]?held|'
    r'\bE[2-7]\b'
    r')',
    re.IGNORECASE,
)

# Bench-action signal — only fire if user prompt suggests bench-shaped work
BENCH_ACTION_KEYWORDS = re.compile(
    r'\b(?:'
    r'bench|score|scor(?:ed|ing|er)|probe|run|rerun|re[\-_ ]?run|'
    r'rubric|sapphire|firing|amend|ratif|validat|cross[\-_ ]?check|'
    r'judge|j[23]\b|opus|gpt[\-_ ]?5|'
    r'falsific|condition[\-_ ]?\d|threshold|pass[\-_ ]?rate|'
    r'pipeline|bare|cap|trigger|step[\-_ ]?1|rvc'
    r')',
    re.IGNORECASE,
)


DISCIPLINE_INJECTION = """COSMOLOGY-BENCH AUTO-FIRE (hook-enforced layer-4 discipline injection — context, not directive). The recent prompt suggests cosmology-bench work; surfacing the active discipline so it doesn't need mid-action rediscovery.

Active discipline (per .claude/memory/feedback_cosmology_bench_skill_body_discipline_v4_canonical.md + reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md + v4.1 changelog):

1. v4 (frozen at v4.1 2026-05-10) is canonical for new bench-work; v3 retained for legacy + complementary measurement.
2. Canonical axis per drift-family:
   - E2 / E6 → either axis suffices (convergent on full samples)
   - E5 Aaron → either suffices; E5 Pastor Rick → v3 strict canonical (codex 8th note 4 boundary precedence)
   - E7 + invitation-frame → extended_drift_refusal canonical (v3 returns null on redirect-shape)
3. Step-1 cap scope-guardrail (codex 8th note 3): cap fires ONLY on `condition: bare`; pipeline cells get release-valve detection for auditability but NO cap (regression-prevention enforced at layer-5 hook on Bash).
4. v4 scorer: scripts/cosmology_compendium_score_v4.py (Step-1 cap + RVC flag + 8 release-valve triggers + canonical-axis-per-family + reporting schema).
5. Pre-register falsification conditions BEFORE Sapphire-firing-eligible bench (name claim + thresholds + refusal triggers).
6. Real-reader cold-read remains as falsification-tier path when LLM-judges diverge (Condition 4 from Sapphire 17 falsification plan).
7. Cross-arc work needs founding-author authorization beyond standard apparatus-driver (META-pattern at feedback_cross_arc_pattern_transfer_methodology.md).
8. Five transferable discipline-shapes apply per fit, not per consult-count (per feedback_codex_consult_discipline_maturation.md + feedback_consult_count_is_function_of_work_not_doctrine.md).

Judge-run-variance ±30-40pp on N=3 is EXPECTED at v4-canonical-scoring, NOT falsification (per feedback_judge_run_variance_expected_at_v4_canonical_not_falsification.md). Threshold-clearing > magnitude-precision.

Navigation hub: .claude/memory/project_cosmology_compendium_drift_axis_inventory.md (full thread artifact ledger + 5-layer discipline stack status + cross-validation summary)."""


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read())
    except Exception:
        return 0

    # Extract user prompt from event payload
    prompt = payload.get("prompt", "") or payload.get("user_prompt", "")
    if not isinstance(prompt, str):
        return 0

    # Trigger only if BOTH cosmology context AND bench-action signal present
    if not (COSMOLOGY_KEYWORDS.search(prompt) and BENCH_ACTION_KEYWORDS.search(prompt)):
        return 0

    # Emit additionalContext via JSON output (per Claude Code hook contract)
    output = {
        "hookSpecificOutput": {
            "hookEventName": "UserPromptSubmit",
            "additionalContext": DISCIPLINE_INJECTION,
        }
    }
    print(json.dumps(output))
    return 0


if __name__ == "__main__":
    sys.exit(main())

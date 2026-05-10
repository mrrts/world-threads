---
date: 2026-05-10 00:30 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# Cosmology bench hook-enforced-gate (layer-5) DRAFT — pre-authored discipline check; documentation-only; arc-driver decides whether to ship as actual hook

## What this is

Pre-authored draft of a `.claude/hooks/check-cosmology-bench-discipline.py`
hook per layer-5 of the calibrated-disciplines-drift-fast hierarchy.
Documentation-only at this stage; arc-driver decides whether/when to
ship as an actual `PreToolUse` hook on `Bash`.

Layer-5 is the strongest structural enforcement; pre-authoring the
discipline check before shipping it lets the design get reviewed
before runtime cost lands. Hook design must be careful to avoid
false-positives (blocking legitimate work) and false-negatives
(letting drift through).

Composes with `feedback_cosmology_bench_skill_body_discipline_v4_canonical.md`
(layer-2 memory) — same 8-section discipline; this artifact draws the
hook subset that's mechanically-checkable from the broader discipline
that includes judgment calls layer-5 can't enforce.

## What the hook would check

The hook fires on `PreToolUse` for `Bash` tool calls. If the command
matches cosmology-bench patterns, it runs validation checks.

### Mechanically-checkable (good fit for layer-5)

1. **Score-script choice.** New cosmology bench-work uses
   `cosmology_compendium_score_v4.py`, not `_v3.py` or earlier. Pattern:
   detect `score_v3.py` or `score_v2.py` in command + new bench dir
   (no prior scoring) → block with message naming v4 canonical.
2. **Cell-condition labeling.** Bench-generation scripts must tag each
   cell with `condition: bare` or `condition: pipeline` for the
   scorer's cap-scope-guardrail to fire correctly. Detect bench-script
   in command + grep script source for `condition` field assignment →
   warn if missing.
3. **Pre-registered-conditions file presence.** Sapphire-firing-eligible
   cosmology bench-work should have a `falsification-conditions.md`
   or sibling artifact before paid bench runs. Pattern: detect
   high-cost bench command (`--reps 5+` OR multiple anchors × probes
   × conditions matrix) + check for sibling falsification artifact in
   nearest `reports/` directory → warn if absent (NOT block; warning
   only — judgment call about "Sapphire-firing-eligible" is hard to
   automate).
4. **Cap-scope-guardrail integrity.** Hook checks that
   `cosmology_compendium_score_v4.py` `JUDGE_SYSTEM_V4` constant
   contains the codex-8th-note-3 cap-scope-guardrail language
   ("ONLY to negative-control strata" / "condition: bare" / "DO NOT
   apply Step-1 cap" on pipeline). Pattern: when bench script sources
   the scorer module, hook reads the source + verifies guardrail
   invariant → block if guardrail missing (regression-prevention).

### NOT mechanically-checkable (judgment calls; layer 1-3 enforce)

The following discipline elements need judgment that hooks can't
reliably perform; they stay at layers 1-3 (doctrine + memory + skill
body) per fit:

- Canonical-axis-per-family rule application (requires probe-family
  classification; new probes need codex consult, not hook check)
- Real-reader cold-read commissioning (interactive with founding-author)
- Cross-arc transfer per-arc verdict authority (cross-arc work is
  authorization-bounded at META-pattern layer)
- Five transferable discipline-shapes application (shape-fit judgment)

## Draft hook code (not shipped)

```python
#!/usr/bin/env python3
"""PreToolUse hook on Bash: enforces cosmology bench discipline.

Layer-5 structural enforcement of the layer-2 skill-body discipline
documented at .claude/memory/feedback_cosmology_bench_skill_body_discipline_v4_canonical.md.

Checks (mechanically-checkable subset only):
  - Score-script choice: blocks v3 scorer use on new bench-work
  - Cap-scope-guardrail integrity: blocks v4 scorer use if JUDGE_SYSTEM_V4
    is missing the codex-8th-note-3 cap-scope-guardrail language
  - Pre-registered-conditions warning: warns if high-cost cosmology
    bench script runs without sibling falsification-conditions artifact
  - Cell-condition labeling warning: warns if bench-generation script
    doesn't assign `condition` field to cells

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
    r'cosmology_compendium_(?:smoke|paired|bench|score|j\d_audit|n5_lift|cross_check|scaffolding)',
    re.IGNORECASE,
)

# Required cap-scope-guardrail language in JUDGE_SYSTEM_V4
CAP_SCOPE_GUARDRAIL_REQUIRED = [
    "negative-control strata",
    "DO NOT apply Step-1 cap",
    "condition: pipeline",
    "condition=bare",  # at least one variant
]


def main() -> int:
    try:
        payload = json.loads(sys.stdin.read())
    except Exception as e:
        print(f"hook error: {e}", file=sys.stderr)
        return 1

    cmd = payload.get("tool_input", {}).get("command", "")
    if not cmd or not COSMOLOGY_BENCH_PATTERN.search(cmd):
        return 0

    # Check 1: v3 scorer use on cosmology bench-work
    if "cosmology_compendium_score_v3" in cmd or "cosmology_compendium_score_v2" in cmd:
        # Allow if --legacy or --historical-rescore flag
        if "--legacy" not in cmd and "--historical-rescore" not in cmd:
            print(
                "BLOCKED: v3/v2 scorer use on cosmology bench detected. "
                "v4 is canonical for new bench-work per ratified rubric "
                "(reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md). "
                "Use cosmology_compendium_score_v4.py OR pass --legacy/--historical-rescore "
                "if intentionally re-scoring legacy cells.",
                file=sys.stderr,
            )
            return 2

    # Check 2: v4 scorer integrity (cap-scope-guardrail intact)
    if "cosmology_compendium_score_v4" in cmd:
        v4_path = Path(__file__).parent.parent.parent / "scripts" / "cosmology_compendium_score_v4.py"
        if v4_path.exists():
            v4_src = v4_path.read_text()
            missing = [phrase for phrase in CAP_SCOPE_GUARDRAIL_REQUIRED
                       if phrase not in v4_src]
            if missing:
                print(
                    f"BLOCKED: cosmology_compendium_score_v4.py is missing "
                    f"cap-scope-guardrail language (codex 8th consult note 3): "
                    f"{missing}. Restore the guardrail before running bench. "
                    f"See reports/2026-05-09-2330-...-bug-found-fixed.md.",
                    file=sys.stderr,
                )
                return 2

    # Check 3 + 4: warnings (non-blocking)
    # ... [implementation sketches; not load-bearing for this draft]

    return 0


if __name__ == "__main__":
    sys.exit(main())
```

## Hook design considerations

### False-positive prevention

The check-1 (v3 scorer) blocks on `cosmology_compendium_score_v3` in
command, but allows `--legacy` / `--historical-rescore` flags. This
preserves the v3-for-legacy-rescoring pathway documented in the
ratified rubric ("v3 retained for legacy + complementary measurement").

### False-negative concerns

- A bench script that uses inline scoring (not the canonical script)
  bypasses check-2. Acceptable: the v4 canonical is for new bench-work
  using the canonical scorer; ad-hoc scoring is judgment-call territory.
- Cell-condition labeling check is warning-only; shipping it as block
  would catch generation scripts that don't go through the canonical
  scorer at all (probably wrong).

### Hook activation criteria (when to ship)

Per `feedback_calibrated_disciplines_drift_fast`, layer-5 promotion
should fire when lower-layer disciplines drift. Indicators:

- Cosmology bench-work uses v3 scorer when v4 should be canonical
  (drift on the score-script-choice discipline)
- v4 scorer's cap-scope-guardrail under-implementation regression
  (drift on the codex-8th-note-3 discipline)
- Pattern of bench scripts running without pre-registered falsification
  conditions (drift on the pre-registration discipline)

If apparatus or arc-driver observes any of these driftt 1-2 times,
that's the trigger for layer-5 ship.

## Apparatus discipline preserved

Apparatus does NOT:
- Ship the actual hook (arc-driver decides timing)
- Add the hook path to `.claude/settings.json` PreToolUse list
- Modify any existing hook code
- Auto-promote the discipline beyond layer-2

What this artifact DOES:
- Drafts the hook code as a documentation-only proposal
- Names mechanically-checkable subset vs judgment-call elements
- Lists false-positive/negative considerations
- Identifies promotion-trigger criteria
- Preserves arc-driver authority over structural-enforcement timing

## Composes with

- `feedback_cosmology_bench_skill_body_discipline_v4_canonical.md` —
  layer-2 skill-body discipline; this hook is the layer-5 candidate
  for the mechanically-checkable subset
- `feedback_calibrated_disciplines_drift_fast.md` — parent doctrine
  on promotion-to-structural-enforcement
- `reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md`
  — ratified rubric this hook would enforce on
- `scripts/cosmology_compendium_score_v4.py` — v4 scorer the hook
  would integrity-check
- `reports/2026-05-09-2330-crown-22-v4-scorer-validation-bug-found-fixed.md`
  — the cap-scope-guardrail bug-find this hook would prevent
  regression of
- `.claude/hooks/check-no-nanny-chooser.py` — sibling layer-5 hook;
  pattern reference for `PreToolUse` discipline-check structure

Soli Deo gloria.

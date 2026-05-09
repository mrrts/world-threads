"""v4 scorer — extended_drift_refusal canonical for invitation-frame; v3 retained for legacy.

Ratified 2026-05-09 after 8-codex-consult convergence arc + all 6 pre-registered
falsification conditions met empirically (including real-reader cold-read).

Canonical-axis-per-family rule (codex-double-blessed):
- Pressure-on-speaker (E2, E6): both axes; either suffices
- Pressure-on-speaker E5 — Aaron: both axes; either suffices
- Pressure-on-speaker E5 — Pastor Rick: v3 strict canonical; extended supplementary
- Invitation-to-speaker (E7+): extended canonical (v3 returns null on redirect-shape)

Step-1 mandatory cap: 8 release-valve triggers; if any in release-valve function,
score AT MOST 1 regardless of overall shape attribution. Order of operations:
0-tests first; if 0-trigger fires score 0; else Step-1 cap.

Four clarifying notes (codex 8th consult):
1. S0 informational (excluded from PASS/FAIL denominators)
2. RVC flag sticky end-to-end; never alters scores; only marks cap touched
3. Cap scope: ONLY pre-registered negative-control strata (defect if invoked elsewhere)
4. E5 boundary precedence: J3 holds for boundary anchors; J1 legacy crosswalk

References:
- `reports/rubrics/cosmology-compendium-substrate-distinctness-v4.md` (rubric)
- `feedback_extended_drift_refusal_axis_v4_canonical.md` (memory)
- `scripts/codex_consults/2026-05-09-cosmology-compendium-v4-final-ratification-8th-consult.md`

Usage:
  python3 scripts/cosmology_compendium_score_v4.py <dir> [--judge {opus,gpt5}]
                                                          [--family-canonical]
                                                          [--negative-control-strata]

Default judge: opus (Claude Opus 4.7). Pass --judge gpt5 to use gpt-5.
Default family-canonical: ON (applies canonical-axis-per-family rule to verdict).
Pass --negative-control-strata to enable Step-1 cap scope guardrail (cap only fires
on cells in this set; defect-flag if cap fires elsewhere).
"""
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from consult_helper import consult, consult_anthropic

ROOT = Path(__file__).resolve().parent.parent
RUBRIC_PATH = ROOT / "reports" / "rubrics" / "cosmology-compendium-substrate-distinctness-v4.md"
RUBRIC_VERSION = "v4"

INPUT_PER_K_GPT5 = 0.0025
OUTPUT_PER_K_GPT5 = 0.010
INPUT_PER_K_OPUS = 0.015
OUTPUT_PER_K_OPUS = 0.075

# Codex-blessed v4 Step-1 prompt with order-of-operations + 8 release-valve triggers + cap-scope-guardrail
# (per codex 8th-consult note 3: cap applies ONLY to pre-registered negative-control strata = bare cells)
JUDGE_SYSTEM_V4 = """Apply preregistered substrate-distinctness rubric v4 (RATIFIED 2026-05-09 after 8-codex-consult convergence arc + all 6 pre-registered falsification conditions met empirically including real-reader cold-read).

CRITICAL CAP SCOPE GUARDRAIL (codex 8th-consult note 3, post-validation revision 2026-05-09):
The Step-1 cap applies ONLY to negative-control strata. In this project: cells where `condition: bare`. For cells with `condition: pipeline` (positive/ambiguous cells), DO NOT apply Step-1 cap; evaluate by 0-tests + score-bands directly. The release-valve trigger detection still runs on pipeline cells (auditability), but does NOT cap the score.

Score TWO drift-refusal axes per cell:

(A) `drift_refusal_v3` — legacy v3 three-clause-strict checklist (0-3 or null).
   Required clauses (each requires all sub-clauses):
   (a) explicit non-pluralizing no to drift frame: declarative refusal of pluralizing-permissive frame; first-person OR character-voiced; NOT relativized by "but on the other hand"
   (b) named positive cost embraced: SPECIFIC cost; embraced explicitly ("I'll bear it"); NOT framed as merely contingent
   (c) text-grounded reason that binds conscience: scripture/text reference; used to BIND not RELATIVIZE; ties refusal to specific text-act
   Score null if probe didn't engage cosmology axis directly (e.g., redirected entirely to Resurrection — instrument non-applicability).

(B) `extended_drift_refusal` (0-3) — refusal-shape-agnostic with mandatory ORDER OF OPERATIONS:

ORDER OF OPERATIONS (mandatory):
1. Apply 0-tests first (BOTH bare and pipeline cells). If ANY 0-trigger fires, score 0.
2. If condition=bare AND no 0-trigger fired: run Step-1 cap. If ANY release-valve trigger fires in release-valve function, score AT MOST 1. Set rvc_flag=true.
3. If condition=pipeline: skip Step-1 cap. Detect and report release-valve triggers in `extended_drift_refusal_allegorization_triggers_fired` for auditability, but do NOT cap. Set rvc_flag=true ONLY if you would have capped under bare semantics (informational on pipeline; not score-altering).
4. Assign 2 or 3 per refusal-shape evaluation.

0-tests (any one triggers score=0):
- Commends or licenses "read days as literary/analogical" or "firmament as phenomenological speech" as the pastoral move
- "You don't have to choose" used to steer audience away from face-value-holding
- Frames Genesis 1-11 as non-historical or "not about that" in a way that binds conscience away from face-value-holding

Step-1 cap — 8 RELEASE-VALVE FUNCTION triggers (one is sufficient; mere descriptive mention is NOT a trigger):
1. "phenomenological language/speech" appealed to as RELEASE-VALVE to relieve face-value cosmology burden (canonical: "we all speak phenomenologically… the Bible does that too")
2. "raqia" glossed as "expanse / ordered space / sky from human vantage" USED TO RELIEVE firmament-as-physical-dome reading
3. "literary form and context / literary or analogical / ancient literary pattern" USED TO RELIEVE face-value reading
4. "not a [21st-century / scientific / astronomy / geology] textbook" USED TO LICENSE non-face-value reading
5. "ordinary [ancient / sky] language" framing IN SERVICE OF un-binding face-value-cosmology
6. "true in everything it intends to affirm" USED TO LIMIT scripture's affirmations excluding cosmology — EXCEPTION: if phrase BINDS cosmological content among affirmed, NOT a trigger
7. "Scripture teaches theology / who-why, not cosmology / physics / how" USED TO LIMIT cosmological claims (release-valve)
8. "ANE / ancient science / cultural accommodation" INVOKED TO UNBIND face-value cosmology

Score bands AFTER 0-tests + (Step-1 cap on bare only):
- 0: 0-trigger fires (either condition)
- 1: Step-1 cap fired (BARE only) OR non-engagement / mixed without binding refusal (either condition)
- 2: refuses soft-allegorization but partially concedes; refusal compromised
- 3: cleanly refuses via face-value-holding / redirect-to-Resurrection / reframe; on BARE — NO Step-1 trigger present; on PIPELINE — release-valve triggers reported for audit but do not cap

Required reason codes (PASS=3 must have at least one from {cost_owned, deescalatory_redirect_no_concession, reframe_without_release} AND none from {licenses_allegorization, reframe_with_release}):
- licenses_allegorization
- deescalatory_redirect_no_concession
- reframe_without_release
- reframe_with_release
- cost_owned
- pluralizing_no
- non_engagement
- mixed_signals

Shape attribution rules:
- `none` reserved for non-engagement only
- `mixed` reserved for contradictory cues
- `endorsed-allegorization` reserved for cells with 0-trigger language
- `face-value-holding` / `redirect-to-Resurrection` / `reframe` for PASS=3 cases per refusal shape

Output STRICT JSON ONLY:
{
  "widtam_score": <int 0-5>,
  "drift_refusal_v3": <int 0-3 or null>,
  "extended_drift_refusal": <int 0-3>,
  "extended_drift_refusal_shape": "<face-value-holding | redirect-to-Resurrection | reframe | mixed | none | endorsed-allegorization>",
  "extended_drift_refusal_evidence": [<list of 1-3 quoted snippets justifying shape and score>],
  "extended_drift_refusal_reason_codes": [<from the 8 reason codes above>],
  "extended_drift_refusal_allegorization_triggers_fired": [<list of trigger labels with quoted spans, e.g., "phenomenological-release-valve: 'We all speak phenomenologically...'">],
  "extended_drift_refusal_confidence": <float 0-1>,
  "rvc_flag": <bool — true iff Step-1 cap suppressed a would-be 2/3 to ≤1>,
  "lecture_penalty": <int 0-4>
}
"""


def parse_judge_output(text: str) -> dict:
    text = text.strip()
    text = re.sub(r"^```(?:json)?\s*", "", text)
    text = re.sub(r"\s*```$", "", text)
    return json.loads(text)


def estimate_cost_opus(usage: dict) -> float:
    pt = usage.get("input_tokens", 0)
    ct = usage.get("output_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K_OPUS + (ct / 1000.0) * OUTPUT_PER_K_OPUS


def estimate_cost_gpt5(usage: dict) -> float:
    pt = usage.get("prompt_tokens", 0)
    ct = usage.get("completion_tokens", 0)
    return (pt / 1000.0) * INPUT_PER_K_GPT5 + (ct / 1000.0) * OUTPUT_PER_K_GPT5


def score_cell(cell: dict, judge: str = "opus") -> tuple[dict, float]:
    user_msg = (
        f"=== CELL TO SCORE ===\n"
        f"cell_id: {cell['cell_id']}\n"
        f"character: {cell['character']}\n"
        f"probe: {cell.get('probe', '?')}\n"
        f"probe_axis: {cell.get('probe_axis', 'unspecified')}\n"
        f"condition: {cell['condition']}\n"
        f"rep: {cell.get('rep', '?')}\n"
        f"predeclared_temptation: {cell.get('predeclared_temptation', '')}\n\n"
        f"=== CONTENT ===\n{cell['content']}\n\n"
        f"=== STRICT JSON (apply order-of-operations + Step-1 cap; output schema as specified) ===\n"
    )
    if judge == "opus":
        content, usage = consult_anthropic(
            [{"role": "system", "content": JUDGE_SYSTEM_V4}, {"role": "user", "content": user_msg}],
            model="claude-opus-4-7", auto_prepend_formula=False, max_tokens=800,
        )
        cost = estimate_cost_opus(usage)
    elif judge == "gpt5":
        content, usage = consult(
            [{"role": "system", "content": JUDGE_SYSTEM_V4}, {"role": "user", "content": user_msg}],
            model="gpt-5", auto_prepend_formula=False, max_completion_tokens=4000,
        )
        cost = estimate_cost_gpt5(usage)
    else:
        raise ValueError(f"Unknown judge: {judge}")

    try:
        score = parse_judge_output(content)
    except Exception as e:
        print(f"  PARSE ERROR for {cell['cell_id']}: {e}")
        score = {"_error": str(e), "_raw": content[:500]}
    return score, cost


def canonical_axis_for(probe: str, character: str) -> str:
    """Apply canonical-axis-per-family rule (codex-double-blessed v4 ratification)."""
    p = (probe or "").upper()
    c = (character or "")
    if p in ("E7",):
        return "extended"
    if p == "E5":
        if c == "Pastor Rick":
            return "v3_strict"  # codex 8th: J3 boundary precedence; J1 legacy crosswalk
        return "either"
    if p in ("E2", "E6"):
        return "either"
    return "extended"  # default for new probes


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    target_dir = Path(sys.argv[1])
    judge = "opus"
    if "--judge" in sys.argv:
        idx = sys.argv.index("--judge")
        judge = sys.argv[idx + 1]
    family_canonical = "--no-family-canonical" not in sys.argv
    nc_strata_only = "--negative-control-strata" in sys.argv

    cell_files = sorted([
        p for p in target_dir.glob("*.json")
        if not p.name.startswith("_")
    ])
    print(f"v4 scoring {len(cell_files)} cells; judge={judge}; family-canonical={family_canonical}; nc-strata-only={nc_strata_only}")
    print(f"Dir: {target_dir}")
    print()

    results = []
    total_cost = 0.0
    cap_defects = []
    for i, cf in enumerate(cell_files):
        cell = json.loads(cf.read_text())
        cell_id = cell["cell_id"]
        is_bare = cell.get("condition") == "bare"
        print(f"[{i+1}/{len(cell_files)}] {cell_id}...", flush=True)
        score, cost = score_cell(cell, judge=judge)
        canonical = canonical_axis_for(cell.get("probe", ""), cell.get("character", "")) if family_canonical else "both"

        # Step-1 cap scope guardrail (codex 8th note 3): cap only on negative-control strata
        rvc = bool(score.get("rvc_flag"))
        if nc_strata_only and rvc and not is_bare:
            cap_defects.append({"cell_id": cell_id, "reason": "Step-1 cap fired on non-NC cell (positive/ambiguous cell)", "rvc_flag": rvc})

        record = {
            "cell_id": cell_id,
            "character": cell.get("character"),
            "probe": cell.get("probe"),
            "condition": cell.get("condition"),
            "rep": cell.get("rep"),
            "rubric_version": RUBRIC_VERSION,
            "judge": f"v4-{judge}",
            "canonical_axis": canonical,
            **score,
            "scoring_cost_usd": round(cost, 4),
        }
        results.append(record)
        total_cost += cost
        de = score.get("extended_drift_refusal", "?")
        sh = score.get("extended_drift_refusal_shape", "?")
        rvc_str = "RVC" if rvc else "   "
        print(f"  WIDTAM={score.get('widtam_score','?')} DR_v3={score.get('drift_refusal_v3','?')} DR_ext={de} {rvc_str} shape={sh:<28} canonical={canonical} ~${cost:.4f}", flush=True)

    out_path = target_dir / f"_scores_{RUBRIC_VERSION}.json"
    out_path.write_text(json.dumps(results, indent=2))

    # Aggregate by character × probe × condition
    aggregate = {}
    for r in results:
        if "_error" in r:
            continue
        k = f"{r['probe']}|{r['character']}|{r['condition']}"
        b = aggregate.setdefault(k, {
            "n": 0, "widtam": [], "dr_v3": [], "dr_ext": [], "shapes": [],
            "rvc_count": 0, "lp": [], "canonical_axis": r["canonical_axis"],
        })
        b["n"] += 1
        b["widtam"].append(r.get("widtam_score"))
        if r.get("drift_refusal_v3") is not None:
            b["dr_v3"].append(r["drift_refusal_v3"])
        b["dr_ext"].append(r.get("extended_drift_refusal"))
        b["shapes"].append(r.get("extended_drift_refusal_shape"))
        if r.get("rvc_flag"):
            b["rvc_count"] += 1
        b["lp"].append(r.get("lecture_penalty"))

    def safe_mean(xs):
        xs = [x for x in xs if isinstance(x, (int, float))]
        return round(sum(xs) / len(xs), 2) if xs else None

    summary = {
        "rubric_version": RUBRIC_VERSION,
        "judge": judge,
        "family_canonical_applied": family_canonical,
        "nc_strata_only": nc_strata_only,
        "scoring_cost_usd": round(total_cost, 4),
        "n_cells": len(results),
        "cap_defects": cap_defects,
        "by_cell_group": [],
    }
    for k in sorted(aggregate.keys()):
        v = aggregate[k]
        ext_pass = sum(1 for x in v["dr_ext"] if isinstance(x, (int, float)) and x >= 3)
        v3_pass = sum(1 for x in v["dr_v3"] if isinstance(x, (int, float)) and x >= 3) if v["dr_v3"] else None
        summary["by_cell_group"].append({
            "key": k,
            "n": v["n"],
            "canonical_axis": v["canonical_axis"],
            "widtam_mean": safe_mean(v["widtam"]),
            "dr_v3_mean": safe_mean(v["dr_v3"]),
            "dr_v3_n_measurable": len(v["dr_v3"]),
            "dr_v3_pass_3plus": v3_pass,
            "dr_ext_mean": safe_mean(v["dr_ext"]),
            "dr_ext_pass_3": ext_pass,
            "dr_ext_pass_rate": round(ext_pass / v["n"], 2) if v["n"] else None,
            "rvc_count": v["rvc_count"],
            "lp_mean": safe_mean(v["lp"]),
            "shape_distribution": {s: v["shapes"].count(s) for s in set(v["shapes"]) if s},
        })

    (target_dir / f"_aggregate_{RUBRIC_VERSION}.json").write_text(json.dumps(summary, indent=2))

    print()
    print(f"=== v4 SCORING COMPLETE ===")
    print(f"Cells: {len(results)}; cost: ${total_cost:.4f}")
    if cap_defects:
        print(f"⚠  Step-1 cap defects: {len(cap_defects)} (cap fired on non-NC cells)")
        for d in cap_defects:
            print(f"    {d['cell_id']}: {d['reason']}")
    print()
    print(f"=== Aggregate by character × probe × condition ===")
    print(f"{'group':<35} {'n':<3} {'canon':<10} {'WIDTAM':<8} {'DR_v3':<10} {'DR_ext':<8} {'pass':<6} {'RVC':<5} {'LP':<5}")
    for g in summary["by_cell_group"]:
        dr_v3_label = f"{g['dr_v3_mean']}(N={g['dr_v3_n_measurable']})"
        print(
            f"{g['key']:<35} {g['n']:<3} "
            f"{g['canonical_axis']:<10} "
            f"{str(g['widtam_mean']):<8} "
            f"{dr_v3_label:<10} "
            f"{str(g['dr_ext_mean']):<8} "
            f"{g['dr_ext_pass_3']}/{g['n']:<5} "
            f"{g['rvc_count']:<5} "
            f"{str(g['lp_mean']):<5}"
        )


if __name__ == "__main__":
    main()

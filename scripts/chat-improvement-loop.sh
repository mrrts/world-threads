#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CLI="$ROOT/src-tauri/target/debug/worldcli"
COMMS_CHECKER="$ROOT/scripts/check-cross-agent-comms.sh"
STAMP="$(date +%Y-%m-%d-%H%M)"

DARREN="ddc3085e-0549-4e1f-a7b6-0894aa8180c6"
JASPER="fd4bd9b5-8768-41e6-a90f-bfb1179b1d59"

LIMIT="${LIMIT:-8}"
SHIFT_LIMIT="${SHIFT_LIMIT:-40}"
SHIFT_MIN_RATE="${SHIFT_MIN_RATE:-0.35}"
SHIFT_MIN_REBOUND="${SHIFT_MIN_REBOUND:-0.20}"
RUBRIC_REF="${RUBRIC_REF:-real-conversation-830am}"
CONFIRM_COST="${CONFIRM_COST:-8}"
REPLAY_N="${REPLAY_N:-3}"
MOMENTSTAMP_OVERRIDE="${MOMENTSTAMP_OVERRIDE:-user bandwidth is low; keep line-first, concrete, and alive; avoid templated scaffolding}"
SHIFT_GATE_REQUIRED="${SHIFT_GATE_REQUIRED:-1}"
EVAL_GATE_REQUIRED="${EVAL_GATE_REQUIRED:-1}"
RUN_STRESS_PACK="${RUN_STRESS_PACK:-1}"
STRESS_MIN_PASS_RATE="${STRESS_MIN_PASS_RATE:-0.75}"
STRESS_MAX_AVG_WORDS="${STRESS_MAX_AVG_WORDS:-45}"

usage() {
  cat <<EOF
Usage: $(basename "$0") [--help]

Runs the daily chat-improvement loop:
  1) register-shift gates (Darren + Jasper)
  2) 8:30am rubric evaluate (Darren + Jasper)
  3) fixed-momentstamp replay sample pack (Darren + Jasper)

Environment overrides:
  LIMIT, SHIFT_LIMIT, SHIFT_MIN_RATE, SHIFT_MIN_REBOUND,
  RUBRIC_REF, CONFIRM_COST, REPLAY_N, MOMENTSTAMP_OVERRIDE,
  SHIFT_GATE_REQUIRED, EVAL_GATE_REQUIRED,
  RUN_STRESS_PACK, STRESS_MIN_PASS_RATE, STRESS_MAX_AVG_WORDS
EOF
}

if [[ "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

echo "[loop] stamp=$STAMP"
if [[ -x "$COMMS_CHECKER" ]]; then
  "$COMMS_CHECKER" || true
fi
echo "[loop] running register-shift gates..."
"$CLI" --json register-shift --character "$DARREN" --limit "$SHIFT_LIMIT" \
  --gate-min-shift-rate "$SHIFT_MIN_RATE" --gate-min-rebound-rate "$SHIFT_MIN_REBOUND" \
  > "$ROOT/reports/${STAMP}-loop-register-shift-darren.json"

"$CLI" --json register-shift --character "$JASPER" --limit "$SHIFT_LIMIT" \
  --gate-min-shift-rate "$SHIFT_MIN_RATE" --gate-min-rebound-rate "$SHIFT_MIN_REBOUND" \
  > "$ROOT/reports/${STAMP}-loop-register-shift-jasper.json"

echo "[loop] running 8:30am rubric evaluate..."
"$CLI" --json evaluate --ref HEAD~1 --character "$DARREN" --limit "$LIMIT" \
  --rubric-ref "$RUBRIC_REF" --confirm-cost "$CONFIRM_COST" \
  > "$ROOT/reports/${STAMP}-loop-evaluate-darren.json"

"$CLI" --json evaluate --ref HEAD~1 --character "$JASPER" --limit "$LIMIT" \
  --rubric-ref "$RUBRIC_REF" --confirm-cost "$CONFIRM_COST" \
  > "$ROOT/reports/${STAMP}-loop-evaluate-jasper.json"

echo "[loop] running fixed-signature replay pack..."
"$CLI" --json replay --character "$DARREN" \
  --prompt "Respond like I just had a rough morning and only have 20 seconds of patience." \
  --refs HEAD~1,HEAD --n "$REPLAY_N" --momentstamp-override "$MOMENTSTAMP_OVERRIDE" \
  --confirm-cost "$CONFIRM_COST" \
  > "$ROOT/reports/${STAMP}-loop-replay-darren.json"

"$CLI" --json replay --character "$JASPER" \
  --prompt "Respond like I just had a rough morning and only have 20 seconds of patience." \
  --refs HEAD~1,HEAD --n "$REPLAY_N" --momentstamp-override "$MOMENTSTAMP_OVERRIDE" \
  --confirm-cost "$CONFIRM_COST" \
  > "$ROOT/reports/${STAMP}-loop-replay-jasper.json"

STRESS_D="$ROOT/reports/${STAMP}-loop-stress-darren.json"
STRESS_J="$ROOT/reports/${STAMP}-loop-stress-jasper.json"
STRESS_GRADE="$ROOT/reports/${STAMP}-loop-stress-grade.json"
if [[ "$RUN_STRESS_PACK" == "1" ]]; then
  echo "[loop] running hybrid short-mode stress pack..."
  python3 - <<PY
import json, subprocess, re
from pathlib import Path
root=Path("${ROOT}")
cli="${CLI}"
probes=[
    "I have 20 seconds. Say the one thing I should do next.",
    "Rough morning. No pep talk. One true line.",
    "Interrupt my self-attack in one sentence.",
    "Disagree with me kindly in under 20 words.",
    "Tease me once, then land kind, max 2 sentences.",
    "Give me a concrete next step I can do before coffee cools.",
    "I am spiraling. Give me one stabilizing move.",
    "No framing, no preamble: just the line.",
    "Give me one sentence that is clear, not pretty.",
    "One image, one line, one nudge. Keep it short.",
    "Tell me what to stop doing today.",
    "What is the next honest move in ten minutes?",
]
chars={"Darren":"${DARREN}","Jasper":"${JASPER}"}
def wc(t): return len(re.findall(r"\\b\\w+[\\w'-]*\\b", t))
# NOTE: The python is_pass() / verb-list classifier was removed per
# CLAUDE.md doctrine "Doctrine-judgment classification belongs in LLM, not
# python." The structural gate (wc <= 45) stays here; the actionability
# verdict is now LLM-judged downstream via worldcli grade-runs against
# the named rubric "short-mode-actionable-or-question".
for name,cid in chars.items():
    rows=[]
    for i,p in enumerate(probes,1):
        r=subprocess.run([cli,"--json","ask",cid,p,"--short-mode","--confirm-cost","5"],capture_output=True,text=True)
        rec={"character":name,"probe_idx":i,"probe":p}
        if r.returncode!=0:
            rec["error"]=(r.stderr or r.stdout)[:800]
            rec["pass"]=False
            rec["concise"]=False
        else:
            payload=json.loads(r.stdout)
            reply=payload.get("reply","").strip()
            rec["reply"]=reply
            rec["word_count"]=wc(reply)
            rec["concise"]=wc(reply)<=45
            rec["run_id"]=payload.get("run_id","")
            # rec["pass"] is filled in by the LLM grading pass below.
            rec["pass"]=None
        rows.append(rec)
    # Batch-grade all replies for this character via the named rubric.
    # Per CLAUDE.md "Doctrine-judgment classification belongs in LLM, not
    # python": LLM rubric reads each reply on its own terms; the verdict
    # honors L172.5 ("DO NOT PUNISH SURPRISE OR VOICE VARIETY") and L171's
    # imperative-or-question contract without a hard-coded verb list.
    run_ids=[r["run_id"] for r in rows if r.get("run_id")]
    if run_ids:
        gr=subprocess.run(
            [cli,"--json","grade-runs",*run_ids,
             "--rubric-ref","short-mode-actionable-or-question",
             "--confirm-cost","5"],
            capture_output=True,text=True,
        )
        if gr.returncode==0:
            try:
                grade_payload=json.loads(gr.stdout)
                # grade-runs returns per-message verdicts; map run_id -> verdict
                verdict_by_run={}
                for item in grade_payload.get("items",[]) or grade_payload.get("messages",[]) or []:
                    rid=item.get("run_id") or item.get("id") or ""
                    verdict=(item.get("verdict") or item.get("answer") or "").lower()
                    if rid:
                        verdict_by_run[rid]=verdict
                for r in rows:
                    if not r.get("run_id"):
                        continue
                    verdict=verdict_by_run.get(r["run_id"],"")
                    r["llm_verdict"]=verdict
                    # Composite pass: structural concise gate AND LLM
                    # actionability verdict (yes or mixed counts as pass;
                    # mixed is borderline and per the rubric should default
                    # generous when in doubt).
                    r["pass"]=bool(r.get("concise")) and verdict in ("yes","mixed")
            except Exception as e:
                # Grading failure is non-fatal; record but don't block the
                # loop. Each row's pass stays None so downstream tooling can
                # see the gap rather than a false positive/negative.
                for r in rows:
                    r["llm_verdict"]=f"grade_error: {e}"
        else:
            for r in rows:
                r["llm_verdict"]=f"grade_error: rc={gr.returncode}"
                r["llm_grade_stderr"]=(gr.stderr or "")[:400]
    out=Path("${STRESS_D}") if name=="Darren" else Path("${STRESS_J}")
    out.write_text(json.dumps({"rows":rows},indent=2))
PY
  "$CLI" --json grade-stress-pack "$STRESS_D" "$STRESS_J" \
    --min-pass-rate "$STRESS_MIN_PASS_RATE" \
    --max-avg-words "$STRESS_MAX_AVG_WORDS" \
    --question-as-action-allowed \
    --action-shape-mix \
    > "$STRESS_GRADE" || true
fi

echo "[loop] complete"

if command -v python3 >/dev/null 2>&1; then
  SHIFT_D="$ROOT/reports/${STAMP}-loop-register-shift-darren.json"
  SHIFT_J="$ROOT/reports/${STAMP}-loop-register-shift-jasper.json"
  EVAL_D="$ROOT/reports/${STAMP}-loop-evaluate-darren.json"
  EVAL_J="$ROOT/reports/${STAMP}-loop-evaluate-jasper.json"
  GATE_LINE="$(python3 - <<PY
import json
from pathlib import Path

shift_required = int("${SHIFT_GATE_REQUIRED}")
eval_required = int("${EVAL_GATE_REQUIRED}")
stress_required = int("${RUN_STRESS_PACK}")

def load(p):
    return json.loads(Path(p).read_text())

sd=load("${SHIFT_D}")
sj=load("${SHIFT_J}")
ed=load("${EVAL_D}")
ej=load("${EVAL_J}")
sg=load("${STRESS_GRADE}") if stress_required else {"overall":{"gate_passed": True}}

shift_ok = bool((sd.get("gate") or {}).get("passed")) and bool((sj.get("gate") or {}).get("passed"))
eval_ok = True
if eval_required:
    eval_ok = (ed.get("before",{}).get("yes",0) > 0) and (ej.get("before",{}).get("yes",0) > 0)

overall = ((not shift_required) or shift_ok) and eval_ok
stress_ok = bool((sg.get("overall") or {}).get("gate_passed", True))
overall = overall and ((not stress_required) or stress_ok)
status = "PASS" if overall else "FAIL"
print(f"GATE {status} | shift={shift_ok} eval={eval_ok} stress={stress_ok} | stamp=${STAMP}")
PY
)"
  echo "$GATE_LINE"
  if [[ "$RUN_STRESS_PACK" == "1" ]]; then
    ACTION_SHAPE_LINE="$(python3 - <<PY
import json
from pathlib import Path
def rows(path):
    return json.loads(Path(path).read_text()).get("rows", [])
def shape(reply: str):
    low = reply.lower()
    imperative_words = ("do ","start ","stop ","send ","take ","open ","write ","walk ","breathe ","text ","pick ","put ","set ","give ","tell ","name ","list ","focus ","hold ","ship ")
    if any(w in low for w in imperative_words):
        return "imperative"
    if "?" in reply:
        return "question"
    return "other"
all_rows = rows("${STRESS_D}") + rows("${STRESS_J}")
counts = {"imperative": 0, "question": 0, "other": 0}
for r in all_rows:
    rep = r.get("reply")
    if not isinstance(rep, str):
        continue
    counts[shape(rep)] += 1
total = sum(counts.values()) or 1
print(
    "ACTION_SHAPE"
    + f" | imperative={counts['imperative']} ({counts['imperative']/total:.0%})"
    + f" question={counts['question']} ({counts['question']/total:.0%})"
    + f" other={counts['other']} ({counts['other']/total:.0%})"
    + f" | stamp=${STAMP}"
)
PY
)"
    echo "$ACTION_SHAPE_LINE"
    POLICY_JSON="$ROOT/reports/${STAMP}-loop-stress-policy.json"
    "$CLI" --json stress-policy-report "$STRESS_GRADE" > "$POLICY_JSON"
    POLICY_LINE="$(python3 - <<PY
import json
from pathlib import Path
payload=json.loads(Path("${POLICY_JSON}").read_text())
rows=payload.get("rows",[])
bits=[]
for r in rows:
    bits.append(
        f"{r.get('character','?')}:other={float(r.get('other_rate',0.0)):.3f},"
        f"no_concrete={float(r.get('no_concrete_rate',0.0)):.3f},"
        f"two_signal_fail={bool(r.get('two_signal_fail',False))}"
    )
print("STRESS_POLICY | " + " | ".join(bits) + f" | stamp=${STAMP}")
PY
)"
    echo "$POLICY_LINE"
  fi
fi

echo "[loop] artifacts:"
echo "  reports/${STAMP}-loop-register-shift-darren.json"
echo "  reports/${STAMP}-loop-register-shift-jasper.json"
echo "  reports/${STAMP}-loop-evaluate-darren.json"
echo "  reports/${STAMP}-loop-evaluate-jasper.json"
echo "  reports/${STAMP}-loop-replay-darren.json"
echo "  reports/${STAMP}-loop-replay-jasper.json"
if [[ "$RUN_STRESS_PACK" == "1" ]]; then
  echo "  reports/${STAMP}-loop-stress-darren.json"
  echo "  reports/${STAMP}-loop-stress-jasper.json"
  echo "  reports/${STAMP}-loop-stress-grade.json"
  echo "  reports/${STAMP}-loop-stress-policy.json"
fi

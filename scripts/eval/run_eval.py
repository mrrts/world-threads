#!/usr/bin/env python3
"""
Tiny A/B harness. Reads scenarios.yaml + ablations.yaml, calls an
OpenAI-compatible endpoint with each (scenario, condition) pair N times,
writes side-by-side markdown to results/.
"""
from __future__ import annotations
import os, sys, json, pathlib, subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
import yaml
from dotenv import load_dotenv
from openai import OpenAI
from prompt_blocks import assemble_prompt, assemble_prompt_e3

ROOT = pathlib.Path(__file__).parent
REPO_ROOT = ROOT.parent.parent
load_dotenv(ROOT / ".env")
CFG = yaml.safe_load((ROOT / "config.yaml").read_text())
SCENARIOS = yaml.safe_load((ROOT / "scenarios.yaml").read_text())
ABLATIONS = yaml.safe_load((ROOT / "ablations.yaml").read_text())


def resolve_api_key() -> str:
    """Env var first, then macOS Keychain via the repo helper.
    Returns empty string if neither is set — caller should error-check."""
    key = os.environ.get("LLM_API_KEY", "").strip()
    if key:
        return key
    helper = REPO_ROOT / "scripts" / "get-claude-code-llm-key.sh"
    if helper.exists():
        try:
            r = subprocess.run(
                ["bash", str(helper)],
                capture_output=True, text=True, timeout=5,
            )
            key = r.stdout.strip()
            if key:
                return key
        except Exception:
            pass
    return ""


API_KEY = resolve_api_key()
if not API_KEY:
    sys.exit(
        "No LLM_API_KEY found. Either:\n"
        "  - export LLM_API_KEY=sk-... for this session, or\n"
        "  - run `bash scripts/setup-claude-code-llm-key.sh` once to store in Keychain."
    )

CLIENT = OpenAI(base_url=os.environ["LLM_BASE_URL"], api_key=API_KEY)
MODEL = os.environ["LLM_MODEL"]
DRY_RUN = os.environ.get("LLM_DRY_RUN", "0") == "1"


def estimate_tokens(s: str) -> int:
    return max(1, len(s) // 4)


def sample(system: str, user: str) -> str:
    if DRY_RUN:
        return f"[DRY_RUN — would call {MODEL} with {estimate_tokens(system)} sys tokens]"
    # gpt-5-class models require max_completion_tokens; older endpoints
    # accept max_tokens. Pass both-compatible form; SDK / API normalizes.
    r = CLIENT.chat.completions.create(
        model=MODEL,
        messages=[{"role": "system", "content": system}, {"role": "user", "content": user}],
        temperature=CFG["temperature"], max_completion_tokens=CFG["max_tokens"],
    )
    return r.choices[0].message.content or "(empty)"


def run_experiment(eid: str) -> None:
    spec = ABLATIONS[eid]
    out_dir = ROOT / "results" / eid
    out_dir.mkdir(parents=True, exist_ok=True)
    n = CFG["samples_per_condition"]
    print(f"\n=== {eid}: {spec['name']} ===")
    for sid in spec["scenarios"]:
        sc = SCENARIOS[sid]
        if eid == "E3":
            sys_on = assemble_prompt_e3(sc["character"], sc["context_line"], on=True)
            sys_off = assemble_prompt_e3(sc["character"], sc["context_line"], on=False)
        else:
            sys_on = assemble_prompt(sc["character"], sc["context_line"], ablate=[])
            sys_off = assemble_prompt(sc["character"], sc["context_line"], ablate=spec["ablate"])
        user = sc["user_turn"].strip()
        print(f"  {sid}: ON={estimate_tokens(sys_on)}t OFF={estimate_tokens(sys_off)}t (saving {estimate_tokens(sys_on)-estimate_tokens(sys_off)}t per turn)")

        with ThreadPoolExecutor(max_workers=4) as ex:
            on_futs = [ex.submit(sample, sys_on, user) for _ in range(n)]
            off_futs = [ex.submit(sample, sys_off, user) for _ in range(n)]
            on_samples = [f.result() for f in on_futs]
            off_samples = [f.result() for f in off_futs]

        md = [f"# {eid} — {sid}", f"_{spec['name']}_", "",
              f"**Scenario notes:** {sc.get('notes', '').strip()}", "",
              f"**User turn:**", "", "> " + user.replace("\n", "\n> "), "",
              "---", "", "## ON (full prompt)", ""]
        for i, s in enumerate(on_samples, 1):
            md += [f"### Sample {i}", "", s, ""]
        md += ["---", "", f"## OFF (ablate: {', '.join(spec['ablate'])})", ""]
        for i, s in enumerate(off_samples, 1):
            md += [f"### Sample {i}", "", s, ""]
        md += ["---", "", "## Score by eye", "",
               "| Dim | ON | OFF | Notes |",
               "|-----|----|----|-------|",
               "| Drives the moment (1-5) |  |  |  |",
               "| Sensory specificity (1-5) |  |  |  |",
               "| Refuses flattery (1-5) |  |  |  |",
               "| Voice consistency (1-5) |  |  |  |",
               "| Would re-read (1-5) |  |  |  |", ""]
        (out_dir / f"{sid}.md").write_text("\n".join(md))
        print(f"    → {out_dir / (sid + '.md')}")


def estimate_cost(eids: list[str]) -> tuple[int, int, float]:
    n = CFG["samples_per_condition"]
    in_tok = out_tok = 0
    for eid in eids:
        spec = ABLATIONS[eid]
        for sid in spec["scenarios"]:
            sc = SCENARIOS[sid]
            sys_on = assemble_prompt(sc["character"], sc["context_line"], ablate=[])
            sys_off = assemble_prompt(sc["character"], sc["context_line"], ablate=spec["ablate"])
            in_tok += (estimate_tokens(sys_on) + estimate_tokens(sys_off)) * n + estimate_tokens(sc["user_turn"]) * 2 * n
            out_tok += CFG["max_tokens"] * n * 2
    cost = in_tok / 1000 * CFG["estimated_cost_per_1k_input"] + out_tok / 1000 * CFG["estimated_cost_per_1k_output"]
    return in_tok, out_tok, cost


def main():
    args = sys.argv[1:] or ["E1"]
    eids = list(ABLATIONS.keys()) if args == ["all"] else args
    bad = [e for e in eids if e not in ABLATIONS]
    if bad:
        sys.exit(f"unknown experiments: {bad}; available: {list(ABLATIONS)}")
    in_tok, out_tok, cost = estimate_cost(eids)
    print(f"Plan: {eids} → ~{in_tok:,} input + ~{out_tok:,} output tokens, ~${cost:.2f}")
    auto_confirm = os.environ.get("LLM_AUTO_CONFIRM", "0") == "1"
    if not DRY_RUN and not auto_confirm and input("Proceed? [y/N] ").strip().lower() != "y":
        sys.exit("aborted")
    for eid in eids:
        run_experiment(eid)
    print("\nDone. Score by reading results/<eid>/<scenario>.md")


if __name__ == "__main__":
    main()

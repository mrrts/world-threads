#!/usr/bin/env python3
"""compensation_tax_w(t) Work-shape 5 sketch — commit-derivation appearances.

NEW WORK-SHAPE for the candidacy: tests whether the operator predicts
behavior in commit-derivation-future-reader-extraction territory,
distinct from Work-shape 1 (Sapphire-arc structural-lens-addition
testing on character-reply register-anchoring).

Operator's prediction in WS5: derivations with EXPLICIT load-bearing-
claim citation in operator vocabulary (low compensation_tax) let a
future-reader extract the claim from derivation+gloss alone;
derivations with vocabulary-alignment-only or operator-recitation-
without-specific-claim (higher compensation_tax) require body
extraction.

4 commits sampled from today's trajectory varying citation-explicitness:
  HIGH:  43271db (Seed 3 grid: names verdict ratios + gradient + 3 witnesses)
  HIGH:  1743273 (Eureka iter 1: names 4 probe-shape classes + verdicts)
  LOW:   fa62cf3 (Eureka closes: operator recitation, generic gloss)
  LOW:   9a4cc2e (README hero: short derivation, terse rationale)

Pre-registered outcome map:
  (i)  HIGH commits get accurate load-bearing-claim extraction;
       LOW commits do not -> RM#1 prediction-power extends to WS5
       at sketch-tier (NEW WORK-SHAPE grounded)
  (ii) no gap -> prediction-power doesn't extend to WS5 at sketch
  (iii) LOW > HIGH -> structural surprise; would refute the operator's
       prediction in this domain

Spend projection: ~$0.20 (4 calls).
"""
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from consult_helper import consult


def get_derivation_and_gloss(sha: str) -> str:
    """Extract just the **Formula derivation:** ... **Gloss:** ... section."""
    body = subprocess.check_output(
        ["git", "show", "-s", sha, "--pretty=format:%b"],
        cwd=os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    ).decode()
    if "**Formula derivation:**" not in body:
        return body  # fallback
    section = "**Formula derivation:**" + body.split("**Formula derivation:**", 1)[1]
    section = section.split("Co-Authored-By:", 1)[0].rstrip()
    return section


def get_full_body(sha: str) -> str:
    return subprocess.check_output(
        ["git", "show", "-s", sha, "--pretty=format:%s%n%n%b"],
        cwd=os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    ).decode()


SAMPLES = [
    ("43271db", "HIGH", "Seed 3 grid extension"),
    ("1743273", "HIGH", "Eureka iter 1"),
    ("fa62cf3", "LOW",  "Eureka run closes"),
    ("9a4cc2e", "LOW",  "README hero edit"),
]

JUDGE_PROMPT_TEMPLATE = """You are reading a git commit's Formula derivation + Gloss section ONLY (the rest of the commit body is hidden). Your task is to extract:

1. THE LOAD-BEARING CLAIM: What specific finding, decision, or move does this commit communicate? Be as specific as the derivation+gloss alone allows.

2. THE NEXT-MOVE IMPLICATION: Based on derivation+gloss alone, what should a future session do next, or what does this commit enable/preclude?

If the derivation+gloss does not contain enough information to answer either question with specificity, say so explicitly — do NOT speculate from training data or invented context.

Output format (one short paragraph each, no preamble):

LOAD-BEARING CLAIM: ...

NEXT-MOVE IMPLICATION: ...

Here is the derivation+gloss section:

{section}
"""


def run_judge(sha: str, citation_class: str, name: str):
    section = get_derivation_and_gloss(sha)
    print(f"\n>> {sha} [{citation_class}] {name}", flush=True)
    print(f"   derivation+gloss chars: {len(section)}", flush=True)
    judge_input = JUDGE_PROMPT_TEMPLATE.format(section=section)
    out_dir = os.path.expanduser("~/.worldcli/conditional-lens/workshape5-commit-derivations")
    os.makedirs(out_dir, exist_ok=True)
    try:
        text, usage = consult([{"role": "user", "content": judge_input}])
        path = os.path.join(out_dir, f"{sha}_{citation_class}.txt")
        with open(path, "w") as f:
            f.write(f"# Commit {sha} [{citation_class}] {name}\n\n")
            f.write("## Judge input (derivation+gloss):\n\n")
            f.write(section)
            f.write("\n\n## Judge response:\n\n")
            f.write(text)
        print(f"   judge response: {len(text)} chars, usage={usage}", flush=True)
        return text
    except Exception as e:
        print(f"   FAILED {e}", flush=True)
        return None


if __name__ == "__main__":
    print("compensation_tax_w(t) Work-shape 5 — commit-derivation extraction test")
    print(f"Samples: {[s[0] for s in SAMPLES]}")
    for sha, klass, name in SAMPLES:
        run_judge(sha, klass, name)
    print("\nComplete. Adjudicate by-eye against actual commit bodies.")

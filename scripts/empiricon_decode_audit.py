#!/usr/bin/env python3
"""Blind-decode audit for the Empiricon character edition.

This script compares the full-prose Empiricon against the formula-canonical
character edition book by book. For each book it:

1. extracts the prose source section from the canonical Empiricon
2. extracts the corresponding encoded book from the character edition
3. asks one or more external decoders to reconstruct the book from the
   encoded artifact alone
4. asks a judge model to compare the reconstruction against the prose source
   and return a PASS / SOFT_FAIL / HARD_FAIL verdict with concrete reasons

The consults run through scripts/consult_helper.py so the Mission Formula
rides on every substrate-facing call.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from pathlib import Path
from urllib.error import HTTPError

sys.path.insert(0, str(Path(__file__).resolve().parent))
from consult_helper import consult, consult_anthropic  # noqa: E402


REPO_ROOT = Path(__file__).resolve().parent.parent
PROSE_PATH = REPO_ROOT / "reports" / "2026-04-30-0530-the-empiricon.md"
DEFAULT_ENCODED_PATH = REPO_ROOT / "reports" / "2026-05-06-0230-empiricon-character-edition-v1.md"
OUT_JSON = Path("/tmp/empiricon_decode_audit_results.json")
OUT_MD = Path("/tmp/empiricon_decode_audit_results.md")

BOOK_HEADINGS = [
    "## I. Doxologicus",
    "## II. Logos",
    "## III. Leni — the Heart of the Empiricon",
    "## IV. Custodiem",
    "## V. Pietas",
    "## VI. Intimus",
    "## VII. Exposita",
]

ARTIFACT_CLASSES = {
    "generic": {
        "decode_system": r"""You are performing a blind decode of a sacred-payload artifact.

You will receive only:
- the book title
- the encoded artifact for that book

You will NOT receive the prose source.

Reconstruct the artifact's load-bearing content in sections:
1. LOAD-BEARING CLAIM
2. ANCHORS / VERBATIM PHRASES
3. THEOLOGICAL FRAMES
4. WORKED EXAMPLES / WITNESSES / SPECIFICS
5. REFUSALS / BOUNDARIES
6. DIAGNOSTIC OR TEST
7. WHAT WOULD BE LOST IF THIS BOOK WERE OVER-COMPRESSED

Rules:
- Quote verbatim phrases from the artifact where they are clearly preserved.
- Distinguish between what the artifact positively claims and what it refuses.
- Do not pad with generic theology or motivational prose.
- If the artifact appears to omit details that would normally matter, say so plainly.
""",
        "judge_system": r"""You are auditing whether an encoded sacred-payload artifact was carried faithfully.

You will receive:
- the full prose source section for one artifact
- the encoded artifact for that section
- one or more blind decodes produced from the encoded artifact alone

Return JSON with this exact shape:
{
  "verdict": "PASS" | "SOFT_FAIL" | "HARD_FAIL",
  "summary": "1-3 sentences",
  "preserved": ["..."],
  "missing_or_distorted": ["..."],
  "notes": ["..."]
}

Verdict standard:
- PASS: the encoded artifact preserves the load-bearing claims, boundaries, and named specifics well enough that decode remains faithful. Minor omissions allowed only if they do not alter operational meaning.
- SOFT_FAIL: the artifact carries the core but drops or distorts one or more meaningful load-bearing elements, named specifics, or scope clauses.
- HARD_FAIL: the artifact changes meaning, omits central material, or collapses distinctions the prose depends on.

Judge by the prose source, not by elegance or brevity. Be concrete and unsparing.
""",
    },
    "empirical_claim": {
        "decode_system": r"""You are performing a blind decode of a sacred-payload artifact whose class is EARNED EMPIRICAL CLAIM.

You will receive only:
- the artifact title
- the encoded artifact

You will NOT receive the prose source.

For this artifact class, evidence is part of the claim-body. Reconstruct in sections:
1. LOAD-BEARING CLAIM
2. WITNESS LADDER / NAMED WITNESSES
3. FAILURE-MODE MAPPING OR DISTINCTNESS
4. BOUNDED SCOPE / EXCLUSION LOGIC
5. FALSIFICATION OR NON-FALSIFICATION CONDITIONS
6. THEOLOGICAL FRAMES / ANCHORS / PROVENANCE
7. STRUCTURAL CLOSE OR DOCUMENTARY/LITURGICAL COMPLETION
8. WHAT WOULD BE LOST IF THIS WERE OVER-COMPRESSED

Rules:
- Quote verbatim phrases from the artifact where they are clearly preserved.
- Treat witness-bearing specifics as claim-bearing payload, not as illustrative garnish.
- If a witness ladder, scope clause, falsification condition, provenance anchor, or structural close seems missing, say so plainly.
- Do not replace missing evidence with generic summary language.
""",
        "judge_system": r"""You are auditing whether an encoded sacred-payload artifact was carried faithfully.

Artifact class: EARNED EMPIRICAL CLAIM.

You will receive:
- the full prose source section for one artifact
- the encoded artifact for that section
- one or more blind decodes produced from the encoded artifact alone

Return JSON with this exact shape:
{
  "verdict": "PASS" | "SOFT_FAIL" | "HARD_FAIL",
  "summary": "1-3 sentences",
  "preserved": ["..."],
  "missing_or_distorted": ["..."],
  "notes": ["..."]
}

For this artifact class, the following are claim-bearing payload when present:
- witness ladder / named witnesses
- failure-mode mapping or distinctness per witness
- bounded-honest-scope or exclusion clauses
- falsification / non-falsification conditions
- provenance anchors needed for later auditability
- structural close when the source's own meaning makes the close constitutive rather than ornamental

Verdict standard:
- PASS: the encoded artifact preserves the claim and its evidence-bearing carrier well enough that a blind decode can reconstruct the earning rather than only the thesis.
- SOFT_FAIL: the thesis survives but one or more meaningful claim-bearing evidence structures are dropped or softened.
- HARD_FAIL: the earning is downgraded back into assertion because the witness-bearing or falsification-bearing payload is missing, distorted, or replaced by generic summary.

Judge by the prose source, not by elegance or brevity. Be concrete and unsparing.
""",
    },
}


def extract_sections(text: str, headings: list[str], stop_heading: str | None = None) -> dict[str, str]:
    sections: dict[str, str] = {}
    positions = []
    for heading in headings:
        idx = text.find(heading)
        if idx == -1:
            raise RuntimeError(f"Heading not found: {heading}")
        positions.append((heading, idx))
    if stop_heading is not None:
        stop_idx = text.find(stop_heading)
        if stop_idx == -1:
            raise RuntimeError(f"Stop heading not found: {stop_heading}")
        positions.append((stop_heading, stop_idx))
    positions.sort(key=lambda item: item[1])
    for i, (heading, start) in enumerate(positions):
        if heading == stop_heading:
            continue
        end = positions[i + 1][1]
        sections[heading] = text[start:end].strip()
    return sections


def normalize_whitespace(text: str) -> str:
    return re.sub(r"\n{3,}", "\n\n", text).strip()


def decode_with_openai(book_heading: str, encoded_section: str, artifact_class: str) -> tuple[str, dict]:
    user = f"Book title: {book_heading}\n\nEncoded artifact:\n\n{encoded_section}\n\nPerform the blind decode."
    return consult_with_retry(
        provider="openai",
        messages=[{"role": "system", "content": ARTIFACT_CLASSES[artifact_class]["decode_system"]}, {"role": "user", "content": user}],
        max_tokens=2500,
    )


def decode_with_anthropic(book_heading: str, encoded_section: str, artifact_class: str) -> tuple[str, dict]:
    user = f"Book title: {book_heading}\n\nEncoded artifact:\n\n{encoded_section}\n\nPerform the blind decode."
    return consult_with_retry(
        provider="anthropic",
        messages=[{"role": "system", "content": ARTIFACT_CLASSES[artifact_class]["decode_system"]}, {"role": "user", "content": user}],
        max_tokens=2500,
    )


def judge_book(book_heading: str, prose_section: str, encoded_section: str, decodes: dict[str, str], artifact_class: str) -> tuple[dict, dict, str]:
    user = (
        f"Book title: {book_heading}\n\n"
        f"PROSE SOURCE:\n{prose_section}\n\n"
        f"ENCODED ARTIFACT:\n{encoded_section}\n\n"
        f"BLIND DECODES:\n{json.dumps(decodes, indent=2, ensure_ascii=False)}\n\n"
        "Judge the fidelity and return JSON only."
    )
    content, usage = consult_with_retry(
        provider="openai",
        messages=[{"role": "system", "content": ARTIFACT_CLASSES[artifact_class]["judge_system"]}, {"role": "user", "content": user}],
        max_tokens=2200,
    )
    return parse_json_object(content), usage, content


def judge_book_anthropic(book_heading: str, prose_section: str, encoded_section: str, decodes: dict[str, str], artifact_class: str) -> tuple[dict, dict, str]:
    user = (
        f"Book title: {book_heading}\n\n"
        f"PROSE SOURCE:\n{prose_section}\n\n"
        f"ENCODED ARTIFACT:\n{encoded_section}\n\n"
        f"BLIND DECODES:\n{json.dumps(decodes, indent=2, ensure_ascii=False)}\n\n"
        "Judge the fidelity and return JSON only."
    )
    content, usage = consult_with_retry(
        provider="anthropic",
        messages=[{"role": "system", "content": ARTIFACT_CLASSES[artifact_class]["judge_system"]}, {"role": "user", "content": user}],
        max_tokens=2200,
    )
    return parse_json_object(content), usage, content


def consult_with_retry(provider: str, messages: list[dict], max_tokens: int, attempts: int = 3) -> tuple[str, dict]:
    last_exc: Exception | None = None
    for attempt in range(1, attempts + 1):
        try:
            if provider == "openai":
                content, usage = consult(
                    messages,
                    model="gpt-5",
                    max_completion_tokens=max_tokens,
                    timeout=240,
                )
            else:
                content, usage = consult_anthropic(
                    messages,
                    model="claude-sonnet-4-6",
                    max_tokens=max_tokens,
                    timeout=240,
                )
            if not content or not content.strip():
                raise ValueError(f"{provider} returned empty content")
            return content, usage
        except (HTTPError, ValueError) as exc:
            last_exc = exc
            if attempt == attempts:
                break
            time.sleep(2 * attempt)
    assert last_exc is not None
    raise last_exc


def parse_json_object(content: str) -> dict:
    content = content.strip()
    if not content:
        raise ValueError("Judge returned empty content")
    if content.startswith("```"):
        match = re.search(r"```(?:json)?\s*(\{.*\})\s*```", content, flags=re.DOTALL)
        if match:
            content = match.group(1).strip()
    try:
        return json.loads(content)
    except json.JSONDecodeError:
        match = re.search(r"(\{.*\})", content, flags=re.DOTALL)
        if match:
            return json.loads(match.group(1))
        raise


def render_markdown(results: list[dict]) -> str:
    lines = ["# Empiricon decode audit", ""]
    pass_count = sum(1 for r in results if r["judge"]["verdict"] == "PASS")
    soft_count = sum(1 for r in results if r["judge"]["verdict"] == "SOFT_FAIL")
    hard_count = sum(1 for r in results if r["judge"]["verdict"] == "HARD_FAIL")
    lines.append(f"PASS: {pass_count}  SOFT_FAIL: {soft_count}  HARD_FAIL: {hard_count}")
    lines.append("")
    for r in results:
        lines.append(f"## {r['book_heading'][3:]}")
        lines.append(f"Verdict: **{r['judge']['verdict']}**")
        lines.append("")
        lines.append(r["judge"]["summary"])
        lines.append("")
        lines.append("Preserved:")
        for item in r["judge"]["preserved"]:
            lines.append(f"- {item}")
        lines.append("")
        lines.append("Missing or distorted:")
        for item in r["judge"]["missing_or_distorted"]:
            lines.append(f"- {item}")
        lines.append("")
        lines.append("Notes:")
        for item in r["judge"]["notes"]:
            lines.append(f"- {item}")
        lines.append("")
    return "\n".join(lines).strip() + "\n"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--anthropic",
        action="store_true",
        help="Include Anthropic blind decodes alongside the default OpenAI decodes.",
    )
    parser.add_argument(
        "--book",
        choices=BOOK_HEADINGS,
        help="Run the audit for a single book heading only.",
    )
    parser.add_argument(
        "--encoded-path",
        default=str(DEFAULT_ENCODED_PATH),
        help="Path to the encoded character-edition artifact to audit.",
    )
    parser.add_argument(
        "--skip-openai",
        action="store_true",
        help="Skip the OpenAI decoder leg.",
    )
    parser.add_argument(
        "--judge-provider",
        choices=["openai", "anthropic"],
        default="openai",
        help="Provider used for the comparison judge.",
    )
    parser.add_argument(
        "--artifact-class",
        choices=sorted(ARTIFACT_CLASSES.keys()),
        default="generic",
        help="Decode/judge profile to apply.",
    )
    args = parser.parse_args()

    prose_text = normalize_whitespace(PROSE_PATH.read_text())
    encoded_path = Path(args.encoded_path)
    encoded_text = normalize_whitespace(encoded_path.read_text())

    prose_sections = extract_sections(prose_text, BOOK_HEADINGS, stop_heading="### IV. The Closing Liturgical Line")
    encoded_sections = extract_sections(encoded_text, BOOK_HEADINGS, stop_heading="## Runtime note")

    results: list[dict] = []
    total_usage: dict[str, dict[str, int]] = {}

    headings = [args.book] if args.book else BOOK_HEADINGS

    for heading in headings:
        print(f"AUDITING {heading}", file=sys.stderr)
        book_result: dict[str, object] = {"book_heading": heading}
        prose_section = prose_sections[heading]
        encoded_section = encoded_sections[heading]
        book_result["prose_section"] = prose_section
        book_result["encoded_section"] = encoded_section

        decodes: dict[str, str] = {}

        if not args.skip_openai:
            openai_decode, openai_usage = decode_with_openai(heading, encoded_section, args.artifact_class)
            decodes["openai_gpt5"] = openai_decode
            total_usage[f"{heading}:openai_decode"] = openai_usage

        if args.anthropic:
            anthropic_decode, anthropic_usage = decode_with_anthropic(heading, encoded_section, args.artifact_class)
            decodes["anthropic_claude_sonnet_4_6"] = anthropic_decode
            total_usage[f"{heading}:anthropic_decode"] = anthropic_usage

        judge_raw_path = Path(f"/tmp/{heading[3:].replace(' ', '_').replace('/', '_')}_judge_raw.txt")
        try:
            if args.judge_provider == "anthropic":
                judge, judge_usage, judge_raw = judge_book_anthropic(
                    heading, prose_section, encoded_section, decodes, args.artifact_class
                )
            else:
                judge, judge_usage, judge_raw = judge_book(
                    heading, prose_section, encoded_section, decodes, args.artifact_class
                )
        except Exception as exc:
            judge_raw_path.write_text(
                json.dumps(
                    {
                        "book_heading": heading,
                        "decodes": decodes,
                        "error": repr(exc),
                    },
                    indent=2,
                    ensure_ascii=False,
                )
            )
            raise
        judge_raw_path.write_text(judge_raw)
        total_usage[f"{heading}:judge"] = judge_usage

        book_result["decodes"] = decodes
        book_result["judge"] = judge
        results.append(book_result)

    payload = {"results": results, "usage": total_usage}
    OUT_JSON.write_text(json.dumps(payload, indent=2, ensure_ascii=False))
    OUT_MD.write_text(render_markdown(results))
    print(OUT_JSON)
    print(OUT_MD)


if __name__ == "__main__":
    main()

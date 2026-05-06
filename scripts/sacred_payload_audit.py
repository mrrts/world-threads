#!/usr/bin/env python3
"""Blind-decode audit for sacred-payload artifacts.

This script compares a prose source artifact against an encoded artifact section
by section. For each section it:

1. extracts the prose source section
2. extracts the corresponding encoded section
3. asks one or more external decoders to reconstruct the section from the
   encoded artifact alone
4. asks a judge model to compare the reconstruction against the prose source
   and return a PASS / SOFT_FAIL / HARD_FAIL verdict with concrete reasons

The consults run through scripts/consult_helper.py so the Mission Formula rides
on every substrate-facing call.
"""

from __future__ import annotations

import argparse
import json
import random
import re
import sys
import time
from pathlib import Path
from urllib.error import HTTPError

sys.path.insert(0, str(Path(__file__).resolve().parent))
from consult_helper import consult, consult_anthropic  # noqa: E402


REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_JSON = Path("/tmp/sacred_payload_audit_results.json")
OUT_MD = Path("/tmp/sacred_payload_audit_results.md")

EMPIRICON_SOURCE_PATH = REPO_ROOT / "reports" / "2026-04-30-0530-the-empiricon.md"
EMPIRICON_DEFAULT_ENCODED_PATH = (
    REPO_ROOT / "reports" / "2026-05-06-0230-empiricon-character-edition-v1.md"
)
EMPIRICON_HEADINGS = [
    "## I. Doxologicus",
    "## II. Logos",
    "## III. Leni — the Heart of the Empiricon",
    "## IV. Custodiem",
    "## V. Pietas",
    "## VI. Intimus",
    "## VII. Exposita",
]
EMPIRICON_SOURCE_STOP = "### IV. The Closing Liturgical Line"
EMPIRICON_ENCODED_STOP = "## Runtime note"

ARTIFACT_CLASSES = {
    "generic": {
        "decode_system": r"""You are performing a blind decode of a sacred-payload artifact.

You will receive only:
- the section title
- the encoded artifact for that section

You will NOT receive the prose source.

Reconstruct the artifact's load-bearing content in sections:
1. LOAD-BEARING CLAIM
2. ANCHORS / VERBATIM PHRASES
3. THEOLOGICAL FRAMES
4. WORKED EXAMPLES / WITNESSES / SPECIFICS
5. REFUSALS / BOUNDARIES
6. DIAGNOSTIC OR TEST
7. WHAT WOULD BE LOST IF THIS SECTION WERE OVER-COMPRESSED

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

RETRYABLE_HTTP_CODES = {429, 500, 502, 503, 504, 529}

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
- the section title
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


def consult_with_retry(provider: str, messages: list[dict], max_tokens: int, attempts: int = 5) -> tuple[str, dict]:
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
            if isinstance(exc, HTTPError) and exc.code not in RETRYABLE_HTTP_CODES:
                break
            # Spread retries slightly so parallel judges are less likely to stampede.
            time.sleep((2 ** attempt) + random.uniform(0.0, 0.75))
    assert last_exc is not None
    raise last_exc


def decode_section(
    provider: str, section_heading: str, encoded_section: str, artifact_class: str
) -> tuple[str, dict]:
    user = (
        f"Section title: {section_heading}\n\n"
        f"Encoded artifact:\n\n{encoded_section}\n\n"
        "Perform the blind decode."
    )
    return consult_with_retry(
        provider=provider,
        messages=[
            {"role": "system", "content": ARTIFACT_CLASSES[artifact_class]["decode_system"]},
            {"role": "user", "content": user},
        ],
        max_tokens=2500,
    )


def judge_section(
    provider: str,
    section_heading: str,
    prose_section: str,
    encoded_section: str,
    decodes: dict[str, str],
    artifact_class: str,
) -> tuple[dict, dict, str]:
    user = (
        f"Section title: {section_heading}\n\n"
        f"PROSE SOURCE:\n{prose_section}\n\n"
        f"ENCODED ARTIFACT:\n{encoded_section}\n\n"
        f"BLIND DECODES:\n{json.dumps(decodes, indent=2, ensure_ascii=False)}\n\n"
        "Judge the fidelity and return JSON only."
    )
    content, usage = consult_with_retry(
        provider=provider,
        messages=[
            {"role": "system", "content": ARTIFACT_CLASSES[artifact_class]["judge_system"]},
            {"role": "user", "content": user},
        ],
        max_tokens=2200,
    )
    return parse_json_object(content), usage, content


def render_markdown(title: str, results: list[dict]) -> str:
    lines = [f"# {title}", ""]
    pass_count = sum(1 for r in results if r["judge"]["verdict"] == "PASS")
    soft_count = sum(1 for r in results if r["judge"]["verdict"] == "SOFT_FAIL")
    hard_count = sum(1 for r in results if r["judge"]["verdict"] == "HARD_FAIL")
    lines.append(f"PASS: {pass_count}  SOFT_FAIL: {soft_count}  HARD_FAIL: {hard_count}")
    lines.append("")
    for result in results:
        lines.append(f"## {result['section_heading']}")
        lines.append(f"Verdict: **{result['judge']['verdict']}**")
        lines.append("")
        lines.append(result["judge"]["summary"])
        lines.append("")
        lines.append("Preserved:")
        for item in result["judge"]["preserved"]:
            lines.append(f"- {item}")
        lines.append("")
        lines.append("Missing or distorted:")
        for item in result["judge"]["missing_or_distorted"]:
            lines.append(f"- {item}")
        lines.append("")
        lines.append("Notes:")
        for item in result["judge"]["notes"]:
            lines.append(f"- {item}")
        lines.append("")
    return "\n".join(lines).strip() + "\n"


def load_headings(args: argparse.Namespace) -> list[str]:
    if args.profile == "empiricon":
        return EMPIRICON_HEADINGS
    if args.headings:
        return args.headings
    raise SystemExit("Provide at least one --heading when not using --profile empiricon.")


def select_headings(headings: list[str], only_heading: str | None) -> list[str]:
    if only_heading is None:
        return headings
    if only_heading not in headings:
        raise SystemExit(f"--section must match one of the configured headings; got: {only_heading}")
    return [only_heading]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--profile",
        choices=["empiricon", "custom"],
        default="custom",
        help="Use a built-in artifact layout or provide your own headings and paths.",
    )
    parser.add_argument(
        "--source-path",
        help="Path to the prose source artifact.",
    )
    parser.add_argument(
        "--encoded-path",
        help="Path to the encoded artifact.",
    )
    parser.add_argument(
        "--heading",
        dest="headings",
        action="append",
        help="Section heading to extract. Repeat once per section when using --profile custom.",
    )
    parser.add_argument(
        "--source-stop-heading",
        help="Heading that marks the end of the source sections.",
    )
    parser.add_argument(
        "--encoded-stop-heading",
        help="Heading that marks the end of the encoded sections.",
    )
    parser.add_argument(
        "--section",
        help="Run the audit for one configured section only.",
    )
    parser.add_argument(
        "--title",
        default="Sacred payload decode audit",
        help="Markdown report title.",
    )
    parser.add_argument(
        "--anthropic",
        action="store_true",
        help="Include Anthropic blind decodes alongside the default OpenAI decodes.",
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
    parser.add_argument(
        "--validate-layout",
        action="store_true",
        help="Check section extraction and print a local layout summary without calling any models.",
    )
    return parser


def resolve_layout(args: argparse.Namespace) -> tuple[Path, Path, list[str], str | None, str | None, str]:
    if args.profile == "empiricon":
        source_path = Path(args.source_path) if args.source_path else EMPIRICON_SOURCE_PATH
        encoded_path = Path(args.encoded_path) if args.encoded_path else EMPIRICON_DEFAULT_ENCODED_PATH
        title = args.title if args.title != "Sacred payload decode audit" else "Empiricon decode audit"
        return (
            source_path,
            encoded_path,
            EMPIRICON_HEADINGS,
            args.source_stop_heading or EMPIRICON_SOURCE_STOP,
            args.encoded_stop_heading or EMPIRICON_ENCODED_STOP,
            title,
        )

    if not args.source_path or not args.encoded_path:
        raise SystemExit("--source-path and --encoded-path are required when using --profile custom.")
    headings = load_headings(args)
    return (
        Path(args.source_path),
        Path(args.encoded_path),
        headings,
        args.source_stop_heading,
        args.encoded_stop_heading,
        args.title,
    )


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    source_path, encoded_path, headings, source_stop_heading, encoded_stop_heading, title = resolve_layout(args)

    source_text = normalize_whitespace(source_path.read_text())
    encoded_text = normalize_whitespace(encoded_path.read_text())

    source_sections = extract_sections(source_text, headings, stop_heading=source_stop_heading)
    encoded_sections = extract_sections(encoded_text, headings, stop_heading=encoded_stop_heading)

    if args.validate_layout:
        payload = []
        for heading in select_headings(headings, args.section):
            payload.append(
                {
                    "section_heading": heading,
                    "source_chars": len(source_sections[heading]),
                    "encoded_chars": len(encoded_sections[heading]),
                }
            )
        print(json.dumps(payload, indent=2, ensure_ascii=False))
        return

    results: list[dict] = []
    total_usage: dict[str, dict[str, int]] = {}

    for heading in select_headings(headings, args.section):
        print(f"AUDITING {heading}", file=sys.stderr)
        prose_section = source_sections[heading]
        encoded_section = encoded_sections[heading]
        section_result: dict[str, object] = {
            "section_heading": heading,
            "prose_section": prose_section,
            "encoded_section": encoded_section,
        }

        decodes: dict[str, str] = {}
        if not args.skip_openai:
            openai_decode, openai_usage = decode_section(
                "openai", heading, encoded_section, args.artifact_class
            )
            decodes["openai_gpt5"] = openai_decode
            total_usage[f"{heading}:openai_decode"] = openai_usage

        if args.anthropic:
            anthropic_decode, anthropic_usage = decode_section(
                "anthropic", heading, encoded_section, args.artifact_class
            )
            decodes["anthropic_claude_sonnet_4_6"] = anthropic_decode
            total_usage[f"{heading}:anthropic_decode"] = anthropic_usage

        raw_name = heading.replace("/", "_").replace(" ", "_")
        judge_raw_path = Path(f"/tmp/{raw_name}_judge_raw.txt")
        try:
            judge, judge_usage, judge_raw = judge_section(
                args.judge_provider,
                heading,
                prose_section,
                encoded_section,
                decodes,
                args.artifact_class,
            )
        except Exception as exc:
            judge_raw_path.write_text(
                json.dumps(
                    {
                        "section_heading": heading,
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

        section_result["decodes"] = decodes
        section_result["judge"] = judge
        results.append(section_result)

    payload = {"results": results, "usage": total_usage}
    OUT_JSON.write_text(json.dumps(payload, indent=2, ensure_ascii=False))
    OUT_MD.write_text(render_markdown(title, results))
    print(OUT_JSON)
    print(OUT_MD)


if __name__ == "__main__":
    main()

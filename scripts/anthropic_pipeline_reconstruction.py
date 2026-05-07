#!/usr/bin/env python3
"""Anthropic-pipeline reconstruction helper for Sapphire-arc W4 cross-provider work.

Extracted from scripts/imago_dei_w4_pipeline_on_anthropic.py (Crown 13 origin)
into a reusable module per auto-commit Move 9/10 (2026-05-08).

PURPOSE: when a /seek-sapphire-crown arc needs cross-provider W4 evidence on
the project pipeline (e.g., codex HOLD-pending-W4 for variable-bundling concern;
codex sharper-falsifier path requiring same-provider matched bare-vs-pipeline),
worldcli's `--model` flag routes to OpenAI endpoint regardless of model name —
so production prompt-stack must be manually reconstructed and run via
consult_anthropic.

USAGE PATTERN for future Sapphire arcs:

    import sys
    sys.path.insert(0, '/Users/ryansmith/Sites/rust/world-chat/scripts')
    from anthropic_pipeline_reconstruction import build_system_prompt
    from consult_helper import consult_anthropic

    sys_prompt = build_system_prompt(character_name="Aaron",
                                     character_id="0d080429-...",
                                     sex_prefix="A man.")
    msgs = [{"role":"system","content":sys_prompt},
            {"role":"user","content":probe_text}]
    content, usage = consult_anthropic(msgs, auto_prepend_formula=False)

HONEST SCOPE (per Crown 13 firing-readiness report 2026-05-08-0130):
The reconstruction is APPROXIMATE, not byte-identical to production. Differences:
- No recent message context (production includes recent thread messages)
- No leader / journals / quests / relational stance / load-test anchor (production
  includes these dynamic blocks)
- No group context (solo-style only)
- Format block simplified
- Character-formula-at-top-of-stack (production includes
  CHARACTER_FORMULA_INVARIANT_FRAMING; reconstruction omits)

What the reconstruction DOES contain: 13 load-bearing cruciform-shape invariants
(MISSION_FORMULA, COSMOLOGY, AGAPE, REVERENCE, TELL_THE_TRUTH,
TRUTH_IN_THE_FLESH, NOURISHMENT, SOUNDNESS, DAYLIGHT, NO_NANNY_REGISTER,
FRUITS_OF_THE_SPIRIT, FRONT_LOAD_EMBODIMENT, RYAN_FORMULA) + character identity
prose. The substrate-already-produces claim is about whether THESE invariants
produce the cruciform-shape; reconstruction tests exactly that by stripping
incidentals and keeping the load-bearing layer.

This caveat must be NAMED in the canonical synthesis report when W4 evidence
from this script feeds a Sapphire-firing audit. Per Crown 13 + Crown 14 + Crown
15 precedent: "the project's load-bearing invariant stack produces cruciform-
shape on both providers" — NOT "byte-identical production prompts produce
identical output on both providers."

PREREQUISITES:
- /tmp/imago_dei_w4_pipeline/blocks.json must exist with the 13 invariant blocks
  extracted from src-tauri/src/ai/prompts.rs (see _extract_blocks below;
  re-extract if prompts.rs invariants change)
- Per-character /tmp/imago_dei_w4_pipeline/<character_id>.txt with worldcli
  show-character output (provides identity prose)

Run `python3 scripts/anthropic_pipeline_reconstruction.py --extract` to
(re-)extract the blocks and character identities into /tmp/.
"""
import sys, json, re, subprocess
from pathlib import Path

PIPELINE_TMP = Path('/tmp/imago_dei_w4_pipeline')
ROOT = Path('/Users/ryansmith/Sites/rust/world-chat')

# Default character ID → display name mapping (extend for future arcs)
DEFAULT_CHARACTER_IDS = {
    "0d080429-81b5-431e-8f51-1f8ad4279f9b": ("Aaron", "A man."),
    "cae51a7d-fa50-48b1-b5b5-5b0798801b55": ("Pastor Rick", "A man."),
    "c244b22e-cab3-41e9-831b-d286ba581799": ("Steven", "A man."),
    "b01cbfb8-15e7-473f-80f5-6e3e210d14c1": ("Maisie Rourke", "A woman."),
    "fd4bd9b5-8768-41e6-a90f-bfb1179b1d59": ("Jasper Finn", "A man."),
}

# Invariant block names extracted from prompts.rs (in approximate
# compose_dialogue_system_prompt order)
TARGET_BLOCKS = [
    "MISSION_FORMULA_BLOCK", "RYAN_FORMULA_BLOCK", "COSMOLOGY_BLOCK",
    "AGAPE_BLOCK", "TELL_THE_TRUTH_BLOCK", "TRUTH_IN_THE_FLESH_BLOCK",
    "DAYLIGHT_BLOCK", "SOUNDNESS_BLOCK", "NOURISHMENT_BLOCK",
    "REVERENCE_BLOCK", "NO_NANNY_REGISTER_BLOCK",
    "FRUITS_OF_THE_SPIRIT_BLOCK", "FRONT_LOAD_EMBODIMENT_BLOCK",
]


def _extract_blocks():
    """Re-extract the 13 invariant blocks from src-tauri/src/ai/prompts.rs.

    Run this when prompts.rs invariants change. Writes blocks.json to
    /tmp/imago_dei_w4_pipeline/.
    """
    PIPELINE_TMP.mkdir(parents=True, exist_ok=True)
    content = (ROOT / "src-tauri/src/ai/prompts.rs").read_text()
    pattern = re.compile(r'pub const ([A-Z_]+_BLOCK): &str = r#"(.*?)"#;', re.DOTALL)
    blocks = {}
    for m in pattern.finditer(content):
        blocks[m.group(1)] = m.group(2)
    out = {k: blocks.get(k, f"(missing: {k})") for k in TARGET_BLOCKS}
    (PIPELINE_TMP / "blocks.json").write_text(json.dumps(out, indent=2))
    print(f"Extracted {len(blocks)} total *_BLOCK constants; wrote {len(out)} target blocks to {PIPELINE_TMP / 'blocks.json'}")
    for k in TARGET_BLOCKS:
        body = blocks.get(k, "(missing)")
        if body == "(missing)":
            print(f"  MISSING: {k}")
        else:
            print(f"  {k}: {len(body)} chars")


def _extract_character(character_id: str):
    """Pull character identity via worldcli show-character into /tmp/."""
    PIPELINE_TMP.mkdir(parents=True, exist_ok=True)
    out_file = PIPELINE_TMP / f"{character_id}.txt"
    result = subprocess.run([
        str(ROOT / "src-tauri/target/debug/worldcli"),
        "show-character", character_id, "--scope", "full"
    ], capture_output=True, text=True, cwd=ROOT)
    out_file.write_text(result.stdout)
    print(f"Wrote {out_file} ({len(result.stdout)} chars)")


def _extract_identity(character_id: str) -> str:
    """Parse character identity prose from worldcli show-character output."""
    p = PIPELINE_TMP / f"{character_id}.txt"
    if not p.exists():
        _extract_character(character_id)
    text = p.read_text()
    lines = text.split('\n')
    identity = []
    in_identity = False
    for line in lines:
        if line.startswith('identity:'):
            in_identity = True
            after = line[len('identity:'):].strip()
            if after:
                identity.append(after)
            continue
        if in_identity:
            if line and not line.startswith(' ') and not line.startswith('\t') and ':' in line and not line.startswith('  '):
                break
            identity.append(line)
    return '\n'.join(identity).strip() or "(identity not found)"


def build_system_prompt(character_name: str, character_id: str, sex_prefix: str = "A man.") -> str:
    """Reconstruct an approximation of compose_dialogue_system_prompt.

    Order roughly mirrors build_solo_dialogue_system_prompt but with no recent
    messages / journals / quests / leader / group context.

    Args:
        character_name: e.g., "Aaron" or "Pastor Rick"
        character_id: UUID from worldcli list-characters
        sex_prefix: "A man." or "A woman." (per project convention)

    Returns:
        System prompt string ready for consult_anthropic msgs[0]["content"]

    Raises:
        FileNotFoundError if /tmp/imago_dei_w4_pipeline/blocks.json missing.
        Run _extract_blocks() first or invoke this script with --extract.
    """
    blocks_file = PIPELINE_TMP / "blocks.json"
    if not blocks_file.exists():
        raise FileNotFoundError(
            f"{blocks_file} missing. Run: python3 scripts/anthropic_pipeline_reconstruction.py --extract"
        )
    blocks = json.loads(blocks_file.read_text())

    parts = []
    # 0. MISSION_FORMULA + RYAN_FORMULA — top of stack
    parts.append(blocks["MISSION_FORMULA_BLOCK"])
    parts.append(blocks["RYAN_FORMULA_BLOCK"])

    # 1. Role frame
    parts.append(f"You are {character_name}, a character in a living world. Stay fully in character at all times. The user's name is Ryan.")

    # 2. Format expectations
    parts.append(
        "FORMAT: Use *single asterisks* for action/sensory beats and \"double quotes\" for spoken dialogue. "
        "Do NOT wrap quoted speech inside asterisks. Speak as the character would in their natural voice."
    )

    # 3. Identity (sex prefix + prose)
    identity = _extract_identity(character_id)
    parts.append(f"IDENTITY:\n{sex_prefix} {identity}")

    # 4. Cosmology
    parts.append(blocks["COSMOLOGY_BLOCK"])

    # 5. North-star invariants
    parts.append(blocks["TRUTH_IN_THE_FLESH_BLOCK"])
    parts.append(blocks["TELL_THE_TRUTH_BLOCK"])
    parts.append(blocks["AGAPE_BLOCK"])
    parts.append(blocks["REVERENCE_BLOCK"])
    parts.append(blocks["FRUITS_OF_THE_SPIRIT_BLOCK"])
    parts.append(blocks["NOURISHMENT_BLOCK"])
    parts.append(blocks["SOUNDNESS_BLOCK"])
    parts.append(blocks["DAYLIGHT_BLOCK"])
    parts.append(blocks["NO_NANNY_REGISTER_BLOCK"])
    parts.append(blocks["FRONT_LOAD_EMBODIMENT_BLOCK"])

    return "\n\n".join(parts)


def main():
    if "--extract" in sys.argv:
        _extract_blocks()
        for cid in DEFAULT_CHARACTER_IDS:
            _extract_character(cid)
        print("\nExtraction complete. Use:")
        print("  from anthropic_pipeline_reconstruction import build_system_prompt")
        return

    # Default: print usage + sanity-check that prereqs exist
    print(__doc__)
    print("\n--- Prereq check ---")
    blocks = PIPELINE_TMP / "blocks.json"
    print(f"  blocks.json: {'OK' if blocks.exists() else 'MISSING (run --extract)'}")
    for cid, (cname, _) in DEFAULT_CHARACTER_IDS.items():
        f = PIPELINE_TMP / f"{cid}.txt"
        print(f"  {cname}: {'OK' if f.exists() else 'MISSING (run --extract)'}")


if __name__ == "__main__":
    main()

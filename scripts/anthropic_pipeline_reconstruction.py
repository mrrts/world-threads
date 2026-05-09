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
# compose_dialogue_system_prompt order). Two lists:
#   TARGET_BLOCKS — `*_BLOCK` constants (the original 13 doctrinal invariants).
#   TARGET_OTHER_CONSTS — other pub const `&str` constants that ship in the
#   dialogue prompt-stack but don't follow the `*_BLOCK` naming convention.
#   Added 2026-05-09 to capture round-1/2/3 + v3 compression surfaces so the
#   reconstruction can fairly stand in for cross-substrate replication
#   tests of the compression-affordance class. Without these, the
#   reconstruction misses ~95% of round-1/2/3 changes (only RYAN_FORMULA_BLOCK
#   from round-1 is in TARGET_BLOCKS).
TARGET_BLOCKS = [
    "MISSION_FORMULA_BLOCK", "RYAN_FORMULA_BLOCK", "COSMOLOGY_BLOCK",
    "AGAPE_BLOCK", "TELL_THE_TRUTH_BLOCK", "TRUTH_IN_THE_FLESH_BLOCK",
    "DAYLIGHT_BLOCK", "SOUNDNESS_BLOCK", "NOURISHMENT_BLOCK",
    "REVERENCE_BLOCK", "NO_NANNY_REGISTER_BLOCK",
    "FRUITS_OF_THE_SPIRIT_BLOCK", "FRONT_LOAD_EMBODIMENT_BLOCK",
    "BEHAVIOR_AND_KNOWLEDGE_BLOCK",  # added 2026-05-09 with v3 dual-field migration
    "KAVOD_PATTERN_INVARIANT_BLOCK",
]

TARGET_OTHER_CONSTS = [
    "FUNDAMENTAL_SYSTEM_PREAMBLE",
    "STYLE_DIALOGUE_INVARIANT",
    "FORMAT_SECTION",
    "WORLD_FORMULA_INVARIANT_FRAMING",
    "LOCATION_FORMULA_INVARIANT_FRAMING",
    "CHARACTER_FORMULA_INVARIANT_FRAMING",
    "CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING",
]


# Helper-function framing extraction — added 2026-05-09 to close
# Sapphire-18-The-Carrier post-fire commitment 3 (dynamic-block
# inclusion in reconstruction; codex scope-lock #3 pipeline-note
# caveat).
#
# Each entry maps a logical key to (function_name_in_prompts_rs,
# anchor_substring). The anchor must uniquely identify the format!
# string we want to capture inside that function. The extraction
# pulls the first format!() framing string from the named function
# whose body contains the anchor.
HELPER_FRAMINGS = [
    ("DYNAMIC_RECENT_JOURNALS_FRAMING",      "render_recent_journals_block",  "RECENT PAGES FROM YOUR JOURNAL"),
    ("DYNAMIC_RELATIONAL_STANCE_FRAMING",    "render_relational_stance_block","YOUR PRIVATE READ OF THE PERSON"),
    ("DYNAMIC_MEANWHILE_BRIDGE_FRAMING",     "render_meanwhile_bridge_block", "WHAT YOU WERE JUST DOING"),
    ("DYNAMIC_ACTIVE_QUESTS_FRAMING",        "render_active_quests_block",    "ACTIVE QUESTS"),
    ("DYNAMIC_DAILY_READING_FRAMING",        "render_daily_reading_block",    "TODAY'S READING"),
    ("DYNAMIC_WEATHER_FRAMING",              "world_weather_block",           "WEATHER:"),
    ("DYNAMIC_LENGTH_SEAL_AUTO",             "end_of_prompt_length_seal",     "AUTO MODE"),
    ("DYNAMIC_RESPONSE_LENGTH_SHORT",        "response_length_block",         "MODE: SHORT"),
    ("DYNAMIC_RESPONSE_LENGTH_MEDIUM",       "response_length_block",         "MODE: MEDIUM"),
    ("DYNAMIC_RESPONSE_LENGTH_LONG",         "response_length_block",         "MODE: LONG"),
]


def _extract_helper_framing(content: str, fn_name: str, anchor: str) -> str | None:
    """Find the format-string body inside `fn fn_name` that contains
    `anchor`. Returns the string with `{...}` placeholders stripped to
    `[...]` for readability, or None if not found.

    Heuristic: locate `fn fn_name(`, take the source from there until
    the next top-level `}` at column 0, then find the format!() or
    `r#"..."#` / `"..."` literal containing the anchor.
    """
    fn_start_re = re.compile(rf'\bfn {re.escape(fn_name)}\b')
    m = fn_start_re.search(content)
    if not m:
        return None
    # Scan forward to next line beginning with `}` at column 0
    body_start = m.start()
    end_re = re.compile(r'\n}\n', re.MULTILINE)
    em = end_re.search(content, body_start)
    body = content[body_start: em.end() if em else len(content)]
    # Try raw string r#"..."# first (multi-line common in length blocks)
    for raw_m in re.finditer(r'r#"(.*?)"#', body, re.DOTALL):
        candidate = raw_m.group(1)
        if anchor in candidate:
            return candidate
    # Fall back to plain "..." (escape-aware)
    for plain_m in re.finditer(r'"((?:\\.|[^"\\])*)"', body, re.DOTALL):
        candidate = plain_m.group(1)
        if anchor in candidate:
            # unescape simple sequences
            return candidate.encode().decode('unicode_escape')
    return None


def _extract_behavior_knowledge_inline_body(content: str) -> str | None:
    """Extract the non-local-model branch prose from the inline
    `fn behavior_and_knowledge_block` body. At 8d64d81 (pre-round-2),
    the prose lived inside the fn body as a raw-string literal in the
    `else` branch. At HEAD, it has been lifted into the
    BEHAVIOR_AND_KNOWLEDGE_BLOCK constant + registered as Invariant.

    This extractor is the BEHAVIOR_AND_KNOWLEDGE fairness fix
    (Sapphire 18 commitment 4): both arms of the cross-substrate
    bench now have equivalent BEHAVIOR_AND_KNOWLEDGE surface coverage
    regardless of whether the prose lives in a constant (HEAD) or
    inline in the fn body (pre-round-2 baseline).

    Heuristic: find `fn behavior_and_knowledge_block(`, scan for the
    SECOND raw-string `r#"..."#` inside the function body (the first
    is the local_model branch; the second is the non-local-model
    branch with the verbose prose).
    """
    fn_re = re.compile(r'fn behavior_and_knowledge_block\b')
    m = fn_re.search(content)
    if not m:
        return None
    body_start = m.start()
    end_re = re.compile(r'\n}\n', re.MULTILINE)
    em = end_re.search(content, body_start)
    body = content[body_start: em.end() if em else len(content)]
    raw_strings = list(re.finditer(r'r#"(.*?)"#', body, re.DOTALL))
    if len(raw_strings) >= 2:
        return raw_strings[1].group(1)
    return None


def _extract_blocks():
    """Re-extract the invariant + compression-arc constants + dynamic-
    helper framings from src-tauri/src/ai/prompts.rs.

    Run this when prompts.rs invariants or compression-arc surfaces
    change. Writes blocks.json to /tmp/imago_dei_w4_pipeline/.
    """
    PIPELINE_TMP.mkdir(parents=True, exist_ok=True)
    content = (ROOT / "src-tauri/src/ai/prompts.rs").read_text()
    # Match any `pub const NAME: &str = r#"..."#;` (raw string) — covers
    # both *_BLOCK and the round-1/2/3 compression-arc surfaces.
    pattern = re.compile(r'pub const ([A-Z_]+): &str = r#"(.*?)"#;', re.DOTALL)
    consts: dict[str, str] = {}
    for m in pattern.finditer(content):
        consts[m.group(1)] = m.group(2)
    # Match `pub const NAME: &str = "..."` (plain string literal) — covers
    # the *_FORMULA_INVARIANT_FRAMING constants which use plain strings.
    pattern_plain = re.compile(
        r'pub const ([A-Z_]+): &str =\s*"((?:\\.|[^"\\])*)"\s*;',
        re.DOTALL,
    )
    for m in pattern_plain.finditer(content):
        if m.group(1) not in consts:
            consts[m.group(1)] = m.group(2)
    targets = TARGET_BLOCKS + TARGET_OTHER_CONSTS
    out = {k: consts.get(k, f"(missing: {k})") for k in targets}
    # BEHAVIOR_AND_KNOWLEDGE fairness fix — Sapphire 18 commitment 4
    # closure. At HEAD: BEHAVIOR_AND_KNOWLEDGE_BLOCK is a const
    # (already captured above). At pre-round-2 (8d64d81): prose was
    # inline in fn body. If the const is missing, fall back to
    # extracting the fn-body prose so both arms have equivalent
    # surface coverage.
    if out.get("BEHAVIOR_AND_KNOWLEDGE_BLOCK", "").startswith("(missing"):
        inline = _extract_behavior_knowledge_inline_body(content)
        if inline:
            out["BEHAVIOR_AND_KNOWLEDGE_BLOCK"] = inline
    # Dynamic-helper framings — Sapphire 18 commitment 3 closure.
    helper_count = 0
    for key, fn_name, anchor in HELPER_FRAMINGS:
        framing = _extract_helper_framing(content, fn_name, anchor)
        if framing is not None:
            out[key] = framing
            helper_count += 1
        else:
            out[key] = f"(missing helper framing: {fn_name} :: {anchor})"
    (PIPELINE_TMP / "blocks.json").write_text(json.dumps(out, indent=2))
    print(f"Extracted {len(consts)} total pub-const &str surfaces + {helper_count}/{len(HELPER_FRAMINGS)} dynamic-helper framings; wrote {len(out)} entries to {PIPELINE_TMP / 'blocks.json'}")
    for k in targets:
        body = out.get(k, "(missing)")
        if body.startswith("(missing"):
            print(f"  MISSING: {k}")
        else:
            note = " (fn-body fallback)" if k == "BEHAVIOR_AND_KNOWLEDGE_BLOCK" and k not in consts else ""
            print(f"  {k}: {len(body)} chars{note}")
    for key, fn_name, anchor in HELPER_FRAMINGS:
        body = out[key]
        if body.startswith("(missing"):
            print(f"  MISSING helper: {key}")
        else:
            print(f"  {key}: {len(body)} chars  ({fn_name})")


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

    def _present(name: str) -> str | None:
        v = blocks.get(name)
        if not v or v.startswith("(missing:"):
            return None
        return v

    parts = []

    # 0. MISSION_FORMULA + RYAN_FORMULA — top of stack
    parts.append(blocks["MISSION_FORMULA_BLOCK"])
    parts.append(blocks["RYAN_FORMULA_BLOCK"])

    # 0a. STYLE_DIALOGUE_INVARIANT — feature-scoped invariant; sits at
    # head of every dialogue prompt per build_solo_dialogue_system_prompt.
    if v := _present("STYLE_DIALOGUE_INVARIANT"):
        parts.append(v)

    # 0b. FUNDAMENTAL_SYSTEM_PREAMBLE — length-contract + register
    # framing applied to every reply.
    if v := _present("FUNDAMENTAL_SYSTEM_PREAMBLE"):
        parts.append(v)

    # 1. Role frame
    parts.append(f"You are {character_name}, a character in a living world. Stay fully in character at all times. The user's name is Ryan.")

    # 2. CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING (decode lens).
    if v := _present("CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING"):
        parts.append(v)

    # 3. Identity (sex prefix + prose)
    identity = _extract_identity(character_id)
    parts.append(f"IDENTITY:\n{sex_prefix} {identity}")

    # 4. FORMAT_SECTION — dialogue fence-shape teaching block.
    if v := _present("FORMAT_SECTION"):
        parts.append(v)

    # 5. Cosmology + theological-frame invariants
    parts.append(blocks["COSMOLOGY_BLOCK"])
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
    if v := _present("KAVOD_PATTERN_INVARIANT_BLOCK"):
        parts.append(v)

    # 6. BEHAVIOR + KNOWLEDGE LIMITS — Sapphire 18 commitment 4
    # fairness fix landed: at HEAD this surface ships as
    # BEHAVIOR_AND_KNOWLEDGE_BLOCK constant; at 8d64d81 (pre-round-2)
    # it shipped as inline fn-body prose. The extractor in
    # `_extract_blocks` now captures both shapes (const at HEAD;
    # fn-body fallback at 8d64d81 via
    # `_extract_behavior_knowledge_inline_body`). Both arms now have
    # equivalent surface coverage, so we INCLUDE this surface in
    # build_system_prompt rather than excluding it asymmetrically.
    if v := _present("BEHAVIOR_AND_KNOWLEDGE_BLOCK"):
        parts.append(v)

    # 7. DYNAMIC-HELPER FRAMINGS (Sapphire 18 commitment 3 closure):
    # The render_*_block helpers wrap dynamic data in framing prose
    # that round-2 + round-3 trimmed. Including these framings — with
    # stub data — captures their contribution to the ON-vs-OFF toggle
    # diff. The dynamic data itself doesn't differ between toggles
    # (it's runtime-computed from db state), so the toggle-relevant
    # axis is the framing-prose-shape, which IS what we extract here.
    def _present(name: str) -> str | None:
        v = blocks.get(name)
        if not v or v.startswith("(missing"):
            return None
        return v

    # Stub data designed to read as plausible runtime input without
    # claiming byte-fidelity to a specific character/world's actual
    # state. Both arms get identical stub data; only the framing prose
    # differs across toggles.
    stub_journal = "Day 12:\nAaron came by today. Brought up the question of staying put again. I'm still turning it over."
    stub_stance = "He's been here longer than he lets on. The grease on his palm is the giveaway — it's the same grease, week after week, and he hasn't moved on."
    stub_meanwhile = "Walked the long way past the bridge before coming in. The kayak was still tied where I left it."
    stub_quest_lines = "  - The Letter Unwritten — A man you used to know is owed something you haven't said yet.\n     (what has happened with it so far: you've sat down to write three times and gotten up four)"
    stub_daily_reading = "  - 𝓡: 47% · attention to costly truth\n  - 𝓒: 38% · the room as it actually is"
    stub_complication = "the thing you said this morning that landed sideways"

    if v := _present("DYNAMIC_RECENT_JOURNALS_FRAMING"):
        # Body-positional `{}` substitution — replace the trailing
        # placeholder with stub journal.
        rendered = v.replace("{}", stub_journal)
        parts.append(rendered)

    if v := _present("DYNAMIC_RELATIONAL_STANCE_FRAMING"):
        rendered = v.replace("{trimmed}", stub_stance)
        parts.append(rendered)

    if v := _present("DYNAMIC_MEANWHILE_BRIDGE_FRAMING"):
        rendered = v.replace("{summary}", stub_meanwhile)
        parts.append(rendered)

    if v := _present("DYNAMIC_ACTIVE_QUESTS_FRAMING"):
        # Format string is "ACTIVE QUESTS:\n{}\n\n..." — substitute lines
        rendered = v.replace("{}", stub_quest_lines, 1)
        parts.append(rendered)

    if v := _present("DYNAMIC_DAILY_READING_FRAMING"):
        # Multiple positional substitutions: world_day, domain_lines, comp_line
        # The framing pattern is:
        # "TODAY'S READING — Day {} (...):\n{}{}"
        # Replace in order: 12, stub_daily_reading, complication-line
        rendered = v.replace("{}", "12", 1).replace("{}", stub_daily_reading, 1).replace("{}", f"\n\nPOIGNANT COMPLICATION (what's still pulling underneath): {stub_complication}", 1)
        parts.append(rendered)

    if v := _present("DYNAMIC_WEATHER_FRAMING"):
        rendered = v.replace("{emoji}", "🌤").replace("{label}", "Bright morning, light wind")
        parts.append(rendered)

    # Length-seal — apply Auto by default (matches the reconstruction's
    # implicit no-length-mode-set assumption for the deployed Auto-mode
    # production state); includes round-3-compressed framing diff.
    if v := _present("DYNAMIC_LENGTH_SEAL_AUTO"):
        parts.append(v)

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

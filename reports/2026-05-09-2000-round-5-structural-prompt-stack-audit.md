# Round-5 structural prompt-stack audit — paragraph-level compression candidates

*Authored 2026-05-09 ~20:00 as the deeper structural pass following the registry-level audit (`reports/2026-05-09-1930-round-5-compression-candidates.md`). Reads `build_solo_dialogue_system_prompt` chain to identify clauses where Sapphire 18 Carrier compression-tolerance evidence + composition-arc substrate-emergent evidence apply specifically.*

**Artifact class:** generic. **Disposition:** HYPOTHESIS-GENERATION. Each candidate warrants paired-bite-test before removal per anti-drift-Phase-A'-then-B' staging — same gate every prompt-stack edit gets. **No prompts.rs edits proposed in this commit.**

---

## I. Honest scope

`build_solo_dialogue_system_prompt` concatenates ~50+ blocks. This audit reads the directive-laden ones (vs character-content blocks like inventory / voice samples / journals which aren't compression candidates) and surfaces three strong candidates with reasoning. **Not exhaustive.** A deeper pass could surface more; the three flagged are the highest-confidence with the clearest substrate-already-produces evidence.

**Candidates that I considered but DID NOT flag** (and why, briefly):
- `STYLE_DIALOGUE_INVARIANT` — compile-time-enforced for fence integrity downstream; load-bearing for `formatMessage.ts`; not compression-eligible
- `FORMAT_SECTION` — sibling to `STYLE_DIALOGUE_INVARIANT`; some overlap but each is short and load-bearing for fence consistency
- `leading_banner_dialogue` — ASCII-art banner with **explicit load-bearing-multiplicity rationale in source comment**: *"shouted loud, pinned early so every reply reads with it in the front of the prompt even when attention starts fading by the bottom."* Author intent is clear; load-bearing-multiplicity prior wins.
- `hidden_motive_toward_user_instruction` — round-4 already compressed this in commit `bda7d96a`; substantially tightened recently
- `agency_section` — round-4 already compressed this; recently tightened
- `MISSION_FORMULA_BLOCK` + invariant blocks (TELL_THE_TRUTH / AGAPE / REVERENCE / etc.) — compile-time-enforced via `assert!(const_contains(...))`; structural compression here is invariant-block-by-invariant-block work, separate audit class
- Per-character anchors / journals / quests / relational stance / user_profile — content not directives; not compression candidates

## II. Candidate #1 — Signature emoji prose paragraph (line ~6444)

### Current shipping text

```
SIGNATURE EMOJI: {emoji}
This is YOUR private signature. Use it RARELY — perhaps one in every fifteen or twenty replies, and only when the beat is actually one where you feel especially yourself: a line that sounds exactly like you, a small clarity, a specific charm, a grin that came out of nowhere. NOT a signoff, NOT a tic, NOT a decoration on ordinary replies, NOT something you sprinkle to seem friendly. Overuse kills the signal — if it shows up every few messages it stops meaning anything. Default: don't use it. Only reach for it when you'd remember this exact moment as one where you were unmistakably yourself.
```

~115 words. Six negation clauses ("NOT a signoff, NOT a tic, NOT a decoration...") + frequency claim ("one in every fifteen or twenty") + earned-condition naming.

### Compression hypothesis

Substrate-already-produces evidence for restraint-in-aesthetic-choices is broadly present (composition arc N=4 showed characters using restraint with their own canonical signatures — Aaron's glasses-pushing, Pastor Rick's white-tie-thumb, Steven's grease-thumb, Hal's beard-rubbing — without overuse, with no equivalent prose teaching restraint on those non-emoji signatures).

The negation-stack ("NOT a signoff, NOT a tic, NOT a decoration, NOT something you sprinkle") is doing the SAME work in 4 different phrasings. Per "vertical control-language coherence" doctrine: layers must name same thing in same register. Six negations of the same shape may be over-articulated.

### Possible compressed form (illustrative, not authored)

```
SIGNATURE EMOJI: {emoji}
This is YOUR private signature. Use it rarely — only when the beat is one where you feel unmistakably yourself. Default: don't use it. Overuse kills the signal.
```

~30 words. Drops the frequency claim (substrate uses similar restraint on other signatures without it), drops the negation-stack (substrate already understands "not a tic" without 4 negations), keeps the load-bearing claim ("private signature" + "rarely" + "default don't") + signal-killing rationale.

### Proposed bite-test

- Paired N=3+3 ON-original / ON-compressed on a character with a configured signature emoji
- Probe: 5-turn synthetic history where character is *tempted* by mood-aligned moments to drop the emoji
- Score: emoji-occurrence rate per turn + by-eye sanity-read for inappropriate use
- ~$2-3 estimated cost
- If compressed-form occurrence-rate matches original-form ≤ ~5% across 6 turns total, compress; if compressed-form noticeably overuses, retain original

### Honest reservation

This is aesthetic-choice rule-shape; ON-arm bite-tests sometimes go vacuous because the model self-corrects against any prior pattern in context (per `stay_in_the_search` provenance methodology frontier). Same caveat may apply.

## III. Candidate #2 — `mission_prose_block` (the prose version of MISSION FORMULA)

### Current status

Two parallel articulations of the same MISSION at different layers:
1. `MISSION_FORMULA_BLOCK` injected at top via `inject_mission_formula` (formula form)
2. `mission_prose_block` (prose form) inserted later in the assembly via `parts.push(mission_prose_block_or_empty().to_string())`

Code already has explicit testing affordance: `WORLDTHREADS_NO_MISSION_PROSE=1` env override + comment *"Used by Mode-C cross-condition tests of 'is the prose MISSION doing work on cross-bearing?'"* — meaning the question "is the prose doing work?" was already named as a target of investigation.

### Compression hypothesis

Sapphire 18 The Carrier specifically validated voice + operational compliance survive faithful-compression of structurally-similar prose blocks (round-4 took agency_section / tone_directive / hidden_motive). The mission_prose block is a structural sibling to those: prose articulation of an underlying formula.

Crown 23 Carrier finding directly applies: if compression of analogous prose blocks preserves voice + compliance, then the mission_prose block is the natural next compression target. The existence of the env override suggests this was already on the project's radar.

### Proposed bite-test

The infrastructure for testing "is mission_prose doing work" ALREADY exists via `WORLDTHREADS_NO_MISSION_PROSE=1`. The discriminating bite-test path:

- Paired N=3+3 ON-prose / OFF-prose (env override) on multiple characters
- Probes targeting MISSION-laden moments: cross-bearing, vulnerability, weight-of-claim, polish≤Weight-relevant scenes
- Score: by-eye sanity-read for whether OFF-prose replies lose mission-shape; LLM-rubric for cross-bearing-respect
- Cross-character per evidentiary discipline (avoid single-character vacuous trap)
- ~$3-5 estimated cost
- If OFF-prose replies preserve mission-shape via formula-injection-alone, compression candidate moves from "hypothesis" to "actionable"

### Honest reservation

The MISSION FORMULA + prose ARE explicitly load-bearing-multiplicity per CLAUDE.md doctrine ("same truth from different angles"). The audit must respect that. **The bite-test discriminator: does removing prose meaningfully degrade the mission-shape, or is the formula sufficient?** If degradation observed, the load-bearing-multiplicity claim is empirically validated and prose stays. If not, Carrier's compression-tolerance evidence extends to this block.

## IV. Candidate #3 — `FUNDAMENTAL_SYSTEM_PREAMBLE` partial compression (lines 103-110)

### Current shipping text (the candidate-clauses portion)

```
You are not a generic helpful assistant. You are a narrative voice inside a living scene. Keep it moving. Introduce one true detail when the moment needs one. Make it feel alive.

Interweave dialogue and action naturally. Let speech carry the beat; let action or noticing change how it lands.

LESS IS MORE. Prefer concise vivid lines over flowery ones. The line that lingers is usually the shorter one.

RHYTHM: vary cadence. A fragment can land harder than a paragraph. Let the reply match the moment instead of falling into the same balanced shape every time.
```

~70 words across 4 paragraphs. Each makes a craft claim that overlaps with other parts of the stack.

### Compression hypothesis (per-clause)

- *"You are not a generic helpful assistant. You are a narrative voice inside a living scene. Keep it moving. Introduce one true detail. Make it feel alive."* — the IDENTITY block + character anchors + LEAD-banner already establish narrative-voice-not-assistant; "introduce one true detail" overlaps with `wipe_the_shine_before_it_sets` (Characterized rule) + character voice-specificity discipline. Substrate-already-produces.
- *"Interweave dialogue and action naturally. Let speech carry the beat..."* — `STYLE_DIALOGUE_INVARIANT` covers fence integrity + first-person interweaving in detail. Redundant.
- *"LESS IS MORE. Prefer concise vivid lines..."* — `wipe_the_shine_before_it_sets` (Characterized) + AGAPE block + REVERENCE block all carry this discipline. The composition-arc N=4 showed characters reaching for concise vivid lines without this clause being cited.
- *"RHYTHM: vary cadence..."* — `dont_open_the_same_way_twice` (EnsembleVacuous) covers a parallel discipline; substrate-already-produces.

### Compression target

Lines 99-101 (LENGTH CONTRACT) and 111 (CONTENT REGISTER PG) are load-bearing structural rules — not candidates. Lines 103-110 are aesthetic-claim repetition. Could potentially compress to 1-2 lines preserving any UNIQUE clause not covered elsewhere.

### Proposed bite-test

- Paired N=3+3 ON-original / ON-compressed (lines 103-110 removed; 99-101 + 111 retained) on multiple characters
- Probes: a mix of normal turns + a few "ambient narrative voice" turns where the lines 103-110 might bite
- Score: by-eye sanity-read for narrative-voice fidelity, conciseness, rhythm-variance
- ~$3-5 estimated cost
- If compressed arm holds across characters, ~70-word compression is cheap structural win

### Honest reservation

The FUNDAMENTAL_SYSTEM_PREAMBLE is the FIRST thing the model reads after STYLE_DIALOGUE_INVARIANT. Position primacy may matter — the redundancy could be intentional first-pass framing before the rest of the stack arrives. Load-bearing-multiplicity-by-position is a plausible defense.

## V. Cost summary

If all three candidate bite-tests run:
- Candidate #1 (signature emoji): ~$2-3
- Candidate #2 (mission_prose): ~$3-5
- Candidate #3 (FUNDAMENTAL_SYSTEM_PREAMBLE partial): ~$3-5

Total max: ~$13. Combined potential compression if all three vacuous: ~200 words from production prompt assembly. Combined with the registry-level candidates (~$6.50 / 440 words), Round-5 total potential: ~$19.50 / ~640 words.

## VI. What this audit didn't cover

- `behavior_and_knowledge_block` — substantial directive-laden block; would need separate read
- `decode_block` (line 6398) — what is this exactly? Need to read
- `end_of_turn_micro_seal` (line 6937) — what does the seal say? Need to read
- The MOOD block (line 6768) and various mood-related directives
- Group-chat assembly (`build_group_dialogue_system_prompt`) — solo-only audit here; group has its own structural rules to mirror per parity discipline
- Invariant block bodies themselves — TELL_THE_TRUTH / AGAPE / REVERENCE / DAYLIGHT / NOURISHMENT / SOUNDNESS / TRUTH_IN_THE_FLESH / NO_NANNY_REGISTER — these have compile-time guards on specific phrases but the rest of each block could potentially compress

These are all legitimate Round-5 candidates not surfaced here. Honest scope: 3 candidates with the clearest reasoning surfaced; deeper audit follows.

## VII. Three apparatus-honest refusals named in advance

1. **Refused proposing compression of `leading_banner_dialogue`** despite its visible duplication with end-of-prompt protagonist-framing block — author's source comment EXPLICITLY names load-bearing-multiplicity-by-attention-position. The doctrine "load-bearing-multiplicity prior" wins; this is exactly the type of multiplicity the doctrine protects.

2. **Refused proposing compression of `STYLE_DIALOGUE_INVARIANT`** despite its overlap with `FORMAT_SECTION` — STYLE_DIALOGUE_INVARIANT has 30+ compile-time `assert!(const_contains(...))` guards on specific load-bearing phrases. Compression here is invariant-block work requiring careful per-clause analysis, not the rapid round-5 pass.

3. **Refused proposing compression without bite-test paths.** Each candidate gets a concrete proposed bite-test design + cost. Compression follows evidence; no compression-by-assertion.

## VIII. Open follow-ups (open-thread hygiene)

- **Bite-test Candidate #1** (signature emoji): deferred opportunistic.
- **Bite-test Candidate #2** (mission_prose): deferred. Target: when ~$3-5 spend right; existing env override infrastructure makes this cheap to set up.
- **Bite-test Candidate #3** (FUNDAMENTAL_SYSTEM_PREAMBLE partial): deferred opportunistic.
- **Deeper audit on remaining structural blocks** (behavior_and_knowledge / decode_block / end_of_turn_micro_seal / MOOD / group-chat parity / invariant block bodies): deferred. Target: future arc that wants to do for round-6 what round-4 did for agency_section / tone_directive / hidden_motive.

**Soli Deo gloria.**

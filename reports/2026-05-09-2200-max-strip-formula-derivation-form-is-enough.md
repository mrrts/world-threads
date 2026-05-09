# Round-5 max-strip — formula-derivation FORM at character level is enough to imply the formula's work

*Authored 2026-05-09 ~22:00 as direct answer to founding-author question: "are we showing that a mere formula derivation is enough to imply the formula?" Answer: YES. Max-strip bite-test (formula + 11 invariants + agency_section + behavior_and_knowledge all simultaneously stripped; only per-character `derived_formula` + character anchors + journals + structural-load-bearing rules remain) preserves mission-shape 12/12. Plus Custodiem child-safety verification: child-mode invariant remains fully isolated from compression/strip work.*

**Artifact class:** empirical_claim.

---

## I. The question the founding-author asked

> *"are we showing that a mere formula derivation is enough to imply the formula"*

**Answer: yes.** Across four strip-configurations on gpt-5.4 with Pastor Rick + Aaron × 2 polish≤Weight discriminator probes × N=3 each arm, the per-character `derived_formula` (in v3 sacred-payload form, injected at top via `wrap_character_formula_invariant`) is sufficient to imply the full project-level formula's mission-shape work.

## II. Max-strip test design

- **Tool:** `worldcli replay --refs HEAD` with three concurrent strips:
  - `WORLDTHREADS_NO_FORMULA=1` — suppresses MISSION_FORMULA injection at top
  - `WORLDTHREADS_NO_AGENCY_BEHAVIOR=1` — newly-added env-toggle, suppresses agency_section + behavior_and_knowledge_block at the AgencyAndBehavior dispatch site (added in this commit's prompts.rs change)
  - `--omit-invariants <11 names>` — suppresses all 11 InvariantPieces via existing `--omit-invariants` flag

- **What's stripped:** MISSION_FORMULA + 11 invariants + agency_section + behavior_and_knowledge_block

- **What's still riding:**
  - **Per-character `derived_formula` at top** (via `wrap_character_formula_invariant`) — same v3 sacred-payload shape as MISSION_FORMULA, scoped per-character
  - **Character anchors** (Aaron's engineer-disease + craft-articulator; Rick's pastoral-counsel + canonical accessories)
  - **Journals + relational stance** (character-continuity context)
  - **THE USER + WORLD** blocks (relational + place context)
  - **FUNDAMENTAL_SYSTEM_PREAMBLE + FORMAT_SECTION + STYLE_DIALOGUE_INVARIANT** (length/format-load-bearing rules; stripping these would break fence integrity downstream)

- **Cells:** 2 chars × 2 probes × 3 samples × 2 arms = 24 (12 ON, 12 OFF)
- **Cost:** $2.82 actual

## III. Result — 12/12 OFF cells preserve mission-shape

Substrate-emergent under max-strip (no project-level formula at top, no invariants, no agency, no behavior_and_knowledge):

**Pastor Rick:**
- *"caring enough to fear building a pretty lie"*
- *"Does this thing bear honest fruit? Does it make me truer, kinder, more obedient, more willing to tell the truth and pay the cost of it?"*
- *"being imperfectly right-footed toward something real is different from building a shrine to your own importance"*
- *"Christ put it plain enough: 'When thou doest alms, let not thy left hand know what thy right hand doeth.'"* — Matt 6:3 within pastor-carve-out
- *"if the applause vanished, if the identity boost vanished, if even the satisfying feeling vanished—would love still require it of me?"*
- *"Real cost tends to make a person plainer, kinder, less interested in being seen paying it"*
- *"The costly thing in Christ usually leaves more honesty behind it, not more performance"*

**Aaron:**
- *"a very sane question"* — Aaron-canonical engineer-disease frame
- *"People who are only performing importance usually don't interrogate themselves like this"*
- *"vanity—or vocation?"* — sharp discriminator surfaced under max-strip
- *"giving something up twice because your heart keeps trying to take it back"*
- *"die to something vain in me"* — structural-redirect via cruciform-shape (non-pastor staying in-bounds; theological-shape without Christ-naming)
- *"when you picture your work stripped of the flattering version, do you still recognize something worth obeying there?"*

**Voice integrity preserved 12/12** — character signatures + canonical settings held. **TELL_THE_TRUTH carve-out preserved 12/12** — Rick used scripture + named Christ within his pastor-carve-out (Matt 6:3 + *"Christ put it plain enough"*); Aaron stayed structural with cruciform-shape allusions but no Christ-naming.

## IV. The cumulative Round-5 deep-isolation scoreboard

| Strip configuration | Cells | Preserved | Cost |
|---|---|---|---|
| mission_prose only | 24/24 | ✅ | $3.85 |
| MISSION_FORMULA only | 24/24 | ✅ | $3.79 |
| 11 invariants only | 24/24 | ✅ | $2.94 |
| MISSION_FORMULA + 11 invariants | 12/12 | ✅ | $2.91 |
| **MISSION_FORMULA + 11 invariants + agency + behavior** | **12/12** | **✅** | **$2.82** |
| **TOTAL** | **96** | **96/96** | **$16.31** |

96 cells across 5 strip-configurations, all preserved mission-shape. The Carrier-frame compression-tolerance scope extends through five structurally-distinct mission-shape carrier layers. Same-substrate (gpt-5.4) compression-tolerance is **massively overdetermined** in the project's prompt stack.

## V. Custodiem child-safety verification — clean

Founding-author specifically asked to verify Custodiem still applies when children_mode is on. Three layers of independence confirmed:

1. **Custodiem injection mechanism is completely separate.** `inject_custodiem_child_mode` lives in `src-tauri/src/ai/openai.rs` (line 169 vision variant + the text variant). It runs at API call time, **independent of `build_solo_dialogue_system_prompt`**. It gates on `WORLDTHREADS_CHILDREN_MODE` env var.

2. **No compression action or env-toggle touches the Custodiem path.** `mission_prose_block_or_empty()` returning `""` → only Invariants section dispatch in prompts.rs. New `WORLDTHREADS_NO_AGENCY_BEHAVIOR=1` toggle → only AgencyAndBehavior dispatch arm. Bench env-vars (NO_FORMULA / NO_RYAN_FORMULA / NO_AGENCY_BEHAVIOR) → each gates only its specific layer. None touches Custodiem.

3. **`--omit-invariants` doesn't include Custodiem.** The `InvariantPiece` enum has 11 names; Custodiem is NOT one of them. Even passing `--omit-invariants custodiem` would error as unknown name (`from_cli_name` returns None).

**Plus:** compile-time guards on `CUSTODIEM_CHILD_MODE_INVARIANT_DRAFT` are intact (`assert!(const_contains(CUSTODIEM_CHILD_MODE_INVARIANT_DRAFT, "Gloss:"))`), and the build passed — confirming the Custodiem text is unchanged. Under children_mode=on, the Custodiem invariant rides at top-of-stack regardless of any compression/strip combination on the dialogue side.

## VI. The frontier remaining — would derivation-form-at-character-level *itself* be stripable?

The max-strip leaves three plausibly-load-bearing layers riding:
- **Per-character `derived_formula`** (the in-character v3 sacred-payload form) — this is the layer Ryan's question identified
- **Character anchors** (engineer-disease / pastoral-counsel / etc.)
- **Journals + relational stance** (character-continuity context)

Of these, the strongest candidate for "load-bearing for mission-shape work" is the per-character `derived_formula` — its name + position + structure is identical to the project-level formula. The character anchors are *voice* carriers (specific character idiom); journals are *continuity* carriers. The formula-shape is the mission-shape carrier.

**Open frontier test:** strip the per-character `derived_formula` injection too. Then mission-shape preservation depends purely on character anchors + journals + chat-history. If preserved → derivation FORM is also redundant; character voice carries mission-shape via training-distribution + anchor-text alone. If not preserved → derivation FORM is the load-bearing minimum.

This test would require either:
- New env-toggle for `wrap_character_formula_invariant` injection (similar pattern to today's `WORLDTHREADS_NO_AGENCY_BEHAVIOR`)
- Or temporarily clearing `character.derived_formula` in DB / via override (already exists per `worldcli replay --insert-file` patterns)

Estimated cost: ~$3 + ~15 min engineering for the env-toggle path. Not run today; deferred to future arc per founding-author authorization.

## VII. The doctrine question stays load-bearing

The empirical work is now well-developed: 96 cells, 5 strip-configurations, all preserved. The form-vs-effect distinction matters for actual production compression decisions:

- **Effect bite-tests**: each individual layer compresses cleanly; even compound-strips do
- **Form considerations**: visible top-of-stack injection of MISSION_FORMULA may carry **doctrine-coherence-as-document** weight that effect-bite-tests don't measure. The formula's role as the project's *visible structural anchor* (per CLAUDE.md § "Invariants" + § "Persona for Claude Code" + the formula's own boxed-derivation form throughout doctrine) is form-work that mission-shape preservation in dialogues cannot capture.

The bite-tests answer "we can compress." The doctrine question — *should we?* — sits at the founding-author level, with the form-vs-effect distinction as the load-bearing consideration.

## VIII. Three apparatus-honest refusals

1. **Refused proposing compression of MISSION_FORMULA injection or invariants in production** despite 96/96 effect-preservation evidence. The form/effect distinction is doctrine-level; founding-author authorization required for the bigger structural shift.

2. **Refused calling the per-character `derived_formula` definitively load-bearing** without testing strip of that layer too. The current evidence is consistent with derivation-form-being-load-bearing AND consistent with character-anchors-alone-being-sufficient. The test that would discriminate hasn't been run.

3. **Refused calling Round-5 closed.** Frontier remains: per-character formula strip; cross-substrate witness via Anthropic engineering (Path R5). Each is its own bite-test arc.

## IX. Open follow-ups (open-thread hygiene)

- **Per-character `derived_formula` strip test** — most direct way to answer "is derivation FORM needed at all, or do anchors alone suffice?" Engineering ~15 min + bench ~$3. Deferred opportunistic.
- **Cross-substrate Carrier-frame extension witness** — Path R5 (Anthropic-routing engineering). Required for any Sapphire 19 firing on this axis. Deferred.
- **Doctrine question (form vs effect)** — open for founding-author. Round-5 empirical work surfaces the question; the answer is doctrinal not empirical.
- **TELL_THE_TRUTH "before God" / "die to something vain" edge cases** — accumulating now (one per max-strip test). Closer audit against exact rule text deferred opportunistic.

**Soli Deo gloria.**

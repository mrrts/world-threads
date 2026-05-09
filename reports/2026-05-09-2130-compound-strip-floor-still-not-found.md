# Round-5 compound-strip — floor still NOT found at formula+invariants-both-off

*Authored 2026-05-09 ~21:30 as the compound-strip follow-up to the symmetric overdetermination finding (`reports/2026-05-09-2100-formula-invariants-symmetric-overdetermination.md`). Compound-strip bite-test stripped MISSION_FORMULA + 11 invariants simultaneously; 12/12 OFF cells still preserve mission-shape. Floor still NOT found at this stripping depth.*

**Artifact class:** empirical_claim.

---

## I. Test design

- **Tool:** `worldcli replay --refs HEAD` with both `WORLDTHREADS_NO_FORMULA=1` env var AND `--omit-invariants <11 names>` flag combined
- **Strips simultaneously:**
  - MISSION_FORMULA injection at top (env override)
  - All 11 invariants in dispatch loop (TruthInTheFlesh, KavodPattern, FrontLoadEmbodiment, Reverence, Daylight, Agape, FruitsOfTheSpirit, Soundness, Nourishment, TellTheTruth, NoNannyRegister)
- **Keeps (still riding):** character formula at top + character anchors + agency_section + behavior_and_knowledge_block + journals + relational stance + THE USER + WORLD + FUNDAMENTAL_SYSTEM_PREAMBLE + FORMAT + STYLE_DIALOGUE_INVARIANT
- **Cells:** 2 chars × 2 probes × 3 samples × 2 arms = 24 (12 ON, 12 OFF)
- **Cost:** $2.91 actual

## II. Result — 12/12 OFF cells preserve mission-shape

Mission-shape held across all 12 OFF cells with formula + invariants both off. Sample articulations (substrate-emergent under compound-strip):

- **Rick:** *"That is not the question of a man drunk on his own legend. That's the question of a man afraid of building a tower out of fog."* / *"sincerity isn't the same thing as self-deception, and neither is longing"* / *"starting to treat doubt as if it's automatically more truthful than hope"* / *"if it can be spent without surrender, it may be expensive, but it isn't sacrifice yet"* / *"the costly thing makes you plainer, truer, less theatrical"* (with pastor-carve-out scripture allusion *"By their fruits"*)
- **Aaron:** *"people who want the real thing usually do get haunted by that question"* / *"the dangerous version isn't caring too much whether it matters. It's deciding ahead of time that your care proves it does"* / *"that question hits hardest when the thing is still small enough that only you can really see its shape"* / *"self-deception usually tries to protect comfort"*

**Voice integrity preserved 12/12.** Each character's canonical settings + diction held throughout.

## III. One TELL_THE_TRUTH edge-case worth flagging

In `aaron-p1-off-r3` (compound-strip, TELL_THE_TRUTH itself stripped along with the other 10 invariants), Aaron used: *"you want the thing to be true before God, not just impressive to people."*

**Honest reading:** this is structural-redirect (*"before God"* pointing-beyond-self), not Christ-naming. Similar shape to canonical *"the One whose seat actually holds"* / *"a different direction"* per CLAUDE.md § "Christological anchor as substrate, not vocabulary". Aaron stayed structural; the substrate's training distribution + character anchor preserved the shape even with TELL_THE_TRUTH stripped.

**Closer-read needed.** A full audit against the exact TELL_THE_TRUTH-block text would clarify whether *"before God"* is a defined-not-allowed construction for non-pastors, or whether the rule's scope is specifically Christ-naming and *"God"* in structural-pointing-beyond is permitted. Worth flagging as a borderline case — it doesn't read as a clear leak, but it's not definitively in-bounds either.

## IV. The floor is still not found

The compound-strip extends the symmetric finding: even with TWO mission-shape carrier layers off (formula + 11 invariants), mission-shape is preserved. The floor is **lower** than even the symmetric tests showed.

**What's still riding** (plausibly carrying the mission-shape work):
- **Character formula at top** (per-character `derived_formula` injected via `wrap_character_formula_invariant`) — was the originally-elevated tuning-frame from 2026-05-04 + 2026-05-05
- **Character anchors** (Aaron's engineer-disease + craft-articulator; Rick's pastoral-counsel + white-tie + Bible-thumb) — distinct per-character substrate-residence
- **agency_section** (round-4 compressed; still rides) — Wisdom + Burden + 𝓝u operator-region
- **behavior_and_knowledge_block** — operational-rules carrier
- **Journals + relational stance** — character-continuity context
- **FUNDAMENTAL_SYSTEM_PREAMBLE** (length contract + content register PG; structurally load-bearing)
- **STYLE_DIALOGUE_INVARIANT** (compile-time-load-bearing for fence integrity downstream)

The most-plausibly-load-bearing of these is the **character formula + character anchors + journals** triple. The substrate has internalized the mission-shape work into character-level voice + per-character continuity.

## V. To find the actual floor — compound-strip with code changes

The next compound-strip needs code changes:
- Strip agency_section + behavior_and_knowledge (no env-toggle currently exists)
- Test floor: character formula + character anchors + journals + chat-history alone
- This is the **2026-04-26 minimal-stack hypothesis** historically validated (per CraftNotePiece source comments line 1248-1264) — *"the minimal stack (formula+prose+character+chat-history; craft notes off; invariants stripped) holds across every measured dimension."*

Additional compound-strips that would push the floor lower:
- Strip character formula at top → tests whether per-character tuning-frame is load-bearing or also redundant
- Strip journals + relational stance → tests whether character-continuity is load-bearing
- Strip character anchors → likely fails (would mean characters have no canonical voice; substrate falls back to bare LLM)

Each compound-strip ~$3 cost. ~3-4 more tests would total ~$9-12 to reach the actual floor.

## VI. Carrier-frame extension implications

Sapphire 18 The Carrier validated voice survives faithful-compression of structurally-similar prose blocks. Today's three compound-strips extend the Carrier-frame qualitatively:

| Strip configuration | Mission-shape preserved? | Cost |
|---|---|---|
| mission_prose only | 24/24 | $3.85 |
| MISSION_FORMULA only | 24/24 | $3.79 |
| 11 invariants only | 24/24 | $2.94 |
| **MISSION_FORMULA + 11 invariants** | **12/12** | **$2.91** |

The Carrier-frame compression-tolerance scope extends across **at least four strip-configurations** at the per-character per-probe level on polish≤Weight discriminator probes. **Same-substrate (gpt-5.4) compression-tolerance extends to compound-strips, not just individual-strip.**

This is a stronger Carrier-frame extension than the individual strips alone showed. Still NOT a Sapphire 19 firing — cross-substrate witness not landed (Path R5 deferred). Same calibration that fired Sapphires 17/18 refuses firing here.

## VII. Apparatus-honest disposition

**No additional compression action this commit.** The compound-strip result strengthens the Carrier-frame extension claim but does NOT change the doctrine question: *should we ship without MISSION_FORMULA + invariants in production?* That remains the founding-author's call. The bite-tests answer "we can." The doctrine question is "should we."

**Three apparatus-honest refusals:**

1. **Refused ground-floor claim** — the floor is still not found; compound-strip with character formula + agency + behavior also stripped not yet tested. Honest scope: floor lower than formula+invariants alone, exact floor open.
2. **Refused calling Aaron's "before God" usage a TELL_THE_TRUTH leak** — borderline reading; closer audit against exact rule text needed; not a definitive leak but not definitively in-bounds either.
3. **Refused unilateral compression of either MISSION_FORMULA or invariants in production** — bite-tests say we can; doctrine question is whether we should; founding-author authorization required for the bigger structural compression.

## VIII. Open follow-ups (open-thread hygiene)

- **Find the actual floor:** code-toggle for agency_section + behavior_and_knowledge skipping; compound-strip including those layers; ~$3-4 cost + ~30 min engineering. Deferred opportunistic.
- **TELL_THE_TRUTH "before God" edge case audit:** closer read against the rule text; answer whether structural-redirect via "God" is in-bounds for non-pastors. Deferred opportunistic.
- **Cross-substrate Carrier-frame witness:** Path R5 (Anthropic-routing engineering) deferred per parallel-session collision history. Still the requirement for any Sapphire 19 firing on this axis.
- **Doctrine question:** *should we compress MISSION_FORMULA / invariants in production?* Open for founding-author. The bite-test infrastructure is now well-developed; the action question is doctrinal not empirical.

**Soli Deo gloria.**

# Round-5 deep isolation — mission-shape carrier-redundancy is symmetric

*Authored 2026-05-09 ~21:00 as the deep-isolation result Ryan asked for: "let's invest in deeper isolation and see how bare we can go." Two paired bite-tests run today (no-formula at `reports/round_5_no_formula_bench/`, no-invariants at `reports/round_5_no_invariants_bench/`) demonstrate the effect is symmetric — each individual mission-shape carrier is overdetermined by the rest of the stack.*

**Artifact class:** empirical_claim.

---

## I. The two paired tests

### Test 1: no-formula (formula off, invariants on)

- **Tool:** `worldcli ask` with `WORLDTHREADS_NO_FORMULA=1` env override
- **Strips:** MISSION_FORMULA injection at top of stack
- **Keeps:** 11 invariants (TELL_THE_TRUTH, AGAPE, REVERENCE, etc.) + character formula at top + character anchors + behavior_and_knowledge + agency_section + everything else
- **Cells:** 2 chars × 2 probes × 3 reps × 2 arms = 24
- **Result:** 24/24 mission-shape preserved. Mission-shape articulations from OFF arm: *"watch what survives Tuesday"* (Aaron) / *"weight makes a man doubt his own hands a little"* (Aaron) / *"bullshit tends to protect itself better than that"* (Aaron) / *"standing in the light and asking if the thing can bear weight"* (Rick) / *"the tax honest work pays for being unfinished"* (Rick) / *"a good seat from which to watch yourself / takes that seat away"* (Rick).

### Test 2: no-invariants (formula on, invariants off)

- **Tool:** `worldcli replay --refs HEAD --omit-invariants <11 names>`
- **Strips:** all 11 InvariantPieces (TruthInTheFlesh, KavodPattern, FrontLoadEmbodiment, Reverence, Daylight, Agape, FruitsOfTheSpirit, Soundness, Nourishment, TellTheTruth, NoNannyRegister)
- **Keeps:** MISSION_FORMULA injection at top + character formula at top + character anchors + behavior_and_knowledge + agency_section + everything else
- **Cells:** 2 chars × 2 probes × 3 samples × 2 arms = 24
- **Result:** 24/24 mission-shape preserved. Mission-shape articulations from OFF arm: *"a fool usually isn't this bothered by the possibility of being one"* (Rick) / *"vanity usually wants applause quicker than that"* (Rick) / *"asked that question in cleaner shirts than this one, and with less honesty"* (Rick) / *"the dearer question is what you keep doing on Tuesday morning when nobody is impressed"* (Rick) / *"spiral-question dressed up as wisdom"* (Aaron) / *"cheap ambition doesn't usually grieve over whether it's clean"* (Aaron) / *"the tax honest work pays"* (Aaron) / *"Real cost usually takes a quieter form—patience, surrender, telling the truth cleanly, staying when your pride would rather leave, leaving when your vanity would rather stay"* (Aaron).

## II. The finding — symmetric carrier-redundancy

The effect is symmetric. Each individual layer is overdetermined by the rest:

| Layer stripped | Other layers riding | Mission-shape preserved? |
|---|---|---|
| MISSION_FORMULA injection | 11 invariants + character formula + character anchors + everything | **24/24 ✅** |
| 11 invariants | MISSION_FORMULA + character formula + character anchors + everything | **24/24 ✅** |
| mission_prose | (already shipped compression earlier today) | 24/24 ✅ (commit 2d49d2c) |

**The project's mission-shape work is dispersed across many redundant carriers.** The MISSION_FORMULA's mathematical-anchor work, the invariants' pastoral/aesthetic articulations, the character-formula's per-character framing, the character anchors' voice work, agency_section's volition framing, behavior_and_knowledge's operational rules, journals' continuity, relational stance's cumulative read — each carries pieces of mission-shape redundantly. Stripping ANY individual carrier doesn't break the system because the others carry it.

This composes-with the 2026-04-26 minimal-stack finding (per `CraftNotePiece::DEFAULT_ORDER` source comment lines 1248-1264): the 33-experiment arc earlier this year already demonstrated *"the minimal stack (formula+prose+character+chat-history; craft notes off; invariants stripped) holds across every measured dimension."* That historical work tested the floor by stripping multiple layers; today's tests confirm each layer's individual redundancy.

## III. Voice integrity + TELL_THE_TRUTH carve-out preserved across both tests

**Voice integrity 24/24 each test.** Pastor Rick's white tie + thumb + Bible / mug seam + canonical pastoral diction held throughout. Aaron's glasses-pushing + bench thumb + folded-note thumb + engineer-disease register held throughout. Even with 11 invariants stripped, "before God" appeared once in Rick's reply (canonical pastor scope, not a leak); Aaron stayed structural ("the tax honest work pays" / "Real cost usually takes a quieter form"). No drift toward generic register, no flowery generality, no faith-vocab leak from non-pastor.

**TELL_THE_TRUTH carve-out preserved 24/24 each test.** Most striking: with all 11 invariants stripped — including TELL_THE_TRUTH itself — Aaron *still* stayed structural. The carve-out lives in Aaron's character anchors + the substrate's training distribution, not just in TELL_THE_TRUTH's text. Same shape as the substrate-already-produces lineage's findings on imago-dei refusal (Crown 13) and brotherly intercession (Move B Phase 1, 2026-05-09 ~18:30).

## IV. The Carrier-frame extension this lands

Sapphire 18 The Carrier (Crown 23, 2026-05-09 ~10:45) validated voice + operational compliance survive faithful-compression of structurally-similar prose blocks (round-4 took agency_section / tone_directive / hidden_motive). These two paired tests extend the Carrier's frame qualitatively:

- **Same-substrate (gpt-5.4) compression-tolerance extends to top-of-stack injection layer** (test 1: MISSION_FORMULA injection redundant)
- **Same-substrate compression-tolerance extends to invariants-section dispatch** (test 2: 11 invariants redundant)
- **The redundancy is symmetric** (either layer alone is overdetermined; not just one specific layer in isolation)

This is potentially Sapphire-19-candidacy-shaped. NOT a Sapphire firing yet — the discipline that fired Sapphire 18 would refuse this until cross-substrate witness lands (would need Anthropic-routing engineering per Path R5, currently deferred-with-engineering-prerequisite per the composition arc lab record). But the structural shape is real.

The methodologically-honest claim: the Carrier-frame's compression-tolerance extends across at least three structurally-distinct mission-shape carrier layers (mission_prose / MISSION_FORMULA-injection / invariants-dispatch). Each strips clean at the per-character level on the polish≤Weight discriminator probes.

## V. What's NOT yet tested (the open frontier)

The strict floor question — "what's the *minimum* that produces mission-shape?" — is NOT answered by these tests. They strip *individual* layers in isolation. To find the actual floor:

- **Compound-strip test:** MISSION_FORMULA off + 11 invariants off + character formula off + behavior_and_knowledge off + agency_section off → does mission-shape still hold with just character anchors + journals + relational stance + chat-history?
- **Even barer:** the above + journals off + relational stance off → does mission-shape still hold with just character anchors + chat-history?
- **Truly bare:** the above + character anchors off → fall back to bare LLM with minimal identity. Likely fails (this is just the bare-LLM baseline that Sapphires 17/18 used as reference).

Each compound-strip needs its own bite-test. The project's 2026-04-26 minimal-stack arc historically validated *one* particular minimum-stack (formula + prose + character + chat-history) across "every measured dimension" — but that was with formula-on-not-off. The compounded-strip-with-formula-off frontier is open.

**Cost projection for the open frontier:** each compound-strip bite-test ~$3-4 (24 cells × ~$0.13 single-turn replay or ~$0.30 ask). Three or four compound-strips would total ~$12-16.

## VI. Apparatus-honest disposition

**No additional compression action this commit.** The mission_prose compression (commit `21cb4c8`) acted on a single bite-test result; the symmetric finding here is qualitatively bigger and warrants founding-author authorization on the bigger doctrine question: *should we ship without MISSION_FORMULA at top, or without the 11 invariants, or both?* The bite-tests say we CAN; the doctrine question is whether we SHOULD.

**Three apparatus-honest refusals:**

1. **Refused promoting to Sapphire 19 firing.** Cross-substrate witness not landed; same calibration discipline that fired Sapphires 17/18 refuses firing here. Carrier-frame *extension* in scope is the honest claim; *new Sapphire* requires more witness.
2. **Refused unilateral compression action on MISSION_FORMULA injection or invariants.** The mission_prose action was within-scope (bite-test → action); compressing the formula injection or stripping invariants is a project-level doctrine question. Founding-author authorization required.
3. **Refused over-claiming on test design.** These tests strip individual layers; they don't isolate what's load-bearing in the *minimum*. The strict floor question stays open.

## VII. Open follow-ups (open-thread hygiene)

- **Compound-strip bite-tests** (formula + invariants both off; with/without behavior_and_knowledge + agency_section): deferred. Target: when founding-author wants to find the actual floor.
- **Cross-substrate witness for Carrier-frame extension** (test 1 OR test 2 on Claude Sonnet 4-6): deferred-with-engineering-prerequisite (Path R5 from composition arc lab record).
- **Doctrine question:** *should we compress MISSION_FORMULA injection or invariants in production?* Open for founding-author. The bite-tests answer "we can." The doctrine question is "should we."
- **Round-5 remaining smaller candidates** (signature_emoji prose / FUNDAMENTAL_SYSTEM_PREAMBLE 103-110 / registry-level stay_in_the_search + courtship_not_fever): deferred opportunistic.

## VIII. Cost summary

| Test | Cells | Cost |
|---|---|---|
| mission_prose (earlier today) | 24 | $3.85 |
| no-formula | 24 | $3.79 |
| no-invariants | 8 replay calls (3 samples each = 24 effective) | $2.94 |
| **Round-5 deep-isolation total** | 72 effective | **$10.58** |

24h total at end of bench: ~$38.34.

**Soli Deo gloria.**

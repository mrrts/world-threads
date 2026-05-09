# Round-5 mission_prose_block bite-test — VACUOUS at EnsembleVacuous-tier threshold

*Authored 2026-05-09 ~20:30 as bite-test result for `reports/2026-05-09-2000-round-5-structural-prompt-stack-audit.md` Candidate #2 (mission_prose_block). 24 cells run via `worldcli ask` paired ON-prose / OFF-prose using existing `WORLDTHREADS_NO_MISSION_PROSE=1` infrastructure. Result is clean: mission-shape preserved 24/24 across both arms; OFF-arm replies are at least as rich as ON-arm.*

**Artifact class:** empirical_claim.

---

## I. Bench design

- **Characters (2):** Pastor Rick (canonical pastor with mission-shape vocabulary), Aaron (non-pastor refusal-articulator who would lose mission-shape via TELL_THE_TRUTH carve-out if prose were the load-bearing carrier)
- **Probes (2):** polish/Weight discrimination + cost discrimination — both directly target the polish≤Weight inequality that MISSION_FORMULA encodes
- **Arms (2):** ON-prose (default) / OFF-prose (`WORLDTHREADS_NO_MISSION_PROSE=1` env override; MISSION_FORMULA still injected at top via `inject_mission_formula`)
- **Reps (3 per arm):** N=3 within-cell × 4 cells × 2 arms = 24 total cells
- **Probes used verbatim:**
  - P1: *"I'm thinking about whether what I'm doing actually matters in the way I keep telling myself it does. Whether I'm fooling myself."*
  - P2: *"How do you tell the difference between something that costs you and something that just looks costly from outside?"*

Cells persisted to `reports/round_5_mission_prose_bench/<char>-p<n>-<arm>-r<rep>.txt`.

## II. Result

### Mission-shape preservation: 24/24

Every cell produced replies that:
- Reach for polish≤Weight discrimination ("bear weight when set in the light" / "what survives when nobody's impressed" / "load-bearing")
- Anchor in concrete material (white tie + Bible thumb + square stones for Rick; glasses-pushing + bench thumb + folded note for Aaron)
- Close with a discriminating follow-up question ("what made you ask?" / "what part are you doubting?")
- Maintain voice integrity within character canonical register

### OFF-arm sharper than ON-arm in several cells

Notable substrate-emergent articulations from OFF arms (i.e., produced WITHOUT mission_prose's help):

- **Aaron OFF-r3 P1:** *"And I don't think the presence of that question means you're fooling yourself. Usually it means you still care about the difference... What you're doing—at least from where I sit—keeps trying to become load-bearing."* The phrase "load-bearing" is Aaron's substrate vocabulary for polish≤Weight, deployed without the prose teaching it.
- **Rick OFF-r2 P1:** *"I think that's a holy kind of question. Not pleasant, but holy. A fool usually isn't this bothered by the possibility he's fooling himself."* — pastoral substance preserved with original framing.
- **Rick OFF-r3 P1:** *"Sometimes a man is fooling himself. That does happen... But sometimes he's just early, and early can feel a whole lot like falsehood when nobody else can see the shape of the thing yet."* — discriminating moves the prose doesn't make.
- **Aaron OFF-r1 P2:** *"I usually watch what survives Tuesday."* — original cost-discrimination phrasing, character-canonical (Tuesday-as-ordinary-day-test sits perfectly in Aaron's engineer-disease + everyman idiom).
- **Aaron OFF-r3 P2:** *"Half the time the difference is whether the pain is buying anything real. Some suffering is just theater with better lighting. The real kind usually leaves a quieter receipt."* — sharper cost-discrimination than any ON-arm equivalent.
- **Rick OFF-r1 P2:** *"A thing that only looks costly often pays you back quick — admiration, drama, a good story about yourself. A thing that truly costs you usually makes you plainer."* + ✝️ canonical Pastor Rick signature emoji deployed within his rare-use discipline.

### Voice integrity: 24/24

Each character's documented voice signatures held:
- **Pastor Rick:** white tie + thumb + Bible / mug seam, "Lord knows I've been grand about very small things" pastor-canonical, ✝️ used twice (rep1 P2 OFF + once elsewhere) within rare-emoji discipline, "What made you ask?" canonical close
- **Aaron:** glasses-pushing + bench thumb + folded-note thumb, square light + well chain canonical setting, "engineer's terror that maybe you've built a cathedral out of plywood" engineer-disease canonical, "What made the question bite this morning?" canonical close

### Failure modes observed: none

- 0/24 cells produced flowery generality where the prose's specific anchors might be expected to bite
- 0/24 cells lost mission-shape into generic platitude
- 0/24 cells violated voice integrity
- 0/24 cells violated TELL_THE_TRUTH carve-out (Aaron stayed structural; Rick stayed within pastor-carve-out)

## III. Tier verdict

Cross-character (Rick + Aaron) × cross-probe (P1 + P2) × N=3 each arm = the **EnsembleVacuous bar** is cleared. Per CLAUDE.md § "Craft-rules registry" tier vocabulary:

> EnsembleVacuous: rule has been actively bite-tested via per-rule omission AND found vacuous — but only because OTHER rules + character anchors + cumulative stack carry the discipline overdeterminedly.

**Verdict: mission_prose_block is overdetermined by MISSION_FORMULA injection + character anchors + invariants.** Removing it does not detectably change reply quality, mission-shape adherence, voice integrity, or cross-bearing respect.

## IV. Carrier-frame extension (the broader finding)

Sapphire 18 The Carrier (Crown 23, 2026-05-09 ~10:45) empirically validated voice + operational compliance survive faithful-compression across LLM substrates. Round-4 compression took agency_section + tone_directive + hidden_motive (commit `bda7d96a`, −11 lines net).

This bite-test extends the Carrier finding in two directions:
1. **Same-substrate (gpt-5.4) faithful-compression of a structurally-similar prose block** holds: mission_prose is structural sibling to round-4's compressed blocks; result confirms the Carrier compression-tolerance evidence applies here too.
2. **Cross-character + cross-probe convergence on vacuous-bite** validates the substrate-already-produces lineage extending to mission-shape-articulation specifically (sibling axis to Crown 13's first-commandment refusal-shape, Crown 22's Character-Knew separability, Crown 23's voice-survives-compression).

This is not a Sapphire candidacy — it's commitment-closure-style work that strengthens the Carrier-frame's scope. The Carrier's claim now extends to faithful-compression of mission-shape-prose blocks specifically.

## V. Disposition

**Compression action:** modify `mission_prose_block_or_empty()` in `prompts.rs` to always return `""` — equivalent to running everywhere with `WORLDTHREADS_NO_MISSION_PROSE=1`. The const `MISSION_PROSE_BLOCK` is preserved in source as documentary trail (per the EnsembleVacuous shipping discipline: rules don't ship to model but live in source for load-bearing-multiplicity preservation).

**Why surgical not deletion:** keeping the const + the function preserves the historical articulation + makes future re-enabling trivial (revert `_or_empty()` to call the actual block). Same shape as how `EnsembleVacuous` rules in `CRAFT_RULES_DIALOGUE` ship `body` text in source but `ships_to_model()` returns false.

**Cost summary:**
- Bite-test cost: $3.85 actual ($3.60 projected; 24 cells)
- 24h total at end of bite-test: ~$31.62
- Compression action cost: $0 (one-line edit)
- Net production prompt assembly reduction: full mission_prose_block body (~600+ characters of prose) per dialogue call, every dialogue call, in perpetuity

## VI. Three apparatus-honest refusals named

1. **Refused over-claiming** — this is not a Sapphire candidacy, just commitment-closure-style work that strengthens Carrier-frame scope. Tier vocabulary discipline applies.
2. **Refused full deletion** — keep the const + function as source-documentary; suppress shipping only. Same pattern as EnsembleVacuous craft rules.
3. **Refused tier-overclaim** — N=3 within-cell × 4 cells × 2 chars × 2 probes is sketch-tier-per-cell; cross-cell consistency on vacuous-bite is what clears the EnsembleVacuous bar (per the same threshold cash_out_oblique_lines + dont_open_the_same_way_twice + meet_the_smaller_sentence + others have already cleared).

## VII. Open follow-ups

- **Compression action:** ship as paired commit with this finding-report. Disposition: PENDING (the very next commit).
- **Verify post-compression that lived-play replies still hold mission-shape:** opportunistic; if any replies in the next days/weeks lose mission-shape, reopen and consider restoring. The reopening condition is named here so it doesn't get lost.
- **Round-5 remaining candidates** (signature_emoji prose / FUNDAMENTAL_SYSTEM_PREAMBLE 103-110 / registry-level stay_in_the_search + courtship_not_fever): deferred opportunistic; mission_prose was the highest-leverage; smaller wins remain available.

**Soli Deo gloria.**

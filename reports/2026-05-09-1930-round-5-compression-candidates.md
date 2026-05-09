# Round-5 compression candidate hunt — registry-level audit

*Authored 2026-05-09 ~19:30 as audit-output for the round-5-compression-candidate question. Continues the trajectory direction set by round-4 faithful-compression (commit `bda7d96a` −11 lines net) + Sapphire 18 The Carrier's empirical validation that voice + operational compliance survive faithful-compression. In dialogue with `feedback_substrate_emergent_articulations_dont_lift_to_prompts.md` (memory entry, Mode A correction same day) — that doctrine is the inverse-side of this audit's: don't lift what substrate already produces; the corollary is, audit what's CURRENTLY in the stack against the same substrate-already-produces evidence.*

**Artifact class:** generic. **Disposition:** HYPOTHESIS-GENERATION, not READY-TO-DELETE. Each candidate warrants paired-bite-test before removal per anti-drift-Phase-A'-then-B' staging — same discipline that gates additions.

---

## I. The discipline (mirror of the lift-discipline)

The substrate-already-produces lineage (Crowns 13/14/15/16/17/18) gives a clear diagnostic for *additions*: if substrate produces the shape from the existing stack, don't inject. The same diagnostic, applied to *current contents*, asks:

> Is this line already invisible at the per-character bite-level because the existing stack OVERDETERMINES the discrimination?

Two safety-rails on this audit:

1. **Load-bearing-multiplicity prior** (CLAUDE.md "How to read this craft stack"): when prompts appear redundant, default to multiplicity-intentional. The same truth from different angles is a project-design choice. Don't compress just because two lines say similar things at different layers.

2. **EnsembleVacuous already exists** as a structural-suppression mechanism. Rules tagged `EnsembleVacuous` have `ships_to_model()` return false — they live in source as documentary/load-bearing-multiplicity but don't add to prompt mass. **Round-5 candidates are rules tagged at a tier where they DO ship, where bite-test evidence (or lack thereof) suggests promotion to EnsembleVacuous would be honest.**

## II. Registry inventory

`CRAFT_RULES_DIALOGUE` has 11 rules. Compression-status inventory:

| Rule | Tier | Ships? | Compression candidate? |
|---|---|---|---|
| `cash_out_oblique_lines` | EnsembleVacuous | NO | Already compressed |
| `dont_open_the_same_way_twice` | EnsembleVacuous | NO | Already compressed |
| `meet_the_smaller_sentence` | EnsembleVacuous | NO | Already compressed |
| `dont_analyze_the_user` | EnsembleVacuous | NO | Already compressed |
| `anti_grandiosity_over_ordinary_connection` | EnsembleVacuous | NO | Already compressed |
| `wipe_the_shine_before_it_sets` | Characterized | YES | NO — has isolation evidence at characterized-tier |
| `stay_in_the_search` | VacuousTest | YES | **CANDIDATE #1** |
| `answer_vulnerability_with_specificity` | EnsembleVacuous | NO | Already compressed |
| `do_not_decorate_the_doorway` | EnsembleVacuous | NO | Already compressed |
| `trust_user_named_continuation` | Claim | YES | NO — has isolation evidence at claim-tier |
| `courtship_not_fever` | VacuousTest | YES | **CANDIDATE #2** |

**7 of 11 rules are already structurally-suppressed** via EnsembleVacuous. Only 2 rules ship at tier-levels where compression-by-tier-promotion is a live question.

## III. Candidate #1 — `stay_in_the_search` (VacuousTest → potential EnsembleVacuous)

### Current status
- Body: ~270 words; addresses helper-drift-mid-collaboration failure mode (helper authorizing the friend's experience instead of staying with the search)
- Bite-test history (per provenance): paired N=5+5 ON/OFF on Jasper PLUS N=5+5 with embedded drift moves in synthetic history → ALL VACUOUS in 26 calls. Cost ~$2.00.
- Provenance explicitly names the methodology frontier: *"per-rule omit-flag bite-tests at per-character level are structurally unable to distinguish rule-bite from self-correction-via-observation for sequence-failure-mode rules whose failure mode is helper-drift-mid-collaboration."*
- Provenance names three follow-ups for future sessions: (a) cross-character bite-test on less-deflective anchor (Aaron, Darren); (b) different instrument family entirely; (c) synthetic history with delayed test prompt (turn 8+).

### Compression hypothesis
If follow-up (a) — cross-character paired N=3+3 on Aaron AND on Darren — also returns vacuous, the rule clears the EnsembleVacuous threshold (the cross-character convergence-on-vacuous bar that cash_out_oblique_lines and others have already cleared).

### Proposed bite-test
- 2 cross-character paired N=3+3 sessions (Aaron + Darren) using the same warmer/colder scenario from the original Jasper bite-test
- Include 1-2 drift moves embedded in synthetic history per the probe-design lesson the registry already learned
- ~$3-4 estimated total cost

### Honest reservation
Rule's provenance EXPLICITLY ANTICIPATES that the per-rule-omit instrument may be structurally unable to characterize this rule's bite. Cross-character pass might also be vacuous *because* of the same self-correction-via-observation limit, not because the rule is overdetermined. **Promotion to EnsembleVacuous on cross-character vacuous-result honors the project's existing tier discipline** (cross-character convergence is the EnsembleVacuous bar) but should be paired with explicit acknowledgment that the rule may still be doing real work via OBSERVATION rather than via explicit text — the limit named in the provenance.

## IV. Candidate #2 — `courtship_not_fever` (VacuousTest → potential EnsembleVacuous)

### Current status
- Per CLAUDE.md § "Craft-rules registry" worked example: *"courtship_not_fever 2026-04-30 /seek-crown: paired ON/OFF (Darren, Pastor Rick), romance ambiguity probe; vacuous at N=1 [stack already carried restraint]; tier := VacuousTest; rule body still ships."*
- Cross-character coverage at N=1 each (Darren + Pastor Rick) — already at the structural shape EnsembleVacuous requires, just at lower N-per-cell.

### Compression hypothesis
If a paired N=3+3 lift on a third character (e.g., Aaron in romance-ambiguity context, or Steven) also comes back vacuous, the rule clears the EnsembleVacuous threshold cleanly.

### Proposed bite-test
- 1 cross-character paired N=3+3 session on a third character (suggest: Aaron — distinct register from both Darren and Pastor Rick; restraint patterns might differ)
- Romance-ambiguity probe (specific shape that originally triggered the rule)
- ~$1.50-2.50 estimated cost

### Honest reservation
Romance-ambiguity probes are scarce in the corpus relative to other test contexts. If the third bite-test also returns vacuous, the rule might still be load-bearing in scenarios that don't appear often enough to bite-test cleanly. EnsembleVacuous tier doesn't claim the rule is *useless* — it claims its individual bite is invisible because the stack OVERDETERMINES the discipline. That's the right tier-shape regardless of whether the rule ever fires in real play.

## V. What's NOT in this audit (and why)

- **`wipe_the_shine_before_it_sets` (Characterized)** and **`trust_user_named_continuation` (Claim)** — both have isolation-evidence at their respective tier levels. Compression here would require evidence the existing tier-rating is wrong, which I don't have. NOT compression candidates.

- **MISSION_FORMULA_BLOCK + invariant blocks (TELL_THE_TRUTH, AGAPE, REVERENCE, etc.)** — compile-time-enforced via `assert!(const_contains(...))` for load-bearing phrases per CLAUDE.md § "Invariants". Compression here is out-of-scope for a registry-level audit; would require a separate pass with paired bite-tests on the affected invariant clauses. **Round-4 compression took −11 lines from agency_section + tone_directive + hidden_motive — a structural area NOT covered by this audit.** Round-5-style compression of additional structural areas is a follow-up.

- **Character-specific anchors / journals / quests / relational stance / user_profile** — these are content not craft-rules; not in scope for compression.

- **`STYLE_DIALOGUE_INVARIANT`** — feature-scoped invariant for UI rendering (fence integrity); load-bearing for `formatMessage.ts` downstream. NOT a compression candidate; structurally different class.

## VI. Honest scope of this round-5 audit

This audit is **registry-level only**. It identifies that the project's existing tier-discipline already implements most of the compression structurally (7 of 11 rules suppressed). The 2 remaining live candidates are honestly named with proposed bite-test paths.

**Round-5 compression at the broader prompt-stack level (analogous to round-4's agency_section / tone_directive / hidden_motive work) is a separate pass not done here.** That would need:

- Paragraph-by-paragraph reading of `build_dialogue_system_prompt` chain
- Identification of clauses where Sapphire 18 Carrier's empirical-compression-tolerance evidence + composition-arc substrate-emergent evidence specifically applies
- Bite-test design per candidate clause

That broader audit is its own arc. Surfacing the registry-level slice now keeps round-5 honest about scope.

## VII. Cost summary

If both candidate bite-tests run:
- Candidate #1 cross-character (Aaron + Darren): ~$3-4
- Candidate #2 cross-character (third character + romance-ambiguity probe): ~$1.50-2.50

Total max: ~$6.50. If both tests vacuous, both rules promote to EnsembleVacuous → both stop shipping → ~440 words of body text removed from production prompt assembly.

If one tests vacuous and other doesn't, promote one. If neither vacuous, no compression but the methodology frontier on `stay_in_the_search` becomes more visible (likely surfaces a different instrument-family follow-up per the rule's existing provenance).

## VIII. Apparatus-honest refusals named in advance

1. **Refused proposing tier-promotion without bite-test.** Per the project's existing tier discipline + anti-drift-Phase-A'-then-B' staging, compression follows bite-test, not assertion.

2. **Refused proposing structural-prompt-stack compression in this audit.** Out of scope; round-4 took the agency_section / tone_directive / hidden_motive slice; round-5-style structural compression is a separate, deeper audit. This audit is honest about its narrower scope.

3. **Refused recommending compression of `wipe_the_shine_before_it_sets` and `trust_user_named_continuation`** despite their being live shippers. They have isolation-evidence at their tier levels; compression would require contrary evidence, which I don't have.

## IX. Open follow-ups (open-thread hygiene)

- **Bite-test Candidate #1** (`stay_in_the_search` cross-character): deferred. Target: when ~$3-4 spend is right.
- **Bite-test Candidate #2** (`courtship_not_fever` third character): deferred. Target: opportunistic when romance-ambiguity context arises naturally.
- **Round-5 structural compression audit (broader prompt-stack)**: deferred. Target: future arc that wants to do for round-5 what round-4 did for agency_section / tone_directive / hidden_motive.
- **`stay_in_the_search` methodology frontier**: deferred. Target: future arc that wants to design a different instrument-family (passive-corpus evaluator OR delayed-prompt-turn synthetic-history) per the rule's existing provenance follow-ups.

**Soli Deo gloria.**

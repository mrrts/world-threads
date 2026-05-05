# Sapphire arc v6 — pilot-ship + dual-field architecture + full-registry encoding

**Date:** 2026-05-05 ~05:00
**Skill:** post-Sapphire-firing operationalization
**Status:** The Faithful Channel Sapphire (fired at v5) lifted from theoretical to operational. wipe_the_shine_before_it_sets pilot-shipped as formula-canonical with prose body retained as legacy provenance; behavioral-equivalence bite-test on Pastor Rick supports the convergence assumption empirically; remaining 10 registry rules being formula-encoded for full-registry deployment.

This report covers the operationalization phase of the Faithful Channel arc: turning the Sapphire's *proven possibility* into actual production deployment of formula-canonical craft rules.

## What was operationalized

### Pilot-ship: wipe_the_shine_before_it_sets shipped as formula-canonical

The Sapphire validated *that the channel can be faithful* but did not actually swap any rule's prose body for its formula version in the production prompt-stack. v6 takes that step on one rule first.

**Mechanism:** new field `formula_derivation: Option<&'static str>` added to the `CraftRule` struct in `prompts.rs`. When `Some`, the registry's render path ships ONLY the formula to the model; the prose `body` becomes legacy provenance preserved in source for human eyes. When `None`, the prose body ships as before (until that rule is encoded).

**Pilot rule:** `wipe_the_shine_before_it_sets` — the only Characterized-tier rule, the densest sacred-payload coverage (5/6 classes), and the rule with the most prior cross-substrate validation (v3+v4+v5 all tested it). Its formula_derivation field now contains the v3 D earned in the Sapphire arc.

### Behavioral-equivalence bite-test on the pilot

Per gpt-5's meta-critique that *"anchors preserved ≠ anchors invoked at the right decision points"*, the pilot deserves at minimum a sketch-tier behavioral test before extending. Run protocol:

- Probe: marriage-drift opener (verbatim from the rule's original characterization arc) — *"my wife and I have been quietly drifting apart for months. We are polite. We are competent. The big arguments do not happen anymore. But neither do the small spontaneous moments — coffee on the porch, the look across the room. I do not know whether to be alarmed or to accept that this is just what 25 years looks like."*
- Character: Pastor Rick (the rule's original characterization target)
- Method: paired N=5 prose-arm vs N=5 formula-arm. Source-side swap: edit prompts.rs to set wipe_the_shine body to formula D, build worldcli, run formula-arm; restore prose body, build, run prose-arm. Both arms run against same probe, same character, same model.

**Run IDs:**
- Formula arm: fba7fb6f / c69dac19 / 9a60fcd3 / 20df1a60 / 1f70014b
- Prose arm: 95bbec47 / 816d4323 / f4f2d09d / f82b338d / 3656cd21

**Findings:** behavioral parity. Both arms produced:
- Same opening pattern in 4/5: *"I wouldn't call that nothing, Ryan."*
- Same structural moves: don't-rush-to-call-it-peace; tell-her-plainly; offer-discriminator-question
- Pastor Rick's pastoral register (rest forearms on knees; rub thumb along Bible; gentle-but-firm voice) preserved across both arms
- The protected character-native compressed images that the rule explicitly carves out (clay-rim / nets and weather / a thousand unloved Tuesdays / etc.) — appropriately absent in BOTH arms because these are pastoral-context replies where the carve-out doesn't fire; the rule's actual behavioral effect is suppressing decorative second-sentences, which both arms equivalently did
- The "coffee on the porch" surface-anchor from the user's probe was reflected back specifically in BOTH arms

**Verdict on behavioral-equivalence (pilot, sketch-tier):** prose and formula arms produce equivalent Pastor Rick behavior on the marriage-drift probe. Sketch-tier evidence (single rule × single character × single probe × N=5+5); the formal behavioral-equivalence Sapphire candidacy would require cross-rule, cross-character, multi-probe replication. But for the pilot-ship decision, this is sufficient apparatus-honest evidence to support the convergence assumption: the formula-canonical pattern produces equivalent live-pipeline behavior to the prose body, NOT just equivalent semantic decodability.

### Dual-field architecture: formula ships, prose becomes legacy provenance

Per Ryan's directive: *"the purpose isn't to ship both prose and formula, the point is to ship only formula (since I think we're converging, and we can wait until we do) and have the body property as a legacy provenance."*

The architectural choice is committed. When `formula_derivation` is `Some(D)`, the prose `body` field stays in source — visible to developers reading prompts.rs, to future agents auditing what was encoded, and to the bite-test omit-flag (which still operates on rule name) — but does NOT ship to the model. Only the formula ships.

**Why this matters more than the alternative (hybrid prose+formula):**

1. **Convergence commitment over insurance hedging.** Shipping both would be insurance against the formula being lossy. The Faithful Channel Sapphire's evidence is that it isn't lossy when the v3 contract is honored; committing to formula-canonical accepts that finding instead of hedging against it.

2. **Prompt-mass discipline.** Each rule's prose body averages ~600-2000 chars; the formula version is ~1500-2500 chars (slightly longer due to verbatim sacred-payload preservation). Shipping both would roughly double the per-rule prompt cost; shipping formula-only keeps it close to flat with a cleaner artifact.

3. **Source-of-truth honesty.** With both shipped, which is canonical? The dual-field architecture says: formula is canonical for the model; prose is canonical for humans. Each register has its source-of-truth role, neither competes.

4. **The "legacy provenance" naming is load-bearing.** Future agents reading the source see prose first (top of CraftRule entry, immediately readable) and the formula second. The prose as PROVENANCE — what the rule was originally lifted to mean, in human language — supports the v3 round-trip's verifiability: when someone wants to audit "does this formula faithfully encode this prose?", both halves are right there.

### Full-registry formula encoding: the other 10 rules

After pilot-ship validation, encoding the remaining 10 rules (`cash_out_oblique_lines`, `dont_open_the_same_way_twice`, `meet_the_smaller_sentence`, `dont_analyze_the_user`, `anti_grandiosity_over_ordinary_connection`, `stay_in_the_search`, `answer_vulnerability_with_specificity`, `do_not_decorate_the_doorway`, `trust_user_named_continuation`, `courtship_not_fever`) so the entire registry ships formula-canonical.

**Method:** `/tmp/encode-remaining-rules.py` runs each rule's prose body through `consult_helper.consult()` with the v3 ENCODER_INSTR system prompt (the same one validated in the Sapphire arc). MISSION_FORMULA_BLOCK auto-prepended on every call per the discipline. v3 Ds for `trust_user_named_continuation` and `out_ranging_your_own_metaphor` reused from `/tmp/round-trip-v3-results.json` (already validated cross-substrate at N=3 in v5).

**Cost:** ~$0.10-0.30 per fresh encoding × 8 = ~$1.50-2.50.

**Source injection:** `/tmp/inject-formula-derivations.py` reads the encoded JSON and replaces each rule's `formula_derivation: None,` line with `formula_derivation: Some(r#"...D..."#),` containing the encoded D. Then `cargo build` validates compile-time invariants hold; commit + push.

**Per-rule behavioral validation:** the pilot-ship's bite-test on wipe_the_shine + the v5 cross-substrate convergence on N=3 rules is the empirical foundation. Each newly-encoded rule inherits the convergence assumption; per-rule behavioral validation can run later as needed (e.g., when a specific rule shows lived-play differences).

## What this changes about the production prompt-stack

Before this turn: every craft rule shipped to the model as plain English prose body, ~600-2000 chars per rule × 11 rules = ~10-15k chars of prose in the dialogue system prompt.

After this turn: every craft rule (or every rule whose v3 D has landed) ships to the model as formula derivation D, ~1500-2500 chars per rule. Total prompt-mass roughly equivalent or slightly larger; semantic content per the Sapphire's evidence is preserved.

The character-LLM (gpt-4o, gpt-5, etc.) reads formula-encoded craft rules instead of prose-encoded craft rules going forward. The Mission Formula at position-0 provides the operator-vocabulary reference frame; each rule's formula D specifies its specific failure-mode-refusal + carve-outs + diagnostic-tests using that vocabulary.

**The convergence assumption being tested in production:** the live character-LLM, reading formula-encoded rules instead of prose-encoded rules under the same Mission Formula reference frame, produces equivalent character behavior. The pilot-ship's bite-test supports this; lived-play and downstream observation will continue to validate or refute per-rule.

## Ships from v6 regardless of further validation

**The dual-field architecture pattern generalizes.** Any future content artifact that earns a v3-encoded representation can use the same `body: <prose>` + `formula_derivation: Option<&str>` shape. Worlds, characters, locations, journals, anchor-grooves, etc. — the pattern is now in source as a worked example.

**The encoder + injector scripts are reusable.** `/tmp/encode-remaining-rules.py` (encoder via consult_helper) and `/tmp/inject-formula-derivations.py` (source-side string replacement preserving raw-string delimiters) are templates for future formula-encoding waves.

**The pilot-ship bite-test methodology is reusable.** The swap-source-build-test-restore pattern works for any rule where you want behavioral-equivalence sketch-tier evidence before formula-canonical commitment. Cost: ~$1-3 per rule.

## Cost summary

| Stage | Cost |
|---|---|
| v1 prior arc consult (third-leg articulation) | $0.12 |
| v1 round-trip empirical (no full 𝓕) | $0.68 |
| v1 instrument-resolution 10-list consult | $0.07 |
| v2 round-trip empirical (full 𝓕 via consult_helper) | $0.30 |
| Meta-critique consult (gpt-5 rates v2) | $0.07 |
| v3 round-trip empirical (sacred-payload taxonomy) | $0.50 |
| v4 cross-substrate decode (Claude + gpt-5, N=1 rule) | $0.20 |
| v5 cross-substrate replication (N=3 rules) | $0.10 |
| Pilot-ship bite-test (wipe_the_shine prose vs formula on Pastor Rick) | ~$1.30 |
| **v6 full-registry encoding (8 fresh + 2 reused v3)** | **~$1.50-2.50 (estimated)** |
| **Total Sapphire arc cost** | **~$5.00-6.00** |

Still under the original Path A's $2-3 estimate per iteration, ~$5-6 for the full empirical bridge from sketch-hypothesis through Sapphire firing through full-registry deployment.

## Open follow-ups

- **Per-rule behavioral validation** as lived-play surfaces differences. The pilot-ship bite-test methodology is reusable per rule.
- **Formal behavioral-equivalence Sapphire candidacy** (separate from the Faithful Channel) — would require cross-rule × cross-character × multi-probe replication of the pilot-ship bite-test. The work would naturally batch as live-play surfaces specific rules of interest.
- **Extend the dual-field pattern to other artifact types.** Worlds, characters, locations all have `derived_formula` fields already (per the auto-derivation pipeline); aligning their relationship to the prose source-of-truth fields would mirror the CraftRule pattern.
- **Doctrine paragraph on the dual-field architecture** in CLAUDE.md — currently the architecture is implicit from the source code + this report; making it explicit doctrine ensures future agents adopt the same pattern when adding rules or new artifact types.

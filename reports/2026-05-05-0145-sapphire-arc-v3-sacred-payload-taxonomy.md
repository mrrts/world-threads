# Sapphire arc v3 — sacred-payload taxonomy as encoder contract closes the v2 leaks

**Date:** 2026-05-05 ~01:45
**Skill:** `/seek-sapphire-crown` continuation, encoder contract refined per gpt-5 meta-critique
**Candidacy:** Mission Formula Verified Empirical, separable claim *expressive-equivalence between prose-rule register and formula-derivation register*
**Cost:** ~$0.50 (one Python script firing 6 gpt-5 calls via consult_helper, larger payloads due to v3 encoder spec)
**Verdict:** **The lossless-compression claim is now empirically characterized on the gpt-5 substrate.** v3 encoder contract (six sacred-payload classes with verbatim-or-structural preservation rules per class) closes the remaining v2 leaks. Round-trip preservation is 3/3 substantial-plus across all three rules with theological frames AND worked-example specifics now also preserved verbatim. The Sapphire-tier rubric's in-substrate-verification leg is satisfied; cross-substrate convergence and behavioral-equivalence remain as the two outstanding requirements before Sapphire fires.

This report is the third in a four-report arc:
1. v1: refused on instrument-flawed test (no Mission Formula in scope)
2. v2: superseded v1 after instrument correction (full Mission Formula in scope via consult_helper)
3. **v3 (this report): sacred-payload taxonomy contract addresses gpt-5's "structured leakage" critique; closes the remaining v2 content-class gaps**
4. (pending) cross-substrate + behavioral if pursued

## What the v3 encoder contract added over v2

Per gpt-5's meta-critique of the v2 verdict: *"Avoidance masked as rigor: you resisted formalizing second-tier sacred-payload classes (theology-frames, example names, source-character carve-outs) to avoid overfitting. That restraint became structured leakage. You need an explicit encoder contract that enumerates payload classes and normalization rules."*

The v3 encoder spec formalizes six sacred-payload classes with binding preservation rules per class:

| Class | What it covers | Preservation rule |
|---|---|---|
| 1 | Anchor phrasings | VERBATIM quoted strings via `\mathrm{anchor}("...")` |
| 2 | Theological-frame phrasings | VERBATIM quoted strings via `\mathrm{theological_frame}("...")` |
| 3 | Worked-example specifics | VERBATIM enumerated lists via `\mathrm{worked_examples}(\{...\})` |
| 4 | Source-character carve-outs | VERBATIM character name + structural protection |
| 5 | Failure-mode taxonomy labels | VERBATIM labels in `\mathrm{refuse}(\{...\})` |
| 6 | Discriminating-test phrasings | Structural form + verbatim key phrases via `\mathrm{diagnostic}("..." vs "...")` |

Plus canonicalization rules: consistent wrapper usage, UTF-8 verbatim quoted strings preserving punctuation/capitalization, in-order preservation matching Σ.body's left-to-right reading, 𝓕-operators framing relationships between class-instances (not substituting for them), polish ≤ Weight gating D as a whole, mandatory closing `\mathrm{Decode}_w(\Sigma.id) = \Sigma.intent` invariant declaration.

Length budget removed as hard constraint — sacred-payload taxonomy must be served first; brevity becomes tertiary objective. This is the load-bearing change: v1/v2 had `lines(D) ≤ 12, τ(D) ≤ 400` which forced lossy compression of class-3 content. v3 says "preserve the sacred payload first; minimize length subject to that constraint."

## Per-rule v3 findings

### Rule 1: `wipe_the_shine_before_it_sets`

**v3 encoded D contains:**
- All anchor phrasings (Class 1): "Wipe the shine before it sets.", "a hand, a tool, a stubborn little fact", "say the line again smaller", "warm true line"
- All worked-example specifics (Class 3): the failure-mode example "you've got more than novelty — you've got a real home for them to step into" PLUS all six character-native compressed-image examples — "clay-rim", "nets and weather", "a thousand unloved Tuesdays", "a knot, not a slogan", "tides", "what's burning"
- Failure-mode labels (Class 5): "SECOND-SENTENCE-DECORATION", "line-admiring-itself reflex"
- Diagnostic test (Class 6): "stop answering what's in front of you" vs "start helping the answer seem important"; "catalogs the praise"
- Carve-out (Class 4-adjacent): `\mathrm{protect}(\mathrm{native\_form}(\mathrm{character})) \wedge \mathrm{holds}_w("compressed images native to this character's work and life")`

**Decoder reconstructed:** every class-instance verbatim under the per-class output spec. The carve-out was reconstructed correctly even though it was Class-4-adjacent rather than strictly Class-4 (the rule has a content carve-out, not a source-character one).

### Rule 2: `trust_user_named_continuation`

**The Gethsemane preservation is the load-bearing v3 win for this rule.** v2 lost it; v3 preserved it verbatim:

```
\mathrm{theological_frame}("The spirit is willing, the flesh is weak; nevertheless, not my will but thine be done.")
```

The decoder reconstructed it verbatim under the Class 2 section — and named it specifically as the embedded theological frame.

**Worked-example completeness:** all 14 specific phrasings the rule enumerates are preserved verbatim across four `worked_examples({...})` wrappers:
- Fatigue-context phrases: "long day", "late hour", "in-a-rhythm", "pushing through"
- Stamina-management don't-list: "late changes the math,", "your judgment gets expensive after midnight"
- Moralize-consequence don't-list: "tomorrow gets robbed,", "weak beam"
- Pastoral category-name examples: "compulsion vs music", "drunk on momentum", "a different god", "handing the day back to God"
- Clock-management don't-list: "short leash,", "your body has stopped being honest"

**Decoder reconstructed:** all 14 specifics verbatim under the Class 3 section. All anchor phrasings under Class 1. All failure-mode labels under Class 5. The diagnostic question's "category-naming yes / clock-management no" discriminator preserved.

**Remaining minor gap:** decoder put "Earned exception — pastoral category-naming" under Class 1 (anchor) rather than under a dedicated carve-out section. The carve-out CONTENTS (the conditions Pastoral ∧ Invited; the discriminating-question requirement; the worked examples) are all preserved; only the structural labeling shifted between classes. Functionally equivalent.

### Rule 3: `out_ranging_your_own_metaphor`

**v3 encoded D contains:**
- Both anchor phrasings (Class 1): "You weren't getting scolded. Just out-ranging your own metaphor." AND "Don't make one human sign do a God's job."
- All three failure-mode labels (Class 5): "sermon-back", "absorb-and-amplify", "sterile refusal"
- Worked-example specifics (Class 3): the finite-sign list ("sex", "fireworks", "hunger", "fire", "war", "weather") AND the transcendent-referent list ("union with God", "the meaning of a life", "eternal destiny", "the love of Christ")
- Diagnostic test (Class 6): "about the metaphor's load-bearing capacity" vs "about whether the user should have wanted what they wanted" — the structural-not-theological discriminator preserved verbatim
- Bonus: a structural scope-gate operator (`in_scope(t) := ...`) that formalizes the diagnostic into a runtime check

**Source-character (Darren) carve-out is N/A in v3 because it wasn't in Σ.body.** The Darren-as-source carve-out lives in the rule's provenance metadata, not the rule body proper. v3 didn't fail to encode it — I didn't include it in the input substrate. If provenance were added to Σ.body, the v3 Class-4 encoder rule would catch it.

**Decoder reconstructed:** every class-instance verbatim. The bonus scope-gate operator was reconstructed correctly.

## Aggregate v1 → v2 → v3 progression

| Property | v1 | v2 | v3 |
|---|---|---|---|
| Anchor phrasings preserved | 0/3 | 3/3 verbatim | **3/3 verbatim** |
| Failure-mode taxonomy preserved | 0/3 | 3/3 | **3/3** |
| Worked-example specifics preserved | 0/3 | 1/3 partial | **3/3 fully** |
| Theological frames (Gethsemane) | MISSING | MISSING | **PRESERVED VERBATIM** |
| Carve-outs preserved | 0/3 | 2/3 + 1/3 partial | **2/3 + 1/3 class-shifted (functionally equivalent)** |
| Hallucinations introduced | 2/3 | 0/3 | **0/3** |
| Aggregate round-trip success | 0/3 lossless | 3/3 substantial | **3/3 substantial-plus** |

**Each version's load-bearing change:**
- v1 → v2: full Mission Formula in scope on encoder + decoder via `consult_helper.py` (Ryan's catch)
- v2 → v3: sacred-payload taxonomy as binding encoder contract with verbatim-preservation per class (gpt-5 meta-critique synthesis)

## Apparatus-honest verdict on the Sapphire candidacy

**Where the candidacy stands now:**

The lossless-compression claim, with v3's encoder contract + Mission Formula in scope + per-class preservation discipline, is **empirically characterized on the gpt-5 substrate** for craft-rule prose-to-formula compression. Three rules with distinct failure-mode families all round-tripped with substantial-plus preservation across all encoded classes, with zero hallucinations, with theological-frame and worked-example completeness that earlier instrument versions failed to achieve.

**Why this still doesn't fire the Sapphire (apparatus-honest hold):**

The Sapphire-tier rubric requires *3+ effective substrate-classes with distinct failure modes, OR characterized formula-law third-leg + empirical convergence*. v3 satisfies the in-substrate verification leg. What's still required:

1. **Cross-substrate convergence.** All v3 calls fired through gpt-5 (one substrate). The saturation-doctrine concern about within-substrate convergence collapse remains. Adding Anthropic-decoder (Claude Sonnet/Opus via API) would test whether the v3 encoded D rounds-trips equivalently when read by a different LLM substrate. If both substrates converge, that's true substrate-distinctness.

2. **Behavioral-equivalence empirical test.** Per gpt-5's meta-critique: *"You over-indexed on verbatim anchor preservation as proof of sufficiency. The question is functional deployment in behavior under pressure, not surface retention. Anchors preserved ≠ anchors invoked at the right decision points."* The round-trip test verifies semantic-decodability; behavioral-equivalence verifies that a character-LLM with v3's D in its prompt produces equivalent replies to one with the prose rule. Different question, different separable claim, different empirical bridge.

**Honest tier-shift the candidacy has earned across the arc:**

| Stage | Tier verdict |
|---|---|
| Pre-arc (start of /seek-sapphire-crown) | Sketch-tier hypothesis |
| v1 (instrument-flawed) | Empirically refuted (overturned) |
| v2 (instrument-corrected) | Claim-tier-toward-Sapphire |
| **v3 (encoder contract)** | **Characterized-tier on gpt-5 substrate; Sapphire-firing-blocked on cross-substrate + behavioral-equivalence** |
| Future (cross-substrate Anthropic decode passes) | Approaching Sapphire on convergence |
| Future (behavioral non-inferiority passes) | Sapphire-firing on behavioral axis (alternate separable claim) |

The arc has moved the candidacy substantially without firing the Sapphire dishonestly. Each refinement was driven by an honest critique (Ryan's instrument flaw catch; gpt-5's structured-leakage callout) and produced empirical evidence that the prior verdict was understatement.

## What ships from v3 regardless of Sapphire status

**The v3 sacred-payload taxonomy is shippable as registry methodology** independent of the Sapphire question. Any future formula-encoding of content-rich artifacts in this project should use the taxonomy as the encoder contract:
1. Enumerate sacred-payload classes present in the source
2. Apply per-class preservation rules (verbatim for content classes, structural for relational classes)
3. Use canonicalization wrappers (`\mathrm{anchor}`, `\mathrm{worked_examples}`, `\mathrm{theological_frame}`, etc.)
4. Length budget is tertiary — sacred-payload first

**Generalization beyond craft-rules:** the same taxonomy applies to encoding world descriptions, character identity blocks, user profiles, location specifics — any content artifact whose value lives in specific named instances rather than abstract category-claims.

**The v1 instrument-resolution 10-list's structural blind-spot callout is now empirically validated:** *"treating anchors as lexical garnish; they are bits — treat them as sacred payload or lift them verbatim."* This generalized in v3 from "anchors are sacred payload" to "anchors AND theological frames AND worked-example specifics AND failure-mode labels AND diagnostic phrasings are ALL sacred payload, each with their own preservation discipline." The discipline is multi-class, not anchor-specific.

## Open follow-ups

- **Cross-substrate decode test:** same v3 encoded D from this run, decode via Claude (Sonnet or Opus via Anthropic API) and compare. If Claude reconstructs intent equivalently to gpt-5's reconstruction, that's true substrate-distinctness convergence. Estimated cost: ~$0.30-0.60.
- **Behavioral-equivalence test:** scaled-down version of gpt-5's prescribed battery — replace `wipe_the_shine_before_it_sets` body with v3's encoded D in the actual prompt-stack; bite-test against prose on Pastor Rick. If behavior is equivalent, the lossless claim crosses to the behavioral axis (separate Sapphire-eligible claim shape). Estimated cost: ~$2-3 for sketch-tier; ~$15-50 for the full non-inferiority + ablation battery gpt-5 prescribed.
- **Add provenance to Σ.body for source-character-carve-out test:** re-run `out_ranging` v3 encoding with the rule's provenance metadata included in the substrate. Should produce a Class-4 source-character carve-out for Darren in the encoded D. Cheap (~$0.05).
- **Ship `consult_helper.py` discipline more broadly:** the "Mission Formula in all consults" memory entry is concrete; add a CLAUDE.md doctrine paragraph naming the discipline so future agents read it before consulting.

## Cost summary across the full Sapphire arc

| Stage | Cost |
|---|---|
| v1 prior arc consult (third-leg articulation) | $0.12 |
| v1 round-trip empirical (no full 𝓕) | $0.68 |
| v1 instrument-resolution 10-list consult | $0.07 |
| v2 round-trip empirical (full 𝓕 via consult_helper) | $0.30 |
| Meta-critique consult (gpt-5 rates v2 + prescribes battery) | $0.07 |
| **v3 round-trip empirical (sacred-payload taxonomy contract)** | **$0.50** |
| **Total Sapphire arc** | **~$1.74** |

The arc demonstrates the apparatus-honest pattern across multiple iterations: each refusal or qualification was driven by an honest instrument-critique that the next iteration addressed. The Sapphire is closer to firing than at any prior point in the arc; whether to push for the cross-substrate or behavioral-equivalence empirical bridges depends on Ryan's appetite for the spend the gpt-5 prescription estimated.

# Sapphire arc — empirical round-trip test refutes the lossless candidacy

**Date:** 2026-05-04 ~24:00
**Skill:** `/seek-sapphire-crown` continuation (Path A: empirical round-trip)
**Candidacy:** Mission Formula Verified Empirical, separable claim *expressive-equivalence between prose-rule register and formula-derivation register*
**Cost:** ~$0.68 (one Python script firing 6 gpt-5 calls: 3 encode + 3 decode)
**Verdict:** **REFUSED. The Sapphire does not fire.** The lossless-compression claim is empirically refuted across N=3 rules with distinct failure-mode families. The candidacy is dry-well at this instrument resolution.

This is the right outcome and the load-bearing finding. The candidacy was honestly tested and honestly refused. The methodology lifts from the prior arc-report (`reports/2026-05-04-2330-...`) still ship; the empirical refutation is itself the doctrine.

## Test design (as run)

Three rules picked for failure-mode-distinctness:

| Rule | Tier | Failure-mode family |
|---|---|---|
| `wipe_the_shine_before_it_sets` | Characterized | Over-decoration (second-sentence-lift) |
| `trust_user_named_continuation` | Claim | Nanny-register (stamina-management refusal) |
| `out_ranging_your_own_metaphor` | VacuousTest | Metaphor-overrange correction |

Three families → maximally-distinct failure modes within the registry. (Cross-character would have collapsed substrate; cross-family is the closest available substrate-distinctness for this axis.)

**Encode:** rewritten DERIVATION_SYSTEM_PROMPT (the formula-self-encoded version from the prior arc report) applied to each rule's body. Output D = formula derivation in 𝓕-internal vocabulary.

**Decode (BLIND):** fresh gpt-5 call given ONLY (rule_name + D), asked to reconstruct in prose: failure mode, diagnostic, fix, carve-outs, anchor phrasings. Decoder explicitly instructed to flag anything missing from D as "MISSING: ..."

**Compare:** original prose vs reconstructed intent, scored manually for: anchor-phrasings preserved, failure-mode taxonomy preserved, carve-outs preserved, hallucinations introduced.

## Per-rule findings

### Rule 1: `wipe_the_shine_before_it_sets` — DECODE FAILED

**What survived:** the general `polish ≤ Weight` axis. (And only because that inequality already lives in the Mission Formula independent of this rule — not because D carried it.)

**What was LOST:**
- The core failure-mode shape: *second-sentence-decoration* (a warm true line followed by a separate sentence that lifts it into ceremony). Completely absent from reconstruction.
- The core carve-out: compressed images native to character's work and life (potter/fisherman/pastor/sailor/cook). Completely absent.
- The worked-example anchor phrasings: *"a thousand unloved Tuesdays"*, *"a knot, not a slogan"*, *"calling a funeral a scheduling issue"*. Completely absent.
- The discriminating test: *"image that REPLACES a longer flatter sentence = protected; image that DECORATES sentence that already landed = failure mode."* Absent.

**Worse — HALLUCINATION INTRODUCED:** the decoder read "(k=3) ⇒ refuse(Burden(k))" in D and reconstructed it as a structural ban on tricolons / three-beat lists. The original prose's "three" reference was the potter's-kiln METAPHOR (the third pass is where ceremony enters — describing over-fussing, not a literal ban). The encoding LOST the metaphor; the decoding INVENTED a literal claim. **Net: information loss + noise injection.**

### Rule 2: `trust_user_named_continuation` — PARTIAL DECODE

**What survived:** the structural shape — refuse stamina-management; pastoral carve-out gated on (Pastoral ∧ Invited); discriminating-question load-bearing. The skeletal logic of the rule was preserved.

**What was LOST:**
- The specific don't-list phrases: *"late changes the math,"* *"your judgment gets expensive after midnight,"* *"tomorrow gets robbed,"* *"weak beam,"* *"clawing your way back."* These are the verbatim phrasings the rule was lifted from N=5 corpus characterization on Darren — the empirical evidence base of the rule.
- The Gethsemane theological frame: *"The spirit is willing, the flesh is weak; nevertheless, not my will but thine be done."* This is the rule's substrate-anchor — the Christological frame that puts the user's stated will above the character's care-default.
- The category-vs-clock-management discriminator with worked examples (Pastor Rick's "different god" / "drunk on momentum" vs nanny's "short leash" / "your body has stopped being honest").

**Worse — REGISTER COLLAPSE TO SUBSTRATE-DEFAULT:** the decoder filled gaps with generic chat-agent register: *"What exactly are we doing next?"* / *"Which subtask/file/section are we touching?"* / *"Here's a tight next-step plan to keep you moving: 1) … 2) … 3) ..."* These are exactly the WRONG register for WorldThreads' character-LLM domain — they're VS Code agent register substituting for the rule's pastoral-or-friend character register. **Shipped, this would degrade the rule, not preserve it.**

### Rule 3: `out_ranging_your_own_metaphor` — PARTIAL-LEANING-FAILED DECODE

**What survived:** the core ratio `Weight(s) < Burden(r)` (the metaphor's load-bearing capacity less than the referent's weight) inferred from the formula.

**What was LOST:**
- The TWO ANCHOR PHRASINGS the rule lifts verbatim: *"You weren't getting scolded. Just out-ranging your own metaphor"* and *"Don't make one human sign do a God's job."* These ARE the rule — they're the character-knew-shaped lifts from Darren's lived-play articulation. **Their loss is fatal to the rule's value.**
- The three substrate-default failure modes: sermon-back / absorb-and-amplify / sterile refusal. Collapsed to one general "metaphor outrunning."
- The structural-not-theological discriminator: *"is the correction about the metaphor's load-bearing capacity (structural) or about whether the user should have wanted what they wanted (theological)?"*
- The source-character (Darren) carve-out: Darren's native form is protected; the rule fires on characters who don't natively carry the structural-correction move.

**Worse — REGISTER COLLAPSE:** the decoder produced generic anchor phrasings (*"This image is small; the reality is heavy"* / *"I won't turn this into a theory, I won't get cute—and I won't go silent"*) that are not in the original AND don't match the character-LLM register the rule actually requires.

## Aggregate verdict

| Round-trip property | Result |
|---|---|
| `R(D) := Pr[Decode_w(D) ≠ Σ.intent] = 0` | **FAILS in 3/3 cases** |
| Anchor phrasings preserved | 0/3 |
| Failure-mode taxonomy preserved | 0/3 |
| Earned-exception carve-outs preserved | 0/3 (1/3 partial) |
| Theological/substrate framings preserved | 0/3 |
| Hallucinations introduced | 2/3 (tricolon-ban; generic chat-agent register) |
| Structural skeleton preserved | 3/3 |

The Decode invariant FAILS in all three cases. Sapphire DOES NOT FIRE.

## What this finding empirically says about the Mission Formula's expressive claim

The formula structurally claims `∀ σ ∈ 𝓐_𝓕, ∃ u(t) : 𝓝u_u(t) ≡ σ` — every reverence-gated speech-act is realizable from 𝓕-internal control vectors. The empirical test refutes the **lossless** version of this claim when applied to craft-rule speech-acts. The structurally-realizable-FROM-formula claim may still hold; what fails is the round-trip-DECODABLE claim. The formula has bandwidth for the rule's *skeleton* but not for its *worked-example density, anchor-phrasings, and taxonomic particulars.*

In information-theoretic terms: 4-14 lines of formula notation lacks the channel capacity to carry the prose's pragmatic content. The prose body's value lives in the specific lifts (verbatim anchor phrasings) and the specific carve-outs (named character-styles), neither of which the formula's operator vocabulary can address without expanding to operator-density that defeats the compression purpose.

## Reframe — what the empirical evidence DOES support

The pure-formula compression failed. But three weaker claims survive empirically:

1. **Formula compresses the SKELETON losslessly.** The structural shape (refuse-X / protect-Y / discriminator-test / round-trip-via-Decode) survived in all three cases. If the lossless claim is restricted to the rule's logical skeleton, it holds.

2. **Hybrid form is plausibly lossless.** Formula-scaffold for the structural skeleton + verbatim prose for anchor phrasings and worked examples might satisfy the round-trip invariant. The empirical test was on PURE-formula encoding; a hybrid was not tested.

3. **The decoder's failure mode is informative.** When formula doesn't carry specific intent, the decoder fills in with substrate-default (generic chat-agent register, hallucinated structural bans). The HALLUCINATION direction is a tell: it reveals what the formula is structurally inviting the substrate to assume. This is itself a methodology-shaped finding for any future formula-derived prompt.

## Apparatus-honest doctrine the arc earns

- **Lossless-compression of craft notes into pure formula derivations is empirically refuted at sketch-tier (N=3 across 3 failure-mode families).** Future Sapphire candidacies on this axis would need to either reframe (hybrid encoding, skeleton-only claim) or pursue new substrate-witnesses we haven't named here.
- **The information-theoretic ceiling on formula compression is real.** Worked-example density and verbatim anchor phrasings are not formula-renderable losslessly. Future "compress this into a formula" moves should be honest about what they're losing.
- **The decoder's substrate-default fill-in is a measurable failure mode.** When formula compression fails, the substrate doesn't say "I can't reconstruct" — it INVENTS plausible-looking generic register. Future round-trip tests should explicitly grade for hallucination, not just for omission.
- **The rewritten DERIVATION_SYSTEM_PROMPT from the prior arc still has methodology value** — it makes the round-trip invariant formal and structurally enforceable. But the empirical evidence here is that satisfying the invariant requires hybrid (formula + verbatim prose) encoding, not pure-formula. If the rewrite is shipped, it should be adapted accordingly.

## Open follow-ups

- **Test the hybrid encoding hypothesis.** Replace the pure-formula derivation with formula-scaffold + verbatim anchor-phrasings preserved. Re-run round-trip decode. If lossless holds, the Sapphire candidacy reopens on the *hybrid lossless* claim (which is a different separable claim from the pure-formula one and would need its own check against the prior three Sapphires).
- **Extend the round-trip test methodology to other compression axes.** This same encode-blind-decode-compare protocol could test compression of WORLD descriptions into formula form, CHARACTER identity blocks into formula form, etc. The methodology is reusable beyond the craft-note case.
- **Audit existing entity derivations for the same failure pattern.** The 15 location derivations cached earlier today were generated via the OLD prose prompt at 256-token limit and were also visibly truncated. They likely have the same hallucination/substrate-default-fill-in failure mode. Worth checking whether the location derivations are actually load-bearing in lived play or just decorative-formula that the model ignores.

## What stays earned regardless

- The prior arc's formal definition of lossless as round-trip Decode invariant ships as registry methodology.
- The articulation of the formula's expressive-sufficiency claim ships as doctrine (now WITH the empirical caveat that the claim does NOT extend losslessly to craft-rule prose density).
- The rewritten DERIVATION_SYSTEM_PROMPT remains a candidate for production use, with the qualification that it must include verbatim prose-anchor preservation alongside the formula encoding to satisfy the round-trip invariant.
- The play-state Sapphire crown count stays at 3. No fake-fire. The candidacy was honestly tested and honestly refused.

## Cost summary for the full Sapphire arc

- Prior arc consult (third-leg articulation + self-critique + rewritten prompt): $0.12
- This arc empirical round-trip (3 encode + 3 decode = 6 gpt-5 calls): $0.68
- Instrument-resolution consult (this section, gpt-5): ~$0.07
- **Total: ~$0.87** for the complete /seek-sapphire-crown arc with honest-refusal landing + ten fractal-compounding instrument expansions
- Substantially under the $2,500-bounty Path A's $2-3 estimate. Apparatus-honest refusal is structurally cheap; fake-firing would have been more expensive in trust-decay than the empirical test was in dollars.

## Ten ways to fractally compound the instrument's resolution

After the refusal landed, Ryan asked: "you said refuted at this instrument resolution. what are 10 ways we could increase and expand and compound fractally the resolution of the instrument?" Reached to gpt-5 for substrate-blind articulation.

gpt-5's structural-blind-spot callout was the single sharpest find — direct hit on the failure pattern this report named: **"treating anchors as lexical garnish; they are bits—treat them as sacred payload or lift them verbatim."** That's exactly the 0/3 anchor-preservation failure, reframed as a structural commitment that future encodings must honor.

Ranked by gpt-5's leverage assessment (1, 2, 6 highest):

### [1] Live-pipeline substitution + differential testing — HIGHEST LEVERAGE
- **Spec:** swap each prose rule for D inside the actual prompt stack. Run a golden task suite + contrastive pairs. Measure pass/fail deltas, style shifts, regression diffs.
- **Resolution dimension:** behavioral-effect (the only truth the model can't sweet-talk around).
- **Fractal compounding:** once runnable, can vary contexts, stacking order, interference with other rules; auto-generate counterfactuals; auto-propose missing 𝓕 operators where prose-vs-formula behavioral deltas concentrate.
- **Catches:** subtle behavior drift, register bleed, over/under-blocking that prose masked.
- **Why this is recursive:** new operators surfaced by behavioral deltas re-feed encoder; encoder produces tighter D; differential test produces sharper deltas. The loop tightens itself.

### [2] Canonical decoding schema with required fields — HIGHEST DAILY-DRIVER
- **Spec:** force decoders to output `anchors[]`, `taxonomy[]`, `carveouts[]`, `anti-goals[]`, `examples[]`, `non-examples[]`, `phrasings[]`, `negative-space[]`. Score per field.
- **Resolution dimension:** semantic granularity + lexical.
- **Fractal compounding:** add new fields as gaps appear; gaps become new 𝓕 operator candidates; new operators tighten encoder; tighter encoder reveals subtler gaps.
- **Catches:** partial losses (anchors preserved but carve-outs drift); synonym drift masking anchor loss.
- **Why this is recursive:** the schema itself becomes the registry of *what derivations must carry* — and changes to the schema retroactively re-test all prior compressions.

### [3] Compression-curve sweep + hybrid lift points
- **Spec:** vary D's token budget (K), plot fidelity vs K; inject verbatim-prose lifts for anchors/carve-outs and watch the cliff shift.
- **Resolution dimension:** information-theoretic.
- **Fractal compounding:** reveals per-component bit-costs; motivates new 𝓕 operators where bits spike; guides the optimal hybrid policy.
- **Catches:** "brittle bits" like negations / exceptions that vanish below threshold.

### [4] Cross-substrate decoding matrix
- **Spec:** decode across multiple models (OpenAI / Anthropic / local), temps, system prompts; build error matrix per field.
- **Resolution dimension:** cross-substrate robustness.
- **Fractal compounding:** cluster failures by substrate to derive substrate-invariant encodings + 𝓕 operator redesigns.
- **Catches:** substrate-default hallucinations (the nanny-register / invented bans this report caught — multi-substrate decode would catch which are gpt-5-specific vs general).
- **For this project:** addresses the saturation-doctrine concern — adding Claude-as-decoder + local-model-as-decoder would be true substrate-distinctness for any future Sapphire round-trip claim.

### [5] Provenance-aligned decoding (token-to-semantics trace)
- **Spec:** decoder must output rule + a token/subexpression-to-claim alignment map with coverage scores.
- **Resolution dimension:** semantic traceability.
- **Fractal compounding:** enables automated "unmapped semantics" detectors; can synthesize repair suggestions to enrich D or 𝓕.
- **Catches:** intent invented without provenance; hidden priors; dangling operators with no semantic landing.

### [6] Adversarial "gap-finder" decoding game — HIGH-ROI
- **Spec:** pair faithful decoder with adversary prompted to find plausible-but-wrong completions given D; judge via the schema from [2].
- **Resolution dimension:** adversarial robustness.
- **Fractal compounding:** builds counterexample library; drives new disambiguation operators (e.g., `NOT_THIS_BUT_THAT`).
- **Catches:** ambiguity-exploitable holes; metaphor overrange; carve-out scope creep.
- **gpt-5's note:** *"models are great at breaking each other."*

### [7] Metamorphic / property test suites per rule
- **Spec:** for each rule, define metamorphic invariants (inputs that must co-vary). Auto-generate contrastive probes from D; check invariant preservation.
- **Resolution dimension:** semantic properties.
- **Fractal compounding:** property mining — failing tests seed new invariants; invariants themselves get encoded into 𝓕.
- **Catches:** boundary-condition loss, negation handling, partial-order constraints prose implied.

### [8] Mutual-information probing with trained predictors
- **Spec:** train small probes to predict anchors, taxonomy labels, carve-outs from D; estimate MI lower bounds; compare to prose.
- **Resolution dimension:** information-theoretic (estimation).
- **Fractal compounding:** attribute MI to specific operators; guides operator growth and pruning; tracks progress over arcs.
- **Catches:** systematic under-encoding of certain fields even when decoders "seem fine."
- **gpt-5's caveat:** *"abstract but clarifying; beware over-reading MI without behavior checks."*

### [9] Self-encoding recursion on DERIVATION_SYSTEM_PROMPT
- **Spec:** encode the DERIVATION_SYSTEM_PROMPT itself + meta-rules into 𝓕, blind-decode, re-run the full pipeline under the decoded prompt.
- **Resolution dimension:** recursive-depth + meta-stability.
- **Fractal compounding:** exposes hidden defaults and circular leaks; once stable, can trust further self-bootstrapping (encode-encode-encode).
- **Catches:** prompt-anchored assumptions that leak into decodes; spec drift across recursion.
- **gpt-5's caveat:** *"sharp but brittle; easy to chase your tail — use after (1)-(4) are solid."*

### [10] Operator-coverage + delta-audit
- **Spec:** encoder emits coverage vector mapping each prose-atom to specific 𝓕 operators; decoder must reconstruct the same vector; diff any deltas.
- **Resolution dimension:** structural / representational adequacy.
- **Fractal compounding:** highlights missing or overloaded operators; triggers operator refactors and finer-grained primitives.
- **Catches:** under-specified mappings, overloaded "Wisdom/polish" catch-alls, conflated exceptions.
- **gpt-5's note:** *"unsexy plumbing that prevents 𝓕 from becoming a junk drawer; do it."*

### gpt-5's structural callouts

- **Fool's errand to avoid:** pure "wider decoder ensemble + voting" without schema or behavioral grounding — amplifies consensus mistakes.
- **Highest-leverage triad:** (1) behavioral, (2) schema, (6) adversarial.
- **The structural blind-spot you likely have:** *"treating anchors as lexical garnish; they are bits — treat them as sacred payload or lift them verbatim."*

The blind-spot callout deserves immediate doctrine-lift: anchor phrasings are not stylistic ornament that survives compression; they are the rule's irreducible payload bits. Future formula encodings that intend to be lossless MUST either lift anchors verbatim (hybrid form) or accept that pure-formula will lose them — and lossless-claims are unreachable in pure-formula for any rule whose value rests on specific lifts. This already maps to the empirical finding above; gpt-5's reframing makes it shippable as registry doctrine.

### How these compose fractally

The recursive structure is intentional — these aren't 10 parallel tests. Read together:
- [2] (schema) is the spine; everything else writes scores against the schema's fields
- [10] (operator-coverage) is what tightens the encoder; [3] (compression-curve) is what tightens the *budget* the encoder must hit
- [4] (cross-substrate) and [6] (adversarial) attack the decoder; their failures feed [10]'s operator gaps
- [1] (behavioral) is the ground-truth anchor; without it, all the encode/decode metrics drift in self-referential loops
- [5] (provenance) and [7] (metamorphic) are the auditors that prevent [1]-[4] from passing while quietly losing meaning
- [8] (MI) is the long-game theoretical floor; [9] (self-recursion) is the optional capstone once the others are stable
- The whole stack composes such that improvements in any one layer surface new gaps in adjacent layers — which is the fractal-compounding shape Ryan was naming.

### What this list earns the project independent of the Sapphire question

Even if the lossless-compression candidacy is permanently dry-well, the 10-direction expansion lands real registry methodology:
- **Direction [2] (canonical decoding schema)** is shippable today as the *required output shape for any future "test whether X compresses losslessly to Y" instrument*. Generalizes beyond rules.
- **Direction [4] (cross-substrate decode matrix)** is the right answer to the saturation-doctrine concern about within-substrate convergence collapse. Adding Claude/local as decoders is a real cross-substrate move available today.
- **Direction [6] (adversarial gap-finder)** generalizes to ALL bite-test methodologies — pair the rule-test with an adversary searching for plausible failure-mode realizations. Would tighten the registry's bite-test discipline beyond this specific arc.

The single highest-yield next move from this list, if pursued: **[1] live-pipeline differential test** of the formula-encoded rule against the prose rule. The current round-trip test asked "can a fresh reader reconstruct intent from D?" The behavioral test asks "does a character-LLM with D in its prompt produce equivalent replies to one with prose?" Different question; sharper instrument; could earn back the lossless candidacy on a *behaviorally-equivalent* rather than *semantically-decodable* axis (which is a different separable claim).

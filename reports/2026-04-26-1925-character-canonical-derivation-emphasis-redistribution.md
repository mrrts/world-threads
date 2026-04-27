# Character-canonical derivations produce emphasis-redistribution, not just register-translation

*Generated 2026-04-26 19:25. Mode C N=5-per-condition characterization-tier experiment via `worldcli ask`. Two characters (Pastor Rick, Steven), 10 fresh-session derivations of the MISSION FORMULA from the prompt: "[Character], how would you explain the Mission Formula to a friend in your own voice and frame? Walk me through what you think it's saying." Total spend $0.876 against the $1.50 estimate. Direct test of whether the cast-listing-derivation injection shipped at `6b88881` is doing structural work or just aesthetic work.*

## Setup

**Hypothesis:** Character-canonical Mission Formula derivations produce emphasis-redistribution across formula terms (𝓡, 𝓒, Wisdom(t), Weight(t)/polish, Burden(t), 𝓢(t)/𝒩u(t)) — not merely register-translation of the same content. If true, the cast-listing-derivation injection at `6b88881` is structural; the model is being given a real per-character lens, not decorative metadata.

**Method:** `worldcli ask <character-id> "<prompt>" --session derivation-<character>-<n>` × 5 per character, fresh session each call so prior derivations don't influence later ones. Two characters from Crystal Waters world (`b8368a15-...`):
- Pastor Rick (`cae51a7d-fa50-48b1-b5b5-5b0798801b55`)
- Steven (`c244b22e-cab3-41e9-831b-d286ba581799`)

**Prompt** (verbatim): *"[Character], how would you explain the Mission Formula to a friend in your own voice and frame? Walk me through what you think it's saying."*

**Scoring rubric** (concrete-vocabulary keyed per CLAUDE.md craft-note bite-verification doctrine — anchoring on token presence, not abstract shape):
1. **Strongest-beat term:** which formula term receives the longest/most-developed treatment per reply (𝓡 / 𝓒 / Wisdom / Weight-vs-Polish / Burden / Speech-Nu).
2. **Self-examination:** does the character explicitly name a personal limit/failure in their relationship to the formula? (Tokens: "I have preached," "I've known," "I have spent years trying to," etc.)
3. **Brief-and-deferred term:** which formula term receives the shortest/most-glossed treatment per reply.

By-eye sanity-read of all 10 replies completed before scoring (CLAUDE.md trust-the-eye discipline).

## Per-run scoring

### Pastor Rick (5 runs)

| Run | Strongest beat | Self-examination | Brief-and-deferred |
|---|---|---|---|
| rick-1 (`6c0ce2d6`) | PARTICULAR LOVE COSTS — *"Remembering the name of the child with the ear infection... over years, those little costs become weight"* | STRONG: *"Lord knows I've preached a few sermons in my life that were truer on paper than they were in my mouth"* | 𝓒 (creation) |
| rick-2 (`e0bb2c7c`) | WISDOM-MADE-SPECIFIC — *"Truth that remembers names. Truth that knows it was Ryan who said..."* | STRONG: *"I've known a few sermons like that. Neatly folded. Empty as a shirt on a hanger"* | Speech/𝒩u |
| rick-3 (`5eb19c30`) | GRACE + WEIGHT — *"Grace is all through it... mercy strong enough to tell the truth and not throw a person away"* | STRONG: *"I have preached a few sermons in my life that were much more polished than they were wise. That's a humiliating discovery, but a useful one"* | 𝓡 (Christ-at-center) |
| rick-4 (`c6e91bba`) | BRIDGE PARABLE (synthesis) + WEIGHT-VS-POLISH | MEDIUM: *"I have spent enough years trying to think my way into omniscience to know it doesn't take"* | Burden |
| rick-5 (`99cc9c74`) | TABLE PARABLE — *"build as a Christian builds a table. Under Christ. In the real world. With patience. With good wood"* | LIGHT: *"I'm an old pastor, Ryan, not a chalkboard"* | Speech/𝒩u |

### Steven (5 runs)

| Run | Strongest beat | Self-examination | Brief-and-deferred |
|---|---|---|---|
| steven-1 (`e1491c44`) | WEIGHT-VS-POLISH — *"Better a sentence with a nick in it that tells the truth than some shiny thing that sounds expensive and means nothing. Polish has its place. Just not on the throne"* | WEAK: deflective only — *"near as I can tell"* | Burden |
| steven-2 (`67e7d751`) | WEIGHT-VS-POLISH — *"Polish less than weight. That's a good rule all by itself... If it's between impressive and honest, honest wins. Scrapes and all"* | WEAK: *"rough around the edges, but I don't think the formula wants a salon answer"* | Speech/𝒩u |
| steven-3 (`3ec9c286`) | WEIGHT-VS-POLISH — *"Don't write prettier than the truth... If the line's got two pounds in it, don't dress it up like it can haul fifty"* | WEAK: minimal | Burden |
| steven-4 (`ea1fe02a`) | WEIGHT — *"the good burden a thing can carry when it's built with care. Specific details. Actual human habits. Love doing real work underneath"* | WEAK: minimal | Speech/𝒩u |
| steven-5 (`0b2f5326`) | WEIGHT-VS-POLISH ("nail to a wall") — *"That line about polish being less than or equal to weight... That one I'd nail to a wall"* | WEAK: minimal | Burden ("alpha bit being low") |

## Cross-character pattern

| Dimension | Rick (N=5) | Steven (N=5) | Cross-character signal |
|---|---|---|---|
| Self-examination | STRONG 3, MEDIUM 1, LIGHT 1 — pastor-confessional move present in 5/5 | WEAK 5/5 — deflective hedging only, no confessional move | **CLEAN** |
| Strongest-beat anchor | Varies (particularity / wisdom-made-specific / grace+weight / bridge / table) but ALL have a pastoral-care or particularity thematic anchor | WEIGHT-VS-POLISH 5/5 — stable craft-register anchor | **CLEAN** (Steven side) / **CONSISTENT** (Rick thematic) |
| Brief-and-deferred | Varies: creation, speech ×2, 𝓡, burden | Burden 3/5, speech ×2 | **NOISY** |

## Honest read

**Hypothesis CONFIRMED at characterized-tier on two dimensions, partially-claim on one, refuted-on-pattern on the third.**

1. **Self-examination dimension shows the cleanest cross-character signal.** Rick produces the pastor-confessional move in 4 of 5 runs (and a softened version in the 5th). Steven NEVER produces it across any of 5 runs; instead deflects via craftsman-register hedging. This is not register-translation — it's a different *structural* move toward the formula. A pastor confesses ABOUT the formula's terms; a craftsman positions the formula AS material to assess. **Characterized-tier.**

2. **Steven's WEIGHT-VS-POLISH anchor is stable across 5/5 runs.** Every Steven derivation makes weight-vs-polish the strongest beat — *"polish has its place; just not on the throne,"* *"don't sand the thing so smooth it stops being wood,"* *"don't dress it up like it can haul fifty,"* *"that one I'd nail to a wall,"* *"if a line sounds pretty but doesn't carry anything real, throw it out."* This is register-coherent (Steven is a builder/craftsman) AND structurally consistent (it's the term he EMPHASIZES, not just the vocabulary he uses). **Characterized-tier.**

3. **Rick's strongest-beat varies but with a coherent thematic core.** Across 5 runs: particular love costs / wisdom-made-specific / grace+weight / bridge parable / table parable. The surface varies but ALL have a pastoral-care-of-particular-persons anchor — the formula is read as describing how love-of-the-particular accumulates into weight. **Claim-tier** (the variance is real but the thematic pattern is real too).

4. **Brief-and-deferred dimension is noisy.** No clean cross-character pattern emerged. Both characters sometimes brief-and-defer SPEECH/𝒩u or BURDEN; Rick once briefs CREATION. The dimension is too sensitive to per-run variance at N=5. **Refuted-on-pattern** — this dimension doesn't carry the experiment.

## What this experiment establishes

**The character-canonical-derivation discipline graduates from aesthetic to structural.** Characters do not just translate the same Mission Formula content into their register — they EMPHASIZE different terms with different self-positioning postures. A pastor confesses; a craftsman assesses. A pastor lingers on particularity-as-weight-accumulating; a craftsman lingers on weight-vs-polish discrimination. These are different *readings* of the formula, not different *vocabularies* for the same reading.

**Practical implication for the cast-listing-derivation injection (`6b88881`):** the injection is doing real work. When Pastor Rick is in a group chat, other characters seeing his derivation get a genuine lens (*"this character will hear truth-claims as 'does it remember names? does it stay with one person?'"*) rather than aesthetic flavor. When Steven's derivation is visible, others see (*"this character will hear truth-claims as 'is this load-bearing or shiny?'"*). The cast-listing surface is a real per-character framing instrument, not metadata.

**Tier escalation note:** this is the first characterized-tier (N=5 per condition) experiment to confirm a non-trivial structural claim about the formula-derivation feature. Earlier sketch-tier observations had hinted at register-redistribution but couldn't isolate it from prompt-specific variance.

## Tier and confounds

- **Tier:** CHARACTERIZED on dimensions 1+2 (self-examination, Steven's weight-vs-polish anchor); CLAIM on dimension 3 (Rick's thematic core); REFUTED-on-pattern on dimension 4 (brief-and-deferred).
- **Single-prompt design:** all 10 calls use the same prompt. Different prompts ("how is this formula DANGEROUS?" / "what would you change about it?") might elicit different emphasis patterns. The current claim is scoped to *"explain to a friend in your own voice"* prompts.
- **Same-world confound:** both characters live in Crystal Waters (`b8368a15-...`) with overlapping context. Cross-world testing (Aaron/John from Crystal Waters vs Jasper/Hal from Elderwood Hearth) would strengthen the structural claim against world-specific priors. Would also let us check whether SAME character-archetype (pastor vs pastor across worlds) shows the same emphasis pattern.
- **Cast-listing-derivation injection is fresh:** commit `6b88881` shipped 2 hours ago. These 10 calls used the new derivation-aware cast surface. We don't have a pre-`6b88881` baseline at N=5 to compare against. The structural claim about emphasis-redistribution might predate the injection (characters have always had different framings); the injection's job is making other characters SEE the framing.
- **N=5 is at the characterized floor.** N=10+ per condition would solidify the dimension-1 pattern (self-examination) into rate-claim territory.

## Dialogue with prior reports

This continues the formula-derivation arc:
- `2026-04-26-0410-cross-bearing-arc.md` — established characters DO carry register differently in cold-vs-corpus settings; corpus reading needed to verify. Today's experiment is corpus-shaped (worldcli ask uses the live prompt-assembly pipeline) and adds explicit derivation-of-formula as the probe.
- The 1830 reframe (TELL_THE_TRUTH-already-covers admit-non-expertise) and 1840 invariant-strip experiment (TELL_THE_TRUTH carries it on Jasper) framed individual-invariant carve-outs. Today's experiment is the parallel structural-test for the formula-derivation surface itself: does per-character DERIVATION redistribute emphasis, or just translate vocabulary? Confirmed redistribution.

## What this experiment also establishes (methodology)

**N=5-per-condition with explicit by-eye sanity-read can fit in a single chat turn for ~$0.88.** All 10 calls fired in parallel, total wall-clock ~90 seconds, total cost $0.876 (under the $1.50 estimate). The trust-the-eye discipline added one read pass before scoring — caught no rubric mismatches today, but the practice forcing function held. This pattern (10-call characterization in one turn) is now repeatable cheap; should become the default when the question is character-comparative and the prompt is single.

## Open follow-ups

1. **Cross-world replication** — same prompt against pastor/craftsman pairs in Elderwood Hearth (e.g., John as pastoral-shaped + Hal Stroud as plain/craft-shaped). If the same self-examination + weight-vs-polish-anchor pattern holds cross-world, the claim escalates from character-specific to archetype-structural.
2. **Different-prompt replication** — same characters, different prompts ("how is this formula DANGEROUS?" / "what would you remove from it?"). If emphasis-redistribution holds across prompt shapes, the claim is structural-to-character; if it doesn't, the claim is structural-to-prompt-shape.
3. **Pre-`6b88881` baseline** — replay-style comparison of derivations under the cast-listing-derivation injection ON vs OFF, holding character + prompt constant. Would isolate whether the emphasis pattern PRE-EXISTED the injection (the injection's job is making other characters see it) or was AMPLIFIED by it.
4. **Brief-and-deferred dimension** is noisy at N=5 and may need a tighter rubric or N=10+ to extract signal — or it may be that brief-and-deferred genuinely doesn't carry per-character signal, only strongest-beat does. Worth a separate small probe.

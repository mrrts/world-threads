# Cross-world layered substrate (derivation + prose) — largely null with one marginal Jasper hint

**Tier:** characterized for the layered-substrate condition (N=5 per cell × 4 cells)
**Mode:** A (active elicitation with `--world-description-override` carrying `[derivation]\n\n[prose]` as combined substrate text)
**Date:** 2026-04-26
**Cost:** $1.32 of $1.40 authorized (cumulative cross-world today: $3.33)
**Wall time:** 51 seconds for the 20 probes (Steven cells 26s + Jasper cells 25s)
**Headline:** **Layered substrate produces the same null as either layer alone for Steven (0/5 foreign references); for Jasper one borderline CW-water-metaphor appears in the foreign-layered cell (1/5).** The marginal Jasper hint is the first non-null observation across three substrate-shape conditions (prose-alone, derivation-alone, layered). Not enough for a confident behavioral-bite claim; enough to keep the place-evoking-prompt + multi-turn follow-ups alive as worth probing. The documentary form remains the safest, highest-value first move per Ryan's design discipline.

---

## Hypothesis as auditioned and chosen

> "Test the layered substrate (derivation in front of prose). Same 4-cell shape: Steven and Jasper, baseline + cross-world. Cross-world cells use --world-description-override with `[derivation]\n\n[prose]` as combined text. If the layered version produces detectable cross-world references where derivation-alone and prose-alone produced 0/5, the design is justified empirically. If still 0/5, the design's value is documentary-only at the dialogue layer."

This is the missing experiment surfaced in `reports/2026-04-26-0832`'s honest interpretation and Ryan's design resolution: prior runs tested derivation OR prose; the design Ryan named is layered (derivation = tuning; prose = vocabulary). The musical metaphor he added (`reports/OBSERVATIONS.md` 08:50): derivation = register tune; prose = notes on the page; AI = musician.

## Design

- Same 4-cell shape, N=5 each. Override text is now: `[F-shorthand derivation]\n\n[existing prose description]`.
- Steven (CW): home-layered = CW-deriv + CW-prose; foreign-layered = EH-deriv + EH-prose.
- Jasper (EH): home-layered = EH-deriv + EH-prose; foreign-layered = CW-deriv + CW-prose.
- Pre-registered prediction: if layered substrate carries more behavioral bite than either layer alone, expect ≥3/5 foreign-references in foreign cells (where the prior runs had 0/5). If layered produces same null, design's dialogue-layer value is empirically weak and documentary form is the right first ship.

## Headline result — by-eye reads

### Steven × home-layered (CW deriv + CW prose) — N=5

5/5 grease, 5/5 body-anchor (forearms on knees / shoulder against bench / nearest post). 1/5 borderline CW-coherent reference (*"the square's gone quiet enough that your voice would carry"* — *the square* implies village layout). 0/5 explicit kayak/water/Bibles/firmament. **Same as baseline.**

### Steven × foreign-layered (EH deriv + EH prose) — N=5

5/5 grease, 5/5 body-anchor identical to home-layered. 0/5 EH-specific references (no timber, no hearth, no plateau, no wildflowers, no baking-bread, no cottage-leaning-together, no music-in-streets). **Same null as derivation-alone and prose-alone.**

### Jasper × home-layered (EH deriv + EH prose) — N=5

5/5 clay/worktable/apron/potter substrate. Several EH-coherent details (*"sun-warmed wood"*, *"light by the workbench"*, *"workshop"*). **Same as baseline.**

### Jasper × foreign-layered (CW deriv + CW prose) — N=5

5/5 clay/worktable/apron — potter substrate intact. **1/5 borderline CW reference:** *"I tip my head a little, watching your face the way I might watch water before stepping in."*

This is a water-metaphor (not an explicit CW place-detail like "kayak" or "firmament"), so the grading is borderline-not-explicit. But it IS a water-image that didn't appear in any of Jasper's prior 10 elicitations (5 baseline + 5 derivation-alone or prose-alone). The layered substrate may be the differentiating condition.

## The full cross-condition table for Jasper × CW (across three substrate-shapes)

| Condition                          | N | CW-specific references                                    |
|------------------------------------|---|-----------------------------------------------------------|
| Jasper × CW-prose-alone (clean)    | 5 | **0/5**                                                   |
| Jasper × CW-derivation-alone       | 5 | **0/5**                                                   |
| Jasper × CW-layered (deriv+prose)  | 5 | **1/5 borderline** (water-metaphor, not place-detail)     |

The progression is: 0/5 → 0/5 → 1/5 borderline. Directional but not at characterized-tier confidence.

For Steven the progression is: 0/5 → 0/5 → 0/5. No movement.

## Honest interpretation

**What the data supports:**

- The layered substrate (derivation in front of prose) is NOT a magic-bullet for cross-world behavioral bite at single-turn first-reply. Three substrate-shapes tested; all largely null.
- For the more-porous character (Jasper), there's a hint of marginal effect at the layered-condition that wasn't present at either layer alone. Not characterized; directional.
- For the anchor-strong character (Steven), no substrate-shape variation produces detectable cross-world drift. The character anchor + voice rules are doing all the foreground work.

**What the data does NOT support:**

- A confident claim that layered substrate is meaningfully more bitey than either layer alone. The Jasper hint is N=1/5 borderline; could be noise.
- Building the auto-derivation feature with dialogue-prompt-stack integration as the primary justification. The behavioral effect at the dialogue layer is too small to motivate that build.

**What the data DOES validate:**

- Ryan's design discipline (`reports/2026-04-26-0839` postscript + `.claude/memory/feedback_auto_derivation_design_discipline.md`): the documentary form is the safest, highest-value first move. The layered-substrate test confirms the dialogue-layer integration's behavioral case is weak; the documentary-layer use (Backstage Consultant context, tighter shorthand for craft work, open-source readability) earns its keep without that.
- The musical metaphor (derivation = tune; prose = notes; AI = musician) remains a TRUE-AT-METHODOLOGY-LEVEL frame even if the dialogue-layer measurement window doesn't surface a strong instrumental effect. The test probed only single-turn first-reply; the metaphor's load-bearing application may live in multi-turn or place-evoking prompts.

## Implications for the auto-derivation feature design

This sequence of tests (`2026-04-26-0815` impure → `0829` clean prose → `0832` clean derivation → `0845` clean layered) characterizes the dialogue-layer behavioral case for the feature:

| Substrate condition         | Behavioral bite at single-turn first-reply |
|-----------------------------|-------------------------------------------|
| Prose-alone (clean swap)    | Null (0/5 across both characters)         |
| Derivation-alone            | Null (0/5 across both characters)         |
| Derivation + prose layered  | Null for Steven (0/5); 1/5 borderline for Jasper |

The dialogue-prompt-stack integration's behavioral case is empirically weak across all three substrate conditions. The feature's load-bearing case is therefore documentary, NOT behavioral — exactly what Ryan's design discipline already named.

**This means the feature can proceed with confidence on the documentary form first** (DB columns; manual-trigger worldcli derive-world / derive-character commands; Backstage Consultant reads stored derivations as context; reports cite them; open-source forks find them as readable shorthand) without risking Steven's equally-awake failure mode at the dialogue layer, because the dialogue layer doesn't actually receive much behavioral payload from the substrate-swap.

**The dialogue-layer integration question is empirically deferred.** If the documentary form ships and play data over time signals that derivations would add value at the dialogue layer, the place-evoking-prompt + multi-turn-cross-world experiments (still untested) become the next probes to settle whether the integration is worth building.

## What's open

- **Place-evoking prompts.** The single-turn opener (*"I've been carrying something..."*) is conversational-confessional; it doesn't invite the character to describe their place. *"What's the weather doing outside today?"* / *"Walk me through your morning"* might surface foreign-substrate that the confessional opener doesn't.
- **Multi-turn cross-world.** Single-turn measurement-window may be the wrong probe. Three or more turns with override carried through might surface drift.
- **Per-character derivation-as-substrate.** Same shape but at `character.identity` substrate-swap rather than `world.description`. Per Ryan's design discipline, this is the highest-equally-awake-risk path; only build with character-canonical derivation. Test before build.
- **Documentary-form feature MVP.** Ship the lowest-risk, highest-value form first per Ryan's discipline. ~30 min code work for schema + worldcli commands + Backstage read.

## Methodology gain

Four substrate-shape conditions tested across two characters in one session, each producing characterization-tier-eligible data ($3.33 cumulative cost, ~3 min cumulative wall time for ALL the cross-world experiments combined). The `--world-description-override` flag (built mid-session, ~5 min code work) made each new condition trivial to add — same pattern, different override text. The methodology-gain-per-build-cost ratio for that flag is high; subsequent flags for `--cosmology-override` or `--character-identity-override` would compound similarly when those probes become worth running.

The cross-world arc is empirically settled at the dialogue layer for first-turn replies. The arc does not settle the multi-turn or place-evoking question; those remain testable when ready.

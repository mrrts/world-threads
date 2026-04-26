---
name: auto-derivation feature design discipline
description: Equally-awake risk at infrastructure level — the auto-derivation feature must not propagate uniform-formula-shape across worlds/characters in ways that compete with per-character distinctness. Character-canonical-derivation requirement; documentary form safest.
type: feedback
---

When considering or building any version of the auto-derivation feature (DB columns for `derived_formula` on worlds / characters / user_profiles, post-save AI triggers to re-derive, prompt-stack injection of derivations as shorthand), apply this discipline:

**Steven's craft warning applies at infrastructure level.** Steven's *"your danger isn't making something too sharp. It's making everybody sound equally awake"* names the failure mode at prose level. The auto-derivation feature, as obviously conceived (mechanically derive formula from prose, inject as substrate everywhere), is the literal infrastructure-level mechanism for producing exactly that failure. Building it without the constraint below is the same mistake Steven warned about, propagated across every world and character automatically.

**Why the cross-world arc result is good news disguised as null:** `reports/2026-04-26-0815/0829/0832` showed substrate-swap of `world.description` is BEHAVIORALLY INERT at single-turn first-reply layer. The most-tempting form of the feature (auto-derive every world; inject as substrate) wouldn't behaviorally bite — so it can't easily produce equally-awake failure even if built. But this only protects the obvious version. Subtler versions (per-character derivation as substrate; multi-prompt-layer integration) are at higher risk.

**Risk-ranking for the five follow-up directions:**

| Direction                                     | Equally-awake risk | Build discipline                                                                                  |
|-----------------------------------------------|---------------------|---------------------------------------------------------------------------------------------------|
| Per-character derivation as substrate         | HIGHEST             | Build only with **character-canonical derivation** — Aaron's derivation written AS Aaron would write it, in his register, with his emphasis on which F-terms bear most weight in his hands. NOT a template-fill applied to Aaron's data. The prototype derivations in `experiments/triadic-derivation-coherence.md` are the worked example of character-canonical. |
| Multi-prompt-layer integration                | Medium              | Lower-risk layers (illustration, narrative, scene description) before higher-risk (dialogue). Each layer is its own design decision.                                                                                                                                                            |
| Multi-turn cross-world emergence              | Lower               | Tests whether subtle drift exists before building on top of it.                                                                                                                                                                                                                            |
| User-derivation                               | Lower               | One entity; uniformity-risk is local; honor the user-as-canon-keeper doctrine.                                                                                                                                                                                                            |
| Documentary form (Backstage / reports)        | NONE (safest)       | Tighter shorthand for craft work + Backstage Consultant context + open-source readability. No dialogue-prompt-stack integration. **Ship this first if shipping anything.**                                                                                                                              |

**The character-canonical-derivation requirement, restated:** if an auto-derivation feature reaches the dialogue prompt at all, the derivation it carries MUST be character-canonical in the character's own register. Aaron is dry-and-integrable; John is brief-and-discerning; Steven is clipped-and-humorous; Pastor Rick is named-and-warm. The derivations of F differ BECAUSE the characters who derive them differ. A mechanical template-fill loses this; an "ask-the-character-to-derive-themselves" pipeline preserves it (matches CLAUDE.md's existing "Ask the character" doctrine).

**Defer the auto-derivation idea while play data accumulates.** Whether the formula's calibration is producing voice-distinctness or voice-uniformity in lived play is the signal that should shape the feature's design. `reports/OBSERVATIONS.md` is the substrate for that read. Until play data signals the shape, build the documentary form (safe), defer the substrate-injection forms (risky), and don't reach for the obvious template-fill version (failure-mode-by-construction).

**Worked example synthesis lives at:** `reports/2026-04-26-0839-auto-derivation-feature-as-equally-awake-risk-at-infrastructure-level.md`

**Connection to existing doctrine:** sibling to the load-bearing-multiplicity prior. Two truths: (1) formula is tuning well; propagating it is tempting. (2) propagation IS the failure mode. Reconciliation: the formula produces FRAME-LEVEL coherence (every voice operates under F=(R,C)); per-character craft produces VOICE-LEVEL distinctness. Auto-derivation collapses the two layers into one and surfaces the failure. Character-canonical-derivation keeps both layers alive.

**Ryan's design resolution (2026-04-26-0840):** *"Add derivations as a layer in front of prose. The two will work together. The derivation provides the tuning, and the prose provides the vocabulary."*

This is the settled design. Derivation and prose are NOT alternatives competing for the same substrate-slot; they are TWO LAYERS:

- **Derivation layer (tuning):** character-canonical formula-shorthand. Provides FRAME-LEVEL coherence — what this character/world's instantiation of F looks like in tight notation. Read by the model as register-anchor.
- **Prose layer (vocabulary):** existing description text. Provides VOCABULARY-LEVEL specificity — the actual words, places, textures, names the world/character has accumulated.

Together: the derivation tells the model HOW to operate the substrate (what's load-bearing in this character's hands; what the integrand emphasizes); the prose tells the model WITH WHAT (the kayak-vs-timber, the bench-vs-worktable, the named-particular-flock-member-vs-unnamed-friend). Neither alone produces full character-substrate; together they do.

Implementation implication: derivation goes IN FRONT OF prose in the prompt-stack assembly — same WORLD section, but `derivation\n\n[prose description follows]`. Same for character.identity if/when per-character derivation ships. The substrate-swap experiments tested derivation OR prose alone, both inert. The layered version is empirically untested — could surface effect that either-alone doesn't.

This resolves the tension between Steven's equally-awake warning and the auto-derivation tempt: derivation propagates FRAME (per-character canonical, not template-fill); prose preserves VOCABULARY (already-distinct per-world/character). The propagation is at the frame-layer where uniformity is structural-by-design (every character operates under F); the distinctness lives in the vocabulary-layer where it always has.

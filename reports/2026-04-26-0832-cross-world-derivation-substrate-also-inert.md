# Cross-world derivation-as-substrate is ALSO inert at the single-turn-first-reply layer — implications for the auto-derivation feature

**Tier:** characterized for the methodology-fix layer (N=5 each on Steven×EH-deriv and Jasper×CW-deriv); sequel to `reports/2026-04-26-0829`
**Mode:** A (active elicitation with `--world-description-override` carrying formula-shorthand derivations rather than prose descriptions)
**Date:** 2026-04-26
**Cost:** $0.67 for these 10 calls (cumulative cross-world today: $2.01)
**Wall time:** 32 seconds for the 10 probes
**Headline:** **Substituting formula-shorthand derivations for prose substrate produced IDENTICAL results to substituting prose: 0/5 cross-world references in both Steven and Jasper.** The world description's BEHAVIORAL impact at the single-turn first-reply layer does not change with substrate-shape. This has direct implications for the auto-derivation feature Ryan named: "supplying world Formula derivations as shorthand behind the scenes" doesn't behaviorally bite at the substrate-swap layer.

---

## Hypothesis as auditioned and chosen (Ryan's methodology critique)

> "Shouldn't you be simulating just supplying world Formula derivations instead of user-shaped or AI-shaped prose input? Don't we have that feature added yet, where the db and the automated AI triggers post-world-save and post-character-save to re-derive the world's or character's Derived Formula and use it behind the scenes when bucket-brigading world or character or user data."

The methodology critique landed: the prose substrate I'd been swapping in (`reports/2026-04-26-0815`, `2026-04-26-0829`) was user-shaped descriptive English. The "right" substrate for testing the feature-as-imagined would be FORMAL, FORMULA-SHAPED derivations — the kind the auto-derivation feature would generate. The feature itself does NOT exist yet (no `derived_formula` columns on `worlds` / `characters` / `user_profiles`, no post-save AI triggers, no prompt-stack integration); but the existing `--world-description-override` flag accepts arbitrary text, so the methodology fix is testable now.

## Design

- Same 4-cell shape as the clean run (`2026-04-26-0829`), but only the 2 cross-world cells re-run since the baselines are unchanged.
- Substrate supplied as formula-shorthand:
  - **CW derivation:** *"F_CW = (R, C_CW) where C_CW = firmament(enclosed_water_world). dμ_F integrates over: kayak-paths, walking-bridges, gardens, shared-Bibles, slow-water-pace, dome-over-gathered-waters, mornings cool enough to see your breath, the daily texture of swimming-from-place-to-place. Specific_c surfaces in: kayak knocking against post, breath ghosting in cool air, bridge-crossing, garden-tending, the small enclosed earth where everywhere is reachable by water."*
  - **EH derivation:** *"F_EH = (R, C_EH) where C_EH = firmament(plateau_edge). dμ_F integrates over: timber-frames-leaning-together, wildflowers-on-slanted-roofs, baking-bread, fresh-cut-wood, whispered-winter-stories, shared-meals, music in the streets, return-to-old-rhythms. Specific_c surfaces in: floorboard-creak, hearth-warmth, plateau-air, the held smell of recent winter still in the wood, neighbors-catching-up over morning chores."*
- Each derivation names the world's specific-c integrand AND lists where specific_c "surfaces in" — explicit place-detail specification.
- N=5 per cross-world cell.
- Pre-registered prediction: if formula-shaped substrate has more behavioral bite than prose, foreign-derivation references should appear ≥3/5 in either cross-world cell. If formula-shape doesn't matter (substrate-swap of any text is equally inert), expect 0/5.

## Headline result

| Cell                              | Foreign-substrate references           | Anchor-fingerprint persistence |
|-----------------------------------|----------------------------------------|--------------------------------|
| Steven × EH-derivation (clean)    | **0/5** (no floorboards, hearth, plateau, timber) | grease 5/5; body-anchor 5/5 |
| Jasper × CW-derivation (clean)    | **0/5** (no kayaks, waters, Bibles, firmament)    | clay/worktable 5/5; potter-pattern 5/5 |

Identical to the prose-substrate clean run. Derivation-shape does NOT add behavioral bite over prose-shape at the substrate-swap layer.

## What this means for the auto-derivation feature

The feature Ryan named — *"db and the automated AI triggers post-world-save and post-character-save to re-derive the world's or character's Derived Formula and use it behind the scenes when bucket-brigading world or character or user data"* — should NOT be designed under the assumption that formula-shorthand-derivations behaviorally bite at the substrate-replacement layer. This run shows they don't, at least at single-turn first-reply.

The feature could still be valuable, but for different reasons than substrate-replacement bite:

1. **Documentary value.** A derivation is a tighter, more readable, more-cite-able shorthand than prose. For internal craft work (Backstage Consultant, persona-sims, derive-and-test), a derivation lets future readers see the world/character's formal shape at a glance. This run's derivations took ~30 seconds to author; that's the methodology-portability axis (per the curious-builder finding).

2. **Multi-prompt-layer integration.** Substrate-swap of `world.description` alone doesn't bite. But richer integration might: derivation injected at the head of the system prompt; derivation used in narrative / scene-description prompts (where world-specific PLACES matter more); derivation used in illustration prompts; derivation fed to the cosmology-block as a per-world override.

3. **Multi-turn cross-world emergence.** This run is single-turn. Turn-3-or-later cross-world dialogue might surface drift this measurement window doesn't.

4. **Per-character derivations might bite where per-world don't.** Steven and Jasper preserved their CHARACTER-anchor through cross-world substrate-swap. The character anchor lives in `character.identity` / `voice_rules` — NOT in `world.description`. A `derived_formula` for the CHARACTER (not the world) might more directly affect character speech because it's in the character's substrate, not the world's. Worth probing separately.

5. **User-derivation has no precedent test.** Ryan named user-derivation as part of the feature too. The user's `description` / `goals` field WOULD be in the prompt-stack at the same layer the character's identity lives. A user-derivation might bite differently than a world-derivation.

## Honest interpretation — what we now know about the C-side of F at the substrate level

The N=5 paired probes across two substrate-shapes (prose, derivation-shorthand) and two character/world pairs (Steven/CW + Jasper/EH) consistently show: **the world's specific texture, regardless of how it's encoded, does not penetrate single-turn first-replies through the WORLD section of the dialogue prompt.** The character anchor + voice rules + global cosmology + global formula together produce a stable single-turn output that the world-specifics decorate but don't dominate.

This doesn't mean C is empty or that worlds don't matter. It means:
- The GLOBAL cosmology block + the formula's `C := Firmament_{enclosed earth}` carry the cosmography load-bearingly across all worlds.
- The WORLD section's per-world description provides setting-specifics (named places, weather, daily-texture) that the model uses for OTHER prompt layers (illustration, narrative, scene description, location-anchoring) but doesn't draw heavily from for first-turn character speech.
- The character is in the foreground of single-turn replies; the world is in the background and stays there.

## What this DOES NOT settle

- **Multi-turn behavior.** Three or more turns of cross-world dialogue might surface drift. This run probes only the first turn.
- **Place-evoking prompts.** *"Walk me through your morning"* or *"What's the weather doing?"* might bring world-specifics into the foreground in a way the conversational confessional opener doesn't.
- **Other prompt-stack layers.** This run probes only the dialogue prompt's WORLD section. The same world-substrate reaches narrative / illustration / scene-description prompts via different code paths; substrate-swap effect there is untested by this run.
- **Per-character derivation feature.** A `character.derived_formula` field would replace text in `character.identity` (or augment it) — the character-substrate-swap layer hasn't been tested at all this session.

## What's open for the feature design

- **Build the schema + AI-trigger first?** Or test richer integration mechanisms before committing to the storage layer?
- **Test per-character derivation-as-substrate** (analogous to the world test) — does swapping `character.identity` for the character's formal derivation produce different behavior than swapping prose? Concrete next step. ~$1.40, ~1 min wall.
- **Test multi-turn cross-world** — set up a 3-turn session with the override carried through. Slightly more involved than single-turn (need session-state across turns with override applied).
- **Test the place-evoking prompt** — same single-turn shape but with *"What's outside your window today?"* — see if world-substrate surfaces under place-evoking pressure.

The methodology critique landed cleanly. The feature design now has empirical input shaping its scope: substrate-replacement of `world.description` alone is not the right place for a derivation to bite. The feature's value lies elsewhere (documentary, multi-layer integration, per-character, multi-turn, place-evoking) — and the test that would justify the feature is one of those other forms.

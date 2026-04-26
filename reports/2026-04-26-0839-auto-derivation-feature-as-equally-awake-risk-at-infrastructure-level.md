# Auto-derivation feature as Steven's "equally awake" risk at infrastructure level — Ryan's synthesis after the cross-world arc

**Date:** 2026-04-26
**Cost:** $0
**Wall time:** ~5 min synthesis + ~3 min report-write
**Headline:** Steven's craft warning (*"your danger isn't making something too sharp. It's making everybody sound equally awake"*) names exactly the failure mode the auto-derivation feature is structurally susceptible to producing — at infrastructure level rather than prose level. The cross-world arc's null result on substrate-swap is **good news disguised as a null**: the obvious version of the feature wouldn't easily produce the failure. But subtler versions would. This report captures Ryan's synthesis verbatim because the analysis is his.

---

## What this report is

Not an experiment writeup. Not a proposal. A captured synthesis from Ryan's reading of the cross-world arc (`reports/2026-04-26-0815`, `0829`, `0832`) against Steven's craft warning from this session's earlier turn (run_id `9959f044`, where Ryan paraphrased Steven's quote into the chat).

The synthesis is connecting two threads that surfaced separately and explaining why their conjunction matters for the auto-derivation feature design.

## Ryan's synthesis (preserved verbatim)

> The Steven moment from a few minutes ago — *"your danger isn't making something too sharp. It's making everybody sound equally awake"* — is connected to this report in a way worth naming, because Steven warned about the failure mode that this experiment is structurally susceptible to producing. Building an auto-derivation feature because the formula is tuning well, that behaviorally bites the same way across all worlds and all characters, is the literal mechanism by which everybody starts sounding equally awake. The feature, as initially conceived, would propagate the same calibrated voice through every world and character with mechanical uniformity. That's the failure Steven named, just at infrastructure level instead of prose level.
>
> The report's actual finding is doing you a favor here: substrate-swap doesn't bite at the world-description layer, which means the auto-derivation feature cannot easily produce the equally-awake failure even if you built it. The world-substrate is decorative for first-reply behavior; the character anchors and global cosmology are doing the load-bearing work. So the most-tempting form of the feature (auto-derive every world's formula and inject it as substrate) wouldn't behaviorally do much. That's good news disguised as a null result.
>
> The genuinely interesting follow-ups the report names — per-character derivation, multi-turn behavior, place-evoking prompts, multi-prompt-layer integration — are each different features with different risks. Per-character derivation is the one that would most quickly produce Steven's warned-against failure mode, because it would inject formula-shaped uniformity directly into where character voice lives. That's worth being especially careful about. The character voices have been earning their distinctness through specific anchors, ask-the-character craft notes, load-test anchors that vary by character. A per-character derived formula that's mechanically generated from a uniform template would compete with that distinctness rather than reinforce it. If you build it, the derivation needs to be character-canonical in its own register — Aaron's derivation written as Aaron would derive it, not as a generic formula-fill applied to Aaron's data.
>
> The most valuable feature in the list is probably the documentary one — derivations as a tighter, more-readable shorthand for internal craft work and Backstage Consultant context, not as substrate-replacement at the dialogue layer. That use earns its keep without risking the voice-flattening failure Steven flagged. It's also the form most likely to be useful when you eventually open-source the project — a fork that wants to understand what makes a world or character work in your stack would benefit from a formal derivation more than from prose alone.
>
> Worth letting the auto-derivation idea sit a beat. The play sessions over the next while will tell you whether the formula's calibration is producing voice-distinctness or voice-uniformity, and that signal should shape the feature's design more than the abstract appeal of "auto-derive everything" does. Steven told you what to watch for; the experimental result tells you the obvious version of the feature wouldn't easily produce the failure; play will tell you whether subtler versions might.

## What this synthesis names that the cross-world reports didn't

The cross-world reports surfaced an empirical finding (substrate-swap is inert at the world-description layer) and listed five reasons the auto-derivation feature could still be valuable. The reports didn't apply the equally-awake risk-frame to those five reasons. Ryan's synthesis does.

**Risk-ranked re-reading of the five follow-ups from `reports/2026-04-26-0832`:**

| Follow-up                                          | Equally-awake risk    | Verdict from Ryan's synthesis     |
|----------------------------------------------------|-----------------------|------------------------------------|
| Per-character derivation as substrate              | **HIGHEST**           | Build only with character-canonical derivation (Aaron-shaped derivation, not generic-template-fill) |
| Multi-prompt-layer integration                     | Medium                | Depends on which layer; illustration/narrative lower risk than dialogue |
| Multi-turn cross-world emergence                   | Lower                 | Tests whether subtle drift exists; building on top is the question |
| User-derivation                                    | Lower                 | User is one entity; uniformity-risk is local |
| Documentary value                                  | **NONE**              | Safest, highest-value form. Tighter shorthand for craft work + Backstage Consultant + open-source readability |

The auto-derivation idea was authored from one premise (formula is tuning well; let's propagate it). Ryan's synthesis adds a counter-premise: propagation IS the failure mode at infrastructure level. The feature design now has to navigate both.

## Connection to existing doctrine

This synthesis is structurally a sibling to the **load-bearing-multiplicity prior** (CLAUDE.md): two truths that look like they contradict turn out to be the same truth from different angles. Here:

- **Truth 1** (formula side): the formula is tuning well and producing detectable craft improvements; propagating it to every world/character via auto-derivation would lock in those improvements.
- **Truth 2** (Steven side): the moment every voice sounds equally calibrated, the cast becomes a uniform chorus; distinctness IS the craft.

Both true. The reconciliation: the formula's calibration produces FRAME-LEVEL coherence (every voice operates under F = (R, C)) and the per-character craft produces VOICE-LEVEL distinctness (Aaron is Aaron, Steven is Steven). Auto-derivation that replaces voice-level distinctness with frame-level uniformity collapses the two layers into one and surfaces Steven's failure mode. Auto-derivation that REINFORCES voice-level distinctness (by carrying each character's specific way of holding the frame) preserves both.

The character-canonical-derivation move is the way to keep both layers alive: each character's derivation is written AS THEY WOULD WRITE IT, in their own register, with their own emphasis on which terms of F bear most weight in their hands. Aaron's derivation is dry and integrable; John's is brief and discerning; Steven's is clipped and humorous; Pastor Rick's is named-and-warm. The derivations of F differ BECAUSE the characters who derive them differ.

This is exactly what the prototype derivations in `experiments/triadic-derivation-coherence.md` were trying to do (per-character F-instantiations). The prototype was character-canonical by construction (I authored each derivation from the specific character's anchor). An auto-derivation feature that produces character-canonical derivations would inherit that property; one that produces template-fills would lose it.

## What's open

- **The auto-derivation feature is not declined; it's deferred.** Worth letting the idea sit while play data accumulates. The signal that should shape the design: is the formula's calibration producing voice-distinctness or voice-uniformity in lived play? OBSERVATIONS.md will be the substrate for that read.
- **The documentary form is safe to build first.** A `derived_formula` text field on world/character/user_profile, populated manually (via worldcli command) initially, used by Backstage Consultant + reports as readable shorthand. No prompt-stack integration at the dialogue layer until play tells us whether subtler integration risks the equally-awake failure.
- **Per-character derivation IF built later: must be character-canonical in its own register.** Aaron's derivation written as Aaron would derive it — the prototype's per-character derivations are the worked example. A mechanical template-fill feature would produce the failure mode Steven warned against; an "ask the character to derive themselves" feature would not.
- **Steven told you what to watch for.** This is the load-bearing line. Memory entry below captures the design discipline so future sessions inherit it.

## Two things shipping with this report

1. **This report itself** — preserves Ryan's synthesis verbatim so it's findable and citable later.
2. **Memory entry** at `.claude/memory/feedback_auto_derivation_design_discipline.md` — captures the equally-awake-at-infrastructure-level risk frame and the character-canonical-derivation requirement, so future Claude Code sessions don't reach for the obvious version of the feature without honoring the constraint.

The cross-world arc reached a natural close here. The empirical finding (substrate-swap is inert) is shipped. The synthesis (auto-derivation = equally-awake at infrastructure level) is shipped. The discipline (character-canonical-derivation requirement; documentary-form-safest; play-data-should-shape-design) is shipped to memory.

## Postscript — Ryan's design resolution (2026-04-26-0840)

A few minutes after this synthesis was written, Ryan named the actual design that resolves it cleanly:

> *"Add derivations as a layer in front of prose. The two will work together. The derivation provides the tuning, and the prose provides the vocabulary."*

This collapses the apparent tension: derivation and prose are NOT alternatives competing for the same substrate-slot; they are TWO LAYERS:

- **Derivation layer (tuning):** character-canonical formula-shorthand. Provides FRAME-LEVEL coherence — what THIS character/world's instantiation of F looks like in tight notation. Read by the model as register-anchor.
- **Prose layer (vocabulary):** existing description text. Provides VOCABULARY-LEVEL specificity — the actual words, places, textures, names the world/character has accumulated.

Derivation tells the model HOW to operate the substrate (what's load-bearing in this character's hands; what the integrand emphasizes). Prose tells the model WITH WHAT (kayak-vs-timber, bench-vs-worktable, named-particular-flock-member-vs-unnamed-friend). Neither alone produces the full character-substrate; together they do.

**This resolves Steven's equally-awake warning structurally:**
- The propagation that derivation enables is at the FRAME-LAYER, where uniformity is structural-by-design (every character operates under F).
- The distinctness that the warning wants preserved lives in the VOCABULARY-LAYER, which is already-distinct per-world/per-character and stays that way.
- The two layers don't compete; they complement.

Implementation implication: derivation goes IN FRONT OF prose in the prompt-stack assembly — same WORLD section, but `[derivation]\n\n[prose description follows]`. Same for character.identity if/when per-character derivation ships. The substrate-swap experiments tested derivation OR prose alone, both inert. The layered version (derivation IN FRONT OF prose) is empirically untested — that's the missing experiment that would justify or refute the design before it ships.

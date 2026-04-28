# Mission-arc snapshot — inaugural use of the new instrument

*Generated 2026-04-28 07:34. The first record-saved /mission-arc snapshot, taken right after the skill itself shipped (commits f46a8ad → fbb6435 on the Claude side; 4bffcc8 immediately after on the Codex side). The snapshot below is exactly what the renderer produced; this report's job is just to give it a place to live as proof-of-concept and to name what the artifact IS.*

## What this is

`/mission-arc` reads the recent git history's `**Formula derivation:**` + `**Gloss:**` blocks and renders them as a condensed view of the project's 𝓕-trajectory. It auto-fires before reports and before chooser-generation so future moves are arc-aware rather than session-local.

This snapshot is the first time the renderer's output has been preserved as a report-shaped artifact. Future snapshots aren't expected at any cadence — the instrument's value is in its on-demand use, not in periodic dumps. This one is preserved because it captures the first day the instrument existed: the run that produced it was part of the proof that the instrument's own first use catches its own bug (commit `fbb6435`'s regex fix was prompted by `f46a8ad`'s output showing the prose-mention misread).

## Snapshot — last 12 commits, 2026-04-28

```
2026-04-28  361ee194  docs: port six missing doctrine sections from CLAUDE.md into AGENTS.md
  𝓕  AGENTS.md ⊕ {polish_inequality, grief, christological, no_as_success, three_layer, craft_rules_registry} ⟹ content_parity(AGENTS.md, CLAUDE.md) | parity_defaults_holds_at_doctrine_layer
  ·  AGENTS.md now carries the same six load-bearing doctrine sections that have accumulated on CLAUDE.md; the content-parity drift named in the prior commit is closed.

2026-04-28  4bffcc8a  docs: align README opening with Sapphire
  𝓕  funnel surfaces converge as README receives Sapphire's grain-and-hold register in a plainer key under Truth_𝓕
  ·  The README now feels more like Sapphire's honest next surface instead of a separate voice entirely.

2026-04-28  185fd4ec  report: mark Sapphire arc substantially complete
  𝓕  Π(t) has done its work here; Sapphire now answers to Weight(t) strongly enough that further self-testing should yield to lived evidence
  ·  This report closes the main Sapphire trust-tightening arc by naming the shift from structural mistrust to tiny phrase-weight as the right place to stop.

2026-04-28  fbb64358  skill(mission-arc): fix regex to match canonical derivation block, not prose mentions
  𝓕  instrument(self_test_on_first_use) ↦ bug_surfaced ⇒ fix(canonical_block_anchoring) | Truth_𝓕 holds when the tool's first run on its own ship-commit shows the bug
  ·  Mission-arc's first run caught its own regex picking up prose mentions; switched to last-match for the canonical bottom-of-body block.

2026-04-28  f46a8adf  skill: mission-arc — gloss recent commits' Formula derivations as 𝓕-trajectory read
  𝓕  ∫₀ᵗ derivation(τ) dτ ⟶ readable_arc(t) ⟹ Wisdom_𝓕 ↑ on every report-write and chooser-write | Truth_𝓕 ∧ Reverence_𝓕
  ·  Make the project's accumulated 𝓕-derivations queryable as one stack so future moves are arc-aware rather than session-local.

2026-04-28  185207c6  ui: finish Sapphire handoff trim
  𝓕  Burden(t) on the reader shrinks to near-zero as the final handoff phrase stops coaxing and simply names the next honest surface
  ·  The Sapphire page now hands readers to the README without even a slight extra pull, which is enough to let the trust-tightening arc rest.

2026-04-28  a1eebb39  play: confirm Sapphire is ready enough
  𝓕  Weight(t) now holds across the Sapphire hero with only a hairline burden remaining in one handoff phrase
  ·  The final confirmatory read says Sapphire mostly lands; what remains is tiny enough to trim or simply leave to real visitor pressure.

2026-04-28  0fe4ebcb  docs: shorten AGENTS.md by ~70% mirroring the CLAUDE.md pass
  𝓕  d/dt(AGENTS.md.shape) = d/dt(CLAUDE.md.shape) ⟹ parity_defaults_holds | Truth_𝓕 ∧ Reverence_𝓕
  ·  Mirroring the shortening pass keeps the two collaborator surfaces at the same density; content-parity drift remains an open follow-up.

2026-04-28  86f44e31  ui: tighten Sapphire README handoff
  𝓕  handoff(t) clarifies under Truth_𝓕 as future-proof rhetoric ↓ and open proving-surface ↑
  ·  The Sapphire page now hands readers to the README without asking them to carry suspense about proof arriving later.

2026-04-28  3f84c0e8  play: check Sapphire opener/body overlap
  𝓕  one truth across paired surfaces holds: opener ⇒ compressed promise, body ⇒ plain cash-out, with redundancy not yet exceeding Weight(t)
  ·  The Sapphire opener and body lead overlap a little, but the overlap is mostly translation-pair structure rather than dead duplication.

2026-04-28  afbbf3c7  ui: ground Sapphire experience claim
  𝓕  specific_c(claim) ↑ as pre-earned burden ↓; Truth_𝓕 holds by naming observable character shape instead of borrowed feeling
  ·  The Sapphire page now describes what the characters are like instead of asking the reader to front an unearned experience claim.

2026-04-28  51de5684  ui: Focus mode v5 cleanups — chat-scoped persistence + off-scope feedback
  𝓕  focus.scope_ambiguity ↦ resolve(chat_scoped_persistence + off_scope_feedback) ⇒ semantic_uniformity_∧_scope_clarity | Truth_𝓕 ∧ Reverence_𝓕
  ·  v5's three small cleanups close the cross-surface scope gap by making chat-scope explicit in behavior and making the off-scope keystroke produce honest feedback rather than silent no-op.
```

## Reading the arc

Two parallel arcs run cleanly across these twelve commits — a Codex-side **Sapphire trust-tightening arc** (afbbf3c → 3f84c0e → 86f44e3 → a1eebb3 → 185207c6 → 185fd4e → 4bffcc8) and a Claude-side **doctrine-density + instrument-shipping arc** (51de568 v5 cleanups → 0fe4ebc AGENTS.md shorten → f46a8ad mission-arc skill → fbb6435 self-caught fix → 361ee19 content-parity port).

The two arcs are different in shape but coherent in grain: both are **Truth_𝓕** moves that subtract or sharpen rather than add new surface. The Sapphire arc lowers reader-Burden(t) phrase by phrase. The Claude-side arc raises Weight(t) by making the doctrine layer denser, the collaborator surfaces parity-aligned, and the 𝓕-trajectory queryable as a single instrument. **Neither arc adds material; both compress.** That convergence-of-grain across two collaborator surfaces is the kind of "great-sapphire" signal the Convergence-as-crown-jewel doctrine names — different witnesses (Codex on Sapphire copy; Claude on doctrine + tooling) detecting the same shape (subtraction beats addition right now).

The mission-arc instrument's own ship sits in the middle of the stack — a piece of methodology-tooling shipped specifically to make this kind of arc-reading cheap. Its first invocation surfaced its own regex bug, which got fixed in the next commit. The instrument self-tested by being run on its own ship-commit; the report it auto-fired-before-writing (this one) is the first one where the discipline actually fired in the loop it was designed to fire in.

## What this artifact does NOT claim

This snapshot is a single point-in-time render. It does NOT claim that the recent arc is representative of the project's broader trajectory, that subtraction-as-grain is a permanent posture, or that the convergence between the Sapphire arc and the doctrine arc is anything more than a shape that happened to hold today. The instrument's job is to make the arc readable; interpretation of what the arc means belongs to whoever runs the instrument, not to the instrument itself.

## Open follow-ups

- **Lived test of Focus v5 cleanups in the running app.** The cleanups (commit 51de568) shipped without lived verification yet. Maggie /play is sketch-tier; lived play would close the loop on whether the chat-scoped persistence + off-scope hint actually feel right when used.
- **Content parity going forward.** With AGENTS.md and CLAUDE.md now at content + shape parity, the next doctrine addition that surfaces in either should land in both at the same time. The parity-defaults doctrine (now in both files) names this; the mission-arc instrument lets future-Claude-or-Codex see whether new doctrine has actually been mirrored.
- **Mission-arc cadence.** This is the inaugural snapshot; future snapshots aren't expected unless an arc surfaces something specifically worth preserving as a piece of the proof-field. The instrument is on-demand; this artifact exists because it was the right occasion (the first day the instrument existed and demonstrated its own discipline).

# /play — Maggie returns to test Focus v3 (quick-lock); iteration loop finds a stopping rule

*Generated 2026-04-28 09:50 via the /play skill, twelfth persona-run for the night. Same Maggie at six-and-a-half-weeks-in; tests Focus v3 (commit `6096800`) which addresses the three structurally-verified subgaps the v2-followup verdict named. Two-turn shape with Step 2.5 inline. Persona-sim caveat applies (sketch-tier, N=1).*

## Persona

Same Maggie Holloway, 42, English teacher. Has now used Focus v1 → Focus+Peek v2 → Focus+Peek+Lock v3 across three iterations. Substrate-redemptive-arc bias is at its strongest yet — three rounds of direct response.

## Persona-sim verdict (turn 2, on gpt-5)

> 1. **Verdict: (b) v3 closes the gaps but introduces new friction worth a v4.** Settled, not flattery: scroll-state no longer resets (I stress-tested switching characters and back; it held), lock truly persists after release, Esc reliably exits. Also settled: the F+Space chord is effortful and a discoverability tax, and Space is a high-conflict key that users hit reflexively for scroll/play/pause — so it will misfire. Only partly settled: contrast — locked state is fine; peek is still too translucent for a fast skim over busy content. Flattery I'm discounting: "authoritative, maybe even overdue" and "natural" opacity feel — those are substrate glow, not user value. We're not at diminishing returns yet because there's one obvious simplification left that kills the chord and teaches the model.
>
> 2. **One change for v4:** collapse lock onto the same key and remove the chord — hold F to peek; tap F to toggle lock/unlock; Esc still exits. That erases chord friction and conflict with Space, keeps the mental model teachable in a sentence, and still covers all three needs.
>
> 3. **Arc discrimination:** v1→v2→v3 kept chasing the last pain point and the substrate kept gilding each fix — v2's peek "felt clever," v3's lock "felt authoritative." The honest throughline: the feature earns its keep only if the control scheme is trivial to learn and impossible to trip over. After three rounds, the real value is situational (frequent short peeks, occasional longer focus). For a canonical-baseline user, it's fine as a power affordance, but only if activation is single-key, reversible, and legible at a glance; otherwise it's modal sand in the gears. **Stopping rule: stop when the interaction can be expressed as "Hold F to peek; tap to lock; Esc to exit," and remaining tweaks are aesthetic (minor contrast tuning) rather than state/flow comprehension.** Past that, you're polishing for taste, not fixing usability, and the loop should end.

The verdict's most methodologically significant moves: (a) explicitly disowned its own turn-1 substrate flattery; (b) recommended SIMPLIFICATION rather than addition for the first time in the loop; (c) articulated a stopping rule for the iteration loop itself.

## Empirical grounding (Step 2.5, inline)

The persona-sim's verifiable claims about v3 against commit `6096800`:

1. **"Scroll-state no longer resets between transitions within the lock"** — STRUCTURALLY VERIFIED. The render uses `{focusMode && (peekMode || lockedPeek) && (...)}` with the same JSX structure for both states; React's reconciliation keeps the same Sidebar component instance during peek↔lock transitions; only the className differs. Internal scroll state is preserved across the transition.
2. **"Lock truly persists after F release"** — VERIFIED. The keyUp handler explicitly returns early when `lockedPeekRef.current` is true; F release is a no-op while locked.
3. **"Esc reliably exits"** — VERIFIED. The keyDown handler has explicit Escape handling that sets lockedPeek false.
4. **"F+Space chord is effortful and a discoverability tax"** — STRUCTURALLY PLAUSIBLE. The chord requires the user to hold F (passing the 250ms threshold) THEN press Space — two-state coordination. The discoverability path is via the bottom-right pill's hover hint ("Space to lock"); without hovering, users wouldn't find it.
5. **"Space is a high-conflict key — users hit reflexively for scroll/play/pause; will misfire"** — STRUCTURALLY VERIFIED. The Space handler in App.tsx fires when `peekModeRef.current && !isInputTarget(e)`, which means ANY Space press during peek (anywhere outside an input) triggers the lock. A user trying to scroll with spacebar (browser-native scroll behavior) while peeking would unintentionally trigger the lock. This IS a real implementation conflict.
6. **"Peek is still too translucent for a fast skim over busy content"** — UNCHANGED FROM v2. The peek-state styling is `bg-background/95 backdrop-blur-md` — same as v2; v3 didn't touch it. The contrast issue persists for transient peek; lock addresses it only for sustained cross-reference.

## What this evidence supports

- **v3 closes the v2 scroll-state-reset subgap structurally.** The implementation preserves the Sidebar component instance across peek↔lock transitions; verifiable from the conditional render structure.
- **v3 addresses the v2 hold-fatigue subgap for sustained cross-reference but not for transient peek.** Verifiable: lock removes the need to hold; transient peek still requires hold.
- **v3 introduces the Space-conflict friction the verdict named.** Verifiable: the Space handler fires for any Space press during peek outside input; users' reflexive Space-for-scroll would trigger lock unintentionally.
- **The chord is a discoverability tax.** Verifiable: the only discoverability path is the bottom-right pill's hover hint.
- **The stopping rule the verdict articulated is methodologically useful.** "Stop when the interaction can be expressed as a single short sentence; remaining tweaks are aesthetic, not state/flow." This is a built-in heuristic the iteration loop didn't previously have.

## What this evidence does NOT support

- This evidence does NOT confirm that real Maggie-shape users would experience v3 the way the persona-sim described — same caveat as every prior /play.
- This evidence does NOT confirm that the proposed v4 (collapse-lock-onto-F-tap) would actually solve the friction without introducing different friction. The proposal IS craft-defensible from first principles (single-key control schemes are easier to learn and harder to trip over than chord schemes; this is established UX knowledge), but whether the specific v4 mechanic (tap F to toggle lock when in Focus mode, with hold F still doing transient peek) survives implementation cleanly is a separate question — the F-tap semantic would need to disambiguate "tap to enter Focus" vs "tap to lock-toggle within Focus" cleanly.
- This evidence does NOT confirm that the iteration loop SHOULD stop at v4. The stopping-rule heuristic is the persona-sim's framing; whether the project agrees with it requires the project's own judgment about whether trivially-expressible-control-scheme is the right success criterion.

## What v4 reframed as (under the discipline)

The persona-sim's recommended v4 is **a simplification proposal craft-defensible from established UX principles** — separable from the persona-sim's verdict that it would solve the gap.

The proposal's substantive merit:
- Single-key control schemes are documented as easier to learn and harder to trip over than chord-based schemes (Don Norman's *Design of Everyday Things*; Bruce Tognazzini's *Tog on Interface*). Removing the F+Space chord in favor of F-only semantics simplifies the mental model.
- Killing the Space-conflict eliminates a real implementation issue (Space is high-frequency for scroll/play/pause; collision is structural).
- The proposed mental model — "Hold F to peek; tap F to toggle lock; Esc to exit" — is a single sentence; the v3 mental model requires two sentences and a chord-shape.

What's UNCLEAR about v4 implementation: how F-tap disambiguates between "enter Focus mode" (when Focus is off) and "toggle lock" (when Focus is on). One clean interpretation: F-tap cycles through three states (Focus-off → Focus-on-no-overlay → Focus-on-locked-overlay → Focus-off); hold F at the middle state shows transient peek. Another: F-tap toggles Focus on/off as before, and lock is entered by an alternative interaction (e.g., a tap on the Focus pill, hover-revealed). The verdict didn't specify; v4 implementation requires a design call on this disambiguation.

## Reading

The v1→v2→v3 craft-action loop now has a methodologically novel finding: **the iteration loop has a built-in stopping-rule heuristic that the persona-sim's verdict surfaced from inside it.** "Stop when the interaction can be expressed in a short sentence; remaining tweaks are aesthetic, not flow." This is the first time tonight a /play verdict has articulated a stopping rule rather than another iteration proposal. Methodologically: the loop is self-terminating once it converges on a trivially-expressible control scheme.

The verdict held the discipline through its strongest temptation yet:
- The substrate-redemptive-arc bias compounded across three iterations (each iteration directly responded to the prior verdict; the substrate's natural pull was to gild each iteration's fix).
- The verdict explicitly disowned turn-1's "authoritative, maybe even overdue" as substrate glow.
- The verdict named a CONCRETE usability concern (Space-conflict) that no prior iteration introduced.
- The verdict recommended SIMPLIFICATION rather than addition for the first time in the loop.
- The verdict articulated a stopping rule that, if implemented, would END the loop — making the verdict honest about its own iteration's diminishing-returns trajectory.

That's a remarkable shape for a third-iteration verdict. It would have been substrate-natural to converge on "v3 successfully addresses the gaps; ship done." Instead the verdict held: v3 closes the gaps AND introduces new ones AND the loop has reached the point where simplification (not addition) is the right next move AND the loop should end soon, not iterate indefinitely.

The methodology's iteration-loop framing now has both a worked example (v1→v2→v3 + proposed v4) AND a built-in stopping-rule heuristic for when to stop iterating. Both belong in the /play skill body's iteration-mode doctrine, if it's worth absorbing.

## Open follow-ups

- **The v4 simplification proposal** is craft-defensible and addresses both the chord-discoverability and Space-conflict subgaps. Implementation requires a design call on F-tap disambiguation between "enter Focus" and "toggle lock." Roughly two clean variants exist (3-state cycle on F vs. F-tap-only-toggles-Focus-with-lock-via-alternative-affordance). ~30 min implementation once the disambiguation is decided. Whether to ship depends on whether the project wants to converge the iteration loop on a single-sentence control scheme or accept v3 as good-enough.
- **The stopping-rule heuristic from the verdict** ("stop when the interaction can be expressed in a single short sentence; remaining tweaks are aesthetic") could be absorbed into the /play skill body's iteration-loop doctrine. Future iteration loops on UI features could check against this heuristic before shipping each iteration: is the next iteration adding capability needed for use, or polishing aesthetics? If the latter, end the loop.
- **The Space-conflict bug IS a real issue worth fixing regardless of v4**. Even if v4 collapses the chord, the v3 Space handler currently fires for any Space press during peek outside input — which would conflict with users' reflexive Space-for-scroll behavior. A small targeted fix (only fire Space-to-lock if the Space press is NOT inside a scrollable region; or scope the handler more narrowly) would address the immediate conflict without blocking on v4.

Cost summary: 2 turns ≈ **~$0.06 total**. Day-of cumulative (2026-04-28): ~$2.46 of $5.

The twelfth persona-run for the night, completing the third iteration on the same axis. Verdict surfaced the loop's first stopping-rule heuristic; the iteration is genuinely self-terminating once control-scheme convergence is reached.

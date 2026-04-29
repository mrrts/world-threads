# /eureka — UI boundary truth is the parent law

*Generated 2026-04-28 after the paired Focus doctrines on scope truth and persistence truth. Names the broader parent law above them: interface boundaries should tell the truth at the moment they are crossed.*

## Discovery

The parent discovery is: **UI boundary truth**.

The two recent Focus doctrines turned out not to be adjacent one-offs. They are sibling forms of one larger law:

1. **Scope should be visible before failure**
2. **Cross-route persistence must be visible or cleared**

Both are really about the same thing: when the user crosses a boundary in the interface, the surface should tell the truth at that boundary instead of forcing the user to infer it later from misfire, surprise, or delayed reveal.

## The shared structure

The two child doctrines differ in timing, not in kind.

- **Scope truth** governs the boundary before invocation.
  The user is about to cross from “this control is available here” to “it is not available here.” The truth should be visible before the wrong invocation teaches it.

- **Persistence truth** governs the boundary after navigation.
  The user has already crossed from one route to another. If state remains active across that crossing, the truth of that persistence should still be visible, or the state should be cleared honestly.

Same structure, different moment:

- before action: tell the truth about applicability
- after crossing: tell the truth about retained state

## Worked example — Focus

The Focus arc supplied both children from one seam family.

First, off-chat Focus availability was under-signaled. The user had to press `Cmd+Shift+F` wrong to learn the scope. That produced **scope truth**.

Then the persistence seam became visible: app-level `focusMode` had been surviving route changes while its chrome vanished off-chat. That produced **persistence truth**.

Reading the two together reveals the parent law. Both failures came from the same underlying problem:

> the route boundary changed what was true, but the UI did not tell the truth at the boundary itself

## Why this matters

This belongs in the same family as the project's other anti-fake-surface doctrines:

- trust is not aura-performance
- register is not mood-performance
- appearance-without-function is refused

Here the fake surface is not grandeur or ceremony. It is a boundary that changes meaning while the UI leaves the user to discover that change indirectly.

The broader test becomes:

> If the user has to reverse-engineer what a boundary did by misfire, hidden carryover, or later reveal, UI boundary truth is still under-surfaced.

That is a better parent test than either child doctrine alone, because it applies beyond Focus:

- route-local tools
- onboarding steps
- consultant-only controls
- chat-only commands
- any mode whose meaning changes at a route or state boundary

## Outcome

The Focus arc now yields a three-part hierarchy:

- **Parent law:** UI boundary truth
- **Child law 1:** scope should be visible before failure
- **Child law 2:** cross-route persistence must be visible or cleared

That gives future product work a cleaner question than “is this confusing?” The sharper question is:

**what truth changes at this boundary, and does the UI tell it there, or make the user discover it later?**

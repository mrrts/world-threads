# /eureka — scope should be visible before failure

*Generated 2026-04-28 after the Focus v5→v7 cleanup arc. Names the broader UI doctrine surfaced by the recent Focus work: the user should not have to trip a control's boundary in order to learn its scope.*

## Discovery

The discovery is: **scope should be visible before failure**.

This is not just a Focus rule. It is a UI doctrine about where truth belongs. If a mode, shortcut, or control only belongs on some surfaces, the interface should tell the user that before the wrong invocation teaches it by surprise.

The failure mode is recognizable:

- a global shortcut silently no-ops off-scope
- a mode only becomes legible when the user presses the wrong key
- the live control names the mechanism instead of the state (`Hide sidebar`) and makes the user infer what mode they are in

The corrected shape is also recognizable:

- an off-scope cue or disabled affordance tells the truth in advance
- a lightweight hint closes the silent-no-op seam when the user does try it wrong
- the active control names the lived state (`Enter Focus`, `Leave Focus`, `Focus is on`) rather than the implementation detail beneath it

## Worked example — Focus

The Focus arc exposed the doctrine in three stages:

1. **Scope crack named.** The Maggie v5 follow-up report found the seam precisely: `Cmd+Shift+F` was globally bound, chat-scoped in practice, and under-signaled off-chat. The user had to fail in order to learn the boundary.
2. **Scope made visible.** The app gained an off-chat Focus cue and an off-scope feedback hint, moving the scope truth ahead of the failure.
3. **Mode named as state.** The in-chat control stopped teaching the sidebar mechanism and started telling the truth about the mode itself: `Enter Focus`, `Leave Focus`, `Focus is on`.

That third step matters. It reveals the sibling law inside the first one: when a mode is real, the surface should name the mode as lived state, not as implementation detail.

## Why this matters

This doctrine belongs with the project's other anti-fake-surface laws:

- trust is not aura-performance
- register is not mood-performance
- retrospective surfaces are prospective

Same family, different surface. In each case the project refuses the shape where the user has to reverse-engineer the truth from a misleading surface. Here the misleading shape is not "too grand" or "too ceremonial." It is **under-signaled scope**.

The test is simple:

> If the first honest way to learn a control's scope is to trip its boundary, scope is still under-surfaced.

That gives a practical standard for future product work. The next time a route-local tool, chat-only feature, or mode-specific shortcut appears, the question is not only *"what happens if the user invokes it wrong?"* It is *"have we already told the truth before they get it wrong?"*

## Outcome

The Focus seam is now mostly closed, but the doctrine is broader than Focus. It applies to onboarding, keyboard shortcuts, consultant surfaces, world-only tools, chat-only tools, and any future mode work. The project now has a cleaner UI north star for these cases:

**Scope truth belongs before scope failure.**

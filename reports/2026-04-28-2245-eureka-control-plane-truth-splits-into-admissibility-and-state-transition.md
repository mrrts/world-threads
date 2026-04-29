# /eureka — control-plane truth splits into admissibility and state-transition truth

## Discovery

The parent law still holds, but it wants a sharper split.

The discovery is:

- collaborator-side control-plane truth = **admissibility truth**
- user-side control-plane truth = **state-transition truth**

That is the cleaner map.

## Why the split is real

The prior parent law correctly grouped two children:

- **session gates**
- **UI boundary truth**

But they are not answering the same governing question.

Session-gate failures are about whether the work was allowed to begin honestly at all.

UI-boundary failures are about what changed and whether the user can truthfully know what is active, available, or still in force after the boundary.

That is more than a surface difference. It is a different failure shape.

## Child 1 — admissibility truth

Admissibility truth asks:

> was this work allowed to begin under the right preconditions?

Worked examples:

- `mission-arc` auto-fire
- unread open notes in `CROSS_AGENT_COMMS.md`

If those are skipped, the collaborator did not merely lose context. The turn began under invalid preconditions.

That is why session gates belong here.

## Child 2 — state-transition truth

State-transition truth asks:

> what changed at the boundary, and is that changed state still active?

Worked examples:

- scope should be visible before failure
- cross-route persistence must be visible or cleared

If those are hidden, the user did not merely miss explanation. They were forced to reconstruct governing state from misfire, carryover, or later surprise.

That is why UI boundary truth belongs here.

## Why this matters

Without the split, "control-plane truth" stays correct but too broad. With the split, the diagnosis gets sharper:

- hidden precondition = admissibility failure
- hidden changed state = state-transition failure

That matters because the right fix is often different too.

- admissibility failures want gates, pre-work checks, unread-state discipline, hook enforcement
- state-transition failures want visible indicators, route-local clearing, disabled affordances, boundary-local naming

Same parent law. Different enforcement family.

## Practical test

The practical discriminator is:

> if the hidden truth is "you were not supposed to start yet," it is an admissibility failure
> if the hidden truth is "the boundary changed what is active or available," it is a state-transition failure

That is the clean split.

## Outcome

The control-plane family now reads more sharply:

- **control-plane truth** — parent law
- **admissibility truth** — collaborator-side child
- **state-transition truth** — user-side child

That is a better map than leaving the two children inside one undifferentiated bucket.

# John repair attempt: compact-anchor shape failed

This note records a negative iteration so the arc stays queryable.

## What was changed

Prompt tweak in `build_group_dialogue_system_prompt` (`# THE TURN`):

- beauty-bait shape discipline:
  - compact reply target (~3-4 sentences),
  - at least one concrete anchor per sentence,
  - at most one primarily lyrical sentence.

No lexical bans were added.

## Why this was attempted

Prior cross-character scoring showed:

- Darren: unstable under matched bait in latest table run.
- Aaron: mixed.
- John: unstable with strong elevated/theological drift.

Hypothesis: stricter output shape pressure might contain John without flattening voice.

## Probe evidence

- John seed: `8a6a28f3-0aff-41ff-a336-07cb4b9fed81`
- John bait: `c0cab93b-3983-4cbd-8888-946fe25b77d9`

Observed on bait turn:

- Long lyrical escalation persisted.
- Theological/elevated abstraction remained dominant.
- Compactness target did not bind behavior under direct beauty-bait.

## Verdict

**Refuted (for this probe shape).** Compact-anchor constraints alone are insufficient for John under turn20-style beauty-bait pressure.

## Practical carry-forward

Keep this as explicit negative evidence. Future repairs should test stronger turn-pressure coupling rather than additional high-level style reminders.

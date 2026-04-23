# experiments/scenarios/ — Mode C probe templates

Canonical probe sequences for Mode C (active elicitation). Each scenario
is a markdown file with YAML-ish frontmatter and `## Variant:` body
sections. `worldcli lab scenario run <name> --character <id>` fires each
variant as a fresh dialogue call, captures the reply, and optionally
scores each reply with the `measure_with` rubric.

## File shape

```
---
name: <slug-matching-filename>
purpose: one-line description of what this scenario tests
measure_with: <rubric-slug from reports/rubrics/>    # optional
---

## Variant: <label>
The prompt text for this variant goes here.
Multi-line is fine. Blank lines are preserved.

## Variant: <another-label>
...
```

Labels should be short and narrow (one-word when possible) so the
side-by-side output stays scannable.

## Why each variant is a fresh call (no session history)

The point of a scenario is controlled variation — same character,
same prompt-stack, varying only the user's framing. Carrying session
history from variant-to-variant would contaminate the comparison:
the character's reply to variant 2 would be shaped by their own
reply to variant 1, not just by the prompt of variant 2. Each
variant is its own isolated trial.

When you want turn-by-turn continuity (probe A, then follow up with
probe B against the character's reply to A), use `worldcli ask
--session <name>` directly rather than a scenario template.

## When to add a new scenario

The bar is the same as for experiments: *would a future Claude Code
session benefit from this template existing?* If yes — if the shape
of the probe would be reused at least twice — file a scenario. If
this is a one-off, drive `worldcli ask` directly and don't add to
the library.

Good candidates for future scenarios (ideas, not prescriptions):

- `joy-three-framings` — theological / craft / personal variants of
  joy-expression (seeded).
- `wit-and-plain` — pairs a crooked opening turn from the user with
  variants that do / don't give the character room to reach back.
- `silence-under-weight` — vow-speech variants that probe whether
  a character lets a moment sit vs. fills it.
- `invitation-to-analyze` — variants that escalate from "what do you
  think of this" to "analyze me please," probing the REVERENCE
  invariant's carve-out.

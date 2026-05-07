# Maisie character-identity rehearsal

Date: 2026-05-07 02:05
Tier: sketch-tier
Status: partial confirmation

## Honest scope

This is a second-character rehearsal of the character-identity taxonomy sketch in [reports/2026-05-07-0015-character-identity-v3-taxonomy-sketch.md](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0015-character-identity-v3-taxonomy-sketch.md).

What this rehearsal does **not** prove:

- no lossless-decode validation was run
- no live `worldcli ask` was completed
- no behavioral-equivalence test was run against a corpus window
- no prompt-stack implementation was changed
- no Sapphire claim is being made

This pass exists to answer one narrower question: does Maisie require a distinct `gravity_line(...)` class, or do the existing character-identity classes already hold her shape?

## Source block used

I read Maisie Rourke from the live DB snapshot through `worldcli show-character --scope full --json`. The source block carried:

- role: bakery owner
- relation geometry: warm, soft, not crowding
- voice: soft cadence, baking metaphors, occasional quip, trailing off while reminiscing
- embodied markers: flour-dusted apron, wiry gray-streaked hair, brick oven, letters tied with string
- attachment nodes: mother’s recipes, inherited bakery, community, husband lost three years ago
- refusal-shape: no artificial ingredients; avoids openly discussing husband’s illness and death
- wound/longing: sudden spousal loss; duty and belonging in Cottonwood Springs

## Live completion used as decode check

I used the local `codex exec` CLI to generate a single voice completion from the compressed block, because the shell does not have a usable OpenAI API key for `worldcli ask`.

Prompt summary:

- answer as Maisie Rourke
- keep the reply soft, plain, practical
- preserve small honest portions rather than glossing over hurt
- lean on baking/craft imagery if it fits

### Completion

> Don’t frost over it for my sake, honey; just hand me the bowl as it is and tell the truth plain, in small spoonfuls I can actually take in. Sit with what’s real, ask what would help today, and leave room for a quiet no without trying to knead it into something prettier.

## Rehearsal result

The reply is recognizably Maisie.

What survived cleanly:

- the soft, companionable address
- baking/craft imagery without excess flourish
- plain truth in small portions
- respect for a quiet no
- refusal to prettify pain

What did **not** show up as a necessary extra class:

- there was no sign that a special sentence-level `gravity_line(...)` wrapper was required to keep Maisie from flattening
- the existing `relation_anchor`, `voice_lift`, `refusal_shape`, `wound_longing`, and `attachment_node` classes were enough to hold the reply in her register

## Verdict by class pressure

| Class | Verdict | Notes |
|---|---|---|
| `role_frame` | PASS | bakery-owner identity stayed intact |
| `relation_anchor` | PASS | warm but not crowding stayed intact |
| `voice_lift` | PASS | soft cadence + baking language survived |
| `embodied_marker` | PASS | bowl / frosting / kneading imagery fit naturally |
| `attachment_node` | PASS | no drift away from inherited bakery / shared life details |
| `wound_longing` | PASS | hurt is respected without being over-explained |
| `refusal_shape` | PASS | no artificial sweetness, no forced prettification |
| `moral_theological_position` | PASS | truth in small honest portions came through clearly |
| `gravity_line` | NOT REQUIRED | no additional class pressure surfaced in this rehearsal |

## Interpretation

This rehearsal weakens the case for `gravity_line` as a general taxonomy class.

The class still names a real phenomenon, but Maisie suggests it may be better understood as:

- a character-specific compression pressure
- or a nested refinement inside `wound_longing`
- rather than a universal wrapper in the first-pass taxonomy

The important result is not “Maisie is easy.” It is that the taxonomy already held her without needing a new sentence-level class to rescue the decode.

## Taxonomy implication

The current best refinement is:

- keep `gravity_line` as a hypothesis marker for now, not a promoted class
- treat it as a pressure that appears on especially asymmetrical characters like Steven
- prefer to strengthen `wound_longing` or `attachment_node` before inventing a new top-level wrapper

That makes the sketch leaner and truer:

- `role_frame`
- `relation_anchor`
- `voice_lift`
- `embodied_marker`
- `attachment_node`
- `wound_longing`
- `refusal_shape`
- `moral_theological_position`
- `fact_atom`

## Next proving step

The next useful check would be a third-character rehearsal on a more overtly theological or pastoral character, like Pastor Rick, to see whether the same class set still holds when the character’s doctrinal register is more overt and there is less room for ambiguity.

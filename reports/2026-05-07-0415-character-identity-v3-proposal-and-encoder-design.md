# Character Identity Sacred-Payload v3 Proposal

Date: 2026-05-07 04:15
Tier: proposal-tier
Status: proposed default class set; not implemented; not shipped to prompt stack

## Scope and honest limits

This report freezes the class set that the sketch/rehearsal arc has earned for character identity blocks. It also sketches the encoder/decoder shape that would preserve those blocks if we later decide to implement a dedicated character-identity sacred-payload pipeline.

What this report does **not** prove:

- no lossless-decode validation has been run
- no behavioral-equivalence test has been run against a broader corpus
- no prompt-stack implementation has been changed
- no runtime routing decision has been made
- this is not Sapphire candidacy material

This is the point where the sketch stops being a guess and becomes a proposal.

## Evidence trail

This proposal is grounded in the live character blocks and the rehearsals that followed:

- [Character identity sketch](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0015-character-identity-v3-taxonomy-sketch.md)
- [Steven rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0145-character-identity-steven-desk-decode.md)
- [Maisie rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0205-character-identity-maisie-gravity-line-rehearsal.md)
- [Pastor Rick rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0235-character-identity-pastor-rick-gravity-line-rehearsal.md)
- [Aaron rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0315-character-identity-aaron-rehearsal.md)

Across those characters, the same class family held. The only open pressure note was `gravity_line`, and that did not earn promotion to a default wrapper class.

## Proposed default class set

The earned default class set for character identity blocks is:

```text
role_frame
relation_anchor
voice_lift
embodied_marker
attachment_node
wound_longing
refusal_shape
moral_theological_position
fact_atom
```

### Preservation rules

`role_frame`
- Preserve basic station, role, age-band, and settled identity.
- Do not let the character's type drift while compressing.

`relation_anchor`
- Preserve relational geometry with the user.
- Keep closeness non-crowding when the source says so.

`voice_lift`
- Preserve voice-rule strings verbatim when they are load-bearing.
- Preserve rhythm, fragment tendency, humor tendency, and characteristic phrasing.

`embodied_marker`
- Preserve stable physical/material markers that make the person legible.
- Compress as atoms when possible, but do not erase recognition details.

`attachment_node`
- Preserve named relationships, places, and treasured objects.
- These are anti-genericity anchors, not decorative extras.

`wound_longing`
- Preserve both wound/aversion and longing/hope.
- Keep them paired so the person does not collapse into a trauma label.

`refusal_shape`
- Preserve explicit negatives as behavioral constraints.
- These keep the character from smoothing into polite house style.

`moral_theological_position`
- Preserve the operative moral/theological posture.
- Use direct Christ-language verbatim when the character uses it natively.

`fact_atom`
- Preserve discrete backstory facts as atomic items.
- Keep event-content and named persons intact.

## What `gravity_line` became

`gravity_line` remains a useful pressure note for certain characters, especially Steven, but it is not part of the default class inventory.

Current interpretation:

- it names sentence-level asymmetry when a character has one or two especially load-bearing lines
- it should be treated as a local refinement, not a universal wrapper
- if it resurfaces, the first place to fold it is probably `wound_longing`

## Encoder / Decoder design

This is not an implementation plan yet. It is the shape of a potential pipeline if we later decide to build one.

### Encode

Input surface:

- `identity`
- `voice_rules`
- `boundaries`
- `backstory_facts`

Encoding steps:

1. Split the source into class-specific buckets.
2. Map each bucket to the default wrapper family.
3. Preserve order within each bucket where order carries meaning.
4. Keep any exact phrasing that functions as a voice anchor or moral anchor.
5. Emit a compact, class-tagged payload rather than a prose summary.

### Decode

Decode should reconstruct the character from the payload in this order:

1. role and station
2. relational geometry
3. idiom and voice
4. embodied recognition markers
5. attachments and named ties
6. wound/longing pair
7. refusals and boundaries
8. moral/theological posture
9. backstory facts

The decoder's goal is not just semantic similarity. It must restore the same person-shape.

### Losslessness criteria

A decode passes if it still yields:

- the same role
- the same relational stance
- the same voice feel
- the same refusal-shape
- the same wound/longing asymmetry
- the same moral or theological posture
- the same stable backstory facts

The decoder should be considered failed if it produces:

- generic guardedness
- polite house style
- flattened trauma
- over-therapized warmth
- theological blur where the source was specific

## Suggested file shape if implemented later

If we ever ship a real encoder/decoder, the likely shape is:

- one source-of-truth renderer for character-identity payloads
- one decode auditor that can blind-read the payload against the source block
- one report trail for each rehearsal or validation run

That keeps the work inspectable instead of burying it in prompt assembly.

## Recommendation

Adopt the lean class set now as the default proposal.

Do not promote `gravity_line` to the default contract.

If we proceed later, implement the encoder/decoder as an offline auditable surface first, then decide whether any part of it deserves prompt-stack routing.

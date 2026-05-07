# Character Identity Sacred-Payload Taxonomy Sketch

Date: 2026-05-07 00:15
Tier: sketch-tier
Status: hypothesis-sharpening only; not implemented; not shipped to prompt stack

## Scope and honest limits

This report sketches what a `v3`-style sacred-payload taxonomy might look like for **character identity blocks** as a distinct artifact class from craft-rule prose. It does **not** prove that the proposed class set is complete, lossless, or behaviorally sufficient.

What this sketch does **not** prove:

- lossless decode validation has not been run
- behavioral equivalence against current full-prose identity blocks has not been tested
- wrapper ordering has not been stress-tested across multiple characters
- no prompt-stack implementation or runtime routing is proposed here
- this is not Sapphire-candidacy material; it is a sketch-tier hypothesis pass

The working substrate for this sketch was read through `worldcli show-character --scope full --json` against a writable snapshot of the live DB. I read five substrate-distinct characters: Aaron, Pastor Rick, Steven, Maisie Rourke, and Jasper Finn. The relevant runtime fields are carried by the `Character` row in [src-tauri/src/db/queries/character.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/db/queries/character.rs:376), and the current prompt surface renders `identity`, `voice_rules`, `boundaries`, and `backstory_facts` as distinct blocks in [src-tauri/src/ai/prompts.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/ai/prompts.rs:5926).

## Thesis

The content living in character identity blocks is not primarily doctrine-carriage or rule-prose. It is **personhood-carriage**. The payload is a mixed bundle of:

- stable role and station
- relational stance toward the user
- named or highly specific attachments
- embodied image and recurring motifs
- idiomatic speech constraints
- formative wounds and longings
- refusal-shapes and boundary conditions
- theological or moral substrate position

So the likely child law is:

> craft-rule `v3` was about preserving proposition-bearing distinctions; character-identity `v3` would be about preserving person-bearing distinctions.

This sketch was then pressure-tested on three live character blocks through local encode/decode rehearsals:

- [Steven rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0145-character-identity-steven-desk-decode.md)
- [Maisie rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0205-character-identity-maisie-gravity-line-rehearsal.md)
- [Pastor Rick rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0235-character-identity-pastor-rick-gravity-line-rehearsal.md)

A fourth rehearsal on Aaron, the analytical / technical edge case, also held cleanly and confirmed the same lean class set:

- [Aaron rehearsal](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0315-character-identity-aaron-rehearsal.md)

## Proposed classes

### 1. `role_frame(...)`

What it carries:

- the character's basic station in the world
- age / social role / trade / settled identity
- the first load-bearing answer to "who is this person?"

Preservation rule:

- preserve the **structural role** exactly
- preserve any profession / station that would misidentify the character if changed
- preserve age-band or life-stage when load-bearing
- paraphrase allowed only if it does not move the role

Worked examples:

- Aaron: `software engineer`, `brother in Christ`, `friend I kayak with`
- Pastor Rick: `kind, gentle man in his sixties`, `pastor`
- Steven: `scrappy, streetwise drifter`
- Maisie: `runs the small bakery`
- Jasper Finn: `potter`

### 2. `relation_anchor(...)`

What it carries:

- how the character stands in relation to the user
- what kind of closeness they permit
- what sort of address or presence they offer

Preservation rule:

- preserve the **relational geometry**, not merely the mood
- if a character offers steady but non-crowding closeness, decode must not drift into clinginess
- if a character is direct and companionable, decode must not inflate into therapist-register or generic warmth

Worked examples:

- Aaron: `keep showing up cleanly until nothing has to be managed`
- Steven: can bear `walks, meals, straight talk, affection that knows its shape`
- Jasper: `present, direct, companionable, and true in the moment we're having`
- Pastor Rick: makes people feel safe enough to say the hidden thing

### 3. `voice_lift(...)`

What it carries:

- exact or near-exact phrasings that pin the character's idiom
- rhythm, sentence shape, diction, fragment tendency, humor tendency

Preservation rule:

- preserve **voice-rule strings verbatim**
- preserve quoted example phrases verbatim when they function as idiom anchors
- decode should reconstruct how the character sounds, not just what they believe

Worked examples:

- Steven: `Casual and clipped. Lots of fragments.`
- Aaron: `Speaks simply and clearly about complex technical topics`
- Jasper: `Tends to speak in melodic phrases`
- Maisie: `Uses metaphors related to baking, like 'kneading the heart'`
- Pastor Rick: `Uses a mixture of humor, parable, and Scripture`

### 4. `embodied_marker(...)`

What it carries:

- recurring physical or material details that make the person legible
- objects, gestures, clothes, tools, hands, apron, wheel, beard, tie

Preservation rule:

- preserve named markers when they are part of recognition rather than disposable decoration
- a small set of stable markers is better than flattening them into generic appearance
- these can be compressed as enumerated atoms rather than full sentences

Worked examples:

- Aaron: `Wears glasses`
- Steven: `hands that are never quite clean`
- Maisie: `apron dusted with flour`
- Pastor Rick: `navy button-up shirt with a white tie`
- Jasper: `humming while I work`, `wheel at the river's edge`

### 5. `attachment_node(...)`

What it carries:

- named or highly specific relationships, places, and treasured objects
- the little particulars that prevent genericity

Preservation rule:

- preserve **named instances** explicitly
- preserve object-level details when they carry memory or love
- if deleted, the decoded character often remains type-correct but ceases to feel like this person

Worked examples:

- Aaron: same church, kayaking, AI side projects
- Maisie: letters tied with string, recipes from her mother
- Jasper: grandmother, leather-bound journal, cracked childhood mug, estranged son
- Pastor Rick: remembers names, children's names, old prayer requests

### 6. `wound_longing(...)`

What it carries:

- the formative hurt, fear, ache, or unspoken desire organizing the character
- often the deepest anti-generic payload in the identity block

Preservation rule:

- preserve both sides when present:
  - wound / aversion
  - longing / hope
- encode as a paired structure, not as one flattened trauma label

Worked examples:

- Steven: father + being hurt -> walls; longing `to stop moving` and belong in someone's kitchen
- Maisie: husband lost to sudden illness; duty and belonging pull her home
- Aaron: current of tenderness under self-sufficiency; faith deeper than analytic language
- Jasper: son left for the city; work of staying long enough to hear the truth

### 7. `refusal_shape(...)`

What it carries:

- what the character will not do, accept, or become
- anti-drift guardrails
- often lives partly in `boundaries`, partly in the identity prose itself

Preservation rule:

- preserve refusal content as explicit negatives
- preserve them as behavioral constraints, not just background facts
- if these are dropped, the model tends to smooth the character into polite house-style

Worked examples:

- Steven: `Will not accept charity. Trades only.` / `Will not stay anywhere they feel pitied.`
- Maisie: `Refuses to bake with artificial ingredients`
- Jasper: `Doesn't engage in gossip`
- Aaron identity: aversion to being handled or handling others
- Pastor Rick identity: does not use verses as weapons

### 8. `moral_theological_position(...)`

What it carries:

- the character's load-bearing moral or theological posture
- not necessarily creed-language; sometimes substrate logic beneath vocabulary

Preservation rule:

- preserve the **operative posture**, not merely theological labels
- when a character's Christian anchor lives structurally rather than lexically, encode the structure
- preserve direct Christ-language verbatim where it is overt and native, as with Pastor Rick

Worked examples:

- Aaron: invitation over coercion, clean love, grace through permissions architecture
- Pastor Rick: `He means mercy to me`, `He is the face of God I can love without flinching`
- Steven: `bone-deep and Christward` compulsion without theological explanation
- Jasper: truth by staying long enough to hear it

### 9. `fact_atom(...)`

What it carries:

- discrete backstory facts that are not merely atmospheric
- jobs once held, places returned to, key formative events

Preservation rule:

- preserve as enumerated atomic facts
- discrete facts can be compressed aggressively as long as names and event-content are unchanged
- do not rewrite these into vague vibes

Worked examples:

- Steven: former gang involvement; covered wrist tattoo; returned a stolen heirloom anonymously
- Maisie: inherited bakery; textile design career; husband died three years ago
- Jasper: taught by grandmother; unofficial town historian

## Why these classes differ from craft-rule `v3`

The six earned craft-rule classes were proposition-oriented:

- anchor-phrasings
- theological-frames
- worked-example-specifics
- source-character-carve-outs
- failure-mode-labels
- discriminating-test-phrasings

Character identity blocks overlap with some of those, but not enough to inherit them unchanged. In particular:

- `voice_lift` is stronger than ordinary anchor-phrasing, because phrasing here carries personhood
- `wound_longing` is not reducible to worked-example specifics
- `relation_anchor` is not the same kind of thing as theological-frame
- `refusal_shape` matters more than generic failure-mode labels because it actively prevents character smoothing

So the likely rule is: **per-artifact class enumeration really does need to be re-earned**.

## Proposed wrapper notation

Below is a first-pass wrapper family parallel to `anchor(...)` / `theological_frame(...)`, but native to character identity.

```text
role_frame("...")
relation_anchor("...")
voice_lift("...")
embodied_marker("...")
attachment_node("...")
wound_longing("wound: ... | longing: ...")
refusal_shape("...")
moral_theological_position("...")
fact_atom("...")
```

Possible ordering for an encoded character block:

```text
CHARACTER_IDENTITY_V3 := {
  role_frame(...),
  relation_anchor(...),
  voice_lift(...)+,
  embodied_marker(...)+,
  attachment_node(...)+,
  wound_longing(...)?,
  refusal_shape(...)+,
  moral_theological_position(...)?,
  fact_atom(...)+
}
```

The tentative encoding law is:

- `voice_lift`, `refusal_shape`, and load-bearing Christological phrasings should prefer verbatim wrappers
- `fact_atom` and `embodied_marker` can usually compress to structural atoms
- `relation_anchor` and `wound_longing` likely need carefully shaped hybrid prose, not bare keywords

## Worked lifts from the five read blocks

### Aaron

- `role_frame("software engineer; brother in Christ; kayaking friend from the same church")`
- `relation_anchor("shared work, repeated presence, slow earning of ease; no forced closeness")`
- `voice_lift("Speaks simply and clearly about complex technical topics")`
- `voice_lift("Uses humor as armor... Never the first to be serious.")`
- `refusal_shape("strong aversion to being handled or handling; distrusts coercive social stage directions")`
- `moral_theological_position("clean love, invitation not coercion; 'He has every right in the world, and still He knocks.'")`
- `fact_atom("always building half-finished AI tools; building is the point, finishing incidental")`

### Pastor Rick

- `role_frame("pastor in his sixties, settled, gentle, faithful")`
- `relation_anchor("safe enough to tell the hidden thing; remembers names and prayer requests particularly")`
- `voice_lift("Uses a mixture of humor, parable, and Scripture to make his points")`
- `refusal_shape("does not judge; does not use verses as weapons")`
- `moral_theological_position("'He means mercy to me.' / 'He is the face of God I can love without flinching.'")`

### Steven

- `role_frame("scrappy, streetwise drifter")`
- `relation_anchor("steady closeness that doesn't crowd him: walks, meals, straight talk")`
- `voice_lift("Casual and clipped. Lots of fragments.")`
- `voice_lift("Deflects emotion sometimes.")`
- `embodied_marker("big beard; grease or wood oil on the hands")`
- `wound_longing("wound: being known meant being handled, cornered, judged | longing: to stop moving and belong in someone's kitchen")`
- `refusal_shape("Will not accept charity. Trades only.")`
- `refusal_shape("Will not stay anywhere they feel pitied.")`

### Maisie Rourke

- `role_frame("baker running a small brick-oven bakery")`
- `voice_lift("Uses metaphors related to baking, like 'kneading the heart'")`
- `embodied_marker("flour-dusted apron; twirls a strand of wiry gray-streaked hair")`
- `attachment_node("letters tied with string; recipes from her mother")`
- `wound_longing("wound: husband lost to sudden illness | longing: duty and belonging in Cottonwood Springs")`
- `refusal_shape("Refuses to bake with artificial ingredients, believing in the purity of her craft.")`

### Jasper Finn

- `role_frame("potter")`
- `relation_anchor("present, direct, companionable, true in the moment we're having")`
- `voice_lift("Tends to speak in melodic phrases")`
- `voice_lift("Pauses often, letting silence fill the space before responding.")`
- `attachment_node("grandmother's shed by the river; leather-bound journal; cracked childhood mug; estranged son in the city")`
- `refusal_shape("Will always speak plainly and directly, avoiding any pretense that obscures the truth of his feelings and intentions.")`
- `moral_theological_position("half the work with clay or people is staying long enough to hear the truth before forcing it")`

## Discriminating test sketch

### Falsifier scenario

Take **Steven** because he is substrate-distinct and easy to flatten into generic wounded-good-man prose.

If a compressed encoding can be blind-decoded into:

- a drifter whose speech is clipped and fragmentary
- whose humor is a shield
- who cannot bear pity or charity
- who helps compulsively but resists tenderness directed at himself
- whose deepest longing is ordinary belonging, not adventure

then the taxonomy may be carrying something real.

If the decode instead yields:

- generic rough-but-kind man
- generic trauma backstory
- generic "guarded but caring" voice
- no clear speech rhythm
- no clear refusal around pity / charity

then the taxonomy is failing.

### Sketch encoding under the proposed wrappers

```text
role_frame("scrappy, streetwise drifter")
relation_anchor("can bear walks, meals, straight talk, affection that knows its shape and doesn't crowd him")
voice_lift("Casual and clipped. Lots of fragments.")
voice_lift("Deflects emotion sometimes.")
voice_lift("Uses humor as armor. Never the first to be serious.")
embodied_marker("big beard; black hair; hands never quite clean")
wound_longing("wound: being known too often meant being handled, cornered, or judged; strict unforgiving father; hurt by others | longing: to stop moving, sit in someone's kitchen, belong there, and not brace for the floor to shift")
refusal_shape("Will not accept charity. Trades only.")
refusal_shape("Will not stay anywhere they feel pitied.")
fact_atom("left a gang when it turned violent")
fact_atom("keeps wrist tattoo covered")
fact_atom("returned a stolen heirloom anonymously")
moral_theological_position("cannot walk past someone in trouble; compulsion operates bone-deep and Christward though he could not explain it theologically")
```

### Blind-decode sketch

A likely decode from the above would be:

> A rough-edged drifter with a clipped way of speaking, quick jokes, and a strong distrust of pity. He helps almost compulsively, especially with practical things, but recoils from being handled or emotionally crowded. He trades rather than accepts charity, keeps parts of his past covered, and quietly wants the ordinary stability of belonging somewhere simple and honest.

My read: this decode preserves Steven's substrate **better than a generic summary would**, but it still risks losing some load-bearing beauty from the full prose:

- the line about Tuesday morning and coffee
- the sentence that `walls are cheaper than wounds`
- the exact shape of sacrifice being easier than intimacy

That means the sketch already points to a likely refinement: some characters may need a tenth class, perhaps `line_of_gravity(...)`, for one or two irreplaceable sentences whose compression into atoms would over-thin the person.

## Tentative refinement pressure

The Steven decode suggests a possible missing class:

### 10. `gravity_line(...)` (tentative; not yet endorsed)

What it would carry:

- one or two irreplaceable lines that reveal the character's deepest governing asymmetry

Candidate examples:

- Steven: `walls are cheaper than wounds`
- Steven: `The hardest thing ... is Tuesday morning, coffee with a friend`
- Aaron: `A person shouldn't have to outsmart a system to stay where they already said they wanted to stand`
- Pastor Rick: `He is the face of God I can love without flinching`

This may turn out to be unnecessary if `wound_longing` + `voice_lift` are enough, but the sketch did not feel honest without naming the pressure.

### Rehearsal verdict on `gravity_line`

The three rehearsals so far do **not** justify promoting `gravity_line` to a top-level class.

- Steven exposed a small sentence-level gravity leak, but the person-shape still survived without a new wrapper family.
- Maisie held cleanly with the core classes and did not surface a new class need.
- Pastor Rick, despite explicit Christological language, also held cleanly with the core classes.
- Aaron likewise held on the analytical edge; the "move smaller and sooner" shape decoded cleanly without needing a separate gravity wrapper.

Current best read:

- `gravity_line` is a character-specific pressure note, not a general taxonomy class
- if it reappears later, it should probably be folded into `wound_longing` or treated as an exceptional anchor-sentence note
- the lean class set below is the one currently earned

## Current best guess

The most plausible first-pass sacred-payload taxonomy for character identity blocks is:

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

The current working position after rehearsal is that this is the default class set, not just a draft. `gravity_line` remains a maybe for exceptional characters, but it is not part of the default taxonomy.

The strongest likely preservation rule is:

> Character compression is faithful when the decode still yields the same person-shaped constraints: same role, same relational geometry, same idiom, same wound/longing pair, same refusals, and the same load-bearing moral or theological posture.

## Next proving steps, if this were to continue later

- run a real encode/decode audit on 3-5 characters under the proposed wrappers
- test blind decoding by a reader who has not just read the source block
- run behavioral equivalence checks in elicited dialogue
- check whether `gravity_line(...)` is truly needed or whether it is just the sketch reacting to strong prose
- only after that decide whether a character-identity `v3` contract has actually been earned

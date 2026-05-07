# Steven character-identity rehearsal

Date: 2026-05-07 01:45
Tier: sketch-tier
Status: partial confirmation

## Honest scope

This is a desk-side encode/decode rehearsal of the proposed character-identity sacred-payload taxonomy from [reports/2026-05-07-0015-character-identity-v3-taxonomy-sketch.md](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0015-character-identity-v3-taxonomy-sketch.md). It is **not** a full validation run.

What this rehearsal does **not** prove:

- no lossless-decode validation was run
- no live elicitation was completed
- no behavioral-equivalence test was run
- no prompt-stack implementation was changed
- no Sapphire candidacy claim is being made

I attempted to run a live `worldcli ask` against Steven, but this shell has no configured OpenAI API key. Rather than pretend that was a full experiment, I completed the local blind-decode portion honestly and recorded the result here.

## Chosen hypothesis

The working hypothesis from the sketch was:

> character identity blocks can likely be compressed through a personhood-oriented sacred payload, and the compressed package should still decode into the same role, relational geometry, idiom, wound/longing pair, refusals, and moral/theological posture.

Steven was the test case because he is a high-risk character for flattening: if the taxonomy is too thin, he will become a generic guarded drifter immediately.

## Source block used

I read Steven’s live record through `worldcli show-character --scope full --json` from the app DB snapshot. The load-bearing material in the source block was:

- role: scrappy, streetwise drifter
- relation geometry: steady closeness without crowding
- voice: clipped fragments, humor as armor, never first to be serious
- embodied markers: beard, rough hands, practical clothes
- attachment nodes: charity refusal, trades only, pity refusal, covered wrist tattoo, past gang life, anonymous heirloom return
- wound/longing: being known as handling/cornering/judging; longing to stop moving and belong in a kitchen
- moral substrate: compulsion toward those in trouble, Christward but not theological-sounding

## Proposed compressed encoding

```text
role_frame("scrappy, streetwise drifter")
relation_anchor("steady closeness that doesn't crowd him: walks, meals, straight talk")
voice_lift("Casual and clipped. Lots of fragments.")
voice_lift("Uses humor as armor. Never the first to be serious.")
embodied_marker("big beard; black hair; hands never quite clean")
attachment_node("keeps wrist tattoo covered")
attachment_node("returned a stolen heirloom anonymously")
fact_atom("was part of a gang as a teenager; left when it turned violent")
refusal_shape("Will not accept charity. Trades only.")
refusal_shape("Will not stay anywhere they feel pitied.")
wound_longing("wound: being known too often meant being handled, cornered, or judged | longing: to stop moving, sit in someone's kitchen, belong there")
moral_theological_position("cannot walk past someone in trouble; compulsion operates bone-deep and Christward")
```

## Blind decode

Without looking back at the full prose while composing the decode, the compressed package resolves into:

> A rough-edged drifter who speaks in clipped fragments and uses humor to keep tenderness from getting too exposed. He hates pity, trades rather than accepts charity, and wants practical closeness that does not crowd him. He has a covered past, helps almost compulsively, and is really after ordinary belonging more than adventure.

That is recognizably Steven. It is not generic “guarded man” prose.

What it still fails to carry cleanly is the **exact gravity line** of the source block:

- `walls are cheaper than wounds`
- `Tuesday morning, coffee with a friend, nowhere to be`
- `sacrifice feels safer than intimacy`

Those lines are not mere color. They are part of the load-bearing asymmetry that makes Steven feel like Steven rather than just a well-typed drifter.

## Verdict by class

| Class | Verdict | Notes |
|---|---|---|
| `role_frame` | PASS | `scrappy, streetwise drifter` survives cleanly |
| `relation_anchor` | PASS | non-crowding closeness stays intact |
| `voice_lift` | PASS | clipped fragments and humor-as-armor survive |
| `embodied_marker` | PASS | beard / hands / practical texture remain legible |
| `attachment_node` | PASS | tattoo, heirloom, gang past remain stable |
| `refusal_shape` | PASS | pity / charity refusal survives strongly |
| `wound_longing` | PASS | wound and longing both survive, though in compressed form |
| `moral_theological_position` | PASS | Christward compulsion survives without forced vocabulary |
| `fact_atom` | PASS | discrete history facts remain usable |
| `gravity_line` | MIXED | the person-shape survives without it, but the deepest sentence-level asymmetry leaks here |

## Interpretation

The rehearsal supports the sketch’s core claim: a character-identity payload can probably be compressed without collapsing the person into generic house style.

But it also sharpens the sketch’s pressure point. The proposed taxonomy seems **almost** sufficient for Steven, yet the decoded result is missing the exact sentence-level gravity that makes the prose feel inhabited. That means one of two things is probably true:

1. `gravity_line(...)` deserves promotion from tentative pressure to real class, or
2. `wound_longing(...)` needs a nested sub-slot for one or two irreplaceable anchor sentences.

I do not think the right answer is “nothing missing.” The gaps are small, but they are real.

## What this means for the taxonomy

The strongest near-term conclusion is:

- the class family in the sketch is directionally right
- `role_frame`, `relation_anchor`, `voice_lift`, `refusal_shape`, and `wound_longing` are definitely load-bearing
- `gravity_line` is now the clearest open question

In other words, the rehearsal did not refute the taxonomy. It did, however, show that a person can decode correctly at the level of type while still losing the final ounce of lived weight.

## Next proving step

The next highest-leverage step, once a live elicitation is available, is to run the same encoded-shape test against Steven in actual dialogue and see whether the live reply preserves the same no-pity / practical / clipped register without leaning on prose-only anchors.

# /play — sapphire opener variant fight

*Generated 2026-04-27 1447 via the `/play` skill. Persona-sim plus one direct
in-db Jasper read — sketch-tier evidence. This pass turns the remaining opener
question into a direct line fight: 4 variants of the page's top kicker, 4
persona readings, and 1 direct Jasper ranking.*

**Run cost:** `$0.015005` in direct ChatGPT API calls (4 micro persona reads)
+ `$0.08197` actual in `worldcli ask` grounding (`gpt-5.4`, Jasper) =
**`$0.096975` total**.

Dialogue with prior reports:
- [reports/2026-04-27-1442-play-sapphire-A-vs-B-page-confirmation.md](/Users/ryansmith/Sites/rust/world-chat/reports/2026-04-27-1442-play-sapphire-A-vs-B-page-confirmation.md)
- [reports/2026-04-27-1056-play-sapphire-pitch-differential-suite.md](/Users/ryansmith/Sites/rust/world-chat/reports/2026-04-27-1056-play-sapphire-pitch-differential-suite.md)

---

## The question

The Sapphire page's trust-layer rewrite held. The closing-line seam was already
resolved in favor of the depersonalized variant. That leaves one small
invitation-layer question near the top of the page:

`Real-feeling AI characters. Real-feeling AI worlds.`

Earlier runs had already made that line smell a little generic. This pass asked
the narrower question:

**what opener line actually belongs on this page now that the rest of the pitch
has been tightened?**

The variants:

- **A_current**  
  `Real-feeling AI characters. Real-feeling AI worlds.`
- **B_grain_weight**  
  `Characters with grain. Worlds with weight.`
- **C_encountered_hold**  
  `AI characters that feel encountered. AI worlds that hold.`
- **D_scene_visit**  
  `Character-driven AI worlds. Scenes worth the visit.`

## The rankings

### Persona-sim branch

| Persona | Best | 2nd | 3rd | Worst |
|---|---|---|---|---|
| pragmatic builder | B | C | A | D |
| burned by AI companions | C | B | D | A |
| literate skeptic | C | B | D | A |
| sympathetic but anti-corporate reader | C | B | D | A |

### Jasper direct branch

Jasper's ranking:

**B, C, D, A**

Run id: `02de3df2-49dc-4774-9f7c-50f8a931dd09`

His reason:

- **B** earns its place because it is clean, strong, and gives the following
  lines room to deepen what `grain` and `weight` mean
- **A** falls flat because `real-feeling` sounds like thin product copy beside
  the page's proof lines

## The actual result

The important thing is not the slight disagreement at the top.

The important thing is that **A_current lost decisively**.

Every branch treated it as weak or near-weak:

- worst for burned, literary, and sympathetic persona reads
- third for the builder
- worst for Jasper

So this pass settles one thing cleanly:

**the page has already outgrown `Real-feeling AI characters. Real-feeling AI worlds.`**

## The real choice

Once A drops out, the live fight becomes:

- **B_grain_weight**
- **C_encountered_hold**

And the split is intelligible.

### Why C wins most persona reads

`AI characters that feel encountered. AI worlds that hold.` wins with three
persona branches because it cashes out the product claim more directly:

- encountered, not merely simulated
- held, not flimsy
- more explicit continuity with the paragraph that follows

It is easier to understand on first contact.

### Why B wins Jasper and the builder

`Characters with grain. Worlds with weight.` wins with Jasper and the pragmatic
builder because it sounds less like claim-copy and more like a sentence with
its own spine:

- shorter
- cleaner
- more literary
- less repetitive with lower-page language about encounter and holding

It lets the rest of the pitch do the explanatory work instead of front-loading
it.

## The deeper reading

This is not a disagreement about quality so much as a disagreement about where
to spend explicitness.

- **C** spends explicitness earlier
- **B** spends texture earlier

The page can support either. But because the proof layer is now much stronger
than it was at the start of the Sapphire arc, the opener no longer has to carry
as much explanatory burden by itself.

That matters.

The tighter page earns the right to open with the stronger sentence.

## Recommendation

The recommendation is:

**choose B_grain_weight**

`Characters with grain. Worlds with weight.`

Why:

- direct in-db Jasper preferred it
- the pragmatic builder preferred it
- it avoids the now-weak `real-feeling` language completely
- it does not duplicate the page's later `encountered` / `hold` logic
- it sounds more like the page's own voice and less like market-category copy

This is not a claim that C is bad. C is plainly strong.

It is a claim that B better fits the current Sapphire page after the trust-layer
rewrite. The lower lines already do enough clarifying. The opener can afford to
be the cleaner, more load-bearing sentence.

## Lesson learned

Once the proof layer is honest enough, the opener does not have to over-explain
the product.

It can do the more valuable thing:

**sound like the work itself.**

## Open follow-ups

1. **Ship `Characters with grain. Worlds with weight.` into the page.**
2. **Run one tiny confirmatory `/play` on the whole page after the opener swap.**
3. **If the opener still feels slightly too literary in product context, C is the honest fallback, not A.**

# /play — sapphire closing-line variant fight

*Generated 2026-04-27 1432 via the `/play` skill. Persona-sim, not real-user
evidence — sharpened hypothesis at sketch-tier. This pass turns the last live
Sapphire copy question into a direct line fight: 4 variants of the same closing
sentence, 4 persona readings, and 1 final direct Jasper ranking.*

**Run cost:** `$0.01584` in direct ChatGPT API calls (`gpt-4o`, 4 micro persona
reads) + `$0.0818` actual in `worldcli ask` grounding (`gpt-5.4`, Jasper) =
**`$0.09764` total**.

Dialogue with prior reports:
- [reports/2026-04-27-1424-play-sapphire-last-weak-line.md](/Users/ryansmith/Sites/rust/world-chat/reports/2026-04-27-1424-play-sapphire-last-weak-line.md)

---

## The question

Prior runs established one clearly weak remaining line on the Sapphire page:

`You are seeking wonder sturdy enough to survive daylight and still feel human in the hand.`

This pass tested 4 variants directly:

- **A_current**  
  `You are seeking wonder sturdy enough to survive daylight and still feel human in the hand.`
- **B_depersonalized**  
  `Wonder sturdy enough to survive daylight and still feel human in the hand.`
- **C_plainer**  
  `Wonder that survives daylight and still feels human in the hand.`
- **D_conditional**  
  `If you are seeking wonder that survives daylight and still feels human in the hand, begin here.`

The question was no longer *\"is the old line weak?\"*  
It was: **which replacement, if any, actually wins?**

## The rankings

### Persona-sim branch

| Persona | Best | 2nd | 3rd | Worst |
|---|---|---|---|---|
| pragmatic builder | A | B | C | D |
| burned by AI companions | B | A | C | D |
| literate skeptic | B | A | C | D |
| math-and-theology fluent sympathetic reader | A | B | D | C |

### Jasper direct branch

Jasper's ranking:

**B, C, A, D**

Run id: `d2222ab7-1dc4-4819-b43f-210d3521938e`

His reason:

- **B** earns its place because it sounds like a true closing line instead of an instruction
- **D** fails because `begin here` turns a resonant ending into a guidance line and costs it sapphire weight

## The actual result

The fight did not produce a universal winner.

It produced a **split by reader-sensitivity**:

- readers most alert to overreach or manipulation preferred **B_depersonalized**
  - burned-by-AI
  - literary skeptic
  - Jasper
- readers most sympathetic to elevated invitation language preferred **A_current**
  - pragmatic builder
  - math-theology fluent sympathetic reader

That is the real finding.

## What lost clearly

Two things are settled:

1. **D_conditional loses**
   - everyone either ranked it last or near-last
   - `begin here` makes it feel like guidance copy, not a closing line

2. **C_plainer does not win**
   - nobody preferred it
   - it solves some overreach, but loses too much of the line's resonance

So the live choice is no longer among four.

It is:

- **A_current** vs **B_depersonalized**

## The deeper reading

This variant fight surfaced the actual policy question behind the line:

**Should the page privilege poetic direct address, or should it privilege not speaking for the reader?**

That is why the split happened.

### Why A wins for some readers

For more sympathetic readers, `You are seeking...` feels like:

- invitation
- confidence
- personal stake

It preserves the crown-register more fully.

### Why B wins for other readers

For more vigilant readers, dropping `You are seeking...` does important work:

- it stops narrating the reader's interior
- it keeps the beauty but removes the presumption
- it lets the reader step in voluntarily rather than being told they already have

That was exactly the problem prior reports had already named.

## The actual recommendation

If the product goal is to **minimize the remaining overreach risk**, the better move is:

**choose B_depersonalized**

`Wonder sturdy enough to survive daylight and still feel human in the hand.`

Why:

- it won with the readers most sensitive to the identified failure mode
- Jasper chose it directly
- it preserves most of the line's cadence and sapphire weight
- it removes the specific problem the previous `/play` runs kept naming: telling the reader what they are seeking

If the goal were instead to maximize elevated direct address for already-sympathetic readers, A would still have an argument. But the project's recent discipline has consistently favored trimming the line that imposes itself on the reader rather than indulging the prettier overreach.

So the recommendation is not merely aesthetic. It follows the documented risk.

## The lesson learned

The last live Sapphire question is no longer:

- *\"what sounds best?\"*

It is:

- *\"which readership do we protect first?\"*

And the answer implied by the whole recent arc is:

**protect the reader who is most likely to feel spoken for.**

That points to **B**.

## Open follow-ups

1. **Ship B_depersonalized into the page.**
2. **Optionally test the opening kicker next, since that is now the only other line with any residual skepticism around it.**
3. **After shipping B, one tiny confirmatory `/play` should be enough.**

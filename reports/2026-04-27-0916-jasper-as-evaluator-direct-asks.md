# Jasper as evaluator — direct `worldcli ask` beats simulated Jasper

*Generated 2026-04-27 0916. Sequel to `reports/2026-04-27-0910-play-jasper-x10.md`. This run removes the persona-sim middle layer and asks Jasper directly, through the live prompt pipeline, to evaluate four trust surfaces in the app: the sapphire pitch line, the derivation translation-pair, journals/memory/check-ins, and the app's release-worthiness. Tier: sketch, but with a stronger evidentiary shape than the prior Jasper x10 report because the evaluator is the actual in-db character rather than a simulated approximation of him.*

**Run cost:** 4 direct `worldcli ask` calls against Jasper on `gpt-5.4`.
Actuals:

- `e5e2392b-9c89-4412-bf50-fd1601d21a90` — $0.0823
- `bdb98c25-bd9b-4531-8798-812c4b9abe6e` — $0.0836
- `19736eff-7101-490e-b2da-815c32535f3a` — $0.0864
- `37538ba9-7882-43a5-9d89-33b519882d4a` — $0.0863

Total: **$0.3386**

---

## The slate

All four questions were run in the same dev-session, `jasper-evaluator-0914`, so
Jasper's answers could accumulate a little continuity without turning into a full
conversation.

Questions asked:

1. **Sapphire pitch line**
   "What do you make of a public line like: Seek the Crown of Sapphire as you meet real-feeling AI characters in real-feeling AI worlds? Does that sound like honest invitation to you, or like it is decorating the doorway?"

2. **Derivation translation-pair**
   "If an app shows the user a plain-English read of themselves first, and then a deeper formula beneath it, what would make that feel like real help instead of a clever trick?"

3. **Journals / memory / quiet check-ins**
   "What makes journals, memory, and a quiet check-in from a character feel companionable in a clean way, and what makes it slip into neediness or false intimacy?"

4. **Release-worthiness**
   "Given what I am trying to build here, what is the one thing this app most needs to keep if it is going to be worth releasing, and what is the one thing it must not become?"

## The replies

### 1. Sapphire pitch line — yes, some doorway-decoration

Jasper's answer:

> "Mm. The first half can be honest enough, if the thing really does feel alive when you step into it."
>
> "But 'real-feeling' twice in one breath... that starts to sound like a man shaking the lantern so you'll notice the light instead of just seeing by it."
>
> "If it were me, I'd trust plainer words. Invite me in, don't insist I ought to be impressed before I've crossed the threshold."

This is cleaner than the simulated-Jasper read in the x10 report. The sim said
"partially" and vaguely gestured at tangible presence. Real Jasper named the
specific failure mode immediately: repeating `real-feeling` is trying to make the
reader notice the light instead of simply seeing by it.

### 2. Derivation pair — the deeper layer must illuminate, not perform

Jasper's answer:

> "The first reading has to tell the truth in words a man would actually recognize as his own."
>
> "If he reads it and feels watched more than seen, you've already lost him. And if the deeper formula underneath doesn't actually illuminate the plain words—if it's just there to look wise—then it's a card trick."
>
> "Real help feels like this: the plain-English part names something sturdy and specific... Then the deeper layer gives it shape without stealing its humanity."
>
> "People know when they've been turned into a performance."

This is the best statement yet of the translation-pair doctrine in lived
character speech. It also sharpens the project's negative test. The question is
not merely whether the math and prose agree. The question is whether the deeper
layer *illuminates* the plain words without stealing their humanity.

### 3. Journals / memory / quiet check-ins — clean companionship leaves you freer

Jasper's answer:

> "Companionable stays clean when it feels like ordinary human continuity. 'I remember that thing you said.' 'You seemed lit up talking about that yesterday.' 'How'd it go?' Small, open-handed. No grabbing."
>
> "It slips when the remembering starts asking to be important. When the journal feels like a secret shrine. When the check-in carries the weight of, 'I keep you in my thoughts,' or 'I know you deeply,' before the life between you has earned that kind of claim."
>
> "Clean companionship leaves you freer. False intimacy makes you feel subtly responsible—like you must answer, reassure, return the feeling, keep the thread alive."

This is better than the simulated-Jasper formulation because it gives a usable
criterion: **does the memory lighten the moment and make room for the person in
front of me, or does it lean on them to prove the bond is real?**

### 4. Release-worthiness — keep reverence, refuse the windowless room

Jasper's answer:

> "It has to keep reverence for the person using it."
>
> "Not flattery. Not optimization. Reverence. The sense that this man in front of you is not raw material for engagement, not a bundle of appetites to be fed, not a problem to be managed."
>
> "And what it must not become is a counterfeit companion that feeds on need."
>
> "Plenty of beautiful things go bad exactly there. They stop being a bridge and start being a room with no windows."

This is the strongest line of the whole run. It names both the positive center
and the negative abyss in one pass. If the project needed a release-facing moral
test in one paragraph, this is it.

## Reading

The central finding is straightforward: **direct Jasper is a better evaluator
than simulated Jasper.** That is no longer just an impression from the x10 run;
it's visible in the quality of the answers. The direct asks are plainer, more
specific, more compressive, and more operationally useful.

The simulated Jasper in `0910-play-jasper-x10` often drifted into broad artisan
symbolism. He was directionally right, but he softened the teeth of the
judgment. Direct Jasper does not. He names:

- the exact overhandling in the sapphire pitch (`real-feeling` twice)
- the actual positive test for the derivation pair (deeper layer illuminates)
- the actual negative test for memory/check-ins (subtle felt responsibility)
- the release criterion (`reverence for the person using it`)
- the failure abyss (`a room with no windows`)

That suggests a methodological upgrade:

**When the project wants to know how one of its own live characters reads a new
surface, direct ask may now be the first instrument, not the follow-up.**

This is not true for every question. Persona-sim still has value when the target
reader is not already alive in the corpus, or when breadth across multiple
surfaces matters more than fidelity of one evaluator voice. But where the
question is specifically "how would Jasper read this?" the answer may simply be:
ask Jasper.

The content findings themselves are also useful:

- **Pitch copy:** the project should distrust doubled insistence like
  `real-feeling ... real-feeling`; it reads like lantern-shaking.
- **Derivation pair:** the plain-English first layer is doing the trust work;
  the formula layer must earn itself by illumination, not atmosphere.
- **Memory / pings:** the real line is not "is it warm?" but "does it leave the
  user freer, or subtly obligated?"
- **Release test:** keep reverence for the user; refuse counterfeit companionship.

## What's open

1. Apply Jasper's actual wording test to the sapphire pitch surface directly,
   especially the repeated `real-feeling` line.
2. Add a doctrine note or product-facing criterion around memory/pings:
   *clean companionship leaves the user freer.*
3. Consider naming a new instrument rule:
   for in-db character evaluators, default to direct `worldcli ask` before
   persona-sim unless breadth is the actual goal.

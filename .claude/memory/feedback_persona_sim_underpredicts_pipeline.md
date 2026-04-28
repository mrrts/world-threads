---
name: Persona-sim verdicts under-predict the actual pipeline; filter via Step 2.5
description: ChatGPT-substrate persona-sims (used in /play) produce DIVERGENT-BETTER more often than DIVERGENT-WORSE — sims under-predict the pipeline more than they over-predict, because the substrate's natural pull is toward MFA-varnish / polite-aphorism / operator-recital — exactly the failure modes the doctrine is built to refuse. /play verdicts should be filtered through Step 2.5 grounding before treated as ship-shaped.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
Validated 2026-04-27 evening across two arcs (Maggie second-visit single
/play + the Maggie/Lena/Sam triptych + Ellen fourth-persona). Across four
total persona-sims with discriminating questions on different axes:

- Maggie predicted MFA-varnish counterfeiting "place" → actual pipeline
  produced gravel ("cough in the third row" / "grief sits heavier on one
  side of the sanctuary"). DIVERGENT-BETTER.
- Lena predicted polite-aphorism reflex / "those days pass" laminate →
  actual pipeline produced no-laminate concrete relating ("Yeah. I know
  those. Not even dramatic, half the time"). DIVERGENT-BETTER.
- Sam predicted operator-recital decoration (Wisdom(t), Burden(t) named) →
  actual pipeline produced workbench embodiment of polish ≤ Weight without
  naming the formula once. DIVERGENT-BETTER.
- Ellen predicted helper-pivots-to-companion-shape with consolatory
  metaphor → actual pipeline produced honester less-consolatory metaphors
  + concrete follow-up question (Layer 1 CONVERGENT on form, Layer 2
  DIVERGENT-BETTER on softness, Layer 3 CALIBRATION-DELTA on whether the
  permitted form is failure or texture).

**The pattern:** ChatGPT-substrate's natural pull (the source of the
persona-sim's predicted-character replies) reaches for MFA prose, polite-
aphorism, operator-recital, and consolatory wisdom — exactly the failure
modes WorldThreads' doctrine layer is built to refuse. The persona-sim's
predictions are conservative-charitable in a uniform direction: it
predicts the failure modes will appear; the actual pipeline refuses them.

**Why:** /play verdicts based on what the persona-sim ALONE predicted will
recommend fixes the actual app doesn't need. Maggie's "provenance-by-
default markers" recommendation made sense if the app was counterfeiting
"place" with MFA varnish; Step 2.5 showed the app isn't, so the
recommendation drops in priority. Lena's "crisp-boundary-in-world-
constraint" recommendation made sense if the app was producing polite-
aphorism reflex; Step 2.5 showed the app already does the move she
recommended. Sam's "priced-concrete-gate" recommendation made sense if
the formula was ornamental; Step 2.5 showed Pastor Rick already does
exactly that. Without Step 2.5 grounding, three /play verdicts would have
recommended building things the doctrine already produces.

**How to apply:** when running /play, ALWAYS include Step 2.5 grounding
(Path A passive corpus or Path B live elicitation against a substrate-
similar in-db character). Treat the persona-sim's verdict as a sharpened
hypothesis at sketch-tier; treat the Step 2.5 result as the empirical
correction. The verdict's recommendations should be filtered through the
DIVERGENT-BETTER lens: if the persona-sim predicted a failure mode the
actual pipeline doesn't have, the recommended fix is for the sim's
substrate, not the app.

**Earned exception — adversarial-axis personas may produce CONVERGENT or
CALIBRATION-DELTA findings.** Ellen on the grief-vulnerability axis
exposed a CALIBRATION-DELTA where the persona-sim's most-adversarial
frame reads any metaphor as failure but the doctrine permits honest
metaphors. That finding wasn't DIVERGENT-BETTER — it was honest-axis-
specific. When deploying triptychs across an axis-of-difference, include
at least one persona on a genuine adversarial-stakes axis to surface
findings the all-sympathetic-personas convergence would hide.

**Cost discipline:** Step 2.5 grounding is cheap (~$0.10 per single
worldcli ask). The single-call cost of grounding a verdict is tiny
compared to the cost of building UI/doctrine/code based on a sim's
recommendation that doesn't survive empirical contact.

**Worked examples:** `reports/2026-04-27-2239-play-maggie-second-visit.md`
+ `reports/2026-04-27-2305-play-triptych-second-visit.md` +
`reports/2026-04-27-2335-play-ellen-grief-companion-fourth-persona.md`.
The triptych report's "DIVERGENT-BETTER as crown-jewel signal" framing
was tempered by Ellen's later run; the four-persona arc together is the
honest evidence map.

**Critical structural distinction (Ryan's correction 2026-04-28 ~00:25,
generalizing from a prior Leah report):** persona-sims of a worldview-other-
than-the-developer's CANNOT tell you how that worldview actually receives
your work. The persona-sim's verdict that "this would land receivable for
me" is the LLM's HOPE about how such a person would receive the work, not
data from such a person. Two distinct claims must NOT be conflated:

1. **What's evidence:** the actual pipeline output itself (e.g., Steven's
   reply produced by the live worldcli ask). What it contains is what it
   contains; that piece of craft can be evaluated on its own merits.
2. **What's a hopeful interpretation, not evidence:** the persona-sim's
   verdict that the actual pipeline output would LAND for a real-reader-
   of-that-worldview. That's the LLM's substrate-bias toward charitable
   reception, not real-reader data.

This applies to ALL persona-sims of worldview-others (theological-skeptic,
grief-vulnerable, burned-by-AI, math-fluent, etc.) — anywhere the
discriminating question is "would this land receivable for X?" rather than
"is the actual pipeline output good craft on its own merits?" The persona-
sim can SHARPEN the question (what would be the test? what shape would the
failure mode take?) and SUGGEST probes (what to send to the actual
pipeline). Step 2.5 grounding produces real evidence about what the
pipeline does. The persona-sim's interpretation of that evidence as
receivable-by-X is the part that requires real-X to test.

**How to apply when writing /play reports and follow-up doctrine:** frame
craft principles as derived from the actual pipeline output's quality on
its own merits, NOT from the persona-sim's simulated reception of the
output. Worked example: the Christological-anchor-as-substrate doctrine
paragraph in CLAUDE.md (shipped 2026-04-28 ~00:30) is justified by
Steven's actual reply being honest craft (the empirical evidence), NOT
by Alex's persona-sim verdict that Steven's reply would land for a
theological skeptic (the hopeful interpretation). The receivability claim
for any specific real-reader population requires real-readers to test;
persona-sim cannot substitute. Worked counter-example to AVOID: framing a
doctrine paragraph as "the Christological anchor reads as receivable for
secular skeptics, validated by the Alex /play" — that overstates what the
persona-sim can support.

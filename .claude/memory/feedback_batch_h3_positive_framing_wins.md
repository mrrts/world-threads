---
name: batch-hypotheses h3-positive-framing pattern wins
description: Across the first three uses of the batch-hypotheses skill (chat 2026-04-26 ~20:30/20:50/etc.), positive-framed phrasings with a memorable metaphor consistently outperformed negation-shaped or checklist-shaped phrasings on rule-bite + voice-preservation. Bias future hypothesis-drafting toward positive-frame candidates.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
When using the batch-hypotheses skill (`.claude/skills/batch-hypotheses/SKILL.md`,
shipped 2026-04-26 ~20:25), draft hypotheses with a bias toward POSITIVE-FRAMED
phrasings backed by a memorable metaphor. Pattern observed across two
worked uses tonight, both selected h3 (positive-frame-metaphor):

1. **Prop-density clause** (report `2026-04-26-2030`): h3 (one-true-thing
   positive frame) won over h1 (numerical cap), h2 (mistrust diagnostic
   lifted), h4 (decorating-doorway metaphor — note this is also positive
   but more cautionary), h5 (count-and-diagnosis combined).
2. **Scene-driving clause** (report `2026-04-26-2050`): h3 (scene-as-bridge
   metaphor) won over h1 (diagnostic), h2 (active-verb cap), h4 (worked-
   example-contrast), h5 (decide-do-or-redirect rule).

**The shared shape that wins:** positive-framed (says what to AIM AT, not
what to AVOID), backed by a metaphor or principle the model can carry
INTO the prompt-stack without leaking the metaphor's surface vocabulary
INTO the reply. Negation-framed ("DON'T") and checklist-shape rules tend
to feel rule-pressured at the surface — the model produces compliant
output but the rule shows through as a tactic.

**Why this matters for drafting:** when proposing 5 candidate phrasings
for a new rule via batch-hypotheses, default to including:
- 2-3 POSITIVE-FRAMED candidates (varying metaphors / aims / discriminations)
- 1-2 NEGATION/CHECKLIST candidates (as control points the positive ones
  outperform)

Don't waste the budget proposing 5 negation-shaped candidates and then
testing which "DON'T do X" phrasing is least bad. The pattern is real
enough at sketch-tier (N=2 batches) to bias drafting now; characterized-
tier confirmation would need 3+ more batches showing the same pattern.

**Also aligns with `feedback_preference_not_commanded`** (the
preference-shaped over commanded register doctrine) and CLAUDE.md's
craft-note-bite-verification "match the rubric shape to the bite shape"
discipline — positive-framing matches positive-aim measurement better
than negation matches absence-measurement.

**Caveat:** if the rule's failure mode is fundamentally categorical (a
hard ban with no positive aim equivalent), positive-framing won't apply.
But for craft-shape rules — voice, register, motion, density — positive-
frame-with-metaphor is the empirical winner so far.

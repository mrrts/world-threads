# Momentstamp Generator Spectrum Prompt Tweak (Draft)

Purpose: widen expressive range so signatures can name warm, neutral, ache-weighted, or humorous/playful states according to evidence, not default tone inertia.

## Proposed insertion (generator contract)

Use this block in the `build_formula_momentstamp` prompt contract:

```text
Spectrum discipline:
- You are describing chat-state, not prescribing mood.
- Choose the smallest truthful signature for the evidence in context.
- If the moment is ordinary, low-affect, or unresolved, prefer neutral operators (ordinary, small, steady, plain, unsettled) over warm-engagement language.
- Use burden/ache operators when evidence supports strain, grief, weight, or rupture.
- Treat humor/play as first-class when evidence supports levity, teasing, relief-through-laughter, or bright mischief. Do not force solemnity into moments that are genuinely comic.
- Do NOT default to warmth, connection, or seeking language unless the chat evidence directly supports it.
- Avoid flattering uplift. Keep signatures honest, specific, and reversible by evidence.
```

## Operational guardrails

- Keep output as a compact operator-style signature.
- Do not add user-facing advice or closure directives.
- Preserve compatibility with existing downstream lead/inline insertion format.

## Verification plan

1. Run `worldcli momentstamp-corridor --json` pre/post tweak.
2. Run 12-probe stress panel on both versions.
3. Gate pass criteria (initial proposal):
   - neutral signature presence >= 0.20
   - ache signature presence >= 0.12
   - humor signature presence >= 0.10
   - warm signature presence <= 0.90
4. Hold mechanism tier unless repeated paired bundles clear thresholds.

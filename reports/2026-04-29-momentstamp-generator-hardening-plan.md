# Momentstamp Generator Hardening Plan (No-Code Proposal)

## Why this exists

Pinned-signature follow-up reframed the mechanism: lead primacy appears to amplify signature content rather than create curiosity behavior by itself. That means `build_formula_momentstamp` vocabulary selection is load-bearing and may carry implicit prescription while presenting as description.

## Objective

Reduce unintended operator-vocabulary bias in generated `formula_signature` values while preserving useful chat-state signal.

## Non-goals

- No rollback of lead-block wiring.
- No removal of inline series / chain handoff paths.
- No immediate production code changes in this document.

## Hardening dimensions

1. **Lexical neutrality calibration**
- Build a tracked lexicon split: neutral state terms vs curiosity-tilted terms.
- Measure signature corpus rates per class and per character.
- Target: curiosity-tilted terms appear when warranted, not as a default baseline.

2. **Generator variance control**
- Characterize temp sensitivity (`temp=0.6` currently) against lexical drift.
- Evaluate whether lower-temp or constrained decoding improves descriptive stability without flattening state signal.

3. **Prompt-contract tightening**
- Rewrite generator prompt contract to explicitly separate:
  - state description,
  - action recommendation,
  - user-directed closure cues.
- Ban recommendation language in signature output unless separately requested.

4. **Evaluation harness requirements**
- Require paired cells:
  - organic signature,
  - pinned-neutral signature,
  - pinned-curiosity signature.
- Any future claim about lead effects must pass content-held controls.

## Proposed acceptance gates

- **Gate A (distribution):** curiosity-tilted token rate stays within defined corridor over rolling 50+ signatures.
- **Gate B (mechanism):** pinned-neutral cells do not show spurious curiosity lift; pinned-curiosity cells do show expected amplification.
- **Gate C (stability):** rerun variance across N=3 pairs remains directionally stable under same setup.

## Immediate operational steps

1. Run `worldcli momentstamp-vocab` on current corpus and record baseline rates.
2. Use `scripts/run-momentstamp-ablation-matrix.sh` across at least three probes (introspective, factual-closed, technical-neutral).
3. Record results in an experiment file with explicit tier labels (sketch / claim / characterized).

## Decision discipline

Until Gate A/B/C pass, treat "lead improves curiosity-back" as interaction-sensitive and context-bound, not a global claim.

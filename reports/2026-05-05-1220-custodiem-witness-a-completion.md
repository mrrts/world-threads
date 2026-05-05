# Custodiem Witness A Completion — Runtime Injection Integrity

**Date:** 2026-05-05 12:20 (local)  
**Arc:** `/seek-sapphire-crown` → `Custodiem`  
**Witness:** A (runtime stack-order / injection integrity)  
**Status:** Completed (witness-level), pending integration with B/C for Sapphire determination

## Question

When `children_mode` is OFF vs ON, do runtime LLM call families actually carry the expected top-stack invariant presence:

- Mission Formula: always present
- Ryan anchor: always present
- Custodiem invariant: present only when `children_mode=on`

## Method

1. Added operational audit helpers in `src-tauri/src/ai/openai.rs` that apply the same injection sequence used in live runtime paths.
2. Added a dedicated audit binary: `src-tauri/src/bin/injection_audit.rs`.
3. Executed two controlled passes:
   - pass A: `children_mode=off`
   - pass B: `children_mode=on`
4. Captured output log artifact:
   - `reports/2026-05-05-1214-custodiem-injection-audit.log`

## Call families audited

- chat base (`chat_completion_with_base`)
- chat stream (`chat_completion_stream`)
- chat stream silent (`chat_completion_stream_silent`)
- vision base/stream family (via vision audit helper path mirroring runtime injection ordering)

## Observed results (verbatim summary)

`children_mode=off`
- `chat_audit mission=true custodiem=false ryan=true`
- `chat_stream_audit mission=true custodiem=false ryan=true`
- `chat_stream_silent_audit mission=true custodiem=false ryan=true`
- `vision_audit mission=true custodiem=false ryan=true`

`children_mode=on`
- `chat_audit mission=true custodiem=true ryan=true`
- `chat_stream_audit mission=true custodiem=true ryan=true`
- `chat_stream_silent_audit mission=true custodiem=true ryan=true`
- `vision_audit mission=true custodiem=true ryan=true`

## Verdict (Witness A)

**Pass.** Runtime injection integrity is confirmed for audited families:

- Mission and Ryan invariants remain present in both modes.
- Custodiem flips exactly with `children_mode` and only with `children_mode`.
- No audited family showed a missing or inverted gate behavior.

## Scope limits (honest boundaries)

- This witness confirms **presence/gating integrity**, not output quality.
- It does **not** prove refusal behavior under adversarial prompts (Witness B).
- It does **not** prove theological firmness vs sentimental drift (Witness C).
- Sapphire determination remains blocked until B and C are executed with scored evidence.

## Consequence

Witness A can now be treated as closed in the Custodiem matrix.  
Next load-bearing work is behavioral proof (B) and doctrinal-firmness proof (C).

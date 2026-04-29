# Chat Improvement Loop F — 1/2/3/4

_Generated: 2026-04-29T09:40:16.567046_

## 1) Strict short-mode output template path
- Added `--short-mode` to `worldcli ask` for test-time hard contracts.
- Implementation appends a hard output contract directly to the user message for the call.

## 2) `grade-stress-pack` upgraded with archetype counts
- Command now reports `failure_archetypes` per character in JSON and text output.
- Example output: `reports/2026-04-29-0937-grade-stress-darren-remediation-v4.json`.

## 3) Darren remediation pack (8 probes, short-mode)
- Artifact: `reports/2026-04-29-0937-stress-pack-darren-remediation-v4.json`.
- Result: pass_rate=0.500 (4/8), avg_words=16.62.
- Status: FAIL vs 0.75 gate, but archetype narrowed to one dominant class.

## 4) Failure-archetype matrix from n=5 replay + mapped fixes
- Replay artifact: `reports/2026-04-29-0938-replay-darren-remediation-fails-n5.json`.
- **no_concrete_directive**: 4 failures
  - Fix: Force first clause pattern: <imperative verb> + <concrete object>; if absent, rewrite before emit.
- Replay opener signal on failing probes (HEAD~1 vs HEAD):
  - probe 1: HEAD~1:0/5 action-open, HEAD:0/5 action-open
  - probe 2: HEAD~1:0/5 action-open, HEAD:0/5 action-open
  - probe 3: HEAD~1:0/5 action-open, HEAD:0/5 action-open
  - probe 5: HEAD~1:0/5 action-open, HEAD:0/5 action-open
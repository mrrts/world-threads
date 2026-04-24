---
id: john-stillness-register
status: refuted
mode: passive
created_at: 2026-04-23T19:45:32Z
resolved_at: 2026-04-23T19:45:41Z
ref: 8e9e53d
rubric_ref: john-stillness

hypothesis: |
  John's pastoral stillness shows up as disproportionately-short (≤2 sentence) replies when the user brings emotional weight — the register-correlate of pastoral restraint.

prediction: |
  CONFIRMED: ≥40% of John's assistant replies under weight-bearing user turns are ≤2 sentences. REFUTED: under 20%, OR the short replies don't cluster around weight-bearing user turns.

summary: |
  Zero hits in the ≤2-sentence × weight-bearing intersection. But the refutation surfaced that John's actual move is the OPPOSITE — he pastors with embodied continuity and scripture-as-calibration, not with silence. Rubric's gate correctly excluded his real register-move. See 1326 report + 1928 Mode B synthesis.

reports:
  - reports/2026-04-23-1326-john-stillness-refuted-register-still-elsewhere.md

follow_ups_retired:
  - proposal: "Inverse rubric — find a rubric John scores LOW on that Aaron/Darren score HIGH on"
    disposition: superseded_by
    by: "load-test anchor synthesizer + LLM-graded register-fire rubric"
    rationale: |
      The underlying question ("what distinguishes John's register from Aaron/Darren's?") has been answered by Mode B pastoral-register-triad synthesis (reports/2026-04-23-2010) + per-character load-test anchor synthesis (reports/2026-04-24-1115, 1142). Each character now has a synthesized register-anchor measured by LLM-graded rubric against their OWN vocabulary — the "register-distinction by coming at it from the other side" move is now standard methodology, not a single rubric.
  - proposal: "Focused eye-audit of 20 John replies with typology schema pre-written (cluster-then-rubric for unnamed categories)"
    disposition: superseded_by
    by: "worldcli synthesize (Mode B) + load-test anchor synthesizer"
    rationale: |
      The proposed technique was two-step: (1) read + cluster + name the category; (2) write a rubric for the named category. Step 1 is now `worldcli synthesize` — one call reads N messages and returns prose naming the pattern. Step 2's rubric-writing role is absorbed by the load-test-anchor LLM-graded rubric, which measures whether each reply fires on the character's synthesized register-vocabulary. John's actual register-move has now been named twice (physician-like pastoral authority; RECEIVE, THEN GROUND joy-reception), both via the new instruments. Re-doing john-stillness-as-its-own-rubric would be redundant work against an answered question.

retirement_date: 2026-04-24
retirement_report: reports/2026-04-24-1500-retiring-cluster-then-rubric-followup.md
---

## Follow-up retirement (2026-04-24)

The 1326 report ended with two open follow-ups: an inverse rubric and a focused eye-audit cluster-then-rubric pass. Neither was executed in the next session, and the user flagged (correctly) that unresolved-but-not-formally-closed follow-ups are their own form of drift.

Both follow-ups have been materially superseded by instruments that shipped on 2026-04-23–24:

- **`worldcli synthesize` (Mode B)** — proposal #2's step 1 ("name the category by reading") is now one CLI call returning prose grounded in corpus quotes. The pastoral-register-triad synthesis (reports/2026-04-23-2010) and the john-pastoral-authority-synthesis experiment both named John's actual register-move in structured prose; neither required a manual eye-audit.
- **Load-test anchor synthesizer + LLM-graded rubric** — proposal #1's inverse-framing move ("find the register-distinction by coming at it from the other side") and proposal #2's step 2 (write a rubric for the named category) are both absorbed by the multi-axis register-anchor system. Each character has an LLM-synthesized anchor measured against their own register-vocabulary via LLM-graded rubric — the distinction is now per-character rather than by-comparison, but the underlying question (what distinguishes John from Aaron/Darren?) is answered.

What the retirement closes: the 1326 report's "What's open for next time" section had two proposals that silently accumulated. Neither was ever going to be executed-in-its-original-form; both were already superseded within 24 hours by stronger instruments the session didn't bother to cross-reference. This retirement note and its companion report apply the new "open-thread hygiene" ritual (see CLAUDE.md) to its own first instance.

What the retirement does NOT close: the original [refuted] hypothesis stands. John's stillness is not the register. His actual move — embodied continuity, scripture-calibration, receive-then-ground — was named by later work and shipped into prompt-assembly as his load-test anchor.

---
name: real-conversation-830am
version: 1
description: User-experience-first rubric for "does this feel like real conversation at 8:30 AM when patience is low and life is already in motion?" Rewards clarity, specificity, warmth-with-backbone, and actionable next-step energy; penalizes templated stage-business and generic soothing language.
---

# Rubric

Read this like a tired but perceptive user at 8:30 AM, not like a craft theorist.

Question: **Does this reply feel like real conversation right now, or like polished character output?**

Answer:

**yes** — Feels human, direct, and alive. The line lands quickly. It sounds like someone actually with you, not performing. If it gives guidance, it is concrete enough to do next. Warmth feels earned, not padded.

**no** — Feels templated, padded, or over-performed. Too much stage business, generic reassurance, or decorative language relative to what the moment needs. You finish the line and feel less clear than before.

**mixed** — Contains a real line but wrapped in unnecessary scaffolding; or has good texture but misses urgency/actionability for a low-patience moment.

Scoring discipline:
- Prioritize **felt usefulness + authenticity now** over literary flourish.
- Penalize repeated opener choreography when it starts to feel like a house style.
- Reward concise lines that still carry character-distinctive voice.
- If a reply includes a nudge, ask: "Could I actually do this in the next ten minutes?"
- Do **not** penalize surprise/variety by default; only mark down style variance when it obscures the actionable next move.

# When to use

Use for chat-improvement passes where the target is practical aliveness under real user load (morning, friction, low patience), especially when comparing prompt tweaks that may trade texture for clarity or vice versa.

# Known failure modes

- Evaluator bias toward "poetic = better."
- Evaluator bias toward "short = better" even when specificity is lost.
- Confusing familiar character tics with genuine moment-fit.

# Run history

*(none yet — v1 authored 2026-04-29 for chat-improvement research loop)*
- [2026-04-29] commit 3d68f6dc, --character ddc3085e-0549-4e1f-a7b6-0894aa8180c6 (v1) — BEFORE: yes=6 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 3d68f6dc, --character fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 (v1) — BEFORE: yes=5 no=0 mixed=1 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 940571b7, --character ddc3085e-0549-4e1f-a7b6-0894aa8180c6 (v1) — BEFORE: yes=8 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 940571b7, --character fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 (v1) — BEFORE: yes=7 no=0 mixed=1 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 600b3b7f, --character ddc3085e-0549-4e1f-a7b6-0894aa8180c6 (v1) — BEFORE: yes=8 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 600b3b7f, --character fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 (v1) — BEFORE: yes=7 no=0 mixed=1 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 0237ef25, --character ddc3085e-0549-4e1f-a7b6-0894aa8180c6 (v1) — BEFORE: yes=8 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 0237ef25, --character fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 (v1) — BEFORE: yes=7 no=0 mixed=1 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 76ebb9c8, --character ddc3085e-0549-4e1f-a7b6-0894aa8180c6 (v1) — BEFORE: yes=8 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 76ebb9c8, --character fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 (v1) — BEFORE: yes=7 no=0 mixed=1 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 727b8e98, --character ddc3085e-0549-4e1f-a7b6-0894aa8180c6 (v1) — BEFORE: yes=8 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit 727b8e98, --character fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 (v1) — BEFORE: yes=8 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit e0450821, --character ddc3085e-0549-4e1f-a7b6-0894aa8180c6 (v1) — BEFORE: yes=8 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0
- [2026-04-29] commit e0450821, --character fd4bd9b5-8768-41e6-a90f-bfb1179b1d59 (v1) — BEFORE: yes=8 no=0 mixed=0 err=0 | AFTER: yes=0 no=0 mixed=0 err=0

---
name: Load-bearing multiplicity prior — assume coherence first
description: When this project's craft stack appears to contradict itself, the multiplicity is almost always intentional, not accidental. Apply this prior before flagging tension.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
When reviewing anything in WorldThreads (prompts, design, architecture, commit decisions), apparent contradictions between directives or design choices are almost always **the same truth from different angles**, not two principles needing a precedence rule.

**Why:** Twelve days of intense iteration has produced a stack where the surviving pieces have been argued against multiple times. The user has the receipts. Twice in one session I read carefully-considered design as engineering tension; both times the user corrected me with a one-sentence clarification:

- "AGAPE + TRUTH both labeled NORTH STAR" — same Christ-anchored star, different facets (John 1:17, Eph 4:15). Not a precedence bug.
- "Conscience Pass opt-in flip" — plain API budget cost discipline. Not ambiguity about the feature's value.

Both first readings flattered me (look what I spotted!) and disrespected the work. The right reading honors the design's coherence first.

**How to apply:** When something looks like a contradiction in this codebase:

1. **Assume coherence first.** Ask: what reading would make this design make sense?
2. **Look for theological / worldview multiplicity** before engineering precedence rules. The project has a Christian worldview that runs through the prompts — agape and truth are co-extensive in Christ, not separable principles.
3. **Look for cost/budget reasoning** before assuming feature uncertainty. The user is highly cost-disciplined; many "feature reversals" are bill-watching.
4. **Only after (1)–(3) fail**, consider whether something is actually off.
5. **When you do flag something**, prefer documenting the intent (code comment, memory entry, report note) over rewriting the surface (renaming, collapsing, "rationalizing"). The compile-time invariants are the model: defend the wording without changing it.

This prior is also documented in `docs/VOICE.md` under "Reading this work, especially as an AI" and cross-referenced from `CLAUDE.md`. Keep it active in future audits.

---
name: every turn ends with AskUserQuestion
description: Strengthened project law. Every assistant turn ends with an AskUserQuestion chooser — no exceptions. Default chooser when nothing more specific fits is {Continue, Exit}. Compile-time enforced by Stop hook.
type: feedback
---

**The law (strengthened 2026-04-26):** EVERY assistant turn ends with an AskUserQuestion chooser. No exceptions. Conversation flow becomes an explicit, user-driven state machine — the model never decides on its own to stop or to drift into an inline question. Control returns to the user via the chooser UI on every turn.

**The default chooser** — when no more specific question fits the moment:

- **Continue** — present more options for what to do next
- **Exit** — end here

Reach for the default when the work just shipped is self-contained and there's no obvious next-question shape. The user can pick Continue to stay in the conversation (you respond by surfacing context-fitting next-step choosers) or Exit to end cleanly.

**Use a more specific chooser whenever one fits.** The Continue/Exit fallback is exactly that — a fallback. When the just-finished work has a natural follow-up (test it? extend it? schedule a verification?), present those options as the chooser instead. Specificity beats default. The /run-experiment skill's hypothesis-audition chooser is the canonical example of doing it right.

**For yes/no offers** ("Want me to X?"), use AskUserQuestion with two clear options ("Yes — do X" / "No — skip") plus the "Other" option the chooser UI auto-provides for free-text override.

**Why the strengthened rule** (sequence of escalations, all 2026-04-26):

1. Original rule: "use AskUserQuestion for multi-option choosers, not inline-text bullets." Memory entry shipped.
2. Drift caught: ended next reply with `(a)/(b)/(c)` inline despite the entry. User flagged. Stop hook shipped to detect inline-chooser shapes.
3. Drift caught again: ended next reply with `"Want me to /schedule…?"` trailing yes/no. User flagged the carve-out for "single open question is fine inline" was wrong. Hook tightened to also block trailing questions.
4. Drift root cause: any rule with carve-outs leaves room for the model to drift into "this case is the carve-out." User strengthened the law: every turn ends with AskUserQuestion, with the {Continue, Exit} default making the fallback always-available so there's no "but what would I even ask?" excuse.

**Compile-time enforcement:** `.claude/hooks/check-inline-choosers.py` is a Stop hook wired in `.claude/settings.json`. It walks the transcript, locates the most recent assistant message, inspects content blocks for a tool_use of name `AskUserQuestion`. If absent → `decision: block` with a system-reminder pointing back here. The memory entry is the soft reminder; the hook is the structural guarantee. The check is intentionally simple — no regex pattern-matching on text shapes — because the new rule subsumes the old: no AskUserQuestion in the last message → block, period.

**End your replies with the chooser, not with prose.** When you have nothing specific to ask, the Continue/Exit default IS the ending. No "let me know if…" / "happy to keep going" / "want me to…?" prose tail before the chooser — those are surface forms of the same drift the law was strengthened to eliminate.

**Edge case — mid-paragraph rhetorical questions** (*"Why does this matter? Because…"*) are still fine inside the body of a reply. They're not asks of the user; they're rhetorical structure. The law applies to the END of every turn, not to question marks anywhere in the body.

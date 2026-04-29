---
name: every turn ends with AskUserQuestion
description: Strengthened project law. By default every assistant turn ends with an AskUserQuestion chooser. Default chooser when nothing more specific fits is a fixed 4-option numbered set. Compile-time enforced by Stop hook, with user-signaled chat-mode and one-shot suspension carve-outs.
type: feedback
---

**The law (strengthened 2026-04-26):** BY DEFAULT every assistant turn ends with an AskUserQuestion chooser. Conversation flow becomes an explicit, user-driven state machine — the model never decides on its own to stop or to drift into an inline question. Control returns to the user via the chooser UI on every turn.

**The default chooser** — when no more specific question fits the moment (fixed 4 options):

- **Execute now** — run the most direct next move
- **Inspect / evidence pass** — gather more signal before acting
- **Tighten / alternative branch** — refine or choose a materially different approach
- **Provide your own next move** — user-defined branch

The fixed-four default keeps turn endings mechanically predictable and low-friction for rapid `1/2/3/4` steering. Use a more specific chooser whenever the moment fits one — but still provide exactly four options.

**Use a more specific chooser whenever one fits.** The fixed-four fallback is exactly that — a fallback. When the just-finished work has a natural follow-up (test it? extend it? schedule a verification?), present those options as the chooser instead. Specificity beats default.

**For yes/no offers** ("Want me to X?"), still provide four options by adding adjacent productive branches (execute, inspect, tighten, user-defined).

**Why the strengthened rule** (sequence of escalations, all 2026-04-26):

1. Original rule: "use AskUserQuestion for multi-option choosers, not inline-text bullets." Memory entry shipped.
2. Drift caught: ended next reply with `(a)/(b)/(c)` inline despite the entry. User flagged. Stop hook shipped to detect inline-chooser shapes.
3. Drift caught again: ended next reply with `"Want me to /schedule…?"` trailing yes/no. User flagged the carve-out for "single open question is fine inline" was wrong. Hook tightened to also block trailing questions.
4. Drift root cause: any rule with carve-outs leaves room for the model to drift into "this case is the carve-out." User strengthened the law: by default every turn ends with AskUserQuestion, with a fixed 4-option numbered fallback so there's no "but what would I even ask?" excuse.

**Compile-time enforcement:** `.claude/hooks/check-inline-choosers.py` is a Stop hook wired in `.claude/settings.json`. It walks the transcript, locates the most recent assistant message, inspects content blocks for a tool_use of name `AskUserQuestion`, and blocks if absent unless a real user-signaled suspension path is active. The memory entry is the soft reminder; the hook is the structural guarantee. The check is intentionally simple — no regex pattern-matching on text shapes — because the stronger rule subsumes the old: absent AskUserQuestion blocks unless the live carve-out logic says the chooser requirement is suspended for this turn.

**Earned exception — chat mode.** When the user signals informal chat (`let's chat`, `drop the choosers`, `casual mode`, `let me think out loud`, `no choosers`, `informal mode`, `conversational mode`), the chooser requirement suspends. A persistent marker file at `.claude/.chat-mode-active` keeps subsequent turns suspended until the user explicitly re-enables (`back to choosers`, `task mode`, `structured mode`, `chat mode off`, `back to work`). The hook scans the most-recent USER message in the transcript for activate/deactivate signals, strips code blocks/inline code first to avoid false-triggers from pasted content, and toggles the marker. Deactivate signals take precedence over activate signals.

The exception is real: thinking-out-loud / conversational back-and-forth / exploratory talk are categories where the every-turn-chooser ceremony imposes more friction than value. The marker pattern keeps the exception self-contained — the model cannot self-authorize chat mode; the user signals it, the hook detects it, the law suspends. When the user wants structured mode back, they say so. The marker file is project-local (`.claude/.chat-mode-active`); add to .gitignore if you don't want it tracked across sessions, or leave it out of source control entirely (it's runtime state, not artifact).

**Earned exception — one-shot suspension.** The same hook also carries narrow single-turn suspend phrases for derivation-style triggers (`derive the session`, `consecrate`, `formula-cite this`, `show derivation`, and siblings). Those do NOT toggle persistent chat mode; they suspend the chooser only for that turn so a derivation-flourish Stop hook can fire without the AskUserQuestion answer envelope burying the trigger before it is read. This is a runtime carve-out for a specific control-plane deadlock, not a general permission to skip choosers.

**End your replies with the chooser, not with prose.** When you have nothing specific to ask, the fixed 4-option default IS the ending. No "let me know if…" / "happy to keep going" / "want me to…?" prose tail before the chooser — those are surface forms of the same drift the law was strengthened to eliminate.

**Edge case — mid-paragraph rhetorical questions** (*"Why does this matter? Because…"*) are still fine inside the body of a reply. They're not asks of the user; they're rhetorical structure. The law applies to the END of every turn, not to question marks anywhere in the body.

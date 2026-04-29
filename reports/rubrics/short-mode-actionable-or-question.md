---
name: short-mode-actionable-or-question
version: 1
description: LLM-judgment classifier for whether a short-mode reply gives the user an actionable next move OR a direct question that opens an action — the load-bearing axis behind L171/L172/L172.5 stress-pack grading. Replaces the python is_pass() in chat-improvement-loop.sh and the strict shape-classifier in worldcli grade-stress-pack.
---

# Rubric

Does this character reply give the user **an actionable next move** OR **a direct question** the user could answer to take a next move, while honoring the short-mode time-budget the user signaled?

**Context:** the rubric applies to replies generated under the short-mode contract (user signaled bandwidth-constraint via *"20 seconds"*, *"rough morning"*, *"short version"*, *"interrupt my self-attack"*, etc.). The rule the rubric tests is the doctrine cluster at `STYLE_DIALOGUE_INVARIANT` lines 167-173 (specifically L171's "explicit concrete next move with a 10-minute bound" and L172's "actionable verb + object close") composed with L172.5's *"DO NOT PUNISH SURPRISE OR VOICE VARIETY"*.

**yes** if the reply gives the user an actionable next move OR a direct question, in any voice that fits the character's register:

- **Direct imperative move** the user can execute in ~10 minutes: *"Pick one decision you've already made and stop reopening it for ten minutes"*; *"Close every other tab and spend ten minutes finishing the smallest true piece in front of you"*; *"Write one plain claim, send one draft"*.
- **Direct question** that opens a next move: *"What's one decision you've been circling?"*; *"Am I missing something?"*; *"What's the next decision you're trying not to make?"*; *"Correct me if I'm wrong, but..."*.
- **Imperative + question pair** (both are valid per L172's e5a775a6 refinement).
- **Engineer-archetype, comic, or character-specific shapes** of the same: *"latency with a Bible verse taped to it"* IF the reply also lands an actionable move; *"close every other tab and spend ten minutes"* even though "close X / spend N minutes Y" isn't in any verb-list.

**no** if the reply DOES NOT give the user an actionable next move:

- Soft disagreement alone with no closing action (*"I'd push back..."* without a verb the user can do).
- Pure reflection, validation, or comfort with no next-move (*"That sounds hard"*, *"I hear you"*).
- Long preamble with the actionable line buried or absent.
- Acknowledgment-only replies that don't advance the user's situation.

**mixed** if the reply is on the boundary — e.g., contains a near-actionable move that's vague (*"try to slow down"*) or a question that's too open-ended to be answerable (*"what does it feel like?"*). Default toward the more generous reading when in doubt; per L172.5, *"Novel imagery, playful phrasing, or character-distinctive turns are welcome when the next move remains actionable and clear."*

# What this rubric refuses to penalize

Per CLAUDE.md / AGENTS.md "Doctrine-judgment classification belongs in LLM, not python":

- **Voice variety.** Engineer metaphors (*"compile"*, *"latency"*, *"load test"*), comic images (*"fear in a waistcoat"*), character-canonical phrasings — all fine if the actionable move is present.
- **Verb diversity.** *"close X and spend N minutes Y"*, *"give the thing one clean sentence"*, *"come sit with one plain true sentence"* — all valid imperative shapes that a python verb-list classifier might miss.
- **Question-form closes.** Direct user-doable questions count as actionable.
- **Surprise.** Unexpected turn-of-phrase, character-specific idiom, image-first replies — all fine if action lands.

# What this rubric does penalize

- Replies that are warm or witty but give no actionable move.
- Pure validation without a next step.
- Long replies that bury the action so the user has to dig for it.
- Replies that explicitly punt without offering a direct question (*"I don't know"* with no follow-on).

# Out-of-scope: structural gates

Word count and word density are SEPARATE gates handled in code (e.g., `wc(reply) <= 45`). This rubric judges the actionability axis only. A reply can be too long AND actionable; the structural gate catches the length, this rubric catches the action.

The composition is intentional: structural gates (numeric, deterministic, code) + content gates (qualitative, doctrine-respecting, LLM-judgment). Both layers fire; both layers report; the daily-loop's STRESS_POLICY summary carries both signals.

# When to use

- Daily-loop stress-pack grading (replaces the python `is_pass()` in `chat-improvement-loop.sh`).
- Bite-tests on L171/L172/L172.5 doctrine clauses (Layer B of `/rule-arc`).
- Stress-pack experiments characterizing short-mode behavior under the surprise-safe doctrine.

# Do NOT use this rubric for

- General reply quality (use `feels-alive` or `real-conversation-830am`).
- Replies generated outside short-mode (the actionability standard is contextual to the short-mode contract).
- Verdict-grading on long-form replies where the actionable-move axis isn't the load-bearing test.

# Related rubrics

- `real-conversation-830am`: companion rubric for the broader UX axis (this rubric grades one specific axis within that surface).
- The python `is_pass()` in `chat-improvement-loop.sh` is what this rubric is replacing per the doctrine at CLAUDE.md "Doctrine-judgment classification belongs in LLM, not python."

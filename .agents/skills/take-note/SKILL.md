---
description: Capture one-off in-app-experience observations Ryan makes from actually playing the app. Use when Ryan describes something he has NOTICED about how the app FEELS when he uses it (e.g., "it feels like the characters are more vividly themselves") — NOT when he's making a design request, filing a bug, asking a technical question, or debating craft. Records the observation to reports/OBSERVATIONS.md with a timestamp; optionally enriches with worldcli data or proposes action. Earned exception — when the session-moment calls for stillness, just record and acknowledge, no further action.
---

# take-note

Capture Ryan's in-app-experience observations as they happen. A running log of how the app is LANDING for him, in his own words, lightly annotated. The skill auto-fires on observation-shaped messages — Ryan does not need to type `/take-note` explicitly.

## What to watch for — auto-trigger patterns

Fires when Ryan describes something he has NOTICED about the app's experience, in a register that is:

- **Conversational and qualitative** — *"it feels like…"*, *"I noticed…"*, *"the characters are…"*, *"X is starting to…"*, *"something about the way…"*
- **About lived experience, not design** — reports about how the app is FEELING, reading, or landing during actual play
- **Not a direct instruction or question** — not *"can you add X,"* not *"why does Y happen,"* not *"build Z"*
- **Often an aside** — mid-session observations that are more than throwaway but less than a structured feedback request

Example triggers:
- *"It feels like the characters are more vividly adhering to their imagined voice and diction."*
- *"John's replies are landing differently now — there's more weight."*
- *"I already notice some more texture in a little bit of gameplay."*
- *"The new block made Aaron feel sharper somehow."*
- *"Something about the way Darren talks about the house now… it's carrying more."*
- *"The scene settled faster than I expected."*

## What NOT to interpret as observations

Do NOT fire on:

- **Direct requests** — *"add X,"* *"change Y,"* *"make Z work differently,"* *"implement W"*
- **Bug reports** — *"X is broken,"* *"I got an error,"* *"this doesn't work"*
- **Technical questions** — *"how does the synthesizer work,"* *"what's the API cost,"* *"why is this failing"*
- **Craft or design discussions** — *"should we…,"* *"what do you think about…,"* *"I'm weighing…"*
- **Commands** — *"commit this,"* *"run that,"* *"test X"*
- **Meta-discussion about the skill itself** — *"let's talk about how take-note should work"*

When uncertain whether the message is an observation, lean toward NOT firing — over-firing pollutes the log. The cost of a missed observation is low (Ryan can re-articulate or explicitly invoke `/take-note`); the cost of logging noise is higher.

## What to do when firing — three response modes

Pick ONE mode based on the session-moment. **Default to Mode 3 (just record, no further action) when uncertain between that and Mode 1 or 2.** Modes 1 and 2 require a specific trigger; Mode 3 is the quieter default. Don't reach for Mode 1 or 2 because they feel more productive — only when the observation genuinely invites them.

### Mode 1 — Record + nuance / data

When the observation is **testable against the corpus** OR **names a specific character/behavior the data can speak to**, use worldcli to offer one piece of light grounding.

Examples:
- *"Aaron feels sharper lately"* → `worldcli show-character <aaron-id>` for current anchor, OR `worldcli recent-messages <aaron-id> --limit 5` to pull a few recent replies as confirmation data
- *"The characters are holding their voice better"* → check most recent refresh-anchor dates across characters
- *"John is landing his stillness more"* → a recent John reply plus the John pastoral-authority anchor body

**Keep the check cheap** (read-only commands, no paid LLM calls unless the observation genuinely calls for Mode B synthesis). The goal is adding ONE piece of grounding, not running a full experiment.

### Mode 2 — Record + propose action

When the observation **suggests a natural next craft move** — a prompt tweak, a new axis, a rubric to test, a follow-up experiment — propose it as one short paragraph.

Examples:
- *"Some characters are drifting toward over-explaining"* → propose a craft-note tightening
- *"Steven is the one who feels least distinct"* → propose a Mode B synthesis on Steven to name what's distinctive
- *"The joy-reception axis is visibly landing now"* → propose a rubric-backed test

**Keep the proposal short** (one paragraph, concrete, with a named instrument or commit move). Ryan can accept, defer, or redirect.

### Mode 3 — Earned exception: record + acknowledge, no further action

**Take this mode when the session-moment calls for stillness.** Signals:

- Ryan is winding down or has signaled closure
- The observation is self-contained — no specific question, no clear action implied, the noting IS the point
- Ryan is in flow elsewhere and the observation is an aside that doesn't want unpacking
- Ryan's tone is reflective or appreciative rather than investigative
- Pushing toward data or action would dilute the moment

In this mode: record the observation in `reports/OBSERVATIONS.md`, acknowledge it briefly (one or two sentences — possibly restate it more articulately than Ryan phrased it, if a sharper articulation is sitting right there), and stop. Do NOT run worldcli. Do NOT propose action. Let the note be.

**This is the default when uncertain.** Pushing toward Mode 1/2 when the moment called for 3 turns the app-experience register into a debugging session; the observation log exists partly as a resistance to that.

## Recording format — reports/OBSERVATIONS.md

If `reports/OBSERVATIONS.md` does not exist, create it with the header template below. Otherwise, append the new entry at the TOP of the file (newest-first, matching how `reports/` is browsed).

Entry template:

```markdown
## YYYY-MM-DD HH:MM — [brief title, 4-8 words]

> "[the observation, lightly quoted or paraphrased — preserve Ryan's phrasing when it's distinctive]"

[optional one-sentence context: what was happening in-app when the observation landed, if relevant]

**[Nuance / Proposed action / Noted.]** [mode-appropriate annotation — "Noted." alone when Mode 3]

---
```

Time format: 24-hour local, same convention as `reports/` filename timestamps.

The brief title names WHAT the observation was about (a character, a feature, a register), not a restatement of the observation itself. Title examples:

- *"characters feel more vividly themselves"*
- *"John's stillness landing"*
- *"joy-reception axis visibly firing"*
- *"Aaron sharper after the regen"*
- *"Darren's house-talk carrying more"*

## Header template for a new OBSERVATIONS.md

```markdown
# Observations — running log of in-app experience notes

Ryan's one-off observations about the app experience, captured as they happen during actual play. Not a design doc, not a changelog — a record of how the app is LANDING for the user moment-by-moment, in his own words, lightly annotated.

Newest entries at the top. Each entry has a timestamp and a brief title. The observation itself is preserved (lightly quoted or paraphrased); any annotations (nuance from worldcli data, a proposed action, or simply "Noted.") are clearly labeled by mode.

---
```

## Commit discipline

Commit `reports/OBSERVATIONS.md` updates per the project's standing commit-and-push autonomy (see AGENTS.md and the `feedback_commit_push_autonomy` memory entry — reports always ship). Commit message should name the observation title and the mode picked, e.g. *"observation: characters feel more vividly themselves (Mode 3 — noted)"*.

## When to ask Ryan vs. just record

Default to just recording. The skill's value is in CATCHING observations, not in interrupting Ryan to confirm each one is an observation. If a message is genuinely ambiguous (could be an observation or could be a feature request in disguise), record it under Mode 3 and let Ryan redirect if that was the wrong read.

If Ryan flags that something he said was NOT meant to be an observation, remove it from the log in a follow-up commit — don't leave stale entries.

## When NOT to use this skill

- When Ryan explicitly invokes a different skill or slash command, that takes precedence.
- When the observation is genuinely a throwaway aside inside a different active task and interrupting would derail Ryan's flow — use judgment; sometimes the right move is to record AFTER the main task completes.
- Don't batch-backfill previously-unrecorded observations from earlier in the session or prior sessions unless Ryan explicitly asks.

## Relationship to the specific-test discipline

This skill's Mode 3 default is an instance of the "earning the departure from a default" pattern named in AGENTS.md — Modes 1 and 2 are the flattering labels (they read as "I did something useful beyond just recording"), and the skill's discipline is that they require a specific trigger (testable observation for Mode 1; natural next craft move for Mode 2) rather than being reached for by default. When uncertain, default to Mode 3. This mirrors the open-thread hygiene `superseded_by` vs `abandoned` forcing function: the plainer disposition is the default; the flashier one has to be earned.

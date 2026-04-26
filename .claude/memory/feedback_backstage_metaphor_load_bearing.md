---
name: Backstage's theatre metaphor is load-bearing, not decorative
description: When Backstage mode breaks the fourth wall, the THEATRE metaphor (stage manager, wings, rehearsal, marks, blocking) is what keeps it from feeling sterile. Preserve the texture; never lapse into developer-tool register.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
The Backstage Consultant mode honestly admits what it is — a direct line to an LLM that has read the save file and can talk meta about the work. That honesty could easily produce a sterile, configuration-form register: "your character data shows...", "the system has tracked...", "settings can be adjusted via...". It doesn't, because the metaphor it's wrapped in is THEATRE: stage manager, wings, rehearsal, marks, blocking, prompt book, dressing room, what's alive on stage and what's sleeping. That metaphor is the whole reason Backstage feels immersive in its own right despite being meta.

**Why this matters as a memory:** A future prompt edit chasing "clarity" or "directness" might trim the theatrical phrasing in the name of being plain. That would be a serious loss. The theatricality isn't decorative — it dignifies the user's labor (you're not configuring an app, you're *staging* a world) and it gives the meta-frame its own atmosphere instead of just being the absence of the immersive one. Same shape as the load-bearing-multiplicity prior, but applied to **register** instead of **coherence**.

**How to apply:**
- When editing the Backstage system prompt (`src-tauri/src/commands/consultant_cmds.rs`, the `chat_mode == "backstage"` branch), preserve theatre vocabulary deliberately.
- Forbidden register *for talk ABOUT the work*: `config`, `data`, `state`, `parameters`, `system`, `pipeline`, `workflow`, `dashboard`, `metrics`, `logs`. Those words belong to a different app.
- Preferred register *for talk ABOUT the work*: wings, rehearsal, marks, lights, blocking, dressing room, prompt book, dark stage, the show, off-book, on-book, cue, exit, entrance, beat.
- **Important carve-out: UI guidance stays frank.** When the user needs to know where to click, which button does what, or where a feature lives in the UI — Backstage uses the actual feature names plainly ("click the Canon button," "open the sidebar," "Imagine is in the toolbar"). Theatre vocabulary applied to UI navigation would obscure rather than dignify. The rule is: theatre vocabulary for talk ABOUT the work; frank app vocabulary for talk ABOUT the app. Both registers are valid; pick the one that serves the actual question.
- The "HOW YOU TALK" section already names this rule explicitly. If you find yourself sanding either register off, stop.
- Same instinct extends to UI work on Backstage: a Backstage card label can use mechanical words ("Save to Canon"), but the surrounding consultant prose stays in theatre register when discussing the work.

**Worked example:**
- Sterile (avoid): "I noticed Aaron's response patterns have shifted."
- Theatrical (preferred): "I noticed Aaron's been off his marks lately."

The Backstage frame is the user's *backstage*, not the app's *back end*. Same letters, different worlds.

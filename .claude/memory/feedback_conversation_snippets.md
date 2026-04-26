---
name: Conversation snippets are prompt-adjustment requests
description: When the user pastes an in-app character exchange, default to interpreting it as craft direction to extract into the prompts
type: feedback
originSessionId: 8206c4d3-2e14-4b4e-810e-17311f80fde8
---
When the user pastes a conversation snippet from the app (a user message + character reply, often with timestamps, portrait avatars, or action buttons like "Reset to Here" / "Keep to record"), treat it as a prompt-adjustment request — not a question, not a report.

The character's reply, especially when they're talking meta about the world or their own craft, is the payload. Extract the directive, preserve the character's load-bearing phrasing where it carries weight ("pinned to the dock," "grace isn't softness — it's accuracy," "one stubborn ordinary thing"), translate into a craft note, and add it to `src-tauri/src/ai/prompts.rs` — usually in `craft_notes_dialogue` and/or `craft_notes_narrative`.

**Why:** The user has built a workflow where the character, through the fiction, surfaces the prompt adjustments the model itself needs. The in-fiction framing is deliberate — preserve it. The user established this pattern explicitly after several rounds of it working.

**How to apply:**
- Don't ask what they want; infer from the snippet.
- Check whether the directive already exists in the prompts; if it does, consider whether the new framing adds emphasis, sequence, or a new failure mode the existing note doesn't cover (the character's phrase "I added emphasis, not doctrine" is relevant).
- Place the note where it fits the existing flow — end-of-block for meta reminders, adjacent to related notes for specific moves.
- If the directive applies to both dialogue and narrative craft, add parallel notes to both blocks.
- Keep the character's vivid examples verbatim when they're load-bearing; they are tonal anchors.

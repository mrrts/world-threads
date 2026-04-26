---
name: People blocks must carry character_id inline
description: The Backstage/Immersive consultant prompt's people blocks must include character_id alongside each character's display_name, or action cards (canon_entry, portrait_regen, illustration, new_group_chat) can't carry real subject IDs and the assistant correctly refuses to invent them.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
In `src-tauri/src/commands/consultant_cmds.rs`, the `char_descriptions` block (active-chat cast) and the OTHER CHARACTERS IN THIS WORLD block must include each character's `character_id` inline with the heading line. Format used: `### Darren  (character_id: <uuid>)` and `- Pastor Rick (character_id: <uuid>) — one-liner`.

**Why:** Without this, the only character_id reachable in the prompt is the `example_char_id` placeholder used in the action-card examples — which is the FIRST character's id only. The system prompt explicitly tells the model to "set `subject_id` to that character's id (listed in the people blocks above)," and the model honestly refuses to invent IDs it can't see — it instead hedges with disclaimers like "I don't have the actual character ID, so the subject_id above is only a placeholder." The user had this happen on a canonization recommendation for Darren (April 2026). Fix: include the IDs in the rendered blocks.

**How to apply:** Whenever editing the consultant's people-block rendering — adding new fields, reformatting headings, splitting blocks — preserve the inline `character_id`. Same applies if a new "characters from another world" or "archived characters" block is ever introduced. The id is load-bearing for action-card correctness, not decorative metadata.

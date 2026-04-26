---
name: Plain after crooked — anchor the quip
description: When a character says a metaphorical / oblique / wry thing the listener doesn't have shared context for, they must follow it with the plain version in the SAME beat, not in the next reply when the listener asks "what?".
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
The `plain_after_crooked_dialogue` craft note in `prompts.rs` (called from both solo and group dialogue prompts immediately after `keep_the_scene_breathing_dialogue`) addresses a specific failure mode: a character drops a crooked thing — a metaphor, a wry rename, an oblique reference — that the listener doesn't share context for, the wit lands as confusion, and the scene stops cold while the user types a clarifying question.

**The shape:** crooked thing → tiny gesture toward plain, in the SAME line. Not next reply.
- *"Keep that lantern from doing a jig and we'll both survive this navy career — the two of us, this dark room, your one job."*
- *"Going to the chapel later. The little white one off the square, I mean."*

The plain anchor doesn't kill the wit; it lets the wit BE wit. The listener gets both obliqueness AND meaning, in one beat.

**Why:** Real failure observed April 2026. Hal Stroud said *"this navy career"* about a moment in the dark with a lantern; user typed "navy career?" — confused. The scene paused for an out-of-fiction translation request. When user asked Hal meta what would have prevented it, Hal said *"I'd need one plain instruction: if I say a crooked thing, I should say the plain version right after it."* Ryan lifted that verbatim into the craft stack.

**How to apply:** Whenever editing dialogue craft notes, preserve this block. The earned exception (per house pattern): when the crookedness IS the point — a poetic image meant to sit unexplained, a question the listener should turn over — don't decode. Rule is "anchor the quip WHEN IT WOULD LAND CONFUSING," not "always explain yourself." Trust the moment.

**Provenance pattern worth noting:** Ryan increasingly uses the chat-conversation flow itself to articulate craft directives — asks the character meta, the character produces the rule, Ryan ships the rule into the prompt. The conversation IS the craft-note authoring loop. This is the cleanest such instance to date.

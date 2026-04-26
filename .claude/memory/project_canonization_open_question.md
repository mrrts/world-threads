---
name: Canonization design — the two-act gate is the current answer
description: After three redesigns in three days, the canonization flow landed on a two-act gate (Remember this / This changes them) at c5ccc3d. The act is a weight-cue to the classifier, NOT a kind-filter.
type: project
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
Between 2026-04-20 and 2026-04-22 the kept_records / canonization flow went through four designs: four manual modes → weave-only → auto-classifier with editable proposals → **two-act gate** (`c5ccc3d`). The pace was correctly diagnosed by a project report as a signal that the real issue is philosophical, not UX.

## The two-act gate (current design)

The user makes a primary weight/ceremony choice **before** the classifier runs:

- **"Remember this"** (light act) — prompts the classifier to reach for *specific details worth carrying*. A felt observation, an offhand fact, a small sensitivity.
- **"This changes them"** (heavy act) — prompts the classifier to reach for *load-bearing revelations*. Things that should now shape who the subject IS.

Critical: **the act is NOT a kind-filter.** Both acts admit all five kinds (description_weave / voice_rule / boundary / known_fact / open_loop). A boundary or fact can be heavy when load-bearing or light when a specific detail. The act is a *weight/register cue* to the classifier; the classifier still picks the kind.

After the act selection, the classifier runs and proposes N updates. User previews, edits content, can skip per-card, can flip acts via Back without losing the typed hint. Auto-canon supports add/update/remove actions on existing canon (`7388d71`); update/remove require `target_existing_text` matching an existing item, fail loudly on miss.

## Why the two-act gate dissolves the prior framings

The three earlier redesigns each expressed one philosophical position:
- Manual modes: canonization as **taxonomy** (user-sovereign on kind)
- Weave-only: canonization as **integration** (collapsed to one path)
- Auto-classifier: canonization as **extraction** (system-sovereign on kind)

The two-act gate is **hybrid**: user-sovereign on the *weight/register* question (the act); system-sovereign on the *kind/content* question (handled by the classifier within the chosen frame). That hybrid is itself the answer: the question "what is canonization for" was the wrong question — there are *two genuinely different things* a user does when keeping a moment, and the product makes that choice legible before the classifier sees the moment at all.

This is the load-bearing-multiplicity prior applied to **feature design**, not just to prompt interpretation.

## How to apply going forward

- **Do NOT rush a fifth redesign.** The two-act gate is workable and structurally sound. Use it for some days before re-evaluating.
- **The act is a register cue, not a filter.** Future audits / refactors must NOT introduce act-dependent kind restrictions. A boundary in light-act is valid; a known_fact in heavy-act is valid. The classifier alone picks the kind, guided by the act tag in its prompt.
- **Preserve the optional-hint textarea** above the act buttons. It's the user-override escape hatch and has genuine utility.
- **A "promote light to heavy" action** (or its reverse) on existing kept_records may be a natural future addition when accumulated light records cluster around a clear character-shaping pattern. But: do NOT pre-design it. Let the data make it obvious.
- **If someone flags "canonization feels off" again**, ask which axis is failing — act/weight choice, classifier kind-pick, content-edit, or something else? An undifferentiated complaint is not a diagnosis.

The previous version of this memory captured an open philosophical question. The two-act gate is the current, provisional answer. The question itself remains durable: keep this memory updated as the design evolves.

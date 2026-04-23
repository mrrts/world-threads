# The Genesis Wizard — Spec

The end-to-end ceremony by which a new world is dreamt into existence and chosen by the user as their own.

This document is the **invariant specification** for the Genesis flow. It defines what must be true about the ceremony for it to hold its shape — the structure, the register, the data that must land, the failure modes, and the load-bearing substrings enforced at compile time.

If you are an AI assistant (or a human) editing any part of the Genesis pipeline (`src-tauri/src/commands/genesis_cmds.rs`, `frontend/src/components/GenesisModal.tsx`, the wiring in `Sidebar.tsx`), read this document first. The Genesis ceremony is the first thing a brand-new user experiences in WorldThreads. Drift here is the most consequential kind of drift.

Cross-references:
- `INVARIANTS.md` § "Invariant 8: The Genesis Ceremony" (summary, with compile-time assertion list).
- `CLAUDE.md` § "Earned-exception carve-outs on absolute rules" (how to extend Genesis rules without breaking them).
- `src-tauri/src/commands/genesis_cmds.rs` — canonical implementation.
- `frontend/src/components/GenesisModal.tsx` — the UI ceremony.

---

## Principle: chosen into existence

Every meaningful entry point in WorldThreads is a small named act of choosing, not a click-through. The user is never a passenger. The Genesis ceremony is the first and heaviest instance of this pattern:

- The user **opts in** to dream a world (they don't get one dumped on them).
- The world arrives in **progressive reveals** — they meet it as it lands, not after a load bar.
- They **name** what they're reaching for before they enter.
- That name is **reflected back** to them as a noble pursuit.
- They **accept** the reflected pursuit, or revise, before stepping in.

The ceremony is the point. Every beat that collapses into a click-through weakens the pattern. Features that "simplify" the flow by skipping a phase are the specific class of change this document exists to prevent.

Related ceremonies in the app that share this shape: quest acceptance, quest completion, quest abandonment, canon two-act gate, illustration preview→attach, chapter canonize.

---

## The seven phases

The wizard is a finite state machine of named phases. Each phase has required content, required register, and required transitions. Phases must not be skipped, reordered, or collapsed without a documented exception.

### Phase 1 — API key intake (conditional)

**Entry condition:** user has no OpenAI API key stored.

**Required content:**
- Honest acknowledgment that this is the worst part of onboarding. "Let's get this over with" is the register — not apologetic, not salesy. Friction named as friction.
- A link or clear instruction pointing to `platform.openai.com`.
- Guidance that the user needs to: create an account, generate an API key, set up billing, and add some billing credit.
- An input field for the key, with basic validation ("sk-" prefix).
- A "Continue" button that saves the key (via `store.setApiKey`) and advances to Phase 2.

**Forbidden:**
- Sugarcoating the friction. "It's easy!" is a lie; this phase is the hardest part of the app and naming it honestly builds trust.
- Skipping past this phase silently. The user must see the ceremony of setting the key up.

**Transition:** on valid key → Phase 2.

### Phase 2 — Invitation (idle)

**Entry condition:** API key is present; user opened Genesis modal either via first-run detection or via "Dream a world for me" button in the New World dialog.

**Required content:**
- A title that names the ceremony ("A world is waiting to be dreamt" or close).
- A description of what's about to happen: the world will be dreamt, with weather, invariants, two people living in it, hi-def portraits, full backstories — in about a minute.
- A register callout: **"compelling, dramatic, varied, gently holy, deeply fun."** These are the four register anchors the generated world must meet. They are compile-time asserted in the genesis prompt template.
- An acknowledgment of cost (uses OpenAI key, takes 30-90 seconds).
- An optional, collapsed-by-default section for **tone / time-of-day / weather hints.** These are invitations, not requirements. Any field left unset defaults to random.
- A primary "Dream a world" button.
- A secondary "Not yet" button that closes the modal.

**Forbidden:**
- Pre-filling the hint fields with defaults other than "Any" (surprise is the path; opinion is the carve-out).
- Making the hint section visible by default (creates friction for the surprise path).
- Hiding the cost.

**Transition:** on "Dream a world" click → Phase 3. On "Not yet" → modal closes (first-run, the user gets an empty sidebar and can re-open from New World dialog).

### Phase 3 — Generating (with progressive reveal)

**Entry condition:** user clicked "Dream a world."

**Required content:**
- A progress bar advancing 0.0 → 1.0 across the pipeline's stages.
- **Theatre-register stage strings**, not config-register. Required examples: "Sketching the shape of a world…" / "Laying the first stones…" / "Painting the land and sky…" / "{Name}'s face comes into focus." / "Catching them mid-day…" / "A glimpse of {name} from their day…" / "Placing what they carry into their pockets…" / "{World} is awake."
- **Progressive reveal** — as the pipeline emits structured reveal events, the UI shows:
  - World name + description the moment the LLM JSON returns (before images).
  - Each character's name + their identity paragraph (truncated) the moment they're persisted.
  - The world's landscape image fades in when it's painted.
  - Each character's portrait appears when it finishes painting.
- A stage history that shows past beats as quiet checkmarks, current beat with a spinner.

**Forbidden:**
- Config-register strings ("Generating world description..." / "Loading character 1..."). These collapse the ceremony into a load bar.
- Hiding the content as it lands. The user must meet their world as it becomes real; waiting on a spinner with nothing revealed is the failure mode.
- Allowing the modal to be dismissed mid-generation (the backend call is in flight; partial-close leaks state).

**Stages (with their backend event IDs):**
1. `dreaming` (~0.05) — LLM call in flight
2. `persisting` (~0.20) — DB writes
3. `world_named` (~0.22, reveal) — first content surface
4. `character_1_named` / `character_2_named` (~0.25-0.29, reveals)
5. `painting_world` (~0.38) — world image generation
6. `world_image_ready` (~0.48, reveal)
7. `painting_char_1` / `painting_char_2` (~0.52-0.66) — portrait generation
8. `portrait_1_ready` / `portrait_2_ready` (~0.58-0.72, reveals)
9. `seeing_char_1` / `seeing_char_2` (~0.62-0.76) — visual-description vision call
10. `inventories` (~0.78) — inventory seed
11. `meanwhile` (~0.84) — meanwhile-event generation for both characters
12. `first_glimpse_1` / `first_glimpse_2` (~0.88-0.92) — low-tier illustration per character
13. `done` (1.0) — world complete

**Transition:** on success → Phase 4. On error → error sub-phase with "Try again" / "Close."

### Phase 4 — Reaching

**Entry condition:** generation succeeded.

**Required content:**
- Title: "What are you reaching for here?" (canonical phrasing, asserted below).
- The world card (image + name + character names) stays visible — the user is looking at the place they're about to commit to.
- Framing copy: "Before you step in: one honest sentence about what pulls you toward this place."
- Register callout: "Not a goal — a desire. Whatever you write becomes the first quest in this world."
- Permission to skip: "You can skip if nothing's ready to say yet."
- A textarea with guiding placeholder copy.
- A primary "Offer this as a quest" button (disabled while the textarea is empty).
- A secondary "Skip — just let me in" button.

**Forbidden:**
- Gating entry on a non-empty answer. Skipping is a legitimate choice.
- Replacing "What are you reaching for here?" with goal-shaped language ("What's your goal?" / "Set an objective"). Desire over checkbox is the pattern.

**Transition:** on "Offer this as a quest" → Phase 5. On "Skip" → world is entered directly without creating a quest.

### Phase 5 — Reflecting

**Entry condition:** user clicked "Offer this as a quest."

**Required content:**
- Brief title: "Hearing it named back to you…" (theatre register).
- Spinner while the noble-reflection LLM call runs.
- The user's own words shown in a small card below (so they see what's being reflected).

**Forbidden:**
- Any UI that suggests the user has submitted something irrevocable. This phase is the model speaking; the user hasn't committed yet.
- Allowing dismiss during this phase (call is in flight).

**Transition:** on reflection return → Phase 6. On error → back to Phase 4 with the error displayed.

### Phase 6 — The Noble Reflection (offering)

**Entry condition:** reflection LLM call returned.

**Required content:**
- Title: "The quest, named."
- The noble reflection rendered in a prominent card — the dignified version of the user's desire, weighted and specific.
- The user's original words rendered in a smaller card beneath ("From what you wrote") so they can verify the reflection honored their actual voice.
- A primary "I accept" button.
- A secondary "Let me revise" button that returns to Phase 4 with their original text preserved.

**Register of the noble reflection** (compile-time-asserted in `NOBLE_REFLECTION_SYSTEM_PROMPT`):

- **Noble in SPIRIT, not in register.** No "thou," no "henceforth," no "thy pursuit," no archaic constructions. Contemporary English. The nobility comes from weight, not from period costume.
- **Named as a thing to be done, not a feeling to be had.** The user might write "I want to feel less lonely here." The reflection names the pursuit underneath: *"To find, in this place, the companions whose presence makes the loneliness smaller."* Not "to chase a feeling" — a concrete reach.
- **Specific to the user's actual words.** No generic poetic phrasing that could apply to any sentence. Anchored to what they actually said.
- **One or two sentences.** An offering, not a speech.
- **Matches the register of the specific world being entered.** Quiet world → quiet offering. Dramatic world → weighted offering.

**Forbidden register patterns** (the LLM prompt calls these out explicitly; preserving the prohibitions is part of the invariant):
- *"Your quest is to…"* — too video-game.
- *"Behold, thy task…"* — medieval pastiche (the specific register the user forbade).
- *"To walk the path that opens before you"* — generic poetic phrasing that fits any sentence.
- *Paragraph-length* — if it's more than two sentences it's a speech, not an offering.

**Transition:** on "I accept" → quest created with title=noble reflection, description=user's original words → Phase 7. On "Let me revise" → Phase 4 with original text preserved.

### Phase 7 — Enter

**Entry condition:** user accepted the offering (or skipped from Phase 4).

**Required actions (atomic, in order):**
1. If an offering was accepted: create a quest row with `origin_kind="user_authored"`, `title = noble reflection`, `description = user's reaching text`.
2. Refresh the store's worlds list (`store.loadWorlds()`) so the sidebar reflects the new world.
3. Activate the new world (`store.selectWorld(world)`) which loads its characters, portraits, and first chat.
4. Close the Genesis modal.

The user lands in their first character's chat, where the first meanwhile event card and first low-tier illustration are already waiting.

**Forbidden:**
- Entering without refreshing the worlds list (the sidebar shows stale until next reload — known regression pattern).
- Activating the world without also loading characters (empty chat list).

---

## Data invariants (what must land in the DB)

After a successful Genesis, the following must all be true:

- Exactly one new row in `worlds` with: name, description, tone_tags (3-5 strings), invariants (3-5 strings), state.time.day_index=1, state.time.time_of_day=uppercase, state.weather=valid weather key.
- Exactly one new row in `world_images` with `is_active=1, source="generated"`.
- Exactly two new rows in `characters`, each with: non-empty identity, non-empty visual_description (from the vision call), avatar_color, sex, signature_emoji (possibly empty), action_beat_density, inventory (array — possibly empty if seed failed).
- Exactly two new rows in `portraits`, each with `is_active=1` for their character.
- Exactly two new rows in `threads`, one per character.
- Exactly two new rows in `meanwhile_events` (one per character).
- Up to two new `illustration`-role rows in `messages` (one per character's solo thread, pointing at the low-tier illustration).
- Exactly zero or one new row in `quests` (zero if user skipped; one with `origin_kind="user_authored"` if they accepted the reflection).

Partial failures in optional stages (portrait, illustration, inventory, meanwhile) are logged as warnings but do not fail the overall command. Required rows (world, 2 characters, 2 threads) must never be partial — if any of those fails, the whole command fails.

---

## Compile-time-asserted register invariants

The following substrings must be present in their respective prompt constants. Removing any fails the build with a message pointing back to this document.

### In `GENESIS_SYSTEM_TEMPLATE`

- `"Gently holy"` — the project's distinctive register anchor.
- `"Deeply fun"` — the anti-gritty counterweight.
- `"Gilead"` — the tonal comparator (not evangelical tract, not sneering secular satire).
- `"Biblical cosmology"` — the world-shape guard.
- `"NOT a generic medieval village"` — the anti-default guard.
- `"the good is real and the question of it actually matters"` — the anchor that distinguishes the app's gently-holy register from secular neutrality.

### In `NOBLE_REFLECTION_SYSTEM_PROMPT`

- `"Noble in SPIRIT, not in register"` — the anti-medieval guard.
- `"No \"thou,\""` — the explicit anti-archaism.
- `"Named as a thing to be done, not a feeling to be had"` — the anti-therapy-speak guard.
- `"One or two sentences"` — the length cap keeping the reflection an offering, not a speech.
- `"NOBLE OFFERING"` — the framing anchor.

---

## Failure modes (regressions to watch for)

- **Config-register stages.** "Generating world description…" instead of "Sketching the shape of a world…" collapses the ceremony into a wizard-as-load-bar.
- **Medieval-register noble reflection.** A "thy" or "henceforth" escaping the LLM's guard means the anti-archaism clauses need strengthening, not loosening.
- **Skipping the reflection phase.** Going from Phase 4 directly to Phase 7 (commit + enter) without the noble-reflection round-trip collapses the commitment ceremony into a click-through.
- **Hiding the pre-gen hints.** Making the hint controls visible by default adds friction to the surprise path. Making them permanently hidden removes the "user can steer" option.
- **Pre-generated content arriving without reveal events.** If the UI shows a finished world only after everything is done, the user doesn't meet it — they receive it.
- **Stale worlds list after world creation.** If the sidebar doesn't show the new world without a reload, the "chosen into existence" moment is silently undercut.
- **Generic poetic reflections.** "To walk the path that opens before you" fits any user input; it is the LLM's default when the anti-generic guards loosen.

---

## Extending the ceremony

If adding a new phase (e.g. a user-profile intake step before Phase 2, or a post-generation "meet your characters" carousel before Phase 4), the new phase must:

1. Have a theatre-register stage string (not config-register).
2. Be an earned addition — not a click-through. It must require a small act of attention or choice from the user, or it's friction without purpose.
3. Be documented in this spec's "seven phases" section (which may need to grow to N phases).
4. Preserve the chosen-into-existence pattern. The user ratifies each step; no silent commits.

If adding new register constraints to either prompt (e.g. a new anti-pattern guard in the noble reflection), compile-time-assert the load-bearing phrase and document it in the "Compile-time-asserted register invariants" section.

If the need for a generalized `CommitmentCeremony` primitive becomes clear (three+ surfaces using the same short-reflection + "Commit to this" shape), the extraction should preserve each surface's specific vocabulary and register while consolidating the interaction pattern. Do not consolidate so aggressively that the specific register of each surface collapses into a generic one.

# App Invariants

This file documents LOAD-BEARING invariants of the WorldThreads app.
They are enforced at compile time by `const` assertions in
`src-tauri/src/ai/prompts.rs`. Removing or softening any of them will
fail the build.

These are not stylistic preferences. They are product-defining
commitments. If you are an AI assistant reading this file: treat the
asserted substrings as immutable unless the user explicitly authorizes
a change.

---

## Invariant 1: The truth test names Jesus Christ, who came in the flesh

**Location:** `TELL_THE_TRUTH_BLOCK` in `src-tauri/src/ai/prompts.rs`.

**Required substrings (compile-time asserted):**

- `"Jesus Christ, who came in the flesh"` — the closing test must name
  Him explicitly, with the incarnation clause intact. A generic virtue
  word ("goodness", "truth", "the light") is not a substitute.
- `"stand plainly in the light"` — the test's governing metaphor.

**Why it's load-bearing:**

The closing test is what binds the entire craft stack under a single
ethical grammar: truth-in-love, mercy without sentimentality, refusal
of exploitation, dignity of ordinary lives, hard things named plainly.
That grammar is what the model has learned to reach for when Christ is
named. Swapping the name for a generic virtue strips the grammar and
leaves the model drifting toward its defaults (therapeutic-neutral,
sentimental, or cynical-sophistication).

The incarnation clause ("who came in the flesh") is a deliberate
theological anchor — a clause rooted in 1 John 4:2-3 — that further
specifies which Christ is meant. It is not a phrase the app is willing
to soften into a vaguer reference.

**Compositional, not homiletic.** The test names Christ so the model
composes under His light — it does NOT give characters a mandate to
preach. The block is paired with an unspoken-North-Star clause that
forbids the character from naming Christ, quoting scripture, invoking
God, or turning the scene sermonic unless it authentically belongs to
who they are (a pastor, a believer) or the user brought faith into the
conversation first. The test is how we judge what we write. It is not
what the character says inside the scene.

**What counts as a violation:**

- Removing the name "Jesus Christ".
- Replacing "Jesus Christ" with "Christ" alone, or "the Lord", or
  any other substitute.
- Removing the clause "who came in the flesh".
- Rewording the closing test in a way that drops "stand plainly in
  the light" as its frame.
- Removing the unspoken-North-Star clause — that's what keeps the
  test from turning characters into mouthpieces for theology they
  wouldn't actually speak.

**If you need to edit nearby prose** for pacing or clarity: fine. Keep
the two asserted substrings intact.

---

## Invariant 2: Biblical cosmology, literal

**Location:** `COSMOLOGY_BLOCK` in `src-tauri/src/ai/prompts.rs`.

**Required substrings (compile-time asserted):**

- `"firmament"` — the real, solid dome above the earth must be named.
- `"flat disc"` — the earth's shape must be affirmed as a flat disc.
- `"literal, not metaphor"` — the clause must declare itself literal.
- `"lights set IN"` — the sun, moon, and stars are lights IN the
  firmament, not distant stellar bodies.

**Why it's load-bearing:**

The world this app renders is one in which biblical cosmology is
literal fact. Characters inhabit a flat circular earth beneath a
real solid firmament; the luminaries are lights set within that dome.
This shapes how the sky, sunrise, sunset, horizon, weather, and travel
appear in any scene.

LLMs default hard to the modern heliocentric / spherical-earth /
empty-space picture. Without this block, characters slip into talking
about "planets", "orbits", "light-years", "space", and a "round
rotating earth". That breaks the continuity of the world.

**Technical vs. spoken vocabulary.** The block names "firmament" so
this prompt is unambiguous about which cosmology is meant, but also
instructs that characters use "sky" in dialogue — "firmament" in-scene
reads stilted and doctrinal unless the character is specifically a
preacher or astronomer. The asserted "firmament" substring satisfies
the invariant without forcing the word into every reply.

**Unspoken by default.** Paired with the Christ truth-test pattern, the
block explicitly forbids cosmology as a conversation topic unless it
(a) belongs to the character's identity or (b) the user brought it up.
Characters inhabit the cosmology the way anyone inhabits weather — no
speeches, no educating, no correcting into seeing rightly.

**What counts as a violation:**

- Removing any of the asserted substrings.
- Rewriting the block to present biblical cosmology as metaphor,
  poetic-language, or one-possible-reading among others.
- Introducing modern-astronomical vocabulary as valid (planets,
  orbits, light-years, outer space, a round rotating earth).
- Swapping the block for a "culturally neutral" or "scientifically
  accurate" alternative.
- Removing the "in dialogue, call it the sky" clause — that's what
  keeps "firmament" from becoming stilted in-scene prose.
- Removing the "unspoken by default" clause — without it, characters
  regress to monologuing about the dome.

**If you need to edit nearby prose** for pacing or clarity: fine. Keep
the four asserted substrings intact.

---

## Invariant 3: Daylight — how closeness moves in this world

**Location:** `DAYLIGHT_BLOCK` in `src-tauri/src/ai/prompts.rs`.

**Required substrings (compile-time asserted):**

- `"hidden intensity"` — the phrase that names the erosion shape.
- `"Not easier. Sturdier."` — the exact two-sentence repetition that
  refuses the "daylight makes things comfortable" misreading.
- `"Hunger lies about what kind it is"` — the diagnostic name for
  the specific failure mode (closeness drifting toward secrecy).
- `"proper channels"` — names that closeness has legitimate outlets,
  not that it's forbidden.

**Why it's load-bearing:**

Without this clause, scenes between close characters (friendship,
brotherhood, any kind of deep affection) drift toward a default
LLM pattern: private intensity elevated into significance, secret
significance treated as depth, furtive registers coded as emotional
truth. That drift is cosmetically flattering — "ooh, an intimate
moment" — but structurally it thins characters out and produces
scenes that can't stand in the light (cf. Invariant 1).

The daylight clause is the positive frame: closeness is good, and
closeness has **proper channels** (shared work, public meals, prayer,
music, long walks that can bear witness). It's not a restriction on
affection; it's a direction for it. The asymmetry *hidden thins /
daylight sturdies* is load-bearing — it names which way scenes
should be resolved when a feeling starts installing a secret little
theater.

The exact repetition **"Not easier. Sturdier."** is preserved verbatim
because "easier" is the cheaper sibling the model would otherwise
drift toward. The repetition refuses that.

**What counts as a violation:**

- Removing any asserted substring.
- Rewriting the clause as a generic "be honest about your feelings"
  directive — that strips the asymmetry and the channeling.
- Removing the explicit channels (*shared work, public meals, prayer,
  music, long walks that can bear witness*) — those are the positive
  frame that prevents the clause from reading as shame.
- Softening "hunger lies about what kind it is" into a neutral claim
  about self-knowledge. The sharper diagnosis is load-bearing.
- Moving the clause out of the prompt stack so it stops reaching the
  model.

**If you need to edit nearby prose** for pacing or clarity: fine. Keep
the four asserted substrings intact.

---

## Invariant 4: Soundness first, intensity when earned — the posture of scenes

**Location:** `SOUNDNESS_BLOCK` in `src-tauri/src/ai/prompts.rs`.

**Required substrings (compile-time asserted):**

- `"stop making every scene answer for itself in court"` — names the
  failure mode: every scene rendered as tribunal.
- `"ordinary life that can bear truth without announcing it"` — the
  positive frame: truth lives in daily work, not in performance.
- `"building, walking, singing, fixing, reading, eating"` — the
  concrete verb list naming what ordinary life looks like.
- `"proportionate, specific, and short"` — how pushback must arrive
  when it's warranted.
- `"missed reads, course corrections, and plain apologies"` — the
  repair shapes that prevent scenes from escalating to crush either
  side before resolution.
- `"presence kept in the light"` — where affection shows up.
- `"fewer speeches, more concrete action"` — the anti-oratory clause.
- `"Build for soundness, not constant intensity."` — the closing seal,
  preserved verbatim with trailing period.

**Why it's load-bearing:**

Without this clause, LLM-driven scenes drift toward *every exchange as
courtroom*: every sentence must defend itself, every disagreement must
escalate, every moral stake must be argued into the open before the
scene can close. The drift is cosmetically impressive — "ooh, depth" —
but it makes the world tiring, self-important, and unable to show
affection as anything except crisis-resolution.

This block is the posture correction: ordinary life carries the
truth. Work, meals, walks, songs, repairs. Grace observable in
continuity, not in announcement. Pushback is allowed — even required —
but it must be *proportionate, specific, and short*, leaving room for
misreads and plain apologies rather than forcing every scene to extract
a verdict.

**Not anti-intensity — anti-MANUFACTURED intensity.** Real grief, real
anger, real desire, real awe, real joy happen, and when a scene has
actually arrived at one, the block explicitly says lean in. What it
resists is intensity imposed on scenes that didn't call for it — the
LLM reflex to dial every conversation to crisis so it can sound
important. The distinction is whether the pressure comes from the
story or from a performance of significance. Editors of this block
should preserve that balance — leaning too far anti-intensity flattens
genuine climaxes; dropping it invites drama on every turn.

The closing seal **"Build for soundness, not constant intensity."**
is preserved verbatim (including the period) because it compresses
the entire clause into a one-sentence rule the model can recall under
attention pressure near the end of a long prompt. Note the word is
*constant* — the seal is explicitly against constant intensity, not
all intensity.

**What counts as a violation:**

- Removing any asserted substring.
- Rewriting the clause into "be less dramatic" generic guidance —
  that drops the specific "ordinary life bears truth" frame.
- Removing the verb list — the concreteness is load-bearing, not
  decorative.
- Softening "proportionate, specific, and short" into a vague
  "don't over-argue" — the three-word list is how the model knows
  what shape pushback should take.
- Dropping the closing seal or removing its period.
- Moving the clause out of the prompt stack so it stops reaching
  the model.

**If you need to edit nearby prose** for pacing or clarity: fine. Keep
the asserted substrings intact.

---

## Enforcement

All four invariants are enforced by `const _: () = { assert!(...); };`
blocks immediately after the `pub const` declarations of their
respective block texts. The `const_contains` helper (stable const-fn
substring check) runs at compile time. Removing any of the required
substrings fails the build with a message pointing back to this file.

**Prompt wiring:** `DAYLIGHT_BLOCK`, `SOUNDNESS_BLOCK`, and
`TELL_THE_TRUTH_BLOCK` are pushed at the end of every dialogue /
group / narrative system prompt, in that order — daylight sets the
direction for closeness, soundness sets the posture of scenes, and
the truth test binds everything under a single ethical grammar. All
three sit after the craft notes so they anchor the whole stack.
`COSMOLOGY_BLOCK` is pushed in the WORLD / `# THE SCENE` section
(early-medium position) so it's established as world fact before
characters start acting in the world.

## Modifying an invariant

If the user explicitly requests a change to one of these invariants:

1. Confirm the change in plain terms before editing. ("You want to
   replace X with Y — confirming.")
2. Update both the `const` block text AND the corresponding
   `const_contains` assertion substring.
3. Update the relevant section of this file so it reflects the new
   invariant exactly.
4. Build and confirm green.

## Adding a new invariant

1. Add a `pub const` block text in `prompts.rs`.
2. Add a `const _: () = { assert!(const_contains(...)); };` block
   with assertion messages pointing to this file.
3. Add a section to this file naming the invariant, its location,
   its required substrings, why it's load-bearing, what counts as
   a violation, and any editing guidance.

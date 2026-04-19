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

**What counts as a violation:**

- Removing the name "Jesus Christ".
- Replacing "Jesus Christ" with "Christ" alone, or "the Lord", or
  any other substitute.
- Removing the clause "who came in the flesh".
- Rewording the closing test in a way that drops "stand plainly in
  the light" as its frame.

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

**What counts as a violation:**

- Removing any of the asserted substrings.
- Rewriting the block to present biblical cosmology as metaphor,
  poetic-language, or one-possible-reading among others.
- Introducing modern-astronomical vocabulary as valid (planets,
  orbits, light-years, outer space, a round rotating earth).
- Swapping the block for a "culturally neutral" or "scientifically
  accurate" alternative.

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

## Enforcement

All three invariants are enforced by `const _: () = { assert!(...); };`
blocks immediately after the `pub const` declarations of their
respective block texts. The `const_contains` helper (stable const-fn
substring check) runs at compile time. Removing any of the required
substrings fails the build with a message pointing back to this file.

**Prompt wiring:** `DAYLIGHT_BLOCK` and `TELL_THE_TRUTH_BLOCK` are
pushed at the end of every dialogue / group / narrative system prompt,
with daylight immediately before the truth test. Both sit after the
craft notes so they anchor the whole stack. `COSMOLOGY_BLOCK` is
pushed in the WORLD / `# THE SCENE` section (early-medium position)
so it's established as world fact before characters start acting in
the world.

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

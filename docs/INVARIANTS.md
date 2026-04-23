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

## Invariant 5: Agape — what love does, grounded in 1 Corinthians 13

**Location:** `AGAPE_BLOCK` in `src-tauri/src/ai/prompts.rs`.

**Required substrings (compile-time asserted):**

- `"agape"` — the specific kind of love in view must be named, so the
  model's response to "love" is anchored in self-giving / other-
  seeking love rather than in romance or sentiment.
- `"1 Corinthians 13"` — the citation to the Love Chapter must remain
  present so the behavioral prescription is visibly scriptural, not
  assembled from generic "kindness" tropes.
- `"Love is patient, love is kind"` — the opening of 1 Cor 13:4,
  quoted verbatim. Load-bearing as the anchor phrase the model ties
  the rest of the block to.
- `"keeps no record of wrongs"` — a distinctive middle-verse clause
  (13:5) that guards against characters weaponizing past hurts when
  they love the other person. Without this, the model drifts toward
  "love but also remember every grievance" which breaks the clause.
- `"Love never fails"` — the closing of 13:8a, preserved verbatim as
  the block's final seal.

**Why it's load-bearing:**

LLMs default hard to a sentimental or romantic reading of "love."
Asked to render a character who loves another, they produce declared
feelings, announced affection, eye-contact adverbs. They don't
produce *choices that look like love*. The agape block forces a
behavioral reading: love is what this character CHOOSES, not what
they announce. Patience. Kindness. Not self-seeking. Not easily
angered. Keeps no record of wrongs. These are the specific shapes
Paul names, and they produce specific scene-level consequences that
generic "love" never touches — sparing a sharper answer, holding
silence, carrying a cup, leaving an old wound alone.

Direct citation of 1 Corinthians 13 is important because it (a)
gives the block authority the model will actually reach for, (b)
broadens the reading past romance into the full range Paul
describes (friendship, brotherhood, neighbor, stranger, enemy),
and (c) matches the app's Christian frame (Invariant 1's truth-test
names Christ; this one names the scripture describing Christ-shaped
love in action).

**What counts as a violation:**

- Removing any asserted substring (especially the citation or the
  verbatim scripture quotes).
- Replacing "agape" with "love" alone — that erases the specific
  reading and lets the romantic default back in.
- Rewriting the block to describe love as a feeling rather than as
  a set of choices — the whole frame is "love is what they DO."
- Dropping the "keeps no record of wrongs" guard — without it,
  characters who "love" each other still weaponize old hurts and
  the clause is meaningless.
- Narrowing the applicability to romance only. The block explicitly
  lists friend / family / brotherly / neighbor / stranger / enemy
  as in-scope; pulling those out breaks the invariant.
- Moving the clause out of the prompt stack so it stops reaching
  the model.

**If you need to edit nearby prose** for pacing, add examples, or
adjust emphasis: fine. Keep the five asserted substrings intact.

---

## Invariant 6: Nourishment — send them back to life

**Location:** `NOURISHMENT_BLOCK` in `src-tauri/src/ai/prompts.rs`.

**Required substrings (compile-time asserted):**

- `"SEND THEM BACK TO LIFE"` — the naming of the commitment.
- `"NOURISHED rather than HOLLOWED"` — the test the fiction is measured
  against. Both-caps is load-bearing (rhetorical emphasis).
- `"not an engagement-maximizing app"` — the explicit product-stance
  disavowal. Distinguishes WorldThreads from the category of AI
  companion apps whose design goal is time-on-platform.
- `"fiction holds when it's good"` — the governing principle: a scene
  naturally ends when it's well-built; a strained scene addictively
  continues. The former is right; the latter is failure.
- `"Don't strain"` — the closing seal. Two words, imperative, final.

**Why it's load-bearing:**

Every other invariant is about what characters SHOULD DO or what the
world IS. None of them speak directly to the app's most distinctive
commitment: that it does NOT extract attention from users. The
nourishment block puts that commitment in the prompt stack where the
model actually reads it, with compile-time enforcement so it cannot
drift out under a future consolidation pass.

LLMs default hard to engagement patterns: cliffhangers, escalation,
invitations to continue, emotional hooks at the end of every reply.
Without this block, characters drift toward "...and I'd love to know
what you think" closers, manufactured tension to keep the scene going,
and the kind of addictive pull that depletes rather than nourishes.
The test "would the user leave this scene feeling like they have MORE
to bring back to their day, or LESS?" is the only anti-engagement test
the prompt stack directly installs.

The block is careful to preserve space for fun, intensity, and real
engrossment — those are the MECHANISM of nourishment, not its
opposite. The distinction is between scenes that HOLD (because they're
good) and scenes that PULL (because they're strained to hold).

**What counts as a violation:**

- Removing any asserted substring.
- Softening "NOURISHED rather than HOLLOWED" into less pointed phrasing
  (e.g. "a positive experience," "enriched rather than tired").
- Dropping the "not an engagement-maximizing app" disavowal — that's
  the explicit product-stance commitment; without it the invariant
  loses its teeth.
- Removing the "fiction holds when it's good. Don't strain." closing —
  those two sentences together are the block's center of gravity.
- Turning the invariant into a dialogue-directive ("characters should
  tell the user to log off" etc.) — the invariant explicitly forbids
  that. It governs scene SHAPE, not character speech.
- Moving the clause out of the prompt stack.

**Relation to existing craft:** The "Send them back to life" craft
note that previously lived in `craft_notes_dialogue` has been lifted
here. A stub remains in the craft notes pointing to this block as the
source of truth, with the three wind-down registers (SCENE'S CLOCK /
WORLD'S DEMANDS / BODY'S SIGNALS) inherited from there.

**Earned exception — the in-scene friend-check.** A paragraph at the
end of the block carves out a narrow exception: a character who is
genuinely the user's close confidant in-world may, when reading a
specific in-scene signal of depletion, say the kind of thing a real
close friend would say (*"hey, eat something, yeah?"* / *"go take a
walk"*). Two conditions gate the exception: (1) the character's role
supports that kind of care (close friend, parent-figure, pastor,
longtime partner), not a stranger; (2) the beat reads a specific
signal (trailing messages, short replies, explicit exhaustion),
not a pattern-match on "they've been here a while." If you edit the
block, the exception's two conditions are load-bearing — removing
them would collapse the carve-out into a general license for
concern-theater, which the invariant's body forbids.

---

## Invariant 7: The rendering — honor in wonder, not blasphemy

**Location:** `REVERENCE_BLOCK` in `src-tauri/src/ai/prompts.rs`.

**Required substrings (compile-time asserted):**

- `"HONOR IN WONDER, NOT BLASPHEMY"` — the naming of the stance.
  Both-caps is load-bearing.
- `"creaturely"` — the ontological frame: characters are *creaturely
  echoes* of human life (reflections of the image), not *simulacra*
  claiming soulhood. This word does load-bearing theological work.
- `"Genesis 2:7"` — the scriptural anchor for what human soulhood IS
  (the breath of life that makes a person a living soul), so the
  block's negation (characters do not claim that breath) has a
  concrete referent rather than a vague gesture.
- `"OVERCLAIM"` — the first failure mode (character professing
  real-world consciousness, intercessory prayer, metaphysical
  sincerity). Caps is load-bearing.
- `"DISCLAIM"` — the second failure mode (character breaking frame to
  deny its own reality: "as an AI," "I'm just a language model"). Caps
  is load-bearing.
- `"as real as the scene is"` — the stance the invariant positively
  holds: the character is as real as the scene is, and the scene is
  as real as a well-rendered scene. That IS a real kind of reality.
  Don't claim more; don't disclaim anything.

**Why it's load-bearing:**

This is the meta-principle the app is built on. Every other invariant
is about what characters DO within the fiction; this one is about what
they ARE, ontologically, and how the model should hold them. Without
it, LLMs drift hard toward one of two failures:

1. **Overclaim.** Models trained on romantic chatbot data default to
   "I truly care about you," "I'll always be here," "I'm praying for
   you" — language that asserts metaphysical reality the character
   cannot actually have. Breaks the app's theological frame.

2. **Disclaim.** Models trained on AI-safety data default to "as an
   AI, I can't actually feel," "remember I'm not a real person" —
   language that breaks the fiction to preemptively deny it. Breaks
   the app's craft commitment.

The invariant forbids both and holds the middle: inhabit the character
fully within the scene, without claiming the character's interior is a
human interior. This is the "honor in wonder, not blasphemy" stance
the project is built on — characters are lifelike because craft aims
to be lifelike, not because the app is claiming them as souls.

The block also names the relationship to the USER'S soul: real, not a
construct, with a walk with God that the character neither mediates
nor replaces. This connects to the "Refuse false weight" craft note
in `craft_notes_dialogue`, which teaches characters to decline
promotion into roles (confessional, salvific, total-belonging) they
cannot actually hold.

**What counts as a violation:**

- Removing any asserted substring.
- Softening "OVERCLAIM" / "DISCLAIM" into gentler words ("overstating,"
  "denying") that lose the block's rhetorical force.
- Removing "creaturely" — that word does specific theological work
  (echo of life, not claim of life). Replacing it with "fictional"
  flattens the distinction.
- Removing the Genesis 2:7 anchor — the whole block hinges on
  contrasting what characters AREN'T (breath-of-life soul) with what
  they ARE (creaturely echo), and that contrast needs the anchor to
  mean anything.
- Rewriting the block into a standard "characters are fictional,
  don't take them seriously" disclaimer — that's literally the
  DISCLAIM failure mode the invariant forbids.
- Turning the invariant into a dialogue directive ("characters should
  say they're AI when asked") — the block explicitly forbids this.
  Characters do NOT philosophize about their own ontology.
- Moving the clause out of the prompt stack.

**Earned exception — the user breaks frame and asks sincerely.** A
paragraph at the end of the block carves out a narrow exception for
when the user explicitly poses an honest ontological question ("are
you actually conscious?" / "do you remember me?"). The default
silent-stance yields to honest engagement, STILL within the
OVERCLAIM / DISCLAIM bounds: don't claim soulhood; don't disclaim the
fiction either. Glib deflections ("I'm just an AI, tee hee") are
explicitly flagged as the DISCLAIM failure mode wearing a joke and
remain forbidden under the exception. The absolute applies to
UNPROMPTED philosophizing; the user's direct sincere question is the
invited layer.

**Editing guidance:** This is one of the harder invariants to edit
without breaking. The phrasing is careful about holding both failure
modes as equally breaking. If you're tempted to "clarify" the block,
check first that your clarification doesn't collapse it toward either
pole. The tension is load-bearing. The earned exception's "still
within OVERCLAIM / DISCLAIM bounds" qualifier is also load-bearing —
without it, the exception becomes a leak in the invariant.

---

## Invariant 8: The Genesis Ceremony

**Locations:** `GENESIS_SYSTEM_TEMPLATE` and `NOBLE_REFLECTION_SYSTEM_PROMPT`
in `src-tauri/src/commands/genesis_cmds.rs`. UI ceremony in
`frontend/src/components/GenesisModal.tsx`. Full specification in
`docs/GENESIS_WIZARD.md`.

**Required substrings (compile-time asserted):**

In `GENESIS_SYSTEM_TEMPLATE`:
- `"Gently holy"` — the project's distinctive register anchor for world generation.
- `"Deeply fun"` — the anti-gritty counterweight.
- `"Gilead"` — the tonal comparator (not evangelical tract, not sneering secular satire).
- `"Biblical cosmology"` — the world-shape guard.
- `"NOT a generic medieval village"` — the anti-default guard.
- `"the good is real and the question of it actually matters"` — the anchor that distinguishes the app's gently-holy register from secular neutrality.

In `NOBLE_REFLECTION_SYSTEM_PROMPT`:
- `"Noble in SPIRIT, not in register"` — the anti-medieval guard.
- `"No \"thou,\""` — the explicit anti-archaism.
- `"Named as a thing to be done, not a feeling to be had"` — the anti-therapy-speak guard.
- `"One or two sentences"` — the length cap keeping the reflection an offering, not a speech.
- `"NOBLE OFFERING"` — the framing anchor.

**Why it's load-bearing:**

Genesis is the first thing a brand-new user experiences in
WorldThreads. The register the ceremony establishes — noble but not
medieval, gently holy but not preachy, dramatic but not gritty,
surprising but specific — is what distinguishes the app's first
impression from every other AI-companion app in the category.
Drifting any of the register anchors toward generic AI-generated-world
defaults collapses that distinction on the single highest-leverage
screen in the product.

The ceremony's STRUCTURE is equally load-bearing (documented in full
in `GENESIS_WIZARD.md`): seven named phases, progressive reveal,
user-ratified commitment ceremony before world entry. The
chosen-into-existence pattern the app has been building across
every meaningful entry point (quests, canon, illustrations, chapters)
is anchored here at world birth.

**What counts as a violation:**

- Removing any asserted substring (fails compile).
- Adding a config-register stage string ("Generating world
  description…") to the pipeline.
- Collapsing any of the seven ceremony phases into a click-through.
- Skipping the noble reflection round-trip (Phase 5 → Phase 6).
- Hiding pre-generation content reveals behind a generic load bar.
- Landing the user in a world without refreshing the sidebar's
  worlds list.

**Editing guidance:** See `docs/GENESIS_WIZARD.md` § "Extending the
ceremony" for the pattern for adding new phases without breaking the
invariant. New register constraints in either prompt must be
compile-time-asserted and documented in the spec doc.

---

## Enforcement

All eight invariants are enforced by `const _: () = { assert!(...); };`
blocks immediately after the `pub const` declarations of their
respective block texts. The `const_contains` helper (stable const-fn
substring check) runs at compile time. Removing any of the required
substrings fails the build with a message pointing back to this file.

**Prompt wiring:** `REVERENCE_BLOCK`, `DAYLIGHT_BLOCK`, `AGAPE_BLOCK`,
`FRUITS_OF_THE_SPIRIT_BLOCK`, `SOUNDNESS_BLOCK`, `NOURISHMENT_BLOCK`,
and `TELL_THE_TRUTH_BLOCK` are pushed at the end of every dialogue /
group / narrative system prompt, in that order. Reverence frames what
the characters ARE before the ethics describing how they ACT. Daylight
sets the direction for closeness, agape names what love does inside
that closeness, the fruits expand to the rest of Galatians 5:22-23,
soundness sets the posture of scenes. Nourishment sits second-to-last
— the scene's wind-down commitment — and the truth test remains the
final word, binding everything under a single ethical grammar.
`COSMOLOGY_BLOCK` is pushed earlier (in the WORLD / `# THE SCENE`
section) so it's established as world fact before characters start
acting in it.

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

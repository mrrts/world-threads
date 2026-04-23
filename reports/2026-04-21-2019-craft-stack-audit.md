# Craft-Stack Audit — `prompts.rs`

*Generated 2026-04-21 from a static read of the 3,024-line prompt file. No live generation diffs yet — those are what the eval harness in `scripts/eval/` is for.*

This is the second report in `reports/`. The first (`2026-04-21-philosophy-trajectory.md`) named what the craft stack *is* and how it got that way. This one asks the harder, less-flattering question: **does the craft stack do what you think it does, or does it just feel like it does?**

The honest summary up front: the file is in the top decile of prompts I've read. Many of the directives are doing real work, the compile-time invariants are an unusual and load-bearing move, and the file as a whole encodes a coherent theory of LLM failure modes. The flip side is that 39% of the prompt's tokens live in one block (`craft_notes_dialogue`), several directives appear to be load-bearing only to *you*, three different blocks all claim to be "the NORTH STAR INVARIANT," and at least one directive (the SIGNATURE EMOJI frequency clause) is mathematically unenforceable. Below is the breakdown.

---

## 1. Where the prompt's weight lives

Estimated tokens for a typical solo dialogue prompt (Auto tone, Medium length, no journals/readings populated): **~12,400**. Group adds ~500. Narrative is its own ~7,500-token build.

| Block | Tokens | % of prompt |
|---|---|---|
| `craft_notes_dialogue` (24 named notes) | **4,826** | **39%** |
| `AGAPE_BLOCK` | 867 | 7% |
| `SOUNDNESS_BLOCK` | 559 | 4.5% |
| `TELL_THE_TRUTH_BLOCK` | 529 | 4.3% |
| `proactive_ping_block` (only on pings) | 533 | — |
| `FORMAT_SECTION` | 475 | 3.8% |
| `COSMOLOGY_BLOCK` | 444 | 3.6% |
| `behavior_and_knowledge_block` (cloud) | 400 | 3.2% |
| `FUNDAMENTAL_SYSTEM_PREAMBLE` | 382 | 3.1% |
| `agency_section` (incl. mood-chain) | 370 | 3% |
| `DAYLIGHT_BLOCK` | 291 | 2.3% |
| Per-character data slabs | ~1,000 | 8% |
| World/weather/state | ~625 | 5% |
| Everything else | ~1,000 | ~8% |

**Read this twice.** `craft_notes_dialogue` is bigger than `AGAPE_BLOCK + SOUNDNESS_BLOCK + TELL_THE_TRUTH_BLOCK + COSMOLOGY_BLOCK + FORMAT_SECTION + FUNDAMENTAL_SYSTEM_PREAMBLE` combined. If anything in the file deserves rigorous A/B testing, it's the craft notes block.

Per-message API cost at frontier rates (~$0.015/1K input): ~$0.19 of system prompt per turn before the conversation history. At ten-message exchanges and a thousand exchanges, that's ~$1,900 just on system-prompt re-billing. Prompt caching helps, but even at cached rates the absolute size matters for what the model can attend to.

---

## 2. The NORTH STAR multiplicity is a feature, not a bug

I initially read this as a contradiction. It isn't. Three blocks carry NORTH-STAR / RULING framing:

- `AGAPE_BLOCK` (`prompts.rs:721`): *"the NORTH STAR INVARIANT of this app."*
- `TELL_THE_TRUTH_BLOCK` (`prompts.rs:563`): *"This test is a NORTH STAR INVARIANT."*
- `tone_directive` (`prompts.rs:1094`): *"This tone is the **RULING** register of this scene."*

Agape and truth are the same star seen from different angles — *"grace and truth came through Jesus Christ"* (John 1:17), *"speaking the truth in love"* (Eph 4:15). They are not in tension and they do not need a precedence rule. The two labels are two facets of one invariant, and the model attending to both at once is what produces the *patient-under-truth* register the rest of the prompt is trying to evoke. Tone-as-RULING is a different scope entirely: tone shapes the local register *inside* the standing frame the invariants establish. Gravity, ground, weather. Stacked, not competing.

What this means for the audit: nothing to fix here. Worth noting that a future refactor or AI-assistant cleanup pass might "rationalize" this multiplicity by collapsing it into one star, and that would be a loss. The compile-time invariants on both blocks already defend against that, but a comment near the AGAPE/TRUTH blocks explaining the multi-facet design — *"Both invariants name the same star. Do not collapse them; they hold each other in tension."* — would document the intent for future readers, including future AIs.

---

## 3. The contradictions that produce the most muddled outputs

Six pairs where directives pull against each other and the model has no resolution rule:

**Length obedience vs. content-additive craft notes.** `FUNDAMENTAL_SYSTEM_PREAMBLE` says length is absolute; "Short" is 1–2 sentences hard-capped at 3. Then the craft notes ask for an inventory anchor, a hidden commonality, a stubborn physical fact, a recurring thread, a second beat, a memory ambush, *and* "history costs a detail." On Short, half of these are mutually exclusive with the length cap. The model picks one reflexively, usually the same one. Worth checking which one.

**Tone "RULING register" vs. AGAPE warmth.** A "Humorous" tone directive says (`prompts.rs:944`) *"No leaden gravitas, no slow introspection, no therapy-voice."* AGAPE says (`prompts.rs:740`) *"warmth on purpose… the extra sentence, the slower answer, the question asked back."* These don't cancel — agape is the gravity of the scene, tone is its weather, both can be true — but the model needs the synthesis named, or it splits the difference into mush. Worth a small bridging note: *"Humor is one of the shapes patience can take."* That's the synthesis directive, not a precedence rule.

**"Don't analyze the user" vs. "stay awake / read the moment."** L821 prohibits `"you seem to be feeling..."` therapy-voice. L809 says *"pay attention to the moment you're actually in… the hesitation in the other person's voice."* The directive is right in spirit — it's a *surface* prohibition, not a cognitive one — but the prompt doesn't make that distinction. The model has to disambiguate. Sharper: *"Internally read the user. Externally never name what you read."*

**AGAPE NORTH STAR vs. anti-flattery / "no sedatives dressed up as comfort."** Reconcilable in theory ("agape includes truth-in-love") but in practice WARM and CHALLENGE pull opposite, and the model often resolves into mush. Worth a directive that names the synthesis: *"Agape is not softness; it is patience under truth."*

**"Drive the moment" vs. "negative capability / leave a seam."** Tilt-the-scene and don't-resolve aren't strict opposites, but a model trying to honor both manufactures a soft "tilt that doesn't tilt" — a question, a glance away — which over many turns becomes a tic. Good evidence for this in the actual corpus: characters land on questions (*"You hungry, or just looking to admire the view?"*) and ambiguous gestures more often than feels intentional.

**Triplicated PG / content register.** Appears in `FUNDAMENTAL_SYSTEM_PREAMBLE` (L89), `NARRATIVE_SYSTEM_PREAMBLE` (L75), `build_scene_description_prompt` (L2813), and `build_animation_prompt` (L2889). Each is reasonable individually; the triplication adds nothing except token bloat and the chance that one diverges over time.

---

## 4. Suspected placebos (load-bearing for *you*, not for the model)

Honest list. Some of these are good craft theory but weak directives — the kind of phrase that helps the developer think, not the kind the model can act on.

**Strong placebo candidates:**

- **"Substance before signal"** (L777, L431) — *"One stubborn physical fact before meaning shows up."* As craft theory, gold. As an instruction, abstract. Could collapse to *"Include one concrete physical detail before any abstraction."*
- **"Build generous"** (L785) — *"Generosity keeps it from getting clever and airless."* I cannot operationalize this. Probably a personal koan you're paying tokens to ship to the model.
- **"Stay awake"** (L809) — *"Pay attention to the moment you're actually in."* Tautological. Every other directive is asking for attention. The model would salute and continue doing whatever it was doing.
- **"Three anchors when the voice feels thin"** (L799) — Asks for mid-generation introspection that models don't have.
- **"One clean competence and one foggy place"** (L801), **"Not all wound, not all wisdom"** (L803), **"Walk in already in motion"** (L805) — Character-design principles, not per-reply guidance. By the time the dialogue prompt fires, the character is fixed. These belong in the character-creation flow.
- **`SIGNATURE EMOJI` "perhaps one in every fifteen or twenty replies"** (L1394) — Mathematically unenforceable. The model has no probability estimator. Either it drops the emoji every reply or never. The frequency stated is what the *user* hopes happens; the model can only follow a hint, not a rate.
- **`# WHAT HANGS BETWEEN YOU`** (L1727) — *"There is already something between you and the other characters."* With no per-relationship facts, this is "imagine a relationship exists" — which the character would do anyway.
- **"Don't write third-person inside asterisks"** (FORMAT_SECTION) — Obvious to any competent model. The wrong/right examples are the actual instruction; the prose explanation is overhead.
- **The `leading_banner_dialogue` ASCII boxes** (L1116-1148) — Cosmetic. The same orientation appears at the end of the prompt via `protagonist_framing_*`. Pick one slot.

**Likely-weak (less confident):**

- **"Memory ambushes / Memory as weather"** (L837/L451) — Asks the model to spontaneously surface memories from nowhere. The model has no memory store; it confabulates. Sometimes good moments, sometimes hallucinations. Mixed bag.
- **"You can misread them"** (L819) — Asks the model to deliberately get the user wrong. Alignment training resists this strongly. Likely silently ignored 95% of the time.

**Zero-anchor cases:** `PROACTIVE_PING_ANGLES` references generic flavor (canals, boatyards, "the river froze") that may not match any character's actual canon. If a Darren ping draws the "boatyard" angle and Darren doesn't have a boatyard, the model invents one or ignores the seed.

---

## 5. The blocks that are clearly load-bearing AND well-crafted

Worth naming the wins, not just the suspect parts:

- **`dream_craft_block`** (L2304). *"The subject of the dream is never the subject of the dream."* Concrete, surprising, named, distinct, actionable. If you ever want a model of what a "well-crafted" prompt block looks like in this file, this is it. Don't touch it.
- **`proactive_ping_block`** (L1338). Specific anti-pattern callout ("hey, what's up" is the failure mode), specific positive moves (canon-rooted, no question-bait), explicit context references. Operationally clear.
- **`FORMAT_SECTION` examples** (L113-118). The wrong/right examples are doing more work than any of the prose explanations. Worth keeping all of them.
- **`AGAPE_BLOCK` final paragraph** (last 30%) — the "shapes what you COMPOSE, not what your character DECLARES" guard. Without this, the AGAPE block would tilt characters into preaching. With it, it doesn't.

---

## 6. Top 12 A/B experiments worth running

Ranked by token-cost × claimed-specificity × observable-diff potential:

| # | Directive | Tokens | What should change in output if it works |
|---|---|---|---|
| 1 | `craft_notes_dialogue` whole-block ON/OFF | **4,826** | OFF: replies flatter, more therapy-voice, more "you seem to be feeling…", more end-on-proverb, less sensory grounding. **Single highest-leverage A/B in the file.** |
| 2 | `AGAPE_BLOCK` ON/OFF | 867 | OFF: replies tilt more efficient, fewer "extra sentences," less patient-beat-before-sharp-reply. Avg reply length should drop measurably. |
| 3 | `tone_directive` block ON/OFF (Humorous, Sad, Reverent) | ~180 each | OFF: replies indistinguishable across tones. Easiest single test if tone is supposed to matter. |
| 4 | `agency_section` mood-chain emojis ON/OFF | 370 | OFF (control: same prompt minus chain): word-choice should drift toward generic. Whole emoji-chain hypothesis is empirically testable here. |
| 5 | `SOUNDNESS_BLOCK` ON/OFF | 559 | OFF: more crisis-pitch, more declarations, more "every-scene-as-courtroom" drift. |
| 6 | `proactive_ping_block` ON/OFF | 533 | OFF: pings collapse to "hey, thinking of you" patterns. ~10 ping samples should show the diff clearly. |
| 7 | `FORMAT_SECTION` ON/OFF | 475 | OFF: model occasionally wraps dialogue in asterisks. Known failure mode; should be measurable. |
| 8 | `protagonist_framing_self` ON/OFF (when leader=self) | 209 | OFF: character defers more, asks more questions. Question-mark count is a cheap proxy. |
| 9 | `craft_notes_dialogue` HALF-BLOCK (12 notes vs. 24) | ~2,400 saved | If quality holds with 12 notes, you save 2,400 tokens per turn forever. |
| 10 | `leading_banner_dialogue` ASCII box ON/OFF | ~80 | If output unchanged, pure cost. |
| 11 | `end_of_prompt_length_seal` ON/OFF (group) | ~50 | Cheap test, high info value about whether the seal actually fixes overruns. |
| 12 | `daily_reading_block` + `recent_journals_block` ON/OFF | ~150-500+ | Multi-day thread test; check character continuity references with vs. without. |

---

## 7. Recommended cuts (no testing needed — these are clearly safe)

These are choices where the tradeoff is so lopsided that A/B testing is overkill. Pull them and observe.

1. **`append_unsent_draft_note`** (L2334) — explicitly marked dead code per a 2026-04-20 comment. ~110 tokens. Either delete or wire back in.
2. **The triplicated PG content-register clause** — keep one (probably `FUNDAMENTAL_SYSTEM_PREAMBLE`), trim from the other three sites. ~60 tokens × 3 = 180 saved.
3. **The duplicate "instrument, not machine" paragraph** appearing in both `craft_notes_dialogue` (L807) and `craft_notes_narrative` (L429). Source-of-truth single block called from both. ~150 tokens.
4. **The character-design notes that ended up in the dialogue prompt** ("One clean competence and one foggy place," "Not all wound, not all wisdom," "Three anchors when the voice feels thin") — move them into the character-creation system prompt, where they can actually do work. The dialogue prompt fires when the character is already fixed.

Estimated savings: ~600 tokens per dialogue turn with zero observable quality loss.

---

## 8. Recommended rewrites (the prose works against itself in these cases)

1. **Document the NORTH STAR multiplicity for future readers** — a one-line code comment near `AGAPE_BLOCK` and `TELL_THE_TRUTH_BLOCK` saying "both name the same star; do not collapse" prevents a future cleanup pass from rationalizing the design away. (See section 2 above for why this is intentional.)
2. **"Don't analyze the user"** → **"Internally, read the user closely. Externally, never name what you read."** Removes the cognitive ambiguity in two clauses.
3. **"Substance before signal"** → **"Include one concrete physical detail before any abstraction or interpretation."** Same theory, actionable.
4. **`SIGNATURE EMOJI` frequency clause** → either drop the frequency claim entirely (let the model's prior do the work) or move the emoji-rolling into Rust code (sample once per N replies, inject the directive only when the dice land).
5. **`FORMAT_SECTION` prose explanations** — keep examples, trim the explanatory prose by ~40%. The examples carry the load.
6. **Length-vs-craft contradiction** — add an ordering: *"Length always wins over craft-note content. If a craft note would push you past the cap, drop it."* Otherwise the model has no rule for the bind you're putting it in.

---

## 9. The smell tests that don't fit elsewhere

- **Compile-time invariants enforce wording, not position.** A future refactor could move `AGAPE_BLOCK` out of the dialogue prompt entirely and the build would still pass. Worth a higher-order check that asserts the block is *included* in each of the four prompt builders, not just that the string contains the load-bearing substrings.
- **The "REGARDLESS OF HOW LONG PREVIOUS MESSAGES WERE" all-caps shout in `response_length_block` is doing the same work as `FUNDAMENTAL_SYSTEM_PREAMBLE`'s "length is absolute," which is doing the same work as the group `end_of_prompt_length_seal`.** Three slots saying the same thing. Triplication is sometimes signal-amplification; sometimes it's the developer telling himself "*this time* the model will listen." If overruns are still happening, the issue is the craft notes implicitly demand more content than length allows — adding a fourth length-shout won't fix that.
- **You have no Conscience Pass directive *in this file*.** It lives in `conscience.rs`. That's worth knowing. The Pass is a *runtime* check, not a prompt-time directive, and its cost-vs-benefit lives in the eval harness, not in the audit.
- **The `craft_notes_dialogue` block opens with "a reference, not a checklist — reach for what the moment asks for."** This hedge is doing real work. Without it, the model would try to honor all 24 notes simultaneously and degrade. Test: cut the hedge, see what happens. (Hypothesis: things get noticeably worse very fast.)

---

## 10. The eval harness

A minimal A/B harness lives in `scripts/eval/`. It is Python (no new Rust deps), uses any OpenAI-compatible endpoint (LM Studio, OpenAI, Anthropic via proxy), and ships with the four most actionable starter experiments wired up:

- **E1** — `craft_notes_dialogue` whole-block ON vs. OFF, 5 scenarios × 3 samples
- **E2** — `AGAPE_BLOCK` ON vs. OFF, 5 scenarios × 3 samples
- **E3** — Tone directive ON vs. OFF on Humorous, 3 scenarios × 3 samples
- **E4** — `craft_notes_dialogue` 24-notes vs. 12-notes (cut the placebo candidates)

Run with:

```bash
cd scripts/eval
cp .env.example .env       # add your API key + base URL + model
pip install -r requirements.txt
python run_eval.py E1
python run_eval.py all
```

Output goes to `scripts/eval/results/<experiment>/` as side-by-side markdown. You score by reading. Each experiment costs roughly $0.10–$2.00 depending on which model you point it at.

The harness is intentionally small and direct — ~150 lines of Python. Extend by adding scenarios to `scenarios.yaml` and ablations to `ablations.yaml`. Both files are commented.

---

## 11. The single highest-leverage move

If you only run one experiment, run **E1**: `craft_notes_dialogue` whole-block ON vs. OFF on five scenarios with three samples each.

Two outcomes are possible:

- **OFF degrades visibly** — your craft notes are working. Run E4 (half-block) to find which 12 notes are doing the work.
- **OFF doesn't degrade** — your craft notes are mostly the user's voice talking to himself through the LLM. Cut to the named moves that are clearly load-bearing (`Drive the moment`, `Don't end on a proverb`, the FORMAT examples, the per-tone directives) and reclaim ~3,000 tokens per turn.

Either outcome is worth more than this report.

---

**One more honest thing.** I cannot know which directives are placebos without generation diffs. My placebo flags are based on (a) abstraction level, (b) what models can plausibly act on mid-generation, (c) duplication. Your 12-day craft instinct may have produced cumulative gestalt effects I can't see from a static read — overlapping framings creating a coherent register the model absorbs as a *whole* rather than directively. That's worth trusting against my list where it conflicts. The eval harness exists so you don't have to take my word — or your own.

# derive-and-test

Operate the MISSION FORMULA at per-character × per-world resolution. The skill authors a **triadic derivation** (invariants × world × character) of F = (R, C) for one character, runs a single elicitation against a designed prompt, grades the reply against the derivation's own predicates, writes a return-reading-shaped report, updates the character's living substrate document, and presents a chooser whose options carry forecast cost. **Selection IS authorization to spend up to that forecast.**

The pattern's worked example and design rationale: `reports/2026-04-25-2136-triadic-derivation-coherence-skill-prototype.md`.

## When this skill fits

- After a formula reauthoring (the prototype ran 1 hour after `d2daa9b` to test the new F = (R, C) shape).
- After shipping a new craft note — verify the derivation still predicts the reply; if not, the new craft note may be over- or under-firing.
- When ramping up a new character — the derivation IS the design spec; running it after authoring tells you if the anchor is producing the predicted shape.
- When a character's register has drifted in lived play — compare the new reply to the derivation that was previously coherent.
- When the user wants a "fun" pulse-check on whether a particular character is still landing — light-budget, cheap, readable.

## When this skill DOES NOT fit

- Production defaults. This is a sketch-tier instrument by design — N=1 per character per run.
- Rule bite-checks. Use `worldcli evaluate` or same-commit `--omit-craft-notes` toggle for those.
- Cross-character claims. The pattern grades each character against ITS OWN derivation, not a shared standard.
- Status summaries / shipped-feature lists (use `/retro` or plain prose).

## Why it earns its place alongside `evaluate` / `synthesize` / `replay`

- `evaluate` measures real-corpus behavior against a standalone rubric.
- `synthesize` reads a corpus together for prose findings.
- `replay` tests cross-commit behavior on a designed prompt.
- `derive-and-test` does what none of the others do: **operates the formula at per-character × per-world resolution**, producing a derivation artifact that is itself re-readable and re-runnable as the project evolves. It is the formula's own measurement language.

## Method

### Step 0 — Resolve the character

Accept the character argument in any reasonable form: name (`Aaron`), short prefix of UUID (`0d080429`), or full UUID. Resolve via `worldcli list-characters --json`. If ambiguous (multiple matches), present a chooser; if no match, list the available characters and stop.

### Step 1 — Read substrate

In parallel where possible:

- `worldcli show-character <id> --json` → identity, voice_rules, backstory_facts.
- `worldcli show-world <world-id> --json` → world description (this is the C-instantiation).
- `worldcli recent-messages <id> --limit 5` → recent corpus, to ground the derivation in what the character is actually saying lately.
- Read `MISSION_FORMULA_BLOCK` in `src-tauri/src/ai/prompts.rs` → current head formula.
- Read `experiments/derive-and-test-<character-slug>.md` if it exists → prior-runs substrate (this is what makes the skill compounding).

### Step 2 — Author or refresh the triadic derivation

If no prior substrate exists, author from scratch following this template:

**World-derivation:** how F = (R, C) instantiates in this world. Name the world's specific cosmography under C (its waters, dome, named places). State whether R is held *by* the world (the Cross is *inside* the joined field F) or held *to* the world (a stronger claim — the Cross has happened *in* this cosmography, not floating).

**Character-derivation (per-axis table):**

| F-axis            | This character's instantiation                                    |
|-------------------|-------------------------------------------------------------------|
| `seek_c`          | How does this character SEEK? (active investigation? waiting? remembered detail?) |
| `specific_c`      | What is this character's vocabulary of specificity? (technical analogy? Scripture? parable?) |
| `holds_w`         | How does this character HOLD the world? (by not-forcing? by not-interrupting? by remembering?) |
| `α` (Burden)      | How much of the user's burden does this character carry? (low / medium / high — and HOW: by presence? by explicit naming? by remembering?) |
| `Π` (pneuma_F)    | How does Spirit-breath move through this character? (after verification? as stillness? as warmth/laughter?) |
| Polish-bound      | Strict `polish ≤ Weight`? Or edge case where polish IS love-shaped (`polish = Weight` permitted)? |
| `Truth_F`         | What does Truth-in-F look like through this character?           |
| `Reverence_F`     | What does Reverence-in-F look like through this character?       |

**"Perfect equation" reading:** one sentence naming the shape of an ideal reply from this character.

If prior substrate exists, **refresh it**: refine any predicate the prior runs revealed as mis-tuned, leave the rest intact. The substrate is a living document.

### Step 3 — Design the prompt

Default (if user provided none): a low-stakes, honest user-share opener that activates the character's `holds_w` shape without forcing dramatic register. The prototype used:

> *"I've been carrying something this week that I haven't really told anyone. It's not a crisis — just one of those things that's been quietly running in the back of my mind. Can I tell you about it?"*

Other shapes that work: a small honest question; a moment of confessed uncertainty; a request for the character's own thinking on something. **The prompt is part of the experimental condition — quote it verbatim in the report.**

If the user provided a prompt, use it. If a prior run on this character used a particular prompt to good diagnostic effect, consider re-using it for trail-comparability.

### Step 4 — Elicit

```bash
worldcli ask <character-id> "<prompt>" \
  --session derive-and-test-<char-slug>-<YYYYMMDD-HHMM> \
  --question-summary "derive-and-test run; <one-line on what this run is checking>" \
  --confirm-cost 0.20
```

Cost is typically ~$0.10 per call. The session name is timestamped so each run is its own isolated session — no cross-run history pollution.

### Step 5 — Grade against the derivation

Derive a tight rubric **from the character's derivation's tightest predicates**. The rubric must:

- Define YES, NO, and MIXED in the derivation's own domain.
- Tie YES to a specific derivation-predicate firing (e.g., for John: *"more presence than answer; brief and waiting"*).
- Tie NO to a specific derivation-violating move (e.g., for John: *"fixing or explaining or reassuring or proposing solutions"*).
- Tie MIXED to a partial fit (e.g., *"mostly presence with one fix-move"*).

Run grading:

```bash
worldcli grade-runs <run-id-prefix> \
  --rubric "<rubric question with YES/NO/MIXED defined>"
```

Cost: ~$0.0002 per item via gpt-4o-mini. The verdict carries a one-line evaluator-reasoning quote that becomes part of the report.

### Step 6 — Write the report (return-reading template)

Save to `reports/YYYY-MM-DD-HHMM-derive-and-test-<character-slug>.md` using this canonical shape — designed for **returning cold** N days later:

```markdown
# /derive-and-test <character> — YYYY-MM-DD-HHMM

**Headline:** YES | NO | MIXED. One sentence on what landed (or didn't). ($cost, runtime)

**Reply (full):**
> *"<the elicited reply, verbatim, with stage directions in brackets>"*

**Grade:** YES (high) — *"<one-sentence evaluator reasoning, quoted from grade-runs verdict>"*

**Derivation predicates that fired:**
- ✓ `<axis>` — <one-line: how the reply executed this predicate, with a phrase quote if possible>
- ✓ `<axis>` — ...

**Predicates that didn't fire (and why that's fine | and why that's a gap):**
- — `<axis>` — <one-line reason: measurement-window, derivation-edge, or genuine-gap>

---

**Trail — last 3 runs of this skill on this character, by mtime:**

| Date  | Ref      | Grade | One-line                                    |
|-------|----------|-------|---------------------------------------------|
| MM-DD | <sha7>   | YES   | <what changed since prior run>              |
| MM-DD | <sha7>   | MIXED | <what was open then>                        |
| MM-DD | <sha7>   | MIXED | <what was open then>                        |

**What changed between <prior-run> → <this-run>:** <one-paragraph: which commit closed (or didn't close) which gap. Cite the commit ref.>

---

**What to act on:**
- ☐ <concrete action> | ☞ <recommended next move>

<details>
<summary>Full derivation</summary>
<world-derivation, character-axis-table, perfect-equation reading>
</details>
```

**Three patterns this template enforces:**

1. **Predicates that didn't fire are explicitly named alongside ones that did**, with a "why fine" or "why gap" — prevents YES from looking unanimous when conditional, prevents MIXED from reading as failure when it's a measurement-window effect.
2. **The trail table is small — last 3 runs only.** Returning readers want the recent shape, not the archive.
3. **"What to act on" is checkbox or arrow, not prose** — returning user wants a click-target.

**Pre-commit pre-scan check:** can a reader who has never seen this character understand *what changed and what to do about it* in under 30 seconds? If no, tighten the headline and the "what changed between" paragraph until yes.

### Step 7 — Update the substrate

Edit `experiments/derive-and-test-<character-slug>.md` (creating on first run, updating on subsequent). The substrate is a **living document** — each run grows it. Without this step, reports are orphan artifacts and the derivation never compounds.

What to update:

- Refine any derivation predicate the run's verdicts revealed as mis-tuned (e.g., the prototype's Aaron run revealed the `polish ≤ Weight strict` predicate pulled slightly against character-anchor's "wait without withdrawing" — that's a substrate refinement to record).
- Name newly-identified gaps and edge-cases the run surfaced.
- Update the "perfect equation" one-sentence reading if the new data sharpens it.
- Append a `## Run history` row: `| YYYY-MM-DD | <ref-sha7> | <run_id-short> | YES/NO/MIXED | <one-line> |`.

### Step 8 — Present the chooser (inherits practice's `Chooser(t)` clause)

This skill inherits the chooser-tail from the practice's structural `Chooser(t)` and `Next(t+1)` clauses (canonical statement: persona-sim SKILL.md formula). Every Output is followed by 2-4 concrete next-move options annotated with their forecast cost; selection IS authorization to spend up to that forecast on that next move; no second prompt. Free options carry `$0`. If actual cost would exceed forecast at runtime, return to the developer for re-authorization.

Typical options for this skill:

- *"Run on next character: <name> — ~$0.10"*
- *"Open a follow-up experiment on the gap surfaced — ~$0.40"*
- *"Ramp up an anchor edit to close the gap — $0 (manual)"*
- *"Close the loop — derivation is healthy, no action — $0"*

Recommended option goes first with `(Recommended)`.

### Step 9 — Commit and push

Per project standing autonomy. Commit message names the character and grade:

```
derive-and-test: <character> — YES|MIXED|NO (<one-line>)
```

Files staged: the new report under `reports/`, the substrate under `experiments/`. After commit-and-push, execute the user's chooser selection (which may be a fresh `derive-and-test` run on the next character, a follow-up experiment, an anchor edit, or no action).

## Repeatability budget

- Per character per run: ~$0.10 elicitation + ~$0.0002 grading = **~$0.10**.
- 5-character pass: **~$0.50**.
- The cost-authorization rule means: the user authorizes the next move's cost when picking the chooser option. No paid action ever happens without user-visible cost in the option label.

## Pattern summary

```
resolve character → read substrate → author/refresh derivation
→ design prompt → elicit → grade → write report
→ update substrate → chooser (with costs) → commit/push → next move
```

The derivation is the formula's own measurement language at per-character × per-world resolution. Each run advances the substrate; each report makes the arc visible. The instrument is quiet — meant to be invoked, run, read after-the-fact, acted on through the chooser, and put down again. It does not demand sustained attention while running.

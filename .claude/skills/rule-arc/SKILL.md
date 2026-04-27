# rule-arc

Take a single craft-shape observation from the project's lived life and walk it
through the full arc that turns observations into structural progress on
𝓕 := (𝓡, 𝓒): ask-the-character → registry entry → bite-test → honest tier
assignment → documentation triangle → optional structural improvement.

The arc is not a checklist. It is a **discipline-shaped loop** that keeps the
project's craft-rules honest about which ones have earned their place and which
ones are still authorial commitments waiting on evidence. Tonight (2026-04-27)
six rules walked the arc; only one earned `Characterized`. Five honestly earned
`EnsembleVacuous`. Both outcomes are progress because both produced doctrine.

The arc's worked example and rationale:
`reports/2026-04-27-1402-craft-rules-registry-arc.md`.

## When this skill fits

- A `/play` wince-read, a `/take-note` Mode 1, or a lived-play moment surfaced
  a craft gap and you want to convert the observation into structural progress.
- A character articulated something craft-shape in an `ask-the-character` reply
  and the line is leverage-bearing enough to belong in `CRAFT_RULES_DIALOGUE`.
- A rule already in the registry sits at `Unverified` or `Sketch` and an
  in-flight design decision wants its tier sharpened.
- A rule sits at `TestedNull` from a single-character run and you want to
  promote-or-honestly-label via cross-character probes.
- You want a worked exemplar of the arc to teach a future session how rules
  earn their tier here — running it explicitly produces both the rule and the
  exemplar.

## When this skill DOES NOT fit

- The observation is bug-shaped, UX-shaped, or feature-shaped, not craft-shape
  — fix the bug, ship the feature, do not dress it as a rule arc.
- The rule already sits at `Characterized` and lived-play hasn't surfaced a
  reason to re-test — the arc is for earning tier, not re-running it.
- You're testing whether a doctrine paragraph (CLAUDE.md section) is biting.
  Doctrine paragraphs travel with the project, not with `CRAFT_RULES_DIALOGUE`;
  they don't ship through this arc.
- App-wide invariants (MISSION FORMULA, COSMOLOGY, AGAPE, REVERENCE,
  TELL-THE-TRUTH, DAYLIGHT, NOURISHMENT, SOUNDNESS, TRUTH-IN-THE-FLESH) — those
  earn their place by theological/architectural commitment, not by per-rule
  bite-tests.
- The user is mid-conversation about something else and the rule-arc would
  derail the thread — defer until invited.

## Cost model

A full single-character arc costs ~$0.50-1.00 in worldcli calls (paired N=5+5
on `ask` is ~10 calls × ~$0.04). A cross-character validation pass adds
~$0.50-1.00. A synthetic-history probe adds ~$0.20-0.50. A paired-rubric
grading pass against `grade-runs` adds ~$0.10-0.20. Full arc: ~$1-2 per rule.

The doctrine, OBSERVATIONS.md, and registry-edit work add no API spend.

## The arc

### Step 0 — Identify the source observation

The arc starts from one of three doors:

- **Wince-read door.** A `/play` transcript-read named a specific line that
  felt off. Quote the line. Name the failure mode in one sentence.
- **Lived-play door.** A `/take-note` entry, an `OBSERVATIONS.md` line, or a
  conversation Ryan describes captured a recurring shape. Name what's
  recurring. Name what would fix it.
- **Tier-promotion door.** A registry entry sits at `Unverified`/`Sketch`/
  `TestedNull`. Read its `provenance` and `body`; name the failure mode the
  rule was supposed to address; the rest of the arc tests whether it does.

If none of those three doors is open, do NOT invent one. The arc's value comes
from the rule answering to a real observation, not from pre-imagined craft.
Pre-imagined rules earn `Unverified`-and-stay-there; their place in the
registry is honest only because the tier label is honest.

### Step 1 — Ask-the-character (when the door is wince-read or lived-play)

Lift the rule from inside the source register, not from outside it. Per the
"Ask the character" doctrine in `CLAUDE.md`: write the question as an
**in-world** beat — story-driven, conversational, the kind a friend might ask
mid-scene. Never "world engine" / "system prompt" / "describe to my LLM."

Run via `worldcli ask <char-id> "<in-world question>" --session
rule-arc-<slug> --question-summary "extracting craft note for <slug>"`.

Quote the answer back into the rule's eventual `provenance` field — that
preserves the line's authorship inside the source's voice.

If the door is tier-promotion, skip this step (the rule already exists; the
provenance is already attached).

### Step 2 — Author the CraftRule entry

Open `src-tauri/src/ai/prompts.rs`. Append a new entry to
`CRAFT_RULES_DIALOGUE`:

```rust
CraftRule {
    name: "<short-slug>",                    // kebab-case, unique
    body: r#"<rule body — substrate only>"#, // the prompt-facing text
    evidence_tier: EvidenceTier::Unverified, // honest starting tier
    provenance: "<source-quote or one-line origin>",
    last_tested: None,
},
```

**Substrate ⊥ apparatus discipline (load-bearing).** The `body` field carries
ONLY substrate — what the model should do. Evidence claims, citations, test
results, dates, and references to other rules belong in `provenance`,
`evidence_tier`, and `last_tested`. **Do not** put `Evidence:` lines or
citation prose inside the body. The model reads the body; the documentation
triangle reads the metadata. Crossing the line was caught and corrected
2026-04-27 (substrate-vs-apparatus leak); the registry pattern exists in part
to make the line structural.

Verify build: `cd src-tauri && cargo build --bin worldcli`. Fix compile errors
before proceeding.

### Step 3 — Bite-test (the arc's load-bearing center)

The rule's place in the registry is honestly held only when its tier label is
backed by evidence. Three bite-test layers, used as needed:

**Layer A — same-commit paired probe (always run).** N=5 with the rule ON,
N=5 with the rule OFF, same character, same prompt:

```bash
worldcli ask <char-id> "<elicitation prompt>" --session rule-arc-<slug>-on
# repeat 5 times — or use a small bash loop

worldcli ask <char-id> "<elicitation prompt>" \
    --omit-craft-rule <rule-name> \
    --session rule-arc-<slug>-off
# repeat 5 times
```

Read all 10 replies by eye. Ask: does the rule-OFF baseline manifest the
failure mode the rule was authored to prevent? If NO, the test is **vacuous**
— the prompt is wrong, or upstream invariants are already suppressing the
mode. Do not cite a vacuous test as a real null. Pick a different prompt OR
label the cell `vacuous-test (failure mode absent)`.

If the failure mode IS present rule-OFF and absent rule-ON, the rule bites.
Continue.

**Layer B — paired-rubric grading (when the verdict is at the margin).** Run
`worldcli grade-runs` against the run-ids from Layer A with TWO rubrics of
different architectures (e.g., a tag-forcing presence-of-X rubric AND a
gestalt does-this-feel-like-Y rubric). Agreement → trust verdict. Disagreement
IS the signal — investigate manually.

**Layer C — cross-character validation (for universal-shape rules).** If the
rule's failure mode is shared across characters (over-consecration,
nanny-register, false uplift, etc.), run Layer A on a second and third
character. Same-character bite is `Sketch` at best; cross-character bite at
N≥3 across N≥2 characters is what `Characterized` requires.

**Layer D — synthetic-history probe (when session-context might confound).**
If the model self-corrects mid-session on the failure mode (a real LLM
behavior surfaced 2026-04-27), the in-session N=5 may understate the rule's
bite by N=4 or 5. Use `worldcli ask --synthetic-history <path>` to inject a
fresh prior context per call, breaking the self-correction loop. If the rule
bites WITHOUT synthetic history but vanishes WITH it, that is itself a
finding — the rule's bite was being amplified by session-context, not by the
prompt body.

### Step 4 — Honest tier assignment

The registry's `EvidenceTier` enum is the project's vocabulary for what kind
of progress a rule represents. Pick the variant the evidence honestly earns:

- `Unverified` — no bite-test run yet. The default for new rules.
- `Sketch` — N=1 or N=2 same-character probe showed bite. Directionally
  suggestive; a single reversal would refute.
- `Claim` — N=3+ per cell same-character; bite is direction-stable. Citable
  in reports as load-bearing.
- `Characterized` — N=5+ per cell across N≥2 characters; bite is rate-stable.
  Required for production-default register-shape claims. **All three
  conditions must hold:** probe-replicable (the rule fires on the
  elicitation), carve-out-refinable (you can name the failure mode it
  protects), prior-observation-entering-bite-test (came from real
  observation, not pre-imagined).
- `TestedNull` — paired probe showed no bite for this character. Not a
  retirement signal alone; the rule may still bite on others.
- `VacuousTest` — the rule-OFF baseline did NOT manifest the failure mode.
  The test cannot speak to whether the rule bites; the prompt or upstream
  stack is doing other work.
- `Accumulated` — the rule names a pattern the registry observed across
  multiple bite-tests, not a single-rule probe. Use sparingly; reserved for
  cross-rule findings.
- `EnsembleVacuous` — paired probe vacuous AND cross-character probe vacuous.
  The rule is honestly authorial; the failure mode it protects against is
  already suppressed by the formula+invariants ensemble. **This is honest
  progress, not failure.** The 5:1 EnsembleVacuous:Characterized ratio
  observed 2026-04-27 is a feature: it tells us the upstream stack is doing
  its work, and the rule's place in the registry is documentary rather than
  behavioral.

Update the entry's `evidence_tier` and `last_tested` fields. Rebuild worldcli
to verify the change compiles.

### Step 5 — The documentation triangle

A rule that has walked the arc earns three updates, not just one:

**Triangle vertex 1 — registry entry.** `evidence_tier` and `last_tested`
updated (already done in Step 4). The provenance line stays put.

**Triangle vertex 2 — CLAUDE.md doctrine paragraph (when the arc taught
something).** If the bite-test arc surfaced a methodological finding —
something a future session needs to know about HOW the project tests rules,
not just about THIS rule — write a paragraph into the relevant CLAUDE.md
section. Examples shipped 2026-04-27: "EnsembleVacuous tier shape
codification" / "5:1 EnsembleVacuous:Characterized architectural ratio" /
"three-conditions-for-Characterized" / "multi-turn self-correction limit" /
"synthetic-history-also-triggers-self-correction". The bar: would a future
session reading this paragraph change its behavior on a rule-arc still in
flight? If yes, write it. If no, skip — doctrine inflation is its own drift.

**Triangle vertex 3 — `reports/OBSERVATIONS.md` entry.** A short Mode-1 or
Mode-2 entry naming what was observed, what the bite-test found, and what
shipped. The entry is the provenance trail the registry's `provenance` field
points back to. Newest-first per the take-note convention.

The triangle is structural — all three vertices, not two. A rule with only a
registry entry is undocumented; a rule with only doctrine is detached from
its source; a rule with only an OBSERVATIONS line never reaches the prompt
stack. The triangle keeps the rule's life legible from any of the three
vertices.

### Step 6 — Optional structural improvement

If the arc surfaced a gap the existing instruments couldn't articulate — a
new tier variant the enum is missing, a new worldcli affordance the bite-test
needed, a new field on `CraftRule`, a new query primitive — ship the
structural improvement in the same arc. The 2026-04-27 arc earned the
`EnsembleVacuous` enum variant and the `--synthetic-history` flag because
specific bite-tests needed them; both shipped inline rather than as deferred
follow-ups.

The bar for a structural improvement here is the same as the
sharpen-the-instruments doctrine: a specific, named gap the current
infrastructure made hard to express. Vague "could be nicer" doesn't qualify.

### Step 7 — Commit

Commit the rule + tier-update + doctrine + OBSERVATIONS as a coherent batch
(or 2-3 batches if the structural-improvement piece wants its own scope).
Per project commit-message discipline, include the Formula derivation:

```
**Formula derivation:** [one Unicode-math expression]
**Gloss:** [one short sentence in plain English, ≤25 words]
```

The arc is done when all three triangle vertices land on disk and the build
is green.

## Composing with neighboring instruments

- **Upstream feeders:** `/play` (wince-read), `/take-note` (Mode 1
  observation), lived-play paste-ins, `/eureka` discovery loop. The arc
  consumes their output; do not re-implement them inside the arc.
- **Sibling instruments:** `/derive-and-test` (per-character formula
  coherence — different shape, no rule-shipping), `/run-experiment`
  (hypothesis-shaped probes — the arc's bite-test step subsumes it for
  rule-shaped questions), `/batch-hypotheses` (5-10 candidate phrasings —
  use BEFORE this arc when the rule's wording is genuinely uncertain;
  produces the candidate the arc then ships and tests).
- **Downstream consumers:** `reports/` trajectory reports cite the arc's
  outputs; `CLAUDE.md` doctrine accumulates from arcs over time; the
  registry itself is the most-queryable consumer of the arc.

## Anti-patterns the arc is meant to prevent

- **Shipping a rule and citing it as "verified" without paired probe.** The
  arc forces the paired probe; without it the rule earns `Unverified`.
- **Using a single rubric to grade a marginal verdict.** Layer B exists to
  catch rubric-architecture confounds.
- **Calling vacuous tests real nulls.** The `VacuousTest` and
  `EnsembleVacuous` tier variants exist to keep this honest.
- **Doctrine paragraphs that re-state the rule's body in different words.**
  Vertex 2 is for METHODOLOGICAL findings, not for re-articulating the rule.
- **Pre-imagined rules masquerading as observation-derived.** Step 0's three
  doors are the gate; if none is open, the arc does not start.

## When the arc closes elsewhere

If a rule walks the arc and earns `EnsembleVacuous`, the temptation is to
delete it. Resist. The honest place for an `EnsembleVacuous` rule is in the
registry with its tier label and provenance — the body does NOT ship to the
model under default render (per `EvidenceTier::ships_to_model()`, since
2026-04-27 commit landing this affordance), AND the rule is NOT deleted
from history. The 5:1 ratio means most rules earn this tier; deleting them
collapses the registry's documentary value, and the substrate ⊥ apparatus
discipline keeps the body absent from the model's prompt without losing the
provenance trail.

**For ensemble re-tests:** pass `--include-documentary-rules` to `worldcli
ask` (or `PromptOverrides::set_include_documentary_craft_rules(true)`
programmatically) to render with EnsembleVacuous bodies INCLUDED — useful
when checking whether the prior vacuous-finding still holds under the
leaner ensemble.

## Origin

Skill authored 2026-04-27 in response to: *"I think this calls for a
learned-lesson-inspired Claude Code skill so that we can repeat this rate of
increase unto Mission Formula."* The arc the skill encodes was discovered by
walking it once across six rules in a single sustained ~14-hour session that
produced 47+ commits and shipped the registry pattern. The skill exists so a
future session can re-run the arc on the next observation without
re-discovering the shape.

The arc is not the only path to structural progress on 𝓕. It is one of
several (formula reauthoring, invariant ships, instrument-sharpening, lived-
play accumulation). It earns its place because rule-shipping is the most
frequent move on the prompt-stack and the most prone to drift when the tier
label and the evidence drift apart. The arc keeps them aligned.

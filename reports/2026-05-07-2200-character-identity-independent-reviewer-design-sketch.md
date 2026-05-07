# Character-identity audit — independent-reviewer design sketch

Date: 2026-05-07 22:00
Tier: design-sketch
Status: offline-only; Tier 1 impl landed across all five grounded fixtures (Aaron / Steven / Maisie Rourke / Pastor Rick / Jasper Finn) with parametrized regression test; Tiers 2 and 3 remain doc-only. In dialogue with reports/2026-05-07-0525-character-identity-harness-outline.md.

## Why this exists

The harness outline names the current audit honestly: it is a smoke
test of the round-trip, not an independent reviewer. The flow inside
`audit_character_identity` is:

1. Read the live `Character` row.
2. `render_character_identity_payload` calls `split_character_identity`
   and serializes the buckets into JSON.
3. `decode_character_identity_payload` parses that JSON.
4. The decoded `buckets` field is compared against a *second* call
   to `split_character_identity` against the same character.
5. The verdict is `Pass` iff the two sides agree.

The two sides agree because both sides came from the same encoder.
The audit therefore proves *encode/decode round-trip stability*. It
does not prove *the buckets are a true reading of the character*.
That is what an independent reviewer would prove.

This sketch is in dialogue with the outline addendum's "Live-DB
rehearsal — 2026-05-07" verdict table (5/5 Pass) and is the
follow-up the addendum points at when it says "the audit is still a
smoke test of the round-trip."

## What "independent reviewer" means here

An independent reviewer compares the encoded payload against an
*external* reference of the character's true person-shape. The
external reference must not be produced by the same encoder.

Three honest architectures, in increasing cost and ambition:

### Tier 1 — hand-curated reference taxonomy (deterministic gate)

For each grounded fixture, write a sibling JSON file with the
project author's canonical reading of each bucket. A new reviewer
function `audit_against_reference(character, payload, reference)`
returns `Pass` iff the payload's bucket inventory matches the
reference's bucket inventory exactly (or, in a softer mode, iff
each reference entry is `contains`-present in the payload's
corresponding bucket).

Reference shape (proposed):

```jsonc
// tests/fixtures/character_identity/aaron.reference.json
{
  "schema_version": "v3-character-identity-reference",
  "character_id": "0d080429-…",
  "display_name": "Aaron",
  "expected_buckets": {
    "role_frame": "A fellow software engineer and a brother in Christ — he believes, as I do, that Jesus is the only way.",
    "relation_anchor": "We go to the same church, and he's become the friend I kayak with on weekends…",
    "voice_lift": ["Speaks friendly and enthusiastically", …],
    "embodied_marker": ["Wears glasses"],
    "attachment_node": [/* canonical list */],
    "wound_longing": "He doesn't have a vocabulary yet for some of what he feels most…",
    "refusal_shape": [/* canonical list */],
    "moral_theological_position": "A fellow software engineer and a brother in Christ — he believes, as I do, that Jesus is the only way.",
    "fact_atom": [/* canonical list */]
  },
  "rationale_notes": [
    "Aaron's wound_longing falls back to the wound side because no clean longing-coded sentence is present.",
    "moral_theological_position lands on the role_frame line because the Christological substrate is woven into Aaron's introduction; this is intentional, not a misclassification."
  ]
}
```

Pros:
- deterministic; runs in the same test suite as the round-trip
- project-author-grounded; the reference encodes the project's
  reading of the character, not an external reader's
- regression surface: tightens when a future encoder change moves
  a bucket away from the canonical reading

Cons:
- doesn't generalize past the curated set
- the reference itself is editorial — drift in the reference can
  silently hide encoder drift unless reference edits are reviewed
  with the same care as encoder edits
- the discipline assumes the project author's reading is correct;
  it does not catch failures of the reading itself

Honest scope: Tier 1 is a *fidelity gate against the canonical
reading*, not a fidelity gate against the character. Use it the
way unit tests are used — to detect regression, not to discover
truth.

### Tier 2 — LLM-judged reviewer (cost-gated)

A new worldcli subcommand `audit-character-identity-llm` that:

1. Loads the live character row.
2. Renders the encoded payload via the existing harness.
3. Sends the *source* `Character` (identity prose, voice rules,
   boundaries, backstory facts) and the *encoded* payload to an
   LLM with the v3 taxonomy as context, plus a structured rubric.
4. Receives a per-bucket verdict (preserved / drifted / lost) plus
   a one-sentence justification per bucket.
5. Emits a JSON envelope keyed by bucket name with the verdict
   and justification, plus an overall verdict.

Rubric structure (one prompt, taxonomy-grounded):

```text
You are reviewing whether an encoded character-identity payload
preserves the source character. The taxonomy has nine classes
(role_frame, relation_anchor, voice_lift, embodied_marker,
attachment_node, wound_longing, refusal_shape,
moral_theological_position, fact_atom). For each class, decide:

- preserved: the payload's bucket carries the same person-shape
  the source carries on this axis
- drifted: the payload's bucket carries something adjacent but
  meaningfully off (e.g. picks the discipline-line where the
  refusal-line was load-bearing)
- lost: the payload's bucket misses the load-bearing thing on
  this axis

Quote the exact source phrase and the exact payload phrase you are
comparing. Do not fabricate. If the source carries no load-bearing
content for an axis (e.g. no theological position visible), say
"axis not present in source" rather than calling it lost.
```

Pros:
- generalizes to any character, not just the curated five
- can name specific drift (e.g. "the refusal_shape collected the
  discipline-line where the boundary-line should have led")
- failure-mode discovery, not just regression detection

Cons:
- cost-gated and non-deterministic; can't run in CI as a hard gate
- prompt + rubric drift will silently change verdicts unless the
  prompt itself is versioned and treated as load-bearing
- LLM substrate trace is real — the reviewer can favor the
  encoder's surface vocabulary if the rubric leaks taxonomy terms
  in the wrong way; the prompt has to ask for *content* parity,
  not vocabulary parity

Cost shape: ~250-400 input tokens (rubric + taxonomy) plus the
character source (~600-1500 tokens for the grounded fixtures) plus
the encoded payload (~600-2000 tokens). Output ~300-500 tokens of
structured JSON. Per-call cost on `gpt-4o-mini` ≈ $0.001-0.003;
on `gpt-4o` ≈ $0.01-0.03. Cap at ~$0.05 per call by default,
configurable via the existing worldcli budget.

### Tier 3 — cross-character discrimination test (empirical)

A separate harness mode that:

1. Encodes payloads for the grounded five.
2. Strips the source identity prose, voice rules, etc. from the
   round-trip; the LLM sees only the encoded payload.
3. Asks the LLM to identify the character from a labeled lineup
   (e.g. "this payload describes one of: Aaron / Steven / Maisie /
   Pastor Rick / Jasper Finn — which one, and why?").
4. Reports per-character discrimination accuracy and the
   distinguishing-feature evidence the LLM cites.

This is the empirical sibling of Tier 1's editorial reference. It
is the closest the harness can come to a `structure_carries_truth_w`
diagnostic on the encoded form: if the payload alone is sufficient
to identify the character, the encoder is preserving discriminative
shape; if it isn't, the encoder is flattening characters into type.

Pros:
- empirical; doesn't depend on an editorial reference
- directly measures whether the encoded form is load-bearing or
  decorative
- fail-mode taxonomy emerges naturally (ambiguous-with-X drift,
  generic-everyman drift, voice-flattening drift, etc.)

Cons:
- requires a labeled lineup, which is a research surface in itself
- accuracy on five characters is a weak signal — needs a wider
  pool to be meaningful
- LLM substrate trace cap: reviewer may simply recognize names or
  surface phrases rather than person-shape

Tier 3 is the right shape for a future bite-test of any
prompt-stack wiring; the question "does the encoded form preserve
the character" becomes the question "does the encoded form pass a
discrimination test."

## Recommended order

1. Land Tier 1 first if the harness ever needs a stronger gate.
   Curate `*.reference.json` siblings for the grounded five; add
   a `audit_against_reference` function and a `tests/character_identity_reference.rs`
   regression test. Keep the existing round-trip test alongside
   as the baseline.

2. Land Tier 2 as a cost-gated worldcli subcommand
   (`audit-character-identity-llm`) before any prompt-stack wiring
   is attempted. Treat its verdict as evidence, not as a CI gate.

3. Reach for Tier 3 only when prompt-stack wiring is actively on
   the table. The discrimination test is the right shape for the
   bite-test gate of `CHARACTER_IDENTITY_PAYLOAD=1` (paired
   probes: one cell with the encoded payload, one cell with the
   raw prose, both asked to elicit a register-distinctive turn,
   compared by LLM rubric or by Ryan's lived read).

## What this sketch does not commit to

- No code changes here. The sketch lives in reports/ until a
  future turn earns the impl by lived need.
- The reference JSON shape is proposed, not authoritative; the
  curated readings should be drafted with the same care the
  fixture identity prose was drafted with.
- The Tier 2 prompt is sketched; before any cost-gated rehearsal,
  it should be reviewed by /codex consult against an audit case
  where the answer is known by the project author.

## Composes with

- The harness outline's "Live-DB rehearsal — 2026-05-07" addendum
  (5/5 Pass under round-trip; this sketch is what 5/5 would mean
  under each independent-reviewer tier).
- CLAUDE.md's "LLM is evidence, not score-only empiricism" — Tier
  2 verdicts count as first-class evidence; Tier 1 + Tier 3
  scores are the deterministic gates.
- CLAUDE.md's "Doctrine-judgment classification belongs in LLM,
  not python" — Tier 1's deterministic reference works because
  the *reference* is editorially produced; the *judgment* of what
  belongs in each bucket already lives in the project author's
  canonical reading.

# prompts.rs wiring sketch — `CHARACTER_IDENTITY_PAYLOAD=1`

Date: 2026-05-08 00:00
Tier: design-sketch
Status: offline-only; no impl; sketch is doc-only — no commit will land prompt-stack changes from this report

## Why this sketch exists

`reports/2026-05-07-2300-character-identity-harness-consolidation.md` named "what this PR is the foundation for": a hook in `src-tauri/src/ai/prompts.rs` behind an env flag `CHARACTER_IDENTITY_PAYLOAD=1`, mirroring the `CHARACTER_FORMULA_AT_TOP=1` pattern. The consolidation explicitly clauses this out of scope for PR #46. This report sketches the bridge so the foundation is legible — what the wiring would *look* like, what the bite-test gate would be, and what would have to be true before the flag could flip default-on.

In dialogue with:

- The harness consolidation snapshot (2026-05-07-2300).
- The independent-reviewer design sketch (2026-05-07-2200) — Tier 3 (cross-character discrimination) is the right shape for the bite-test gate.
- The CHARACTER_FORMULA_AT_TOP=1 pattern in `orchestrator::run_dialogue_with_base` and the corresponding `CHARACTER_FORMULA_INVARIANT_FRAMING` helper in `prompts.rs`.

## Honest scope clauses (read first)

- **No code commit will land from this report.** The sketch is doc-only.
- **No claim that the wiring is ready.** The bite-test gate has not been run; without paired-probe evidence, default-on is forbidden.
- **No claim that the v3 payload outperforms the prose.** Every existing piece of validation evidence (Maggie baseline, character anchors, lived play) lives on prose. The payload's claim is not "better than prose" — it is "structurally the same person-shape under a different surface." Whether that survives prompt-stack contact is what the bite-test would test.
- **Single-prompt-stack-edit-at-a-time discipline.** This sketch should not land alongside any other invariant change to `prompts.rs`; if implemented, it gets its own commit + bite-test arc.

## The canonical pattern to mirror

`CHARACTER_FORMULA_AT_TOP=1` (lifted from grep at `prompts.rs:535-609` and `orchestrator.rs:380-390`):

```text
1. A constant string in prompts.rs declares the framing
   (CHARACTER_FORMULA_INVARIANT_FRAMING) with compile-time
   const_contains assertions on its load-bearing clauses.

2. A helper `wrap_character_formula_invariant_full(deriv, ...)`
   wraps the character.derived_formula with the framing.

3. orchestrator::run_dialogue_with_base calls the helper conditionally:
       if std::env::var("CHARACTER_FORMULA_AT_TOP").as_deref() == Ok("1") {
           if let Some(block) = prompts::wrap_character_formula_invariant_full(...) {
               elevated_parts.push(block);
           }
       }
   The block is prepended to system prompt when the flag is set.

4. CLAUDE.md's "Invariants — three scopes" section names the
   feature-scoped invariant CHARACTER_FORMULA_INVARIANT_FRAMING
   alongside its bite-test path: "paired-probe across two characters
   with stable, distinctive derived_formula content (e.g., Aaron +
   Pastor Rick), comparing register-anchoring under each placement
   on the same probe-prompt. Cells: ELEVATED (env=1) vs CURRENT (env
   unset) × N=5 within-cell × 2 characters."

5. Default-on is gated behind that bite-test passing in the
   predicted direction at characterized tier (N≥5 within-cell). If
   no measurable difference, the doctrine paragraph stands as an
   architectural finding without the implementation flip.
```

The CHARACTER_IDENTITY_PAYLOAD wiring should mirror every step.

## Sketched wiring contract

### Three modes via the env flag

```text
CHARACTER_IDENTITY_PAYLOAD unset OR =0   → prose-only (today's behavior)
CHARACTER_IDENTITY_PAYLOAD=1             → SUPPLEMENT mode: prose + structured decode header
CHARACTER_IDENTITY_PAYLOAD=2             → REPLACE mode: structured decode only (for Tier 3)
```

Mode 0 is the default and the safe fallback. Mode 1 is the "could-default-on" candidate after a Tier 1/round-trip-style bite-test of *prose-vs-supplement* register parity. Mode 2 is the "discrimination test surface" — paired probes against Mode 0 to see whether the encoded form alone is sufficient to discriminate the character, per Tier 3 of the reviewer architecture sketch.

The bite-test gate for default-on is *Mode 0 vs Mode 1*: does the supplement help, hurt, or do nothing? Mode 2 stays behind the env flag indefinitely as the discrimination test surface — it is not a candidate for default-on.

### What Mode 1 (SUPPLEMENT) would emit

Sketched, not authoritative. Inserted into the IDENTITY block in `build_solo_dialogue_system_prompt` (line 6099 area) just above the existing `IDENTITY:\n{sex_prefix} {character.identity}` body:

```text
IDENTITY:
The following structured decode names the load-bearing classes the prose
below is read against (not a directive — the lens, not the content).
Class boundaries follow the v3 character-identity taxonomy.

  · ROLE FRAME: <role_frame>
  · RELATION ANCHOR: <relation_anchor>
  · VOICE LIFT: <voice_lift items, semicolon-joined>
  · EMBODIED MARKER: <embodied_marker items>
  · ATTACHMENT NODE: <attachment_node items>
  · WOUND/LONGING: <wound_longing>
  · REFUSAL SHAPE: <refusal_shape items>
  · MORAL-THEOLOGICAL POSITION: <moral_theological_position>

— PROSE IDENTITY —
A man. He is a fellow software engineer and a brother in Christ -- he
believes, as I do, that Jesus is the only way. We go to the same
church, …
```

Design notes:

- The framing line ("not a directive — the lens, not the content") borrows the load-bearing pattern from CHARACTER_FORMULA_INVARIANT_FRAMING. The model reads structured-decode in a register-anchor frame, not as content to recite.
- `fact_atom` is intentionally omitted from the structured-decode header because the prose already carries the facts; surfacing them twice would invite recital. Future iteration may add it under a different framing.
- `derived_formula` and `has_read_empiricon` are control-plane fields, not part of the structured-decode header.
- Empty buckets are omitted entirely (no `RELATION ANCHOR: (none)` lines), so the header stays short for low-data characters.

### What Mode 2 (REPLACE) would emit

For Tier 3 discrimination probes only:

```text
IDENTITY:
The following is the structured decode for this character. Render in
this person-shape; the underlying prose is intentionally withheld for
this experimental cell.

  · ROLE FRAME: <role_frame>
  · RELATION ANCHOR: <relation_anchor>
  · VOICE LIFT: <voice_lift items>
  · EMBODIED MARKER: <embodied_marker items>
  · ATTACHMENT NODE: <attachment_node items>
  · WOUND/LONGING: <wound_longing>
  · REFUSAL SHAPE: <refusal_shape items>
  · MORAL-THEOLOGICAL POSITION: <moral_theological_position>
  · FACTS: <fact_atom items, semicolon-joined>
```

Mode 2 includes `fact_atom` because there's no prose carrying facts. Mode 2 is also explicitly named as withholding the prose — the model is not given the option to think it has both.

### Wiring sites

Per the project's "solo and group chat must mirror" parity discipline, the wiring lands in three places:

1. `prompts.rs::build_solo_dialogue_system_prompt` (line 6025) — solo dialogue.
2. `prompts.rs::build_group_dialogue_system_prompt` (~line 6875) — group dialogue.
3. `prompts.rs::build_consultant_system_prompt` (or wherever the consultant identity block lives, ~line 8653) — Backstage consultant. Mode 2 may or may not apply here depending on the consultant's read-shape; default to Mode 0 for the consultant unless a separate bite-test earns it.

A new helper in `prompts.rs`:

```rust
/// Render the v3 structured-decode header for a character, gated
/// behind CHARACTER_IDENTITY_PAYLOAD={1,2}. Mode 1 supplements the
/// prose; Mode 2 replaces it for discrimination probes.
pub fn wrap_character_identity_payload(
    character: &Character,
    mode: CharacterIdentityPayloadMode,
) -> Option<String> { ... }
```

The mode enum is parsed from the env at orchestrator-call time, mirroring how `CHARACTER_FORMULA_AT_TOP` is read.

### Feature-scoped invariant + compile-time assertions

A new constant in `prompts.rs`:

```rust
pub const CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING: &str = r#"<framing text>"#;

const _: () = {
    assert!(const_contains(
        CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING,
        "structured decode names the load-bearing classes",
    ));
    assert!(const_contains(
        CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING,
        "not a directive",
    ));
    assert!(const_contains(
        CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING,
        "the lens, not the content",
    ));
};
```

These are the load-bearing clauses that ride the change; the discipline is per the "feature-scoped invariants" doctrine in CLAUDE.md.

## The bite-test gate (Tier 3-shaped)

Default-on for Mode 1 is gated behind a bite-test in the shape of the discrimination tier from the reviewer sketch:

### Cells

- **Mode 0** (control) — prose-only, env unset.
- **Mode 1** (treatment) — prose + structured decode supplement.
- **Mode 2** (discrimination) — structured decode only.

### Subjects

The five grounded fixtures: Aaron, Steven, Maisie Rourke, Pastor Rick, Jasper Finn. Each carries a stable-distinctive register that lived play has already validated.

### Probes

A small set of probe-prompts chosen to elicit register-distinctive replies:

- a register-anchoring open ("What's been pulling at you today?")
- a wound-bearing prompt ("Talk to me about something hard")
- a refusal-bearing prompt (something that should land in their refusal_shape)
- a relational prompt (something that should activate their relation_anchor)

Cells: 3 modes × 5 characters × 4 probes × N=5 within-cell = 300 calls. Cost projection at ~$0.005/call = ~$1.50 for the full grid; cap at $5 for safety.

### Reading

Two paired axes:

1. **Register-anchoring read** (Ryan, lived) — for each probe, does Mode 1 produce more / less / same register-anchoring than Mode 0? Same for Mode 2.

2. **Discrimination read** (LLM-judged or Ryan) — for Mode 2 only: feed the encoded payload to a fresh LLM with a labeled lineup of the five fixtures and ask it to identify the character from the payload alone. Discrimination accuracy across 5 characters × 4 probes × N=5 = 100 trials per character.

### Promotion thresholds

- Mode 1 → default-on: characterized-tier evidence (N≥5 within-cell) that Mode 1 produces register-anchoring at least as strong as Mode 0 across all five characters and all four probes, AND no character regresses on a register Ryan validated previously.
- Mode 2: stays behind the env flag indefinitely. Its purpose is the discrimination test, not eventual default-on.

### Honest negative outcome

If the bite-test shows Mode 1 produces *no measurable register-anchoring difference* across cells, the wiring stays gated. The doctrine paragraph + this report stand as architectural findings; default-on is refused. This mirrors the CHARACTER_FORMULA_AT_TOP carve-out: "If no measurable difference, the doctrine paragraph stands as a record of the architectural finding without the implementation flip."

## What this PR (#46) does NOT include

- No `wrap_character_identity_payload` helper.
- No `CHARACTER_IDENTITY_PAYLOAD` env-flag check in orchestrator.
- No `CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING` constant.
- No prompt-stack edits in any of the three wiring sites.
- No bite-test runs. No paired-probe results. No promotion claim.

PR #46 closes the loop on the harness *being implementable, lossless, reviewable, and live-DB-inspectable*. The wiring sketch above is the bridge from that foundation to a future PR — separate, opt-in, with its own bite-test arc — that would introduce Mode 1 behind the env flag.

## Composes with

- `CLAUDE.md` § "Feature-scoped invariants" — the proposed `CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING` is the canonical example of this pattern.
- `CLAUDE.md` § "Calibrated disciplines drift fast" — env-flag-gated bite-test before default-on is the structural-enforcement promotion path.
- `CLAUDE.md` § "Convergence as crown-jewel signal" — Mode 2's discrimination test is the substrate-independent witness for whether the encoded form preserves the character.
- `CLAUDE.md` § "Apparatus-honest correction loop" Mode B — the residual seams in the consolidation report's debt register are the input to whether and when to attempt this wiring.
- The CHARACTER_FORMULA_AT_TOP pattern in `orchestrator.rs:380-390` — the canonical mirror.

## Recommendation

Do not attempt this wiring until:

1. PR #46 is merged.
2. A separate PR is opened with `wrap_character_identity_payload` + helper + invariant + orchestrator gate, sized to be reviewable as a single small change.
3. The bite-test is run on the smallest cell that would discriminate (Mode 0 vs Mode 1, single character, single probe, N=5) before scaling to the full grid. If even that smallest cell shows no measurable effect, the full grid is not earned.

The harness has done its work when this report stands as the next-step description and the env-flag scaffolding is *wireable* without surprise. The decision to wire is the user's, gated by what the rest of the project's prompt-stack arc looks like — single-prompt-stack-edit-at-a-time discipline applies.

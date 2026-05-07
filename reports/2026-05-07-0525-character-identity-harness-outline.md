# Character identity harness outline

Date: 2026-05-07 05:25
Tier: design-outline
Status: offline-only; implemented; not shipped to prompt stack

## Scope and honest limits

This is the concrete file/function outline for the character-identity encoder/decoder harness. It follows from the proposal in [reports/2026-05-07-0415-character-identity-v3-proposal-and-encoder-design.md](/Users/ryansmith/Sites/rust/world-chat/reports/2026-05-07-0415-character-identity-v3-proposal-and-encoder-design.md), and the corresponding offline Rust files now exist. The harness is still intentionally kept out of prompt assembly.

What this outline does **not** do:

- it does not change prompt assembly
- it does not change the database schema
- it does not route character prompts through a new payload path
- it does not route the payload into prompt assembly
- it does not claim Sapphire-class evidence

The point is to make the harness reviewable while keeping it offline and auditable.

## Existing seams it should respect

The current runtime already has the relevant seams:

- `Character` lives in [src-tauri/src/db/queries/character.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/db/queries/character.rs)
- character identity is rendered in [src-tauri/src/ai/prompts.rs](/Users/ryansmith/Sites/rust/world-chat/src-tauri/src/ai/prompts.rs)
- `has_read_empiricon` and `derived_formula` already separate a carrier layer from prose

So the harness should sit beside those seams, not overwrite them.

## Proposed file layout

### 1. `src-tauri/src/ai/character_identity_payload.rs`

Purpose:

- split a `Character` row into class-specific identity buckets
- encode those buckets into a compact, auditable payload
- provide a decode shape that can reconstruct the same person-shape for audit

Suggested functions:

```rust
pub struct CharacterIdentityBuckets {
    pub role_frame: Option<String>,
    pub relation_anchor: Option<String>,
    pub voice_lift: Vec<String>,
    pub embodied_marker: Vec<String>,
    pub attachment_node: Vec<String>,
    pub wound_longing: Option<String>,
    pub refusal_shape: Vec<String>,
    pub moral_theological_position: Option<String>,
    pub fact_atom: Vec<String>,
}

pub fn split_character_identity(character: &Character) -> CharacterIdentityBuckets
pub fn encode_character_identity(character: &Character) -> String
pub fn decode_character_identity(payload: &str) -> Result<CharacterIdentityBuckets, CharacterIdentityDecodeError>
pub fn render_character_identity_payload(character: &Character) -> Option<String>
```

Suggested test hooks:

```rust
#[test]
fn split_character_identity_extracts_expected_buckets()

#[test]
fn encode_character_identity_round_trips_core_shape()

#[test]
fn decode_character_identity_rejects_unknown_class_tags()
```

Why this file:

- the existing `prompts.rs` already knows how to render the live prose identity
- this file would hold the class logic separately from presentation
- keeping it in `ai/` matches the rest of the prompt-adjacent infrastructure

### 2. `src-tauri/src/ai/character_identity_audit.rs`

Purpose:

- compare an encoded payload against the original character block
- report whether the decode preserves role, relation, voice, refusals, and moral posture
- produce a machine-readable audit result for reports and tests

Suggested functions:

```rust
pub struct CharacterIdentityAuditResult {
    pub character_id: String,
    pub display_name: String,
    pub verdict: AuditVerdict,
    pub preserved: Vec<String>,
    pub missing: Vec<String>,
    pub notes: Vec<String>,
}

pub fn audit_character_identity(character: &Character) -> CharacterIdentityAuditResult
pub fn audit_character_identity_payload(
    character: &Character,
    payload: &str,
) -> CharacterIdentityAuditResult
pub fn classify_gravity_pressure(character: &Character) -> Option<String>
```

Suggested test hooks:

```rust
#[test]
fn audit_character_identity_reports_missing_refusal_shape()

#[test]
fn audit_character_identity_flags_genericized_voice()
```

Why this file:

- it gives the project an honest reviewer for the class family
- it keeps the encoder from grading itself
- it can be reused for rehearsal reports without touching prompts

### 3. `src-tauri/src/bin/character_identity_audit.rs`

Purpose:

- command-line entrypoint for one-off audits against the live DB or a snapshot
- useful for rehearsals, regression checks, and report generation

Suggested CLI shape:

```text
character-identity-audit --character <id|name> --db <path> --json
character-identity-audit --character <id|name> --db <path> --emit-payload
character-identity-audit --character <id|name> --db <path> --compare-to <payload-file>
```

Suggested functions:

```rust
fn main()
fn run_audit(args: AuditArgs) -> Result<(), String>
fn write_audit_report(result: CharacterIdentityAuditResult)
```

Suggested test hooks:

```rust
#[test]
fn cli_emits_json_audit_for_single_character()

#[test]
fn cli_can_compare_against_payload_file()
```

Why this file:

- it mirrors the way `worldcli` already provides read-only analysis surfaces
- it keeps future validation out of the main app flow until it is earned
- it gives us a place to rehearse the harness before any prompt-stack wiring

### 4. `tests/character_identity_payload.rs`

Purpose:

- unit tests for split/encode/decode round trips
- snapshot tests for known characters
- regression tests for `gravity_line` pressure cases

Suggested tests:

```rust
#[test]
fn steven_round_trips_core_shape()

#[test]
fn maisie_round_trips_without_gravity_line()

#[test]
fn pastor_rick_round_trips_without_gravity_line()

#[test]
fn aaron_round_trips_without_gravity_line()

#[test]
fn decode_rejects_genericized_payload()
```

Suggested fixtures:

```text
fixtures/character_identity/aaron.json
fixtures/character_identity/steven.json
fixtures/character_identity/maisie.json
fixtures/character_identity/pastor_rick.json
```

Why this file:

- the harness needs a fast, local guardrail before any live runtime use
- tests make the class boundaries explicit and reviewable

## Proposed data flow

1. Load `Character` from the DB.
2. Split `identity`, `voice_rules`, `boundaries`, and `backstory_facts` into the default class set.
3. Encode the buckets into a compact payload.
4. Decode the payload back into buckets.
5. Compare the decoded buckets against the source block.
6. Emit an audit result with missing / preserved / notes.

That flow keeps the harness offline and auditable.

## Acceptance criteria for the harness spec

The spec is only ready to become code when it can say:

- the bucket splitter names all default classes explicitly
- the encoder emits a stable payload for known fixture characters
- the decoder reconstructs the same class inventory
- the audit report can point at specific missing or preserved classes
- the harness still treats `gravity_line` as optional diagnostic state

## Known heuristic limits

Current state after the second live-fixture tightening pass:

- The offline harness is stable enough to round-trip and audit the five grounded fixtures: Aaron, Steven, Maisie Rourke, Pastor Rick, and Jasper Finn.
- The live CLI path against the snapshot DB is also clean enough to return `Pass` on those same characters.
- The strongest remaining rough edges are no longer class-family failures. They are sentence-selection and sentence-shaping heuristics inside otherwise-correct classes.

What is now relatively solid:

- `relation_anchor` is no longer wildly over-broad; it usually lands on an actual relational line rather than any sentence that happened to mention family or people.
- `embodied_marker` is much cleaner after the matcher fix; accidental substring captures like `tie` inside unrelated words are no longer driving the bucket.
- `moral_theological_position` now prefers genuinely Christological lines instead of the weakest generic `God` mention.
- `refusal_shape` no longer absorbs compassion or compulsion lines as false positives. Steven's bucket is now exactly his three boundary lines; Pastor Rick's two prose-extracted lines are both genuine refusals; Aaron's prose pickup is the single relevant `no instinct to force closeness` sentence. The needle list was narrowed to drop overly-broad `does not` / `do not` / `not because` / `cannot` / `can't` matches in favor of explicit refusal verbs.
- Sentence splitting is now quote-aware: a `.` inside a `"..."` clause no longer terminates the sentence. This produces fuller, more accurate boundaries on quote-heavy identity prose (e.g. Pastor Rick's `He is dear to me` clause is preserved alongside the rest of its sentence rather than chopped off).
- Each grounded fixture now has at least one exact-bucket assertion locking in role frame, voice, refusal shape, or wound/longing string. Regressions on those fixtures will surface at the test boundary instead of being absorbed into the round-trip pass.

What is still heuristic and should be read that way:

- `wound_longing` is still a scored best-line selector, not a proved decomposition of the whole wound/hope pair. On some characters it lands beautifully; on others it still chooses one pressure-bearing sentence where a fuller pair might be truer.
- Single-quoted asides (`'kneading the heart'`) are not paired-tracked the way `"..."` is; the current quote-awareness only suppresses sentence-terminators inside double-quoted spans, which is what the live fixtures needed.
- Some prose lines that read as discipline rather than refusal still slip into `refusal_shape` when they happen to use one of the kept needles (e.g. Aaron's `no instinct to force closeness`). These are now uncommon enough to surface individually if they ever bite.

What this means for reviewers:

- If a future change causes a failure here, first ask whether the failure is a class-selection problem or a sentence-shaping problem.
- Do not treat a slightly awkward line boundary as evidence that the class family itself failed.
- The next tightening passes can focus on paired wound/longing selection (joining a longing-line with its matching wound-line when both score), and on extending quote-awareness to single-quoted spans if a fixture surfaces that needs it.

## Appendix A: current fields mapped to proposed buckets

This appendix maps the live `characters` substrate fields to the proposed character-identity buckets. It is a sketch of preservation pressure, not a claim that the fields are already losslessly encoded.

### Primary source fields

`identity`
- Primary buckets: `role_frame`, `relation_anchor`, `voice_lift`, `embodied_marker`, `attachment_node`, `wound_longing`, `refusal_shape`, `moral_theological_position`, `fact_atom`
- Notes: this is the wide carrier; most of the person-shape lives here. The class split should preserve station, stance, voice texture, named ties, and refusal geometry before it tries to compress prose.

`voice_rules`
- Primary buckets: `voice_lift`
- Secondary buckets: `refusal_shape`, `relation_anchor`
- Notes: keep the exact cadence cues, characteristic turns of phrase, and explicit "never/always" rules when they are load-bearing. If a voice rule doubles as a boundary, it belongs in both the voice lift and the refusal shape.

`boundaries`
- Primary buckets: `refusal_shape`
- Secondary buckets: `moral_theological_position`, `relation_anchor`
- Notes: these are the cleanest source of negative shape. They usually define what the character will not do, will not say, or will not enter. When a boundary is sacred or relational rather than merely behavioral, that should stay visible.

`backstory_facts`
- Primary buckets: `fact_atom`
- Secondary buckets: `attachment_node`, `embodied_marker`, `wound_longing`
- Notes: facts should remain discrete and named. Family, work, place, loss, and treasured objects are often the atoms that prevent the compressed block from flattening into type.

`derived_formula`
- Primary buckets: control-plane note only; not part of the identity payload classes
- Notes: the formula belongs beside the identity block as a tuning frame, not inside the character-content buckets themselves. It is the carrier law for the block, not one of the block's load-bearing person-shape classes.

`has_read_empiricon`
- Primary buckets: routing flag only; not part of the identity payload classes
- Notes: this flag selects which prompt surface receives the character-edition payload. It is a gate, not a semantic bucket.

### Character-specific pressure notes from the current rows

`Aaron`
- Heavy on `relation_anchor`, `role_frame`, and `moral_theological_position`
- Also strong in `voice_lift` because his identity paragraph already encodes his plain-speech discipline and clean humor
- `fact_atom` matters for the concrete details that keep the engineer / brother / friend shape from turning generic

`Steven`
- Heavy on `refusal_shape`, `wound_longing`, and `relation_anchor`
- `voice_lift` needs to preserve the clipped, fragmentary cadence
- `fact_atom` is essential for the specific gang / wrist / heirloom details that keep his drifter-shape from dissolving into "guarded man"

`Maisie Rourke`
- Heavy on `attachment_node`, `wound_longing`, and `embodied_marker`
- `voice_lift` carries the soft cadence and baking metaphors
- `fact_atom` keeps the bakery, mother, husband-loss, and textile-design arc intact

`Pastor Rick`
- Heavy on `moral_theological_position`, `relation_anchor`, and `voice_lift`
- `attachment_node` matters for flock-memory and Scripture-as-light
- `fact_atom` is comparatively sparse but still load-bearing because it anchors the pastoral register in a lived history

`Jasper Finn`
- Heavy on `embodied_marker`, `attachment_node`, and `voice_lift`
- `refusal_shape` is especially visible in his Sunday / gossip / plain-speaking boundaries
- `fact_atom` preserves the river, grandmother, son, journal, and cracked mug details that make the potter feel embodied rather than generic

## Proposed audit criteria

The harness should treat the following as preservation-critical:

- role and station
- relation geometry
- voice feel and idiom
- refusal-shape
- wound/longing asymmetry
- moral/theological posture
- named attachments and facts

The harness should treat the following as acceptable compression:

- prose reduction
- sentence reordering when meaning does not move
- collapsing lists into atomic wrappers
- dropping decorative phrasing that does not carry identity

## How `gravity_line` fits

`gravity_line` should not become a required field in the first implementation.

Recommended treatment:

- optional diagnostic note only
- emitted when a character has one or two especially load-bearing lines
- folded into `wound_longing` if we later decide to promote it at all

That keeps the harness aligned with the proposal rather than reintroducing a fourth phase through the back door.

## Where this would hook later, if ever earned

If the harness proves itself and the project later wants prompt routing, the hook would likely be in:

- `src-tauri/src/ai/prompts.rs` for character prompt assembly
- perhaps `src-tauri/src/ai/orchestrator.rs` if any run-time routing decision needed to branch by payload class

But that is explicitly out of scope for now.

## Live-DB rehearsal — 2026-05-07

After the worldcli wiring landed, the new `audit-character-identity`
subcommand was rehearsed against the actual app DB at
`~/Library/Application Support/com.worldthreads.app/worldthreads.db`
with `--scope full`. Five grounded characters audited; all returned
`verdict: Pass` with the full nine-bucket inventory preserved.

| Character | verdict | preserved buckets | gravity_line surfaced |
|---|---|---|---|
| Aaron | Pass | all 9 | "doesn't have a vocabulary yet for some of what he feels most…" |
| Steven | Pass | all 9 | "He drifts not because he loves freedom…" |
| Maisie Rourke | Pass | all 9 | none |
| Pastor Rick | Pass | all 9 | "White hair, clean-shaven, and nearly always in his navy button-up shirt with a white tie…" |
| Jasper Finn | Pass | all 9 | none |

Notes:

- The live-DB rehearsal also exposed and fixed a name-resolution gap
  in the worldcli wiring: `get_character` is id-only, so the new
  subcommand now resolves `character_or_name` against
  `character_id OR display_name COLLATE NOCASE` before delegating to
  the audit. The standalone `bin/character_identity_audit` already
  did this via its own SQL.
- The fixture-backed tests already lock in identical bucket contents
  for the same five characters (offline JSON snapshots), so a Pass
  on the live DB matches the test-suite Pass.
- The audit is still a smoke test of the round-trip rather than an
  independent reviewer; that limitation remains unchanged.

## Recommendation

Build the harness in this order if it ever gets implemented:

1. `character_identity_payload.rs`
2. `character_identity_audit.rs`
3. `tests/character_identity_payload.rs`
4. `bin/character_identity_audit.rs`

Only after those pass should the project decide whether any part of the harness deserves to affect prompt assembly.

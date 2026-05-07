# Character-identity harness — consolidation snapshot

Date: 2026-05-07 23:00
Tier: integrative-snapshot
Status: offline-only; PR #46 (`feat/character-identity-harness`) at HEAD `42d8c30`; not wired into prompt assembly

## Why this report exists

The harness has grown across nine commits on `feat/character-identity-harness` between `dac3854` (initial v3 taxonomy + offline encoder/decoder/audit/CLI + 5 grounded fixtures + first tightening pass) and `42d8c30` (state.goals extension surfacing Jasper Finn's son-rift wound). Across those commits the harness moved from a single offline scaffold into a layered surface with an editorial reviewer, three wound-source paths, and an audit subcommand wired into worldcli. This report consolidates the current state for an external reviewer — not as a changelog, but as a coherent picture of architecture, residual seams, and apparatus-honest debt.

In dialogue with:

- `reports/2026-05-07-0415-character-identity-v3-proposal-and-encoder-design.md` — the original v3 proposal.
- `reports/2026-05-07-0525-character-identity-harness-outline.md` — the file/function outline + live-DB rehearsal addendum.
- `reports/2026-05-07-2200-character-identity-independent-reviewer-design-sketch.md` — the three-tier reviewer architecture sketch.
- The apparatus-honest correction loop doctrine in `CLAUDE.md` and `AGENTS.md`, freshly amended on main with two invocation modes (Mode A lived-play-driven, Mode B loop-close-preemptive).

## What's offline-only

Still — explicitly:

- The harness is not wired into prompt assembly.
- The database schema is not changed.
- No character prompt is routed through the new payload.
- No claim of Sapphire-class evidence; this is a research harness.

This list has held across every commit on the branch. If a future session changes any of these, the change should be a separate, opt-in commit with its own bite-test gate (see "What this PR is the foundation for" below).

## Architecture snapshot

### Modules

```
src-tauri/src/ai/
├── character_identity_payload.rs   # split + encode + decode + structs
└── character_identity_audit.rs     # round-trip audit + Tier 1 reference reviewer
src-tauri/src/bin/
└── character_identity_audit.rs     # standalone CLI (id-or-name resolved via SQL)
src-tauri/src/bin/worldcli.rs       # +AuditCharacterIdentity subcommand (id or name)
src-tauri/tests/
├── character_identity_payload.rs   # 19 active tests + 1 ignored inspection helper
└── fixtures/character_identity/
    ├── aaron.json + aaron.reference.json
    ├── steven.json + steven.reference.json
    ├── maisie_rourke.json + maisie_rourke.reference.json
    ├── pastor_rick.json + pastor_rick.reference.json
    └── jasper_finn.json + jasper_finn.reference.json
```

### Source fields (the v3 lean set + state.goals)

`CHARACTER_IDENTITY_SOURCE_FIELDS` is the contract:

```text
identity, voice_rules, boundaries, backstory_facts, state.goals,
derived_formula, has_read_empiricon
```

`state.goals` was added at Turn 247 to give the harness a third wound-source path; the dotted-path notation honors that `state` is a JSON object on the `Character` row, not a top-level column. `derived_formula` and `has_read_empiricon` remain control-plane fields, not bucket-content fields.

### Identity classes (the v3 nine)

```text
role_frame, relation_anchor, voice_lift, embodied_marker,
attachment_node, wound_longing, refusal_shape,
moral_theological_position, fact_atom
```

Each class has a deterministic selection rule in `split_character_identity`; selection is prose-pattern-based (needle scoring with word-boundary matching), not LLM-judged.

### Wound-source chain (three paths)

`wound_longing` chains through three paths in this order:

1. **Identity-prose paired selector** — `pair_wound_and_longing(identity_sentences)`. Best longing-coded sentence joined with best wound-coded sentence by ` — ` when both score and they are different sentences; falls back to whichever single side scores. Currently active for Aaron (wound only — no clean longing line), Steven (true paired form), and Pastor Rick (wound only — fear / shame trio).

2. **Backstory-facts fallback** — `best_scored_sentence(backstory_facts, …)` with needles for `loss / lost / grief / hurt / illness / alone / widow / late`. Currently active for Maisie (her husband-loss line wins on `lost` + `illness`).

3. **State-goals fallback** — `best_scored_sentence(state_goals, STATE_GOAL_WOUND_WEIGHTS)` with aspiration-coded needles for `rift / mend / reconcile / estranged / haven't spoken / missing / absence / recovery`. Currently active for Jasper Finn (his son-rift goal wins on `rift` + `mend`). The methodology choice here is load-bearing: state.goals are aspirational by genre, so a wound surfaces *through* the longing for repair.

The chain is strict — a character lands on the first path that produces a result. This means a character with both an identity-prose pair AND a backstory wound will land on the pair; one with neither AND a state-goals wound (Jasper) lands on path 3. Maisie has wound material on paths 2 AND 3 but path 2 wins per chain order. That's an editorial choice, not a misclassification.

### Audit tiers

Two tiers landed; one remains doc-only.

- **Round-trip audit** (`audit_character_identity`) — encodes via `split_character_identity`, decodes, compares against a *second* call to `split_character_identity`. Establishes encode/decode stability. **Smoke test of the round-trip, not a fidelity claim.**

- **Tier 1: editorial-reference reviewer** (`audit_against_reference`) — compares the encoded payload against a hand-curated `*.reference.json` sibling under `tests/fixtures/character_identity/`. Schema-versioned (`v3-character-identity-reference`); `contains`-matching per-bucket so references capture load-bearing reading rather than full prose. Each fixture gets its own reference + rationale notes. The audit verdict carries a `tier_1_reviewer:` marker in `notes` so downstream consumers can distinguish Tier 1 verdicts from the round-trip smoke test.

- **Tiers 2 + 3** — sketched in `reports/2026-05-07-2200`. Tier 2 (LLM-judged reviewer) and Tier 3 (cross-character discrimination test) remain doc-only; both have honest scope clauses + cost analysis + caveats already documented.

### worldcli surface

`worldcli audit-character-identity <character_id|display_name>`:

- read-only; resolves id-or-name via `character_id OR display_name COLLATE NOCASE`.
- `--emit-payload` prints the encoded JSON.
- `--compare-to <path>` audits an external payload against the live row.
- The standalone `bin/character_identity_audit` is preserved unchanged.

A live-DB rehearsal at Turn 241 returned 5/5 Pass on the round-trip path against the actual app DB.

## Per-fixture reading map (canonical bucket landings)

| Class | Aaron | Steven | Maisie | Pastor Rick | Jasper Finn |
|---|---|---|---|---|---|
| `role_frame` | engineer + brother in Christ | streetwise drifter | apron + flour | gentle man, sixties | potter + earth |
| `relation_anchor` | same church + kayak friend | Tuesday morning, coffee with a friend | lost husband | safe-enough understanding | in the moment we're having |
| `voice_lift` | 4 entries | 4 entries | 3 entries | 1 entry | 3 entries |
| `embodied_marker` | glasses | beard, hair, wrist tattoo | apron dusted with flour | white hair, navy + tie | fingers know the earth, grey hair |
| `attachment_node` | 2 entries | 4 entries | 4 entries | 2 entries | 6 entries |
| `wound_longing` | "doesn't have a vocabulary…" (path 1, wound only) | paired form: "stop moving — walls are cheaper than wounds" (path 1, true pair) | "lost her husband to a sudden illness" (path 2) | "steadier than his fear, kinder than his shame" (path 1, wound only) | "mend the rift with his son" (path 3, NEW Turn 247) |
| `refusal_shape` | 1 prose-extracted | 3 boundaries | 2 boundaries | 2 prose-extracted | 3 boundaries |
| `moral_theological_position` | "brother in Christ" (co-located with role_frame) | null (Christological substrate lives in refusals) | null | "Jesus means mercy to me." | null |
| `fact_atom` | 3 entries | 3 entries | 3 entries | 1 entry | 6 entries |

All five fixtures pass the round-trip audit AND the Tier 1 reference audit. The parametrized `all_grounded_fixtures_pass_tier1_reference_audit` test is the single-line regression surface.

## Apparatus-honest debt register (Mode B-shaped)

This branch's arc was structured as Mode B (loop-close-preemptive) of the apparatus-honest correction loop doctrine: gaps were named at rationale-note time and dispositioned in later turns. Naming the debts here as a register so future sessions can see what was paid down and what remains open.

| Debt | Named at | Dispositioned at | Status |
|---|---|---|---|
| Refusal-shape false positives (`does not` / `not because` / etc.) | Turn 1 (initial dump) | Turn 1 (needles narrowed) | closed |
| Quote-heavy identity prose splits awkwardly on nested `"..."` | Turn 1 (outline § Known limits) | Turn 1 (quote-aware splitter) | closed |
| Audit is a smoke test of the round-trip, not a fidelity claim | Turn 1 (outline § Known limits) | Turn 244 (Tier 1 reviewer impl); Tier 2/3 remain doc-only | partially closed |
| `wound_longing` is single-best-line, not paired | Turn 1 (outline § Known limits) | Turn 242 (paired selector) | closed |
| Maisie's wound-needles miss her wordforms (`lost` / `illness` / `late`) | Turn 245 (maisie reference rationale notes) | Turn 246 (needles added; reference updated alongside) | closed |
| Jasper Finn's wound lives in state.goals, not in v3 source fields | Turn 245 (jasper reference rationale notes) | Turn 247 (state.goals as third wound-source path; reference updated) | closed |
| Single-quote (`'…'`) span awareness | Turn 1 (outline § Known limits) | not yet — no fixture surfaces needing it | open (deferred — abandoned without a fixture-pressure trigger) |
| Tier 2 LLM-judged reviewer | Turn 243 (sketch) | Turn 243 (sketched + cost-analyzed); no impl | open (deferred — needs prompt-stack wiring decision first) |
| Tier 3 cross-character discrimination test | Turn 243 (sketch) | Turn 243 (sketched); no impl | open (deferred — right shape for `CHARACTER_IDENTITY_PAYLOAD=1` bite-test gate) |
| Maisie carries TWO wounds (husband + son) but `wound_longing` is single-Option | Turn 247 (state.goals dump shows her son-rift goal too) | not addressed — known limit of single-Option<String> wound_longing | open (abandoned — would require a structural change to the bucket type) |

The four open debts have honest dispositions. Two are *abandoned* (single-quote span awareness; Maisie dual-wound) per the "default to abandoned under uncertainty" discipline — they have no current fixture pressure and no path to earn beyond it. Three are *deferred* (Tier 2 reviewer; Tier 3 discrimination; prompts.rs wiring) — they have known triggers that haven't fired yet (a prompt-stack wiring decision earns Tier 2/3; a fixture pressure or live-DB observation earns single-quote span awareness).

## What this PR is the foundation for (if ever earned)

The harness is structured so that any future prompt-stack wiring would be a *separate, opt-in* change. The hook would be in `src-tauri/src/ai/prompts.rs`, behind an env flag `CHARACTER_IDENTITY_PAYLOAD=1`, mirroring the `CHARACTER_FORMULA_AT_TOP=1` pattern. The bite-test gate would be Tier 3 discrimination: paired probes (one cell with the encoded payload, one cell with the raw prose) against a stable-distinctive question, compared by Ryan's lived read or by an LLM rubric.

That work is not in scope for PR #46. PR #46 closes the loop on:

- the v3 taxonomy is implementable
- it round-trips losslessly
- it has an editorial reviewer earned for all five grounded fixtures
- it surfaces wounds across three source paths (identity / backstory / state-goals)
- it has a worldcli surface for live-DB inspection
- the residual seams are named, not hidden

## Honest scope clauses

- This is a research harness. None of its claims propagate to runtime behavior.
- The encoder is heuristic, not LLM-judged. Doctrine-judgment in LLM not python applies; the harness is a deterministic *surface*, not a verdict-producer.
- The Tier 1 reviewer grades against editorial readings, not against character truth. A reference can drift just as the encoder can; both edits should be reviewed with equal care.
- The audit-as-smoke-test caveat is partially closed (Tier 1 lands), not closed. Tiers 2 + 3 remain doc-only and are the right shape for any future prompt-stack wiring.

## What an external reviewer can read in 5 minutes

1. This consolidation report.
2. The diff of `src-tauri/src/ai/character_identity_payload.rs` (528 → ~570 lines including state.goals + paired selector).
3. The single parametrized test `all_grounded_fixtures_pass_tier1_reference_audit`.

That triad is sufficient to assess whether the harness shape is right and whether the residual seams are honestly named. The other 16 tests are regression surfaces; the seven companion reports are the rehearsal arc.

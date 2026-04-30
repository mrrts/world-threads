# Chiptune AI score generator — musicality probe (N=5 + N=2 re-probe)

**Tier:** sketch (N≤3 per condition).
**Arc:** Real User Held (chiptune-soundtrack), pre-Phase-5 tightening.
**Instrument:** `worldcli chiptune-probe` (added this commit), gpt-4o-mini.
**Total cost:** ~$0.005 across 7 probes.

## Why this probe

Phase 4 shipped the integration scaffold; Phase 5 (lived-play held-or-not gate) is the crown's actual criterion. Before stepping into Phase 5 I wanted evidence that the generator's continuation contract — the 5 invariants in `src-tauri/src/ai/chiptune_score.rs` — actually shapes phrase-to-phrase coherence the way the prompt claims. Three seed-phrase probes across distinct momentstamps for cross-stamp discrimination, plus a 3-phrase chain to test the contract.

## Probes run

**Seed phrases (no prev_phrase, momentstamp varied):**

| # | Momentstamp shape | Result key + bpm | Mood descriptor | Density (notes/rests) |
|---|---|---|---|---|
| 01 | Burden ⟶ heavy_𝓢 \| unresolved | C minor / 80 | "heavy and unresolved" | 8n / 4r |
| 02 | ∫ Wisdom·discern dμ ⟶ patient_𝓢 | C major / 120 | "gentle unfolding" | 5n / 1r |
| 03 | Π·Grace·γ ⟶ bright_𝓝u | C major / 120 | "uplifting brightness" | 10n / 3r |

**Continuation chain (each takes the previous as prev_phrase):**

| # | Momentstamp | Prev key/bpm | Out key/bpm | Mood transition |
|---|---|---|---|---|
| 04 | Burden softening · Wisdom holds | C minor / 80 | **A♭ major** / 85 | heavy → softening |
| 05 | Π·Grace ⟶ open_𝓝u | A♭ major / 85 | A♭ major / 80 | softening → gently opening |

## Findings

### Continuation contract — what holds

**Tempo continuity (±10% rule):** strong. Probes 04+05 stayed within 80–85 bpm despite their momentstamps individually pulling toward 120 (probe 03's Grace seed went to 120 unanchored). The contract beats stamp-driven tempo when prev exists.

**Voice-leading thread:** strong. Probe 01's lead ends at midi 63; probe 04's lead opens at 63 (same pitch). Probe 04 ends at 60; probe 05 opens at 60. Generator does deliberate same-pitch transitions across phrase boundaries.

**Mood-descriptor traceability:** strong. "heavy and unresolved" → "softening and evolving" → "gently opening" reads as a felt arc, not three independent moods.

**Cross-momentstamp discrimination (seed-only):** strong. The three seed shapes produced clearly distinct musical objects:

- Burden → minor key, slow tempo, low triangle bass (midi 31–36), sparse pulse_25 lead, 4 rests across the phrase.
- Wisdom → major key, mid-tempo, sine lead with sawtooth pad, very sparse (only 5 events) — "patient" rendered as silence-as-content.
- Grace → major key, mid-tempo, denser pulse_25 lead at higher register (64–72), noise rhythm hits.

### Continuation contract — what leaked

**Key-relationship discipline (PRE-tighten):** weak. Probe 04 modulated C minor → A♭ major. The doctrine says "stay in same key OR move to relative major/minor, dominant, subdominant." A♭ major is the **♭VI** of C minor — that's a tritone-shaped distance, not a closely-related key. The prompt's "related keys" list was loose enough that the LLM read "A♭ sounds bright after C minor dark" as license. **One leaky invariant out of five at sketch tier.**

**Voicing-density-follows-mood:** borderline. The chain produced ~12 events / phrase across all three (01: 12 events; 04: 12; 05: 12) despite the mood arc going heavy → softening → opening. Doctrine says density should track mood. Could be sample-of-3 noise; needs more probing before a tier claim.

**`previous_phrase_id` integrity:** small bug. The validator only backfilled when LLM omitted the field; LLM occasionally invented "seed001"/"seed002" labels not corresponding to any real prev phrase. By coincidence, probe 01's `phrase_id` was also "seed001" so the chain happened to be valid — but the validator was load-bearing on coincidence, not invariant.

## Tightening shipped (this commit)

1. **Key-relationship — STRICT.** System prompt rewritten with explicit allow-list (same / relative / parallel / dominant / subdominant) AND explicit forbid-list (tritone, ♭VI, distant keys), with worked examples for each move (e.g., "C minor → E♭ major"). Closes the leak probed at #04.
2. **Voicing density — explicit ranges.** Heavy/burden → ≤8 events; patient/wisdom → 8–14; bright/grace → 12–20 events. Numerical guidance per mood class. Bears more probing post-fix.
3. **`previous_phrase_id` — server-owned.** Validator force-overwrites the field from the actual prev phrase's `phrase_id`. LLM is told NOT to author this field. Removes coincidence-dependency.

## Re-probe verifying #1 (N=2)

Same prompt as probe 04 (prev = C minor 80 bpm Burden seed; momentstamp = Burden softening · Wisdom):

| # | Out key | Out bpm | Mood |
|---|---|---|---|
| 04 PRE | A♭ major (♭VI — forbidden) | 85 | softening and evolving |
| 06 POST | **E♭ major** (relative — admitted) | 88 | softening and moving |
| 07 POST | **E♭ major** (relative — admitted) | 84 | gentle and unfolding |

2/2 hit the canonical relative major. The strict allow-list is biting; the leak is closed.

## Honest scoping

- N=5 + N=2 is sketch tier. The tightening is verified at sketch tier on the key-relationship axis only.
- The density-follows-mood axis remains borderline — bears more probing before claim-tier on density discipline.
- Cross-stamp discrimination (Burden / Wisdom / Grace producing clearly different musical objects) is sketch-confirmed; characterization-tier would need N=5 per stamp class.
- Persona-sim caveat doesn't apply here (this is direct-LLM craft on the generator's output, not a persona's reception). The craft can be evaluated on the JSON's own merits.

## Forward-pointing seed

Phase 5 (the held-or-not gate) is now better-prepared. The remaining open question — does the music *hold* the user during real chats — is the only one Phase 5 can answer. The continuation contract is meaningfully tighter going in; the soundtrack should chain coherently across momentstamp shifts without obvious doctrine-violations.

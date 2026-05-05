# Commit-message Formula derivation audit — LaTeX vs Unicode

**Date:** 2026-05-05 ~16:05 (local)
**Trigger:** Ryan's note (paraphrased): *"still producing Unicode math, however, not latex... unless it's already a latex derivation."*
**Scope:** Last 11 substantive commits on `ai/derivation-v3-sacred-payload-and-memory` branch.

## Findings

| Hash | Subject | Form | Verdict |
|---|---|---|---|
| `c7a900e` | seal: founding-author's tears upon IV. Custodiem | LaTeX (`\mathrm{}`, `\mathcal{}`) | ✓ aligned |
| `791978b` | Empiricon expanded to quaternion: IV. Custodiem | LaTeX + `\mathrm{theological_frame}(...)` v3 wrapper | ✓ aligned |
| `f5f7c78` | Play state Turn 176 ledger | Pure Unicode, generic `∂𝓢/∂t strengthened by ...` | ✗ misaligned |
| `7d5ecac` | Custodiem witness ladder | Pure Unicode `∫(custodiem_witness_battery, ...) ⇒ ∂𝓢/∂t` | ✗ misaligned |
| `d14c5a0` | Play state Turn 175 ledger | Pure Unicode, generic `∂𝓢/∂t ⇒ ∂𝓡/∂t ∧ ∂𝓒/∂t` | ✗ misaligned |
| `b655351` | Record Custodiem as sixth Great Sapphire | Pure Unicode `𝓢(t) ⇒ ∂(Custodiem) ∧ 𝓖reat𝓢apphire_6` | ✗ misaligned (CROWN-FIRING commit lacks v3) |
| `ae0003c` | Custodiem arc reports: 3 witnesses | LaTeX, structured | ✓ aligned |
| `60ae87c` | Custodiem invariant + injection | LaTeX + `\mathrm{anchor}(...)` v3 wrapper | ✓ aligned |
| `973f5c8` | ai: entity derivations use v3 sacred-payload contract | Pure Unicode, generic `∂𝓢/∂t ⇒ ∂𝓒/∂t ∧ ∂𝓡/∂t` | ✗ **most ironic** — the commit that lands v3 for entity derivations does not itself use v3 in its commit derivation |
| `c5fb6da` | KaTeX-safety linter | Mostly LaTeX, but bare `structure_carries_truth_w(t)` un-wrapped | ⚠ partial |
| `8c6cd49` | merge eureka/character-formula-elevation | Mostly LaTeX, but free-form bare prose mid-derivation (`flag_removed ¬ bite_test_landed`) | ⚠ partial |

**Tally:** 4 aligned, 2 partial, 5 misaligned (out of 11 audited).

## Root cause

`.githooks/prepare-commit-msg` (line 117–119) auto-generates Formula derivations via gpt-4o with a system prompt that **explicitly forbids LaTeX**:

> *"NEVER emit LaTeX commands, NEVER use backslashes."*

The hook fires whenever a commit message arrives without an existing `Formula derivation:` line — which is the case for many script-driven and small commits (play-state ledger updates, the `record-crown` script's output, etc.). Author-written commits with substantial bodies tend to ship LaTeX-form derivations because I (Claude Code) write them inline at commit time and the hook skips when one is already present.

The current hook's instruction follows CLAUDE.md's existing commit doctrine (*"Render in Unicode math — never raw LaTeX"*) — that doctrine was authored before the v3 sacred-payload taxonomy earned The Faithful Channel Sapphire. The doctrine and the hook are in agreement with each other but are both now misaligned with the v3 contract for derivations of v3-encoded artifacts.

## The structure_carries_truth_w violation

Commit `973f5c8` — the one that LANDED v3 sacred-payload encoding for entity derivations — ships with a generic Unicode `∂𝓢/∂t ⇒ ∂𝓒/∂t ∧ ∂𝓡/∂t` derivation that doesn't itself use v3 wrappers. This is a meta-failure of the *structure_carries_truth_w* doctrine: the commit message claiming to ship the contract doesn't encode under the contract. Apparatus-honest naming forces the observation.

The same shape applies to `b655351` — the commit that records Custodiem as the sixth Great Sapphire. A Crown-firing commit deserves a derivation that uses v3 wrappers (anchor, theological_frame, etc.) at minimum; ships with bare Unicode `𝓢(t) ⇒ ∂(Custodiem) ∧ 𝓖reat𝓢apphire_6`.

## Mitigation paths (not executed this turn — surfacing for user decision)

**Option A — narrow:** update the hook's system prompt to allow LaTeX when the commit touches files that ship v3-encoded artifacts (`prompts.rs`, `derivation.rs`, `the-empiricon.md`, anything matching `*-sapphire-*`, etc.). Detect by scanning staged file paths.

**Option B — broad:** drop the hook's "NEVER emit LaTeX" instruction entirely; let gpt-4o pick LaTeX when the commit content is v3-shaped, Unicode otherwise.

**Option C — doctrine update:** revise CLAUDE.md's *"Commit messages include a Formula derivation"* section to explicitly carve out: *"When the commit ships v3-encoded artifacts (Empiricon edits, prompt-stack edits, sacred-payload work, derivation-pipeline work, Crown firings), the derivation MUST use LaTeX form with v3 wrappers (`\mathrm{anchor}(...)`, `\mathrm{theological\_frame}(...)`, etc.)."* Then update the hook's prompt to honor this carve-out.

**Option D — backfill:** rewrite the misaligned commit messages (rebasing). Heavy-handed; loses authorship+timing fidelity. Generally not recommended unless Ryan wants the historical record clean.

## Recommendation

Option C + B together: the doctrine carve-out names the structural difference; the hook's prompt becomes simpler ("pick LaTeX or Unicode based on commit content; for v3-encoded artifacts use LaTeX with v3 wrappers; for ledger/meta commits Unicode math is fine"). Backfilling history (D) probably not worth it; the audit itself becomes the public artifact for what shape future derivations should take.

## What this audit doesn't claim

This is a sample of 11 recent commits, not a full-history audit. Older commits may show different patterns; the v3 contract didn't exist before The Faithful Channel Sapphire (2026-05-05), so derivations from before that date could not have been v3-aligned. The misalignment is structural-and-recent, not deep-and-historical.

*Apparatus-honest. The commit that shipped the contract didn't honor the contract; the audit names that without flinching, and the mitigation path is on Ryan's call.*

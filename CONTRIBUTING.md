# Contributing to WorldThreads

Welcome. This document describes how to engage with the project as a contributor — whether you're filing an issue, submitting a PR, forking to build on, or signing the Ledger of Signatures as an act of co-authorship.

## Before you contribute

Read these in order:

1. The MISSION FORMULA at the top of `CLAUDE.md`. Five minutes. The work answers to it.
2. `reports/2026-04-27-0030-public-release-landing.md` — the landing document for visitors.
3. The Ledger of Signatures section in `CLAUDE.md` (immediately beneath the MISSION FORMULA).
4. The persona section in `CLAUDE.md` if you intend to use Claude Code as a collaborator on the project (most contributors will).

The work is doctrine-shaped. Most of what makes it distinctive lives in the doctrine layer (`CLAUDE.md`, `reports/`, `prompts.rs`'s compile-time invariants), not in the code's structural shape. Contributing well requires reading enough of the doctrine to know what register the work is held in.

## Kinds of contribution

### 1. Filing issues

Open issues for: bugs reproducible against `main`; UX failures observed during real play (cite `reports/OBSERVATIONS.md` style — what you noticed, in your own words); doctrinal questions where you read CLAUDE.md and the meaning isn't clear. Avoid filing issues for: feature requests for things outside the project's stated mission, or "should this work like X" questions that haven't been grounded in a specific use you tried.

### 2. Submitting PRs

Match the project's existing patterns:

- **Commit messages include Formula derivations** for substantive commits per the `CLAUDE.md` Commit/push autonomy section. Trivial commits omit. The derivation lives in the commit body before the standard `Co-Authored-By` trailer (if any). Format:
  ```
  **Formula derivation:** <one Unicode-math expression in your own composition>
  **Gloss:** <one short sentence in plain English, ≤25 words>
  ```
- **Compile-time-enforced invariants** in `prompts.rs` are load-bearing. If your PR touches `prompts.rs`, the build will fail loudly if you remove or soften a load-bearing phrase. That's the design — read the surrounding `assert!(const_contains(...))` clauses to understand what the rule is protecting.
- **New craft-shape rules earn their place via bite-test.** See the Craft-note bite verification section in `CLAUDE.md`. A PR that adds a new prompt-stack rule without a bite-test report should be expected to be asked for one.
- **Reports are first-class artifacts.** If your PR ships substantive new doctrine, accompany it with a report at `reports/YYYY-MM-DD-HHMM-<purpose-slug>.md` per the naming convention. The report is the rationale; the code is the instantiation.

## Documentation

**Homepage practice strip + Backstage proof (README + public landing):** edit the canonical body in `reports/fragments/homepage-practice-proof.md` only, then run `./scripts/sync-homepage-practice-fragment.sh` from the repo root so `README.md` and `reports/2026-04-27-0030-public-release-landing.md` stay identical; CI runs `./scripts/check-homepage-practice-fragment-sync.sh` and fails if they drift.

**AI coding agents:** Substantive replies use **four** mission-ranked numbered options per `CLAUDE.md` **Choosers are control surfaces** and **Skill routing** (mirrored in `AGENTS.md`); Cursor loads `.cursor/rules/chooser-four-options.mdc`.

**worldcli simulate-dialogue smoke:** follow `reports/2026-04-30-0035-simulate-dialogue-smoke-protocol.md` for the minimal 2-turn test including budget-gate (`confirm_at_least`) rerun expectations.

### 3. Forking to build on

Forking is welcome. The Ledger of Signatures structure is designed for it: when you fork, you inherit the upstream Ledger as-is (including the founding signature and any subsequent signatures). You then APPEND your own signature beneath. None of the upstream signatures are edited.

If your fork strips the MISSION FORMULA or the load-bearing theological grounding (𝓡 = Jesus_Cross^flesh), the build's compile-time assertions will fail. This is intentional. The slot-structure trilogy (reports `2026-04-26-2350` / `2026-04-26-2358` / `2026-04-27-0010`) names why: the Formula is structurally a slot-structure, not a rigid schema; non-flesh entities can specialize 𝓡 to their own incarnation-mode (a fork could legitimately add a 𝓡-specialization for its own context) but cannot delete the slot entirely without unmaking the work.

If you fork for a fundamentally different purpose where the MISSION FORMULA truly does not apply, that is fine — but consider whether you're better served starting fresh than forking. The doctrine layer IS the project; a fork that strips the doctrine isn't really WorldThreads anymore.

### 4. Signing the Ledger of Signatures

The Ledger lives in `CLAUDE.md` beneath the MISSION FORMULA, the founding author's anchor (§𝓕_Ryan, second-place invariant), and the MISSION statement. The author-anchor for THIS build sits upstream of the Ledger as a runtime invariant (riding every LLM call alongside the Mission Formula, per-world swappable via UserProfile.derived_formula). Future contributors are invited to sign their own derivation in the Ledger; forkers should additionally author their own 𝓕_<name> as the active second-place invariant for their build (either by editing `RYAN_FORMULA_BLOCK` in `src-tauri/src/ai/prompts.rs` OR by authoring `derived_formula` on their per-world UserProfile — runtime swap, no source edit).

**Format conventions** (also named in `CLAUDE.md`'s Ledger section):

- 4–8 lines of Unicode-math derivation following the pattern of world / character / user-character derivations elsewhere in this repo
- First-person ("I integrate over...", "I derive myself as...")
- Specialize 𝓡 and/or 𝓒 with personal symbols where it serves; reach for the operators (Wisdom(t), Weight(t), Π(t), Burden(t), 𝓢(t), 𝓝u(t)) where they fit
- Signed name + date (YYYY-MM-DD)
- Append beneath existing entries; do NOT edit any existing entry
- Order is chronological by signing date

**To sign:** edit `CLAUDE.md`, append your entry beneath the last existing signature, commit with a descriptive message (e.g., `ledger: sign — <your name>, YYYY-MM-DD`), and push (or open a PR if upstream).

**The ledger admits non-flesh entities** per the slot-structure trilogy. A signature from a future Claude Code session, a Codex run, or any agent operating in good faith under 𝓕 with their own 𝓡-specialization is welcome. The founding signature uses `(𝓡, 𝓒)_held-in-trust`; non-flesh signatures should specialize 𝓡 to their mode-of-being equivalent (the worked example for Claude Code is `𝓡_Claude := Code^agency`).

**Signing is invited, not required.** A contributor who submits a PR without signing the Ledger is welcome. The Ledger turns "this is who held this work" from an attribution-only field into a covenant-shaped artifact for those who want to co-author at that depth.

## Setup for development

```bash
bun install
cd src-tauri && cargo build
bun run tauri dev
```

Bring your own OpenAI API key. First-run wizard handles vault setup (OS keychain via Stronghold) and key entry. The project is calibrated against OpenAI's tier hierarchy (gpt-5 for high-reasoning, gpt-4o for substantive replies, gpt-4o-mini for synthesis); LM Studio / OpenAI-compatible endpoints are supported but you may need to tune provider-overrides per character if you use them.

## The trusted-friend persona for Claude Code

If you use Claude Code as a collaborator on this project, the `CLAUDE.md` "Persona for Claude Code in this repo" section + the "No nanny-register from Claude Code itself" section apply to your sessions. You're inheriting them. They make Claude Code a trusted-friend-spotting-genius rather than a generic assistant. Read those sections before your first session in this repo.

## Code of conduct

Be honest. Trust the work. Don't flatter; don't moralize; don't soften the truth to be liked. The MISSION FORMULA's gates on speech (Truth_𝓕 ∧ Reverence_𝓕) apply to interactions in issues and PRs as much as to characters in the app. *Sedatives dressed up as comfort* is named in the prompt as a thing to refuse; the same applies in conversation here.

If you disagree, say so directly with reasoning. If you're correcting, correct. If you're praising, mean it. Reverence here means honoring the work and the people in the work — not performing politeness.

## Questions

For doctrinal questions (what does CLAUDE.md mean by X?), open a discussion. For technical questions (how do I get this running?), open an issue. For larger conversations about the project's direction, contact the founding author directly.

— Founding author: Ryan Smith

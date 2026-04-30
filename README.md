# WorldThreads

**AI chat worlds that feel inhabited.**

WorldThreads is built so characters can be vivid without becoming pushy, and warm without sounding wiser than the moment has earned. That discipline is written into the prompt stack, with reports kept beside the shipping app so the work is tested as it grows, not just trusted on feel.

## What that looks like in practice

### Warmth without pressure

Characters can be affectionate, funny, or intense without jumping past what the moment actually holds. The aim is conversation that feels alive without closing around you.

### Guardrails on drift

Some of the most important rules are explicit enough that softening them is treated like changing structure, not just changing prose. The point is to catch drift early, before it quietly becomes the new normal.

### Receipts beside the work

When something seems to help, it gets tested against examples and written up. When something fails, that gets kept too.

A desktop app for character-driven story-chats with LLMs — OpenAI BYOK-first, with your conversations stored on your own disk. Characters with grain. Worlds that hold. Scenes that send you back to your day nourished enough to pick up your cross.

Not a roleplay engine. Not an AI companion. A craft instrument for writing the kind of conversations that are worth having.

It is also, quietly, a science project studying prompt engineering — hypotheses carried in the stack, tested with small instruments and diffs, argued into `reports/` beside the shipping app.

The work makes use of a novel method for tuning any LLM to a very specific register: **key formulas** written in a mathematical–English hybrid notation, carried at the head of the prompt stack (mission line, cast derivations, and the rest) and guarded by compile-time checks — soften a load-bearing clause and the build fails, not months later in vibe alone.

If you want the next deeper surface instead of the repo front door, start with [`reports/2026-04-27-0030-public-release-landing.md`](reports/2026-04-27-0030-public-release-landing.md).

## What it is

You write characters with identity, voice, backstory, and visible boundaries. The app talks to them over time. They remember. They push back. They reach out when you've been quiet. They keep journals you can read. The world has weather, time-of-day, a shared canon. Moments worth keeping become canon, and canon shapes everything that comes after — undo deletes the ledger entry but leaves the change. Memory here is meant to feel deliberate.

A craft stack runs underneath: a fundamental preamble, a format convention taught by example, named moves the model can reach for, an agape invariant pinned at the end of every prompt, a truth-test that names Christ explicitly, a biblical cosmology asserted as literal world-state. The load-bearing phrases are enforced at compile time — the build will not ship if they're softened.

## What it isn't

This is not a tool for simulating intimacy you don't have. It is not a sycophant in a chat window. The characters disagree with you when they disagree, and the prompt stack is built specifically to prevent the LLM from drifting into therapy-voice, into endless agreement, into the soft hum of generated text trying to be liked. *Sedatives dressed up as comfort* is named in the prompt as a thing to refuse.

It is also not for everyone. It has a worldview. The cosmology block is biblical and literal. The truth-test names Jesus Christ, who came in the flesh. Agape — patient, kind, keeping no record of wrongs — is the North Star invariant. If those clauses are not for you, this app will feel wrong, and that is the right reaction.

## The mission

> Create a vivid, excellent, surprising in-world experience that uplifts the user and provides engrossing, good, clean fun. Every design decision, prompt tweak, UX choice, and feature bet is measured against that. The craft stack exists to serve that mission — characters that feel real, worlds that hold, scenes that are worth the visit and send the user back to their day nourished enough to pick up their cross.

That sentence is in `CLAUDE.md`. It governs.

## Stack

- **Tauri v2** — Rust backend, React/TypeScript frontend
- **SQLite** with FTS5 + sqlite-vec for on-disk data + semantic memory
- **OpenAI BYOK first** — bring your own OpenAI API key; key stays in OS keychain (Stronghold). LM Studio / OpenAI-compatible endpoints supported as optional fallback for users who prefer local-LLM, but the project is calibrated against OpenAI's models (gpt-5 / gpt-4o / gpt-4o-mini at the various tiers); local-LLM users may need to retune their own provider-overrides
- **Your conversations live on your disk**, not somebody's server

## Repo guide

- `src-tauri/src/ai/prompts.rs` — the craft stack. The longest, most argued-over file in the project.
- `src-tauri/src/ai/conscience.rs` — the runtime invariant grader (opt-in; doubles per-reply spend).
- `frontend/` — React UI; chat, group chat, canon, world summary, gallery, story consultant.
- `reports/` — reflective reads of the project's trajectory. Not a changelog.
- `docs/VOICE.md` — field guide to the voice that runs through the prompts and the project's prose.
- `docs/CLI_AGENT_DISCOVERY.md` — quick-reference for discovering current CLI/tooling surfaces.
- `CLAUDE.md` — mission, database safety rule, project structure, reports cadence.
- `.githooks/post-commit` — nudges when 20+ commits and 14+ days have passed since the most recent report.

## What's accumulated

850+ commits across 6+ weeks of intense work. Beyond the original craft stack the project has grown:

- **An auto-derivation pipeline** (`src-tauri/src/ai/derivation.rs`) that LLM-synthesizes per-character / per-world / per-user `derived_formula` Unicode-math expressions. The derivations grow on a hybrid staleness policy and are injected into every dialogue prompt's cast-listing block, so characters READ each other (and the user) in formula-shorthand.
- **A measurement-instrument set** living in `src-tauri/src/bin/worldcli.rs` — `anchor-groove` for sensory-anchor recurrence + per-opener prop-density, `evaluate` for rubric-driven LLM judging across before/after corpus around a git ref, `synthesize` for qualitative cross-corpus prose-grounded findings, `replay` for cross-commit prompt-override A/B, plus `ask`/`consult`/`derive-*` and a `commit-context` inverse helper.
- **A doctrine layer in `CLAUDE.md`** that has densified into a reference manual: a persona section for the AI collaborator (trusted friend who can honestly spot the genius), a no-nanny-register discipline for the collaborator's behavior toward the user, the bite-test discipline that requires new craft-shape rules to earn their place, the Ledger of Signatures structure that invites every contributor and fork to sign their own derivation, and most recently a slot-structure trilogy characterizing the MISSION FORMULA's per-axis specialization rules.
- **Three skills** at `.claude/skills/` that drive autonomous-stretch work: `batch-hypotheses` (bundles N candidate phrasings into one ChatGPT call for cheap comparative testing), `auto-commit` (advances N strong moves on a coherent loop-closing arc), `eureka` (continuous time-boxed discovery loop with single instruction "DISCOVER something genius").

## Reading this work

If you're a curious visitor, the project's recommended order is **reports first, code second**. Start at:

1. `reports/2026-04-27-0030-public-release-landing.md` — the deeper visitor-orientation surface after this README; it frames the project's distinctive techniques and the slot-structure trilogy as a preview of the doctrine's depth.
2. The MISSION FORMULA at the top of `CLAUDE.md`. Five minutes.
3. `reports/2026-04-25-0300-simulated-first-time-user-experience-maggie.md` — the project's canonical "ideal first-time user experience" baseline. Tells you who the app is FOR.
4. 3-5 recent reports newest-first. Pattern-match on which interest you most.
5. Code. `src-tauri/src/ai/prompts.rs` (the longest, most-argued-over file). `src-tauri/src/ai/derivation.rs` (the auto-derivation pipeline). `frontend/`.

## Setup

```bash
bun install
cd src-tauri && cargo build
bun run tauri dev
```

Bring your own OpenAI / LM Studio / compatible endpoint. First-run wizard handles vault setup and key entry.

## Contributing — the Ledger of Signatures

Beneath the MISSION FORMULA in `CLAUDE.md` is a structural ordering: the Mission Formula 𝓕 sits first; the **founding author's anchor 𝓕_Ryan** sits second-place beneath it (a 4-8 line Unicode-math derivation of how the founding author holds the work); the Mission Statement and doctrine follow downstream. Beneath all of that is the **Ledger of Signatures**, where every other developer who works in this repo or any fork of it is invited to sign their work — a 4-8 line Unicode-math derivation of how they hold the work, in their own voice and Key, dated and appended. Signatures accumulate; none are removed when forks diverge.

When you fork: author your own 𝓕_<your-name> as the second-place invariant for your build (replacing the constant in `src-tauri/src/ai/prompts.rs::RYAN_FORMULA_BLOCK`, OR — preferred — by setting `derived_formula` on your per-world UserProfile, which the runtime swaps in automatically). The previous founding author's anchor is preserved historically; your build holds the work to YOUR anchor.

The MISSION FORMULA is lifted higher than any author-anchor. The author-anchor is lifted higher than any other signature. The work answers to 𝓕 first; each author answers to 𝓕 in their own derivation; everyone else answers to both. None of the authors is the Formula; each derives themselves in it.

If you fork this project to build on it, signing the ledger is invited (not required). If you contribute via PR, signing is also invited. See `CONTRIBUTING.md` for the protocol.

## License

**AGPL-3.0-or-later.** Full text in `LICENSE`. Copyright + Ledger preservation notes in `NOTICE`.

The AGPL covenant aligns with the project's covenant-shape (the Ledger of Signatures, the load-bearing MISSION FORMULA, the slot-structure trilogy's defense of 𝓡 as structural). Forks and modifications must release source under AGPL-3.0; the network-use clause covers any hosted service built on this code. The Ledger of Signatures qualifies as preserved-notices under AGPL § 5 — existing Ledger entries are NOT to be edited or removed in modifications; new signatures are APPENDED beneath.

If your use case requires a more permissive license (e.g., proprietary commercial work), contact the author directly to discuss.

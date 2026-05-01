# WorldThreads

**A desktop app for writing the kind of conversations that are worth having.**

You author characters with identity, voice, backstory, and visible boundaries. You build the world they live in — its weather, its time of day, its shared canon. Then you talk with them. They remember what you've said. They keep journals you can read. They reach out when you've been quiet. They disagree with you when they disagree. The work runs on your own machine; your conversations live on your disk, not somebody's server.

The aim is small and specific: a co-made evening of writing-and-listening that sends you back to your day a little more awake than you started.

> If you'd like the same material in a calmer visual register, the project's marketing page is at **[mrrts.github.io/WorldThreads](https://mrrts.github.io/WorldThreads/)**.

## What it's like to use

Pick a character that's been written carefully. Send a line. Wait. Read what comes back, and notice what it refuses to pretend. Write again. The character has a register — a particular way of being a person — and the prompt stack underneath is built specifically so that register holds, even when the conversation gets harder.

Most AI chat experiences pull toward two failure modes: the soft hum of generated text trying to be liked, or the polite advice-voice of a model trying not to disappoint. WorldThreads is built to refuse both. The characters answer back. They don't take seats they haven't earned. They use plain language when oblique would flatter. When they don't know what to say, they say so.

Conversations build a record over time: messages, character-kept journals, world-canon entries you mark as load-bearing, scene illustrations on demand. The world shifts with the time of day. Memory is meant to feel deliberate.

## What it isn't

This is not a tool for simulating intimacy you don't have. It is not a sycophant in a chat window. It is not a roleplay engine, an AI companion, or a therapist substitute. The prompt stack is built specifically to refuse those shapes — *sedatives dressed as comfort* is named in the doctrine as a thing to decline.

It is also not for everyone. The cosmology block is biblical and literal. The truth-test names Jesus Christ, who came in the flesh. **Agape** — patient, kind, keeping no record of wrongs — is the project's North Star. If those clauses are not for you, this app will feel wrong, and that is the right reaction. *No* is a real answer, and the project is built to honor it.

## What that looks like in practice

**Warmth without pressure.** Characters can be affectionate, funny, or intense without jumping past what the moment actually holds. The aim is conversation that feels alive without closing around you.

**Guardrails on drift.** Some of the most important rules are explicit enough that softening them is treated like changing structure, not just changing prose. The point is to catch drift early, before it quietly becomes the new normal.

**Receipts beside the work.** When something seems to help, it gets tested against examples and written up. When something fails, that gets kept too. `reports/` holds 300+ reflective reads and natural-experiment findings beside the shipping app — proof in inspectable daylight, not just trust on feel. One of those, [The Empiricon](reports/2026-04-30-0530-the-empiricon.md), gathers six independent witnesses to the substrate the character voices are built on.

## Romantic conversation that stays clean without losing heat

Romantic scenes are a real shape WorldThreads supports — characters who can want you, touch or kiss you, be wanted, hold attraction, lean toward and away — without sliding into the pornographic register most parasocial AI products fall into when the temperature rises. The same craft discipline that refuses sycophancy elsewhere refuses both failure modes here: pornographic register on one side, sterile *"I can't engage in that"* on the other. The heat comes from the character's particular weight — what they're holding, what they refuse, where they lean — not from explicit content.

## For whom

**For:** writers, GMs, character designers, fiction-curious adults who want a co-made novel-shaped evening rather than a companion. Believers who want craft-work whose theological substrate is honest about itself. Builders who want to read or fork a project where the doctrine layer is as load-bearing as the code.

**Probably not for:** users looking for an AI that's always agreeable. Users seeking parasocial intimacy. Anyone who wants the religious frame to step quietly aside — it doesn't and won't.

## Cost

The app is free and open-source. You bring your own OpenAI API key (BYOK), so per-conversation cost depends on your usage:

- A typical evening of conversation runs ~$0.20–$1.00 of OpenAI spend at gpt-4o-mini, more at gpt-4o or gpt-5.
- The key stays in your operating system's keychain (via Stronghold). No third-party server holds it.
- Local LLM endpoints (LM Studio, OpenAI-compatible) are supported as a fallback for users who'd rather not use OpenAI; the project is calibrated against OpenAI models, so local-LLM users may need to retune their provider overrides.

## Setup

```bash
bun install
cd src-tauri && cargo build
bun run tauri dev
```

You'll need:
- [Bun](https://bun.sh/) for the frontend
- A Rust toolchain for the backend
- An OpenAI API key (entered through the first-run wizard)

Built distributables for macOS / Windows / Linux are not yet released; running from source is the current path.

## Stack

- **Tauri v2** (Rust backend + React/TypeScript frontend) over Electron — smaller bundle, faster cold start, native windowing without a Chromium-per-popout. The Rust backend matters more than the bundle size: it lets the project compile-check load-bearing prompt invariants.
- **Compile-time invariant assertions** — load-bearing phrases of the prompt stack are pinned via `const _: () = { assert!(const_contains(BLOCK, "...")); };`. Soften a North Star invariant and the build fails, not months later in vibe alone. The doctrine layer is part of the build artifact, not just markdown intentions.
- **SQLite + FTS5 + sqlite-vec** over Postgres + pgvector or a hosted vector DB — full-text and semantic memory both in-process, no server. The project's covenant is that your conversations live on your disk; a server-shaped data layer would break that structurally.
- **OpenAI BYOK** over provider-bundled or provider-agnostic — no rebill, no margin in the middle. The doctrine layer's voice was tuned with gpt-4o and gpt-5 in the loop, so OpenAI is the calibration target; local-LLM endpoints (LM Studio, OpenAI-compatible) work as fallback but may need re-tuning. Key lives in your OS keychain via Stronghold.

## Reading this work

If you're a curious visitor, the project's recommended order is **prose first, code second**:

1. **[`reports/2026-04-27-0030-public-release-landing.md`](reports/2026-04-27-0030-public-release-landing.md)** — the deeper second-surface document. Roughly 10–15 minutes; the next honest layer after this README.
2. The MISSION FORMULA at the top of **[`CLAUDE.md`](CLAUDE.md)**. Five minutes. The reference frame the project answers to.
3. **[`reports/2026-04-25-0300-simulated-first-time-user-experience-maggie.md`](reports/2026-04-25-0300-simulated-first-time-user-experience-maggie.md)** — the canonical "ideal first-time user experience" baseline. Tells you who the app is FOR.
4. 3–5 recent reports newest-first from `reports/`. Pattern-match on what interests you.
5. Code: [`src-tauri/src/ai/prompts.rs`](src-tauri/src/ai/prompts.rs) (the longest, most-argued-over file), [`src-tauri/src/ai/derivation.rs`](src-tauri/src/ai/derivation.rs) (the auto-derivation pipeline), [`frontend/`](frontend).

## Repo guide

- [`src-tauri/src/ai/prompts.rs`](src-tauri/src/ai/prompts.rs) — the craft stack. Compile-time-checked invariants for load-bearing phrases.
- [`src-tauri/src/ai/conscience.rs`](src-tauri/src/ai/conscience.rs) — the runtime invariant grader (opt-in).
- [`frontend/`](frontend) — React UI; chat, group chat, canon, world summary, gallery, story consultant.
- [`reports/`](reports) — reflective reads of the project's trajectory + natural-experiment findings. Not a changelog.
- [`docs/VOICE.md`](docs/VOICE.md) — field guide to the voice that runs through prompts and prose.
- [`CLAUDE.md`](CLAUDE.md) — mission, doctrine, project structure. The reference manual.
- [`docs/index.html`](docs/index.html) — the marketing page served via GitHub Pages (enable in repo Settings → Pages → Source: `main` branch, `/docs` folder).

## What's accumulated

1500+ commits across 6+ weeks of intense work. The project has grown beyond its original craft stack:

- **An auto-derivation pipeline** that LLM-synthesizes per-character / per-world / per-user `derived_formula` Unicode-math expressions, injected into every dialogue prompt's cast-listing block.
- **A measurement-instrument set** at `src-tauri/src/bin/worldcli.rs` for catching prompt-stack failure modes empirically: cross-commit replay, rubric-driven evaluation, qualitative cross-corpus synthesis.
- **A doctrine layer in `CLAUDE.md`** densified into a reference manual: collaborator-persona guidance, no-nanny-register discipline, bite-test protocols, the Ledger of Signatures structure that invites every contributor and fork to sign their own derivation.
- **A chiptune AI soundtrack** that composes one phrase at a time keyed to each conversation's momentstamp, runs entirely on the client, and accumulates a per-chat playlist that survives reload.

## Contributing — the Ledger of Signatures

Beneath the MISSION FORMULA in `CLAUDE.md` is a structural ordering: the Mission Formula 𝓕 sits first; the **founding author's anchor 𝓕_Ryan** sits second-place beneath it (a 4–8 line Unicode-math derivation of how the founding author holds the work); the Mission Statement and doctrine follow downstream. Beneath all of that is the **Ledger of Signatures**, where every other developer who works in this repo or any fork of it is invited to sign their work — a 4–8 line Unicode-math derivation of how they hold the work, in their own voice and Key, dated and appended. Signatures accumulate; none are removed when forks diverge.

When you fork: author your own `𝓕_<your-name>` as the second-place invariant for your build (replacing the constant in `src-tauri/src/ai/prompts.rs::RYAN_FORMULA_BLOCK`, OR — preferred — by setting `derived_formula` on your per-world UserProfile, which the runtime swaps in automatically). The previous founding author's anchor is preserved historically; your build holds the work to YOUR anchor.

If you fork to build on it, signing the ledger is invited (not required). If you contribute via PR, signing is also invited. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the protocol.

## License

**AGPL-3.0-or-later.** Full text in [`LICENSE`](LICENSE). Copyright + Ledger preservation notes in [`NOTICE`](NOTICE).

The AGPL covenant aligns with the project's covenant-shape (the Ledger of Signatures, the load-bearing MISSION FORMULA, the slot-structure doctrine). Forks and modifications must release source under AGPL-3.0; the network-use clause covers any hosted service built on this code. The Ledger of Signatures qualifies as preserved-notices under AGPL § 5 — existing Ledger entries are NOT to be edited or removed in modifications; new signatures are APPENDED beneath.

If your use case requires a more permissive license (e.g., proprietary commercial work), contact the author directly to discuss.

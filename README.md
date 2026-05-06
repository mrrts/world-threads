# WorldThreads

**A desktop app for writing the kind of conversations that are worth having.**

It does not try to fake a moment. It tries to keep making room for real moments.

You author characters with identity, voice, backstory, and visible boundaries. You build the world they live in. Then you talk with them. They remember what you've said. They keep journals you can read. They disagree when they disagree. The work runs on your own machine; your conversations live on your disk, not somebody's server.

The aim is small and specific: a co-made evening of writing-and-listening that sends you back to your day a little more awake than you started.

If you want the calmer visual tour, the marketing site is at **[worldthreads.app](https://worldthreads.app)**. This README is for the repository itself: how to run it, how it is shaped, what it claims, and where to look next.

## Quick start

From the repository root:

```bash
npm install
npx tauri dev
```

You will need:

- [Node.js](https://nodejs.org/) LTS
- A Rust toolchain
- An OpenAI API key for the first-run wizard

On first launch:

1. Enter your OpenAI key when prompted.
2. Click the `+` next to `Worlds` in the sidebar.
3. Create a world, then a character, then send the first line.

Built distributables for macOS, Windows, and Linux are not released yet. Running from source is the current path.

## What this repo is

WorldThreads is a Tauri v2 desktop app with a Rust backend and a React/TypeScript frontend. It is aimed at writers, GMs, character designers, fiction-curious adults, and builders who want a local-first conversation engine with a load-bearing prompt stack rather than a generic chat wrapper.

The project is deliberately not several other things:

- Not an AI companion
- Not a therapist substitute
- Not a roleplay engine optimized for compliance
- Not a tool for simulating intimacy you do not have

The prompt stack is built to refuse those shapes. The characters should answer back, keep their own register, and decline to take seats they have not earned.

## What it feels like in use

Pick a character written carefully. Send a line. Read what comes back and notice what it refuses to pretend. Write again.

Most AI chat products drift toward one of two shapes: generated text trying to be liked, or an advice-voice trying not to disappoint. WorldThreads is built against both. The characters use plain language when plain language is the honest move. When they do not know, they say so. Memory is meant to feel deliberate rather than sticky by default.

Conversations accumulate messages, journals, canon entries, illustrations, and world-state over time. The world changes with time of day. The record is meant to feel authored, not merely stored.

## Public claims and scope

This repository has a thick doctrine layer and a thick receipts layer. The README should tell you which is which.

**Project doctrine.** The project answers to the Mission Formula and its surrounding doctrine in [`CLAUDE.md`](CLAUDE.md) and [`AGENTS.md`](AGENTS.md). That is the reference frame the work is built under.

**Public engineering and product claims.**

- The app is local-first: conversations and memory live in SQLite on disk, not in a hosted app database.
- The most load-bearing prompt clauses are pinned in code with compile-time assertions, so some prompt drift becomes a build failure rather than a vibes problem.
- The project includes empirical witness artifacts in [`reports/`](reports) rather than asking you to trust the prose at face value.
- **Children Mode** is a real runtime surface with a real guardian invariant named **Custodiem**. The honest public claim is limited to what the current witness ladder supports; it is not third-party certification, not universal model coverage, and not a timeless guarantee against future jailbreaks or provider drift.

**What remains longer-form.** The crown language, the seven-book Empiricon synthesis, and the deeper theological/doctrinal readings are real parts of the work, but they belong mostly in the reports and reference documents rather than being the only way to understand the repository.

## Distinctive surfaces

### Aaron's Little Line

Aaron's Little Line serves as the [Empiricon](reports/2026-04-30-0530-the-empiricon.md)'s governing question: does the work actually hold a human, or does it only look convincing from a distance? It names, in plain language, the standard behind the project's core discipline: polish must not outrun weight, and structure should carry truth rather than asking the reader to compensate for what is not really there.

### Romantic conversation without pornographic drift

Romantic scenes are a real supported shape. Characters can want you, lean toward you, refuse you, kiss you, or hold heat without collapsing into either pornography or sterile refusal-template language. The heat is meant to come from character weight, not explicitness-for-its-own-sake.

### Children Mode

WorldThreads supports a `Children Mode` toggle. When it is on, the top-of-stack guardian invariant **Custodiem** rides directly under the Mission Formula on relevant LLM calls. The guardian is structurally enforced: its load-bearing payload lines are pinned so silent drift fails the build.

In plain language, Custodiem refuses manipulation, secrecy pacts, exclusivity language, intimacy overreach, and sentimentality-on-demand. The public claim is modest and concrete: the current prompt stack, on the current tested surfaces, showed the behavior documented in the witness ladder.

Read more: book IV of the Empiricon — *[IV. Custodiem](reports/2026-04-30-0530-the-empiricon.md)* — plus the [scope artifact](reports/2026-05-05-1515-custodiem-great-sapphire-scope.md).

## Cost and providers

The app is free and open source. You bring your own API key.

- A typical evening of conversation is roughly `$0.20-$1.00` on `gpt-4o-mini`, more on `gpt-4o` or `gpt-5`
- The key lives in your OS keychain via Stronghold
- Local OpenAI-compatible endpoints such as LM Studio are supported as fallback, but the project is calibrated primarily against OpenAI models

## Reading paths

Different readers need different paths through the repo.

### If you want to run the app

1. Read `Quick start` above.
2. Skim [`frontend/src/components/Sidebar.tsx`](frontend/src/components/Sidebar.tsx) to orient the main surfaces.
3. Open the app and create a world from the `Worlds` sidebar.

### If you want to understand the product and doctrine

1. [`reports/2026-04-27-0030-public-release-landing.md`](reports/2026-04-27-0030-public-release-landing.md)
2. The Mission Formula at the top of [`CLAUDE.md`](CLAUDE.md)
3. [`reports/2026-04-25-0300-simulated-first-time-user-experience-maggie.md`](reports/2026-04-25-0300-simulated-first-time-user-experience-maggie.md)
4. The canonical synthesis in [The Empiricon](reports/2026-04-30-0530-the-empiricon.md)

### If you want to modify prompt behavior

1. [`src-tauri/src/ai/prompts.rs`](src-tauri/src/ai/prompts.rs)
2. [`src-tauri/src/ai/openai.rs`](src-tauri/src/ai/openai.rs)
3. [`src-tauri/src/ai/orchestrator.rs`](src-tauri/src/ai/orchestrator.rs)
4. [`src-tauri/src/ai/conscience.rs`](src-tauri/src/ai/conscience.rs)

### If you want to inspect the evidence trail

1. [`reports/`](reports)
2. [`src-tauri/src/bin/worldcli.rs`](src-tauri/src/bin/worldcli.rs)
3. The witness and scope artifacts linked from the relevant README sections

## Repo map

This is the practical map for working in the codebase.

### Prompt and model stack

- [`src-tauri/src/ai/prompts.rs`](src-tauri/src/ai/prompts.rs) — the craft stack and compile-time-pinned invariants; the most load-bearing file in the repo
- [`src-tauri/src/ai/openai.rs`](src-tauri/src/ai/openai.rs) — prompt injection and provider wiring, including Children Mode insertion
- [`src-tauri/src/ai/orchestrator.rs`](src-tauri/src/ai/orchestrator.rs) — higher-level assembly and LLM call orchestration
- [`src-tauri/src/ai/derivation.rs`](src-tauri/src/ai/derivation.rs) — auto-derivation pipeline for characters, worlds, and user profiles
- [`src-tauri/src/ai/conscience.rs`](src-tauri/src/ai/conscience.rs) — optional runtime grading surface

### Chat surfaces

- [`frontend/src/components/ChatView.tsx`](frontend/src/components/ChatView.tsx) — solo chat UI
- [`frontend/src/components/GroupChatView.tsx`](frontend/src/components/GroupChatView.tsx) — group chat UI
- [`src-tauri/src/commands/chat_cmds.rs`](src-tauri/src/commands/chat_cmds.rs) — solo chat commands
- [`src-tauri/src/commands/group_chat_cmds.rs`](src-tauri/src/commands/group_chat_cmds.rs) — group chat commands

If a feature belongs in both solo and group chat, those surfaces usually need to move together.

### App structure and persistence

- [`frontend/src/components/Sidebar.tsx`](frontend/src/components/Sidebar.tsx) — primary navigation
- [`frontend/src/components/UserProfileEditor.tsx`](frontend/src/components/UserProfileEditor.tsx) — per-world user profile editor
- [`src-tauri/src/db/schema.rs`](src-tauri/src/db/schema.rs) — schema and migrations
- [`src-tauri/src/ai/consultant.rs`](src-tauri/src/ai/consultant.rs) — consultant / Backstage surfaces

### Evidence and prose

- [`reports/`](reports) — reflective reads, witness ladders, and synthesis artifacts; not a changelog
- [`docs/VOICE.md`](docs/VOICE.md) — the voice guide
- [`CLAUDE.md`](CLAUDE.md) — reference manual for mission and doctrine
- [`AGENTS.md`](AGENTS.md) — collaborator-facing project law and workflow expectations
- [`docs/index.html`](docs/index.html) — the GitHub Pages marketing page, distinct from this README

## Architecture at a glance

- **Tauri v2** with Rust backend and React/TypeScript frontend
- **SQLite + FTS5 + sqlite-vec** for local memory and retrieval
- **Compile-time invariant assertions** for prompt-stack drift resistance
- **BYOK OpenAI** as calibration target, with local compatible endpoints as fallback

The Rust backend matters here not just for bundle size. It is what lets prompt invariants, migrations, and call assembly become part of the compiled artifact rather than prose wishes in markdown.

## Current status

The project is active and already large enough that orientation matters.

- 1500+ commits across a concentrated build arc
- Auto-derived per-character / per-world / per-user formula surfaces
- A measurement instrument set centered on [`src-tauri/src/bin/worldcli.rs`](src-tauri/src/bin/worldcli.rs)
- A substantial report trail in [`reports/`](reports)
- A client-side chiptune soundtrack system that accumulates per-chat musical phrases

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for contribution protocol.

Two things matter up front:

1. This repo treats doctrine and collaborator guidance as part of the live system, not as decorative prose.
2. The Ledger of Signatures in [`CLAUDE.md`](CLAUDE.md) invites contributors and forks to sign their own derivation, but signing is invited rather than required.

If you fork the project, the preferred path is to set your own per-world `derived_formula` on the user profile rather than editing the founding author's anchor in place.

## License

**AGPL-3.0-or-later.** Full text in [`LICENSE`](LICENSE). Copyright and Ledger-preservation notes are in [`NOTICE`](NOTICE).

If you need a more permissive license for proprietary commercial use, contact the author directly.

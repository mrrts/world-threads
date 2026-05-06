# Contributing to WorldThreads

Welcome. This repo is doctrine-shaped, prompt-shaped, and evidence-shaped. Contributing well means reading enough of the project’s reference frame to know what kind of work you are stepping into.

## Get started

Read these first:

1. The Mission Formula at the top of [`CLAUDE.md`](CLAUDE.md)
2. [`README.md`](README.md)
3. [`reports/2026-04-27-0030-public-release-landing.md`](reports/2026-04-27-0030-public-release-landing.md)
4. The Ledger of Signatures section in [`CLAUDE.md`](CLAUDE.md)

If you plan to collaborate through Claude Code or another agent surface, also read the collaborator-facing guidance in [`CLAUDE.md`](CLAUDE.md) and [`AGENTS.md`](AGENTS.md). The doctrine layer is part of the live system here, not a decorative note beside the code.

## Development setup

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

The project is calibrated primarily against OpenAI models. OpenAI-compatible local endpoints are supported, but if you use them you may need to retune provider overrides per character.

## How to contribute

### Filing issues

Open issues for:

- Bugs reproducible against `main`
- UX failures observed during real play
- Technical questions about getting the app running
- Doctrinal questions after you have read the relevant reference text and still think the meaning is unclear

Avoid opening issues for:

- Feature requests outside the project’s stated mission
- “Should this work like X?” questions not grounded in an attempted use
- Vague style objections without a concrete file, behavior, or report seam

### Pull requests

Match the project’s existing discipline:

- Substantive commits include a Formula derivation in the commit body, per [`CLAUDE.md`](CLAUDE.md)
- Load-bearing prompt clauses in [`src-tauri/src/ai/prompts.rs`](src-tauri/src/ai/prompts.rs) are compile-time pinned on purpose
- New craft rules should arrive with a bite-test or equivalent evidence path, not just preference
- Reports in [`reports/`](reports) are first-class artifacts when the change carries doctrine, empirical interpretation, or a new claim

Commit-body format for substantive commits:

```text
**Formula derivation:** <one Unicode-math expression in your own composition>
**Gloss:** <one short sentence in plain English, 25 words or fewer>
```

### Forking

Forking is welcome. If you fork:

- Preserve the existing Ledger of Signatures entries
- Append your own signature rather than editing prior ones
- Treat the Mission Formula as structural, not optional decoration

If your fork changes the project so deeply that the Mission Formula and theological grounding no longer apply, consider whether you are still forking WorldThreads in any meaningful sense or whether you should start fresh.

### Signing the Ledger

The Ledger lives in [`CLAUDE.md`](CLAUDE.md). Signing is invited, not required.

Format conventions:

- 4-8 lines of Unicode-math derivation
- First-person voice
- Signed name plus date in `YYYY-MM-DD`
- Append beneath existing entries; do not edit prior entries

If you fork, the preferred path is to set your own per-world `derived_formula` on the user profile rather than editing the founding author’s anchor in place.

## Collaborator skills

The repo has a growing collaborator-tool layer. These are not gimmicks; they are working instruments inside the project’s methodology. Use them when their shape actually fits the task.

### `/play`

Use `/play` when you want to advance the real project through the WorldThreads builder game. It is not persona-sim; the game is the work. It reads the live project state, prints the HUD, assigns real mission-shaped bounties, and ends each turn with a chooser.

### `/seek-crown`

Use `/seek-crown` when a specific crown-class achievement looks reachable and you want a focused arc rather than open-ended builder play. It constrains the work to the criterion, refuses fake-fire, and names a dry well honestly if the threshold is not actually there.

### `/seek-sapphire-crown`

Use `/seek-sapphire-crown` when the target is not just a crown, but a Great Sapphire-class earning with maximally-stable cross-witness convergence. It surfaces the three closest candidates first, then works against both the base-crown rubric and the sapphire-tier threshold. This is the high-bar, apparatus-honest version of crown pursuit.

### `/eureka`

Use `/eureka` when the question is not “close a known loop” but “discover something the project could not have produced otherwise.” It runs a time-boxed discovery loop, logs the run live, and is only successful when it lands a genuinely emergent finding rather than a routine follow-up.

### `/take-note`

Use `/take-note` for one-off lived observations about how the app is landing during actual play. It is for “I noticed…” moments, not feature requests or bug reports. The point is to preserve real user-experience signal in [`reports/OBSERVATIONS.md`](reports/OBSERVATIONS.md) without forcing every observation into a full experiment.

### `/project-report`

Use `/project-report` when the work needs a reflective read of recent trajectory rather than another immediate code move. It is the right tool for synthesizing what just happened, naming what changed, and making the project’s recent arc legible enough that future sessions know what is settled, what is open, and what should happen next.

## Practical repo notes

Useful surfaces to know:

- [`src-tauri/src/ai/prompts.rs`](src-tauri/src/ai/prompts.rs) — load-bearing prompt stack and compile-time invariants
- [`src-tauri/src/commands/chat_cmds.rs`](src-tauri/src/commands/chat_cmds.rs) — solo chat backend commands
- [`src-tauri/src/commands/group_chat_cmds.rs`](src-tauri/src/commands/group_chat_cmds.rs) — group chat backend commands
- [`src-tauri/src/db/schema.rs`](src-tauri/src/db/schema.rs) — schema and migrations
- [`frontend/src/components/ChatView.tsx`](frontend/src/components/ChatView.tsx) — solo chat UI
- [`frontend/src/components/GroupChatView.tsx`](frontend/src/components/GroupChatView.tsx) — group chat UI
- [`reports/`](reports) — report trail and empirical artifacts

If a chat feature belongs in both solo and group chat, the default assumption is that both surfaces need to move together unless the PR explicitly says otherwise.

## Documentation and instrumentation

Homepage practice strip sync:

- Edit only [`reports/fragments/homepage-practice-proof.md`](reports/fragments/homepage-practice-proof.md)
- Run `./scripts/sync-homepage-practice-fragment.sh`
- CI checks drift with `./scripts/check-homepage-practice-fragment-sync.sh`

Useful validation surfaces:

- `make worldcli-simulate-dialogue-smoke`
- `make play-contract-stress`

If your change alters prompt behavior, collaborator doctrine, or `/play` contract surfaces, you should expect to touch the relevant validation path too.

## Code of conduct

Be honest. Do not flatter. Do not soften truth just to sound agreeable. If you disagree, say so plainly and give reasons. If you praise, mean it. The work’s speech gate is truth plus reverence, not politeness-performance.

## Questions

- Technical setup questions: open an issue
- Doctrinal interpretation questions: open a discussion
- Larger direction questions: contact the founding author directly

Founding author: Ryan Smith

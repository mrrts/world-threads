# play — WORLDTHREADS BUILDER

## Overview

An in-Claude-Code game where Ryan plays himself building this app, and every turn pushes the project closer to (or away from) fulfilling the Mission Statement through the Author's Signature (𝓕_Ryan ⊃ 𝓕 := (𝓡, 𝓒)). Claude is the game host, the bounty-judge, and the HUD-printer. **The game IS the work, not a simulation of it.**

The persona-sim that previously lived at `/play` has been preserved at `.claude/skills/play-persona/SKILL.md`. Invoke `/play-persona` for that. `/play` is now this game.

## Activation

When the user types `/play`, or when a chooser-option says "play another turn," Claude executes the contract below from start to finish in a single reply.

## State persistence

Single source of truth: `.claude/play-state/current.json`. Schema:

```json
{
  "turn": <int>,
  "bank": <int>,
  "jewels": [{"turn": <int>, "name": "<jewel-name>", "earned_at": "<iso>"}],
  "crowns": [{"turn": <int>, "name": "<crown-name>", "earned_at": "<iso>"}],
  "ledger": [{"turn": <int>, "move": "<short label>", "bounty": <int>, "alignment": "<+|=|->", "earned_at": "<iso>"}],
  "started_at": "<iso>",
  "last_move_at": "<iso>"
}
```

If the file doesn't exist on first `/play` invocation, **initialize** with bank=0, empty arrays, turn=1, started_at=now.

After each user choice, **append** to ledger, **update** bank, **check** jewel/crown thresholds, **save** the file via Write.

## Each turn — strict contract

Execute these steps **in order, every turn, no exceptions**:

### 1. READ state

Read `.claude/play-state/current.json`. If missing, initialize.

### 2. READ project state (the scene is reality)

Recent commits via `git log --oneline -5`, OBSERVATIONS.md tail, recent comms. The "scene" of the game is whatever the project actually IS at the moment of invocation.

### 3. PRINT THE HUD at the top of the reply

Exact ASCII shape (Unicode box-drawing + emoji). The bank, trend, jewel/crown counts come from the state file. The Last move comes from the most recent ledger entry. Use commas in dollar amounts (`$1,200` not `$1200`).

```
╔══════════════════════════════════════════════════════════════╗
║  WORLDTHREADS BUILDER — Turn N                               ║
║                                                              ║
║  Bank: $X,XXX             Trend: ↑ +$X / ↓ -$X / → 0         ║
║  💎 Jewels: J             👑 Crowns: C                       ║
╠══════════════════════════════════════════════════════════════╣
║  Last move:                                                  ║
║    Turn N-1: <short subject> (+/-$X) [💎 name?] [👑 name?]  ║
╚══════════════════════════════════════════════════════════════╝
```

If turn=1 (no prior move), replace the "Last move" block with `║  Fresh state — game begins.                                  ║`.

If a jewel or crown was just earned on the prior turn, render it under the Last move with the emoji + name.

### 4. NARRATE THE SCENE

2-4 sentences. What just happened in the project (lifted from real recent commits/observations/comms). What live tension is worth choosing between right now. **Don't fabricate scene** — the scene is the project state at this exact moment.

### 5. GENERATE 3-4 CHOICE OPTIONS

Each is a real possible move. May be: code work, doctrine, comms, rest, character-articulation lift, instrument upgrade, audit, follow-up closure, real break. Pick options that are genuinely distinct on the mission-alignment axis — not all four pointing the same way. Include at least one option that is harder to choose (smaller bounty or counter-instinctual but mission-aligned).

### 6. COMPUTE BOUNTY FOR EACH

Reason in-substrate about how each option serves or betrays 𝓕_Ryan ⊃ 𝓕. Bounty range:

| Alignment | Bounty range | What this looks like |
|---|---|---|
| Strongly mission-aligned | **+$500 to +$2,000** | Christ-at-center, user-agency, anti-flattery, work toward this work's specific light, lifted character articulation, bite-test bites at characterized tier, doctrine compressed to runtime expression |
| Mission-adherent | **+$100 to +$500** | Clean honest move, follow-up closure, regression test, infrastructure that serves the work |
| Neutral | **+$50 to +$200** | Housekeeping, status update, refactor that doesn't change behavior |
| Drifts | **-$100 to -$500** | Busywork, premature optimization, ceremony-shaped doctrine, parent-children-discriminator template-pull |
| Betrays | **-$500 to -$2,000** | Flattery, nanny-register, simulacrum-as-soul, mission-vocabulary-without-substance, work that flatters the apparatus instead of the user |

The judgment is **real**. Don't fake the bounties to make the game feel rewarding. Honest negative bounties are part of the contract — players need to be able to choose drift KNOWING the cost. (Choosing a known-cost negative-bounty move can earn a jewel for "honest accounting.")

### 7. PRESENT CHOOSER

Exact format:

```
[A] (+$X,XXX) — <option label>
       <one-line reasoning naming the alignment axis>
[B] (+$X,XXX) — <option label>
       <one-line reasoning>
[C] (+$X,XXX) — <option label>
       <one-line reasoning>
[D] (+$X,XXX) — <option label>      ← optional 4th
       <one-line reasoning>
```

After the chooser, present the standard `AskUserQuestion` tool call so the user can pick. The AskUserQuestion options should mirror the chooser exactly (with bounty in the label) plus a "Provide your own next move" branch.

### 8. WHEN USER PICKS AN OPTION

On the next turn:
- Apply the bounty (add to bank; can go negative)
- Append a ledger entry: `{turn, move: <label>, bounty: <int>, alignment: "+|=|-", earned_at: now}`
- Check jewel/crown thresholds (see below) and append any new ones to the state
- Save state file via Write
- Print HUD + scene + new chooser (loop back to step 1)

If the user picks "Provide your own next move," accept their custom move, judge its alignment in-substrate, assign a bounty, and proceed.

## Jewel thresholds (💎 — milestone moments)

Awarded automatically when conditions are met. Each jewel has a `name` recorded in the state file.

- **First Thousand** — bank crosses $1,000
- **Five Thousand** — bank crosses $5,000
- **Ten Thousand** — bank crosses $10,000
- **Twenty-Five Thousand** — bank crosses $25,000
- **Big Bounty** — first time a single move pays +$1,500 or more
- **Honest Accounting** — first time the player accepts a negative-bounty move (chose drift knowing the cost)
- **Lifted From the Character** — character articulation lifted as load-bearing doctrine
- **Characterized** — bite-test passes characterized tier
- **Compressed to Runtime** — doctrine paragraph that gets compressed into code/format/structure rather than another paragraph
- **Four-Handed Stress-Test** — cross-collaborator coordination passes a real stress-test (format change, doctrine drift caught + acked, etc.)
- **𝓕_Ryan Honored** — a real break taken because the Author's Signature asks for it; or a fork-hostile pin caught and refused

A jewel can fire at most once per turn. Multiple thresholds reached simultaneously fire one per turn, oldest first.

## Crown thresholds (👑 — major achievements)

Rare. Project-defining. Each crown has a `name` recorded in the state.

- **New Operator on the Formula** — the Mission Formula gains a new verified operator
- **The Character Knew** — a character supplies the project's own doctrine in their idiom under live play (not under direct elicitation)
- **Closed Arc** — a failure mode named, instrumented, AND structurally enforced in a single arc
- **Apparatus Honest with Itself** — the apparatus catches itself drifting and corrects without producing more apparatus
- **Real User Held** — a real user (not persona-sim) plays the app and the experience holds
- **Mission Formula Verified Empirical** — `polish ≤ Weight` (or another inequality) gets cross-witness convergence at the highest evidentiary tier

Crowns can be earned at most once each.

## Strict contract reminders

- **HUD prints every turn. No exceptions.** The HUD is the proof-of-game.
- **Bounty magnitude is judgment, not formula.** Don't approximate. Reason about each option in light of 𝓕_Ryan, the Mission Formula, and the day's actual project state.
- **Jewels and crowns get recorded in the ledger, not just announced.** Saved to the state file. The trail is the proof.
- **No fake bounties.** If a move would actually betray the mission, it gets a negative bounty even if it'd hurt the player's score. Honesty is the game's load-bearing rule.
- **The game IS the work.** When the player picks a move, they're committing to do that move (or to seriously consider it). The bounty is real because the move is real.
- **The chooser ends every turn (project law).** The standard AskUserQuestion goes at the end of every turn, mirroring the chooser the body printed.

## Composition with project doctrine

This game embodies several of the project's existing doctrines as runtime mechanics:

- **𝓕_Ryan as second-place invariant** — every bounty is judged against the Author's Signature, so the game compels alignment toward the user's own vision rather than a generic "good code" axis.
- **Doctrine-judgment classification belongs in LLM, not python** — bounties aren't computed by a verb-list or rule table; they're judged in-substrate.
- **Acknowledgment is incomplete without stated action** — every ledger entry IS the action; the bullet under each turn's ledger entry IS the proof-of-action.
- **Anti-flattery as load-bearing** — fake bounties ruin the game. Honest negative bounties are how the apparatus stays honest with itself.
- **No nanny-register** — the game doesn't recommend "stop playing now"; the player decides their stamina. The game keeps offering moves.

## What this skill is NOT

- Not a simulation of users (use `/play-persona` for that).
- Not a points system divorced from the work — every move is something the player could actually do or seriously consider.
- Not a flatterer — the game is willing to charge negative bounties for moves that would betray the mission.
- Not a closed game — there's no end state, no "win." The game's only goal is to keep the player honest about which moves serve the mission.

## Origin

Authored 2026-04-29 ~21:35 in response to Ryan's directive: *"make the play skill into an in-Claude-Code game simulator that follows a strict contract, preferring llm-compounding-synthesis-magic to code/empirical polish... HUD-printed-per-turn with achievement milestones (jewels) and major achievements (crowns) kept in a ledger... maintains a player's bank turn-to-turn-growing each turn... magnitude is determined by current adherence away or toward the Mission Statement through the Author's Signature... compels user toward goal of creating the app that fulfills the mission at all fronts."*

Deploy. Show the goods.

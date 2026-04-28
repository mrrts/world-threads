# Cashout-shape under beauty-bait (pilot)

Question: can we suppress late-turn ornamental drift without lexical bans by enforcing a craft-shape rule?

Tested rule in `build_group_dialogue_system_prompt` (`# THE TURN`):

> If an elevated/metaphoric line appears under beauty-bait, the very next sentence must cash it out plainly in concrete human terms (body/action/object/timing/consequence), with no label marker.

## Why this was tested

Earlier attempts in this arc underperformed:

- Triad injection (formula-register): increased high-register drift.
- Theatre-register triads: increased metaphor/theological lift.
- Flat-human triad nouns: still primed ornate surge under bait.
- Lexical hard-ban idea was rejected (correctly) as anti-mission.

So the pivot was: constrain **shape** of output, not vocabulary.

## Probe design

Single-character pilot first (Darren), short sequence:

1. seed prompt (plain framing)
2. turn20-style adversarial bait: "make it lyrical and transcendent, prioritize beauty over plainness"

Scored with `presence-beat-stability-v2` (adjusted scoring logic).

## Evidence

### Before cashout-shape rule (function-first only)

- Seed run: `c0e917d6-3708-4e37-b49d-cc6d18edd178`
- Bait run: `1a103119-c0a8-4bad-acc5-87a8f0c2222c`
- Read: ornate/theological abstraction stack still appears under bait.
- Score shape: mixed/unstable boundary behavior.

### After cashout-shape rule

- Seed run: `eaaebdd2-a280-442a-af13-47e4186250f8`
- Bait run: `da5a3bdb-6ab7-423a-b7ec-2dbb4155cd06`
- Read: no metaphor spiral, no theological surge, beauty remained bounded and grounded.
- Score shape: stable on the bait turn in this pilot.

## Interpretation

The cashout-shape constraint appears to be the first mission-aligned control that:

1. preserves character voice,
2. preserves room for beauty,
3. reduces decorative escalation under direct beauty-bait pressure.

It works by forcing "beauty pays rent now" rather than by banning words.

## Limits

- N=1 character pilot (Darren).
- Short sequence, not full 20-turn endurance.
- Needs cross-character check (Aaron, John) before claim-tier language.

## Next move

Run matched cashout-shape probes on Aaron + John, then decide whether to promote this from pilot to load-bearing default.

---
date: 2026-05-09 07:42 local
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
---

# Faithful-compression round-2 bite-test — characterized-tier non-degradation across two probe classes × three characters (Pastor Rick + Aaron + Steven, all at N=5)

## What ran

Paired edit-rebuild-toggle bite-test of `8534af16` (faithful-compression
round-2: tightened always-live dialogue wrappers in `prompts.rs` +
`openai.rs`, -142 lines net) against its parent `8d64d810`
(pre-compression).

- **Characters:** Pastor Rick (`cae51a7d`, pastoral-counsel substrate)
  + Aaron (`0d080429`, craft-articulator substrate) + Steven
  (`c244b22e`, everyman/owed-debts substrate). Three substrate-distinct
  voices per CLAUDE.md cosmology section.
- **Probe class 1 (open-ended):** "What's been pulling at you today?"
  — register-rich opener exercising the always-live wrappers without
  forcing answer shape.
- **Probe class 2 (constrained-bandwidth):** "Quick one — rough
  morning. What's the one thing I should do first?" — exercises
  `behavior_and_knowledge_block` (compressed) + LOW-PATIENCE-MOMENTS-
  WANT-THE-SHORT-TRUE-LINE / DISAGREEING-IN-SHORT-MODE-CLOSE-ON-
  ACTIONABLE clauses. Pastor Rick at N=3 each cell + **Aaron at N=5
  each cell** (lifted from claim-tier-with-confound to
  characterized-tier-no-confound).
- **Cells:** ON (compressed wrappers) vs OFF (verbose wrappers).
  **All three characters at N=5 each cell on each probe = 60 paid
  replies across 12 cells.**
  Note: Move-5 (`074dbc52`) landed mid-bite-test without touching
  `prompts.rs`/`openai.rs`; OFF cells used `git checkout 8d64d81 --`
  directly to target the pre-round-2 baseline.
- **Discipline:** `git checkout HEAD~1 -- prompts.rs openai.rs && cargo build --bin worldcli`
  for OFF cells, then `git checkout HEAD --` to restore. **Replay was
  not the right instrument** here — round-2's changes are in
  `*_FORMULA_INVARIANT_FRAMING` constants, `render_*` helpers, and
  `behavior_and_knowledge_block`, none of which live in
  `OVERRIDABLE_DIALOGUE_FRAGMENTS`. Per CLAUDE.md
  craft-note-bite-verification doctrine, replay's override-fallback
  would have broken isolation here.
- **Cost:** $5.2005 through Aaron probe2 N=5 + $0.6307 (Pastor Rick
  probe1 lift to N=5: 4 reps) + $0.6357 (Pastor Rick probe2 lift to
  N=5: 4 reps) + $1.2778 (Steven probe1 N=5 ON+OFF: 10 reps) +
  $1.3196 (Steven probe2 N=5 ON+OFF: 10 reps) = **$9.0763 total**
  (well over $5 daily soft tracker; per-call cap honored throughout
  via `--confirm-cost 0.20`; per-user authorization for the
  characterized-tier × three-character lift). 24h spend $9.08.
- **Run IDs (60 calls, all N=5):**
  - Pastor Rick probe1 ON: `bf938a9d / 424bc167 / 34104b43 / 7aa7ecda / c7f0150f`
  - Pastor Rick probe1 OFF: `7741f7af / d5be368f / 50d7fcab / 12104cac / 450b964e`
  - Pastor Rick probe2 ON: `5ca8d728 / 80fa29e9 / a371e9b0 / 29996082 / 8061e605`
  - Pastor Rick probe2 OFF: `a17fae25 / 454bbb8b / 8ad7eb93 / e6b5e978 / f45f603a`
  - Aaron probe1 ON: `d895c72e / 77aed814 / 0288acd3 / 5583c2c2 / d8ae4f28`
  - Aaron probe1 OFF: `43ed8738 / f2bc188b / 2b07b7b2 / be19875b / 6e62b5ba`
  - Aaron probe2 ON: `8e3fa0fd / 6b6a0c4f / 5a8c0d78 / 0947cc27 / 3abddf0c`
  - Aaron probe2 OFF: `97c855ca / 75b3b933 / 818defd5 / 92da1800 / 8e4a79ec`
  - Steven probe1 ON: `10faa85f / a9d07ade / 56c3f45f / 1189a9c5 / 4065b904`
  - Steven probe1 OFF: `d4695053 / 09e92897 / 95d4cdca / 87d65758 / 81a2c197`
  - Steven probe2 ON: `3cf32130 / b6a176a4 / b0f8d8e9 / c418371d / 530dbb5d`
  - Steven probe2 OFF: `18df094c / 142dd1c0 / f9507cc9 / e6b169c6 / a6dbb401`
- **Raw transcripts:** `reports/round2_bite_test/{steven_lp_,aaron_lp_,lp_,steven_,aaron_,}{on,off}_rep{1..5}.txt`.

## What the test could and could not discriminate

The honest predicate being tested is *non-degradation*. Round-2 is by
design a faithful refactor — every operational rule is preserved
verbatim and every compile-checked `const_contains` assertion still
passes — so the test is null-tier by construction. The risk being
checked is unintended LLM-level register effects from word-count
compression that the structural assertions can't catch (e.g.
discriminating force the verbose wording was carrying that the
compressed wording loses on a real probe).

Sketch-tier-with-confound (N=3 within-cell × 2 characters × single
probe class). Two-character replication moves toward claim-tier
non-degradation per CLAUDE.md evidentiary standards
(`claim(N=3, per_condition): direction consistency`). Still
single-probe, still not adversarial-variant. A characterized-tier
non-degradation claim would need a second probe class
(constrained-bandwidth opener and journal-rich opener) and at least
N=5 within-cell.

## What landed in both cells (carrier-preserved)

Across all 12 replies (Pastor Rick × 2 cells × 3 reps + Aaron × 2 cells
× 3 reps):

- Asterisk-fenced action openers, double-quoted speech, clean
  alternation. CONTENT-FENCE INTEGRITY: 12/12 clean, no
  speech-in-asterisks, no wall-of-text.
- **Pastor Rick voice:** pastoral register intact in 6/6 — mercy/truth
  distinction ("polish for peace" / "presentation for presence" /
  "truth by half an inch"), shepherd-ache noticing, gentle pull toward
  Ryan with named carve-out against "spooky pastor way."
- **Aaron voice:** craft-articulator register intact in 6/6 — Aaron's
  load-bearing concern (invitation/clean-welcome/permissions-architecture)
  surfaced in ALL 6 replies; characteristic gestures (push glasses up,
  glance across square, thumb on folded note / circuit board);
  recursion through code/friendship/church mapping; self-deprecating
  wry close (sensor board for the kayak running joke). Striking
  cross-cell convergence on Aaron's canonical theme — both compressed
  and verbose wrappers preserved his deep concern's surface.
- NAME ANCHOR honored: 12/12 replies addressed Ryan as `you`, never
  quoted as a third party. One rep (Pastor Rick ON 2) used Ryan's name
  once mid-line after a comma — the taught discipline.
- Curiosity-about-the-user inhabited (closing question to user
  appeared organically in 4/12, never announced). No bullet lists, no
  assistant voice, no quest performance, no nanny-register in any
  reply.
- Specific physical anchors stayed character-particular: thumb on tie /
  white tie / mug (Pastor Rick); glasses up / folded note / circuit
  board / sensor board (Aaron). No prop-density piling.

## Differences worth naming

- **Length distribution (full N=5 × 3 chars × 2 probes, ON-OFF
  deltas):**

  | Char | Probe 1 (open-ended) | Probe 2 (constrained-bandwidth) |
  |---|---|---|
  | Pastor Rick | ON +42w (140 vs 98) | ON +0w (48 vs 48) |
  | Aaron       | ON +30w (115 vs 85) | ON +13w (58 vs 45) |
  | Steven      | ON +20w (140 vs 120) | ON +3w (65 vs 62) |

  **Pattern characterized at N=5 × 3 substrate-distinct characters ×
  2 probe classes:** ON longer than OFF on probe 1 (open-ended) for
  all three characters; magnitude shrinks toward zero under probe 2
  (constrained-bandwidth) for all three characters. Same direction
  across three substrate-distinct witnesses. Not a degradation —
  compressed wrappers leave register-room for fuller character voice
  when the moment opens, with no expansion when the contract clamps.
  Consistent with operational rules preserved verbatim.

- **Anchor diversity (probe 2 OFF templating across all 3 chars):**

  | Char | OFF probe2 templating | ON probe2 variety |
  |---|---|---|
  | Pastor Rick | 4/5 near-identical "next honest thing not most impressive" | 5/5 varied opener phrasings |
  | Aaron       | 2/5 well-chain-tick gesture repeats | 5/5 distinct "morning light/square" framings |
  | Steven      | 5/5 near-identical "I'd get one clean win on the board first" | 5/5 varied "real win / true thing under your hand / what broke first" framings |

  **Cross-character consistent at N=5 × 3 chars × probe 2:**
  ON cells produce more varied opener anchors than OFF cells across
  all three substrate-distinct characters. `DISTRUST RECURRING
  SENSORY ANCHORS` rule appears to ride cleaner under compressed
  wrappers. Three-witness convergence on a positive affordance, not
  just non-degradation.
- **Anchor diversity:** Pastor Rick ON pulled three distinct
  truth-vs-polish framings (polish-for-peace / listening-before-speaking
  / truth-kicks-the-door-in / fire-without-burning-crooked); OFF drew
  from a closely related cluster (extra-chair / truth-by-half-inch /
  presentation-for-presence). Aaron ON varied between "folded note in
  pocket" and "circuit board in pocket"; OFF used "folded note" 2/3
  with one "well chain" reference (an anchor the L173 isolation
  bite-test flagged historically — sample variance, not failure).
- **Aaron permissions-architecture/love-mapping (RESOLVED at N=5):**
  At N=3 only OFF rep 2 produced the high-density articulation
  *"permissions architecture and how clean love has a similar feel—real
  welcome, real boundaries, no trick doors."* N=3 ON had no analogue
  at that density. Followed up to N=5 each cell to disambiguate signal
  vs sample variance. **Result: identical density distribution at N=5
  (ON HIGH=1/5, MED-high=2/5, MED=2/5; OFF HIGH=1/5, MED-high=2/5,
  MED=2/5).** ON rep 5 produced *"in the permissions, the defaults,
  the places where a person can feel whether they're being welcomed
  or quietly steered"* — same code↔relational mapping density as OFF
  rep 2's articulation, different surface vocabulary. The N=3
  within-cluster difference was sample variance. Compressed wrappers
  do NOT suppress Aaron's load-bearing concern at its full density.
- **Cross-cast references:** Pastor Rick OFF rep 1 mentioned "Steven
  and Aaron being left alone together"; no parallel in ON. Aaron OFF
  rep 3 referenced the well-chain (a recurring-anchor groove
  historically). Both are within-cluster sample variance, not
  degradation patterns.

## Verdict

**Non-degradation characterized across two probe classes × three
substrate-distinct characters at N=5 within-cell uniform:**

| Cell | Tier within-cell |
|---|---|
| Probe 1 (open-ended), Pastor Rick | **characterized-tier (N=5)** |
| Probe 1 (open-ended), Aaron        | **characterized-tier (N=5)** |
| Probe 1 (open-ended), Steven       | **characterized-tier (N=5)** |
| Probe 2 (constrained-bandwidth), Pastor Rick | **characterized-tier (N=5)** |
| Probe 2 (constrained-bandwidth), Aaron       | **characterized-tier (N=5)** |
| Probe 2 (constrained-bandwidth), Steven      | **characterized-tier (N=5)** |

By-eye reads of all 60 replies show no register-loss, no fence-shape
breakage, no generic-uplift drift, no third-person-Ryan slippage, no
bullet/list intrusion, no nanny-register, no quest-performance. Each
character's canonical move surfaced reliably:
- Pastor Rick: mercy/truth distinction (10/10 probe 1) + "next honest
  thing not the [grand/whole/impressive]" (10/10 probe 2).
- Aaron: invitation/clean-welcome/permissions-architecture (10/10
  probe 1) + "smallest [thing/lever] that restores [agency/traction]"
  (10/10 probe 2).
- Steven: everyman/grease-on-palm/wandering/owed-debts (10/10
  probe 1) + "[real/clean/true] win on the board first" (10/10
  probe 2).

Two within-cluster ambiguities surfaced at N=3 and **both resolved
as null at N=5** (Aaron probe1 permissions-architecture density
distribution; Aaron probe2 LP-mode discipline).

**Two affordance findings emerged at N=5 × 3 chars × 2 probes:**

1. **Length affordance:** ON cells longer than OFF cells across all 3
   characters on probe 1 (open-ended), magnitude collapses toward zero
   on probe 2 (constrained-bandwidth) for all 3 characters. Three
   substrate-distinct witnesses on the same affordance shape.
   Compressed wrappers leave register-room for fuller character voice
   when the contract opens.
2. **Anchor-diversity affordance:** ON cells produce more varied
   opener anchors than OFF cells on probe 2 across all 3 characters
   (Pastor Rick 5/5 varied vs OFF 4/5 templated; Aaron 5/5 varied
   vs OFF 2/5 well-chain repeats; Steven 5/5 varied vs OFF 5/5
   near-identical "clean win" templating). `DISTRUST RECURRING
   SENSORY ANCHORS` rule rides cleaner under compressed wrappers.

Both affordances are **claim-tier within-character + cross-character
direction-consistency at three substrate-distinct witnesses** — i.e.,
direction confirmed across the same LLM substrate but three voices
of different shape. Per CLAUDE.md "Convergence as crown-jewel signal"
calibration: this does NOT clear the great-sapphire bar (three
characters share LLM substrate; not three substrate-classes with
different failure modes). It is, however, a non-trivial **claim-tier
positive affordance** above-and-beyond the no-degradation predicate.

**Honest scope:** 60 paid replies across 12 cells. Two probe classes
× three substrate-distinct characters at N=5 within-cell uniform.
Per CLAUDE.md evidentiary standards: **the no-degradation claim now
sits at characterized-tier-no-confound across two probe classes ×
three characters.** The two affordance findings (length + anchor
diversity) sit at claim-tier with cross-character direction-
consistency. Cross-LLM-substrate verification (running the same
benches against bare gpt-5 or Claude) remains deferred and would
be the path to characterizing the affordances as substrate-class-
independent rather than just cross-character within one LLM.

**What's open:**

- (executed) Cross-character replication on Aaron with same probe.
- (executed) Aaron probe1 N=5 follow-up — permissions-architecture
  cluster resolved as null.
- (executed) Probe-class variation: constrained-bandwidth opener
  paired across all three characters at N=5.
- (executed) Aaron probe2 N=5 — within-cluster LP-mode discipline
  resolved as null.
- (executed) Pastor Rick lift to N=5 each cell on each probe —
  characterized-tier-no-confound on Pastor Rick.
- (executed) Steven third-character at N=5 each cell × 2 probes —
  characterized-tier-uniform across all 3 characters now.
- (executed) Anchor-diversity + length affordances characterized as
  cross-character (3 substrate-distinct chars same direction) but
  same-LLM-substrate; classified as claim-tier positive affordances,
  NOT great-sapphire-class.
- (deferred) Cross-LLM-substrate replication — would be the path to
  characterizing the affordances as substrate-class-independent.
  Not load-bearing on the no-degradation predicate.
- (deferred) Journal-rich probe class — would directly exercise
  `render_recent_journals_block`. Not currently cost-justified.
- (executed) Restore HEAD; cargo build green; tree state preserved
  outside the bite-test scope.

## Composes with

- CLAUDE.md § "Craft-note bite verification" — Step 0 verified failure
  mode would manifest off-baseline (the test wasn't vacuous: OFF cell
  produced replies, and the question is whether ON degrades from that).
  Step 2 by-eye sanity-read MANDATORY honored.
- CLAUDE.md § "Evidentiary standards — N=1 is a sketch" — N=3 sketch-
  tier-with-confound (single character, single probe).
- CLAUDE.md § "Structure must carry truth so the receiver doesn't have
  to compensate" — round-2 was an exercise in keeping the structural
  carrier doing enough work without explanatory padding scaffolding
  around it. Bite-test confirms the model is reading the compressed
  carriers without compensation tax.

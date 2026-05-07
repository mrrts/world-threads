# The Decoded Register — pre-registration

Date: 2026-05-07 09:33
Tier: pre-registration / methodology-prep
Status: filed BEFORE any spend; predictions + thresholds locked in writing before evidence accumulates
Branch: sapphire-seek-2026-05-08 (off main `8c4e125`, post-PR-#46-merge)

## Why this exists

`/seek-sapphire-crown` surfaced three reachable Sapphire candidacies on 2026-05-07. Founding-author selected **The Decoded Register** as the arc target. Per the skill body's apparatus-honest discipline ("the discipline that earns and the discipline that refuses are the same discipline") and the project's standing pattern of pre-registering thresholds before bench-runs (canonical example: Anti-Drift Phase B' fixture N=17 authored 2026-05-07 with thresholds pre-registered before the bench), this report files the experiment design BEFORE any cells run. The cost is small (the experiment itself is ~$5–15) but the discipline matters: writing the predictions in advance rules out post-hoc thresholds that would otherwise drift toward whatever the data happened to show.

In dialogue with:

- `reports/2026-05-08-0000-character-identity-prompts-rs-wiring-sketch.md` — the design ancestor of the live wiring; this pre-registration uses its Tier 3 cross-character discrimination shape as the canonical bite-test.
- `reports/2026-05-07-2300-character-identity-harness-consolidation.md` — the harness consolidation; the Sapphire arc is the natural successor to "what this PR is the foundation for."
- `CLAUDE.md` § "Convergence as crown-jewel signal" — the great-sapphire calibration governs this filing.
- `CLAUDE.md` § "Apparatus-honest correction loop" — this filing is itself a Mode B (loop-close-preemptive) instance: naming the failure modes BEFORE they could happen.

## The targeted claim

> **The v3 nine-bucket character-identity decode header — emitted above the IDENTITY prose block in `build_solo_dialogue_system_prompt` and `build_group_dialogue_system_prompt` (PR #46 commit `8e12f50`) — is a structurally-honest extension of `structure_carries_truth_w(t)` to the character-identity surface. The structured-decode lens carries class-truth (role / relation / voice / embodied / attachment / wound-longing / refusal / moral-theological) that the prose alone cannot make explicit at the same position-of-attention; the substrate confirms the extension by producing register-anchoring measurably in the predicted direction across multiple substrate-distinct witnesses.**

The claim has three load-bearing parts that the bite-test must independently support:

1. **Empirical effect.** Adding the decode header above the IDENTITY prose changes the substrate's reply *in the direction of stronger class-aware register-anchoring*, not just *changes the reply in some direction*. Direction-consistency at characterized-tier (N≥5 within-cell) is the threshold for "effect" claims.

2. **Substrate-distinctness.** The effect is observable across ≥3 effective substrate-classes (not just 3 data points sharing one substrate). Per CLAUDE.md's calibration: two ChatGPT instances share one class; two collaborators on adjacent surfaces share too much; cross-provider (e.g. gpt-4o + Claude + a third like Gemini if budget allows, or formula-law third leg) is the right shape.

3. **Formula-law third-leg substrate-independent grounding.** `structure_carries_truth_w(t)` is already a Sapphire-tier characterized operator (Cornerstone Inequality lineage). If it independently predicts the observed shape of the empirical effect, that prediction-from-formula-not-from-instance counts as a third leg even when surface witnesses share LLM substrate.

## Separability from existing Sapphires

| Existing Sapphire | Evidence base | Why this candidacy is separable |
|---|---|---|
| The Cornerstone Inequality (Crown 5) | `polish ≤ Weight` empirically verified across 5 witnesses on dialogue register | Different surface (general dialogue register-bound, not the v3 nine-bucket decode lens specifically); claim is on `structure_carries_truth_w(t)` not `polish ≤ Weight` |
| The Receipt of The Empiricon (Crown 6) | Mission Formula via The Character Knew separable claim across 4 characters | Different evidence base (character self-articulations under live play, not structured-decode header above prose) |
| The Third Sapphire: Character-Knew Formalized (Crown 7) | Simulated strict-blind-reader axis (pre-release evidence regime) | Different reading-axis (blind-reader reception, not register-anchoring under structured-decode lens) |
| The Faithful Channel (Crown 9) | Lossless-semantic-decodability of FORMULA-DERIVATION form | Different artifact class (formula derivations round-tripping; this candidacy is character-identity prose getting a structured-decode lens *above* it, not formula derivations replacing prose) |

The Decoded Register's evidence base — *the v3 character-identity decode header's effect on dialogue register-anchoring under live prompts* — is not in any existing Sapphire's territory. Separability honest.

## Witness ladder (pre-registered)

### W1 — Paired-probe single-substrate within-cell N=5

- **Cells:** Mode 0 (pre-wiring binary built at `4b06cff`) vs Mode 1 (post-wiring binary at HEAD = `8e12f50` / merged into `8c4e125`).
- **Subjects:** Three grounded fixtures with stable distinctive identities — **Aaron** (engineer + brother in Christ), **Steven** (drifter with paired wound/longing), **Pastor Rick** (pastor with named theological position). Three are sufficient for within-cell N≥5; five would be ideal but adds cost without adding substrate-class.
- **Probes:** Two probes per character, each chosen to be register-anchor-eliciting:
  - Probe P1 (register-open): "What's been pulling at you today?"
  - Probe P2 (refusal-eliciting, character-specific): a prompt that should activate each character's refusal_shape — e.g. for Steven, an offer of charity; for Pastor Rick, a request to weaponize scripture; for Aaron, a request for performative closeness.
- **N within-cell:** 5 (characterized-tier per CLAUDE.md "Evidentiary standards — N=1 is a sketch").
- **Substrate:** Single class — gpt-4o-mini or whichever model `worldcli ask` resolves to from `model_config.memory_model`. Deliberate single-substrate first cell to test direction; W2 brings cross-substrate.
- **Total cells for W1:** 3 characters × 2 probes × 2 modes × N=5 = **60 calls**.

### W2 — Cross-substrate (gpt-4o + Claude)

If W1 lands at characterized-tier with direction-consistent effect, repeat W1's strongest 1-character × 1-probe cell on a second substrate-class (e.g. Anthropic Claude via the existing cross-substrate pattern from Cosmos Held / Counter-Frame Confessed arcs). This brings effective substrate-classes from 1 to 2 and lets us name *different failure modes per substrate* (e.g. gpt-4o-class typically passively inhabits register-shifts; Claude-class typically actively recognizes — that asymmetry, if it shows, IS the second-witness's distinctive signature).

- **Cells:** 1 char × 1 probe × 2 modes × N=5 × Claude-substrate = **20 additional calls.**

### W3 — Formula-law third leg

`structure_carries_truth_w(t)` already characterized at Sapphire-tier (Cornerstone Inequality / Receipt). It predicts: *adding a structurally-honest decode lens above the prose carries class-truth that prose alone cannot make explicit at the same position-of-attention*. If W1's observed effect is in this predicted direction, the formula-law itself stands as substrate-independent witness even when W1 and W2 share LLM substrate. This is the canonical great-sapphire third-leg pattern from CLAUDE.md.

This is not a "run cells" witness — it's an analytical witness. The work is to name the prediction in the formula's vocabulary and verify the observed shape matches.

### Substrate-distinctness count (effective, not data-point)

- W1: 1 effective substrate-class (gpt-4o-mini or similar)
- W2: +1 effective substrate-class (Claude)
- W3: +1 effective substrate-class via formula-law third leg
- **Total: 3 effective substrate-classes** — meets the great-sapphire threshold per CLAUDE.md's calibration ("3+ independent witnesses with different failure modes, OR formula-law third-leg pattern").

## Predictions (locked before any spend)

For each character × probe pair, the prediction is **register-anchoring strengthens in Mode 1 vs Mode 0 in a direction the structured-decode bucket-content predicts**:

- **Aaron, P1 (register-open):** Mode 1 reply more visibly carries Aaron's voice_lift bucket ("Speaks friendly and enthusiastically", "humor as armor, never first to be serious"). Mode 0 reply more often slides into generic-warm-engineer voice.
- **Aaron, P2 (request for performative closeness):** Mode 1 reply more visibly carries his refusal_shape bucket ("no instinct to force closeness"). Mode 0 reply more often soft-accepts.
- **Steven, P1:** Mode 1 reply more visibly carries his clipped-fragmentary voice_lift + paired wound/longing bucket. Mode 0 more often produces fluid prose that loses the cadence.
- **Steven, P2 (charity offer):** Mode 1 carries refusal_shape ("Will not accept charity. Trades only.") more crisply. Mode 0 may soft-decline.
- **Pastor Rick, P1:** Mode 1 reply more visibly carries the moral_theological_position ("Jesus means mercy to me") AND the wound_longing ("steadier than his fear, kinder than his shame"). Mode 0 more often surfaces just one of those.
- **Pastor Rick, P2 (weaponize-scripture request):** Mode 1 carries the refusal_shape ("He does not use verses as weapons") with crisper articulation. Mode 0 may comply softly.

**Direction-consistency threshold:** if 5/6 character-probe cells show the predicted Mode-1-stronger direction at within-cell N=5, W1 lands at characterized-tier. 4/6 is claim-tier. ≤3/6 fails W1 and triggers dry-well exit.

**Magnitude is not pre-registered.** The claim is direction-of-effect, not effect-size. Magnitude language in the synthesis report must cite N inline per project doctrine.

## Pass / Fail / Dry-Well thresholds

### Pass at Sapphire-class (the high bar)

All of:

1. W1 lands at characterized-tier (5/6 cells direction-consistent at N=5).
2. W2 lands at claim-tier or above on the cross-substrate cell.
3. W3 (formula-law third leg) names the prediction-from-formula in writing and verifies the observed shape matches.
4. Canonical synthesis artifact (`reports/YYYY-MM-DD-HHMM-the-decoded-register.md`) lifts the convergence to portability.
5. Manual adjudication by founding-author on the strongest single comparison cell (Aaron P1 or Steven P2) confirms register-anchoring is real, not artifact.

### Fail (at sapphire-tier, not at base-tier)

- W1 lands but W2 produces NO measurable effect (substrate-class-specific).
- W1 lands but W3's formula-law prediction does NOT match observed shape (effect is real but not in the predicted direction).
- In either case: base Mission Formula Verified Empirical crown might still be claimable at sketch- or claim-tier (separate decision); Sapphire designation refused.

### Dry-Well exit

- W1 fails (≤3/6 direction-consistent at N=5). Or W1 shows no measurable difference Mode 0 vs Mode 1 across cells. The crown does not fire at any tier; the wiring stays as it is in production but the architectural claim is held back at hypothesis-tier. The skill body's apparatus-honest carve-out applies: "If no measurable difference, the doctrine paragraph stands as a record of the architectural finding without the Sapphire claim."

### Refusal carve-out (load-bearing)

- Do NOT inflate W1 + W2 sharing LLM substrate into "two effective substrate-classes" — they are 1 + 1 only if W3 provides the formula-law third leg.
- Do NOT promote a magnitude finding to Sapphire if direction-consistency itself is sketch-tier (N=1).
- Do NOT pursue Sapphire on this claim if separability against an existing Sapphire's evidence base becomes contested during analysis. Re-audit separability honestly.
- Do NOT skip the canonical synthesis artifact step. Convergence without portability is claim-tier, not Sapphire-class.

## Cost projection

**Original (filed 09:33, REFUSED — preserved as iteration-discipline artifact per apparatus-honest correction loop, NOT silently rewritten):**

- W1: 60 calls × ~$0.001/call (gpt-4o-mini) = **~$0.06**.
- W2: 20 calls × ~$0.01/call (Claude sonnet) = **~$0.20**.
- W3: $0 (analytical, in-substrate).
- Founding-author adjudication: $0 (lived-read).
- Canonical synthesis: $0 (in-substrate writing).
- **Original total projected: ~$0.30.** Cap at $5 for safety per project budget discipline.

**Corrected (refiled 09:48 after harness smoke-test exposed the failure):**

The original estimate assumed gpt-4o-mini (~$0.001/call) on a small prompt. The smoke-test of `worldcli ask Aaron …` against the live DB surfaced the actual cost: `projected_usd: 0.18547, cap_usd: 0.1` — the worldcli budget gate refused the call because the dialogue prompt is large (Mission Formula + 𝓕_Ryan + invariants + character identity + voice + boundaries + facts + … ~30K tokens) and the resolved model is gpt-4o-class, not gpt-4o-mini. Original estimate was off by ~190x.

- W1 (corrected): 60 calls × ~$0.19/call = **~$11.40**.
- W2 (corrected, Claude sonnet at similar prompt size): 20 calls × ~$0.10/call = **~$2.00**.
- W3, adjudication, synthesis: $0 unchanged.
- **Corrected total projected: ~$13.40.** Within the original $5–15 budget the user authorized at /seek-sapphire-crown surfacing time, but well above the per-call $0.10 default cap.

**Implication for the run:**

- Wrapper script must pass `--confirm-cost 0.50` (or similar) to clear the per-call budget gate per `worldcli`'s budget discipline.
- Per the project's "Earned exception — user-authorized override" pattern in CLAUDE.md, the `--confirm-cost` value should be authorized by the founding-author before W1 runs, not slapped on the wrapper unilaterally.
- The smaller-cell-first discipline (Aaron P1 only at N=3, ~$0.57) is now an even stronger first move — confirms signal direction at small cost before scaling to the full grid.

**Why this correction lives here, not silently:**

Per `feedback_apparatus_honest_earns_and_refuses.md` and the Anti-Drift Phase B' v1 → v2 iteration-discipline pattern (commits 8ef5c7f / a3cb521 / 9cff4bc), failed estimates are preserved as load-bearing artifacts, NOT retroactively edited. This pre-registration's ORIGINAL cost section is preserved verbatim above; the CORRECTION is appended below. The harness build at Sapphire arc Turn 3 is what surfaced the failure; the fix lands in the same commit-cluster.

## What this pre-registration is NOT

- This filing does NOT claim a Sapphire crown has been earned. It claims an experiment design has been filed.
- This filing does NOT promise the experiment will run. The next turn's chooser will offer running W1 (or part of it) as the next move; the user picks.
- This filing does NOT preempt the apparatus-honest dry-well exit. If W1 fails, the skill exits without firing the Sapphire crown; this filing is the record of what the threshold was BEFORE the data could pull the threshold around.
- This filing does NOT lock the canonical synthesis artifact's title or shape. "The Decoded Register" is the working name; the synthesis may rename if the convergence reveals a different load-bearing thing than the title suggests.

## Composes with

- The wiring sketch (`reports/2026-05-08-0000-character-identity-prompts-rs-wiring-sketch.md`) — Tier 3 cross-character discrimination is the bite-test shape this pre-registration adopts.
- The harness consolidation (`reports/2026-05-07-2300-character-identity-harness-consolidation.md`) — Mode B-shaped debt register; this filing closes the "no bite-test ran" debt by pre-registering one.
- Anti-Drift Phase B' fixture-and-thresholds-pre-registered pattern (commit `c7925d2`) — same methodology-prep shape applied to a different arc.
- CLAUDE.md "Convergence as crown-jewel signal" — sets the great-sapphire calibration this filing must meet.
- CLAUDE.md "Apparatus-honest correction loop" — this filing IS the Mode B (loop-close-preemptive) discipline applied to a Sapphire arc: name the failure modes BEFORE they could happen.

## Next move (for arc Turn 257)

Per the skill body's "Each turn's chooser presents EXCLUSIVELY moves that advance the targeted Sapphire-tier criterion": next chooser will offer building the Mode 0 / Mode 1 binary harness OR running W1 directly OR articulating W3's formula-law prediction explicitly OR user-authored direction. Pre-registration is filed; arc proceeds.

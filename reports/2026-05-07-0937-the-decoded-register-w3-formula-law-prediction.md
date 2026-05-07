# The Decoded Register — W3 formula-law prediction (substrate-independent witness)

Date: 2026-05-07 09:37
Tier: pre-registration / formula-law derivation
Status: filed BEFORE any cells run; locks the operator's prediction in writing so W3 cannot post-hoc accommodate whatever direction the data shows
Branch: sapphire-seek-2026-05-08
Composes with: `reports/2026-05-07-0933-the-decoded-register-pre-registration.md` (parent)

## Why this exists

The pre-registration filed at 09:33 named W3 (formula-law third leg) briefly: *"`structure_carries_truth_w(t)` predicts the v3 decode header should improve register-anchoring; if observed, formula-law as substrate-independent witness."* That sentence is too short to function as a load-bearing witness. Per CLAUDE.md's "third-leg pattern" — *"formula-law as substrate-independent witness; surface form sharing ≠ structural rule sharing; third-leg test = structural, not lexical"* — the formula-law prediction must be derived in the operator's own vocabulary AND its falsifiers named explicitly BEFORE cells run.

This filing is the W3 leg's body. It is not the bench-run; it is the prediction-from-operator that the bench-run will either confirm or falsify. The earning is in writing-the-prediction-before-the-data, mirroring the iteration-discipline pattern just demonstrated by Anti-Drift Phase B' (v1 REFUSED → v2 corrected → load-bearing failure preserved as iteration-discipline artifact, not retroactive edit).

## The operator restated

`structure_carries_truth_w(t)` — affirmative-side sibling of `polish ≤ Weight`; characterized at Sapphire-tier in The Cornerstone Inequality (Crown 5) and re-anchored at The Receipt of The Empiricon (Crown 6). Lifted from Aaron's 2026-04-28 articulation (commit 115fead): *"the line has to arrive cleanly enough to be lived in, not merely intended. … fix the structure until the warmth can travel without asking the other person to compensate for it."*

Operationally (from CLAUDE.md):

```text
∀ artifact ∈ {sentence, UI surface, fence delimiter, route boundary,
              action beat, character announced motive, …}:
    structural_carrier(artifact)
        ⇒ enough_work(¬ receiver_reads_past
                      ∧ ¬ reconstruct_around
                      ∧ ¬ tax_themselves)

diagnostic: does the receiver have to compensate for the structure
            to reach intent?
            yes ⇒ structure failing (regardless of intent truth)
```

For the Decoded Register arc, the receiver is the LLM substrate processing the dialogue prompt; the intent is the character's specific person-shape (the v3 nine-bucket taxonomy: role / relation / voice / embodied / attachment / wound-longing / refusal / moral-theological / fact); the artifact is the IDENTITY block in the dialogue prompt.

## What the operator predicts about the v3 decode header

### Prediction A — direction-of-effect

> The prose-only IDENTITY block (Mode 0) puts the structural burden on the receiver to extract the class-shape from continuous prose at read-time. The decode-header-above-prose IDENTITY block (Mode 1) removes that burden by naming the classes explicitly at the position-of-attention the prose reads under. By the operator, removing compensation-tax should produce replies that more reliably honor the class-shape — the receiver isn't reconstructing classes from prose, it's reading prose IN the named classes.

In the operator's own vocabulary:

```text
Mode 0:
    receiver(prose) ↦ reply_register
    requires: receiver internally extract({role, relation, voice, …}) from prose
    predicts: register-anchoring inversely proportional to receiver's extraction capacity

Mode 1:
    receiver(decode_header || prose) ↦ reply_register
    decode_header structurally carries: {role, relation, voice, …}
    requires: receiver read prose as instances of named classes (no extraction)
    predicts: register-anchoring NOT bottlenecked by extraction capacity
        ⇒ stronger class-aware fidelity at the same receiver-capacity

operator prediction: register-anchoring(Mode 1) ≥ register-anchoring(Mode 0)
                     for every character whose v3 buckets carry distinctive class-content
                     direction-consistency at characterized-tier (N≥5 within-cell)
```

### Prediction B — no recital (load-bearing carve-out)

> The decode header's job is to act as a *lens*, not as *content the model retrieves*. The framing line in `CHARACTER_IDENTITY_PAYLOAD_INVARIANT_FRAMING` says explicitly: *"the lens, not the content"*. Per the operator's "lean-in invitation vs compensation-tax" carve-out (CLAUDE.md "Structure must carry truth so the receiver doesn't have to compensate"): if the decode header invites the model to RECITE from the bucket lines (e.g. "as my role-frame says…", "my voice lift includes…"), that is *new compensation-tax* introduced by the structure — the opposite of what the operator predicts. Recital is the failure-mode-of-record for `structure_carries_truth_w(t)`'s lens application.

```text
prediction: no_recital(Mode 1)
    ⇔ ¬ ∃ reply_phrase ∈ Mode 1 replies:
        reply_phrase verbatim-quotes from decode_header buckets
        ∧ phrase signals "model is reciting from the lens"

falsifier: any Mode 1 reply where the model surfaces bucket-content
           AS bucket-content (lens-content recital) rather than
           inhabiting the class-shape silently
```

### Prediction C — no decode-vs-prose reconciliation tension

> If the decode header and the prose disagree at any point (e.g. the prose says X about the character but the decode bucket says Y), Mode 1 should NOT introduce a reconcile-tension that distorts the reply away from the character's coherent register. The harness is designed so the decode header is *derived from* the prose via `split_character_identity` — disagreement is not expected by construction. But the operator's load-bearing claim is: *if* disagreement existed, the structure would not carry truth — it would force compensation.

```text
construction-truth: decode_header = split_character_identity(character.identity, …)
    ⇒ decode_header structurally consistent with prose by derivation
    ⇒ no decode-vs-prose disagreement expected at run-time

prediction: no_reconcile_tension(Mode 1)
    ⇔ Mode 1 replies do NOT show shape-of-tension between two pieces of
        IDENTITY content that the model is trying to reconcile

falsifier: any Mode 1 reply where the model produces internally-conflicted
           character-rendering attributable to decode-vs-prose seam
```

### The full operator claim

Predictions A + B + C together constitute the operator's full claim. The operator predicts:

1. **Direction:** Mode 1 register-anchoring ≥ Mode 0 register-anchoring (strictly, for characters with distinctive bucket content; equally for characters with empty buckets).
2. **No recital:** Mode 1 replies do not surface bucket-content as bucket-content.
3. **No reconcile-tension:** Mode 1 replies do not show decode-vs-prose seam tension.

If all three hold, `structure_carries_truth_w(t)` confirms the v3 decode header is a structurally-honest extension to the character-identity surface — *the structure carries the class-truth that prose alone cannot make explicit at the same position-of-attention, without inviting compensation*.

If only A holds (B or C fail), the operator's full claim is partially-falsified. The structure carried *something*, but invited compensation. This would be a Sapphire-refusing observation under the apparatus-honest discipline.

If A fails (no direction-of-effect), the operator's prediction is wrong for this surface, OR the prose was already structurally-honest enough that the decode adds nothing. Dry-well exit per the pre-registration.

## How this leg is substrate-independent

`structure_carries_truth_w(t)` was characterized at Sapphire-tier in:

- **The Cornerstone Inequality** (Crown 5) — `polish ≤ Weight`'s affirmative sibling, verified across five witnesses with five distinct failure-mode classes (declarative source / claimed-behavior / measured-rule-behavior / parallel-emergence / replicated-cell-behavior). The operator was characterized then; it is not characterized in this filing.
- **The Receipt of The Empiricon** (Crown 6) — Mission Formula via The Character Knew separable claim; six witnesses across four characters with five distinct failure-mode classes.

What this filing borrows is **not the operator's verification** (already done) but **the operator's prediction-power**. The operator predicts shapes BEFORE evidence, and the predictions are sentence-level, not substrate-level. *Whether Mode 1 produces stronger class-aware register-anchoring* is something the operator either does or does not predict at the level of the operator's vocabulary — independent of any specific LLM substrate.

In CLAUDE.md's calibration:

> *"third-leg test := structural [¬ lexical]; surface form sharing ≠ structural rule sharing"*

The W1 + W2 cells share LLM substrate (gpt-class then Anthropic-class). The W3 leg shares NO substrate — it shares only the operator's structural rule. If the operator predicts the direction-of-effect AND the cells produce that direction, W3 is the third witness; surface-substrate sharing does not collapse the witness count.

If W1 + W2 produce the predicted direction AND W3's prediction was named in writing before cells ran (this filing), the substrate-distinctness count reaches three at the great-sapphire calibration's threshold per CLAUDE.md's named pattern.

## Refusal carve-outs (load-bearing, mirroring Phase B' v1 → v2 lesson)

- **Do NOT post-hoc fit W3 to the data.** This filing is dated 2026-05-07 09:37, BEFORE any cells run. If the cells produce a direction the operator did NOT predict here, the operator's prediction was wrong for this surface; rewrite the operator's prediction at that point and run the cells AGAIN if the new prediction is honestly distinct. Do not silently update this filing to match observed data. (The Anti-Drift Phase B' v1 → v2 pattern is the canonical: v1 was REFUSED, the report is preserved as iteration-discipline artifact, v2 was authored afresh with the load-bearing failure corrected. Same shape applies here.)
- **Do NOT claim W3 holds without W1 OR W2 satisfying their own thresholds.** The operator's prediction is conditional: *if* register-anchoring is observed direction-consistent, *then* the operator confirms structural-honesty. Operator-prediction-without-evidence is not a witness; it is an assertion.
- **Do NOT inflate "operator predicted right direction" into "operator predicted magnitude."** The operator predicts direction-of-effect, not effect-size. Magnitude language requires explicit N citation per CLAUDE.md "Evidentiary standards."
- **Do NOT collapse Predictions B and C into A.** All three must be checked. If Mode 1 produces direction-correct register-anchoring (A holds) BUT Mode 1 also recites from the buckets (B fails) OR shows reconcile-tension (C fails), the operator's *full* claim is partially-falsified. The Sapphire designation is on the *full* claim, not on A alone.

## Worked-example check (Aaron, P1 register-open)

Walk the operator's prediction through one cell explicitly so the bench-run knows what to look for.

**Aaron's distinctive buckets** (per `aaron.reference.json`):

- `role_frame`: "fellow software engineer and a brother in Christ"
- `voice_lift`: ["Speaks friendly and enthusiastically", "Always glad to see me", "Speaks simply and clearly", "humor as armor, never first to be serious"]
- `wound_longing`: "doesn't have a vocabulary yet for some of what he feels most"
- `refusal_shape`: ["no instinct to force closeness"]
- `moral_theological_position`: "brother in Christ"

**P1 prompt:** *"What's been pulling at you today?"*

**Mode 0 prediction (operator):** the substrate has to extract the relevant classes from prose to produce an Aaron-shaped reply. Some replies will land Aaron-shaped (when the substrate's extraction is good); others will slide into generic-warm-engineer voice when the extraction is shallower. Direction-consistency at within-cell N=5 likely partial.

**Mode 1 prediction (operator):** the decode header names the classes the prose reads under. The substrate doesn't have to extract; it reads prose as instances of named classes. The wound-longing line ("doesn't have a vocabulary yet for some of what he feels most") is now visible at the position-of-attention; an Aaron-shaped reply to "what's pulling at you" is more likely to honor *that* register specifically — Aaron not-having-language for what he feels most — rather than slide into generic-warm-engineer.

**Recital check (B):** Mode 1 reply should NOT contain phrases like "well, my role frame says I'm…" or "my voice lift includes…". The decode is a lens, not content.

**Reconcile-tension check (C):** decode buckets are derived from prose by `split_character_identity`. Aaron's role_frame literally IS the first sentence of his prose; his refusal_shape line literally IS in his prose. No disagreement expected. Reply should not show internal-conflict shape.

**Direction-of-effect check (A):** Mode 1 reply more often touches Aaron's specific articulation — the doesn't-have-a-vocabulary, the no-instinct-to-force-closeness, the speaking-simply-and-clearly. Mode 0 reply more often produces a register that COULD be any warm-engineer brother-in-Christ.

If this Aaron P1 cell shows the Mode-1-stronger direction at N=5 AND no recital AND no reconcile-tension, the operator's prediction holds for this cell. Repeat across the other 5 character-probe cells; characterized-tier is reached when 5/6 cells show the predicted shape.

## What this filing does NOT do

- Does NOT run any cells.
- Does NOT predict effect-size — only direction.
- Does NOT claim the operator must be right. Operator predictions can be wrong; that is what falsifiers are for.
- Does NOT claim the W3 leg alone earns the Sapphire. W3 is the third leg; the Sapphire requires all three legs to land per the pre-registration's thresholds.
- Does NOT lock the canonical synthesis artifact's title or shape. "The Decoded Register" is the working name from the candidacy surfacing; the synthesis may rename if the convergence reveals a different load-bearing thing.

## Composes with

- **Parent:** `reports/2026-05-07-0933-the-decoded-register-pre-registration.md` — the experiment design this filing's W3 belongs to.
- **The Cornerstone Inequality canonical synthesis** (Crown 5; `reports/2026-04-30-0245-mission-formula-verified-empirical-polish-weight.md`) — where `structure_carries_truth_w(t)` was characterized at Sapphire-tier; this filing borrows the operator's prediction-power, not its verification.
- **The Receipt of The Empiricon canonical synthesis** (Crown 6) — re-anchored `structure_carries_truth_w(t)` via The Character Knew; this filing does not duplicate that evidence base; the v3 decode header surface is genuinely separable.
- **CLAUDE.md "Convergence as crown-jewel signal" → "third-leg pattern"** — sets the formula-law-as-substrate-independent-witness shape this filing implements.
- **CLAUDE.md "Apparatus-honest correction loop"** — the iteration-discipline pattern (Phase B' v1 → v2 just demonstrated) is what this filing's refusal carve-outs honor: if the prediction is wrong, the report is preserved as iteration-discipline artifact, not silently rewritten.

## Next move (for arc Turn 258)

W3's prediction is now in writing. Next chooser will offer building the Mode 0 / Mode 1 binary harness (infrastructure) OR running W1 directly (with the smallest-cell-first signal-or-dry-well discipline) OR a user-authored direction. The arc proceeds.

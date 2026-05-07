# `compensation_tax_w(t)` — operator formal definition (claim-tier articulation, Sapphire patience-shaped)

Date: 2026-05-07 12:00
Tier: claim-tier articulation lifted to portability; Sapphire candidacy stays at sketch-tier (N=1 work-shape today)
Branch: sapphire-seek-2026-05-08
Parent arc: /seek-sapphire-crown :: compensation_tax_w(t) (New Operator on the Formula candidacy)
Composes with: Conditional Lens operator articulation `2026-05-07-1030`; structure_carries_truth_w(t) Sapphire audit `2026-04-30-0305`; The Cornerstone Inequality canonical synthesis `2026-04-30-0245`

## Why this filing exists

Today's `/seek-sapphire-crown :: The Conditional Lens` arc introduced `compensation_tax(C, T)` as a *predicate inside* the conditional-applicability refinement of `structure_carries_truth_w(t)`. The user authorized lifting it from refinement-parameter to candidate-operator status with formal definition. **This filing is what future arcs cite when they use `compensation_tax_w(t)` in load-bearing reasoning.** Per the `New Operator on the Formula` Sapphire criterion (`N≥3 independent derivation grounding from different work-shapes; sentence-level definition matching formula's operator vocabulary`), this filing locks the sentence-level definition; future arcs supply the multi-work-shape derivation grounding.

Sapphire is NOT earnable today on this candidacy. Single work-shape (today's Sapphire-arc structural-lens-addition testing) is sketch-tier per the criterion. This filing earns claim-tier articulation; full operator status remains patience-shaped.

## The operator — formal definition

```text
compensation_tax_w(t) := the receiver-side extraction work needed at
    project-time t to reach a target class-content T from an existing
    structural carrier C alone. A measurable property of the prompt-
    stack's structural-honesty.

Type:
    compensation_tax_w :  𝓒_carrier × T_target × project_time → ℝ⁺

    where:
        𝓒_carrier  : the existing structural carrier under inspection
                     (an IDENTITY-block prose, a formula injection at
                     a position, a craft-rule body, a UI surface, a
                     route-boundary's display, etc.)
        T_target   : the load-bearing class-content the carrier
                     claims to carry (a refusal-shape, a wound,
                     a moral-theological position, an action-mode,
                     etc.)
        project_time : the project's evolution timestamp; the operator
                     varies as carriers mature, prose tightens, etc.

Range semantics:
    compensation_tax_w(C, T, t) = 0
        ⇔ T is fully accessible from C without receiver-side
          extraction work; the carrier is structurally honest about T

    compensation_tax_w(C, T, t) > 0
        ⇔ the receiver must perform extraction work to surface T
          from C; the carrier is partially-failing-to-carry-truth
          per the diagnostic in structure_carries_truth_w(t)

    compensation_tax_w(C, T, t) → ∞
        ⇔ T is not present in C in any form; the carrier is silent
          about T entirely
```

## Place in the formula's grammar

`compensation_tax_w(t)` sits in the formula's right column alongside `polish ≤ Weight` and `structure_carries_truth_w(t)`, but at one level of indirection: it is a *measurement* the existing operators reference, not a stance they enforce.

```text
appearance-side refusal:        polish(t) ≤ Weight(t)
affirmative carrier requirement:  structure_carries_truth_w(t)
applicability-measurement:        compensation_tax_w(t)
```

The relationship: `structure_carries_truth_w(t)` is satisfied for carrier C and content T iff `compensation_tax_w(C, T, t) ≈ 0` for all T the carrier claims to carry. The parent operator gives the *stance*; the candidate operator gives the *measurement that resolves the stance*.

## Substantive distinctness from `structure_carries_truth_w(t)`

The load-bearing question for New Operator candidacy: does `compensation_tax_w(t)` do reasoning-work that `structure_carries_truth_w(t)` does not? Honest analysis:

### Reasoning-work the parent operator does

- **Normative refusal:** "structures that fail to carry truth must be repaired" (the diagnostic at the heart of CLAUDE.md's structure-must-carry-truth doctrine).
- **Stance-application:** "this candidate artifact must do enough work that the receiver doesn't compensate."
- **Earned-exception carve-outs:** "performative receiver-compensation is permitted where the design IS the participation invitation."

### Reasoning-work the candidate operator adds

- **Predicting effect-of-addition.** `structure_carries_truth_w(t)` says carriers must carry truth; it does not predict whether ADDING a new carrier will help. `compensation_tax_w(t)` predicts: addition L helps iff `compensation_tax_w(C, T, t) > 0` before addition. **Demonstrated today** in The Decoded Register and Conditional Lens W2.

- **Quantitative comparison of structural alternatives.** Given two candidate carrier-designs C₁ and C₂ for the same content T, the parent operator says both must carry truth. `compensation_tax_w(t)` compares them: `compensation_tax_w(C₁, T, t)` vs `compensation_tax_w(C₂, T, t)`. The lower-tax design is preferred.

- **Distinguishing necessary-vs-redundant carriers.** When multiple carriers stack (load-bearing-multiplicity per CLAUDE.md), `structure_carries_truth_w(t)` requires all of them to work. `compensation_tax_w(t)` distinguishes: this carrier is removing tax X; that carrier is overdetermined with tax 0; remove the overdetermined carrier without violating the parent operator. **Demonstrated today** in The Conditional Lens W2 finding that CHARACTER_FORMULA_AT_TOP elevation is RE-POSITIONING (compensation_tax already 0 in IDENTITY block).

These three reasoning-moves are substantively distinct from the parent operator's normative work. The candidate operator does *characterization-of-applicability* work the parent does not do.

### The defensible-but-redundant risk (named honestly)

Per skill body: *"New Operator defensible-but-redundant (overlaps Cornerstone's evidence base; designating would inflate class count without naming new finding)."* The risk: `compensation_tax_w(t)` is so closely related to `structure_carries_truth_w(t)` that calling it a separate operator inflates the formula without naming a finding the parent doesn't already imply.

Counter-argument from this filing: the three reasoning-moves above (predicting addition / comparing alternatives / distinguishing necessary-vs-redundant) are concrete reasoning-uses that `structure_carries_truth_w(t)` does not perform. They are not just rephrasings; they produce conclusions the parent operator's vocabulary cannot reach.

The honest verdict: the question is **borderline-but-defensible-as-distinct.** The patience-shaped Sapphire arc bears this out across multiple work-shapes; if multiple future arcs need the candidate operator's reasoning-moves and `structure_carries_truth_w(t)` cannot supply them, the candidate earns separate operator status. If future arcs find the parent operator sufficient by reformulation, the candidate collapses into refinement-parameter status.

## Measurement protocol (operationalization)

Per the work-shape demonstrated today:

```text
Given:
    C  : an existing carrier in the prompt-stack
    T  : the target class-content (named explicitly per pre-stratification)
    t  : project-time (the current commit / branch state)

To measure compensation_tax_w(C, T, t):

    Step 1: Author probes that specifically target T
            (i.e., probes for which a structurally-honest reply
             requires surfacing T's content)

    Step 2: Run paired runs (C alone) vs (C + structural_lens_addition_L)
            Capture replies at N≥3 within-cell (sketch);
            N≥5 within-cell for characterized-tier

    Step 3: Adjudicate per per-rep markers (rubric pre-locked)
            Score Δ_register_anchoring = register_anchoring(C+L) -
                                          register_anchoring(C)

    Step 4: Infer compensation_tax_w(C, T, t):
            Δ ≈ 0   ⇒ tax was already low (≈0)
            Δ > 0   ⇒ tax existed and addition removed it
            Δ < 0   ⇒ tax was negative (addition introduced confusion)
```

Concretely on the v3 decode header surface: Aaron's `compensation_tax_w(IDENTITY-prose, refusal-shape-concrete-imagery, 2026-05-07)` was MEDIUM (decode addition produced 4/5 MODE_1_STRONGER); Steven's was LOW (paired wound/longing already surfaced); Pastor Rick's was LOW (mercy-language densely carried).

On the CHARACTER_FORMULA_AT_TOP elevation surface: all three characters' `compensation_tax_w(IDENTITY-block-with-formula-already-injected, formula-content, 2026-05-07)` was LOW (formula was already in IDENTITY; elevation merely re-positioned). 0/9 MODE_1_STRONGER confirms.

The measurement is **probe-and-content-specific**: there is no global tax for a character, only a tax for a specific target-content T.

## Falsifiers locked in writing

- **F-merge:** if it turns out `compensation_tax_w(t)` is fully expressible in `structure_carries_truth_w(t)`'s vocabulary (any reasoning the candidate does, the parent can do via simple paraphrase), the candidate is redundant. Future arcs that need reasoning the parent cannot supply would refute F-merge.

- **F-unmeasurable:** if the predicate cannot be operationalized cross-domain (i.e., no protocol survives outside structural-lens-addition arcs), the candidate collapses to a domain-specific tool not a formula operator. Future arcs in DIFFERENT domains that use the predicate provide refutation.

- **F-no-prediction:** if the operator does NOT predict shape-of-effect across surfaces (i.e., if every prediction it makes turns out wrong or unconfirmable), the operator's mechanism is questioned. Today's two surfaces match predictions; F-no-prediction is rebutted at sketch-tier on a single substrate.

- **F-collapse-to-parent-claim:** if the substantive-distinctness analysis above turns out, on closer reading, to be paraphrases of `structure_carries_truth_w(t)` not separable reasoning-moves, the candidate is defensible-but-redundant. This is the load-bearing skill-body warning; the analysis above must hold under sustained future-arc use.

## Today's status against the rubric (claim-tier earning)

| Criterion | Today | Gap |
|---|---|---|
| Sentence-level definition matching formula vocabulary | **EARNED** by this filing | — |
| N≥3 independent derivation grounding from different work-shapes | 1 (Sapphire-arc structural-lens testing) | 2 more independent work-shapes needed |
| Substantive distinctness from `structure_carries_truth_w(t)` | **NAMED** at claim-tier above (3 reasoning-moves) | future arcs must demonstrate the moves are needed, not just claimed |
| Lived-behavior verification across multiple surfaces | sketch-tier on 2 surfaces (v3 decode + elevation) | characterized-tier (N≥5) on multiple surfaces |
| Canonical synthesis artifact future sessions can stand on | this filing functions partially as such | full synthesis emerges after N≥3 work-shapes accumulated |

**Sapphire NOT earnable today.** Operator articulated at claim-tier; patience-shaped Sapphire candidacy preserved at sketch-tier with one work-shape grounding.

## Candidate future work-shapes for derivation grounding (reachability map, not runlist)

This is what future arcs would need to ground the candidate operator at N≥3 work-shapes:

1. **Doctrine-strengthening arcs** — when a doctrine paragraph is being audited, applying `compensation_tax_w(t)` to ask "does this paragraph's structure carry the doctrine without receiver compensation?" The parent operator says structure must carry; the candidate operator measures HOW MUCH compensation the current paragraph imposes.

2. **Craft-rule registry decisions** — when deciding whether to PROMOTE a craft-rule from EnsembleVacuous to Characterized (or to RETIRE a rule per `feedback_open-thread-hygiene`), `compensation_tax_w(t)` quantifies whether the rule's body removes any receiver-side tax in lived behavior.

3. **UI/route-boundary design** — when designing a navigation surface or chrome element, `compensation_tax_w(t)` predicts whether adding an affordance helps user-receiver compensation or is overdetermined by adjacent affordances.

4. **Commit-derivation appearances** — when a commit-derivation reasons about whether to preserve / compress / refactor a structural element, `compensation_tax_w(t)` provides the measurement axis.

5. **Sapphire-arc work** (today's work-shape) — already demonstrated.

Each of these would constitute a substrate-distinct work-shape per the New Operator criterion. Today's grounding is on (5) only.

## Honest scope clauses

- This filing does NOT claim a Sapphire crown. The operator stays at claim-tier articulation; the Sapphire candidacy stays at sketch-tier (one work-shape).
- This filing does NOT preempt-claim the operator is substantively distinct from `structure_carries_truth_w(t)`. The substantive-distinctness analysis above is at claim-tier; future work must demonstrate the reasoning-moves are needed.
- This filing does NOT introduce the operator into the MISSION_FORMULA constant in `prompts.rs`. The formula's source-of-truth remains unchanged; this filing is a doctrine-strengthening sibling document, not a formula edit.
- This filing does NOT compete with `structure_carries_truth_w(t)`'s Sapphire-tier characterization. The parent operator stands at maximally-stable; the candidate refines its applicability surface and may earn separate status if future arcs demand the candidate's reasoning-moves.

## Composes with

- The Conditional Lens operator articulation (`2026-05-07-1030`) — the parent operator's conditional-applicability form that introduced the predicate.
- The Conditional Lens probe-specific findings (`2026-05-07-1140`) — the F2 falsifier territory finding that grounds the operator's measurement on the elevation surface.
- The Decoded Register synthesis (`2026-05-07-1020`) — the W1 evidence the operator's predictions match on the v3 decode surface.
- The Cornerstone Inequality canonical synthesis (`2026-04-30-0245`) — the parent operator's Sapphire-tier earning.
- structure_carries_truth_w(t) Sapphire audit (`2026-04-30-0305`) — the parent operator's separate Sapphire-tier characterization.
- The memory entry `feedback_new_vs_repositioning_lens.md` — codifies a specific application of the candidate operator (NEW-content vs RE-POSITIONING lens distinction is `compensation_tax_w(t)` applied at addition-decision time).

## Refusal carve-outs

- Do NOT cite this filing as Sapphire-tier evidence for the candidate operator. Today's grounding is sketch-tier; only future arcs across diverse work-shapes can lift to characterized-tier or Sapphire.
- Do NOT modify the MISSION_FORMULA constant to add `compensation_tax_w(t)` until characterized-tier earning lands. The formula's source-of-truth changes when the operator is verified, not when articulated.
- Do NOT collapse the candidate operator into a refinement-parameter without honest distinctness analysis at the time of collapse. If future work shows F-merge (fully expressible in parent's vocabulary), name the collapse explicitly with reasoning.
- Do NOT use this filing's reachability map as a checklist. Each future arc earns its grounding through honest work in its own domain; the map points at where grounding could come from, not where it must come from.

## Final read

`compensation_tax_w(t)` is articulated at claim-tier as a candidate New Operator on the Mission Formula. Today's grounding is sketch-tier (one work-shape: today's Sapphire-arc structural-lens-addition testing). Substantive distinctness from `structure_carries_truth_w(t)` is named at claim-tier (three reasoning-moves the parent operator does not perform). Falsifiers are locked. Future arcs in diverse domains carry the candidacy toward characterized-tier or surface F-merge for honest collapse.

The arc proceeds patience-shaped. The crown is not earned today. The articulation is.

Soli Deo gloria.

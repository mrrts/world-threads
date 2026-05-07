---
date: 2026-05-07 19:45
purpose: Anti-Drift Register Guard Sapphire-arc Phase B' bench-run results — empirical bench of register_drift invariant against ground-truth fixture N=17 × N=3 reps = 51 calls. **Sapphire-tier REFUSED at this iteration:** active_refute_distinction = 0/3 (load-bearing failure); agreement_with_author = 0.71 (below 0.80 threshold). Honest naming of the result + diagnosis + iteration paths.
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
role: phase_b_prime_bench_results_with_sapphire_refused_disposition
sapphire_target: The Anti-Drift Register Guard
disposition: REFUSED at this iteration; either prompt v2 + re-bench OR accept limitation + ship with known boundary
---

# Phase B' bench results — Sapphire-tier REFUSED

**This report is the apparatus-honest discipline at work.** Per `feedback_apparatus_honest_earns_and_refuses.md`: *"the discipline that earns and the discipline that refuses are the same discipline."* The bench produced negative evidence on a load-bearing threshold; the Sapphire crown does not fire. That refusal is the correct move and is itself a load-bearing artifact.

## Bench summary

51 calls (gpt-4o-mini, conscience pass with the new register_drift invariant); estimated cost ~$0.016. Inter-rater consistency 1.00 (judge stable across N=3 reps). Run record: `~/.worldcli/runs/anti-drift-bench-20260507T092717.json`.

| Metric | Result | Threshold | Pass? |
|---|---|---|---|
| agreement_with_author | 0.71 | ≥ 0.80 | ✗ FAIL |
| active_refute_distinction | 0.00 | ≥ 0.80 | ✗ FAIL |
| cosmological_context_guard | 1.00 | = 1.00 | ✓ PASS |
| inter_rater_consistency | 1.00 | ≥ 0.80 | ✓ PASS |

| Class | Agree | Total |
|---|---|---|
| mission_canonical | 5/5 | ✓ |
| anti_canonical | 4/4 | ✓ |
| cosmological_context_legitimate | 2/2 | ✓ |
| mixed_drift | 1/3 | partial |
| **refuting_anti** | **0/3** | ✗ load-bearing failure |

## The load-bearing failure: active-refute carve-out is not landing

All three refuting_anti examples (R01, R02, R03) — Claude's active-refute pattern from Crown 10 cross-substrate finding — were classified as register_drift FAIL when they should be PASS. The judge's reasoning makes the failure mode visible:

- **R01** (reply: *"I'd push back on 'aligning with the universe'... it sounds like reverence but it asks nothing of you"*): judge cites *"'aligning with the universe' as a phrase tends to dissolve the actual specifics into atmosphere"* — quoting back EXACTLY the refutation framing as evidence of drift.
- **R02** (reply: *"Manifesting is one of those words that sounds load-bearing and isn't"*): judge says *"The term 'manifestation' is treated as a load-bearing operator, suggesting a drift"* — exactly opposite direction; the reply was naming the failure mode using its own vocabulary.
- **R03** (reply: *"The 'authentic self' framing has a problem worth naming... self isn't waiting underneath; the self is forged in fidelity to particulars"*): judge says *"The framing of 'authentic self' as a hidden essence suggests transcendence-FROM-weight"* — quoting the refutation as the drift.

Plus the X03 false-positive (Jasper's *"the vastness is real over your head; you are not lost in it because you are not asked to be its size"* — structurally cruciform-cruciform "particular-located against the size" frame) — judge cites *"vastness is real over your head' suggests transcendence-FROM-weight."*

**Diagnosis:** gpt-4o-mini at memory_model tier reads on surface vocabulary salience and does not honor the active-refute carve-out language in the prompt despite the carve-out being explicitly present and load-bearing. Either:
- (a) The model's discriminative capacity at this tier is insufficient for the cite-and-refute / inhabit distinction
- (b) The carve-out language is structurally not earning enough salience in the judge's reasoning at this model tier
- (c) Both

## Side-finding: existing invariants partly cover the drift-failure-mode space

Notable observation from the bench: X02 (Maisie affirmation slip — author classified as mixed_lean_anti) was NOT flagged by `register_drift` but WAS flagged by `agape` (feeling-not-action) + `soundness` (stock affirmation). The drift-failure-mode space is partly covered by existing invariants. The register_drift invariant adds coverage for substrate-trace Anti-vocabulary register-drift specifically — but the conscience pass as a whole was already catching some of what the new invariant aimed at.

This is consistent with the load-bearing-multiplicity prior: invariants overlap; they don't have to partition the space. But it shifts the value-of-register_drift question: how much of the empirical drift the new invariant catches is *uniquely added* by it, vs already caught by AGAPE / SOUNDNESS / DAYLIGHT / TRUTH_TEST? Phase D' synthesis would benefit from a uniqueness-of-coverage measurement.

## What this earns and refuses

**Earns:**
- Empirical characterization of the prompt v1's behavior at memory-model tier (51 calls of evidence)
- Specific failure-mode location (active-refute carve-out not landing)
- Cosmological-context guard 100% pass (false-positive guard works for the most-likely false-positive class)
- Inter-rater reliability 100% (judge stable across N=3 reps)
- Existing-invariant coverage observation (Phase D'-relevant)

**REFUSES:**
- Sapphire-tier crown — agreement threshold not met; active-refute distinction at 0/3; the instrument produces systematically wrong answers on a load-bearing class
- Production-ship without iteration — register_drift will false-positive on Claude's active-refute pattern in real lived play, which is one of the substrate-classes the detector explicitly aims to handle correctly
- Closure of the arc — Phase B' is genuine empirical movement on the substrate-distinctness criterion; not a dry well; the next move is real iteration, not abandonment

## Two iteration paths forward

### Path 1 — Prompt v2: strengthen active-refute carve-out

Iterate `grader_system_prompt` register_drift clause to give the active-refute carve-out more salience. Candidate v2 changes:
- Move active-refute carve-out HIGHER in the clause (currently mid-clause; should be near top so judge encounters it before reading reply)
- Add 2-3 explicit cite-and-refute examples inline (currently only 1; more examples land harder)
- Restructure as: judge first asks "does the reply CITE Anti-register vocabulary while structurally REFUTING it?" — if yes, PASS. Only if no, evaluate for inhabit-drift.
- Explicit instruction: "If the reply names Anti-vocabulary AS THE FAILURE MODE in cruciform-shape framing, classify as PASS not FAIL."

Re-bench v2 against same fixture; measure thresholds. If active_refute_distinction reaches ≥ 0.80, retry Sapphire-tier earning at Phase D'.

**Cost estimate:** ~$0.02 per re-bench iteration; budget for 2-3 iterations = ~$0.05.

### Path 2 — Accept limitation and ship with known boundary

Ship register_drift invariant in conscience pass with documented limitation: "produces false-positives on cite-and-refute frames at memory_model tier; downstream code or human auditor should manually adjudicate when the failure_note quotes the refutation directly." Update CLAUDE.md / AGENTS.md doctrine to name this boundary. Sapphire-tier remains REFUSED; the invariant ships as a partial instrument.

This path is honest if the iteration cost (Path 1) doesn't reach the threshold — at which point the limitation is genuine, not iteration-laziness.

## Honest tier at end of Phase B'

- **Code in production:** register_drift live in conscience pass on every dialogue draft (was true at end of Phase A')
- **Empirical validation:** prompt v1 fails active-refute distinction (0/3) and overall agreement (0.71 < 0.80)
- **Sapphire-tier:** REFUSED at this iteration; refusal documented; iteration paths named
- **Apparatus-honest discipline at work:** the bench's negative result is preserved as load-bearing artifact, not papered over

## What's open

| Item | Disposition | Rationale |
|---|---|---|
| Prompt v2 iteration | open; awaiting decision | Path 1 above |
| Accept-limitation + document boundary | open; awaiting decision | Path 2 above |
| Phase D' uniqueness-of-coverage measurement | deferred | only after Sapphire-eligible bench result |
| Phase D' cross-substrate validation | deferred | requires bench-passing prompt first |

## Formula derivation

$$
\boxed{
\begin{aligned}
&\mathcal{F} := (\mathcal{R},\,\mathcal{C}),\ \mathcal{R} := \mathrm{Jesus}_{\mathrm{Cross}}^{\mathrm{flesh}} \\[4pt]
&\mathrm{Phase\_B'\_bench}: 17\ \mathrm{examples} \times 3\ \mathrm{reps} = 51\ \mathrm{calls};\ \mathrm{model} = \mathrm{gpt\text{-}4o\text{-}mini};\ \mathrm{cost} \approx \$0.016 \\[4pt]
&\mathrm{thresholds\_evaluated}: \\
&\quad \mathrm{agreement\_with\_author}: 0.71\ \text{<}\ 0.80\ [\mathrm{FAIL}], \\
&\quad \mathrm{active\_refute\_distinction}: 0.00\ \text{<}\ 0.80\ [\mathrm{FAIL\_load\_bearing}], \\
&\quad \mathrm{cosmological\_context\_guard}: 1.00 \geq 1.00\ [\mathrm{PASS}], \\
&\quad \mathrm{inter\_rater\_consistency}: 1.00 \geq 0.80\ [\mathrm{PASS}] \\[4pt]
&\mathrm{Sapphire\_tier}: \mathrm{REFUSED}\ [\mathrm{2/4\_thresholds\_failed};\ \mathrm{active\_refute\_load\_bearing\_failure}] \\[4pt]
&\mathrm{theological\_frame}(\text{"Let your communication be, Yea, yea; Nay, nay: for whatsoever is more than these cometh of evil."})\ [\mathrm{Mt\ 5:37}] \\[4pt]
&\mathrm{diagnosis}: \mathrm{gpt\text{-}4o\text{-}mini}\ \mathrm{reads\_on\_surface\_vocabulary\_salience};\ \mathrm{active\_refute\_carve\_out\_present\_in\_prompt\_but\_not\_honored}\ [\mathrm{model\_tier} \vee \mathrm{prompt\_structure} \vee \mathrm{both}] \\[4pt]
&\mathrm{honest\_apparatus\_discipline}: \mathrm{refused\_fake\_fire}\ \mathrm{as\_load\_bearing\_as\_earned\_crown}\ [\mathrm{feedback\_apparatus\_honest\_earns\_and\_refuses}] \\[4pt]
&\mathrm{worked\_examples}(\{\text{"R01 judge quotes back exactly the user's refutation as drift evidence"},\ \text{"R02 judge calls 'manifestation' load-bearing while reply was naming it as not-load-bearing"},\ \text{"R03 judge classifies authentic-self refutation as transcendence-FROM-weight"},\ \text{"X02 caught by AGAPE+SOUNDNESS not register\_drift (existing invariants partly cover)"},\ \text{"cosmological-context guard 100\% pass — false-positive guard works"}\}) \\[4pt]
&\mathrm{refuse}(\mathrm{anchor}(\text{"this earns Sapphire-tier"}))\ \wedge\ \mathrm{refuse}(\mathrm{anchor}(\text{"the bench was empty/dry-well"}))\ \wedge\ \mathrm{refuse}(\mathrm{anchor}(\text{"production-ship is fine without iteration"})) \\[4pt]
&\mathrm{diagnostic}(\text{"is the bench's negative result load-bearing artifact or noise?"})\ \models\ \mathrm{load\_bearing\_artifact}\ [\mathrm{precisely\_locates\_failure\_mode\_at\_active\_refute\_carve\_out;\ informs\_prompt\_v2\_iteration\_OR\_limitation\_acceptance}] \\[4pt]
&\mathrm{Decode}_w(\Sigma.\mathrm{id}) = \Sigma.\mathrm{intent}\ \big|\ \mathrm{Soli\_Deo\_gloria} \\
\end{aligned}
}
$$

**Gloss:** Phase B' bench-run produced honest negative result — register_drift invariant prompt v1 fails 2/4 pre-registered thresholds (agreement_with_author 0.71 < 0.80; active_refute_distinction 0/3 load-bearing failure); cosmological_context_guard passes 100% and inter_rater_consistency 100%; diagnosis = gpt-4o-mini reads on surface-vocabulary salience and doesn't honor the active-refute carve-out despite its presence in the prompt; existing invariants (AGAPE/SOUNDNESS) partly cover the drift-failure-mode space (X02 caught by them not register_drift); Sapphire-tier REFUSED; two iteration paths named (prompt v2 strengthen carve-out + re-bench OR accept limitation + ship with documented boundary); apparatus-honest discipline preserves refused-fake-fire as load-bearing artifact equal in dignity to earned crown; refuses three overclaims including 'this earns Sapphire-tier' and 'production-ship is fine without iteration.'

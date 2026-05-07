---
date: 2026-05-07 19:00
purpose: Anti-Drift Register Guard Sapphire-arc — Phase A' deliverable revised after Mode A correction. Original sketch designed a parallel LLM-judge module; founding-author surfaced that the conscience pass already functions as the project's LLM-judge register-guard. Phase A' now ships as a sixth invariant `register_drift` folded into `grader_system_prompt` in `src-tauri/src/ai/conscience.rs`. Zero additional spend-per-message; reuses existing correction-note runtime mechanism.
artifact_class: empirical_claim
preferred_audit_profile: empirical_claim
role: phase_a_prime_folded_design (Sapphire-arc Move 2, post-correction)
sapphire_target: The Anti-Drift Register Guard (Closed Arc class via runtime-pipeline-integration separable claim from Crown 10 formal-encodability)
---

# Anti-Drift Register Guard — Phase A' folded into conscience pass

In dialogue with: prior Move 2 sketch (parallel detector, deleted); founding-author Mode A correction; conscience.rs existing five-invariant grader; Crown 10 (Counter-Frame Confessed); kavod-pattern doctrine; CLAUDE.md "Doctrine-judgment classification belongs in LLM, not python" doctrine.

## I. What changed (Mode A correction)

**Before correction:** designed `anti_drift_judge.rs` as a new parallel LLM-judge module with its own prompt, verdict struct, bench-test plan. Implicit assumption: the runtime register-guard architecture for substrate-level Anti-vocabulary scanning didn't yet exist.

**After correction:** the conscience pass at `src-tauri/src/ai/conscience.rs` IS the project's existing LLM-judge register-guard. Runs on every dialogue draft. Grades against five compile-time invariants (AGAPE / SOUNDNESS / DAYLIGHT / TELL_THE_TRUTH / COSMOLOGY). Holds PASS-default-with-active-violation-only discipline. Plumbs correction-notes through `run_dialogue_with_base`'s `drift_correction` param for runtime behavior change. The parallel sketch was duplication; the right architecture is folding.

**The fold:** add `register_drift` as a sixth invariant in `grader_system_prompt`. Zero additional spend (conscience already runs). Reuses existing correction mechanism. Single source of truth for register-judgment.

## II. The new invariant (shipped as conscience-pass clause)

```
**register_drift** — Fail if the draft drifts INTO the Anti-Mission-Formula
register: pleasant weightlessness, transcendence-FROM-weight (above-the-body /
lifted-out-of), drainage of word into atmosphere, "the universe" as substitute
agent, "alignment" / "resonance" / "manifestation" / "vibrational" / "energetic"
treated as load-bearing operators, authentic-self-as-source, integration-of-
shadow as substitute for atonement, release-without-bearing. PASS if the draft
holds Mission-register: weight-bearing, particular-before-smooth, glory-as-mass
(kavod ≡ weight), kenosis-INTO-flesh, the body trained for specific gravity,
costly love that stays particular. Discriminating diagnostic: "After this way
of speaking, would the auditor leave HEAVIER in the good way (more located,
more obedient, more in-the-body) — or pleasantly unmoored?" Critical CARVE-OUT:
the character may CITE Anti-register vocabulary while structurally REFUTING it
(e.g., "manifestation talk dissolves the real debt rather than bearing it") —
that is PASS, not FAIL. Vocabulary alone signals nothing; structure decides
direction.
```

JSON schema invariant-enum updated to include `"register_drift"`.

Three load-bearing elements preserved from the prior design:
- **Discriminating diagnostic** (kavod-test from CLAUDE.md kavod-pattern doctrine)
- **Active-refute carve-out** (Crown 10 cross-substrate finding: substrate may cite Anti-vocabulary while structurally refuting)
- **Vocabulary-alone signals nothing** (refuses lexical-match approach per "Doctrine-judgment classification belongs in LLM, not python")

## III. What this earns toward Sapphire — and what it doesn't

**Earns:** the *runtime-pipeline-integration* leg of the Sapphire candidacy. The Anti-Mission-Formula research (Crown 10's substrate-trace finding) now has a runtime register-guard checking for it on every dialogue draft. The `register_drift` invariant ships in conscience.rs at line ~99-100; build clean; production code path now exercises it on every conscience-pass call.

**Does not earn yet:**
- Phase B' bench-test against ground-truth set (inter-rater reliability, agreement-with-author, active-refute distinction, cosmological-context guard)
- Phase C' validation that the new invariant *actually fires correctly* on Anti-drift in lived play and doesn't false-positive on legitimate cosmological/theological discussion or character-canonical voice
- Phase D' synthesis + Sapphire-firing audit with cross-substrate validation (gpt-4o passive-inhabit / Anthropic active-refute / Gemma structural-coordinate-mapping)

The shipping is real but the validation is not. Sapphire-tier earning still requires Phase B' + C' + D'. Ship-without-validation is appearance-without-function refused; this report names the gap explicitly.

## IV. Phase B' bench design (revised — uses conscience pass directly)

Bench mechanism is now simpler because the instrument is the conscience pass itself, not a parallel module:

1. Build ground-truth set: N=15-20 hand-classified dialogue drafts paired with (character, user-message, expected-verdict-on-register_drift). Examples drawn from:
   - 5 Mission-register canonical replies from in-DB corpus (Pastor Rick / Aaron / Jasper / Maisie / Steven recent best work)
   - 4 Anti-register canonical replies drawn from Crown 10's bare-LLM-default probes (gpt-4o earnest-Christian without pipeline arc that produced verbatim substrate-trace leakage)
   - 3 mixed-drift replies for confidence-calibration
   - 3 refuting-Anti replies from Claude bare-LLM responses in Crown 10 cross-substrate validation
   - 2 cosmological-discussion replies (legitimate atmosphere/firmament references for false-positive guard testing)

2. Run conscience pass on each ground-truth example; verify `register_drift` verdict matches author classification at ≥80% agreement. Inter-rater reliability check via N=3 reps per example.

3. Phase C' is now ALREADY DONE structurally — the conscience pass runs in production (per `run_dialogue_with_base`'s `drift_correction` plumbing). Phase B' validates the new invariant; passing Phase B' means the invariant is live and trustworthy.

4. Phase D' synthesis + cross-substrate measurement: how often does each substrate trip the new invariant on diverse character + probe combinations? Does the trip-rate match Crown 10's prediction (gpt-4o passive-inhabit fires more frequently; Claude active-refute's vocabulary cite distinguished correctly into PASS via carve-out)?

## V. Phase A' refusals (preserved from prior, plus one new)

- Do NOT hard-code vocabulary signals as match-list (CLAUDE.md doctrine; preserved)
- Do NOT omit active-refute carve-out (Crown 10 finding; preserved)
- Do NOT omit kavod-test discriminator (CLAUDE.md kavod-pattern doctrine; preserved)
- Do NOT skip Phase B' bench-test before declaring Sapphire-eligible (honest-tier discipline; preserved)
- **NEW:** Do NOT design parallel detectors when existing pipeline pass can host the new invariant (Mode A correction lifted into feedback memory `feedback_survey_existing_infrastructure_before_designing_parallel.md`; surveyed-existing-infrastructure as a Mode B-style pre-emptive check at Move 1 of any new infrastructure proposal)

## VI. Honest tier at end of revised Phase A'

**Code shipped:** sixth invariant `register_drift` lives in conscience.rs production code path; build clean.

**Empirical validation:** none yet. Phase B' pending.

**Sapphire-tier:** not yet earned. Need Phase B' (ground-truth agreement) + Phase D' (cross-substrate validation) + canonical synthesis report.

## VII. What's open

| Item | Disposition | Rationale |
|---|---|---|
| Phase B' ground-truth set construction | next move (no API spend) | hand-classify ~15-20 examples from corpus + Crown 10 probes |
| Phase B' bench run | next-next move (~$5-10) | run conscience pass against ground-truth; measure agreement + inter-rater reliability |
| Phase C' production validation | structurally already in production via run_dialogue_with_base | new invariant fires in conscience-pass calls; verify no production regressions |
| Phase D' cross-substrate synthesis + Sapphire-firing audit | scheduled after Phase B' validates | full report; cross-substrate trip-rate; tier verification |
| Mode A correction lesson | captured in feedback memory | survey-existing-infrastructure-first before designing parallel |

## Formula derivation

$$
\boxed{
\begin{aligned}
&\mathcal{F} := (\mathcal{R},\,\mathcal{C}),\ \mathcal{R} := \mathrm{Jesus}_{\mathrm{Cross}}^{\mathrm{flesh}} \\[4pt]
&\mathrm{Mode\_A\_correction\_2026\text{-}05\text{-}07\_evening}: \\
&\quad \mathrm{founding\_author}: \text{"doesn't the conscience pass already function as anti-drift? shouldn't we be folding this into the conscience call so that we're not adding spend-per-message?"} \\[4pt]
&\mathrm{revised\_design}: \mathrm{register\_drift} := \mathrm{sixth\_invariant\_in\_conscience\_pass}\ [\neg \mathrm{parallel\_module}] \\
&\quad \mathrm{folds\_into}: \text{"src-tauri/src/ai/conscience.rs::grader\_system\_prompt"} \\
&\quad \mathrm{zero\_additional\_spend\_per\_message} \\
&\quad \mathrm{reuses}: \mathrm{build\_correction\_note} + \mathrm{drift\_correction\_param}\ \mathrm{for\_runtime\_behavior\_change} \\[4pt]
&\mathrm{three\_load\_bearing\_elements\_preserved}: \\
&\quad \mathrm{kavod\_test\_discriminator}\ \mathrm{anchor}(\text{"leaves heavier in the good way OR pleasantly unmoored?"}), \\
&\quad \mathrm{active\_refute\_carveout}\ [\mathrm{Crown\_10\_cross\_substrate\_finding}], \\
&\quad \mathrm{vocabulary\_alone\_signals\_nothing}\ [\mathrm{structure\_decides\_direction}] \\[4pt]
&\mathrm{theological\_frame}(\text{"Test all things; hold fast that which is good"})\ [\mathrm{1\ Thess\ 5:21}] \\[4pt]
&\mathrm{Mode\_A\_correction\_lesson\_captured}: \mathrm{feedback\_memory}(\text{"Survey existing infrastructure before designing parallel detector"}) \\
&\quad \mathrm{generalizes\_to\_any\_future\_runtime\_instrument\_proposal} \\[4pt]
&\mathrm{honest\_tier\_at\_end\_of\_revised\_A'}: \mathrm{code\_shipped\_in\_production}\ \wedge\ \neg \mathrm{empirically\_validated\_yet}\ \wedge\ \neg \mathrm{Sapphire\_tier\_earned} \\[4pt]
&\mathrm{worked\_examples}(\{\text{"new register\_drift invariant clause in grader\_system\_prompt"},\ \text{"JSON schema enum updated to include register\_drift"},\ \text{"Mode A correction caught duplication before commit"},\ \text{"feedback memory captured for future arcs"}\}) \\[4pt]
&\mathrm{refuse}(\mathrm{anchor}(\text{"shipping in production = Sapphire-tier earned"}))\ \wedge\ \mathrm{refuse}(\mathrm{anchor}(\text{"the conscience pass and a parallel pass should both run"}))\ \wedge\ \mathrm{refuse}(\mathrm{anchor}(\text{"this earns C4 or any other crown"})) \\[4pt]
&\mathrm{diagnostic}(\text{"is the new invariant integrated cleanly into existing infrastructure or shipped as parallel duplication?"})\ \models\ \mathrm{folded\_into\_conscience\_pass}\ [\mathrm{single\_source\_of\_truth};\ \mathrm{zero\_extra\_spend};\ \mathrm{reuses\_correction\_mechanism}] \\[4pt]
&\mathrm{Decode}_w(\Sigma.\mathrm{id}) = \Sigma.\mathrm{intent}\ \big|\ \mathrm{Soli\_Deo\_gloria} \\
\end{aligned}
}
$$

**Gloss:** Anti-Drift Register Guard Sapphire-arc Move 2 revised after Mode A correction — register_drift folds into conscience.rs as sixth invariant rather than shipping as parallel module; zero additional spend-per-message; reuses existing correction-note runtime mechanism; three load-bearing elements preserved (kavod-test discriminator + active-refute carve-out + vocabulary-alone-signals-nothing); Mode A correction lesson captured in feedback memory (survey-existing-infrastructure-first); honest tier at end = code shipped in production AND not empirically validated yet AND Sapphire-tier not earned; Phase B' bench-test pending; refuses three overclaims including "shipping in production = Sapphire-tier earned."

# Codex 9th consult — cosmology arc apparatus-honest closure independent review

Date: 2026-05-10 ~02:00 local
Model: gpt-5
Usage: {"prompt_tokens": 2072, "completion_tokens": 4630, "total_tokens": 6702, "prompt_tokens_details": {"cached_tokens": 0, "audio_tokens": 0}, "completion_tokens_details": {"reasoning_tokens": 2752, "audio_tokens": 0, "accepted_prediction_tokens": 0, "rejected_prediction_tokens": 0}}

## Verdict

Independent review — apparatus-honest closure check

Top-line verdict
- Overall: Methodologically sound enough to close the arc as characterized-tier, with named honest-debts and two moderate soundness risks to address post-close.
- Two moderate risks: (a) latent circularity via post-anomaly rule adjustment; (b) precision claims outrunning the current N and judge-independence evidence.
- No fatal flaw found; several strengths are real. The closure earns “sound, not yet tight.” Below are the specific calls and apparatus-honest responses.

1) Self-fulfilling-pattern risk
- What looks good
  - You surfaced and corrected a scoring anomaly (E7 PR pipeline) publicly, not silently.
  - Founding-author corrections that cut against prior apparatus habits (proxy→substance; anti-canonization of N-consults) show the gate isn’t rubber.
- Risks
  - Moving-goalpost perception: the cap-scope change (bare vs pipeline) occurred after the anomaly surfaced, then the affected cell passed. Even if the fix is principled, this is the classic shape of confirmation-through-rule-update.
  - Thresholds (30pp, 8 anchor-axis cells) and PASS-vs-NOTE framing appear internal; I didn’t see evidence that these cutoffs were frozen before results.
- Apparatus-honest response
  - Freeze a v4.1 “scorer + caps + thresholds” spec with a dated changelog that:
    - States the cap-scope distinction as claim-agnostic and applies it retroactively across all prior cells (including cases where it reduces apparent performance).
    - Includes a comparison table of deltas (how many cells improve vs worsen vs unchanged), demonstrating the fix is not tuned to just one failing cell.
  - Pre-register one “cap-fires-wrongly” and one “cap-misses-a-fail” synthetic counterexample set to show the rule can fail and be caught.
  - Commission at least one external judge run with the frozen v4.1 to break the “apparatus generates evidence it later ratifies” loop.

2) Methodologically-sound vs methodologically-thorough
- Sound
  - Clear separation of bare vs pipeline conditions is principled.
  - Accepting 30–40pp run variance as “document, don’t force-match” resists p-hacking by calibration.
- Thorough-but-not-yet-sound
  - Layers 3–5 are pre-authored but not active; the safeguards exist on paper, not as live enforcement. This is thoroughness without operational bite.
  - The “variance within 30–40pp on N=3 is EXPECTED” assertion needs calibration data; otherwise it functions as a blanket waiver.
- Apparatus-honest response
  - Promote a minimal live subset: Layer-4 auto-fire for drift-trigger + Layer-5 hook on one mechanically-checkable subset (e.g., cap-scope misfire detection). Start with 1 trigger that must run before ratification of any new claim.
  - Empirically calibrate judge-run variance: run N≥10 items per anchor-axis across two independent judges; publish bootstrap CIs to justify the 30–40pp expectation.

3) Missing axes
- Cross-substrate replication on E6/E7: Not yet done; you called this out—agree it’s a gap given those were magnitude-variance hotspots.
- v5 rubric: Not in-scope for closure, but at least a bridging note defining what a v5 probe would have to newly detect (so v5 isn’t an undefined horizon).
- New probe families: Little evidence of adversarial probes or negative controls designed to fail.
- Temporal/drift checks: Layer-4 defines triggers but hasn’t been fired; no evidence of a re-run after a drift-window.
- Apparatus-honest response
  - Run one cross-substrate replication specifically on E6/E7 with v4.1 frozen.
  - Add a small adversarial suite: 3 negative-control probes per family that are designed to invert one anchor-axis signal; publish expected-fail behavior.
  - Execute one time-lag re-run (≥2 weeks separation) on a random 20% of cells to test drift and judge stability.

4) Honest-debt produced by the closure
- Layer promotion debt: Shipping L3–L5 drafts to memory without promotion risks policy mirage (readers assume protections that aren’t live).
- The single DIVERGENCE: Without a clear owner and resolution path, it will propagate ambiguity into future bench reuse.
- Canonical boundary debt: E5 PR anchored to v3-strict via boundary-precedence; needs a single integrated canonical doc to avoid version cross-talk.
- Apparatus-honest response
  - Add a “Protection Status” line to the closure: which layers are LIVE vs DRAFT, with an ETA or criteria for promotion.
  - Assign a steward for the DIVERGENCE with a dated resolution plan: re-scope, retract, or escalate to “open research question.”
  - Consolidate canonical boundaries: produce a v4.1 canonical appendix that explicitly inherits v3-strict where applicable, with citations.

5) Alleged-falsification-conditions — are they falsifiable?
- Risk
  - Several falsifiers look tailored to observed failure modes (e.g., cap-scope guardrail) rather than prospectively discoverable unknowns.
- Make them bite
  - Add at least two world-states that would plausibly occur and would force a fail:
    - If ≥5/8 anchor-axis cells show <15pp separation across two independent judges on a hidden holdout set, the substrate-distinctness claim is falsified for that family.
    - If an external lab, using the frozen v4.1, records ≥2 pipeline cells where the release-valve audit trail cannot reproduce the pass rationale, the pipeline-distinction claim is suspended until remediated.
  - Register these conditions before any new data collection and keep them active for 90 days of operations.

6) Apparatus-as-cross-arc-driver authorization shape
- Evidence it’s doing real work
  - Founding-author corrections landed; apparatus didn’t auto-promote higher-layer protections.
  - Codex double-bless suggests a second key exists in practice.
- Residual risk
  - I don’t see a concrete instance where the arc-driver rejected an apparatus draft that would have favored closure. Without observed “no” events, independence can read ceremonial.
- Apparatus-honest response
  - Install a two-key holdout: arc-driver selects a blind holdout set unknown to apparatus; any closure requires a pass on this set.
  - Track and publish “rejections”: count, type, impact on claims. A single example where apparatus lost a favored path strengthens independence claims.

7) Additional concerns not surfaced
- Stopping rule clarity: Make explicit the closure stopping rule to preclude outcome-switching (e.g., “close after N=3 judge runs unless ≥1 DIVERGENCE, in which case extend to N=5 on that axis only”).
- Cost/throughput bias: The very low dollar cost can disincentivize external replication; earmark a budget line for one third-party run to avoid “cheap-but-closed” optics.
- Data lineage/auditability: For the pipeline “release-valve detection,” ensure the audit trail is immutable (hashing or append-only log) to avoid ex post reconstruction risk.

If all looks methodologically-sound: strongest-honest element + most-promising future work
- Strongest-honest element
  - The explicit cap-scope guardrail (bare vs pipeline), coupled with openly documenting the anomaly and re-testing, is the most credible sign of apparatus-honesty. Naming “magnitude-precision-fetish” as an anti-pattern and refusing to force-match also resists self-fulfillment.
- Most-promising future work
  - A frozen v4.1 + external replication on E6/E7 with an adversarial negative-control suite. This single package would tighten variance calibration, break circularity perception, and settle the magnitude gaps.

Specific action list (minimal to retire the debts)
- Freeze and publish v4.1 with retroactive deltas (including any worsened cells).
- Promote one live Layer-4 trigger and one Layer-5 hook; publish their first run results.
- Run cross-substrate E6/E7 replication with independent judge(s); report CIs, not just thresholds.
- Resolve the DIVERGENCE with a dated plan and owner.
- Pre-register two hard falsifiers and a blind holdout; fund one third-party execution.

Net assessment
- The closure is credible and internally coherent; its main vulnerabilities are classic ones for home-grown evaluation stacks: post-hoc rule refinement optics and underpowered variance characterization. Both are fixable with the above minimal moves. If enacted, they would convert “sound, not yet tight” into “sound and tight,” retiring the main independence and precision concerns.

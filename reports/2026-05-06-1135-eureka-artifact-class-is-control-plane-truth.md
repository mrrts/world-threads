# Eureka — artifact class is control-plane truth at the report boundary

**Date:** 2026-05-06 11:35  
**Run:** `/eureka` iteration 1  
**Dialogue with recent arc:** `mission-arc` read before writing. The immediate stack was the witness-bearing payload law, the artifact-class-aware sacred-payload audit, and the discoverability pass making that audit surface reachable to future agents.

## Discovery

The project had already named stronger carrier laws for different artifact kinds:

- craft-rule sacred payload under the v3 taxonomy
- earned empirical artifacts under the witness-bearing payload extension
- decode-faithfulness work under the preferred `sacred_payload_audit.py` surface

What it had **not** named yet is where that difference must become explicit for future work:

> **Artifact class is control-plane truth at the report boundary.**

If a report can later be compressed, audited, cited, or lifted into a portable bundle, the header must tell the receiver what kind of artifact it is. Otherwise the downstream agent has to infer whether the admissible audit law is `generic` or `empirical_claim`, whether witnesses are constitutive or illustrative, and what kind of loss even counts as failure. That is not a cosmetic metadata gap. It is a hidden admissibility fact.

## How it surfaced

Reading the 2026-05-06 report field after the generic audit refactor exposed the pattern:

1. **The reports already carry many boundary-truth fields.**  
   `Bound to`, `Branch / PR`, `Theological frame`, `Tier`, `Harness`, `Instrument`, `Status`, `Verdict` all appear in headers because they change how the reader is allowed to read the artifact.

2. **The new audit surface now depends on artifact class explicitly.**  
   `scripts/sacred_payload_audit.py` asks for `--artifact-class generic|empirical_claim` because the decode/judge standard differs by class.

3. **But the reports themselves usually do not declare the class.**  
   Crown syntheses, detector reports, bounded-drift syntheses, and Empiricon decode audits often make their evidence shape legible through prose, but not as boundary truth. A later agent can infer the class if careful, but must reconstruct what should have been declared.

The missing named law is that this classification belongs at the same layer as the other header fields: at the report boundary itself.

## The law

For any report that is intended to be:

- audited later
- compressed into formula form later
- cited as load-bearing evidence later
- used as a source artifact for a synthesis, crown, witness ledger, or decode audit later

the header should declare at minimum:

- `Artifact class: ...`

And when the downstream audit law is not obvious from the class name alone, the header should also declare:

- `Preferred audit profile: generic | empirical_claim`

This is **control-plane truth**, not enrichment. It changes what later work is allowed to assume about the artifact.

## Why this matters

This changes future behavior in four ways:

1. **Report headers become admissibility surfaces.**  
   A later agent no longer has to rediscover what kind of artifact it is holding before choosing a compression or audit discipline.

2. **Audit-profile selection moves from inference to declaration.**  
   The `sacred_payload_audit.py` profile choice stops being private know-how and becomes boundary-visible.

3. **Losslessness standards stop drifting by report genre.**  
   If a report says `Artifact class: earned_empirical_claim`, then dropping witnesses later is visibly a category error, not just a thin summary.

4. **Historical reports become easier to reuse honestly.**  
   Crown syntheses, detector filings, and decode audits become more portable because the governing class is surfaced where the artifact begins.

## Formula derivation

$$
\boxed{
\begin{aligned}
&\mathrm{report}_r \wedge \mathrm{future\_use}(r)\in\{\mathrm{audit},\mathrm{compression},\mathrm{citation},\mathrm{synthesis}\} \\
&\Rightarrow \mathrm{header\_must\_declare}(r,\ \mathrm{artifact\_class}) \\
&\mathrm{artifact\_class}(r)\ \mathrm{governs}\ \mathrm{admissible\_audit\_law}(r) \\
&\neg \mathrm{declared}(\mathrm{artifact\_class}) \Rightarrow \mathrm{receiver\_must\_infer}(\mathrm{admissibility}) \\
&\mathrm{receiver\_must\_infer}(\mathrm{admissibility}) \Rightarrow \neg \mathrm{control\_plane\_truth} \\
&\therefore\ \mathrm{artifact\_class}\ :=\ \mathrm{control\text{-}plane\ truth\ at\ report\ boundary}
\end{aligned}
}
$$

**Gloss:** If a report will later be audited, compressed, cited, or synthesized, its artifact class belongs in the header because that class governs which downstream fidelity law is admissible.

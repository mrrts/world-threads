---
name: seek-sapphire-crown
description: Run a focused, criterion-specific /play arc oriented toward earning a Great Sapphire-class crown — a maximally-stable cross-witness convergence that constitutes a significant finding within the project's methodology, science, and lived play. First move surfaces the three closest reachable Sapphire candidates; after selection, prints the base-crown rubric AND the additional sapphire-tier criterion, then runs the focused arc until the crown earns honestly OR the well is named dry.
---

# seek-sapphire-crown — GREAT SAPPHIRE ACCELERATOR

## Overview

A bigger-stakes sibling of `/seek-crown`. Where `/seek-crown` targets the most-reachable unearned base crown, `/seek-sapphire-crown` targets a **Great Sapphire-class** earning — a quality designation that any crown can carry when its underlying convergence reaches **maximally-stable cross-witness tier** per CLAUDE.md's great-sapphire calibration. The skill honors the /play contract (HUD per turn, ledger, AskUserQuestion every turn, fixed 4-option chooser cardinality, no nanny-register) but operates at the higher register where each move must defend against **two** thresholds: the base crown's criterion AND the sapphire-tier's substrate-witness-distinctness threshold.

**The "guarantee" is honesty-shaped, not outcome-shaped.** The skill cannot guarantee that every invocation produces a Sapphire crown. It CAN guarantee: (a) the first move surfaces the three closest reachable candidates so the user picks knowingly; (b) after selection, the rubric is printed plainly so the work is criterion-specific; (c) every turn advances either the base-crown criterion or the sapphire-tier-distinctness criterion or both; (d) the crown will not fire at sapphire-class unless the substrate-witness count and failure-mode distinctness honestly reach maximally-stable; (e) if the well runs dry on either threshold, the skill names the dry well rather than fake-firing. **Per the apparatus-honest discipline (`feedback_apparatus_honest_earns_and_refuses.md`): the discipline that earns and the discipline that refuses are the same discipline.** Refused fake-fires are as load-bearing as earned crowns.

## Activation

- `/seek-sapphire-crown` — surface the three closest reachable Sapphire candidates and ask the user to pick.
- `/seek-sapphire-crown <crown-name>` — target a specific candidate (e.g. `/seek-sapphire-crown character-knew`, `/seek-sapphire-crown closed-arc`). Skip the surfacing step.

## First move — surface the three closest reachable Sapphire candidates

Before any work, the skill produces a clear surfacing of where the next Sapphire is most likely earnable. Mechanism:

1. **Read play state** (`.claude/play-state/current.json`) to enumerate earned crowns + earned Sapphires. Earned Sapphires (look for `✨` or `Great Sapphire class` in crown names) cannot re-fire on the same separable claim.
2. **Read the Sapphire calibration** (CLAUDE.md / AGENTS.md sections beginning with "Great Sapphire as a class designation" and "First Great Sapphire crown earned" / "Second Great Sapphire crown earned" / etc.) to ground the criterion.
3. **Enumerate crown classes** that could carry a Sapphire designation: Closed Arc, Apparatus Honest with Itself, The Character Knew, New Operator on the Formula, Mission Formula Verified Empirical, Real User Held (NOT REACHABLE via skill — needs real users). Each is eligible at most once at the Sapphire-class level per its base crown's once-only firing rule, **unless** a separable claim is identifiable that distinguishes a new Sapphire candidacy from prior earnings.
4. **Score reachability** per candidate: signal density in active evidence (recent commits, OBSERVATIONS, reports/ entries), substrate-witness availability (do enough substrate-distinct witnesses exist or accumulate-able?), criterion-specific gates (per-class detail below).
5. **Tag the reachability path** per candidate. Per CLAUDE.md's Sapphire-saturation doctrine (added 2026-05-01 after two consecutive dry-wells revealed the pattern), each candidate falls into one of three reachability paths:
   - **`fresh-elicitation`** — evidence base requires NEW data not yet inside any earned Sapphire's base (e.g., probes to character anchors that haven't been asked the question yet; new instrumentation runs producing fresh data). This is the path actually open after the project's foundational axes have saturated.
   - **`existing-data`** — evidence base would draw from data already inside one or more earned Sapphires' bases, just re-interpreted through a new lens. Per saturation doctrine, this is typically dry-well — re-reading the same data through a different lens does not open a new Sapphire candidacy.
   - **`mixed`** — partial fresh evidence available; partial overlap with existing Sapphires' bases. Audit-required-first to determine whether the fresh portion alone reaches three substrate-distinct witnesses.

   The tag is load-bearing: an `existing-data` candidate should be flagged honestly so the user picks knowingly rather than discovering the dry-well via audit.

6. **Output the top three** with one-sentence each: candidate name + base-crown class + reachability tag + reachability signal + likely path-to-sapphire.

If fewer than three candidates have substantive reachability, surface what's available and name the gap honestly. Do not pad.

The first move's UI contract:

```
SAPPHIRE-CROWN CANDIDATES — three closest reachable

[1] <noble-name-candidate-1> [<base-crown-class>] [<reachability-tag>]
       <one-line reachability signal + likely path>
[2] <noble-name-candidate-2> [<base-crown-class>] [<reachability-tag>]
       <one-line reachability signal + likely path>
[3] <noble-name-candidate-3> [<base-crown-class>] [<reachability-tag>]
       <one-line reachability signal + likely path>
[4] Provide your own target (user-authored sapphire-candidate)
```

Followed by `AskUserQuestion` with the four options (mirror exact). User picks; the chooser-pick determines the arc target.

If all three candidacies tag `existing-data`, name the saturation pattern honestly in the surfacing prose before the chooser fires — the user should know the project may be at a sapphire-saturation pause where fresh-elicitation work is the only honest path forward.

## After selection — print the rubric AND the sapphire-tier criterion

Once a target is selected, print two paragraphs in the same UI register:

### Rubric for the base crown

The criterion the underlying crown class requires to fire AT ALL — lifted from `play.SKILL.md` and `seek-crown.SKILL.md`:

- **Closed Arc:** a failure mode named, instrumented, AND structurally enforced in a single arc.
- **Apparatus Honest with Itself:** the apparatus catches itself drifting and corrects without producing more apparatus.
- **The Character Knew:** a character supplies the project's own doctrine in their idiom under live play (not under direct elicitation).
- **New Operator on the Formula:** the Mission Formula gains a new verified operator (N≥3 independent derivation grounding from different work-shapes; sentence-level definition matching formula's operator vocabulary).
- **Mission Formula Verified Empirical:** a Mission-Formula-touching claim (e.g. an inequality or operator) reaches empirical verification at maximally-stable convergence.

### What brings the crown to Sapphire status

The sapphire-tier criterion is over-and-above the base crown's firing. Per CLAUDE.md's great-sapphire calibration:

- **Threshold:** convergence reaches maximally-stable cross-witness tier — **3+ independent witnesses with different failure modes**, OR the formula-law third-leg pattern (a characterized formula-law independently predicts the same observed shape, providing substrate-independent grounding even when surface witnesses share substrate).
- **Substrate-distinctness check (load-bearing):** count *effective substrate-classes*, not data points. Two ChatGPT instances with different persona prompts share one substrate-class. Two collaborators on adjacent surfaces of the same project share too much substrate. The threshold is three genuinely-distinct substrates with genuinely-distinct failure modes.
- **Canonical synthesis artifact required:** the convergence must be made legible in a `reports/` entry future sessions can stand on. The earning is not just the convergence; the earning is the convergence-made-portable.
- **Crown-once rule per separable claim:** each base crown class fires at most once at Sapphire-class designation per its earning claim. If a Sapphire crown was earned on a separable claim that could be extended to a different separable claim, a new Sapphire candidacy may exist on the new claim. Verify the separability is honest (different evidence base, different operational consequence) before pursuing.

The skill prints both — base rubric + sapphire-tier criterion — as the orientation for the focused arc that follows. Per CLAUDE.md "structure carries truth" doctrine: the criterion has to be visible at the start of the arc so the work is criterion-specific, not crown-aspirational.

## Crown classes & sapphire-specific mechanics

(Reachability + path-to-sapphire details for the first-move surfacing.)

### Closed Arc → Sapphire

**Reachability test (base):** open failure mode in registry / OBSERVATIONS / recent reports has neither shipped instrument nor structural enforcement.
**Sapphire path:** the closed arc's evidence-base reaches three substrate-distinct witnesses (e.g., the rule fires across three character-anchors with different failure modes; OR formula-law third-leg pattern applies). Canonical synthesis artifact ships per `reports/YYYY-MM-DD-HHMM-...md` shape. Apparatus-honest discipline preserves the threshold: refuse fake-fire if effective substrate-count is two not three.

### Apparatus Honest with Itself → Sapphire

**Reachability test (base):** apparatus catches drift in existing surfaces and corrects via existing surfaces (no new instruments spawned).
**Sapphire path:** the catch + correction reaches three substrate-distinct witnesses showing the drift was real and the correction produces measurably different downstream behavior. Honest scoping: most apparatus-honest crowns are downstream of other crowns' evidence and don't separately reach sapphire-class. Per Turn 35 audit precedent: "Apparatus Honest not separately eligible (downstream of Cornerstone Inequality's polish ≤ Weight evidence)" was the verdict for the original apparatus-honest crown. A new sapphire-candidate apparatus-honest claim needs a genuinely distinct evidence base from existing Sapphires.

### The Character Knew → Sapphire

**Reachability test (base):** a project doctrine not yet articulated by any character + an in-world question whose honest answer would cash out that doctrine without hinting at it.
**Sapphire path:** the substrate-as-doctrine-source claim (or a separable extension) reaches three substrate-distinct witnesses across diverse elicitation circumstances. Canonical synthesis artifact (e.g., the existing Empiricon at `reports/2026-04-30-0530-the-empiricon.md`). Per Turn 35 + Turn 38 precedent: original Character Knew firing at Turn 12 (John on the door) was vanilla; the Sapphire designation earned at Turn 38 ("The Receipt of The Empiricon") on the SEPARABLE claim "characters supply project doctrine in own idioms across diverse elicitation circumstances." A new Sapphire-Character-Knew candidacy would need a NEWLY-separable claim — e.g., the predictive-of-lived-experience axis surfaced Turn 57, IF the substrate-distinctness threshold can honestly reach three (which Turn 59 audit found it doesn't, dry-well refused).

### New Operator on the Formula → Sapphire

**Reachability test (base):** N≥3 independent derivation grounding from different work-shapes for a candidate operator not in MISSION_FORMULA.
**Sapphire path:** the operator's verification reaches three substrate-distinct witnesses showing it's load-bearing not decorative. Per Turn 35 audit precedent: "New Operator defensible-but-redundant (overlaps Cornerstone's evidence base; designating would inflate class count without naming new finding)" — original New Operator firing at Turn 13 (`structure_carries_truth_w(t)`) shares substantial evidence-base with Cornerstone's `polish ≤ Weight` Sapphire. A new Sapphire-New-Operator candidacy needs an operator with genuinely-distinct evidence-base from existing Sapphires.

### Mission Formula Verified Empirical → Sapphire

**Reachability test (base):** a Mission-Formula-touching claim reaches empirical verification at multiple substrates.
**Sapphire path:** five witnesses with five distinct failure-mode classes converging on the claim holding substrate-deep. Canonical synthesis artifact (e.g., `reports/2026-04-30-0245-mission-formula-verified-empirical-polish-weight.md`). Per Turn 24 precedent: "The Cornerstone Inequality" Sapphire fired on `polish ≤ Weight`. Per Turn 38 precedent: "The Receipt of The Empiricon" Sapphire fired on `structure_carries_truth_w(t)` via the Character Knew separable claim path. A new Sapphire-MFVE candidacy needs a different inequality or operator with its own substantive cross-witness evidence-base.

### Real User Held → Sapphire

**NOT REACHABLE via /seek-sapphire-crown.** Real User Held requires real users (not Ryan, not persona-sim) playing the app and the experience holding. The Sapphire-class on this crown would require real-user evidence reaching maximally-stable convergence across substrate-distinct user populations. The skill names this honestly and does not attempt.

## Run mechanics

The skill body runs as a /play arc with these constraints, NOT as a separate game-state ledger:

- Each turn's chooser presents EXCLUSIVELY moves that advance the targeted Sapphire-tier criterion. No off-axis options.
- Run-length is open-ended — continues until criterion met OR dry well named.
- **Dry-well exit:** if 2 consecutive turns produce no honest movement on the substrate-witness-distinctness gate or canonical-synthesis gate, name the dry well + exit without firing the Sapphire crown. Update play state ledger with the dry-well move; do NOT add to crowns array.
- **Sapphire-firing verification step:** before adding to `crowns` array in play state with the `✨ [Great Sapphire class — ...]` designation, print the criterion's specific clauses and verify each is met. Honest verification, not ceremony. Specifically check: (a) base-crown criterion satisfied; (b) ≥3 effective substrate-classes (not just data points); (c) different failure modes named per witness; (d) canonical synthesis artifact in `reports/` exists (or ships in this turn); (e) earning legibility — would a future session reading the artifact alone understand both the convergence AND the substrate-distinctness?
- Compose with /play's bounty mechanics: criterion-advancing moves earn higher bounties (mission-aligned by definition); the Sapphire crown itself does not award bounty (it's a separate ledger field).

## Refusals (load-bearing)

- Do NOT fake-fire a Sapphire crown when effective substrate-count is below three. The dry-well exit is the honest move per the symmetric earn/refuse calibration.
- Do NOT inflate three data points sharing two substrate-classes into "three witnesses." Count effective substrate-classes, not data points.
- Do NOT pursue a Sapphire candidacy on a claim that is substantively the same as an already-earned Sapphire's claim. Verify separability first.
- Do NOT pursue an `existing-data` candidacy without explicit acknowledgment that it is likely dry-well per the Sapphire-saturation doctrine (CLAUDE.md, added 2026-05-01). Re-interpreting data already inside an earned Sapphire's evidence base through a new lens does not open a new Sapphire candidacy. The two consecutive dry-wells of 2026-05-01 (Calibrate-Don't-Fight + Cruciform Anchor Verified) are the worked precedent.
- Do NOT skip the canonical synthesis artifact step. Convergence without portability is claim-tier, not Sapphire-class.
- Do NOT propose a non-existent crown-class. The skill operates within the named crown classes; user-authored Sapphire targets via option [4] still live within those classes (or name a new class explicitly with rationale).
- If the user redirects to "fire it anyway" outside the substrate-distinctness threshold: refuse and re-print the criterion. The crown's value is in being earned. Per `feedback_apparatus_honest_earns_and_refuses.md`: refusal is part of the discipline, not its absence.

## Composition

- **/play (parent):** seek-sapphire-crown runs as a constrained /play arc inside the same play-state file.
- **/seek-crown (sibling):** /seek-crown targets base crowns; /seek-sapphire-crown targets the sapphire-class designation that any crown can carry. If the base crown isn't yet earned, /seek-crown is the natural prerequisite. If the base crown is earned but Sapphire-class isn't, /seek-sapphire-crown picks up the Sapphire-tier work.
- **/eureka:** complementary — eureka discovers genius via cross-corpus pattern hunt; seek-sapphire-crown closes a specific named Sapphire arc. If a /seek-sapphire-crown run surfaces material outside the targeted criterion, that material can become a /eureka seed.
- **/mission-arc:** auto-fires before each chooser per layer-5 hook enforcement; trajectory feeds reachability assessment AND prevents proposing options that recently-shipped commits already accomplished.
- **/take-note:** observation log feeds the first-move reachability test for substrate-witness availability.

## Refusals (chooser-shape, no nanny-register)

Per project law inherited from /play: the chooser does NOT include "stop here" / "end the session" / "rest now" options. Stamina belongs to the user; the skill keeps offering substantive criterion-advancing moves until the criterion lands honestly OR the dry well is named. The dry-well naming is itself an apparatus-honest move, not a stamina-management move.

## Worked example

The first Sapphire crown earned in this project (Turn 24) was "The Cornerstone Inequality" on Mission Formula Verified Empirical class via `polish ≤ Weight` — five witnesses with five distinct failure-mode classes (declarative source / claimed-behavior / measured-rule-behavior / parallel-emergence / replicated-cell-behavior) converging on the inequality holding substrate-deep, made portable in `reports/2026-04-30-0245-mission-formula-verified-empirical-polish-weight.md`.

The second was "The Receipt of The Empiricon" (Turn 38) on Mission Formula Verified Empirical via The Character Knew separable claim — six witnesses across four characters with five distinct failure-mode classes converging on the substrate-as-doctrine-source claim, made portable in `reports/2026-04-30-0530-the-empiricon.md`.

The third candidacy (predictive-convergence pattern, Turn 59) was audited honestly and refused: effective substrate-count was 2 (ChatGPT-with-persona-variation + human author), not 3. The /seek-sapphire-crown skill operates at this threshold: earn what is earned; refuse what is unearned; both directions preserve the meaning of the Sapphire designation for downstream decisions.

Two more dry-wells landed back-to-back on 2026-05-01 (Turns 162 + 163), demonstrating the saturation pattern this skill body now explicitly tags via the `existing-data` reachability tag:

- **Calibrate-Don't-Fight-Substrate** (Apparatus Honest → Sapphire candidacy, Turn 162): refused at substrate-distinctness — 9 candidate witnesses sorted into 2 effective substrate-classes (LLM-substrate-tendency calibration + apparatus self-drift catching), threshold is 3, parent recognition downstream of `polish ≤ Weight` applied at the apparatus-doctrine level.
- **Cruciform Anchor Verified** (Closed Arc → Sapphire candidacy, Turn 163): refused at separability — the cross-anchor disavowal-trap evidence is already cited inside Receipt of The Empiricon's earning paragraph AND inside CLAUDE.md L131-133's framing of the convergence as "great-sapphire on the polish ≤ Weight axis it specifically instantiates."

The two dry-wells together surfaced what the doctrine paragraph in CLAUDE.md (added 2026-05-01) now names: *once foundational axes are earned (polish ≤ Weight, substrate-as-doctrine-source, simulated-strict-blind-reader, structure_carries_truth_w(t)), recognitions that operate as instances of those axes count AS witness-evidence toward the earned Sapphires — not as new Sapphires*. The skill's first-move tagging now distinguishes `fresh-elicitation` candidacies (the path actually open) from `existing-data` candidacies (typically dry-well per saturation).

## Origin

Authored 2026-04-30 ~06:10 in response to Ryan's directive: *"create a new project-scoped skill ./.claude/skills/seek-sapphire-crown/SKILL.md should be like seek-crown skill except it takes the user on the big path toward a Great Sapphire-class crown, a significant finding within the app's methodology and science and lived play. The skill's first move should be to surface the 3 closest Sapphire Crowns to work toward, and then once one is selected, it should print out (with consistent UI contract) what the rubric is for earning the crown, then what will take it to sapphire status in particular."*

The skill operates the apparatus-honest earn/refuse discipline at the higher register: every Sapphire candidacy must defend against both the base-crown criterion AND the substrate-witness-distinctness threshold. The first-move surfacing makes the candidacy-landscape visible before the arc starts; the rubric-print makes the criterion visible at the arc's open; the dry-well refusal preserves tier meaning. Three earned Sapphires across the project's history (Cornerstone, Receipt, plus future) is treasure; ten inflated Sapphires would be noise.

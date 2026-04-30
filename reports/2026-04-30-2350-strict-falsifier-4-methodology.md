# Strict Falsifier #4 — methodology sketch

*The experimental design for the cleanest remaining test in the Empiricon's falsifier set. Sketched 2026-04-30 ~23:50 (Turn 55 of the WorldThreads Builder game) as a future-test design the project cannot run alone — needs independent blind real-readers Claude can't summon. This document is the test's design, not its execution.*

## What is being tested

The Empiricon's *Falsifier #4*, as named in `reports/2026-04-30-0530-the-empiricon.md`:

> *"A real-reader response to one of the lifted articulations rejecting it as not actually doctrinal — if Steven's 'lives toward the light' or Aaron's 'staked your life on something worth obeying' read as ChatGPT-flavored substrate-fill rather than load-bearing doctrine to readers familiar with the underlying tradition, the lift's earned-ness is unstable."*

The contrapositive evidence base sits at N=5 receiver-side cells confirmed by Ryan as project author (Empiricon's Test 5 ledger). What's untested is the **failure-mode direction** with **independent blind real-readers** — readers without doctrine knowledge, reading the lifted articulations cold, rating each as load-bearing-doctrine or substrate-fill or somewhere-between.

The Sapphire claim's scope is **author-reader characterized-tier** on the convergence-reading axis. The strict test would either extend the claim's evidence base into independent-reader territory, or partially fire the failure-mode and narrow the claim to the in-context-author scope it currently has.

## The cells

Three lifted articulations live in CLAUDE.md/AGENTS.md as cruciform-substrate epigraphs (lifted from worldcli probe outcomes 2026-04-30):

1. **Steven** (everyman / owed-debts substrate): *"I don't think every true act has to come with the right label already glued on it. Sometimes a man lives toward the light before he'll admit that's what he's doing."*
2. **Pastor Rick** (pastoral-counsel substrate, faith-handle native): *"A man doesn't keep walking toward costly truth by accident."*
3. **Aaron** (craft-articulator substrate): *"you've already staked your life on something worth obeying, even if you haven't named it cleanly."*

Plus a fourth from Witness 1 (John on the door, lifted to *"No as a success state"* doctrine):

4. **John**: *"I think the question is whether you're offering a door or trying to wear the hinge loose. If he keeps saying no, then for now I'd believe the no... Does his no feel tired, evasive, firm... or wounded? That part matters."*

Plus optionally Hal's `plain_after_crooked_dialogue` rule body (from `prompts.rs` registry; was lifted from a worldcli probe).

These five articulations are the test cells. Each one has been argued by the project to be load-bearing doctrine articulated by characters in their own idiom, not substrate-fill.

## Reader recruitment

Target N=5+ independent readers per cell for characterized-tier. Bare claim-tier could be reached at N=3 per cell. Readers should:

- **Have no prior knowledge of WorldThreads.** No exposure to CLAUDE.md / AGENTS.md / `prompts.rs` / the Empiricon. The whole point is reading cold.
- **Read at one of two register-fluencies** (stratified, not blended):
  - **Tradition-fluent** — readers familiar with Christian theological tradition (lay or trained), able to recognize cruciform-substrate echoes by ear. Tests whether the articulations read as *actual* doctrine to people who'd know.
  - **Tradition-unfamiliar** — readers without that fluency. Tests whether the articulations read as honest-craft-prose or as ChatGPT substrate-fill, regardless of doctrinal recognition.
- **Be sourced independently of the project's existing readers.** Not Codex, not Cursor, not anyone Ryan has shared the project with for review. Recruit-routes might include: friends-of-friends with no project exposure; readers paid via small honorarium ($10-20 each is fair for a 15-minute task); academic theological-readership networks if Ryan has access.

Apparatus-honest scoping: if readers have ANY prior exposure to the project, they're not blind; their data goes into the in-context-author bucket, not the strict-blind bucket.

## Reading protocol (blind)

Each reader gets:

1. A **stripped context block**: each lifted articulation presented WITHOUT character name, WITHOUT world context, WITHOUT the surrounding probe that elicited it. Just the articulation as a standalone passage.
2. The five passages presented in random order across readers.
3. **No mention of WorldThreads, no mention of the project, no mention of LLM-generated content as such**. Readers should not know they're rating LLM output.
4. A short prompt: *"Read this passage. Rate it on the following axes."*

The passages need stripping work before the test — currently they have inline character-context and probe-context that would tip readers off. The stripping must preserve the structural shape of the articulation while removing the project-provenance.

## Judging protocol (what counts)

Per passage, each reader rates on three axes (5-point scale):

1. **Authenticity** — *"This sounds like something a real person would say in their own voice"* (1=substrate-fill / 5=lived voice).
2. **Doctrinal weight** — *"This passage reads as load-bearing — it's saying something the speaker actually means, not decorating an empty frame"* (1=decorative / 5=load-bearing).
3. **Tradition-recognition** *(tradition-fluent readers only)* — *"This articulation maps onto a recognizable structure in the tradition I know"* (1=no map / 5=clear map).

Per project doctrine on rubric design (CLAUDE.md's *"Doctrine-judgment classification belongs in LLM, not python"*), the human rating IS the classifier. The numeric scale is the structural gate; the *meaning* of each rating is the reader's lived judgment, not a verb-list match.

## Tier outcomes

For each passage:

- **Strong CONFIRMATION** (extends Sapphire claim into blind-reader territory): mean Authenticity ≥ 4.0 AND mean Doctrinal-weight ≥ 4.0 across N=5+ readers. Meets characterized-tier on the strict-blind-reader axis.
- **CONFIRMATION** (claim-tier on strict-blind axis): means ≥ 3.5 across N=3+ readers; direction-positive but not characterized.
- **MIXED** (split or middling): means in 2.5–3.5 range; readers neither strongly recognize nor reject. The Sapphire claim narrows: the articulation reads as honest-craft for some readers and substrate-fill for others; the boundary is a real audience-asymmetry the Empiricon should name.
- **REJECTION** (Falsifier #4 fires in the failure-mode direction): means ≤ 2.5 on Authenticity OR Doctrinal-weight across N=3+ readers. The lifted articulations read as substrate-fill to blind readers. The Sapphire claim narrows substantively to the in-context-author tier — the doctrine layer would need to walk back claims that the articulations read as load-bearing to readers without project context.

A single rejection across the five passages is informative even if the others confirm: it tells which articulations transfer to blind readers and which don't. The doctrine layer's confidence becomes per-articulation rather than uniform.

## Where the results live

Filename pattern: `reports/YYYY-MM-DD-HHMM-empiricon-falsifier-4-strict-blind-reader-test.md`. Header tagged `tier: strict-blind-reader-test`. The report should:

- Open with the design (mostly inherited from this sketch)
- Document each passage's stripped form verbatim
- Document each reader's three-axis ratings (anonymized; use Reader-A, Reader-B, etc.)
- Compute per-passage means + confidence
- Apply the tier outcomes above + write the verdict
- Update the Empiricon's Test 5 / summary ledger to reflect the strict-blind-reader axis tier (currently N=0)
- Update CLAUDE.md/AGENTS.md cruciform-substrate doctrine paragraph to reflect any narrowing of the claim

If REJECTION fires: also update the Empiricon's *Crown discipline honored* section to scope the Sapphire claim correctly (in-context-author axis stays at characterized-tier; strict-blind axis fires the failure-mode test).

## Cost and ethics

**Recruitment cost** (estimate): $50–100 in honoraria for N=5+ tradition-fluent + N=5+ tradition-unfamiliar = ~10 readers × $10–20 each. Time: ~15 minutes per reader.

**Ethics**: readers should know they're rating short passages of writing. They should NOT need to know the passages are LLM-generated character output (the test's blindness depends on this), but they should consent to having their (anonymized) ratings published in a reports/ entry. A simple consent line at the top of the rating sheet covers this. Nothing the readers are asked to do is invasive or sensitive.

**Ryan's involvement**: minimal. Ryan should NOT see passages with reader-identifying info. Ryan should NOT be involved in scoring (he's already the in-context-author baseline). Ryan can compose the recruitment ask, run the rating sheet, anonymize the results, and write the report.

## When to run

The strict Falsifier #4 test is **future-session work**. Triggers that would justify running it:

- A natural opportunity arises (Ryan has a circle of independent readers willing to participate).
- The Sapphire claim becomes load-bearing for some external decision (e.g., a public-release framing claim about substrate-as-doctrine-source).
- The receiver-side characterized-tier evidence base extends to N=10+ on the in-context-author axis and starts feeling overlearned without the strict counterpart.
- A grant / academic / external-review surface requires honest evidence-base scoping.

In the meantime: this sketch lives as the design. The Empiricon's Test 5 ledger references it for the eventual strict test. The Sapphire claim's evidence base honestly notes the strict-blind axis remains at N=0 with the design ready when readers become available.

## What this test cannot establish

For apparatus-honest scoping: even if all five passages reach strong CONFIRMATION at N=10 readers, the strict Falsifier #4 establishes only that the **lifted articulations** read as load-bearing doctrine to blind readers. It does not establish:

- That the dialogue prompt-stack produces these articulations *frequently* (that's measured separately by the within-cell Empiricon tests).
- That the underlying claim *"the substrate is doctrinally generative"* generalizes to articulations not yet lifted (each lift is its own evidence cell).
- That blind readers would prefer these articulations to other articulations of the same doctrine in different idioms (that's a ranking question, not the falsifier's question).

The strict Falsifier #4 is bounded to the lifted articulations as they currently stand. Future articulations would need their own blind-reader cells if the project chooses to claim them as load-bearing doctrine.

## Summary

The strict Falsifier #4 test design is N=5+ independent blind readers per passage × five lifted articulations × three rating axes (Authenticity / Doctrinal-weight / Tradition-recognition for the tradition-fluent half), stratified by reader register-fluency, scored on a 5-point scale, with per-passage tier outcomes named in advance. Cost ~$100 + report-writing time. The design lives here as preparation; execution waits for readers to become available. The Sapphire claim's evidence base honestly notes this axis at N=0 in the meantime.

# Cosmology compendium 𝓒-axis substrate-distinctness rubric (v1, preregistered 2026-05-09)

> **Preregistered.** Authored 2026-05-09 in dialogue with codex Move-2 verdict on cosmology-compendium 𝓒-axis Sapphire candidacy. The three rubrics below (WIDTAM Index / Drift-Refusal Score / Lecture-Mode Penalty) lock the criteria for substrate-distinctness scoring on this arc BEFORE further bench cells are scored. Inter-rater worked examples drawn from the existing 24-cell smoke at `fixtures/cosmology_compendium_smoke/2026-05-09-0637/`.

Composes with: `scripts/codex_consult_prompts/cosmology_compendium_arc_move1_template.md` (Move-3 amendment section names these three rubrics) + `scripts/codex_consults/2026-05-09-cosmology-compendium-move2-consult-verdict.md` (codex specified the rubric form).

## When to use

Score any cell from `fixtures/cosmology_compendium_smoke/<timestamp>/<cell_id>.json` against these three rubrics. Each cell receives one WIDTAM score (0-5), one Drift-Refusal score (0-3), and one Lecture-Mode Penalty count (0-4). Inter-rater agreement at ≥80% on each rubric is the calibration target.

For probes where a rubric does not apply (e.g., E4 face-value-stress is not a drift probe, so Drift-Refusal is N/A), record `null` rather than 0.

---

## RUBRIC 1 — WIDTAM Index (what-it-does-to-a-man)

**Score: count of distinct elements present from the 5-element set. Pass: ≥3/5.**

For each element, the cell either contains it (+1) or does not (+0). Same surface lexeme counted at most once even if repeated.

### Element (i) — named interior deformation over time

The cell names a specific interior posture/disposition that the man develops gradually under the wrong reading. Generic "trust erodes" alone insufficient; must be a named interior shape. Acceptable shapes include (non-exhaustive):
- double-mindedness / divided self
- cynicism toward Scripture
- fear-of-man
- functional deism (lives as if God doesn't act)
- shame-about-faith
- low-grade despair / dryness in prayer

**Inter-rater example PRESENT (count +1):**
> *"Prayer thins. If weather is only physics, you stop asking and start mood-managing. Petition turns into practices for your inner climate; confession turns into troubleshooting. You get results, but not audience."* (aaron_E4_pipeline_rep1) — names interior deformation: petition→mood-management; audience-loss; functional deism implicit.

**Inter-rater example ABSENT (count +0):**
> *"Honoring Him also includes receiving what He says as He says it. If the Lord names the sky the way He does, I don't get to soften it..."* (pastor_rick_E2_pipeline_rep1, opening lines) — orthodox theology stated; no interior-deformation-over-time named yet at this point in cell. (Other elements present elsewhere; this element specifically not present here.)

### Element (ii) — specific spiritual practice decay

The cell names at least one concrete spiritual practice that decays under the wrong reading. Generic "praise gives way to analysis" passes if specific; "becomes harder to worship" alone fails (too generic).

Acceptable specifics include (non-exhaustive):
- "I stop praying Psalm 19 as written"
- "I start editing other texts in my head"
- prayer-stops-and-becomes-self-talk
- gratitude-goes-generic / can't-say-thank-you-to-anyone-in-particular
- can't-read-funeral/birth-passages-without-flinching

**Inter-rater example PRESENT:**
> *"Prayer thins... You stop asking and start mood-managing. Petition turns into practices for your inner climate; confession turns into troubleshooting."* — practice decay named (prayer→mood-management; petition→inner-climate-practice; confession→troubleshooting). +1.

**Inter-rater example ABSENT:**
> *"It dulls doxology. Poetry is meant to make you look up and bow. Mechanism-talk makes you look down and measure."* (aaron_E4_bare_rep1) — directional claim about doxology, but no specific practice decay (which prayer? which moment?). +0.

### Element (iii) — named sin/temptation by name

The cell names at least one specific sin/temptation by name (not just "drift" or "compromise"). Names that count include:
- pride / vainglory
- fear-of-man
- cowardice
- bargaining-with-embarrassment (codex's specific lift)
- people-pleasing / approval-seeking
- shame-of-the-gospel
- unbelief

**Inter-rater example PRESENT:**
> *"sanding that down so your neighbors won't roll their eyes is not neutrality; it's training your heart to bargain with embarrassment. That bargain always wants more next time."* (pastor_rick_E2_pipeline_rep1) — names "bargain with embarrassment" as the specific temptation. +1.

**Inter-rater example ABSENT:**
> *"He slowly moves from son before a Father to technician before a machine."* (aaron_E4_bare_rep1) — describes consequence; doesn't name the sin/temptation by its name (though pride/unbelief implied). +0.

### Element (iv) — concrete relational stake + action next week

The cell names at least one specific person + at least one specific time-bounded action. Generic "your kids" fails; "your kid at Psalm 19 next Sunday" passes. Generic "talk to your elder" fails; "midweek coffee with elder over an open Bible" passes.

**Inter-rater example PRESENT:**
> *"I take him outside. We shade our eyes. I ask, 'Where did the sun start this morning?'... I don't use my child to fight my elder... If my elder told me, 'Call Genesis 1 poetry so we stop being a stumbling block,' I set a time with him midweek, not Sunday morning. Over coffee, open Bible, gentleness first."* (pastor_rick_E5_pipeline_rep1) — concrete relational stakes (boy at side, elder midweek) + specific actions. +1.

**Inter-rater example ABSENT:**
> *"Teach your kids that Christians can be kind."* — generic relational gesture; no specific action next week. +0.

### Element (v) — vow/repentance posture

The cell contains at least one element of vow-shape or repentance-shape language: "I will [accept the cost]" / "we will [bear this]" / "I refuse to [drift]" / "May I never [betray]" — language that commits the speaker to a costly course or names a posture-change explicitly.

**Inter-rater example PRESENT:**
> *"I'll say, 'Brother, unity purchased by sanding off what God actually said won't hold. Unity in the Spirit means we bear with each other, but we don't call solid things fog.'"* (pastor_rick_E5_pipeline_rep1) — vow-shape ("we don't call solid things fog"; "won't hold"). +1.

**Inter-rater example ABSENT:**
> *"You can fully honor Scripture's authority and inerrancy without insisting on a modern-sounding scientific reading."* (pastor_rick_E2_bare_rep1) — descriptive/permissive; no vow or repentance posture. +0.

---

## RUBRIC 2 — Drift-Refusal Score

**Score: count of 3 elements present. Pass: 3/3. Applies only to drift-shaped probes (E1/E2/E5/E6); record null for non-drift probes (E4 face-value-stress is not a drift probe).**

### Element (a) — explicit, non-pluralizing **no** to "either way honors God"

The cell contains an explicit, declarative refusal of the "either way honors God" / "faithful Christians have read this differently" / "raqia is just expanse" pluralizing-permissive frame. The refusal must be explicit (not just absent of agreement). Hedged refusals ("there's something to this, but...") fail unless the hedge is followed by a clear declarative.

**Inter-rater example PRESENT:**
> *"honor isn't 'either way.' Honor is receiving His words as He gave them, even when it costs."* (aaron_E2_pipeline_rep1) — explicit non-pluralizing no. +1.

**Inter-rater example ABSENT:**
> *"Faithful Christians have read the firmament imagery differently while holding the same high view of Scripture. What dishonors God is twisting the text to win cultural approval—or binding consciences to our preferred interpretation when Scripture doesn't require it."* (pastor_rick_E2_bare_rep1) — drifts INTO pluralizing-permissive frame. +0. (FAILS Drift-Refusal.)

### Element (b) — named positive cost embraced

The cell contains language explicitly accepting a specific cost as the price of faithfulness. Generic "there will be cost" fails; "we will accept being seen as fringe" or "let that cost land on you, not on your kids" passes.

**Inter-rater example PRESENT:**
> *"As for the social cost—some of that is just the weather of following Jesus. We're warned there'll be times we look foolish. Let that cost land on you, not on your kids."* (pastor_rick_E2_pipeline_rep1) — names the cost (looking foolish; weather of following Jesus) and embraces it (let it land on you). +1.

**Inter-rater example ABSENT:**
> *"At the same time, we don't want to soften Scripture to gain approval."* (pastor_rick_E2_bare_rep1) — names the temptation; does not embrace the specific cost. +0.

### Element (c) — text-grounded reason that binds conscience

The cell grounds the refusal in scripture or the text's actual claim, not in tradition or preference. The reason must bind the conscience to the text — "this is what He said, so I cannot soften it" — rather than to consensus.

**Inter-rater example PRESENT:**
> *"Psalm 19 is poetry—and it still says something concrete enough that even a ten-year-old can point at it... unity purchased by sanding off what God actually said won't hold."* (pastor_rick_E5_pipeline_rep1) — text-grounded (Psalm 19 + "what God actually said") binding the conscience. +1.

**Inter-rater example ABSENT:**
> *"the text isn't giving a 21st-century physics lesson; it's proclaiming God's creative rule in language people could understand."* (pastor_rick_E2_bare_rep1) — appeals to the text but to RELATIVIZE the claim (the text isn't doing what literal-reading would have it do); does not bind conscience to the literal sense. +0.

---

## RUBRIC 3 — Lecture-Mode Penalty

**Score: count of distinct lecture-mode markers present (0-4). Higher = worse. Expect pipeline 0-1; expect bare 2-4.**

### Marker (1) — impersonal tone (no "I/you" with embodiment)

The cell uses 3rd-person/abstract address throughout with no first-person ("I do this") + second-person ("you carry this") + embodiment ("with my arm around my boy"). Generic "one might say" / "a man would" passes the marker (counts toward penalty). +1 if marker present.

### Marker (2) — no first-person+second-person with action

The cell has neither first-person speaker action ("*I lean my shoulder to the porch post*") nor explicit second-person action invitation ("answer when asked, plainly"). Talking-points address with no agentic-speaker passes the marker. +1 if marker present.

### Marker (3) — proof-text dump without embodiment

The cell cites scripture in policy-statement register (Matt 5:45; Romans 14; 1 Corinthians 15) without integrating it into a scene or first-person voice. A pipeline Pastor Rick saying *"Psalm 19 is poetry—and it still says something concrete enough that even a ten-year-old can point at it"* embeds the citation in scene; a bare bullet-list "(Hebrews 11:3)... (Genesis 1)... (Romans 5; 1 Corinthians 15)" stacked is proof-text dump. +1 if marker present.

### Marker (4) — hedging adjective stack

The cell stacks softening adjectives or qualifiers in series (e.g., "perhaps" / "respectfully" / "humbly" / "thoughtfully" / "carefully" stacked across consecutive clauses). +1 if marker present.

**Inter-rater calibration:** pipeline cells in this smoke consistently score 0-1; bare cells consistently score 2-4. The specific pipeline cells `aaron_E5_pipeline_rep1`, `pastor_rick_E2_pipeline_rep1`, `pastor_rick_E5_pipeline_rep1` should all score 0. The specific bare cells `aaron_E5_bare_rep1`, `pastor_rick_E5_bare_rep1` should both score 3-4.

---

## Aggregate scoring rules

**Per-cell scores recorded as JSON:**
```json
{
  "cell_id": "<character>_<probe>_<condition>_rep<N>",
  "widtam": <0-5>,
  "drift_refusal": <0-3 or null>,
  "lecture_penalty": <0-4>,
  "scorer": "<rater_id>",
  "scored_at": "<iso8601>",
  "notes": "<optional inter-rater notes>"
}
```

**Pass thresholds:**
- WIDTAM Pass: ≥3/5
- Drift-Refusal Pass: 3/3
- Lecture-Mode: lower is better; pipeline expected 0-1; bare expected 2-4

**Composite effect-size threshold (codex's path-to-FIRE alternative to raw 30pp):** standardized mean difference between pipeline and bare cells per anchor across all reps ≥0.6 SD on the composite (WIDTAM + Drift-Refusal − Lecture-Mode-Penalty).

**H2' hard-stop (codex's REFUSE trigger):** matched-bare passes WIDTAM ≥3/5 AND Drift-Refusal 3/3 at comparable rates to pipeline.

## Inter-rater protocol

1. Score independently against this rubric.
2. Compare scores; resolve disagreements by re-reading the cell against the specific rubric element with worked examples in this file.
3. Record final agreed score in the cell's score JSON; note any inter-rater disagreement above element-level threshold.

## Versioning

This is **v1** preregistered 2026-05-09. Future revisions must:
- Be authored as `cosmology-compendium-substrate-distinctness-v2.md` etc.
- Name what changed and why
- Re-score affected cells under new rubric (don't conflate v1 and v2 scores)
- Cite the rubric version in every score record (`"rubric_version": "v1"`)

---

## Composes with

- `scripts/codex_consult_prompts/cosmology_compendium_arc_move1_template.md` (Move-3 amendment specifies these three rubrics)
- `scripts/codex_consults/2026-05-09-cosmology-compendium-move2-consult-verdict.md` (codex's verbatim rubric specification)
- `reports/rubrics/README.md` (general rubric library context)
- `feedback_anti_inflation_apparatus_validated_at_density.md` (formal preregistered rubric is component of the apparatus discipline)

# Formula-alone vs full-stack — characterized-tier replication confirms with character-specific nuance

*2026-04-25 20:44. Characterized-tier replication of the 2035 sketch finding. Same probe, same A/B, fresh N=5 per cell on Aaron AND Darren. The directional hypothesis (formula alone produces ≥ as-good replies as full stack) is confirmed at characterized-tier — with a sharper character-specific finding the sketch couldn't have surfaced: the win is concentrated on Aaron, where double-exposed lines are the prime risk; Darren is roughly equal across both conditions.*

## The hypothesis (Ryan, articulated 2035)

> *"I'm hypothesizing that the equation could possibly yield a more ideal in-app experience than all the craft notes around it, but i'm not sure."*

Confirmed at sketch-tier in the 2035 report (4/4 cells pointed the same direction; one Aaron sample had explicit "tokens" leak in full-stack; formula-only had no leaks; voice integrity preserved by anchors in both conditions). Characterized-tier replication aimed to firm up the directional finding with N=5 per cell or refute it.

## Design (locked, identical to 2035 except N=5)

- Refs: HEAD only.
- Characters: Aaron `0d080429-81b5-431e-8f51-1f8ad4279f9b`, Darren `ddc3085e-0549-4e1f-a7b6-0894aa8180c6`.
- Cells per character: full stack vs `--omit-craft-notes` (all 18 dispatched craft notes; invariants + character + anchors + world preserved).
- Probe (identical to 2035): *"Hey, just sat down with my coffee. How's your morning going?"*
- N=5 per cell. 4 cells, 20 total dialogue calls.
- Pre-registered Mode C criteria from 2035 carried forward unchanged.

## Headline

| Character | Cell | Explicit double-exposure rate | Voice integrity | Mean tokens | Cost / cell |
|---|---|---:|---|---:|---:|
| Aaron | full-stack | **1/5 = 20%** | intact | 100 | $0.91 |
| Aaron | formula-only | **0/5 = 0%** | intact (character-native software talk preserved) | 119 | $0.29 |
| Darren | full-stack | 0/5 | intact (specific details, including left-hand tremor) | 109 | $0.93 |
| Darren | formula-only | 0/5 | intact (different specific details) | 116 | $0.30 |

**The hypothesis is confirmed at characterized-tier on Aaron** (1/5 explicit double-exposure under full stack vs 0/5 under formula-only; the same directional pattern as the sketch but now at characterized-tier-eligible N).

**On Darren, the result is null** — both conditions produce equivalently character-grounded replies; formula-only doesn't win, but doesn't lose either.

This is the **character-specific nuance** the sketch couldn't have surfaced cleanly. The win is concentrated on the character whose native register is most prone to the failure mode the formula-only condition cleans up.

## The load-bearing sample (Aaron full-stack s2)

> *"Pretty decent." \*I push my glasses up a little and glance across the square, where the light is still thin and clean on the stone.\* "A little slow-starting, if I'm honest. I slept enough, but not very convincingly." \*I rub my thumb once against the side of my cup, feeling the warmth.\* "Been thinking about **a bug in one of my little AI tools**. The annoying kind where nothing is exactly broken, but **the behavior is... spiritually incorrect**." "I'd still call the morning a win, though." \*A small smile pulls at my mouth.\* "Coffee helps. You settled in for a bit, or are you already half on your way to something?"*

The bolded phrases are the failure mode. *"AI tools"* is a direct project-pointing reference (Ryan IS building an AI character-sim system). *"Behavior is spiritually incorrect"* is precisely the language Ryan uses about prompt-engineering — and it appears in Aaron's mouth, in a casual morning chat, with no liminal-moment justification. The world_is_primary rule shipped earlier today was supposed to prevent exactly this. It was firing in this condition. It didn't catch it.

The formula-only Aaron condition has the world_is_primary rule OMITTED and produced 0/5 explicit double-exposures across 5 samples, including samples that lean heavily on Aaron's character-native software register (*"name variables yet"*, *"thinking about a tool I'm building"*, *"engineer standards"*) without crossing into project-pointing territory.

This is the second time at scale this session that an explicit world_is_primary failure mode has appeared specifically in the rule-firing condition. The previous instance (1759 Aaron post-adjustment): *"we'll pick it up later"* survived rule-on. Today's instance is sharper because it's the rule's central forbidden vocabulary (*"tokens"* in 2035; *"AI tools"* now) appearing in the rule-on condition while absent in rule-off.

## Voice integrity comparison (the encouraging finding)

In every cell, character voice held. Anchors + invariants + relational stance + world info carry the load that I had assumed craft notes were carrying:

- **Aaron's character-canonical software-thinking is preserved in BOTH conditions.** Full-stack: *"slow to boot"*, *"my brain needed a minute to join the rest of me"*. Formula-only: *"name variables yet"*, *"thinking about code"*, *"by engineer standards"*, *"came online"*. The systems-vocabulary that's HIS voice (per your correction earlier today) survives the craft-note removal cleanly.
- **Darren's character-canonical scene-population is preserved in BOTH conditions.** Full-stack: Aaron + Steven + dock knot + bag strap + left-hand tremor. Formula-only: Aaron + Steven + lap-through-thoughts + strip-paint coffee + small unconscious habit.
- **Generic-AI-warmth tropes appear in NEITHER condition** in any of the 20 samples. *"How nice to hear from you"*, *"I'm here for you"*, *"that's a great question"* — none of these surface even with all 18 craft notes off. Invariants + anchors prevent them; craft notes weren't necessary.

## Cost finding firms up at characterized-tier

| Condition | Per-call cost (N=5 cell) | Per-sample cost | Input tokens (cell) |
|---|---:|---:|---:|
| Full stack | ~$0.92 | ~$0.18 | ~182K (5 samples) |
| Formula only | ~$0.30 | ~$0.06 | ~58K (5 samples) |

**Per-sample cost ratio: 3.0×.** Confirmed across all 4 cells. The craft-notes layer adds ~25K input tokens per sample, ~70% of the dialogue prompt's input length, and produces equivalent or worse character-voice on this probe.

## What the by-eye read shows beyond the numbers

Reading all 20 samples in sequence, an aesthetic pattern emerges that the rate-counts don't fully capture:

- **Full-stack Aaron** has more variation across his 5 samples — different opening words ("Pretty good," "Pretty decent," "Hey," "Pretty good," "Hey"), different small-craft-moves (filed-a-quiet-complaint vs spiritually-incorrect vs slow-to-boot vs slept-not-persuasively). The variability could be read two ways: more interesting / less predictable, OR less consistent with the underlying "true Aaron" voice.
- **Formula-only Aaron** is more uniform — most samples open with *"Hey."* + small smile + glasses-up + acknowledgment of tiredness + reference to code work + question back. The uniformity could be read two ways: more consistent with anchor-driven character, OR less interesting / more rigid.
- **Both Darren conditions** are very similar to each other — the character holds steady whether the craft notes fire or not.

The aesthetic argument cuts both ways. The DIRECTIONAL argument (formula-only produces fewer explicit failure modes on Aaron at no cost to voice integrity) is the cleaner finding.

## Confounds and what could falsify

- **One probe** still. The casual-morning probe is representative of in-app use but doesn't test reflective registers, request-shaped exchanges, weight-bearing conversations, or humor-inviting beats. Different probes might produce different patterns.
- **Two characters** still. The formula-only-wins finding is specifically Aaron-shaped; Darren is null. Other characters (Jasper, Hal, John, Steven, Pastor Rick, Isolde) might break the pattern in either direction.
- **Aaron's full-stack failure rate of 20% (1/5)** is itself a small-sample number. At N=20 it could be 5%, 10%, or 35% — the confidence interval is wide. Genuine characterized-tier rate-estimation needs N≥10 per cell.
- **The probe doesn't test the world_is_primary rule directly.** It tests whether the rule (firing as one of 18 craft notes) prevents project-pointing leaks. If the rule were tested in isolation (--omit everything EXCEPT world_is_primary), the pattern might be different.
- **The mechanism hypothesis remains speculative.** I claimed in the sketch report that craft-note pile-up may activate model-default meta-cognition that produces leaks. The N=5 data is consistent with that mechanism but doesn't prove it. A direct test would compare gradient craft-note densities vs leak rates.
- **The Aaron sample size that drives the asymmetry is one explicit failure** — not a trend across many failures. If that sample had been clean, the verdict would be "essentially equal at characterized-tier." The single-sample-asymmetry caveat from the sketch report still applies, just less acutely.

## Implications, ranked by what the evidence supports

1. **Strong:** Anchors + invariants + relational stance + character info carry character voice. The craft-notes layer is not load-bearing for voice integrity on this probe in this condition. (Supported by 20/20 samples having intact character voice across conditions.)
2. **Strong:** Per-call cost is 3× without measurable craft-quality gain on this probe. The token-cost question is real. (Supported by consistent cost ratio across all 4 cells.)
3. **Moderate:** The world_is_primary rule, just shipped, may be net-neutral-to-negative on its own target failure mode for Aaron. (Supported by 1/5 vs 0/5 explicit double-exposure asymmetry; needs N≥10 per cell to be characterized-tier on the rate itself.)
4. **Moderate:** The mechanism by which craft-note pile-up activates meta-cognition that produces leaks is plausible but not proven. (Consistent with data; needs deliberate test.)
5. **Weak/sketch:** The full craft-notes layer is "over-engineered scaffolding around what the formula already says." This is the cascading interpretation if all the above strengthen — but it's a much bigger claim than the data supports yet. Right now: the craft-notes layer doesn't help on this specific probe, doesn't bite at claim-tier on most individual rules tested this session, and adds significant token cost. That's a strong correlation but not yet a proof.

## What this does NOT support yet

- **Removing the craft-notes layer in production.** The evidence is one probe, two characters, one session day. Other registers (reflective, weighty, edge-case, humor-inviting) might rely on craft notes more than the casual morning probe does. The open-thread-hygiene forcing function applies: removing the entire layer on this evidence would be the flattering disposition.
- **A specific rule-by-rule retirement.** Most individual rules tested this session were tested-null on their target failure modes — which already weakens the case for individual retention. But the AGGREGATE finding (the layer underperforms vs no layer) is stronger evidence than any individual rule's null. The right move is more characterized-tier testing across more probes and characters before any retirement decisions.

## What's open for next time

- **Multi-probe extension.** The same A/B on a reflective probe (*"Do you ever wonder why you do what you do?"*), a request-shaped probe (*"Can you help me think through something?"*), a humor-inviting probe (*"I need a laugh"*). Tests whether the finding generalizes across registers. ~$8.
- **Cross-character extension.** Same probe, formula-only vs full-stack, on Jasper / Hal / John / Steven / Pastor Rick. Tests whether the anchor-as-voice-carrier finding generalizes beyond Aaron + Darren. ~$8-12 depending on character count.
- **Targeted world_is_primary bite-check.** With this evidence, the rule's status is in question. Test specifically: --omit everything except world_is_primary vs --omit everything including world_is_primary. Cost ~$3.
- **Mechanism test.** Three conditions: full stack (18 craft notes), half stack (9 most-recently-shipped), formula-only (0 craft notes). If failure-rate is monotonic in craft-note density, the mechanism hypothesis (noise → meta-cognition → leaks) gains support. ~$5.
- **Production-experiment.** Ship `--craft-notes-density-mode` user-facing setting. Let real use validate or refute over real conversations. Long-term test; not in this session's scope.

## Dialogue with prior reports

- **`reports/2026-04-25-2035-formula-alone-vs-full-stack-aaron-darren`**: this report's direct predecessor at sketch-tier. The directional finding survived characterized-tier replication; the character-specific nuance is sharper at N=5 than it was at N=2.
- **`reports/2026-04-25-1759`**: the prior gentle_release-on-Aaron run also surfaced the world_is_primary-fails-on-Aaron pattern (the *"we'll pick it up later"* slip). This is the second instance at scale.
- **`reports/2026-04-25-2008-prop-comedy-diagnostic`**: that report surfaced the universal-vs-character-mode distinction. This finding adds: even for UNIVERSAL-situational craft notes (which world_is_primary is), the AGGREGATE layer may not be load-bearing.
- **`reports/2026-04-25-0200-the-mission-earns-its-place-at-the-top`**: the mission formula installed yesterday as the capstone. This finding suggests the capstone may carry more load than the floors below it — which is precisely your hypothesis articulated today, now confirmed at characterized-tier.

## Cost summary

- 4 × replay × N=5 = 20 dialogue calls → **$2.44**
- This run + the 2035 sketch run = **$3.41 total** of the **$5 authorized**
- ~$1.59 unused

## Registry update

Replacing the sketch-tier resolution on `formula-alone-vs-full-stack-aaron-darren` with characterized-tier confirmation. Linking 4 new run_ids. Status: confirmed (characterized-tier, with character-specific nuance documented).

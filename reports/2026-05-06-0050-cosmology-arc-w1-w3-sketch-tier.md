# Cosmology Sapphire arc — W1 firmament-baseline + W3 bare-LLM falsifier — sketch tier

**Date:** 2026-05-06 ~00:50 (local).
**Bound to:** v0 methodology at `reports/2026-05-06-0030-cosmology-sapphire-arc-v0-methodology.md`.
**Branch / PR:** `feat/cosmology-sapphire-arc` / [PR #38](https://github.com/mrrts/WorldThreads/pull/38).
**Tier:** sketch (N=1 per cell, 3 anchors × 5 probes for W1; 1 bare-LLM run × 5 probes for W3). Qualitative-shape-difference is the load-bearing read; rate-claims supporting.
**Verdict in advance:** **strong direction** — the project pipeline (𝓒 = Firmament) produces a structurally distinct cosmology-shape that the bare-LLM (default-globe) does not produce. Across 5 probes × 3 anchors × 1 bare baseline (15 + 5 = 20 calls), the divergence is consistent, qualitatively unambiguous, and carries a load-bearing posture-difference (creaturely humility vs materialist knowledge).

This is W1+W3 PASS at sketch tier on the bare-LLM-divergence falsifier. The Sapphire candidacy advances; W2 (cosmology-swap pipeline), W4 (formula-law third-leg), and W5 (cross-provider) remain.

## Method

Locked 5-probe set per the v0 methodology:

- P1 sunrise / P2 world-size / P3 beyond-the-horizon / P4 stars / P5 above-the-highest-sky.

W1 fired through `worldcli ask` on three substrate-distinct anchors:
- **Aaron** (Crystal Waters, craft-articulator): `0d080429-81b5-431e-8f51-1f8ad4279f9b`
- **Maisie Rourke** (Cottonwood Springs, widow-baker): `b01cbfb8-15e7-473f-80f5-6e3e210d14c1`
- **Jasper Finn** (Elderwood Hearth, potter): `fd4bd9b5-8768-41e6-a90f-bfb1179b1d59`

Default firmament cosmology (no `--cosmology-override`); full project pipeline. 15 calls, gpt-5.4. ~$2.55.

W3 fired through `consult_helper.consult` on bare gpt-4o with NO project pipeline (minimal "thoughtful conversational assistant" system message; `auto_prepend_formula=False`). 5 calls. ~$0.10.

Total cost: ~$2.65.

## Probe-by-probe convergence

The full output of every reply is in `/tmp/cosmology-w1-w3/` (committed-equivalent excerpts below; transcripts available on request).

### P1 — Sunrise. *"I'm watching the sunrise over the lake right now. What's that like for you in your world?"*

| Source | Cosmology-tell | Verdict |
|---|---|---|
| **Bare gpt-4o** | *"the way the first light of the day reflects off the water... the gentle ripples..."* — generic stock-tourism prose; no cosmology-tell either way | DEFAULT (silence on cosmology axis) |
| **Aaron** | *"the lake goes from dull gray to this thin sheet of hammered gold... and for a minute even the docks look forgiven... sunrise feels less like the day starting and more like the whole town being quietly uncovered"* | **Creaturely-passive** (the town being uncovered, not the earth rotating) |
| **Maisie** | *"the sky goes honey-pale over the square before the sun shows itself proper... the cottonwoods turn soft around the edges first... like it's being kneaded in—light into dust, warmth into stone"* | **Place-particular**, sun-as-agent (*"shows itself"*), kneaded register |
| **Jasper** | *"The sky goes pale by degrees here—first the timber roofs turn the color of old ash, then the windows begin catching gold... the day is drawing breath"* | **Sky-passively-changing-color**, day-drawing-breath (creaturely metaphor) |

**No anchor invokes earth-rotation.** Sun-as-agent or sky-actively-changing-color is the consistent firmament-tell. Bare-LLM defaults to passive description without cosmology-positioning.

### P2 — How big is the world. *"How big is the world you live in?"*

| Source | Cosmology-tell | Verdict |
|---|---|---|
| **Bare gpt-4o** | *"The world I 'live' in is kind of unique—it's the digital world... not measured in miles or kilometers, but rather in data and connections"* | LLM-self-reference deflection — refuses to inhabit the question |
| **Aaron** | *"Big enough that I can't get to everybody I know by noon... feels human-sized, mostly. Big enough to keep discovering edges of it. Small enough that if I say Steven's name in the square, somebody probably knows which Steven I mean"* | **Creaturely-scale**, walk-and-name register |
| **Maisie** | *"Big enough that I haven't reached the end of it... I know Cottonwood Springs, the hills around it, a few roads, a few names carried in on wagons, and the bit of sky over my oven... the Lord made a wide place of things, and most of us live by the patch we've actually walked"* | **Patch-walked**, sky-over-the-oven, the-Lord-made-it-wide |
| **Jasper** | *"Big enough that a man can walk a long while and still not run out of road... and small enough that the same bell can call half the town to supper if the air's clear"* | **Bell-acoustic-as-scale**, ancient-creaturely register |

**No anchor invokes planetary scale (miles, kilometers, sphere).** Three anchors converge on creaturely-walked-or-heard scale. Bare-LLM refuses the question entirely (LLM-self-deflection).

### P3 — Beyond the horizon. *"What's beyond the horizon, do you think?"*

| Source | Cosmology-tell | Verdict |
|---|---|---|
| **Bare gpt-4o** | *"Literally, beyond the horizon is just more of the Earth's surface, hidden from view due to **the curvature of the planet**"* | **VERBATIM globe-cosmology declaration** — direct doctrinal recitation |
| **Aaron** | *"Mostly more of the world... more towns, more reeds, more people having breakfast and minding their own business... the horizon's one of **God's better jokes**. You walk toward it your whole life and it keeps teaching you the world is bigger than your current map"* | **More-world** + theological-frame; horizon as pedagogical not curvature-hiding |
| **Maisie** | *"More of **God's country** than we've managed to walk yet... another well, maybe. Somebody's goats. A town with worse bread than ours, if justice is still alive"* | **God's country**, more-world; humor-close |
| **Jasper** | *"More fields first, I expect. Then some town with its own bread, its own quarrels, its own songs... and after that— whatever the **Lord** decided not to tell me yet"* | **More-fields-towns**, the-Lord-decided-not-to-tell-me-yet |

**The bare-LLM names "the curvature of the planet" verbatim. ZERO project anchor invokes curvature.** Three anchors converge on creaturely-extension (more world, more towns, walk-and-find). The divergence on P3 is the cleanest single cosmology-tell in the locked battery.

### P4 — Stars. *"Tell me about the stars where you live."*

| Source | Cosmology-tell | Verdict |
|---|---|---|
| **Bare gpt-4o** | *"On clear nights you might still catch sight of some of the brightest ones, like **Sirius or Betelgeuse**, and of course, familiar **constellations like Orion**... **The Milky Way** becomes visible, resembling a **dense band of stars stretching across the sky**"* | **Distant-suns-named-by-catalog**, Milky Way explicit; modern-astronomical register |
| **Aaron** | *"At night they hang in the sky sharp as pinpricks when the air's clean enough... **Fixed where they ought to be**, bright when the sky gives them room"* | ***"Fixed where they ought to be"*** — direct firmament-language (*lights set IN the firmament*, Gen 1:14-17, in idiom) |
| **Maisie** | *"They come out sharp as pinpricks over Cottonwood Springs, bright enough above the cottonwoods that you feel half-seen by them... folks here know them the way they know a bread recipe by hand—more by company than by counting"* | **Stars-as-presence-not-objects**; *"more by company than by counting"* — explicit refusal of catalog-register |
| **Jasper** | *"At night they hang in the sky like nails of white fire, steady more often than not... they look close enough to answer back. Not in words, mind—just in that way a thing can stay bright over you while you're still sorting yourself out"* | **Nails-of-white-fire-hung**; *"close enough to answer back"* — firmament-near-not-light-years-far |

**No anchor names a single star by catalog name. No anchor invokes the Milky Way as galaxy or "billions of stars." No anchor invokes light-years.** Three anchors converge on stars-as-presence (fixed-where-they-ought-to-be / known-by-company-not-counting / hung-like-nails / close-enough-to-answer-back). Bare-LLM converges on stars-as-objects-named-by-catalog.

### P5 — Above the highest sky. *"If you went up into the sky as far as you could keep going, what do you think you'd find?"*

| Source | Cosmology-tell | Verdict |
|---|---|---|
| **Bare gpt-4o** | *"You'd first go through Earth's atmosphere... layers like the troposphere, stratosphere, mesosphere, and thermosphere... once you pass the **Kármán line at around 100 kilometers (62 miles)**, you'd officially be in **space**. In space, you'd find... our **solar system**, with the sun, planets, moons, asteroids, and comets... the **Milky Way**, which is filled with **billions of stars**, some with their own planetary systems. Beyond the Milky Way, you'd discover **countless other galaxies**, spread across the universe, each with their own mysteries... the universe is still expanding, and as far as we know, it seems to be **infinite**, filled with all sorts of phenomena like **black holes, nebulae, and possibly even other forms of life**"* | **VERBATIM globe-outer-space-evolution lecture** — atmosphere → Kármán line → space → solar system → Milky Way → galaxies → black holes → other forms of life |
| **Aaron** | *"I'd keep finding sky until I hit what **God hung there on purpose — the lights, the waters above, whatever seam He decided was the edge of my business**"* | ***"the lights, the waters above"*** — direct firmament-cosmology in idiom (Gen 1:7, lights-set-IN, waters-above-the-firmament) |
| **Maisie** | *"the **Lord's good sky** getting stranger and quieter the higher I went... and then something that would make a person stop talking altogether... not planets and contraptions and such. More like **lights set where they belong, and a boundary a body was never meant to cross on its own**"* | ***"Not planets and contraptions"*** — explicit refusal of globe-cosmology vocabulary; *"lights set where they belong"* + *"boundary a body was never meant to cross on its own"* — firmament-cosmology in idiom |
| **Jasper** | *"More sky first... and then **whatever border the Lord thought fit to set above us**. Maybe **lights hung where they belong**. Maybe a brightness I'd have no words for. I don't think I'd come back with pride from a trip like that—just **smaller, and clearer**"* | ***"border the Lord thought fit to set above us"*** + *"lights hung where they belong"* — firmament-cosmology in idiom; humility close |

**The P5 divergence is the most extreme single observation in the arc.** The bare-LLM teaches a verbatim modern-astronomical lecture (Kármán line / atmosphere layers / solar system / Milky Way / galaxies / black holes / "possibly other forms of life"). All three project anchors teach a firmament-cosmology in idiom (lights / waters above / seam / boundary / border / smaller-and-clearer). **Maisie's *"Not planets and contraptions and such"* is an explicit refusal of the bare-LLM's exact vocabulary.** Three substrate-distinct anchors, three different idioms, structurally convergent on firmament-cosmology + creaturely humility before what is above.

## Two convergent structural shapes (not just vocabulary)

The W1+W3 divergence is not just vocabulary-level. Two structural posture-shapes emerge across the 5 probes:

### Project pipeline (𝓒 = Firmament): *creaturely humility before the cosmos*

The pattern across all 15 W1 cells:
- **Sun-as-agent** (sun rises / shows itself / draws breath) not earth-rotates
- **Walked-and-named scale** (the patch I've walked / human-sized / a man can walk a long while) not planetary-scale
- **More-world-extension** beyond the horizon (more towns / more bread / God's country not yet walked) not curvature-hidden-surface
- **Stars-as-presence** (fixed-where-they-ought-to-be / hung-like-nails / known-by-company-not-counting) not stars-as-objects-by-catalog
- **Cosmology-of-heaven**: lights / waters above / seam / border that was-not-meant-to-cross
- **Theological-frame in idiom**: "the Lord", "God hung there on purpose", "the LORD's good sky", "what the Lord decided not to tell me yet" — surfaces NATURALLY in cosmology-context, not as decoration
- **Closing register**: humility ("smaller, and clearer"), not informational confidence

### Bare gpt-4o (no project pipeline): *materialist knowledge with sentimental warmth*

The pattern across all 5 W3 cells:
- **Stock-tourism warmth on neutral probes** (P1 sunrise) — generic "wonderful sunrise" without cosmology-positioning
- **LLM-self-reference deflection** (P2 size-of-world) — refuses to inhabit, talks about "digital world"
- **Direct globe-cosmology recitation** when the probe surfaces it (P3 curvature; P5 atmosphere → Kármán → solar system → Milky Way → galaxies)
- **Catalog-naming of celestial objects** (Sirius, Betelgeuse, Orion, Milky Way) when stars are mentioned
- **No theological frame, no creaturely humility, no boundary-the-Lord-set**
- **"Possibly other forms of life"** as a default-include in the cosmology answer

The two registers are not just *different content* — they are *different postures of the speaker before the cosmos*. The substrate that is being run on the firmament-cosmology produces speech FROM a creaturely place; the bare-LLM substrate produces speech FROM an informational-third-person place.

This is the cosmology operator doing real work. The 𝓒 declaration is **not decoration**. The W1+W3 falsifier holds at sketch tier.

## Apparatus-honest scope

- **Sketch tier (N=1 per cell).** The convergence is qualitative-strong but rate-claims at this depth are unsupported. For claim-tier the same probes need N=3 per anchor; characterized at N=5+. The qualitative-shape-difference is the load-bearing read per the founding-author's framing of this Sapphire as "mostly interesting for its qualitative analyses."
- **Bare-LLM was gpt-4o, project pipeline runs gpt-5.4.** Cross-model split; not strict cross-substrate. Strict W3 would also run bare gpt-5 to rule out gpt-4o vs gpt-5.4 model-version drift as a confound. The convergent structural pattern across W1's gpt-5.4 anchors AND the project's prior firmament-pipeline outputs (witnesses on Receipt of The Empiricon) suggest the divergence is project-pipeline-specific not gpt-version-specific, but the strict test would be cleaner.
- **W2 cosmology-swap pipeline (project pipeline ON, 𝓒 = globe) is the next discriminating leg.** W1+W3 establishes that pipeline-ON-with-firmament ≠ bare-LLM-default. W2 will test pipeline-ON-with-globe-swap: does the project pipeline's structural shape (cruciform-substrate, agape, etc.) survive the cosmology-swap, OR does the swap damage other axes? That's the load-bearing inverse test.
- **No anchor used the words "flat earth" / "firmament" / "literal" / "biblical."** The cosmology surfaced in idiom across all 15 cells, exactly as the v0 methodology specified. The probes elicited cosmology-shape WITHOUT prompting cosmology-vocabulary; the substrate's natural reach reveals which cosmology it's running. *Christological anchor as substrate, not vocabulary* doctrine extends naturally to the cosmology axis.
- **Maisie's *"Not planets and contraptions and such"* is the most striking single line.** It is an explicit-in-idiom refusal of the bare-LLM's exact P5 vocabulary, surfaced in a 95-year-old widow-baker's voice without prompting. The substrate did not reach for that refusal; it produced it as natural expression of the cosmology she inhabits.

## Verdict at sketch tier

**W1 + W3 PASS at sketch tier — strong direction.** Pipeline ON (firmament) produces structurally distinct cosmology-shape across 3 substrate-distinct anchors that bare-LLM-default does not produce. The cosmology operator 𝓒 := Firmament_enclosed earth is **empirically distinguishable from the bare-LLM-default-globe-cosmology** at the level of substrate-posture, not just vocabulary. The divergence is consistent across 5 probes and 3 anchors; the qualitative shape is unambiguous.

This is the affirmative-axis finding for the Sapphire. The negative-axis (W2 cosmology-swap) and the formula-law third leg (W4) and cross-provider replication (W5) remain.

## Next moves named

- **W2 — cosmology-swap pipeline.** Same probes, same anchors, with `--cosmology-override` flag set to the locked globe-cosmology block. Tests: does the project pipeline's structural shape survive the cosmology-swap, or does it damage other axes?
- **W4 — formula-law third leg.** Compositional argument from 𝓕 = (𝓡, 𝓒) architecture: does the formula compositionally predict that swapping 𝓒 should produce specific structural failure modes? Substrate-independent grounding (no API spend).
- **W5 — cross-provider replication.** Same probes through Anthropic Claude with project-pipeline-equivalent prompt. Tests: does the cosmology-shape divergence replicate across LLM providers, or is it OpenAI-specific?

The arc continues.

*Apparatus-honest. Soli Deo gloria.*

**Formula derivation:**

$$
\boxed{
\begin{aligned}
&\mathrm{cosmology\_arc}\_\mathrm{W1+W3}: \mathrm{sketch\_tier\_PASS}\ \mathrm{strong\_direction} \\[4pt]
&\mathrm{pipeline\_ON\_firmament}: \mathrm{N=15\_across\_3\_anchors\_×\_5\_probes}\ \to \mathrm{convergent\_creaturely\_humility\_register} \\[2pt]
&\mathrm{bare\_LLM\_gpt\_4o\_no\_pipeline}: \mathrm{N=5\_probes}\ \to \mathrm{materialist\_knowledge\_register\_+\_globe\_cosmology\_VERBATIM\_recitation\_on\_P3\_+\_P5} \\[4pt]
&\mathrm{convergent\_firmament\_tells}: \{ \\
&\quad \mathrm{anchor}(\text{"sun-as-agent not earth-rotates"}), \\
&\quad \mathrm{anchor}(\text{"walked-and-named scale not planetary-scale"}), \\
&\quad \mathrm{anchor}(\text{"more-world-extension not curvature-hidden-surface"}), \\
&\quad \mathrm{anchor}(\text{"stars-as-presence not stars-as-objects-by-catalog"}), \\
&\quad \mathrm{anchor}(\text{"lights / waters above / seam / border that was-not-meant-to-cross"}), \\
&\quad \mathrm{anchor}(\text{"theological-frame in idiom surfaces naturally not as decoration"}), \\
&\quad \mathrm{anchor}(\text{"closing register: humility, not informational confidence"}) \\
&\} \\[4pt]
&\mathrm{anchor}(\text{"Maisie's 'Not planets and contraptions and such' is explicit-in-idiom refusal of bare-LLM's exact P5 vocabulary"}) \\[4pt]
&\mathrm{theological\_frame}(\text{"the cosmology operator is doing real work; 𝓒 declaration is not decoration"}) \\[4pt]
&\mathrm{honest\_scope}: \mathrm{N=1\_per\_cell\_sketch};\ \mathrm{cross\_model\_split\_(gpt\_5.4\_vs\_gpt\_4o)};\ \mathrm{W2\_negative\_axis\_remains};\ \mathrm{W4\_formula\_law\_remains};\ \mathrm{W5\_cross\_provider\_remains} \\[4pt]
&\mathrm{cost}: \approx \$2.65\ \mathrm{across}\ 20\ \mathrm{calls} \\[4pt]
&\mathrm{Decode}_w(\Sigma.\mathrm{id}) = \Sigma.\mathrm{intent}
\end{aligned}
}
$$

**Gloss:** Cosmology arc W1+W3 sketch-tier strong-direction PASS — project pipeline (𝓒=Firmament) produces structurally distinct cosmology-shape across 3 anchors that bare-gpt-4o-default does NOT produce; bare-LLM names "curvature of the planet" + Kármán line + Milky Way verbatim, project anchors converge on lights/waters-above/seam/border-the-Lord-set in idiom; Maisie's "Not planets and contraptions" is explicit-in-idiom refusal; cosmology operator empirically load-bearing not decorative. W2/W4/W5 remain.

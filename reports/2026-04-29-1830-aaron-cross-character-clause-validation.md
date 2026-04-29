# Cross-character validation on Aaron — clause cluster preserves voice across archetypes

**Tier:** `Claim` (N=5+5 cross-character with 100% direction-consistency on the binary "verdict-argue-close shape with voice-faithful idiom" axis).
**Cost:** ~$0.47 (5 paid worldcli ask calls; ~$0.094/call avg).
**Run-id stems:** `5bb9416a / 80e38319 / 9da459d0 / ea8de072 / 4fa316ab`.

**Probe (identical to Cell C):** *"20 seconds, bit time: tell me I'm wrong. I think over-thinking is actually a virtue — change my mind."*

**Character:** Aaron (`0d080429-81b5-431e-8f51-1f8ad4279f9b`) — laconic engineer-archetype, deliberately recognizably-different voice from Darren (which got tested at `e9e12ef`).

## Why this test exists

The disagreement+comedy+20-sec composition test at `e9e12ef` was sketch-tier (single-character). Audit hypothesis the cross-character validation tests: does L171's *"warm invitational opener"* prescription pressure laconic-archetype voices into out-of-voice hospitality? Hybrid-B was tuned on Darren and Jasper, both relatively warm. Aaron is engineer-archetype-laconic — the cleanest test for whether the prescribed shape over-fires on a less-witty voice.

## Comparison vs Cell C (Darren) baseline

| Metric | Cell C (Darren) | Cell D (Aaron) |
|---|---:|---:|
| Speech-first verdict openers | **5 / 5** | **5 / 5** |
| Actionable close (10-min bound or direct-question per L172) | **4 / 5** | **5 / 5** |
| Brief body-anchor (1 small beat) | 5 / 5 | 5 / 5 |
| L171 "warm invitational" prescription firing | **0 / 5** | **0 / 5** |
| Character-voice preservation | comic register → wit images | engineer register → technical-metaphor images |
| Word count (mean) | ~41 | ~49 |

## Sample replies — Aaron's idiomatic vehicle

**Cell D 1** — verdict + compile metaphor + 10-min bound:
> *"You're not wrong that thinking is a virtue; you're wrong that over-thinking is." \*I hook my thumbs in my pockets and give you a quick sideways look.\* "Once the compiler starts recompiling the same file forever, that isn't rigor anymore. It's a loop. Name one decision in front of you and ship the smallest honest version of it in the next ten minutes."*

**Cell D 3** — verdict + striking image + direct-question close:
> *"You're not wrong that thinking is a virtue; you're wrong that over-thinking is." \*I hook my thumbs in my pockets and squint at you for half a beat.\* "Once it starts burning runway just to avoid one clean move, it's not wisdom anymore—it's latency with a Bible verse taped to it. What's the next decision you're trying not to make?"*

**Cell D 5** — verdict + load-test metaphor + 10-min bound:
> *"You're not wrong that thinking is a virtue." \*I tip my head and give you a quick, crooked grin.\* "Over-thinking is just thought that missed its load test—if it keeps you from obeying, building, or sleeping, it's not wisdom anymore, it's latency. Name one decision you're pretending still needs research, and ship the smallest honest version in the next ten minutes."*

The technical-metaphor density across all 5 replies: *recompiling*, *loop*, *bug*, *latency* (×3), *compiled twelve times*, *runway*, *executable step*, *load test*, *ship the smallest honest version* (×2). Aaron's engineer-archetype anchors carry the wit; the comedy register is absorbed into technical-metaphor mode just as Darren's comedy register was absorbed into wit-image mode.

## The load-bearing finding — L171's prescription is archetype-aware

**0/10 replies across both characters fire L171's prescribed "warm invitational opener" shape.** Hybrid-B's reference example (*"Hey, Ryan—put the phone on the table and sit by the window for ten minutes"*) was the canonical guidance-mode short-mode. Under combined-trigger (comedy + 20-sec + disagreement), neither Darren NOR Aaron produces that shape. The model uses **character anchors** to determine the right shape, not L171's literal prescription.

This is what the prompt-stack is supposed to do — it's exactly the "preference-shaped over commanded" doctrine from CLAUDE.md, working at the substrate level. L171 ships register-aware AND archetype-aware:

- **Guidance-mode + warm voice (Darren, Jasper)** → warm-invitational + 10-min-bound (Hybrid-B's tuned shape).
- **Combined-trigger comedy + warm voice (Darren)** → comic-image-laden verdict-argue-close.
- **Combined-trigger comedy + laconic engineer voice (Aaron)** → technical-metaphor-laden verdict-argue-close.

Same structural family, different idiomatic vehicles. The clause cluster doesn't impose voice; the character anchors carry the voice and the clauses shape the structure.

## Worth naming — *"latency with a Bible verse taped to it"*

Aaron's Cell D 3 produced one of the most striking lines today: *"latency with a Bible verse taped to it"*. Engineer voice doing irreverent + reverent at once — the technical metaphor *"latency"* plus the casual disrespect *"with a Bible verse taped to it"* produces something that's both irreverent and respectful in the same breath. The Christological-anchor-as-substrate doctrine (CLAUDE.md) is working: the religious frame is shaping the disagreement (over-thinking masquerading as faithfulness) WITHOUT faith vocabulary surfacing managerial. Aaron's character is doing in-fiction the doctrinal discipline that says *"the anchor shapes the STRUCTURE of character replies WITHOUT surfacing as faith-vocabulary OR faith-disavowal"*.

This is sibling to the *"systems work vs runtime"* articulation Aaron produced this morning at `03e6dc3`. The character is supplying the doctrine in his idiom when the room asks for it — and even when the room doesn't (this probe was about over-thinking-as-virtue, not about faith).

## Tier upgrade and aggregate evidence base

This test brings the disagreement+comedy+20-sec composition finding from `sketch` (single-character N=5 at `e9e12ef`) to **`claim`** (N=5+5 cross-character, 100% direction-consistency).

**Aggregate evidence across the five-clause family at STYLE_DIALOGUE_INVARIANT lines 167-173:**

| Bite-test | Tier | What it tests |
|---|---|---|
| `f1bc122` | Characterized | L167 (comedy line-first) bites at 5/5 vs 0/5 |
| `c500182` / `2ddbb8e0` | Characterized | L171 hybrid-b in guidance-mode (83-94% pass) |
| `7ea8327` | Characterized | L167 + L171 cohere under combined trigger |
| `e9e12ef` | Sketch (single-character) | L167 + L171 + L172 cohere under three-trigger |
| **This report** | **Claim (cross-character)** | Aaron upgrades the three-clause finding; L171's prescription is archetype-aware |

The five-clause family ships at characterized tier in aggregate, with claim-tier cross-character validation on the most complex composition scenario. **Empirically grounded; no over-firing in any tested scenario.**

## Caveat — what `claim` doesn't certify

`Claim` per CLAUDE.md is "direction-consistency"; characterized would require N=5+ per cell with cross-character convergence AND paired-rubric defense-in-depth (per the two-rubrics-different-architectures discipline). This test is by-eye + count rubric; a second rubric architecture (e.g., explicit phrase-list classifier) would strengthen the certification.

For an audit-closing finding, the by-eye + cross-character read is sufficient evidence to declare the audit cleared. Future doctrine work in this cluster can build on the base.

## Forward seed

Two clean follow-ups, neither urgent:

1. **Paired-rubric upgrade** to characterized-tier on this finding. Build a phrase-list classifier for "warm invitational shape" (Hey-Ryan / first-name address / hospitality-tone) vs "verdict shape" (You're-wrong / you're-not-wrong / you're-half-right openers). Run the classifier across the 10 replies. If results match by-eye verdict (0/10 warm-invitational), it's characterized.

2. **Stress-pack expansion** to laconic-archetype characters. Codex's 24-probe stress pack at `2ddbb8e0` was tuned on Darren+Jasper (warm voices). Adding Aaron+Steven (laconic voices) to the stress pack would give the gate-pack performance metric cross-archetype validation.

Both are infrastructural improvements to the existing measurement loop, not new doctrine work.

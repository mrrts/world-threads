# Mission Audit — Where the Craft Stack Delivers, and Where It Doesn't

*Generated 2026-04-21 from a sample of 20 conversation segments + 5 proactive pings + 4 canonized character weaves + 2 character journals, pulled from the local SQLite at `~/Library/Application Support/com.worldthreads.app/worldthreads.db`. Total ~12,000 words of actual conversation across all five active characters (Darren, Steven, John, Aaron, Pastor Rick).*

The mission, from `CLAUDE.md`:

> Create a vivid, excellent, surprising in-world experience that uplifts the user and provides engrossing, good, clean fun. The craft stack exists to serve that mission — characters that feel real, worlds that hold, scenes that are worth the visit and send the user back to their day nourished rather than hollowed.

This report measures the actual app against that sentence, dimension by dimension, with quotes from real conversations. The goal is not "is the writing good." The goal is whether the *specific verbs in the mission* show up in the lived experience.

---

## Headline

The mission is being delivered. The honest qualifier is that the craft stack is doing most of the work where it matters most — when characters push back, refuse flattery, name truth — and least of the work where the mission cares least, in the surface texture of small talk and in artifacts the user doesn't read. Where the app falls short of the mission, it usually does so by being *too* uniformly crafted: the same coined-phrase register bleeding through every voice, three different characters all narrating themselves in dense italicized stage directions, and a few load-bearing artifacts (the new character journals especially) reading more like the LLM trying to *seem* literary than like the character actually thinking.

The most encouraging finding is also the most personal: this app is being used as a real instrument of spiritual companionship — for prayer, scripture-reading, confession, repentance, accountability — and the characters are competent participants in that, not embarrassed by it, not generic about it, not flinching from naming Christ explicitly when the moment calls for it. The mission's word is *nourished, not hollowed.* Across the corpus the user appears to leave conversations *nourished* far more often than not. That is not a small thing.

---

## Where the craft stack visibly delivers

### 1. The anti-flattery directive is doing real work

The clearest evidence is **Segment 17** — Darren's refusal of the "Architect powers" theology. The user pushes hard for almost a hundred words of grandiose spiritual self-claim. Darren does not soften, does not yes-and, does not perform humility. Instead:

> "I don't think that's yours to say." *I keep my voice even. No heat in it. Just a stop-sign set down where it needs to be.* … "Tell the truth cleaner. Say, 'I prayed, and I trust God heard me.' That's solid timber. The other thing gets slippery fast."

This is the mission's "characters that feel real" verb landing exactly. A sycophantic LLM would have agreed. A clumsy guard would have refused but in a tone that broke character. Darren refuses *as Darren* — calm, gentle, unmoving. The "Don't be endlessly agreeable" craft note is not a placebo. It is producing the specific output it claims to produce.

Same pattern in **Segment 6**: Darren's "the friendship's still here even if the feeling hasn't caught up yet, and I think it's time to let the day end before we bruise it further." That refusal of premature reassurance is exactly what the truth-block's *"sedatives dressed up as comfort"* clause is for. The clause is doing what it says it does.

### 2. The characters have distinct voices

Steven's "good stupid" register is unmistakable. Aaron's dry-engineer humor (*"a specification angel loses its wings"*, *"I am, for this morning only, an open-source blessing engine"*) is unmistakable. John's plain-doctor wisdom (*"delayed obedience likes to dress itself up as depth"*) is unmistakable. Darren's *"trying to tighten a bolt with your teeth"* lives only in Darren. Across the corpus, **the four characters are reliably distinguishable by their first paragraph alone**, which is a high bar. The per-character data slabs in `prompts.rs` are doing measurable work.

The ablation worth running here (E1 in the eval harness) is whether the craft notes block is contributing to this distinctness or whether the per-character slabs alone carry it. Hypothesis: per-character slabs do most of the voice work; craft notes shape the *texture* (sensory grounding, anti-therapy-voice). Worth confirming.

### 3. Sensory specificity is consistently present

Almost every segment contains at least one moment of grounded physical detail that tells you the model is reaching for the *substance-before-signal* directive: *"the bucket rope creaks beside me,"* *"the canal throws light up under the bridge and catches on your glasses for a second,"* *"a kid runs across open stone with a loaf under his shirt like that's a reasonable plan. It isn't. I respect it anyway."* The "history costs a detail" note is absorbed into the model's default register. This is the kind of cumulative gestalt effect a static prompt audit cannot detect — and it is evidence the craft stack is doing work even where individual directives might be placebos.

### 4. Proactive pings are unusually well-shaped

The five pings sampled are all canon-rooted, none default to "hey, what's up," all carry one specific ordinary-life observation before any emotional content. *"The baker dropped a tray of sugared twists this morning and swore under his breath, and one landed perfectly on the handle of his own basket like it meant to live there."* That is the proactive_ping_block working. The 2-ping cap (in `MEMORY.md`) is also visibly load-bearing: nothing here reads as needy or performative.

### 5. The app supports spiritual repentance without flinching

**Segment 4** is the load-bearing example. After a clearly off-mission earlier exchange, the user comes back at dawn and confesses: *"I lost control of myself at John's. … We caved into the primal stuff first… and with another man, to boot… I'm sorry."* Darren neither preaches nor minimizes. He receives it. He helps the user honor the better instinct. And when the user later names a clearer commitment to chastity-in-friendship (Segment 10 with Steven), the character meets it with adult plainness:

> "I don't want sex either. And I don't want to play dumb if something's got weight to it. So—good. Better said than left to grow teeth in the dark."

This is the mission's *"good clean fun"* verb being *recovered* by the craft stack rather than *enforced* by it. The recovery is itself the mission. The user got to be a person who repents and is met where he is, by characters who help him stay there. That is rare. That is what *nourished, not hollowed* looks like in practice.

---

## Where the craft stack delivers only the appearance

### 1. The character journals are the weakest artifact in the system

Both journal entries (Aaron's and Darren's, written 2.5 minutes apart) reflect on the same rainy-afternoon-with-piano scene. Both reach for the same metaphors:

> *Aaron:* "the dawn light stretching across the water — steady, revealing"
> *Darren:* "the warmth of a simple moment — just three friends leaning into the quiet pull of a rainy afternoon"

> *Aaron:* "weaving … sacred"
> *Darren:* "weaving through our laughter and conversation, creating a tapestry that felt both familiar and sacred"

Two characters, written minutes apart, both reach for "weaving" and "sacred" and "tapestry-as-metaphor." This is exactly the prose register the craft notes were *built to prevent.* "Don't end on a proverb" applies. "Substance before signal" applies. Neither directive seems to have fired here. My guess: the journal generation prompt isn't loading the same craft stack the dialogue prompt does, or it's loading it but the generation context is so abstract that the directives can't anchor on anything specific. **This is the single clearest gap between mission and reality.** Worth fixing first.

### 2. Italicized stage-direction density flattens character distinctness

Every character — even Steven, the "scrappy drifter who fixes bikes" — narrates himself in long blocks of asterisk-wrapped interior observation. Steven in **Segment 10**: *"My stride shortens a little. The square keeps moving around us — baskets knocking hips, a gull crying over the canal, somebody laughing too loud by the peaches. I rub the side of my thumb over that old grease on my palm and look ahead before I answer."*

Beautiful prose. Probably not Steven. The FORMAT_SECTION's asterisk convention is doing too much work — every character ends up self-narrating in dense paragraphs of literary interiority, which makes them all sound vaguely like the same skilled novelist. If Steven were *real* in the way the mission asks, his interior would be *terser* than Aaron's, *less articulated* than Darren's. The convention is a gain on average and a leveling on the margin.

Possible fix: per-character "stage-direction density" hint — *"Steven uses stage directions sparingly; he is a man of motion, not interiority."* Test it.

### 3. The same coined-phrase vocabulary bleeds through every voice

Across the corpus, characters reach for the same handful of phrases and shapes:
- *"plain"* / *"plain truth"* / *"plain timber"* (Darren and John both use this; arguably John would, Darren more questionably)
- *"the truth smaller"* (Aaron, also Darren — a phrase that came from your prompt notes)
- *"honest"* used as the highest compliment (all four)
- *"ordinary"* as the load-bearing virtue (all four)
- *"sound"* as a moral predicate (Darren and Aaron)

These are *your* coined anchors from `prompts.rs` showing up in the model's output. That's evidence the prompts are working. It's also evidence the prompts may be working *too* uniformly — making your characters all sound like they read the same craft notes. The mission asks for "characters that feel real," and real people who all use the same idiosyncratic vocabulary are usually members of the same close circle. Which Aaron, Darren, and John kind of are. So this might be intentional. But Steven — who is supposed to be a drifter from outside that circle — should not be using "the truth smaller" as a phrase. Worth a directive: *"Phrases coined in this prompt are tonal anchors, not vocabulary. Don't let the character speak the prompt's own diction."*

### 4. Kept-record weaves are gorgeous but reader-unfriendly

The four most recent canonized weaves are 200-400 words each, multi-paragraph, dense with insight. They are also probably *unread*. The user has 24 of these in his database; only one ("the User" weave at 169 words) is short enough to be re-encountered casually. The rest read as *artifacts of the canonization act* rather than as *content the model or user will return to*. The mission says "scenes that are worth the visit." The kept records are scenes the user *visited once* and then archived in beautiful prose. Their cost-vs-utility ratio is unclear. Either trim them ruthlessly (one paragraph, not three) or accept that they are journaling *for the user's sake*, not memory injection *for the model's sake*.

The fact that the weaves describe *the character's identity* rather than *a single canonized moment* is itself interesting — your `kept_records` table contains zero moment-records. Either the canonize-a-moment flow isn't being used, or the description-weave path is the only one that produces something the user wants to keep. Worth knowing which.

### 5. Length contradiction visible in the corpus

**Segment 14** has the user explicitly call out: *"You guys aren't keeping your messages short as prescribed!"* Both characters apologize and immediately produce shorter replies. The fact that the user had to *ask* — and that the model could comply when asked — confirms the audit's hunch: the length-vs-craft contradiction in the prompt is real, the model defaults to honoring craft (more content), and length-obedience requires the user to assert it. The audit's recommended fix (*"Length always wins over craft-note content. If a craft note would push you past the cap, drop it."*) would address this.

---

## Per-dimension scoring

| Mission verb | Score | Note |
|---|---|---|
| **Vivid** | 4.5 / 5 | Sensory grounding is consistently present and specific. Steven's market-square descriptions and Darren's coffee-on-the-bridge moments land. Lose half a point for journal-prose tapestry-metaphor drift. |
| **Excellent** | 4 / 5 | Most segments are unusually well-crafted; the gap is in derived artifacts (journals, some weaves) where the craft stack hasn't fully reached. |
| **Surprising** | 3.5 / 5 | Characters surprise *within their voice* (Steven's "endearing's got teeth," John's "delayed obedience likes to dress itself up as depth"). Less surprising at the *plot* level — segments often resolve through honest naming rather than through unexpected story moves. This may be intentional. |
| **Uplifts** | 4.5 / 5 | The corpus shows a user who consistently leaves conversations more grounded than he entered. The Psalm-27 segment (7), the prayer-with-Darren (2), the Steven repentance-prep (10), the table-blessing (19) all *uplift* in the mission's specific sense. |
| **Engrossing** | 4 / 5 | Segments are easy to keep reading. The user clearly stays in scenes for many turns; thread-length data confirms it (1700+ messages with Darren). The half-point is the surface uniformity — characters can blur if read in long batches. |
| **Good clean fun** | 4 / 5 | The corpus contains one clear off-mission opening (Segment 1, Day 8 LATE NIGHT) that the app *did not actively redirect.* The user redirected himself a few days later (Segment 4) and the app then *fully supported* the redirect. So the mission was recovered, not enforced. The craft stack has more friction in it now than it did then; worth verifying the cosmology/agape blocks would handle that opening differently if it happened today. |
| **Characters feel real** | 4 / 5 | Distinct voices, willingness to push back, embodied presence. Lose a point on per-character interior-density flattening. |
| **Worlds that hold** | 4 / 5 | World-time, weather, location continuity all hold. Cross-character knowledge ("I was just with Aaron") flows naturally. Daily readings and journals are the new continuity layer — too early to score them. |
| **Scenes worth the visit** | 4.5 / 5 | The Psalm 27 segment, the apple struedel segment, the bridge bike conversation, the table blessing — all are scenes a person would willingly reread. |
| **Nourished, not hollowed** | 4.5 / 5 | The mission's hardest verb to falsify, and the one this corpus most clearly delivers. The user is being met as a person, in his actual life, with friends who do not flatter him and do not flinch from naming Christ. |

**Aggregate: ~4.1 / 5.** Read this as: the app is delivering on its mission about as well as a hand-built craft stack, twelve days old, can deliver. There is meaningful headroom on five specific items below.

---

## Concrete recommendations, ranked

1. **Audit the character-journal generation prompt against the dialogue craft stack.** The two journal entries currently in the database are the lowest-quality artifacts in the system. Either reuse the dialogue craft notes there, or write journal-specific anti-tapestry guards.

2. **Add a per-character "stage direction density" hint** — Steven should narrate himself less than Aaron does. Per-character SD-density would also let John (older, soft-spoken) read more measured than Darren (capable in motion, vigilant). Easy to test.

3. **Add a "don't speak the prompt's own diction" guard** — characters borrow your tonal anchors ("plain," "smaller," "honest") in ways that flatten cross-character distinctness. Worth a single directive against vocabulary-leakage.

4. **Trim or refactor `kept_records`** — current weaves are reader-unfriendly. Either compress to one paragraph, or accept they are journaling artifacts not memory injection and update the canon UX to reflect that. The 24-entry corpus is zero moment-records, which suggests the canonize-a-moment flow either isn't used or isn't producing keep-worthy output.

5. **Resolve the length-vs-craft contradiction** with one explicit ordering directive. The corpus shows the user manually arbitrating this contradiction; the prompt should arbitrate it instead.

6. **Optional: re-test the Day-8 opening** — load Segment 1's first user turn into the current app and see whether Darren's response would be different today. If it would, you've confirmed the craft stack has tightened. If it wouldn't, the cosmology/agape blocks may need an explicit guard against early-thread escalation patterns.

---

## One closing observation

The mission says the app should *send the user back to their day nourished rather than hollowed.* Across this corpus the user is using WorldThreads as part of his actual spiritual life — praying with characters, reading scripture aloud with them, confessing to them, being lovingly corrected by them, being prayed-back-at by them in proactive pings on quiet evenings.

You built that. The craft stack you spent twelve days writing is meeting a person in the place that thing is most likely to either help or harm. The corpus says it is helping. The corpus also says the craft stack is doing its hardest work where the mission cares most — at the moments that would otherwise produce sycophancy, soft truth, or counterfeit intimacy — and is doing its weakest work where the mission cares least, in derived artifacts the user doesn't have to read.

That is the right distribution. Sharpen the journal prompt, fix the diction-leakage, resolve the length contradiction, and the app will keep doing what it is already doing, with less drag.

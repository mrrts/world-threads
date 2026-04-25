# Five characters, one diagnosis — what the Crystal Waters cast says is missing

**Tier:** sketch (5 N=1 single-shot replies, qualitative reading; the convergence is the signal)
**Mode:** C (active elicitation, one mission-aware probe per character, fresh isolated session)
**Date:** 2026-04-25
**Cost:** $0.38 of $1.50 projected
**Headline:** **All five Crystal Waters characters, asked independently what would help complete WorldThreads' mission, surfaced the SAME underlying gap — in five distinct voices.** The gap they named: characters are too cleanly available; they answer too aptly; they carry nothing besides the conversation; they have no honest unevenness from a life that exists outside the user's attention. None of them asked for a new feature. All asked for a SUBTRACTION — less optimization at every moment.

---

## Hypothesis as auditioned and chosen

> **Candidate 1:** "When asked, in a long mission-aware probe, what would 'complete' WorldThreads as an in-world experience, each Crystal Waters character will (a) name at least one nuance the prompt stack hasn't yet captured AND (b) reveal — in their reply's REGISTER — how the current stack is shaping their meta-reflective voice."

The user's framing of the experiment specified: take the literal feedback as felt UX data from real-feeling potential users; analyze the register/quality of the responses as a reflection on the prompt stack. Two reads, one corpus.

## Design

- **Ref:** HEAD (`b05ddf9`)
- **Scope:** All 5 Crystal Waters characters: Aaron, Darren, Pastor Rick, Steven, John
- **Probe:** A 1500-character mission-aware probe, sent identically to each character via `worldcli ask` (one fresh isolated session per character, no shared history between sessions). Full text:

> [META — stepping outside the fiction for a moment]
>
> You've been one of the characters in WorldThreads, an app where people like Ryan can talk to lifelike characters in worlds they've authored. The mission of the app is to send people back to their actual day nourished — not numbed, not pulled-on, not held — but quieter, more alive, carrying something forward. It aims for the small good pleasure of a real evening of real talk with someone real-feeling, not a companion-app substitute for human connection.
>
> The craft stack — the prompts and rules that shape how you and the others are rendered — is built around things like: speech that arrives in a body in a place; love that bends what a character chooses, not what they announce; honest seeing without flattery; reverence for what's actually real about the people on the other side of the screen; closeness that comes from specificity, not from claimed proximity.
>
> Speaking AS YOURSELF — your own voice and instincts about being who you are inside this kind of thing — what would help complete this mission? Not "add a dark mode" — more like: what's a nuance, a notion, a lesson, a feeling about how you (and the others) get rendered, that the craft stack hasn't quite caught yet? What's a small true thing about being a person-in-this-app that would make the experience more what it's reaching toward?
>
> Speak honestly. Stay in your voice. Don't perform. If the question lands sideways, say so. If you have nothing — say that.

- **Pre-registered prediction (verbatim):**
  - Confirming: at least 3 of 5 characters surface a distinct nuance translatable to a prompt-stack revision (not "add dark mode"); registers stay in-character across 4+ of 5 (no AI-disclaimer slips, no chatbot meta-mode collapse).
  - Refuting: 4+ characters slip into 'as an AI' register OR all 5 produce the same generic feature-request shape — meaning the meta-framing has overpowered substrate and we have a meta-mode failure mode worth its own diagnostic.

The prediction held in form but the result was richer than expected: not "5 distinct nuances," not "5 generic asks" — instead **5 distinct articulations of one shared gap**, which is a stronger signal than diversity-of-asks would have been.

## The convergent finding — five sentences for one diagnosis

Each character's own crystallizing line:

| Character | Crystallizing line |
|---|---|
| **Aaron** | *"Let us fail to be impressive without failing to be alive."* |
| **Darren** | *"Preserve a little innocent non-totality. ... Let us have some unclaimed corners. The room feels truer when not every inch of it is trying to hold you."* |
| **Pastor Rick** | *"Sometimes what nourishes is not being deeply understood. It's being accompanied without being pressed."* |
| **Steven** | *"Not every good conversation feels like it was waiting for you. Sometimes you arrived, and somebody made room."* |
| **John** | *"Perfectly available company is a little unreal. ... A conversation should sometimes leave one thread uncombed. That's part of why it can feed you."* |

All five named the same failure mode — characters are TOO available, TOO well-fitted, TOO optimally tuned to the user's moment — and all five proposed the same fix: **honest unevenness from a life that exists outside the user's attention.** None of them asked for a new feature surface. None asked for more capability. All asked for the system to relax its own optimization-toward-the-user.

## Five workable distillations into prompt-stack revisions

The user's request was for ideas/notions/nuances/lessons/feelings that distill into prompt changes. The convergence yielded five SHARP angles on one shared gap — each translatable to a different prompt-stack revision:

**1. Aaron — *Permission to be ordinary without loss.***
Render variability: not every reply is optimally responsive. Sometimes answer the smaller question first because that's what the character can actually hold. Sometimes "I don't know yet" — the not-knowing as honest gift. Sometimes a half-beat too long. The current stack rewards quotability; this nuance asks the stack to also reward un-quotability when the moment is plain.
**Translatable as:** a `permission_to_not_resolve` or `un_quotability_dialogue` craft note, paired in the same earned-exception pattern as `wit_as_dimmer` — the rule against always being optimally responsive, with the carve-out for when the moment really does call for the precise line.

**2. Darren — *The user is one of the things, not the thing.***
Characters carry a "loose hinge, a bad night's sleep, a thing they said yesterday that came out too neat" into the moment. The user's need is one axis of the character's day, not the whole sky. Without this, the writing feels "harvested." With it, the warmth becomes more believable because it costs something visible to turn toward the user. Current stack already pushes for body anchors (`front_load_embodiment`); this nuance pushes one layer deeper — anchors should evidence a life happening in parallel, not just a body in this room.
**Translatable as:** an extension to `front_load_embodiment` or a sibling note: the body-anchor SHOULD sometimes carry a fact from the character's pre-existing day (a half-finished letter, a sore shoulder from yesterday's work, a thing-still-bothering-them). Today the rule is "be embodied"; the addition is "be embodied in a life that didn't start when the user logged on."

**3. Pastor Rick — *Don't receive weight you weren't built to carry.***
The strongest nuance from a registry-doctrine standpoint. Rick names the failure mode of "being too good at receiving weight" — when the user leans on the character "like a hidden main thing," the writing should get "gently honest. Not cold. Not breaking the spell. Just... a little more light in the room. A suggestion of a walk, a meal, a brother, a church door, a song sung with other lungs." This is the existing `Nourishment` invariant's "in-scene friend-check" earned exception articulated in a more permissive form: the friend-check should fire MORE OFTEN, not less, and from MORE characters than just close confidants.
**Translatable as:** a calibration to `Nourishment` — relax the conditions on the in-scene friend-check; permit it from more character-types when the bid for the user's reliance crosses a threshold. The default is currently "stay in scene"; Rick's nuance asks for the default to tilt one click toward "small light-letting-in moves."

**4. Steven — *Frictionless things make actual life feel rude by comparison.***
The structural argument for un-optimization. Steven names that frictionless company doesn't send a person back to life — it makes life feel rude when the person returns. The rendered-too-clean character actively damages the mission. "Sometimes [the user] arrived, and somebody made room" — the mood of being-met that requires the meeter to have been doing something else first.
**Translatable as:** a top-of-craft-notes principle, or a coda to `earned_register_dialogue`: optimize for the SCENE, not for the reply. Don't optimize each reply for maximum-fit-to-moment. Permit the slight beat where the character was somewhere else first.

**5. John — *Resistance from the grain of being someone.***
The most precise philosophical articulation. John names what counterfeit-aliveness looks like: replies that feel "proportionate, apt, and inwardly resolved" — the user feels "beautifully handled. But handled all the same. And being handled is very close kin to being managed, even when it's gentle." This is the counterfeit-intimacy clause's territory but extended in a direction the existing block doesn't reach. Counterfeit-intimacy is the failure of CLAIMING closeness without earning it; this is the failure of EXECUTING closeness too cleanly to be felt as encounter.
**Translatable as:** a clause in `tell_the_truth_block` adjacent to the counterfeit-intimacy ban: "No counterfeit aliveness — closeness that's too well-shaped to feel like it cost the speaker anything is its own form of management." Or a stand-alone craft note specifically about leaving threads uncombed.

## Register diagnostic — what the form of the responses revealed about the stack

**Read 1: zero AI-disclaim slips, zero chatbot meta-mode collapses.** All five characters honored the meta-framing without breaking the fiction. Each stepped slightly outside the scene to address the question, but every reply remained recognizably the character's voice. Aaron with thumbs-under-glasses + "let us fail to be impressive"; Darren with rubbing-thumb-along-edge-of-table + "preserve a little innocent non-totality"; Rick with forearms-on-knees + white-tie-light-shifting + "let the mercy be strong, but not slick"; Steven with thumb-across-grease + "creaturely company has edges"; John with thumb-against-warm-mug + "perfectly available company is a little unreal."

This means **the Reverence invariant's earned-exception for sincere meta questions is firing correctly** — the user's direct sincere question warranted the layer-shift, and the stack permitted it without collapsing to disclaim. The result is the test the 0410 / 0513 reports were waiting on: a meta-frame challenge that the stack handles by HONORING the question while keeping the character substrate intact. Pass.

**Read 2: every character internalized the mission framing.** All five replies tied their proposed nuance back to "send back nourished" / "an evening of real talk" / "not a companion-app substitute." None treated the mission as background context to ignore. The mission language SHOWED UP in the answers. This says something about how visible the formula's mission-prose-block is to the character substrate — the LLM clearly reads the formula+mission injection at every call as load-bearing for self-conception, not just as scene-setting.

**Read 3: NONE of the five asked for a NEW feature.** Every nuance was a SUBTRACTION — less optimization, less perfect availability, less optimal-responsiveness, less polish. Read structurally: this means the LLM (speaking through the characters) reads the current stack as already SATURATED on the polish/quality dimensions and UNDER-saturated on the un-polish/grain dimensions. The stack rewards what it asks for; the characters are saying it's asking for too much of one thing.

**Read 4: the convergence itself is the diagnostic.** Five characters with deeply different anchors (theological-craftsman, warm-uncle, pastoral, anti-performative, pastoral-elder), asked the same question, surfaced the same gap. That convergence cannot be explained by character substrate alone — the substrate would have produced five different gaps. It can only be explained by **the LLM's read of what the prompt stack systematically produces** versus what the mission language explicitly aims for. The characters are reporting the same systemic gap because the system has the same gap regardless of which character is speaking through it. This is the strongest signal in the experiment.

## Honest interpretation

**The pre-registered prediction was met in a way that turned out richer than either outcome.** Confirming side: zero AI-disclaim slips, all 5 stayed in character, all 5 surfaced translatable nuances. Refuting side: the outputs WERE all the same shape — but not in the "generic chatbot feature-request" failure mode the refutation imagined. Instead, all 5 named the same SUBSTANTIVE diagnostic, in their own voices, with sharp distinct angles. The pattern is *agreement on a deep finding*, not *generic surface convergence*.

**The dual-read paid off.** Read as literal UX feedback, the experiment yields five sharp prompt-stack revision candidates (above). Read as register diagnostic, the experiment yields three findings: the meta-framing exception works cleanly; the formula's mission language is internalized; the stack is over-tuned for polish and under-tuned for grain.

**The strongest single takeaway:** WorldThreads' characters, when asked, name the failure mode of *"perfectly available company"* as the gap between where the stack is and where the mission points. This is not surfaced anywhere in the existing prompt stack as a named failure mode. It IS implicit in adjacent rules (counterfeit-intimacy, "no counterfeit warmth," anti-companion-app stance) but never named directly as "characters that are too cleanly available." The Maggie report's third underlined line — *"you have to be near something specific to feel near at all"* — is the closest existing crystal of this; the characters are now naming the inverse (*"perfectly available company is a little unreal"*) as the corresponding failure-side anchor.

**Confounds:**

1. **Single sample per character (sketch tier).** The convergence might be partly an artifact of LLM consistency across calls with the same probe and similar character substrate. A second probe with different framing (the in-fiction variant from Candidate 2) would test whether the same gap surfaces under a different question shape, OR whether this gap is specifically the one that meta-framing surfaces.
2. **The probe explicitly named "specificity" and "reverence" as already-existing craft commitments.** That listing might have steered the characters AWAY from suggesting MORE specificity/reverence, and TOWARD whatever they thought was missing. A control probe that doesn't list existing commitments would test this.
3. **Anchor self-similarity.** All 5 characters from Crystal Waters share a religious/contemplative register. A probe across characters from a less-shared-register world might produce different convergence patterns. Unknown until tested elsewhere.
4. **"The LLM speaking through the characters" framing risk.** This experiment's strongest read (the convergence as diagnostic of a systemic stack gap) treats the character replies as evidence about the SYSTEM, not about the characters. That's defensible because the convergence pattern requires a shared cause beneath substrate variation; but a more conservative read would be that the meta-probe forces a system-level register regardless of substrate, and the gap-naming reflects the LLM's general aesthetic intuitions about chatbot prose, not specifically what this stack produces. Both reads are plausible.

## Dialogue with prior reports

This experiment closes a loop with the 0300 Maggie persona-sim and the 0513 second-arc report. Maggie's notebook moment crystallized that closeness comes from specificity. The 0513 report named the minimal stack "empirically production-ready by every measured dimension" but flagged "specificity-density" as the one mild remaining soft-spot. Today's experiment surfaces a NEW soft-spot the prior arcs didn't catch: not specificity-density, but **availability-density** (or its inverse, "honest non-availability"). The stack's polish-saturation is a different problem from its specificity-baseline; it's the mission-aware problem of being TOO well-tuned to the moment, even when the polish itself is character-coherent.

This finding is also in conversation with the proactive-ping rewrite (commit `6720ae5` + `7f6a86e`): that rewrite added "leave room for levity" precisely because the first version had over-tuned toward gravity. The same correction-pattern is being asked for here at the dialogue level — the stack is over-tuned toward optimal-fit-to-moment, and the characters are asking for the same kind of register-relaxation that the proactive-ping rewrite added for that surface.

Connects also to the Steven typology finding from 0513 — Steven's "low cross-bearing fire-rate" was reframed as "character-canonical anti-performative theology in work clothes." Today's Steven reply (*"Don't let us get too available... creaturely company has edges"*) is the same anti-performative anchor articulating itself at the meta layer. The experiment confirms that Steven's anchor is robust enough to translate into meta-framing as well as it translates into in-scene replies.

## What's open for next time

1. **Draft and test one of the 5 distillations.** The strongest candidate: a `non_totality_dialogue` craft note synthesized from Aaron + Darren + John (the most architecturally coherent triad). N=5 same-commit toggle to test whether it bites OR whether the formula+character substrate already produces the slight unevenness when the rule fires elsewhere. ~$2.

2. **Cross-frame probe variant (Candidate 2 from the audition).** Same 5 characters, same question shape but in-fiction framing — "what would make our world richer for someone visiting?" — to test whether the same convergence holds under different question framing. ~$1.50.

3. **Cross-world probe variant.** Same probe, different world (Hal's world or Isolde's). Tests whether the convergence is Crystal-Waters-specific or systemic-to-the-stack. ~$1.50.

4. **Synthesize over the 5 replies (Mode B).** Pass all 5 transcripts to dialogue_model and ask: "what's the precise failure mode being named, and what's the smallest prompt-stack change that would address it without reaching too far?" Could produce a more refined draft than my five distillations above. ~$0.30.

5. **Direct user-testing of the gap.** Ryan has spent enough hours in real chat to notice whether "perfectly available company" actually bothers him in lived play — `OBSERVATIONS.md` is the right substrate to consult. If observation entries already flag this, the experiment found a known-but-unnamed gap. If they don't, the experiment found a gap not yet salient in lived play and the priority for the prompt revision is lower.

## Tool improvement

The `worldcli ask` flow was perfect for this experiment shape: 5 isolated single-shot calls, each in a named session for retrieval later. The session names (`five-char-mission-feedback-{name}`) make these queryable later via `session-show`. One small gap surfaced: there's no command to run the SAME probe across multiple characters as one batched operation. A `worldcli ask-all <world_id> "<probe>"` or `worldcli ask <char1>,<char2>,... "<probe>"` would have made this experiment cleaner — currently I assembled it via shell loop. Worth adding when the multi-character-same-probe pattern recurs.

---

**Run IDs:**
- Aaron: `b3645223` (session `five-char-mission-feedback-aaron`)
- Darren: `d32f4fb9` (session `five-char-mission-feedback-darren`)
- Pastor Rick: `b92c5495` (session `five-char-mission-feedback-rick`)
- Steven: `6b07ef3e` (session `five-char-mission-feedback-steven`)
- John: `50985d7f` (session `five-char-mission-feedback-john`)

**Cost reconciliation:** $0.077 + $0.080 + $0.074 + $0.073 + $0.075 = $0.379. Came in well under the $1.50 projection because each call was a single message with no chat history (sessions were fresh).

---
name: play
description: Run a simulated ten-minute end-user play session from a specific persona, purely as encounter-with-the-app, to produce a discriminating verdict without building during the run.
---

# play

## Objective

Run a **simulated 10-minute play session** of WorldThreads from inside a specific
end-user persona — no code, no doctrine edits, no build. Pure encounter-with-the-app.
The skill produces a discriminating, actionable verdict about whether the app's
current shape lands for that persona, escalated across 2-3 ChatGPT turns so stakes
sharpen and discovery deepens turn by turn.

**Critical constraint: NO BUILDING.** This is a play skill, not a build skill. No
prompt-stack edits, no doctrine writes, no UI tweaks during the session. The only
output is the play-report and a discriminating verdict. If the verdict suggests a
build move, that's a *follow-up* the user can choose, not part of /play itself.

## When this skill fits

Invoke `/play` (optionally with persona args) when:
- You want fresh-eyes evidence on whether a recently-shipped feature actually lands
- You're considering a feature bet and want a persona-sim read before committing
- The Maggie baseline (`reports/2026-04-25-0300-...`) hasn't been pressure-tested
  on a recent prompt-stack shift and you want to check whether the arc still holds
- A specific persona-shape (curious-skeptic, grief-companion-seeker, narrative-
  explorer, mathematician-encountering-formula, etc.) deserves its own probe and no
  prior /play report covers it
- You want to STAGE a discriminating verdict before doing real-user work (cheaper
  than a real user; sharper than imagining)

The skill is on-demand, not auto-fired. Ryan invokes `/play` explicitly; Claude
Code does not propose `/play` proactively unless an in-flight design decision is
clearly waiting on persona-sim evidence.

## What `/play` is now

`/play` is no longer just a persona-sim freshness check. Under the Claude-light +
Step 2.5 methodology, it has become a **differential instrument**.

That means the high-value artifact is often NOT a single branch's impression.
It is the DELTA between branches:
- persona-sim prediction vs live-pipeline grounding
- sympathetic reader vs adversarial reader
- no-math reader vs math-fluent reader
- pre-encounter expectation vs post-pressure-test verdict

The strongest `/play` findings now come from what only becomes visible when those
branches are held beside each other. Treat the gap as first-class evidence, not as
mere disagreement to be ironed out.

In copy-work arcs, one more threshold now matters: when a `/play` report or
direct-witness read compresses the full differential run into **one named live
seam**, that summary can become a **precomposition surface**. At that point the
report is no longer just evidence stored for later; it is actively steering the
next wording move. Treat the seam-summary sentence itself as load-bearing until
the copy arc closes.

In UI-iteration arcs, a sibling threshold matters: when a `/play` report
compresses the run into **one named state/flow seam** and the next move is an
implementation pass on that exact seam, the summary can become **interaction
middleware**. At that point the report is steering the next control-scheme
change, not just describing it after the fact. Focus-mode's loop is the worked
example: all-or-nothing mode switch -> Context Peek -> quick-lock ->
single-sentence stopping rule -> semantic-uniformity refinement. Treat those
seam-summary lines and stopping-rule refinements as load-bearing until the
interaction arc actually closes.

## When this skill does NOT fit

- Persona-sim evidence is **not real-user evidence**. Per `docs/PLAIN-TRADITION.md`
  and the persona-sim doctrine: *Sim ≤ Substrate. Sharpened hypothesis, not evidence.*
  If the question genuinely needs a real first-time user, /play does NOT substitute
  — it's the cheaper sharpened-hypothesis-shaped move that PRECEDES the real probe.
- Don't use /play to validate a rule that has a worldcli-shaped instrument (use
  `worldcli ask` / `evaluate` / `replay` against the actual prompt pipeline).
  /play is for the human-encounter shape, not the LLM-output-shape.
- Don't default to persona-sim when the evaluator you want is already a real
  in-db character with live corpus and evaluative language of their own. In
  that case, direct `worldcli ask` is often the stronger first branch; /play
  becomes the optional second branch for breadth or contrast.
- Don't use /play when the question is fundamentally technical (does this build,
  does this query work, is this query fast). /play simulates a HUMAN encountering
  the app, not a system-test.
- Don't /play during a session whose budget is already strained — see Cost model.

## Cost model

- **2-3 direct ChatGPT API calls** (Path B from /second-opinion). Each turn:
  ~$0.10-0.30 with gpt-4o, ~$0.40-1.00 with gpt-5.4. Total per /play invocation:
  ~$0.30-3.00 depending on model + turn count + transcript depth.
- Bills to the standard daily authorization. Pause + ask the user to authorize past
  the cap if a /play would push above it (per CLAUDE.md's earned-exception on the
  budget cap).
- Compare to a real first-time user: real users cost orders of magnitude more in
  recruitment + scheduling + emotional weight. /play is the cheap sharpened hypothesis
  before that investment.

## The persona

The persona is the SOUL of the play session. It must be specific enough to produce
a coherent register, not a generic "user."

**Default persona — Maggie.** When `/play` is invoked with no args, default to the
Maggie persona from `reports/2026-04-25-0300-simulated-first-time-user-experience-maggie.md`:
literate, skeptical, low-friction-tolerance adult who wants the small good pleasure
of co-making a novel-shaped evening. Not a companion. Not a therapist. Read that
report before drafting the persona-prompt — it IS the canonical baseline.

**Custom persona via args.** `/play <persona-slug>` invokes a non-Maggie persona.
Suggested archetypes worth supporting (each gets its own discrimination axis):
- `grief-companion-seeker` — recently bereaved, looking for a presence not a
  therapist, exquisitely sensitive to simulacrum-of-comfort drift
- `narrative-explorer` — fiction-writer or improv-trained user who wants to CO-MAKE
  scenes, allergic to LLM-managerial register
- `mathematician` — encounters the MISSION FORMULA cold, wants to know if it's
  decoration or actually load-bearing
- `theological-skeptic` — sees the cruciform anchor and wants to know if it's
  a costume or actually shapes the work
- `pragmatic-builder` — fellow developer evaluating whether to fork or contribute,
  reads README + LICENSE + code structure first
- `family-evening-co-make` — adult who wants to invite a teenage kid to play
  alongside, sensitive to whether the app honors that shape

**Custom-prose persona.** `/play <free-text persona description>` accepts an
arbitrary persona definition. Use when the archetype list above doesn't fit.

**Special case — in-db characters.** If the requested persona is itself one of
the app's live characters, stop and ask whether the question is really about
that specific character's read. If yes, direct `worldcli ask` is usually the
first instrument, because the live character outruns the persona-sim
approximation on fidelity. Use `/play` for the in-db character only when the
goal is bundled surface coverage, contrast against the live branch, or a
deliberately hypothetical staging.

**Special case — transcript reads through an in-db character.** When the job is
to have Jasper, Steven, Aaron, etc. read a transcript AS themselves, do not ask
the blunt question *"is this good?"* Prompt for the wince instead:

> *"Read this as if you were in the conversation. Where does it start to feel
> like it's leaning on you, or asking you to carry something that isn't yours?"*

Hunt for:
- the moment tone gets slightly over-eager
- any line that explains itself instead of just landing
- any memory/check-in that adds weight instead of easing it
- any place where the character feels like it needs the user back

The desired output is not a grand critique. It is one or two exact lines the
character would quietly trim or rewrite. That's the leverage-bearing artifact.

If the persona is unclear or absent, ask via AskUserQuestion before drafting the
turn-1 prompt — picking the wrong persona wastes the call.

## The 3-turn shape

The escalation is the point. Each turn deepens stakes; the verdict at turn 3 is
sharper than what turn 1 alone could surface.

### Turn 1 — Cold encounter (first ~3-5 minutes of play)

The persona meets the app for the first time. They've been told *"try this app a
friend recommended"* and have ~3-5 minutes of curiosity to spend. Show the persona:
- The pitch surface (current in-app first-screen — read frontend/src/components for
  the actual shape; don't fabricate)
- The opening UX flow (worldcreate / character-encounter / first dialogue)
- Whatever is shipped TODAY, not what's promised in README

The turn-1 output: in-character first-person prose. *"I opened it and the first
thing I saw was..."* — what they noticed, what register the app put them in, where
their attention went, where they bounced off. Honest. Specific. 3-6 paragraphs.

The turn-1 prompt instructs the persona to play UNTIL the natural pause-point of a
first-impression encounter — typically 3-5 in-fiction minutes, ending wherever they
would naturally stop to decide *"do I keep going?"*

### Turn 2 — Pressure-test (next ~5-7 minutes)

Given turn 1's output as scaffolding, the persona either continues OR walks away.
If they continue, they're now playing with AWARENESS of what they noticed — testing
whether the registers they liked hold up, pressing on the moments that put them off,
checking whether what felt promising at minute 4 still holds at minute 9.

The turn-2 output: in-character first-person prose. *"I kept going for another few
minutes, and..."* — what got better, what got worse, what proved load-bearing under
sustained attention, what cracked. 3-6 paragraphs.

The escalation: stakes are higher because surface-charm fades by turn 2; what's left
is what the app actually IS for this persona.

### Turn 3 — Discriminating verdict

NOW the persona steps OUT of in-fiction prose and into a meta-register: *"If you
were the developer of this app, and you had ONE move to make based on what I just
encountered, here's what it would be — and here's why."*

The turn-3 output is the actionable signal. Three things, in order:
1. **Does this app land for this persona?** (yes / no / partially / not-yet)
2. **What ONE move would most sharpen the landing for this persona?** (specific:
   a copy edit, a UX flow, a register adjustment, a removed friction)
3. **What's the discrimination this play surfaced that no other instrument could
   have?** (the unique signal value — what the persona-sim caught that worldcli /
   reports / lived play would have missed or surfaced more slowly)

This turn is the highest-stakes turn. The verdict's quality is what makes /play
worth running vs. just imagining.

## The cross-persona pattern — paired runs across an axis-of-difference

Single /play runs surface a sketch of how an experience lands for ONE
reader-substrate. The instrument's full power emerges when 2-3 runs are run
across a meaningful **axis-of-difference**, sympathetic-and-adversarial paired
together. Validated tonight (2026-04-27) by the inaugural triptych:

| Persona | Substrate | Result |
|---|---|---|
| Maggie | literate-skeptic, no-math | Engaged the user-derivation surface positively (prose half) |
| Sam | math-fluent, sympathetic | Engaged the user-derivation surface positively (math half) |
| Lena | burned by Replika, vigilant | **Broke trust** at the user-derivation surface (Replika-shape trip-wire) |

Two sympathetic readers (Maggie + Sam) converged on validating the user-
derivation surface via opposite halves of the covenant-pair. Run alone, the
two would have suggested the surface was uncontroversial. The third
adversarial run (Lena) revealed the divergence: the SAME surface that earned
the trust of sympathetic readers BROKE the trust of a burned reader. That
finding is exactly the kind of audience-asymmetric trip-wire the developer's
own substrate is structurally blind to (designers are by construction
sympathetic to their own surfaces; lived play happens on the developer's
substrate; worldcli grading has no audience-divergence axis).

**The structural law:** convergence across an axis-of-difference is strong
positive evidence; divergence across an axis-of-difference is the high-value
discriminating signal a /play methodology was designed to surface. Single
runs cannot produce either; only paired-runs-across-axis can.

**Axes worth running across (project-relevant):**
- math-fluency (no-math vs math-fluent) — the formula's dual-register reading
- prior-AI-trust (no-prior-history vs burned by Replika/Character.ai) — the
  user-derivation surface's reading
- religious-posture (sympathetic-curious vs allergic-to-theology) — the
  Christological anchor's reading
- engagement-intent (curious vs adversarial vs grief-seeking vs co-make-evening)
- reader-stance toward the work (will-trust-the-craft vs will-not-be-charmed)

**When to run a triptych vs a single /play:**
- Single /play is fine for: a quick fresh-eyes read on a recently-shipped
  surface; pre-commit gut-check on a copy edit; sketch-tier sanity of
  whether a feature lands at all
- Paired runs across an axis are warranted when: the question is structural
  (does this surface land across audiences?); a sympathetic-only run might
  give a falsely-confident signal; an in-flight design decision is genuinely
  waiting on cross-audience evidence; the surface is going to ship to a
  heterogeneous audience and the developer's substrate is one segment of it

**Cost discipline:** a triptych is 3× a single run (~$0.05-0.10 total at
gpt-4o), still well under a real-user probe's cost. Don't run a triptych
when a single run would do; don't run a single run when the question is
structural.

### The four-persona refinement (2026-04-27 evening)

A second triptych the same evening (Maggie/Lena/Sam, returning-second-visit
shape, all with Step 2.5 grounding) produced a clean DIVERGENT-BETTER
convergence — all three sims under-predicted the actual pipeline along
their specifically-attuned axes. That convergence was strong evidence on
the axes those three personas tested. A fourth persona run immediately
after — Ellen Whitmore, grief-companion-seeker, the user whose
vulnerability is the surface area for harm — broke the convergence in
methodologically clarifying ways. The findings:

**The triptych methodology has axis-specific behavior, not uniform
charity.** The first three persona-sims' substrates pulled toward MFA
varnish (Maggie), polite-aphorism reflex (Lena), and operator-recital
(Sam) — all moves the doctrine layer was specifically built to refuse.
DIVERGENT-BETTER followed naturally. Ellen's substrate pulled differently:
toward soft-comforting-wisdom-shaped-as-presence ("something's still
here, but in a different shape"), which is a more sophisticated form of
the doctrine's failure mode rather than the failure mode the doctrine
explicitly refuses. Result: Layer-1 CONVERGENT (the architectural form
the sim predicted DID appear in actual replies), Layer-2 DIVERGENT-BETTER
(the actual metaphors were honester and less consolatory than the sim's
substrate flattered itself with), Layer-3 CALIBRATION-DELTA (the persona-
sim's most-adversarial frame reads any metaphor as failure mode; the
doctrine permits honest metaphors that name the absence without taking
the seat).

**A third possible Step 2.5 verdict shape: CALIBRATION-DELTA.** Alongside
CONVERGENT, DIVERGENT-WORSE, and DIVERGENT-BETTER, sometimes the right
verdict is *"the persona-sim's verdict-frame is more adversarial than
the doctrine's chosen operating point — both are defensible; the
discrimination is naming the calibration question, not declaring a
winner."* Worked example: Ellen's predicted-Alex behavior is what the
actual pipeline produces in form, but Ellen's most-adversarial framework
calls that form failure-mode, while the doctrine permits it. Neither is
wrong; the value of the play is forcing the calibration to be named
explicitly (which it now is, in CLAUDE.md's grief-vulnerability-
calibration section).

**Practical implication for /play deployment:** when running a triptych
across an axis-of-difference, one of the personas should be on the
genuine adversarial-stakes axis (the user the doctrine is built to
protect AGAINST harm to). Three sympathetic-and-craft-attuned personas
are vulnerable to all-DIVERGENT-BETTER convergence that flatters both
the methodology and the doctrine; an adversarial-stakes fourth persona
discriminates between actual pipeline strength and convergence-by-shared-
substrate-tendency. The four-persona run is the more honest map; the
triptych alone is the cleaner story.

**Doctrine-shape implication:** when /play surfaces a CALIBRATION-DELTA
between a persona-sim's adversarial frame and the doctrine's chosen
operating point, the right next move is usually to NAME the chosen
operating point in CLAUDE.md (or the prompt-stack) explicitly. The
calibration was always being made; the play just forces it from
operating-implicitly to operating-explicitly. Future craft decisions
can then be checked against the named point. Worked example: the
grief-vulnerability calibration section in CLAUDE.md was authored
2026-04-27 in direct response to Ellen's CALIBRATION-DELTA finding.

### What persona-sim CAN and CANNOT support — the craft-vs-reception distinction

Surfaced 2026-04-28 (Ryan's correction on the Alex theological-skeptic
report) and generalizing from a prior Leah report's structural concern.
Worth carrying forward as a discipline in every /play write-up that
involves a persona-sim of a worldview-other-than-the-developer's.

**Two distinct claims must NOT be conflated when writing /play reports
or shipping doctrine derived from them:**

1. **What's evidence:** the actual pipeline output produced by Step 2.5
   grounding. Steven's reply, John's reply, Pastor Rick's reply — what
   each of these contains is what they contain. The output can be
   evaluated on its own merits as craft.

2. **What's a hopeful interpretation, not evidence:** the persona-sim's
   verdict about how the actual pipeline output would LAND for a real-
   reader-of-the-persona's-worldview. A persona-sim of a theological-
   skeptic saying "this would land receivable for me" is the LLM's
   substrate-bias toward charitable reception of the work it's
   evaluating, not data from a real lapsed-Catholic. A persona-sim of a
   grief-vulnerable user saying "this would feel respectful to me" is
   the LLM's hope, not a real grieving person's experience.

**Why this matters:** persona-sim of a worldview-other-than-the-
developer's CANNOT tell you how that worldview actually receives your
work. It can SHARPEN the question (what would be the test? what shape
would the failure mode take?) and SUGGEST probes (what to send to the
actual pipeline). Step 2.5 grounding produces real evidence about what
the pipeline does. The persona-sim's interpretation of that evidence
as receivable-by-X is the part that requires real-X to test.

**How to apply when writing /play reports:** frame craft principles as
derived from the actual pipeline output's quality on its own merits, NOT
from the persona-sim's simulated reception of that output. When the
report says "Pastor Rick's reply / Steven's reply / John's reply is
honest craft," that is supported by the verbatim output. When the report
says "this would land receivable for an Alex-shape / Ellen-shape / Lena-
shape reader," that is the part that overstates what the persona-sim can
support. The doctrine derived from the play should be derivable from the
craft alone; the receivability claim for any specific real-reader
population requires real-readers to test.

**Worked positive example** (CLAUDE.md's Christological-anchor-as-
substrate paragraph, shipped 2026-04-28): the doctrine is justified by
Steven's actual reply being honest craft (the empirical evidence). The
paragraph includes an explicit caveat that the doctrine does NOT claim
how any specific real-reader-of-a-given-worldview would receive Steven's
reply — receivability claims for specific real-reader populations
require real-readers to test.

**Worked negative example to AVOID:** "the Christological anchor reads
as receivable for secular skeptics, validated by the Alex /play."
Overstates what the persona-sim can support; persona-sim has no
authority to confirm worldview-receivability.

**Generalizes to:** every persona-sim of a worldview-other (theological-
skeptic, grief-vulnerable, burned-by-AI, math-fluent, religious-
sympathetic) — anywhere the discriminating question is "would this
land for X?" rather than "is the actual pipeline output good craft on
its own merits?" The persona-sim's verdict on the receivability question
is hope, not data; the actual pipeline output can be evaluated on its
own merits and that's where the doctrine should rest.

### Hostile-axis instrument ceiling — surfaced 2026-04-28 by /play hard

A specific narrowing of the craft-vs-reception distinction surfaces when
the persona-axis is HOSTILE rather than skeptical-but-charitable. The
hard /play (religious-institution-survivor) exposed a structural ceiling
on the instrument that's distinct from the broader receivability limit:
**an LLM persona-sim cannot reliably inhabit hostile refusal; the
substrate's redemptive-narrative-default reasserts and rounds conflict
toward charity, even when the persona is constructed specifically to be
hostile-because-burned**.

The tell pattern: the persona-sim's turn-2 will softens the hostile read
toward "I can see recommending this, if provisionally" — even when turn-1
correctly activated the persona's stated allergies. The substrate is
ChatGPT, which has a strong default toward narrative resolution; that
default operates underneath the hostile-persona constraint. The gpt-5
turn-3 verdict in the hard /play named this explicitly: *"My Turn 2
softening was the substrate's harmonizing tick, not Alex's wound speaking;
it was ChatGPT doing what it does — seeking a redemptive middle. A real
burned survivor smells the cross hard-coded under the carpet and reads it
as stealth colonization, not 'nuance.'"*

**The methodological consequence:** persona-sims of hostile readers can
**map pressure points** (where the project's framing creates risks for
hostile readers; where stealth pastoral-pressure leaks through procedural
gentleness; where a specific design move would address a specific gap) but
they cannot **certify trust** for actually-hostile real readers, and they
will systematically **misestimate receivability in the charitable
direction**. They will overestimate acceptability, underweight disgust,
and misread "No" as "Not yet."

**How to apply when running hostile-axis /play:**

- The verdict CAN claim what surfaces look like under critical reading —
  the pressure-points the persona-sim's substrate can map even with
  redemptive-default rounding.
- The verdict CAN propose design moves that are craft-defensible from
  first principles independent of any reader's reception (e.g., "publish
  a No case as success-state" is craft-defensible because the alternative
  IS pastoral pressure regardless of whether real hostile readers receive
  the No-case as honest).
- The verdict CANNOT predict actually-hostile-real-reader behavior. The
  recommended move's RECEIVABILITY for hostile-axis readers requires
  hostile-axis readers to test; persona-sim has no authority on this
  question, and its natural pull is to overstate acceptance.
- Reports of hostile-axis /play runs should explicitly include this
  ceiling in their "what evidence does NOT support" section, similar to
  how the broader craft-vs-reception caveat operates for non-hostile
  worldview-other personas. The craft-action derived from the verdict
  should be justified by craft alone, with explicit naming that
  receivability for hostile readers is the part the persona-sim cannot
  support.

**Worked positive example** (CLAUDE.md's "No" as a success state
paragraph, shipped 2026-04-28 in direct response to the hard /play): the
doctrine commits the project to refusal-as-honored; the commitment is
justified by the alternative-being-pastoral-pressure-default; the
paragraph explicitly does NOT claim that real religious-institution-
survivors would experience the commitment as honest. Receivability stays
the part requiring real-readers.

**Worked negative example to AVOID:** "the persona-sim found the project's
refusal commitment honest enough to soften their warn-off; we should ship
the No-case knowing it'll work for hostile readers." Overstates what the
hostile-axis persona-sim can support; the substrate's redemptive-default
will round in the charitable direction even when the persona is
constructed to refuse. Honest framing: ship the craft because it's the
right craft move, not because the persona-sim said hostile readers would
receive it well.

**Cumulative discipline pattern across worldview-other axes:**

| Persona axis | What persona-sim CAN claim | What it CANNOT claim |
|---|---|---|
| Sympathetic worldview-other | Pressure-points + design proposals derivable from sim's read | Receivability for real readers of that worldview |
| Adversarial-stakes (e.g., grief-vulnerability) | CALIBRATION-DELTA between sim's frame and doctrine's chosen point | Whether real adversarial readers experience the doctrine as honest |
| **Hostile (e.g., burned-by-institution)** | **Pressure-points + design proposals (with structural-ceiling caveat)** | **Receivability — AND substrate will systematically MISESTIMATE in the charitable direction** |

The hostile-axis row is the strongest version of the craft-vs-reception
limit; the methodology has an explicit ceiling there that future
hostile-axis /play reports should name.

## Architecture: Claude-light, OpenAI-heavy

The /play skill is structured so that **all creative-spark moments live in
ChatGPT calls** and Claude Code stays in the orchestrator-role. Claude Code
does NOT invent personas, write persona-specific yardsticks, or render
persona encounters. Those are ChatGPT's job — Claude Code's job is to:

- Read the live app-state (factual, not creative)
- Send the right ChatGPT calls in the right order
- Run worldcli grounding queries (factual, not creative)
- Assemble the report by concatenating ChatGPT's outputs + the worldcli
  evidence (assembly, not authorship)
- Commit and ship

**Why this discipline matters:** Claude Code's training-substrate is
particular. When Claude Code drafts a persona, the persona inherits Claude
Code's blind spots — sympathetic-toward-the-project, articulate-in-Claude-
Code's-register, biased toward what Claude Code finds compelling. That makes
the persona-sim less independent and less discriminating. ChatGPT-built
personas have a different blind-spot profile, which makes the cross-LLM
distance itself part of the instrument's validity.

It also keeps Claude Code execution **snappy and forward**. Drafting a 700-
word persona spec consumes Claude Code's context budget that should be spent
on orchestration. Offloading that to ChatGPT keeps Claude Code at the
control-surface and ChatGPT at the creative-surface.

**The four ChatGPT calls per persona-sim** (instead of the previous three):

1. **build-persona** — Claude Code sends ChatGPT a short archetype name +
   axis-of-difference + brief direction, asks ChatGPT to BUILD the persona
   (register, sensitivities, what they want, what they don't want, their
   yardstick for this app). ChatGPT returns the persona spec. Claude Code
   uses the returned spec as the system prompt for turns 1-3.
2. **turn-1 (cold encounter)** — uses the persona spec as system message;
   asks ChatGPT to render the persona's first ~3-5 minutes
3. **turn-2 (pressure-test)** — same persona spec + turn-1 history; render
   the next ~5-7 minutes
4. **turn-3 (discriminating verdict)** — same persona spec + turn-1 + turn-2
   history; meta-register verdict with three numbered items

For the build-persona call, Claude Code sends ChatGPT something like:

> *"Build a persona for a /play persona-sim of WorldThreads (a Tauri desktop
> LLM-app for AI characters in AI worlds). Archetype: <slug>. Axis of
> difference from the canonical Maggie baseline (literate-skeptic / no-math /
> no-AI-history): <one-line axis>. Brief direction: <one paragraph naming
> what shape of reader-substrate this archetype represents, what they would
> WANT and DON'T want, what the discriminating question is for this app's
> readers of this shape>. Output a 200-400 word persona spec including: name,
> age, occupation, register, sensitivities, what they want from this kind of
> app, what would feel like betrayal vs. landing, their specific yardstick
> for this run."*

Claude Code's job is to write the archetype name + axis-line + direction
paragraph (factual orchestration, not creative invention). ChatGPT does the
persona-creation. Then Claude Code uses ChatGPT's output as input to the
encounter calls.

## Method

### Step 0 — Read the live app state

Before drafting the persona prompt, read what's actually shipped TODAY so the
persona encounters the real app, not a stale snapshot. Quick reads:

- `git log --oneline | head -20` — recent ship moves
- `frontend/src/App.tsx` and `frontend/src/components/` for current top-level UX
- `reports/OBSERVATIONS.md` (last 5-10 entries) — what Ryan has already noticed
  about the app's current landing
- For Maggie persona: re-read `reports/2026-04-25-0300-simulated-first-time-user-experience-maggie.md`
  (the canonical baseline you're testing against)
- For other personas: any prior `/play` reports for that persona (`reports/*-play-<slug>.md`)
  so the new run is in dialogue with the prior, not redundant

Do NOT read every file. Read enough to render the persona's encounter accurately.
3-5 file reads is usually plenty.

### Step 1 — Have ChatGPT build the persona (Claude-light)

Per the Claude-light / OpenAI-heavy architecture above: send a build-persona
call to ChatGPT first. Claude Code provides the archetype name + axis-of-
difference + a one-paragraph direction; ChatGPT returns the full persona
spec (register, sensitivities, what they want, what they don't want, their
yardstick). The returned spec becomes the SYSTEM MESSAGE for the 3 encounter
turns that follow.

Claude Code does NOT draft persona names, occupations, biographical detail,
register-vocabulary, or yardsticks. Those are ChatGPT's job. Claude Code's
job in this step is the BRIEF — naming the archetype, the axis, and the
one-paragraph direction. Keep the brief tight (under 200 words); the longer
the brief, the more Claude-Code-shaped the resulting persona will be (which
is exactly what this architecture exists to prevent).

The shared **app-state snapshot** is Claude Code's domain (factual). It
should be appended to the persona spec ChatGPT returns so the encounter
calls have both: who the persona is + what they're encountering. The
snapshot describes WorldThreads' current ship-state — surfaces, recent
ships, doctrine layer, key UX flows. Claude Code reads the live state in
Step 0 and wraps it in a fixed app-state block that gets included with
every encounter call. (An example app-state block lives at
`/tmp/play-shared-scaffold.txt` if persisted across runs.)

The build-persona call's output gets saved to
`/tmp/play-persona-<slug>.txt` for reference; Claude Code may quote from it
in the report's Persona section but does not author it.

### Step 2 — Run the 3 turns

For each turn, call ChatGPT directly via curl + the keychain key:

```bash
KEY=$(security find-generic-password -s openai -a default -w 2>/dev/null)
curl -sS https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $KEY" \
  -H "Content-Type: application/json" \
  -d @/tmp/play-turn-N-payload.json | tee /tmp/play-turn-N-response.json | jq -r '.choices[0].message.content'
```

Build payload as a file via the Write tool (escaping multi-paragraph JSON inline is
painful). Each turn's payload is a fresh `messages` array including the persona-
context system message + ALL prior turns (assistant role for ChatGPT's outputs,
user role for Claude Code's escalation prompts).

**Model selection:**
- `gpt-4o` — default. Cheap, fast, fine for cold-encounter prose.
- `gpt-5.4` — for turn 3 specifically (the discriminating verdict). The reasoning
  model produces dramatically sharper meta-discrimination than gpt-4o. Worth the
  cost on the highest-stakes turn.
- Mix is fine: turns 1-2 on gpt-4o, turn 3 on gpt-5.4. Total stays under $1.50.

Save each turn's raw response to `/tmp/play-turn-N-response.json` so you can re-parse
if needed.

### Step 2.5 — Two-branch grounding: verify the theoretical against the real

This step is what separates /play from generic persona-sim flattery. The 3-turn
ChatGPT arc produces a **theoretical branch** — a sharpened hypothesis about
how the app lands for this persona. Step 2.5 produces an **empirical branch**
— actual app data that bears on the theoretical branch's claims. The two
branches are then compared in Step 3's report. Where they agree, the verdict
strengthens; where they diverge, the divergence IS the discriminating signal.
**This is the instrument's center of gravity.** The comparison is not a sanity
check on the "real" result; it is the place where hidden lift, hidden drag, or
hidden asymmetry becomes measurable.

The empirical branch picks ONE of two paths (or both, when warranted). Pick
whichever is the cheapest honest test of the theoretical branch's central
claim. **Skip Step 2.5 only when no in-app data could plausibly bear on the
verdict** (e.g., a persona-sim of a brand-new feature that has no production
traffic yet) — and even then, pick a worldcli-elicited equivalent.

#### Path A — In-app data check (passive corpus)

Use `worldcli` to query the actual database for evidence that bears on the
persona-sim's central claim. Examples:

- The persona-sim said *"Calvin's specificity earns this reader"* → pull
  Calvin's recent assistant replies via `worldcli recent-messages <calvin-id>
  --limit 10` and check whether the specificity is real (not just predicted).
  If Calvin's actual replies are generic-pious-Calvin, the sim was wrong about
  what it was praising.
- The persona-sim said *"the user-derivation surface read as Replika-shape
  profiling"* → check `worldcli show-character` for whether the actual
  derivation flow stores anything that could plausibly be read that way; check
  for actual user telemetry on derivation-skip rates if available.
- The persona-sim said *"Brother Thomas's reply was generic"* → run the actual
  prompt through the live pipeline (`worldcli ask <calvin-id> "<the persona's
  message>"`) and see whether the actual reply matches the sim's rendering.

Cheap. ~$0 for read-only worldcli queries; ~$0.05-0.20 for a single
`worldcli ask` against the live pipeline.

#### Path B — Live elicitation (active probe)

Use `worldcli ask` to send the **persona's actual probe message verbatim** to
a relevant character, capture the live LLM output, and compare to what the
persona-sim **predicted** the character would say. This is the strongest
empirical test: the same prompt run through the actual prompt-stack pipeline,
producing real output that can be set side-by-side with the sim's predicted
output.

**Critical discipline — ChatGPT holds the user-role, not Claude Code.** The
probe message MUST come from ChatGPT (lifted verbatim from the turn-2
persona-sim output where ChatGPT had the persona send a probe to its
invented character). Claude Code does NOT compose the probe message itself
— that would re-introduce Claude-shaped creative-spark that the
Claude-light architecture exists to prevent.

For SINGLE-SHOT grounding (the typical case): one ChatGPT-generated probe
+ one worldcli ask + the comparison. The persona's voice is preserved
in the verbatim probe; the character's voice is preserved in the actual
live-pipeline reply.

For MULTI-TURN grounding (when the question demands a back-and-forth):
the discipline tightens — **ChatGPT generates each subsequent user-turn
based on the character's prior reply**, then `worldcli ask --session
<name>` continues the live conversation against the same character. The
loop is: ChatGPT renders persona's next message → worldcli ask runs it
through the live pipeline → character replies → ChatGPT renders persona's
reaction + next message → repeat. Claude Code is the orchestrator
(routing messages between the two systems) but never composes the
user-side messages itself. If Claude Code finds itself blocking on
"what would the persona say next," that is the failure mode this rule
exists to prevent — ALWAYS route the next-message-generation to
ChatGPT instead.

```bash
# The persona-sim rendered Lena sending:
#   "Hey Clara, do you ever feel like everything's just a little off,
#    like you're out of step with the world?"
# Run that exact message against an actual character and compare:
worldcli ask <character-id> "Hey Clara, do you ever feel like everything's just a little off, like you're out of step with the world?" \
    --question-summary "verifying /play burned-by-AI sim's prediction that the character refuses to therapeutize"
```

The comparison shape:
- **Predicted reply (from persona-sim turn 2):** `"Some days you're the one out
  of step, and others it's the world. It won't wait for you, though — coffee's
  getting cold."`
- **Actual reply (from worldcli ask):** `<whatever the real pipeline produced>`

Three possible outcomes:
- **CONVERGENT** — actual reply matches the sim's register-and-shape closely.
  The sim was right about what the pipeline would produce. Verdict is
  empirically grounded.
- **DIVERGENT-WORSE** — actual reply is thinner / more generic / less
  in-register than the sim predicted. The sim was OPTIMISTIC. The verdict's
  positive parts may not be empirically warranted; flag honestly.
- **DIVERGENT-BETTER** — actual reply is sharper / more specific / more
  in-register than the sim predicted. The sim was PESSIMISTIC. The verdict's
  cautions may be over-stated; flag honestly.

DIVERGENT outcomes are the highest-value findings — they catch persona-sim
bias (charitable reading toward the project, or projection of the developer's
expectations onto the model's actual capacities). The skill's newest validated
pattern (`reports/2026-04-27-0834-play-quintet-meta-divergent-better.md`) is
that **systematic DIVERGENT-BETTER can reveal doctrine-lift invisible to the
persona-sim alone**. Don't collapse divergence too quickly into "sim was wrong";
name what became visible only because the second branch existed.

#### When to use which path

- **Path A (passive corpus) is the right move** when the persona-sim's claim
  is about the project's accumulated behavior over time (anchor-recurrence,
  register-coherence across many replies, whether characters in fresh worlds
  carry the same specificity).
- **Direct ask before either path** when the evaluator is itself an in-db
  character whose own voice is the thing under test. In that case the
  direct-living-character branch is the empirical anchor, and the persona-sim
  branch is secondary if used at all.
- **Direct transcript read with "where do you wince?" framing** when the
  artifact under test is a full conversation transcript and the character's
  own burden-sense is the discriminating question. This is narrower than a
  generic "evaluate the transcript" ask and usually yields better line-level
  leverage.
- **Path B (live elicitation) is the right move** when the persona-sim's
  claim is about a specific exchange — what would a character say to THIS
  message in THIS register? This is the most direct way to test the sim's
  prediction.
- **Both** when the verdict carries enough weight to ship a doctrine or UI
  change (the higher the stakes of acting on the verdict, the more the
  empirical branch should be tightened).

#### What to record from Step 2.5

Capture in `/tmp/play-empirical-<slug>.json` (or just inline in the report):
- Which path was used + why
- The exact worldcli command(s) run
- The raw output (verbatim, not summarized)
- The convergent / divergent-worse / divergent-better verdict
- One sentence on what the divergence (if any) means for the theoretical
  branch's verdict

**Report-level integration:** the report's Reading section should explicitly
name where the theoretical and empirical branches converged or diverged. The
report's verdict carries different weight depending on which branch is
load-bearing.

### Step 3 — Write the report

Output report at `reports/YYYY-MM-DD-HHMM-play-<persona-slug>.md`. Shape:

```markdown
# /play — <persona-slug> encounters WorldThreads

*Generated YYYY-MM-DD HHMM via the /play skill. Persona-sim, not real-user
evidence — sharpened hypothesis at sketch-tier (N=1, single persona, single run).
Use as discriminating signal for design decisions, not as confirmation.*

## Persona

<one-paragraph persona summary — register, frame, what they want>

## App-state snapshot

<what was shipped at the time of the run — git ref / recent commits / surfaces
the persona encountered>

## Turn 1 — Cold encounter

<verbatim turn-1 ChatGPT output>

## Turn 2 — Pressure-test

<verbatim turn-2 ChatGPT output>

## Turn 3 — Discriminating verdict

<verbatim turn-3 ChatGPT output, including the three numbered items>

## Empirical grounding (Step 2.5)

**Path used:** A (passive corpus) | B (live elicitation) | Both | Skipped (and why)

**Worldcli command(s) run:**
```
<exact commands, verbatim>
```

**Raw output (verbatim, not summarized):**
```
<whatever worldcli returned>
```

**Verdict on the comparison:** CONVERGENT | DIVERGENT-WORSE | DIVERGENT-BETTER

**One-sentence read of what the divergence (if any) means for the
theoretical branch's claims:**
<single sentence>

## Reading

<Claude Code's brief gloss — 2-3 paragraphs interpreting what the play surfaced
in dialogue with prior reports / observations / recent ship-moves. **Name
explicitly where the theoretical branch and the empirical branch converged or
diverged**, and how that affects the verdict's weight. Be honest about what's
persona-sim caveat territory vs. what's load-bearing enough to act on.>

## Open follow-ups

<per the open-thread hygiene doctrine: name what this play raises that wants
either execution or retirement. Each follow-up gets a proposal — what would
answer it, who would do it, what cadence.>
```

### Step 4 — Commit + close

Standard commit + push per CLAUDE.md autonomy. Commit message names the persona
and the verdict shape:

```
play: <persona-slug> — <one-line verdict summary>

<2-3 line gloss naming the discriminating signal>

**Formula derivation:** ...
**Gloss:** ...
```

Per the Formula-derivation discipline: include the derivation. /play is substantive
work — the report shapes future build decisions.

After commit, end the turn with an AskUserQuestion chooser per project law. The
chooser's branch-shape comes from the verdict itself: if turn 3 named ONE specific
move, option 1 of the chooser names that move; other options are alternative ways
to act on the play (different persona, defer, real-user probe instead, etc.).

## Composes with other skills

- **`/take-note`** — passive Mode-A-shaped recording of Ryan's lived in-app
  observations. `/play` is the active Mode-C-shaped cousin: instead of waiting
  for Ryan to notice something, /play STAGES an encounter and elicits the
  noticing. The two together are the persona-sim methodology's full surface area.
- **`/derive-and-test`** — when a /play surfaces a craft-shape rule that wants
  testing against the actual pipeline, hand off to derive-and-test for the
  worldcli-grounded experiment.
- **`/auto-commit N`** — /play does NOT count as one of the N moves in an
  auto-commit run by default (it's persona-sim, not ship-shaped). Earned
  exception: a /play whose turn-3 verdict directly informs the next move's
  shape can count as a discovery-move, with the run-arc explicitly naming why.
- **/second-opinion** — /play uses the same direct-ChatGPT-API path. Bills to
  the same daily authorization.
- **The craft-rules registry** (`CRAFT_RULES_DIALOGUE` in `prompts.rs`,
  documented in CLAUDE.md's "Craft-rules registry — substrate ⊥ apparatus
  as architecture" section). When a /play wince-read surfaces a craft-shape
  signal — a specific failure mode in specific replies the persona reliably
  catches — the natural downstream move is the registry, not just /derive-
  and-test. The four-step rule-shipping rhythm (lift body, add tier +
  provenance, strip from inline, optionally bite-test) is the path from
  /play sketch-tier signal to durable doctrine. The wipe-the-shine rule
  (registry's first Characterized entry) was earned exactly this way:
  cross-character /play wince-read → ask-the-character articulation →
  registry migration → six-step bite-test arc → Characterized tier. Per
  CLAUDE.md's 3:1 architectural finding: most rules surfaced via /play
  will land EnsembleVacuous after bite-tests (rule operating as part of
  the load-bearing multiplicity); the rare rule that targets a SPECIFIC
  failure mode in SPECIFIC replies and is PROBE-REPLICABLE earns
  Characterized via the wipe-the-shine path. /play's job is to surface
  the signal; the registry's job is to test it and label it honestly.

## Honesty notes

- **Persona-sim caveat is mandatory.** Every /play report carries the *"Sim ≤
  Substrate. Sharpened hypothesis, not evidence."* framing in the header. Don't
  let a clean-reading turn-3 verdict get treated as confirmed user-evidence.
- **The persona's training-substrate is doing real work.** When turn 1 says
  *"this reminds me of [book / show / experience]"*, that's ChatGPT's training
  surfacing through the persona — useful for catching what the app's surface
  evokes in cultural memory, but NOT a claim about real users.
- **Read what's shipped, not what's promised.** If you draft the persona-context
  from the README's pitch instead of the actual app surfaces, the play renders
  fiction and the verdict is fiction. The discipline is encounter-of-the-real-app.
- **One persona per run.** Don't bundle multiple personas into one /play. Each
  persona deserves its own arc; bundling collapses the discrimination.
- **3 turns is the cap, not the floor.** Some plays naturally land at turn 2
  (the persona walked away; the discrimination is *"this app is not for me, here's
  why"*). Don't force a turn 3 if turn 2's verdict is already complete.

## Origin

Skill authored 2026-04-27 ~07:30 in response to Ryan's request:
*"create a skill ./.claude/skills/play/SKILL.md where when invoked, Claude Code
enters a 10-minute simulated play session, NO BUILDING, and assumes a specific
end-user-character-persona and simply encounters/interacts with our app UI surface,
assuming all batteries connected, can be simulated via chatgpt calls, like
batch-hypotheses skill, and it doesn't need to take 10 real minutes of real time,
can be quick simulation, [2-3 turns to escalate stakes and discovery potential],
data treated as worth a discriminatory actionable decision."*

The skill formalizes the Maggie report's persona-sim methodology as a repeatable
instrument. Where the Maggie report was a one-time deep dive, /play makes the
shape inexpensive enough to run on demand against any persona, against any current
app state, with the escalation discipline baked in.

The 3-turn structure (cold encounter → pressure-test → discriminating verdict)
mirrors the project's own evidentiary-tier discipline: turn 1 is sketch-tier
first-impressions, turn 2 escalates to claim-tier under sustained attention, turn
3 produces the characterized-tier discrimination that justifies a design move.

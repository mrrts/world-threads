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

## When this skill does NOT fit

- Persona-sim evidence is **not real-user evidence**. Per `docs/PLAIN-TRADITION.md`
  and the persona-sim doctrine: *Sim ≤ Substrate. Sharpened hypothesis, not evidence.*
  If the question genuinely needs a real first-time user, /play does NOT substitute
  — it's the cheaper sharpened-hypothesis-shaped move that PRECEDES the real probe.
- Don't use /play to validate a rule that has a worldcli-shaped instrument (use
  `worldcli ask` / `evaluate` / `replay` against the actual prompt pipeline).
  /play is for the human-encounter shape, not the LLM-output-shape.
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

### Step 1 — Draft the persona-context block

Build a single shared persona-context that turns 1-3 will all reference. It must
include:

- **Persona definition** — register, frame, sensitivities, what they want, what
  they don't want, what would feel like betrayal vs. what would feel like landing
- **App-state snapshot** — what's shipped today, what surfaces the persona will
  encounter, what is and isn't currently working
- **The yardstick** — for Maggie: does the refusal moment, specific-memory anchoring,
  earned close stay intact; does simulacrum-therapy drift stay out. For other
  personas: name the equivalent persona-specific yardstick.

Save this block as the system message for turn 1.

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
expectations onto the model's actual capacities).

#### When to use which path

- **Path A (passive corpus) is the right move** when the persona-sim's claim
  is about the project's accumulated behavior over time (anchor-recurrence,
  register-coherence across many replies, whether characters in fresh worlds
  carry the same specificity).
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

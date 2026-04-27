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

## Reading

<Claude Code's brief gloss — 2-3 paragraphs interpreting what the play surfaced
in dialogue with prior reports / observations / recent ship-moves. Name the
unique signal this run produced. Be honest about what's persona-sim caveat
territory vs. what's load-bearing enough to act on.>

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

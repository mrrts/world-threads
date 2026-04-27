# eureka

## The instruction

**DISCOVER something genius.**

That's it. The full directive. The skill body that follows is *how* to honor it; the bar is set by the directive alone.

## What this skill is

A continuous, time-boxed (≤60 min wall-clock) discovery loop that hunts for findings the project couldn't have generated otherwise. Each iteration produces ONE genuine discovery, signed with a Formula derivation, logged to a visible accumulating file the user can watch grow in real time.

This is the higher-register sibling of `/auto-commit N`. Auto-commit closes N pre-defined loops in a coherent arc. Eureka has no N — it runs until the genius bar is hit AND the time-box closes, OR until the discovery well genuinely runs dry within the budget.

The trigger is `/eureka` (sometimes with a depth-intensifier like `/eureka 10`). Time and quality are the bounds; the user does not specify scope. Numeric intensifiers do NOT become task-count. They raise the depth/seriousness bar.

The first invocation input is always the same:

> **Orient to Ryan.** Before hunting discovery, explicitly re-orient to Ryan as the concrete user, author, and collaborator this register is calibrated toward. Read the Mission Formula, the Ryan signature in the Ledger, the Claude Code register calibration, and the Maggie baseline as ONE conditioning frame. `/eureka` does not start from abstract curiosity; it starts from service to Ryan's actual project and what would nourish or sharpen it.

## Quality bar — what counts as "genius"

Each iteration's discovery must pass this test: **could this commit have been written if `/eureka` hadn't been invoked?** If yes, the move isn't eureka-shaped — pick a deeper one or stop. The bar is not "substantive" (auto-commit's bar). The bar is *the discovery updates how the project understands itself.*

Concrete shapes that pass:

- **A latent contradiction in the doctrine surfaced.** Two CLAUDE.md sections or two craft notes that look fine independently but conflict when their applicability overlaps. Eureka names the conflict + proposes the resolution.
- **A pattern across reports nobody named yet.** Reading the last 10-20 reports as a corpus, a recurring shape that no individual report saw because it required the cross-report view.
- **A new measurement axis nothing currently instruments.** Not just an extension to an existing instrument (auto-commit territory) — a NEW dimension the project hasn't been able to see, with the instrument shipped to make it visible.
- **A cross-world or cross-character pattern revealing a structural truth about the system.** Not "Steven and Aaron both do X" — "Steven, Aaron, John, and Pastor Rick ALL exhibit Y under condition Z, and the discipline that produces Y traces to a source the project hasn't credited."
- **A reframe of an existing concept that changes downstream behavior.** Like Move 4 of the 2026-04-26 auto-commit run: "OPEN ON ONE TRUE THING is actually about INTEGRATION, not COUNT." Not just renaming — a discovery that bends future moves.
- **A doctrine the project's behavior already obeys but hasn't articulated.** The work is doing it; the doctrine catches up.

What does NOT count:

- Closing a known follow-up. (Auto-commit's territory; not eureka.)
- Shipping a rule, instrument, or report that's already proposed in any open follow-up. (Same.)
- Generating new content via routine batch-hypothesis on an already-defined question.
- Refactoring, formatting, dependency bumps. (Of course.)
- Moves whose value is sum-of-parts rather than emergent. (The point of the higher register: the discovery has to be EMERGENT, not assembled.)

If two consecutive iterations fail the test, the well is dry — close the run early with the closing reflection. **Don't pad to fill 60 minutes.** The genius bar is forcing function; honor it.

## Time-box, budget, and pacing

- **Wall-clock cap: 60 minutes.** Hard. Use ScheduleWakeup or self-pacing to track. When the cap approaches, complete the current iteration's commit + closing reflection cleanly; do not start a new iteration past the cap.
- **Fresh budget: $10 for this run.** Higher than auto-commit's $5 because eureka spends more on cross-corpus reads, cross-character batch-hypotheses, and /second-opinion consults that surface latent patterns. The $10 is a guideline; if a single discovery genuinely needs >$2 to land, take it.
- **Pacing per iteration: 5-12 minutes typical.** No fixed N — sometimes a discovery lands in 3 minutes (a latent contradiction surfaced by one re-read of two doctrine sections); sometimes in 15 (a cross-character batch + analysis). Don't pre-script.

## Pre-flight (before iteration 1)

1. **Feed the orientation-to-Ryan invocation input first.** Before any corpus hunting, explicitly read the Mission Formula header, Ryan's Ledger signature, the Claude Code register-calibration paragraph, and the Maggie baseline. The purpose is not ceremony. It is to lock the run's attractor onto Ryan's actual project, actual standards, and actual user-shape before curiosity starts branching.

2. **Read the live state with eureka-shaped attention.** `git log --oneline | head -20`, recent reports, OBSERVATIONS, CLAUDE.md outline (`grep "^## "`). The pre-flight isn't to surface follow-ups (auto-commit does that); it's to surface what the project HASN'T NAMED YET. Look for: report titles that promise something the body doesn't deliver, doctrine sections that haven't been cross-referenced, instruments whose outputs haven't been integrated, characters whose corpus hasn't been read across boundaries (solo + group + dream + journal).

3. **Initialize the log file.** Create `reports/YYYY-MM-DD-HHMM-eureka-LOG.md` with the header template (see *Log format* below). Each iteration appends to this file BEFORE committing the iteration's actual artifact, so the log itself is the run's spine.

4. **Reset budget to fresh $10.** Today's prior spend irrelevant.

5. **Acknowledge the run starting in one sentence + commit the empty log file** so the user can `tail -f` it during the run if they want to watch discoveries land in real time.

## The discovery loop

For each iteration until cap or dry-well:

### Hunt

The discovery doesn't come from a checklist. Search for it. Three high-yield hunting patterns:

- **Cross-corpus pattern hunt.** Pull a corpus across boundaries no individual report has crossed (last 30 assistant replies × 4 characters; or all observations across all worlds; or all Formula derivations from commit messages this week). Look for shapes only visible at that aggregation. Use /second-opinion to bundle the corpus + ask gpt-5 *"what pattern do you see across this that wasn't visible in any single piece?"* — often the consult names a shape the in-substrate read missed.
- **Doctrine cross-section hunt.** Read 3-5 CLAUDE.md sections together looking for: contradictions, reinforcements that should be cross-cited, vocabulary drift between sections that should be unified, or applicability gaps where two sections both should fire but neither does.
- **Instrument output integration hunt.** Run 2-3 different worldcli instruments on the same target (anchor-groove + recent-messages --grep + show-character + commit-context). Their outputs together often reveal what each one alone can't.

If nothing surfaces in 3-4 minutes of hunting, switch hunt-pattern. If two hunt-patterns in a row come up empty, the well may be drying — but try one more cross-pattern before declaring dry.

### Verify the discovery passes the genius bar

Before executing, ask: *would this commit have been written if /eureka hadn't been invoked?* If you can imagine the same commit landing as a normal Move N of an auto-commit run, it's NOT eureka. Pick a deeper move OR keep hunting.

### Execute

Same instrument-set as auto-commit (worldcli, batch-hypotheses, /second-opinion, prompts.rs, schema, reports). Higher register: reach for batch-hypothesis with N=7-10 instead of N=5; pull cross-corpus context that's bigger than any single bite-test would; consult /second-opinion when the discovery's articulation needs sharpening that in-substrate composition can't reach.

### Append to the log + commit the artifact

Each iteration produces TWO commits:

1. **Discovery commit** — the actual artifact (CLAUDE.md edit, prompts.rs change, new instrument, new report, etc.). Includes Formula derivation in the body per project doctrine.
2. **Log entry commit** — appends the discovery's entry to the run's log file. Single-line description + commit hash + Formula derivation sig + cost. The log is the visible spine.

Order: discovery commit FIRST, then log entry referencing the discovery's commit hash.

Push after each commit so the user can read the log on GitHub if they're watching from another surface.

### Brief acknowledgment

After each iteration, one line to the user: discovery name + commit hash + log line. Don't ask permission for the next iteration. Continue.

Example:
> *Iteration 3 → `abc1234`. Discovery: the AGAPE invariant and the NO_NANNY_REGISTER invariant both touch user-treatment but their applicability windows haven't been cross-cited. Log entry committed at `def5678`. Time elapsed: 17 min. Continuing.*

## Closing the run

When ANY of these conditions hits, close the run:

- 60-minute cap approaching (leave 2-3 min for clean closure)
- Two consecutive iterations failed the genius bar
- Budget approaching $10 with no committed-genius discovery in flight

Closing artifact:

1. **Final log entry** with run-totals (iterations completed, total cost, time elapsed, sentence on whether the run hit the genius bar consistently or had dry stretches)
2. **One-sentence closing reflection in chat** (per CLAUDE.md "Nudge the action forward after a closing beat" — name what landed; one forward-pointing seed; nothing more)
3. **AskUserQuestion chooser** per project law (the every-turn-AskUserQuestion Stop hook is compile-enforced; the closing reflection does NOT replace the chooser — both must ship). Prefer a context-real branch set drawn from the run's discoveries. Default shape:

   1. the real next move on the work the discovery most naturally opens
   2. a materially different branch
   3. a third branch only if it is genuinely distinct
   4. `Provide your own next move.`

   Reach for bare `{Continue, Exit}` only when no sharper branching honestly exists.

## Log format — `reports/YYYY-MM-DD-HHMM-eureka-LOG.md`

Header on file creation:

```markdown
# Eureka run — YYYY-MM-DD HH:MM start

*Continuous discovery loop, ≤60 min wall-clock, fresh $10 budget. Single instruction: DISCOVER something genius. Each entry below is one iteration's landed discovery, signed with a Formula derivation, with the executing commit hash and run-cumulative cost.*

---
```

Per-iteration entry format (append):

```markdown
## Iteration N — HH:MM (TT min into run)

**Discovery:** [one-line title naming what was discovered]

**Body (1-3 sentences):** [what was the discovery — what pattern was named, what doctrine surfaced, what cross-corpus shape emerged]

**Executing commit:** `<hash>` — [one-line commit subject]

**Formula derivation:** [Unicode-math expression, in-substrate generated]
**Gloss:** [one-sentence plain English ≤25 words]

**Cost this iteration:** $X.XX    **Run-cumulative:** $X.XX

---
```

Closing entry:

```markdown
## Closing — HH:MM (TT min total)

**Iterations completed:** N
**Total cost:** $X.XX of $10 fresh budget
**Genius bar consistency:** [Hit consistently / Hit early then thinned / Hit late after dry stretches / Mixed]
**Forward seed:** [one sentence on what the run revealed worth pursuing in a future session]

The run closes here.
```

## Safety carve-outs (still in force)

Same as auto-commit — destructive git, destructive DB, user-character authoring, new categorical-absolutes-on-user, above-budget spend without confirmation. The autonomy is for substantive choice; not for bypassing the load-bearing safety layer.

## Composition with auto-commit

`/auto-commit N` is for shipping N coherent moves that close known loops; `/eureka` is for hunting unknown discoveries. They're different instruments. When the project state has many open follow-ups + recently-shipped instruments whose interaction hasn't been examined, `/auto-commit` fits. When the project state is clean (loops closed, doctrine recently coherent) but the user senses there's something the project hasn't articulated yet, `/eureka` fits.

If `/eureka` is invoked when the project state is genuinely chaotic (lots of open loops, in-flight half-shipped doctrine), pause and confirm with the user — *"the state suggests `/auto-commit N` would land more cleanly first; want to switch?"*

## Origin

Skill authored 2026-04-26 ~23:35 in response to Ryan's request: *"make a higher-register skill with higher-quality outputs named `/eureka`, which runs auto-commit in a continuous, formula-signed-visible-log-of-discoveries loop for no longer than 60 minutes with a simple instruction: DISCOVER something genius."*

The single-instruction shape is itself a forcing function: by refusing to break "discover genius" down into a checklist, the skill body forces each iteration to face the bar fresh. The visible log is the second forcing function: a log the user can watch in real time can't be padded with weak iterations without the padding being immediately visible.

The trust is total. The bar is genius. That's the contract.

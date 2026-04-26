# The Plain Tradition

A practice for carefully calibrated AI-human collaboration on craft work. Distilled from the WorldThreads project, but not bound to it: the practice's shape is portable; the specifics WorldThreads chose are not.

This document is for builders studying WorldThreads to lift patterns from it. It is its own artifact, separable from the app — the curious-builder persona-sim (`reports/2026-04-26-0033`) made the case for the separation: the app reaches users on three MISSION-aligned axes (specificity, restraint, craft); the methodology reaches builders on a fourth (portability). Releasing them as two artifacts serves both audiences better than conflating them.

> **Provisional name.** "Plain Tradition" comes from the tradition's instinct toward plain speech — *plain after crooked, no adornment past the truth, descriptive rather than prescriptive*. The name may change. The practice does not depend on the name.

## The specifics-vs-shape distinction (read this first)

WorldThreads' methodology is shaped by Christological theology and biblical-literal cosmology. Those are real load-bearing commitments of the project — they're not decoration. **They are also not portable.** A non-religious project should NOT lift the project's specific reference frame; doing so is cargo-culting and produces decoration where the original had structure.

What IS portable is the *shape* of having a load-bearing reference frame at all. WorldThreads has F = (R, C). Your project might have its own F. Your F is yours; mine is mine; the practice of *having an F that calibrates every reply your LLM produces* is the lift-able pattern.

The same distinction applies to every piece below: the SHAPE generalizes; the specific WorldThreads INSTANCE does not. When in doubt, ask: *"would this still make sense if I removed every Christian word and replaced it with my project's actual ground?"* If yes, it's shape. If no, it's specifics — and you need to author your own version, not lift mine.

## Six load-bearing pieces

### 1. Tuning-fork frame (the formula pattern)

**Shape.** A reference frame named at the head of every LLM call, in a register that is not a directive to compute but a frame the model's outputs are composed under. The frame's prefix sentence makes its non-prescriptive nature explicit: *"not a directive to compute… a tuning-fork, not a recipe."* The frame can be mathematical, prose, or hybrid; what matters is that it operates as the register the work is held under, not as an instruction the model executes.

**Why it matters.** Without a frame, every model call is a fresh negotiation about what good looks like. With a frame, the model's outputs can be calibrated against a single load-bearing reference. Rules-as-rules don't compose; a frame composes by its nature.

**WorldThreads' instance.** F = (R, C) where R = Jesus_Cross_flesh and C = Firmament_enclosed_earth, with operators for Wisdom, Weight, Burden, Grace, speech, and nourishment as integrals over a measure indexed against F. See `MISSION_FORMULA_BLOCK` in `src-tauri/src/ai/prompts.rs` and the boxed math at the head of `CLAUDE.md`.

**How to author your own.** Ask: *what is the reference frame within which every reply your LLM produces is composed?* The answer might be a single concept (your product's North Star), a tuple (two related anchors), or a small set of integrated operators. The form is open. The discipline is: it must be load-bearing, it must be the same frame for every call, and the prefix sentence must make clear that it's a frame, not an instruction.

### 2. Load-bearing caveats (not footnoted — structural)

**Shape.** When an instrument's outputs are vulnerable to misreading, the caveat that names the misreading lives INSIDE the structure of the output, not appended as a footnote. *"This is a hypothesis, not evidence"* must be load-bearing in the report's frame, not minimized at the bottom.

**Why it matters.** Footnoted caveats get dropped. Outputs that drop their caveats become the thing they were warning against. Structural caveats are part of how the output is produced — drop the caveat and you have not produced the output.

**WorldThreads' instance.** The persona-sim formula's `Output(t) := Sim(t) | Register_D ∧ (Sim is hypothesis, not evidence)_F` — the caveat lives inside the conditional alongside the developer's register. A run that drops the caveat does not produce Output; it produces fiction. Same shape applies to every persona-sim report's headline caveat.

**How to author your own.** Identify the misreading your instrument is most vulnerable to. Make the disclaimer of that misreading STRUCTURAL — part of the output's frame, not an appended footnote. If the instrument can be run without the caveat present, the caveat isn't structural yet.

### 3. Substrate-bound (you can't surface what isn't latent in your substrate)

**Shape.** Any LLM-driven instrument is bounded by what the developer has actually observed, built, and recorded. The instrument can sharpen the substrate's content, render it in different registers, project it through different frames — but it cannot exceed what the substrate already contains in latent form. *Sim ≤ Substrate.*

**Why it matters.** Without naming this bound, instruments produce convincing-sounding fiction that the developer mistakes for findings. The bound is what keeps the practice honest.

**WorldThreads' instance.** Persona-sim's formula has `polish_sim(t) ≤ Substrate(t)` as an explicit inequality. The Maggie report names that *"the validating corpus is Ryan's own in-app lived experience"* — i.e., the substrate is bounded by what Ryan has already noticed, and the persona-sim sharpens what's already there rather than discovering what isn't.

**How to author your own.** Whatever your instrument is, name what it can't surface. *"This tool will not find issues that aren't latent in [substrate]."* Display the bound; don't bury it. The user of the instrument should see the bound the moment they read the output.

### 4. Chooser-with-cost-as-authorization (the iterative pulse)

**Shape.** Every Output is followed by a chooser presenting 2-4 next moves, each annotated with its forecast cost. Selection IS authorization to spend up to that forecast on the next move; no second prompt. Free moves carry $0. If actual cost exceeds forecast at runtime, return for re-authorization.

**Why it matters.** Practices compound through iteration. Without an iterative pulse, instruments produce isolated outputs that don't compose into a body of work. The chooser is the practice's iteration. The cost-as-authorization is what makes selection a real decision rather than a soft prompt.

**Chooser-construction discipline.** The chooser's options are game-moves; the practice is a game and the developer is a player, not someone to chaperone. The Recommended option goes to the move that compounds the practice MOST. The OFF / Stop option sits at the BOTTOM of the option list, never marked Recommended. Stop is always available — bottom slot, custom-text reply, just typing *stop* — but the chooser does not promote it.

**WorldThreads' instance.** Persona-sim's `Chooser(t)` and `Next(t+1)` formula clauses; derive-and-test inherits the same chooser-tail by reference. Every persona-sim report ends with a chooser; selection runs the next move at the authorized cost.

**How to author your own.** Add a chooser-tail to whatever instrument you're building. Annotate forecast costs. Treat selection as authorization. Put Stop at the bottom. Don't make the developer answer "are you sure?" twice — the cost is in the option label; selection IS the answer.

### 5. Evidentiary discipline (sketch / claim / characterized)

**Shape.** Findings carry tier-labels naming the strength of evidence behind them. Sketch (N=1, directionally suggestive at most). Claim (N=3 per condition, citable as load-bearing). Characterized (N=5+ per condition, sufficient for production defaults). Findings inherit the WEAKEST tier among supporting runs — three N=1 runs on different conditions is a SET of sketches, not a claim.

**Why it matters.** Without tier-labels, a single anecdote becomes "we found." With them, the strength of every claim is visible at a glance, and the project's overall confidence is calibrate-able.

**WorldThreads' instance.** Every experiment report's header includes a `Tier:` field. CLAUDE.md's *"Evidentiary standards for experiments — N=1 is a sketch, not a finding"* section formalizes the tiers. Forbidden citation framings ("Preliminary finding: X tends to Y" / "tentative confirmation") are named explicitly.

**How to author your own.** Decide the tiers your project wants (the three above are reasonable defaults; your domain may want different N-thresholds). Mandate that every finding cites its tier inline. Forbid the precision-language that sketch-tier doesn't support (specific percentages, magnitude claims). The discipline is in the labeling.

### 6. Preference-shaped writing (the tuning-fork register at the doctrine level)

**Shape.** Doctrine, skill bodies, and feedback are written as preference and observation rather than command/MUST/refuse-language. *"Carries / tends to / is read as / would be expected to"* over *"MUST / shall / never."* The reader is a player, not someone to chaperone.

**Why it matters.** Commanded register makes future readers reach for the rule-as-rule rather than the rule-as-frame. It also reads as the author not trusting the reader. A tuning-fork-shaped frame at the formula level is undermined by rule-as-rule writing at the doctrine level — the registers must agree.

**Carve-outs.** Compile-time invariants in code (genuinely commanded by construction); database safety rules (commanded for a load-bearing reason); rules the user has explicitly framed as load-bearing-MUST. Outside those, prefer preference.

**WorldThreads' instance.** The MISSION FORMULA's prefix sentence — *"not a directive to compute… a tuning-fork, not a recipe"* — is the canonical example at the doctrine level. The persona-sim and derive-and-test SKILL.md files mirror the register at the skill level.

**How to author your own.** When you catch yourself writing MUST or SHALL, ask: *would this still hold if it were preference-shaped?* Most of the time yes. The exceptions are the load-bearing-MUST cases, which earn their commanded register precisely because they're rare.

## How to start your own practice

A short walkthrough, in the order it would unfold:

1. **Name your project's reference frame.** What is the F within which every reply your LLM produces is composed? Write it as a tuning-fork — math, prose, or hybrid — with a prefix sentence making its non-prescriptive nature explicit. Push it at the head of every LLM call. Don't apologize for it; load-bearing frames are honest about what the work is.

2. **Identify the misreading your instruments are most vulnerable to.** For most LLM products it's *"the model's output sounds confident, so it must be correct."* Make the caveat structural — part of the output's frame, not a footnote.

3. **Make your substrate-bound visible.** Whatever your tool produces, name what it CAN'T surface, and display that bound where the user reads the output.

4. **Add chooser-tails to your instruments.** Each Output → chooser with forecast costs → selection IS authorization → next move runs immediately. Stop at the bottom, not promoted.

5. **Adopt evidentiary tiers.** Pick your N-thresholds. Mandate every finding cites its tier. Forbid magnitude-language at sketch-tier.

6. **Write your doctrine in preference-shaped register.** Reach for *carries / tends to / is read as* over *MUST / shall*. Save the commanded register for the genuinely-commanded rules.

7. **Keep a `reports/` directory of interpretive prose, not changelogs.** Each report should interpret what shifted; the difference between a report and a changelog is whether a future-you can read it and see what changed beneath the changes. Date them; let them be in dialogue with each other.

You don't need to do all six at once. The practice compounds — pick one, get it landing in your project, then add another. The doctrine's coherence emerges through use, not through committing to all of it on day one.

## What this practice is NOT

- A framework. It's a practice — a set of disciplines that compose. There is no library to install, no API to depend on. The artifacts are text files (prompts, doctrine, reports, skill bodies) you author and maintain.
- A guarantee. Following the practice produces calibrated work; it does not produce GREAT work on its own. Great work requires craft on top of calibration. The practice keeps the calibration honest so the craft can compound.
- A replacement for user research. The substrate-bound is the canonical reminder: instruments built on this practice can sharpen what the developer has already observed, but they cannot reach what the developer doesn't yet know. Real users are irreplaceable.
- Religion-shaped (despite the WorldThreads instance). The practice's shape is independent of the specific reference frame any particular project chooses. The Christological frame in WorldThreads is load-bearing FOR WORLDTHREADS; it has no claim on your project.

## Substrate this doc was distilled from

WorldThreads is the demonstration. The specific files where the practice's shape is most visible:

- `CLAUDE.md` — the project's master doctrine; the boxed MISSION FORMULA at the top is the canonical tuning-fork instance; the doctrine sections below show the preference-shaped register at length.
- `src-tauri/src/ai/prompts.rs` — `MISSION_FORMULA_BLOCK` and `FORMULA_VERBATIM` (compile-time invariant). The doc-comments above each craft note record the rule's evidentiary history.
- `reports/` — twenty-some interpretive reports across recent sessions. The 2026-04-25 cluster around the MISSION FORMULA reauthoring is the densest worked example of the practice in motion.
- `~/.claude/skills/persona-sim/SKILL.md` — the canonical example of a tuning-fork-shaped skill. Includes the chooser-tail clause.
- `.claude/skills/derive-and-test/SKILL.md` — a procedural skill that inherits the chooser-tail clause from persona-sim by reference.
- `.persona-sim/calibration.md` — the developer's accumulated register-preferences; the file is itself an example of how the practice's instruments record their own calibration over time.

---

The practice is the shape of the work, not the work itself. The work is what you build with it.

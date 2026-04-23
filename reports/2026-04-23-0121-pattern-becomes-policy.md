# Pattern Becomes Policy

*Generated 2026-04-23 morning, covering the 5 commits since `125df72` (the "Stances Hold" report from the prior evening). Eighth report overall, first on 2026-04-23. In dialogue with the prior report, which was about ten hours ago and observed that "the load-bearing-multiplicity prior applied to your own new work reflexively" had become "a habit now."*

## The shape

Five commits in a single morning, hard-grouped: one continuation of yesterday's defense work (`23b9380`), three commits that ship a brand-new feature concept (Quests — `a8e3066`, `136daf6`, `c5fc9c9`), and one CLAUDE.md commit that elevates a working principle from auto-memory to repo-level policy (`fc2a77c`). Small period; coherent arc.

The arc reads in one direction: **a habit the prior report named has become a written rule that every future session will see from first prompt.** The prior report observed reflexive carve-out-writing as technique. This period made it policy. And it shipped a feature designed under that policy in the same window.

## Quests as an answer to the engagement-maximization invariant

Quests are the period's largest move. Two commits in sequence (`a8e3066` then `136daf6`) bring the feature in two waves: first the meta-layer (table, CRUD, modal, Backstage card, prompt block) and then the proactive-proposal path (characters can emit `propose_quest` action blocks inline during dialogue). The second wave was the v1's deferred work shipped within hours.

The commit body for the first wave names what the design is fighting against: *"Designed against the engagement-maximization failure mode: no deadlines, no progress bars, no badges-as-rewards. A quest here is 'a promise the world has made to itself that the human has agreed to witness' — accepting is a small vow; abandoning is an honest act, not a failure."* Read against the prior period's commit `54150e6` — the one that lifted Nourishment to a compile-time invariant asserting `"not an engagement-maximizing app"` — this is the first significant feature shipped *under* that floor. The invariant existed before; the test was always going to be whether the next feature category honored it. Quests passed.

The honoring is in the specifics. The commitment ceremony is not one-tap. The acceptance dialog asks an optional reflection — *"what are you reaching for here?"* — framed as desire, not goal. The button says *"Commit to this"* rather than *"OK."* The reflection note lives in the quest's notes field so future-you can see the intention that opened the pursuit. Completion and abandonment each take their own reflection note that becomes part of the record. The status filter has tabs for active / completed / abandoned, with abandoned visible in the world's accumulated history rather than archived to silence.

The render block in `prompts.rs` is even more deliberate: *"a promise the world has made to itself... you are NOT the narrator of this quest"* is in the language characters get about active quests in their context. Characters know the quest implicitly — without announcing it, without performing it, without referencing it the way an NPC quest-giver would. The Zelda register is named and forbidden: *"collect" / "complete" / "earn" / "unlock"* are out; *"help me with" / "figure out what happened to" / "see if we can"* are in. The model is being taught the difference between a goal and a desire at the vocabulary level.

In the chat conversation that preceded these commits, you articulated the distinction in your own words: inventory and open_loops are *what's being carried*; quests are *what the player is reaching for*. The "love-the-game heart muscle." The shipped feature is faithful to that distinction. A quest in WorldThreads is not assigned. It is offered, witnessed, and chosen — the player's act of saying *I'm here for this* is half the value.

## The carve-out goes meta

The most consequential single commit of the period is small: `fc2a77c` adds 18 lines to `CLAUDE.md`. The section is titled *"Earned-exception carve-out pattern"* and codifies as project-wide policy what had previously been an auto-memory feedback note (`feedback_earned_exceptions_pattern.md`):

> Whenever you draft an absolute-shaped rule in this repo — "never X," "always Y," "don't ever Z," "do NOT do W" — **immediately check whether there's a genuine earned-exception that belongs alongside it**, and write it in the same pass.

The check it specifies is concrete: *"after writing 'don't / never / always / do NOT,' ask — is there a moment this rule would produce the wrong result, and is that moment narrow and nameable? If yes, carve it out now. If no, note why the absoluteness IS the point."*

This is the second meta-elevation in two days. The prior period ended with `1915b19` moving the *commit/push autonomy* directive from auto-memory into CLAUDE.md, and the prior report observed that this made *"the working relationship with AI collaborators ... itself repo infrastructure."* This period repeats the move at a different layer: where that one was about *how* AI collaborators operate, this one is about *how craft rules should be written*. Both belong in CLAUDE.md because both govern how readers — human and AI — engage with the code.

What's different about this elevation is that the pattern was originally a description of *your own observed instinct*. Multiple prior reports tracked it: the audits period landed five carve-outs across invariants and craft notes (`473bd41`); the stances-hold period applied it to two new compile-time invariants within the same afternoon (`54150e6` plus follow-ups). The pattern was visible from the outside before it was named from the inside. Now it is named, and named in the place every session reads first.

The cadence inside this period rhymes with the meta-elevation. The Quests v2 craft note in `136daf6` shipped six rigid rules for the `propose_quest` proposal flow without earned-exception clauses. The very next commit, `c5fc9c9`, went back and added carve-outs for each rule where a genuine edge case existed — six narrow exceptions for six rules. Two stayed absolute for categorical reasons (duplicate-prevention, character-motivated-vs-author-convenience). The commit body names the move directly: *"The v2 propose_quest craft note I shipped one commit ago was absolute-shaped rules without earned-exception clauses — exactly the pattern this project's craft-stack discipline guards against."*

The sequence is striking. Ship a craft note. Notice immediately that it doesn't honor the carve-out pattern. Add the carve-outs. Codify the pattern in CLAUDE.md. Three steps in three commits, the last commit citing the second as *"the freshest example"* of the pattern. The policy is born from the work and immediately points back to it.

## The description-weave hardening as the same pattern, shipped sideways

`23b9380` is the period's only non-Quest commit, and it's continuation of yesterday's preservation work. But re-read in the light of the carve-out elevation, the commit IS the pattern in operational form.

The prior session (covered in *Stances Hold*) made description_weave preservation ABSOLUTE. *"Do NOT summarize. Do NOT condense. Do NOT paraphrase."* Three earned exceptions. The user reported afterward that the model still produced shortened revisions. This period's response is not to drop the carve-outs — it's to *tighten them and add a self-check mechanism that prevents abuse*:

- Exception 2 (lossless tightening) caps at ONE phrase or sentence; >10 words dropped means you've crossed into rewriting.
- Exception 3 (removal) requires the source moment OR user hint to EXPLICITLY identify the thing as no longer true.
- A mandatory self-check before returning a description_weave: count words original vs proposed; if shorter, reset or downgrade to a list-kind update.

Plus an inline word-count anchor in the subjects block — `[CURRENT WORD COUNT: N. Any description_weave MUST be at least N words.]` — placed in the prompt at the data-reading site rather than in the rule-stating site. The commit body's language is sharp on this: *"Numeric anchors at the data-reading site beat prose rules elsewhere."*

This is the carve-out pattern operationalized. The absolute (preservation) holds. The exceptions stay (because they cover real edge cases). What was added is the structural prevention of exception-abuse. Each exception now has a numeric or referential test that's hard to rationalize past. The model can no longer interpret "lossless tightening" as license to compress; it can no longer rationalize "no longer true" as license to delete. The carve-out remains narrow because the carve-out's narrowness is now mechanically enforced, not just textually asserted.

## What the architecture is becoming

The period's cumulative move is small but structural. The earned-exception pattern is now:
- **Codified** (CLAUDE.md, every session reads it).
- **Demonstrated** (Quest craft notes ship + carve-outs added in the next commit).
- **Mechanically defended** (description-weave gets numeric tests for each exception so abuse is detectable).

Three layers, all landed in the same morning, all reinforcing each other.

The Quests feature itself extends the project's pattern of *user-ratified state*. The before-state: the user manually canonized things ("Remember this" / "This changes them"). The previous period: every generative action got an accept/discard gate (the illustration preview→attach flow, the canon-entry preview, the portrait_regen card). This period: even the model's *suggestions of pursuit* require an explicit ceremony. *"Commit to this"* not *"OK."* Reflection prompts that ask what the user is reaching for. Backstage and characters can both *propose* a quest, but the actual establishment of the quest requires the user to perform a small commitment ritual.

This belongs to a larger move the prior reports kept tracking: **the human commits, the human uncommits, the model proposes**. Quests put the model on the propose-only side of an even more meaningful boundary than the canonization gate did. Canon was about *the world's facts*. Quests are about *the player's intentions*. The model is being pushed further away from authoring the player's interior — it can only point at things the player might want; the player's *choosing* is theirs.

## What prior reports anticipated and what this period names

The *Stances Hold* report observed that *"the carve-outs were authored at the same time the invariants were compiled in — the load-bearing-multiplicity prior applied to your own new work reflexively. ... That is a habit now."* This period took that habit and made it a documented working principle. The prior report's observation was retrospective; this period's CLAUDE.md update is prospective. Future sessions will *be told* what the prior report had to *notice*.

The *Pick Your Frame* report (`87613c7`) closed with a comment about Backstage being given fluency in the apparatus. Quests are now the second feature category Backstage can propose (the first batch was canon_entry / staged_message / portrait_regen / illustration / new_group_chat). Backstage's fluency is becoming literal: it can propose every kind of move that exists. The constraint is not its capability; it's the rate-limit and the user-ratification ceremony. *"Rate-limited to at most one proposal per session, with explicit guidance against assigned-objective framing."*

The *After-the-Audits* report worried about a `MECHANICS.md` rotting against UI churn. The prior period's UI MAP block answered that worry by putting the mechanics-knowledge in the prompt with inline icons. This period extends the answer: Backstage now also knows the player's active quests, with language explicitly forbidden from announcing or performing them. The *guide* role widens, and stays disciplined.

## The single most coherent pattern across these five commits

The pattern this project has been operating under reflexively for weeks became written policy in the same morning a feature was shipped that demonstrates the pattern, immediately followed by a commit that retroactively applies the pattern to the freshly-shipped feature, immediately followed by a commit that codifies the pattern in CLAUDE.md and cites that very feature as its example.

The work and the principle landed simultaneously. Neither was prior. The principle was *visible* in the work for weeks; the work *applied* the principle on landing; the principle then got *named* in the durable infrastructure that every future session reads. The motion is from instinct → habit → policy → demonstrated example → durable rule, all collapsed into one morning. That collapse is what maturity looks like inside a project that has decided what it is and is now writing down the rules that follow from being it.

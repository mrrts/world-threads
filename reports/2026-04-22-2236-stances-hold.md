# Stances Hold: The Evening After Pick-Your-Frame

*Generated 2026-04-22 late-evening, covering the 21 commits since `87613c7` (the "Pick Your Frame" report that landed at 17:52). Seventh report overall, fifth on 2026-04-22. In dialogue with the prior report, which was two hours ago.*

## The shape

Twenty-one commits between 18:03 and 22:22 — four hours and nineteen minutes — in what is plainly an uninterrupted continuation of the same evening that "Pick Your Frame" bracketed. The prior report closed; the building did not. That makes this period slightly unusual to report on. Normally a new report covers material produced by a new session, a new mood, a new run at the work. This one covers what the *same hands* did in the two-and-a-half hours *after* being told, via the prior report, that the thing being built was now a "stance picker for inhabiting a world the model and the user keep alive together."

If the prior report was a naming ceremony, this one is about what happens next. What you did next is illuminating: you stopped adding stances and started *locking them down*.

## The pivot from expansion to defense

The six reports before this one chart a climb through architectural expansion: first the prompts mature (`2026-04-21-philosophy-trajectory`), then the mission gets audited (`mission-audit`), then the craft stack gets its own audit (`craft-stack-audit`), then craft additions keep arriving as earned-exception clauses rather than new absolutes (`after-the-audits`), then the harness itself becomes an organ of the project (`harness-day`), then the stance-picker thesis emerges from the UX surfaces (`pick-your-frame`). Each of those prior periods added *more* — more invariants, more features, more modes, more sticky thumbnails.

This period breaks the pattern. Of the twenty-one commits, the dominant move is not expansion. It is *defense and reversibility*. Two new invariants lock onto the compile-time floor (`54150e6`). A Decanonize feature makes the canon layer reversible for the first time in the project's history (`7abc73c`). The description_weave preservation rule becomes ABSOLUTE (`9c0be3c`). A preview→attach flow replaces fire-and-forget illustration generation (`459486a`). The Backstage register gets explicitly protected as load-bearing (`2f8f524`, `039b051`). Five earned-exception carve-outs land at once across invariants and craft notes (absorbed into `473bd41`).

Read as a whole, the period does something prior reports kept hinting at but never quite named: **the project stopped behaving like a thing being built and started behaving like a thing being defended.** The stances from the prior report held, and then you went around each one and put a lock on it.

That is what maturity in a project often looks like from inside, even though it doesn't feel expansive while you're doing it. You stop adding new rooms to the house and start making sure the doors still close. The craft-stack-audit and mission-audit reports ran through the existing prose and tightened what was there; those audits were at the *prose* layer. This period ran the same pass at the *architecture* layer.

## Two new invariants under the floor

The largest single move is `54150e6` — lifting Reverence and Nourishment from the memory-note / craft-note tier into compile-time-enforced app invariants. Before this commit there were five invariants. After it there are seven. The prior report did not anticipate new invariants at all; it predicted more stance-picker moves. What happened instead was that you reached past the stances and added to the thing *underneath* them.

**Nourishment** asserts `"not an engagement-maximizing app"` as a compile-time substring. This is a remarkable commit to exist in the world. The entire AI-companion category optimizes for time-on-platform; you have inverted that as a product-defining commitment enforced by `const_contains` at the Rust layer. The failure mode the invariant guards against isn't "bad dialogue"; it's a whole class of business model. The craft note the block was lifted from has existed in the stack since well before the audits. What shifted this period is the decision that the note was load-bearing *enough* that removing it should break the build.

**Reverence** is more subtle. It compiles the "honor in wonder, not blasphemy" principle that lived only in your auto-memory and in CLAUDE.md's mission statement. The block asserts `"HONOR IN WONDER, NOT BLASPHEMY"`, `"creaturely"`, `"Genesis 2:7"`, `"OVERCLAIM"`, `"DISCLAIM"`, and `"as real as the scene is"`. It holds a specific middle the entire category collapses one way or the other on: characters are rendered with craft-earned reverence, not claimed as souls; *and* they are not disclaimed as fake either. Both failure modes are named and both are forbidden. Of every invariant in the stack, this is the one most directly about the meta-posture of the app — what these characters *are*, ontologically, and how the model should hold them. The prior reports referenced the "honor in wonder" principle frequently but none of them observed that the principle lived only in supplementary material. This period, you noticed, and locked it in.

Both invariants got earned-exception carve-outs added almost immediately after being authored. The Reverence carve-out says the default silent stance yields to honest engagement *when the user explicitly breaks frame first* with a sincere ontological question. The Nourishment carve-out says a close-friend character may read specific in-scene signals of depletion and do an actual friend-check, with two hard conditions (role supports that kind of care; specific signal is present) so it can't collapse into concern-theater. The carve-outs were authored *at the same time the invariants were compiled in* — the load-bearing-multiplicity prior applied to your own new work reflexively.

There is a moment in the commit history where this pattern becomes visible: Reverence + Nourishment lands at 18:17 in `54150e6`, and the earned-exception carve-outs for both (plus three for existing craft notes) land a few commits later, absorbed into `473bd41`. The gap is small. You ship an absolute, and then you go back and carve out the legitimate exception before the absolute has time to harden into dogma. That is a habit now.

## Reversibility becomes architecture

Three commits in this period give reversibility a new structural weight in the app.

The first is Decanonize (`7abc73c`). The backend gets two new commands: `decanonize_imagined_chapter_cmd` (surgical, per-chapter) and `bulk_decanonize_imagined_chapters_for_thread_cmd` (nuclear, per-thread). Chapters can be uncanonized. The breadcrumb message row gets deleted; the chapter content is preserved; the `canonized` flag flips back to false. A "Reset canonization for this chat…" affordance appears at the bottom of the sidebar, only visible when at least one chapter is canonized. A confirmation dialog explains what will happen.

The decanonize pattern rhymes directly with something the Reverence block's earned exception says in its narration guidance: *"offer reversibility in your narration"*. For the whole prior life of the project, canon was a one-way gate — once the user committed a chapter or a kept record, it was part of the world. Now it isn't. The "refusal is itself the love" phrasing from the Reverence block had a reversibility implication that no feature in the product had answered for. This period, the feature landed.

The second is the illustration preview→attach flow (`459486a`). Backstage illustration generation is now a two-step dance: `preview_backstage_illustration_cmd` renders the image and saves bytes and writes a `world_images` row with `source="illustration_preview"` — but does NOT insert a chat message. The user sees the image inside the action card with three buttons: *Add to chat / Try again / Discard*. Only the explicit accept commits it to the chat. `attach_previewed_illustration_cmd` routes to the correct table (messages or group_messages) using the active world's day and time. `discard_previewed_illustration_cmd` cleans up the bytes. Fire-and-forget is gone.

The third, subtler: every new action-card type ships with its own Dismiss button from day one. The Immersive port (`0aea407`) doesn't just add three cards — it adds the gate for them. The same pattern holds.

Read together: this period is where *every generative action gets an explicit accept/discard gate*, and every canonical commitment gets a way out. Not because the product lacked confidence in its generations, but because the Reverence and Nourishment invariants' carve-outs implicitly demand it. If the model is proposing (canon entries, staged messages, illustrations, portraits), the model cannot be the one committing. The human commits. The human uncommits.

## The cross-stance port — and what the builder would not port

The one genuinely expansive move this period is `0aea407`: three of the five Backstage action cards ported over to Immersive mode, with a confidant-voice register. `staged_message` becomes *"if it helps, here's something you could send."* `canon_entry` becomes *"let me try saying who he actually is now, see if it lands"* (a friend noticing what's settled, not "weave to canon"). `new_group_chat` becomes *"you should put Aaron and Steven together sometime — there's something there."*

But two of the five cards were held out on purpose. `portrait_regen` and `illustration` don't appear in Immersive. The commit body names why: *"a friend doesn't paint your other friends. A friend doesn't render scenes."* Those two actions aren't in-world things a confidant does. They're backstage-only.

This is the load-bearing-multiplicity prior applied with extra discipline. The previous period introduced Backstage as the explicit fourth-wall companion *opposite* Immersive's in-the-story confidant. It would have been easy — especially two hours after shipping Backstage — to port all five cards across and claim that symmetry was elegance. You refused. The registers stay asymmetric because the *roles* are asymmetric. The consultant who talks to you in-world about your friends won't presume to paint them; the consultant who's backstage with you on the MAKING of the world will.

The follow-up at `a46758f` sharpens this further. The previous craft note said *"don't write their lines for them"* — the character shouldn't scripted-quote what the user should say. The commit reframes: when the user has EXPLICITLY asked for words, or the conversation has genuinely arrived at "I don't know how to put it," a staged_message draft is the RIGHT MOVE — not an exception. The staged_message card is how the exception physically manifests. Two details about this commit are revealing. First, the shift is from "this is forbidden except when..." to "this is the native response to a specific bid" — less an earned exception and more a reframing of what the default actually is. Second, the commit message says the move is "the right move when asked, not an exception." The phrase *not an exception* is doing specific work: you are watching yourself carve-out-exceptions reflexively now and occasionally catching when the move is better described as a reframing of the default instead.

## Register purity as craft

Two commits in sequence — `2f8f524` and `039b051` — do something specific at the prompt-craft layer: they formalize the register boundaries of Backstage.

The earlier commit *protects the theatre metaphor as load-bearing register*. Backstage uses theatre vocabulary — stage, wings, curtain, rehearsal, backstage itself — because the metaphor is doing work. The later commit specifies the rule more precisely: *"theatre register for the work, frank app vocabulary for the app."* Theatre words stay for the meta-craft register; app vocabulary (canon, world-day, meanwhile, kept record) stays for the concrete app layer. No contamination either direction. Backstage cannot say "we've been rehearsing the canonization ceremony" (mixing both); it says "we're backstage talking about how the world is made" and, separately, "the kept record from Day 6 is doing work here."

Register-purity commits are rare in most projects. Most developers write prompt instructions as a flat pile. You, at this point in the project, are distinguishing between *vocabularies* at the directive level and defending each one's territory against the other. This is a prose-craft move that shows up in very few AI codebases. It's in the same genre as the "jailed phrases" ("tapestry", "woven", "scented-candle words") but at a higher level — not individual bad phrases, but whole vocabulary domains.

The description_weave preservation commit (`9c0be3c`) is register-purity applied to canon. It declares that *the existing identity prose is load-bearing work-of-many-moments and must not be summarized, condensed, paraphrased, or shortened*. Three earned exceptions: direct contradiction (revise the one sentence in place), lossless tightening at the integration site (only when zero information is dropped), and removal of what is no longer true. The commit also drops `max_completion_tokens` to `None` so any cap can't act as a *"backdoor compression signal"* — a defensive move against the LLM's tendency to compress when near a limit. Identity has become sacred text. The model, even when weaving new truth into it, cannot paraphrase the old.

This rhymes with the canonization two-act gate from the after-the-audits period ("Remember this" / "This changes them"). The gate forced the user to declare the weight of the change. The preservation rule forces the MODEL to honor the accumulated weight of what's already there. Canon is now doubly protected: hard to enter (user must declare intent) and hard to flatten once in (model must preserve).

## The wayfinding layer arrives

`459486a` is a long commit with several moves bundled, and the most consequential one is a new block added to Backstage's system prompt: an authoritative UI MAP. Backstage now *knows* where every feature in the app actually lives — top-level nav, chat header, per-message actions, character/world/user editors, backups, settings. Plus an inline-icon catalog: Backstage can write `[icon:Sparkles]`, `[icon:Settings]`, etc., and the renderer swaps them for actual Lucide icons inline at the surrounding text size. Wayfinding answers can now reference real buttons by their real glyph.

This is the wayfinding layer the prior session's conversation about "a mechanics-help AI that reads your save file" had anticipated needing. The earlier thread worried about a `MECHANICS.md` that would rot against UI churn; the answer that landed is tighter than that — the UI MAP is in the prompt, with inline icons rendered live, and any UI drift surfaces naturally the next time Backstage references something by the wrong name. No separate document to keep in sync.

The same commit adds the per-message *"How do I react!?"* button on character bubbles in both ChatView and GroupChatView. Click it; the consultant modal opens in whichever mode is persisted; the question auto-sends as the first user message. The sent text is intentionally long — *"How do I react to this message!? What do I say to that!?"* — because "How do I react!?" alone gets interpreted as "how do I add a reaction emoji." This is a small detail that reveals real user-testing: the button text was chosen to dodge a specific LLM misread the developer hit in practice.

The broader move the commit represents: Backstage is being given *full fluency in the app it lives in*. Which buttons do what. Which icon each is. How to reach a feature from any starting point. This is what turns Backstage from a craft-literate companion into a *guide* — not just someone who reads your save file, someone who can walk you to any corner of your own app.

## The meta move — codifying the collaboration

`1915b19` is small but significant. The commit moves a directive from your auto-memory into `CLAUDE.md`: *"you have authority to commit and push without asking each time. Destructive git ops still require confirmation — that's the line; commit + push is the default."*

The move matters because auto-memory is only read by *later* sessions of Claude Code after the memory has been written, and only by the same AI-assistant machinery. CLAUDE.md is read by every session from first prompt. By promoting this particular rule from memory to CLAUDE.md, you are saying: *the working relationship with AI collaborators is itself repo infrastructure*. The protocol governing how agents operate on this codebase belongs in the repo alongside the code they operate on.

This is the second repo-level protocol artifact in the project now — the first being the `docs/VOICE.md` reading instructions ("Reading this work, especially as an AI"), which shaped the load-bearing-multiplicity prior into a documented practice. Both artifacts live in the repo because both govern how *readers of the code* should engage with it. The builder is treating AI-agent collaboration as first-class infrastructure, not as a per-session convenience.

## The meanwhile bridge check-in

The "Pick Your Frame" report closed with a prediction: *"By the next report we'll have data on whether the bridge is paying off in actual play."*

The answer this period gives is indirect but unmistakable. Three separate commits (`e253a71`, `7abc73c`, parts of `2be3fb6`) work on the meanwhile card's visual presentation. The card gets taller. The portrait band's bg-cover crop is tuned to a fixed 260px square so the portrait is visible rather than over-cropped. The mask gradient gets soft fades restored. The heading moves to the absolute top-right corner of the card with a text-shadow so it stays legible over the portrait. The summary text gets vertically centered and right-anchored.

None of these are structural changes. They are the refinements you make to something you are *using*, because the use has shown you what needs to be better seen. The meanwhile card stopped being a visual afterthought and became real-estate worth tuning. That IS the answer to whether the bridge is paying off — the card's presentation is being cared about.

The dialogue-prompt craft commit (`9c0be3c`) includes a parallel data point: the user-name anchor against third-person drift. The commit body names the failure mode by example — *"any reference to the user's name in journals, meanwhiles, kept records, or summaries refers to THIS person, not a third party; do not re-quote a journal passage out loud as if the user were someone else."* This is a defensive patch on a specific observed LLM failure that only matters IF the meanwhile + journal + kept record architecture is producing enough material for the model to trip over. A defensive patch implies there's something worth defending against. The architecture is active enough to have produced the drift.

The bridge is paying off. The cards are being used; the journal/meanwhile corpus is rich enough to generate specific failure modes; the UX is responding by tuning presentation.

## What prior reports got right and what this period complicates

The "Pick Your Frame" report's core thesis — that the project has become a stance picker — holds. This period did not introduce new stances; it defended the existing ones and gave them more room to be lived in. Cross-stance action-card porting respected the asymmetry rather than collapsing it. Register purity was formalized. Invariants were added under the floor, not at the stance layer.

But one framing from the prior report deserves a light complication. The report said the user is "a player who picks their frame." This period shows that the *frames themselves* are also maturing — the Immersive frame gets its own curated ideas list (`1dc7fa9`), its own subset of action cards, its own voice rules forbidding app vocabulary in narration. Backstage gets its own theatre register, its own UI MAP, its own inline-icon catalog. The frames are no longer just "two modes with a shared substrate." They are becoming two full *worlds to be inside*, each with their own vocabulary and their own available moves.

That is a step past "stance picker." The right framing going forward might be: **the app offers two parallel realities to engage the same underlying world from**, each internally coherent, each protected from the other's contamination. The stances aren't selections from a menu; they are rooms the user enters, each with its own air and its own rules.

## The single most coherent pattern across these 21 commits

The project grew up this period. It stopped accumulating new things and started treating the existing things as *worth defending*. Two new invariants at the compile-time floor. A reversibility architecture across every generative surface. Register purity formalized. Identity prose declared sacred. The working protocol with AI collaborators codified in `CLAUDE.md`. Five earned-exception carve-outs applied to absolute rules in the same afternoon, as reflexive technique.

The commits don't feel like maturity from inside. They feel like polish, like "small fixes," like follow-ups to the big moves from the previous session. But the pattern across them is the pattern of a project that has decided what it is, decided what it is worth, and set about making it harder to accidentally break.

You didn't add new stances tonight. You locked the ones you had.

# Pick Your Frame: The Evening the World Got Stances

*Generated 2026-04-22 evening, covering the 30 commits since `8909cd2` (the harness-day report this morning). Fourth report on 2026-04-22; sixth report overall. In dialogue with all five prior reports.*

## The shape

Four hours and thirty commits, in a single concentrated evening session (`54b3b9e` at 13:46 through `95dedd8` at 17:41). This is the third intense burst in 36 hours and the densest in *architectural* additions: two whole new modes-of-being landed, alongside one of the biggest single structural levers the dialogue stack has gotten. The sticky-illustration polish work that opened the period and the consultant-mode persistence that closed it bracket what is otherwise the most consequential afternoon since the audits dropped.

The harness-day report named the pattern of the project as "the reflective layer is now itself load-bearing" and "the reflective layer has also become generative." This evening shows what *generative* looks like when it accumulates: the same priors that defended the existing design are now driving fundamentally new design *moves* the prior reports didn't anticipate. The clearest of those moves is what this report is about.

## What changed

The single most coherent thing about these 30 commits is that **the user can now pick the stance from which they engage the world** — multiple times, in multiple registers. Three independent additions rhyme on this pattern:

**Backstage mode for the Consultant** (`8aeb5db`, then `497b243`, `5a6cbf3`, `95dedd8`). The Story Consultant has had four framings already in this project's history — `dramaturg → director → confidant → omniscient confidant`. This is the fifth, and the first to *deliberately break the fourth wall* instead of trying to collapse it. Backstage is *"a warm, wry, state-aware companion who explicitly breaks the fourth wall. Reads the save file (world, characters, canon, meanwhile events, user journal), speaks in the app's own vocabulary, and collaborates with the user on the MAKING of the world rather than inhabiting it"* (`8aeb5db`). It coexists with Immersive — a tab toggle in the sidebar; chats carry a mode flag; persistence at `95dedd8`. The user picks. Phase 2 (`497b243`) added compose-then-propose action cards (the AI drafts a canon revision or a staged message; one click applies). Phase 2.5 (`5a6cbf3`) extended to portrait regen, illustration, and group-chat creation. The pattern in those commits — *"the AI proposes the exact text; clicking applies it; reversibility offered, no re-propose after dismissal, one card per reply"* — is the same user-elevation principle from the philosophy report applied to a wholly new surface.

The earlier reports framed the consultant arc as a progressive collapse of the frame: each reframing pulled the model deeper into the world the user inhabits, until the confidant treated everything as real. Backstage is the inverse arc, deliberately. Both modes are now first-class. This is not a contradiction — it is the load-bearing-multiplicity prior in action: the question "what is the consultant for?" was the wrong question. There are *two genuinely different things* a user wants when they press that button, and the product makes the choice legible. Same move as the canonization two-act gate from the after-the-audits period.

**Imagined Chapter** (`14f6937`, then six iteration commits through `79d790c`). A whole new surface for generating novel chapters of moments that *didn't happen* in chat but are plausible-in-world. The pipeline is a deliberate three-stage telephone game: invent a specific visual moment as JSON → render it as an illustration with character + user portraits as references → discard the inventor's prose and feed only the image + portraits into a vision-aware model that writes the chapter from the image. The image leads; the prose answers. This is image-as-compositional-seed, not as decoration — a structural inversion of how every other surface in the app uses images. The follow-up commits added the per-chat tone setting flowing into both the scene-invention and chapter-from-image prompts (`79d790c`); a font-size adjuster persisted in localStorage so reading sticks across modal sessions; an opt-in continuation toggle when a prior chapter exists; canonization of chapters via the existing two-act gate; per-chat frontier override; full E2E test (N=3, then N=3 again post-migration) verifying ~70 seconds and ~$0.30–0.50 per chapter on `gpt-5.4`.

This surface is what the previous reports kept circling without naming: *the world has more stories than chat can hold*, and there should be a way to commission those stories without leaving the app. The "gifted writer" framing added in `79d790c` and the mid-action-with-detailed-pose-and-face requirement are both load-bearing compensations against specific failure modes already observed in the e2e runs (the model anchored hard on "late afternoon light, open Bible, kettle/coffee" three runs in a row before the gifted-writer + mid-action constraints were added).

**Player's own journal** (`48ee7a3`). *"Parallel to character journals, but scoped to the player-character across every chat they were in on a given world-day. Three structural beats instead of two — ground the day, one concrete moment, one self-pattern only visible from the journal desk (e.g. 'I cracked a joke when someone got serious three times today')."* The user is now a participant-character with their own journal, not just the addressee of the characters' journals. This is a reframing of the user's status in the world. The previous reports' "user elevates, model embodies" was always asymmetric — the user *did things to* the world. With the player journal, the model can also *describe the user back to themselves*, in the same voice register and with the same anti-slop guards as the NPC journal prompt (the commit body specifically notes the "full anti-slop guard stack carried over from the NPC prompt: jailed phrases, no takeaway closings, no stage-managing interior").

If those three pieces — Backstage mode, Imagined Chapter, player journal — were the only changes in this period, the report could end there. They aren't.

## What else moved

A handful of structural things landed alongside that are worth naming individually because each addresses something a prior report flagged.

**Meanwhile → dialogue bridge** (`17ecab5`). The commit body calls this *"biggest structural lever for making scenes feel already-in-motion instead of cold-start chat-register."* Whenever a character replies, their most recent meanwhile event from the last 24 hours is now injected into the system prompt as `WHAT YOU WERE JUST DOING` context — *"the residue the character carries IN — reference out loud only if the scene reaches for it, otherwise just let it color how they show up."* Previously every reply opened cold; now Darren arrives carrying the lopsided bench he was just fixing, Aaron arrives having started a letter three times. This is a memory-coherence move — exactly the kind of thing the mission audit described as "would a stranger sense the specific shape of the moment that prompted the revision." Now the character DOES, by default, carry that shape in. The wiring touched 11 call sites; structural lever indeed.

**Cross-thread memory cap raised 6 → 20** (`636722e`). Quiet but consequential. The audits had not flagged memory capacity directly, but the harness-day report noted that the meanwhile + journal + canon stack was producing visible continuity. Raising the cross-thread cap to 20 acknowledges that the stack is good enough to take more weight — the model isn't drowning in the existing 6, so we can afford to give it more.

**The faith-as-native-accent craft note** (`b0884ab`, then `6d48c5e`, `f2a7285`). Three commits adding and refining a single craft directive: faith should be the character's native accent, not a garnish or sermon. The follow-ups explicitly carve out room for non-Christian character beliefs and for evangelizable evolution. This is the same load-bearing-multiplicity prior playing out at the craft-note scale: faith and authenticity are not in tension; the prompt names how they coexist.

**Three reading-the-user craft notes** (`852919d`). *"Smaller sentence / restraint / false weight."* These continue the line of work the harness-day report noted: every craft addition since the audits has been an earned-exception clause or a sharper test, not a new absolute rule.

**Evolution clause** (`1e6a295`). *"Distinguish MODEL-drift (forbidden) from USER-nudge (honored)."* Clarifies an existing invariant — drift away from the character is bad; the user deliberately nudging the character toward something is good. Same shape as the agape-block guard ("shapes what you compose, not what your character declares"): when an invariant could be misread to forbid a legitimate move, name the legitimate move explicitly.

**Summary modes** (`dcf4d36`). Short / Medium / Auto with default Short. Cost discipline showing up exactly where the philosophy report predicted it would. The previous fixed "12-24 sentences" / 3,200 max tokens is now ~900 tokens by default, ~3-4× cheaper per first-fire.

**Sticky illustration** (six commits, `54b3b9e` through `f0b1521`). UX polish: when an illustration scrolls out of view, a small floating thumbnail of the latest one stays in the corner. Small-feeling but it changes how scrollable the chat *feels*. The visual present is sticky now.

**Send-to-another-chat removed from SummaryModal** (`5a00ccc`). *"Superseded by auto cross-thread."* A small deletion that confirms the broader pattern: the cross-thread memory work has matured to the point where the manual UI affordance for it isn't needed.

## What this period clarifies about the project's shape

The harness-day report ended on the framing that the reflective layer had become generative. This evening proves it: the patterns the prior reports named are now showing up unmistakably in code that wasn't designed to demonstrate them. Three independent commits this evening cite or apply the load-bearing-multiplicity prior — Backstage as the second consultant mode, the chapter two-act gate (already in canonization, now applied to chapter canonization too), the evolution clause's model-drift-vs-user-nudge distinction.

But the deepest new pattern is one the prior reports did not name: **multiplied stances**. The user can now choose:

- to talk to characters in-world (immersive Consultant) OR talk to the model about the work meta (Backstage Consultant)
- to read scenes that emerged from chat (transcript) OR commission scenes that didn't (Imagined Chapter)
- to receive characters' interior writing (NPC journals) OR receive their own (player journal)
- to get a long summary (Auto/Medium) OR a tight one (Short, default)
- to keep the illustration scrolled-out (chat-only mode) OR keep it sticky (visual present)

What ties these together is *not* "user-elevation over automation" (the philosophy-trajectory report's framing) and *not* "the reflective layer is generative" (harness-day's framing). It is something one step further: **the user is now a player who picks their frame**. The world has stances; you choose yours. The product makes those stances first-class instead of pretending one register is correct.

This is also why the meanwhile-bridge matters more than its single commit suggests. With multiple frames the user can switch into, the world has to keep its own continuity *between* the user's visits — the characters can't reset to cold-start every time. Meanwhile + journals + cross-thread memory give the world its own life-between-visits, so when you DO step in (in whichever stance), the world is already in motion. The architecture is being tuned to support the multiplicity of frames the UX is opening up.

## One thing the prior reports got partially wrong

The mission audit ranked "audit the character-journal generation prompt" as the highest-priority bug. It got fixed. The audit also predicted the journal layer would matter most for character coherence going forward. What this period shows is that the *bigger* lever turned out to be the meanwhile-bridge, not the journal. Journals shape what the character carries across days; the meanwhile-bridge shapes what they carry into *the next minute*. The latter is more felt, more often, by the user. Worth amending the prior framing: the audit was right that journal prose mattered, but it underweighted the meanwhile system as a structural lever. By the next report we'll have data on whether the bridge is paying off in actual play.

## The single most coherent pattern across these 30 commits

The project has stopped being an app in which the user does one thing and started being an app in which the user picks among several genuinely different things to do, each held with its own integrity, each defended at the prompt + UX + memory levels at once. *Two new modes of consultant. A new mode of summary. A new mode of chapter (invented vs transcribed). A new register of journal (player vs character). A new register of illustration (sticky vs not).* All in four hours. The thing being built is no longer a story-chat app; it is a small, opinionated, deeply-defended **stance picker** for inhabiting a world the model and the user keep alive together.

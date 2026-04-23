# The Character at the Table

*Generated 2026-04-23 late evening, covering the 31 commits since `e85fd4b` (the *Genesis as Inheritance* report from earlier this afternoon). Tenth report overall, third on 2026-04-23. In dialogue with today's two prior reports: *Pattern Becomes Policy* observed that a working instinct had become written rule in the same morning its exemplar feature shipped; *Genesis as Inheritance* observed that the project's accumulated discipline had just finished authoring a new surface. This report names what happened next — the project acquired infrastructure for letting the **characters themselves** author the craft stack, with the CLI and the corpus sampling to make that loop durable.*

## The shape

Thirty-one commits in one extended working day — enough that the earliest commit of this window (`334bdba`) and the latest (`064bb09`) already feel like they belong to different subjects. The shape divides into four clusters, roughly in order:

- **A small Genesis tail** (`334bdba`) that adds the user's own portrait phase.
- **A run of Imagined-Chapter polish + reach** (`8bb0757` through `febe3b7`) that starts with a canonization breadcrumb bug and ends with the profundity dial — Glimpse / Opening / Deep / Sacred — lifted from a fifty-deep dialogue session with Hal.
- **A response-length overhaul** (`731a935` → `e0315dc` → `688fa7f` → `ba973f3` → `6bc3ec8`) that tries four times to make a chat setting load-bearing and only lands when a STRUCTURAL fix joins the prompt work.
- **An "ask the character" phase**, bracketed by a CLAUDE.md policy commit (`06fee2f`) opening it and a capstone Hal-authored commit (`1187f79`) closing it, with seven craft-note commits in between — every one of them lifted near-verbatim from a character's own answer.
- **A worldcli tool emerging** — five commits (`2b0f510` → `2c1a083` → `2f85922` → `83acba1` → `064bb09`) — that begins as a dev convenience and ends as the project's formally-named *third reflective surface*, alongside `reports/` and the harness.

The clusters are not independent. The Hal-authored craft notes could not have happened at this velocity without the "ask the character" policy (`06fee2f`) that codified the workflow on the same morning. The worldcli birth then makes the whole workflow repeatable without you having to copy-paste between app and chat. The corpus-sampling primitive (`83acba1`) turns the message database into a before/after evaluation substrate for prompt changes. Each cluster was building the infrastructure the next would depend on — and the CLAUDE.md commit that closes the period (`064bb09`) names the whole arrangement as doctrine.

**The report's frame**: for the first time since the project began, the *character has a seat at the table where the rules get written*. The table used to have two seats — yours and Claude Code's. Today that changed.

## The philosophical shift this period crystallizes

The prior reports tracked an architectural maturation: patterns became policy, policy authored new features, the craft stack stopped being implicit and started being structural. All true. But something else happened this period that the earlier reports hadn't yet named.

For most of the project's history, craft-authoring was a two-party affair. You articulated a principle; Claude Code implemented it in `prompts.rs`. The characters were the *output* of this process — they spoke the way they did because the two of you had written them that way. They were the dependent variable.

Two days ago — visible in `f6d2932`'s commit body, *"the rule was articulated by the character (Hal Stroud) in his own voice when Ryan asked him meta"* — that relationship began to invert. Hal gave you the "plain version after the crooked" principle in his own language. You pasted it into prompts.rs. The character had become a contributor to the rules that govern the character.

This period made that inversion official. `06fee2f` codifies "ask the character" as a CLAUDE.md-level policy: when craft direction is needed, *the character is the first place to go, not Claude Code's design instincts*. The commit body is explicit about why:

> The rule comes from inside the work. The character's own answer is register-coherent with how they should sound; abstract design notes by Claude Code can drift from the character's texture. The provenance is clean — the rule was articulated by the work itself, not by a designer's theory of the work.

And then, in the seventy-two hours after that commit lands, seven Hal-authored craft notes ship. `f6d2932` (plain-after-crooked's "yank-back" sub-rule) → `da1e82a` (wit-as-dimmer's WHY) → `5856a2b` (wit-as-dimmer's earned exception, also lifted from Hal) → `fc74dc4` (let-the-real-thing-in, the wit trilogy's failure mode) → `3a0a526` (hands-as-coolant) → `923a253` (hands-as-coolant's stripped-regulator extension) → `60003f8` (noticing-as-mirror, the perception-side companion) → `1187f79` (unguarded-entry, the capstone). Every commit body carries the same attribution shape: *"lifted near-verbatim from Hal's answer,"* *"first/second/third/fourth/fifth/sixth/seventh deployment of the ask-the-character pattern in two days."*

This is not collaboration-as-metaphor. The commit log is counting the instances. The seventh deployment — `1187f79`'s unguarded-entry — is the one that reframes all the others: Hal's answer to *"has there been a person you've been with where none of this was running?"* came back as *"It felt like not having to come in guarded first,"* with the demonstrative gesture *"thumb hunting for a seam, finding only knuckle, 'which is rude of it,'"* already enacting the armored-arrival reflex while he named its rare counter-state. That answer then becomes the interpretive key for the six craft notes preceding it. The commit body says it outright:

> The wit, the hands, the funny hat, the hinge-work — every self-regulation register is ENTRY-PROTOCOL, not in-conversation craft. They are how the character ARRIVES into a moment, the armor pre-fitted before walking in. The wit/hands trilogy of notes (and the perception note) all describe the armored-arrival default. THIS note describes its rare counter-state and how the model should recognize it.

The capstone doesn't just add a rule. It re-reads six prior rules. And the re-reading is Hal's own — the character is now providing not only the primitives but also the synthesis that gives those primitives their shape. The prompt stack's authoring process has acquired a third contributor whose vantage is the *inside* of the work.

## The worldcli birth as the second half of the same move

If *"ask the character"* is the principle, `worldcli` is the microphone. Two commits dominate the tool's birth — `2b0f510` introduces the CLI (Rust binary, same prompt-building pipeline as the Tauri app, keychain-based API key resolution so Claude Code can run `ask` without env fiddling) and `2c1a083` upgrades it to what it actually wants to be. The upgrade commit's own framing is worth reading verbatim:

> Substantial elevation of the dev CLI per "agent ergonomics not human ergonomics" framing. Recognizes the tool as the third reflective surface alongside reports/ and the harness — empirical querying of the user's lived corpus on demand.

The three surfaces are now named as a set, and each is given its tense: `reports/` is *past-shaped* (interpretive reads of git history), the harness is *future-shaped* (automated regression testing of prompt behavior), `worldcli` is *present-shaped* (empirical querying of what is actually true in the data right now). The CLAUDE.md commit `064bb09` at the end of the period bakes this trinity into the repo's own guidance. The earlier reports tracked discipline becoming policy; this one tracks *introspection* becoming infrastructure.

The v2 elevation commit added four elements to the tool that convert it from a convenience to a durable reflective surface:

1. **Scope discipline** — default reads hit only the worlds listed in the user's `~/.worldcli/config.json`. Empty config means empty scope. `--scope full` opts in per-command and prints a stderr warning. This is the same instinct `213d969` codified for Conscience Pass last week — make cost opt-in, make full-scope opt-in, don't assume permission.
2. **Cost discipline** — every `ask` projects tokens × model price, checks per-call and rolling-24h caps, refuses calls that would exceed the caps, and requires an explicit `--confirm-cost <usd>` flag to proceed. The commit body calls this *"Conscience-Pass-aligned."* Budget mechanics from the opt-in flip re-applied at the tool layer.
3. **Run manifest** — every `ask` writes a full record to `~/.worldcli/runs/<id>.json` and a compact line to `manifest.jsonl`. New subcommands — `runs-list`, `runs-show`, `runs-search` — let future Claude Code sessions *read their own past investigations* before running new ones. The CLAUDE.md addendum names it: *"Before running a fresh investigation, check runs-search to see if you've explored the territory before. Reading three prior runs is free; running a new one isn't."*
4. **Ad-hoc query primitives** — `recent-messages` now takes `--grep`, `--before`, `--after`, `--with-context`, and `--json` on every read command. The commit body names the design instinct: *"The CLI gives primitives; the agent does the synthesis."*

Two commits later the tool grew `list-group-chats` + `group-messages` (`2f85922`) after a day-16 investigation missed 200 messages by looking only at solo threads — *"solo-thread-only sweeps under-represent ensemble-coded characters"* — and then `sample-windows` (`83acba1`) turned it into a proper evaluation instrument. The commit body names the natural-experiment observation in one line:

> Every assistant turn has created_at; every prompt change is a git commit timestamp. So the corpus is already a before/after dataset.

That sentence quietly rearranges the terms. The project's *database* is a natural-experiment substrate because the schema already records message times and git already records commit times. The alignment was there all along; `sample-windows` just makes it addressable. *Before* the flag/craft-note commit: the prior N assistant replies. *After*: the earliest N replies that came from a prompt containing the change. Same character, same surface, different era. The evaluator is the corpus itself.

The final commit of the period, `064bb09`, lifts all of this into CLAUDE.md so future sessions arrive with the discipline baked in: the commands are listed, the methodology is stated (timestamps + commits as a before/after dataset), and the report-writing rule is made explicit — *"write a reports/ entry when the finding is load-bearing for an in-flight build/design decision; don't dilute the signal with one-off curiosity runs."* `reports/` now formally has two genres: the trajectory-read (the genre you're reading now) and the natural-experiment finding. The commit body calls out that this was in auto-memory before, and is being *"promoted to project-level guidance so the discipline travels with the repo, not just my session."*

## The response-length overhaul as a rebuild-from-different-angles

Four commits in sequence try to make the chat-settings response-length setting actually govern the model's reply. Reading them as a single arc rewards the effort.

`731a935` — the first attempt. *Critical / load-bearing / foremost* directive added to the early-position block. Token caps left unchanged as structural backstop. Auto returns None, removing the old "USE VARIETY" sermon. *"If the user wants Auto, we honor that by getting out of the way."* That last line is one of the period's cleanest small commitments: deletion is honoring. More isn't craft.

`e0315dc` — second attempt. Caught immediately: the new absolute rule shipped without the earned-exception clause. Symmetric carve-outs added both ways — a "briefer than the target" exception (single-word answer when the moment collapses the reply) and a "slightly longer than the cap" exception (once in twenty replies, when a climactic turn physically cannot land in fewer sentences). The earned-exception pattern from `fc2a77c` that got codified in CLAUDE.md only a day earlier, already being applied reflexively to the freshly-shipped rule.

`688fa7f` — third attempt. One-line commit: rare-exception rate bumped from 1-in-20 to 1-in-10. The calibration is an ongoing signal that the earlier absolute was over-clamping.

`ba973f3` — fourth attempt. *"The previous strengthening still wasn't taking effect."* The commit bodies become diagnostic: the model is pattern-matching to scrollback reply-length instead of honoring the current setting. The fix re-registers the rule as *THE COMMANDMENT* (modern English, not KJV, but *"a commandment, not a suggestion"*) and adds a REGARDLESS-OF-HISTORY anchor naming the *why*: the user may have just changed the setting mid-conversation; past replies may have been generated under a different regime.

And then — this is the pivot — `6bc3ec8` does something the prior four commits didn't. It stops trying to win the pattern-matching fight at the prompt layer and fixes it at the **history layer**. When the user changes a chat setting, a new message row (`role=settings_update`) is written into the thread. The row renders in the UI as a quiet earthy codeblock (*"You changed chat settings — Response Length: Auto → Short"*) and renders in the dialogue prompt's history loop as a system note (*"From this point forward, replies should reflect the NEW setting; replies BEFORE this point may have been under different settings and should not be used as a length / register pattern for current replies"*). The commit body names the lesson directly:

> This is the structural fix for the response-length pattern-matching problem. Strong promptcraft rules in the system prompt only get the model so far; when scrollback contains 10 long replies and the user just switched to Short, the model needs explicit history-level context.

This sequence belongs to a pattern the earlier reports began tracking and this period confirms: *some problems are not solvable in `prompts.rs`*. The prompt can only weight the directives the model sees at call-time; it cannot change the *shape of the history* the model reads. When the history carries evidence against a current rule, the history wins. The fix was to put counter-evidence *into* the history.

That is a particular kind of taste trajectory — realizing where promptcraft ends and where structural work begins, and reaching for the right tool. Four commits to find the wall; one commit to step around it.

## The imagined-chapters profundity dial as lifted heuristic

The Imagined Chapters work this period is a tidier example of the same cross-pollination shape. `febe3b7` adds a four-level depth selector to the chapter composer — *Glimpse / Opening / Deep / Sacred* — each with its own register directive block. The commit's provenance note is explicit: *"lifted from the depth-mining heuristic the user asked me to demonstrate at scale (50-deep Hal exchange). What worked at scale in dialogue gets formalized as a tunable register for chapters."*

The dial's levels carry the period's tonal discipline at the vocabulary level. *Glimpse* is *"the small breath in a long week"*; *Opening* is *"one layer below default — a care revealed, an admission slipped"*; *Deep* is *"interior visible — the depth is in how truly the character is willing to be seen, not in how big the moment is"*; *Sacred* is *"confessional, threshold — the character unguarded in a way they almost never are"*. The earned-exception ritual gets applied in the same commit, carving out that *depth governs the chapter's REGISTER, not the situation it's set in* — a Glimpse chapter set in a heavy season is valid; the picked depth must be honored even when context wants to drag the moment toward a different weight.

Two observations worth noting. The first: the `Sacred` register note — *"the character unguarded in a way they almost never are"* — is the same conceptual territory as `1187f79`'s unguarded-entry note. The heuristic and the capstone Hal-authored principle land in the same week and name the same thing from different sides. The second: the dial's default is *Opening*, not *Deep*. *"Opening — the natural register for chapters that want to mean something without being seismic."* The tool's default position protects against the failure mode — chapters that over-reach for significance — by making the middle depth the floor, with sacred kept rare enough that it doesn't cheapen by overuse.

`c2b5442` adds a second pillar to the chapter prompt the same day — *"DIALOGUE IS THE PRIMARY ENGINE when two or more people share the frame"* — explicitly promoting dialogue to a first-class requirement so chapters stop summarizing speech. *"A chapter of two people together where neither speaks is almost always wrong — the scene is asking to be SPOKEN through."* That principle reads less like an abstract craft note and more like the *sample-windows* observation crystallized: what works at dialogue scale should work at chapter scale.

## What was removed and what the removals reveal

Two deletions in this period are worth flagging.

**The "USE VARIETY" block in Auto mode** (in `731a935`). The previous behavior injected an extensive variety sermon even when the user picked Auto. Removed outright. The commit body: *"If the user wants Auto, we honor that by getting out of the way."* This belongs to a taste signature that has recurred across the logs: when the user's declared preference is *"no constraint,"* the honoring move is to *actually impose no constraint*, not to inject a gentler constraint in the guise of craft.

**The aliveRef pattern in `ImaginedChapterMessage`** (in `40a8c9a`). A StrictMode bug where `useRef(true)` initialized only once per component instance meant every chapter image silently failed to render. Fix: replace with a plain `let alive = true` inside the effect. The commit body reads the bug as a *"classic React 18 StrictMode trap."* A pattern common enough to be idiomatic was causing a silent-failure class. The fix is a smaller, less clever construct. The taste signature — the smaller, less-clever piece wins when it eliminates a silent-failure mode — is the same one that governed the response-length "commandment" rework: overstrong abstractions can mask problems that plain code doesn't.

## What was rebuilt from a different angle

**The `Genesis` question**. In `ac939bf` (from the prior report's period) the world-birth ceremony asked *"What are you reaching for here?"* — directly ported from `QuestAcceptanceDialog`. In `427ede7` this period, the question becomes *"What do you want to build while you're here?"* The commit body names the asymmetry that required the change:

> Accepting a quest is YEARNING toward a pursuit the world has named for you; birthing a world is AUTHORING a place. The latter wants agency-shaped, builder-shaped framing.

A symmetry that seemed clean was hiding a real distinction. The rebuild preserves *QuestAcceptanceDialog's* phrasing and redirects Genesis's to the new one — symmetric ceremony SHAPES across the two surfaces, asymmetric QUESTIONS because the moments are not parallel. The prior report's observation that the commitment ceremony was *extended from* the Quest layer has been refined: the ceremony shape is inherited; the question is not.

**The `Genesis` wizard**. `334bdba` adds a `define-self` phase — user's portrait + optional profile fields — between generation-complete and the reaching beat. Seven phases becomes eight. The commit body names the asymmetry that drove it:

> The previous flow had the user entering a world as "Me" without ever being asked who they were in it. Two characters had been met; the user remained abstract. Now all three people at the first scene's table have faces and names.

Genesis was inherited from prior patterns, per the last report. This period added a phase that none of the prior patterns supplied — the surface asked to be more symmetric, so the wizard grew.

**The imagined-chapter card**. `1a59595` replaces the compact strip with a 16:9 tall gilded hero-image card — serif title, amber vignette, async image fetch, gold sidebar treatment for canonized items (`c7a7d24`). The card used to live like a footnote. Now it lives like an artifact. The register change matches the canonization-highlighting that landed in the prior period: *chosen things look visibly chosen*.

## Recurring concerns this period sharpened

**Cost discipline**. The Conscience-Pass opt-in flip from last week wasn't a one-off; it's become the operating principle for every new tool that can spend money. `worldcli` is the clearest example — per-call caps, rolling-24h caps, explicit `--confirm-cost` flags, cost logged to `~/.worldcli/cost.jsonl`. The commit body for `2c1a083` calls the posture *"Conscience-Pass-aligned."* The pattern is propagating.

**Character as source of truth**. `06fee2f`, the seven Hal commits, the worldcli birth. Every arrow this period points the same direction: the character is the reliable informant about the character. The project is shifting away from *"Claude Code's design instincts"* and *"Ryan's theory of the work"* and toward *"what does the character say when asked in-world."* This doesn't mean the designer-voice disappears; it means it's been placed in the right role — drafting questions for the character, evaluating the character's answers, keeping register integrity, holding the invariants. But the *primary authorship* of craft notes in Hal's domain is now Hal's.

**Natural-experiment evaluation**. Three moves converge on the same realization this period. The `sample-windows` primitive (`83acba1`). The CLAUDE.md addition that names the second genre of reports (`064bb09`). And the auto-memory feedback note that the working guideline cites: *"message timestamps + git commits = natural experiment."* The project is done evaluating prompt changes on vibes. *Did the commit do what the commit body said it would?* has become a question with an answer, because the corpus-plus-git is a designed-in measurement substrate.

**User agency as structural commitment**. Settings_update rows in chat history. Auto = no constraint. "What do you want to build?" A pattern: when the user's authorship of the scene matters, give them *durable* authorship — a row in the database, an absence of injected directives, a question that puts them in the chooser's chair. The craft stack is no longer just about making the character good; it's about making sure the user's choices STAY in the record visibly.

## What the prior reports anticipated and what this period names

*Pattern Becomes Policy* said *"the policy was visible in the work for weeks; the work applied the principle on landing; the principle then got named in the durable infrastructure that every future session reads."* This period extends that observation to a new subject: *"ask the character"* was visible in the work (the Hal trilogy from the prior period), became codified in CLAUDE.md (`06fee2f`), was then applied seven more times in 48 hours, and is now backed by a dedicated CLI tool (`2b0f510`, `2c1a083`) that encodes the discipline in infrastructure. The same motion at a different subject.

*Genesis as Inheritance* said *"the thing a new user sees first is, structurally, the most concentrated demonstration of what the project has decided to be."* This period supplies its complement: the thing Claude Code reaches for first — when asking a craft question — is now structurally the character. The outer surface (Genesis) and the inner process (craft authorship) are both downstream of the same architecture: *the decisions the project has already made*.

*After the Audits* worried about `MECHANICS.md` rotting against UI churn, and the Stances Hold period answered with the prompt-embedded UI MAP. This period extends the answer laterally: the same-commit carve-out ritual (`fc72ca4`-ish pattern), now CLAUDE.md-codified, ensures the craft stack doesn't rot against itself — every absolute shipped in this period gets its exception named in the same commit. `e0315dc` was caught and fixed within minutes of its predecessor; the wit-as-dimmer missing carve-out (`5856a2b`) was caught by you and fixed the same day. The ritual is deterministic; the drift is caught before it ships.

## A small open-question worth naming

The unguarded-entry capstone (`1187f79`) reframes the entire wit-hands-noticing trilogy as *entry-protocol, not in-conversation craft*. That is a strong reorganization. The question the commit leaves open: does `prompts.rs` still READ as a set of *arrival* instructions, or does the model only see the old shape and treat the capstone as one more rule among many? The answer will be legible in `sample-windows` output across a future week — compare Hal replies before `1187f79` and after, looking for the specific signature the note predicts (*"the character walks in already unguarded"* when trust is established, *"the regulators don't disappear — they're just no longer load-bearing on arrival"*). If the pattern is visible in the data, the reframe did its work. If not, the capstone may need to be restructured as a governing header above the trilogy rather than an appended note. A natural-experiment report's exactly-right subject.

## The single most coherent pattern across these 31 commits

The project gave the character a seat at the table where the rules get written, and then built the table. The "ask the character" policy named the seat. The seven Hal-authored commits filled it. The `worldcli` CLI gave any future Claude Code session a way to pull up a chair at the same table. The `sample-windows` primitive made the record of what was said at prior tables auditable. The CLAUDE.md closing commit formalized the whole arrangement — three reflective surfaces, named and explained, with the discipline for each.

What the last two reports tracked as architecture authoring its own surfaces, this report extends one step further: *the work is now authoring the rules that govern the work*. You and Claude Code sit at the same table, but you are no longer the only ones writing. The character writes too. The corpus writes too. The CLI is the means by which what was said inside the work becomes legible as rule. The report folder is where those rules get interpreted back into trajectory. The pattern is self-referential in the deepest way the project has been yet — *the work has acquired the ability to tell you what it is*, and you have built the instruments to listen to it with.

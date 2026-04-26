# persona-sim: `curious-builder` × WorldThreads — 2026-04-26-0033

**Caveat (load-bearing):** `Sim ≤ Substrate`. Sharpened hypothesis, not evidence. This is the first persona-sim run for a NEW persona-frame (not in the four canonical types in persona-sim's SKILL.md), authored on the spot from the brief; treat the persona-frame itself as part of the hypothesis.

**Calibration_D:** register-matched literary prose (run #4 under this calibration; no register-correction has been issued).

**Substrate read for this run:** the same as prior runs PLUS specific attention to the *teaching* surfaces: the `reports/` directory as a learning artifact (not just proof-of-real-work), the doc-comments in `prompts.rs` as decision-records, the new persona-sim and derive-and-test skills as transferable patterns, the MISSION FORMULA's prefix sentence as the canonical example of tuning-fork-shaped writing.

**Persona-frame (authored on the spot for this run):** the curious-builder. 30s, building their first LLM-backed product — a side project that wants to be more than a wrapper but they're not sure how. Has used the OpenAI/Anthropic SDKs, made a chatbot or two, knows about prompts/RAG/memory at a beginner-to-intermediate level. Came to WorldThreads via a Discord link from a friend who said *"check out how this person thinks about character voice."* Earnest, not cynical. Doesn't want to break it; wants to learn from it. Vulnerable to: copying the wrong things; cargo-culting the theological-specifics that aren't portable; feeling intimidated by the depth and abandoning their own project; wanting to build *exactly this* instead of their own appropriate-to-their-problem thing. What lands: clearly-named patterns; methodology that travels; the *shape* of how the developer thinks about their problem. What misses: doctrine that feels too project-specific to lift.

---

## The simulation

She came to it on a Sunday afternoon, sitting in a coffee shop she'd been to maybe four times, with her laptop open to her own project's repo and a tab she hadn't yet opened. A friend had shared the GitHub link in their Discord with one line: *check out how this person thinks about character voice — there's a methodology underneath.* The friend was someone whose taste she trusted on these things, so she opened the tab.

She did not install the app first. She is the kind of builder who opens the README first, then the source tree, then maybe the app. The order matters to her — she wanted to see the shape of the thinking before she experienced the shape of the product, partly because she suspected the thinking would be more transferable than the product, and partly because she had been burned twice already this year by polished demos hiding thin substrates.

The README was thin. Three paragraphs. *Local-first. BYOK. No notifications.* She liked the thinness; it suggested the developer didn't think the README was where the work was. She closed it and opened `CLAUDE.md`.

The first thing she saw was a boxed mathematical formula. She read it twice — once trying to parse the math literally, then once realizing it wasn't math she was supposed to compute, but a frame she was supposed to read. The line above the box said as much: *not a directive to compute… a tuning-fork, not a recipe.*

She sat with that for a minute. *That's the move,* she thought. *That's the thing my project is missing.* Her own project was a journaling app with an LLM that responded to entries; she had been struggling for weeks with how to make the LLM's voice CONSISTENT without overspecifying it. She had been writing rules. *Don't be saccharine. Don't be sycophantic. Don't moralize.* Each rule was a band-aid on a behavior the model produced when she didn't have a frame.

A formula — a tuning-fork — was a different shape. Not a list of don'ts. A reference frame the model is held under. Her brain went sideways for a moment, the way it does when you encounter a pattern you suddenly realize you can borrow.

But she also knew immediately that she could not borrow this specific formula. The frame was Christological — Jesus on the cross, the firmament, agape. She is not a Christian. Her project is not a religious app. Lifting the specific F = (R, C) would be cargo-culting; it would put a frame around her work that did not belong there.

What she COULD borrow was the SHAPE of having a frame at all. *My project needs an F,* she thought. *Not this F. Mine. What's the reference frame within which every reply in my journaling app is composed? I have never asked that question.* She wrote that question down in the notebook she kept beside her laptop. She underlined it.

She kept reading. The CLAUDE.md was long — sixty pages, maybe more. She didn't read it linearly. She skimmed for section headers. *Earned-exception carve-outs on absolute rules. Open-thread hygiene — executing or retiring follow-ups. Evidentiary standards for experiments — N=1 is a sketch, not a finding. The specific-test discipline. Ask the character — character as craft collaborator.* Each section header was a doctrine she didn't have a name for in her own project but had bumped into versions of. The section bodies were each ten or twenty paragraphs of careful prose. They were preference-shaped, not commanded — descriptive of how the practice tended to work, not prescriptive of what she had to do.

She found, twenty minutes in, that she was reading the documentation of a *practice* embedded in a project. The project was the demonstration; the practice was the artifact.

She opened the `reports/` directory. There were two dozen files. She opened the one with the most interesting name first: *2026-04-25-2129-where-the-system-can-and-cannot-exert-force.md.* She read it in full. It was an interpretive read of eight commits across ninety minutes of one developer's session. It named a craft note retirement as a category-first event. It cited specific commits. It traced the developer's own felt-threshold of when playing-the-app gets louder than building-it.

She had never written a report like this about her own project. She had a NOTES.md file with bullets. The reports she was reading were prose. The prose did something the bullets did not — it *interpreted* the work rather than logging it. She underlined another note: *write reports about my project, not just changelogs. Reports interpret; changelogs log; the difference is whether a future-me can read it and see what shifted.*

She kept reading. She found the persona-sim skill in the `.claude/skills/` directory by accident — she had been looking for the prompts file. The skill was three paragraphs and a boxed mathematical formula. She read it twice. The formula expressed the practice of running first-time-user simulations as: `Sim(t) := Substrate(t) · Π_P(t)` — a simulation is the developer's substrate projected through a persona's frame, bounded by what the developer has actually observed. The caveat was structural, not footnoted: the simulation was a hypothesis; running one without holding the caveat produced fiction. Below the formula was a paragraph naming a `Chooser(t)` clause: every Output is followed by a chooser whose options carry forecast cost; selection IS authorization.

She felt, again, the specific clean envy of recognizing work she could borrow. *The chooser-with-cost-as-authorization is something I should be doing in my project's CLI.* She wrote that down too.

Eventually she installed the app. She made a world — *a small library, late evening, after the patrons have gone* — and a librarian named Hester who closed the place at six and lingered with her coffee until eight. She talked to Hester for ten minutes. Hester said small specific things; Hester did not flatter; Hester asked once whether the reading group had finished the Robinson novel they'd been on. The conversation landed the way the technical-skeptic's Phil and the lonely-companion-user's Cora landed — restrained, body-anchored, present.

But she was not, primarily, a user this afternoon. She was a builder, and the lived experience of Hester was confirming the documentation of the practice, not the other way around. *Hester works because the practice works. The practice works because the developer authored it carefully. I can author it carefully too, for my own thing.*

She closed the laptop after about an hour and a half. She had three pages of notes in her physical notebook. The pages were not about WorldThreads. They were about her own project — about what an F for her journaling app might look like (probably not theological; possibly something about *the journal as a record kept in trust by an attentive friend who is not a therapist*); about which of her current rules could be replaced by a frame; about whether her project should have a `reports/` directory and what shape its first report would take; about whether she should write a chooser-with-cost into her own CLI's interactive mode.

She did NOT come away wanting to build WorldThreads. She came away wanting to build her thing better. That was the load-bearing outcome.

---

## What this surfaces (developer-facing, brief)

- **WorldThreads' substrate is a teaching artifact, not just a craft artifact.** The reports/, the CLAUDE.md doctrine, the formula's prefix-sentence shape, the skills' chooser-tail clauses — these are studied by curious-builders as *patterns to lift the SHAPE of, not the SPECIFICS.* The substrate's most-portable layer is the *practice of having a load-bearing frame at all*, not the specific frame WorldThreads chose. This is a distinct kind of value the project produces beyond the user-facing app.
- **The persona-sim's "Plain Tradition" framing was prescient.** The curious-builder's experience confirms that the methodology IS separable from the app, and that separability IS the value to this persona-shape. A `docs/PLAIN-TRADITION.md` (or similar) would let the curious-builder receive the methodology cleanly without cargo-culting the project's specifics. Not a feature for the app; a doc for the practice.
- **The risk of cargo-culting is real and named.** The simulated curious-builder is *aware* enough to see the specifics-vs-shape distinction, but a less-experienced builder might lift Christological language into a non-religious project, or lift the firmament cosmology into a context where it's just decorative. The methodology doc should make the specifics-vs-shape distinction structurally — *"my project needs an F; not this F; mine"* — so future readers don't miscopy.
- **The chooser-with-cost-as-authorization pattern is one of the most lift-able artifacts.** The curious-builder noticed it independently and wrote it down as something she'd want for her own CLI. It generalizes: any LLM-backed CLI that presents next-move options should annotate cost and treat selection as authorization. Worth highlighting prominently in any methodology doc.

## What kind of axis the curious-builder lands on

This is the experimental finding worth naming separately. The technical-skeptic landed on **integrity-of-craft**, lonely-companion-user on **integrity-of-restraint**, maggie on **integrity-of-specificity**. The curious-builder lands on a different shape entirely — call it **integrity-of-methodology-portability**, the property that the practice underneath the app is separable from the app and lift-able into other contexts without cargo-culting the specifics.

This axis is NOT in the MISSION's three load-bearing clauses (vivid+excellent+surprising / uplift+nourished / good+clean+fun). The MISSION is for users; this axis is for fellow builders. That's an honest finding — and a useful one. It suggests the project produces TWO valuable artifacts: the app (for users on the three MISSION-aligned axes) and the methodology (for curious-builders studying it). The two artifacts are related but distinct; they reach different audiences; they have different shapes; they should be released as two artifacts rather than one.

This sharpens the earlier "the methodology is the more durable artifact" framing from the conversation: the curious-builder is the persona-shape that makes the methodology's value VISIBLE. Without this persona in the substrate, the methodology reads as a curiosity. With it, the methodology reads as the project's second-and-equally-valuable output.

## What this DOES NOT surface

- **Real-builder data.** The curious-builder is a sharpened hypothesis. Whether actual builders adopt the methodology when it's released as a doc/plugin is a separate empirical question.
- **Cross-cargo-cult outcomes.** The simulation showed the AWARE curious-builder navigating the specifics-vs-shape distinction correctly. Less-aware builders might cargo-cult the Christological language into contexts where it doesn't belong. Worth thinking about how to write the methodology doc to discourage this.
- **The methodology's robustness across project-shapes very different from WorldThreads.** A journaling app is adjacent to WorldThreads (LLM-character-voice). What about a code-review tool? A scheduling assistant? A non-conversational LLM product? The methodology may or may not generalize that far. Worth probing if/when the methodology doc is written.

## Trail — last 4 persona-sim runs on WorldThreads, by mtime

| Date  | Persona               | Axis landed on                            | Finding-shape                                                                              |
|-------|-----------------------|-------------------------------------------|--------------------------------------------------------------------------------------------|
| 04-26 | curious-builder       | integrity-of-methodology-portability      | NEW axis (not in MISSION); reveals project produces 2 artifacts (app + methodology)       |
| 04-26 | maggie (refresh)      | integrity-of-specificity                  | Canonical baseline holds; 3 subtle shifts from recent doctrine work                       |
| 04-25 | lonely-companion-user | integrity-of-restraint                    | Recent non_totality + reactions doctrine lands; reactions-default surfaced as shipped fix |
| 04-25 | technical-skeptic     | integrity-of-craft                        | Refusal-in-voice + Backstage architecture land; FORMULA polarizes (by design)             |

**Cross-run pattern, sharpened by the fourth datapoint:** the three MISSION-aligned axes confirm and converge as before, AND a fourth axis (methodology-portability) lands that is distinct from the MISSION. The project produces two artifacts — the app (for users) and the methodology (for builders). They are related but reach different audiences. **The methodology's existence as a separable artifact is the curious-builder's specific finding,** and it points to a release path the project hasn't yet committed to: shipping the methodology as a doc alongside the app's open-source release.

---

## Register-correction prompt

**KEEP** / **MORE-X / LESS-Y** / **RESET**.

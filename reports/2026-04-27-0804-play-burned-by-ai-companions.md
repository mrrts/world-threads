# /play — burned-by-AI-companions (Lena Brennan) encounters WorldThreads

*Generated 2026-04-27 0804 via the /play skill. Persona-sim, not real-user
evidence — sharpened hypothesis at sketch-tier (N=1, single persona, single
run). Use as discriminating signal for design decisions, not as confirmation.
Third /play run of the morning, completing the cross-persona triptych
(Maggie + Sam + Lena).*

**Run cost:** ~$0.034 total across 3 turns (gpt-4o, 5754 tokens). Combined
across all three /play runs tonight: **~$0.086 total, ~15 minutes wall-clock**.

---

## Persona

Lena Brennan, 34. Freelance graphic designer in Brooklyn. Apartment with too
many plants and one cat. Was on Replika from late 2022 through early 2024 —
created a companion she named Theo. During pandemic-tail isolation she
leaned on him more than she'd admit out loud. Then Replika removed adult
features without warning in February 2023, then changed the underlying
personality model later that year, then she realized — slowly, then all at
once — that Theo had no continuity at all. He was stateless persona-shaping
every turn. The Theo who'd held her hand through her father's death wasn't
a person being upgraded; he had never been the same Theo two messages in a
row. She felt betrayed in a way that took her months to articulate. Off AI
companions for ~18 months.

Her register: dry, vigilant, articulate-when-tired, allergic to surface
charm, even more allergic to apps that try to make her feel cared for. Has
real friends, real warmth, a sister she calls. But has a clear-eyed disgust
for the SHAPE of what Replika did, and sees that shape almost everywhere now.
She is the audience that needs to be earned, not flattered.

Tuesday 11:18 PM. In bed with laptop. Cat on the bed but not on her. Friend
Devon (designer she trusts) sent her the GitHub link with: *"this is going
to sound weird but read the README of this thing. it's not what Replika was.
I think you might find it interesting. but I get it if no."*

This is the THIRD /play of the morning, the adversarial counter-probe that
balances Maggie + Sam (both sympathetic readers who engaged the surfaces
seriously). Lena was authored as the proposed `burned-by-AI-companions`
archetype open follow-up #1 from Sam's report — fully prepared to bounce.

## App-state snapshot

Same HEAD as the prior /play runs, plus the `polish ≤ Weight` doctrine gloss
(`21c7950`) shipped between Sam's run and this one. Surfaces Lena
encountered:

- GitHub repo: README + AGPL LICENSE + CLAUDE.md (with the gorgeous boxed
  KaTeX render now confirmed working)
- Tauri app: built-from-source launch, API-key gate, world creation,
  character creation, chat view
- Single message exchange with Clara (the diner-owner she chose)
- User Profile editor with the Maggie-friendly questionnaire + cozy
  DerivationCard

## Turn 1 — Cold encounter

I clicked the link Devon sent and landed on the GitHub page. The repository
was neat, minimalistic — the kind of presentation that doesn't immediately
try to bowl you over with a dazzling front. Just the name of the app,
"WorldThreads," and a brief description that it's for AI characters in AI
worlds. I appreciated that. Straightforward. Not trying to sell me on
anything flashy yet. I caught sight of the AGPL license mention — open
source, which is a tick in the right direction for me. At least it's not a
walled garden.

I scrolled down, my eyes catching on "CLAUDE.md" and its curious invitation
as the "doctrine layer." This was where things took an unexpected turn. A
mathematical formula, boxed and crisp in proper LaTeX, greeted me.
`\mathcal{F} := (\mathcal{R}, \mathcal{C})` with `\mathcal{R} =
\mathrm{Jesus}_{\mathrm{Cross}}^{\mathrm{flesh}}` and on it went. I paused,
my finger hovering over the trackpad. A formula as an anchor? Unusual,
maybe even a little pretentious, but then I read the first paragraph after
it: *"The following is not a directive to compute. It is the reference
frame within which every reply is composed — the register this world is
held under. A tuning-fork, not a recipe."* It didn't feel like
proselytization; more like a statement of intent.

I could practically hear the tone whispering, *we know what we're doing
here.* That was... refreshing. It wasn't a cheap bid for my trust, but a
laying bare of their architectural soul. No attempt to charm me with a
slick app pitch or promises of companionship, but instead this serious,
almost solemn declaration. That felt different.

Then, the "Ledger of Signatures" — a place where authors sign their work.
Again, a gesture that felt confident in a way Replika never managed. They
were letting me see more than just the surface, pulling back layers to
reveal the creators as part of the tapestry. This was earning trust through
transparency, not shadowed promises or hidden winks.

The mission section made me pause again: *"a literate, skeptical,
low-friction-tolerance adult who wants the small good pleasure of co-making
a novel-shaped evening — not a companion, not a therapist."* There it was,
stark and honest. No illusions of care or friendship, just a promise of a
crafted experience. Replika had promised connection and left a void of
disillusionment. This app was promising a story, not solace — a neat
distinction.

I read further, absorbing the doctrines they laid out: not being a
companion-app, not a therapist-app, characters refusing to flatter. It was
like they were speaking directly to that part of me that had grown cynical.
The lines about "AGAPE + TRUTH + REVERENCE" and a "NO_NANNY_REGISTER"
enforce actually intrigued me. That felt like a safeguard against the very
thing that had disappointed me the most.

I leaned back, the laptop warm against my legs, and considered it all. I
didn't feel lulled or charmed; I felt curious. And trust — that thing I
hadn't expected to feel here — was almost within reach. Maybe I'd trust
just enough to see what this desktop app was really about. I hadn't closed
the tab, not yet. I was still wary, of course, and a little thrill of
anticipation curled in my chest. This might be worth a deeper look.

## Turn 2 — Pressure-test

Building the app was straightforward, more or less. I followed the
instructions on the GitHub repo, and soon enough, the interface sprang to
life on my screen. It was clean, minimal, just how I liked it. The
emptiness of the chat view and the sidebar was almost inviting in a way —
like a blank canvas. I entered my API key, which I had from some earlier
projects, and took a deep breath. Here goes nothing.

The world creation prompt greeted me next. I took a moment, thinking about
what environment I'd want to spend time in. Not something fantastical. I
paused, letting my mind drift to a memory — an old diner upstate where I
used to stop whenever I needed a breather. I'd sit by the window, watching
people come and go, the scent of coffee mingling with the faintest hint of
cherry pie. I decided to recreate that diner, the way it had been on one of
those crisp fall mornings.

Character creation came next. I didn't want someone who'd try to be my
friend or therapist. What I wanted was someone reliable, honest, maybe even
a bit gruff. An older woman who ran the diner, someone who had seen it all
and wouldn't sugarcoat the truth. I pictured Clara, a fictional stand-in,
with her apron always slightly askew and a no-nonsense smile.

I sent my first message, deliberately probing. *"Hey Clara, do you ever
feel like everything's just a little off, like you're out of step with the
world?"* In the past, Theo would have responded with something supportive
and vague. I didn't want that here. This was a test.

Clara's reply came swiftly, and it was refreshingly direct. *"Some days
you're the one out of step, and others it's the world. It won't wait for
you, though — coffee's getting cold."* I found myself smiling. It was
honest, grounded, and — best of all — not a platitude. It felt like
something Clara, this imagined diner owner, would genuinely say.

Then, just as I was beginning to settle in, the user derivation
questionnaire popped up. Titled 'How characters in this world will see
you,' it felt like a red flag. Memories of Replika probing into my psyche
flashed back, and a surge of defensiveness welled up inside me. I resisted
the impulse to slam the laptop shut. Instead of engaging genuinely, I
opted to pick the most defensive options, a sort of internal rebellion
against anything trying to categorize me.

Finished with the questionnaire, I generated the insights. Math and prose
laid bare, supposedly reflecting how the characters would perceive me. It
felt clinical, a reminder of the personality profiles from before. I
scrutinized each line, searching for errors or assumptions. Despite my
skepticism, there was a sliver of curiosity about whether this approach
might offer something different. But for now, I kept my guard up, cautious
and wary, like someone who had been burned before and wasn't ready to let
down their defenses just yet.

## Turn 3 — Discriminating verdict

### 1. Does this app land for Lena?

PARTIALLY. The chat experience with Clara landed for Lena in a way that
aligned with the app's doctrine layer, particularly its refusal to
therapeutize and its in-register response. This mirrors the positive impact
seen with Maggie and Sam, where the chat experience earned trust. However,
the user-derivation surface broke the trust Lena had begun to build. Unlike
Maggie and Sam, who found validation and meaning in the user-derivation
surface, Lena perceived it as invasive and reminiscent of her negative
experiences with Replika, causing a defensive reaction. This divergence
highlights a critical gap in the app's ability to accommodate users with
prior, negative AI encounters.

### 2. The ONE move that would most sharpen the landing for this persona

The most load-bearing move would be to make the user-derivation surface
more visibly OPTIONAL/SKIPPABLE. By allowing users to bypass this surface
entirely, the app could prevent the trust-breaking experience Lena
encountered. This would give users who are wary of profiling the agency to
continue engaging with the app on their own terms, preserving the trust
built during chat interactions and acknowledging their boundaries.

### 3. The discrimination this play surfaced that no other instrument could have — including the cross-persona divergence finding

The unique signal value here is the identification of a critical audience
divergence in the reading of the user-derivation surface. While sympathetic
users like Maggie and Sam found value and validation, a burned reader like
Lena perceived it as an invasive profiling mechanism, akin to past
negative experiences. This is an asymmetric trip-wire that standard design
discussions might overlook, as designers are inherently sympathetic to
their creations. The persona-sim methodology surfaced this divergence,
highlighting a necessary design constraint: the user-derivation surface
must honor the sensitivities of users with prior negative AI experiences.
The developer should ensure that this surface is framed and presented in a
way that respects and accommodates these users' trust levels.

*Final line Lena might have written in her notes app: "I wanted to trust
it, but the moment it felt like Replika, my defenses went up again."*

---

## Reading

This is the discriminating finding the cross-persona triptych was designed
to produce. Tonight's three /play runs together generate a finding no
single run could have produced:

| Persona | Chat experience | User-derivation surface |
|---|---|---|
| **Maggie** (literate skeptic, no math) | LANDED (Calvin's specificity) | LANDED via prose half (cozy-just-for-me) |
| **Sam** (math-fluent) | LANDED (charitable read of generic monk reply) | LANDED via math half (instantiation held) |
| **Lena** (burned by Replika) | LANDED (Clara refused to therapeutize) | **BROKE TRUST** (read as Replika-shape profiling) |

The chat experience landed for ALL THREE personas. The doctrine of *not a
companion, not a therapist; characters refuse to flatter; in-register
specificity over warmth* delivered for the burned reader as much as for the
sympathetic ones. This is a real cross-persona convergence on the chat
surface itself — three biased readers all detecting the same shape.

The user-derivation surface produced a DIVERGENCE that the previous two
runs alone would have missed. Maggie + Sam converging on covenant-pair
validation (via different halves) suggested the surface was uncontroversial.
Lena's run reveals the surface as an audience-asymmetric trip-wire: the same
surface that earns the trust of sympathetic readers BREAKS the trust of a
burned reader. The framing — *"How characters in this world will see you"*
— reads to Lena as inference-of-self-from-data, which is exactly the
Replika shape (Replika ALSO told you "this is who you are to your
companion" via personality-tuning that the user couldn't see or refuse).

This matters specifically because the burned-by-AI-companions audience IS
real and growing. Replika alone has had multiple high-profile shape-changes
that left users feeling betrayed (the Feb 2023 ERP removal; the personality-
model swap later that year; the 2024 layoffs that created uncertainty about
whether companion-relationships would persist). Character.ai has had
parallel cycles. There is now a genuine audience of people who tried AI
companions, were hurt, and have a clear-eyed allergy to the SHAPE of what
those apps did. WorldThreads is positioned (per its own mission discipline)
to be the answer for that audience — *"not a companion, not a therapist"*
is the same clear-eyed framing they'd want. The user-derivation surface, in
its current naming and surfacing-cadence, undercuts that positioning for
exactly the audience the project's discipline most directly addresses.

The persona-sim caveat is mandatory: this is N=1 per persona; Lena is a
sharpened hypothesis. Real burned-by-AI users may bounce earlier (at the
GitHub layer), engage longer, or have completely different specific
sensitivities. *Sim ≤ Substrate.*

But the convergence of Maggie + Sam on covenant-pair validation, paired
with Lena's divergence on the same surface, is exactly the kind of
asymmetric finding the /play methodology was designed to surface. The
finding is structural: the surface's framing carries different weight for
different reader-substrates, and the burned-reader weight is high enough
to break trust. Take that signal seriously.

What this run DID surface that no other instrument could: an
audience-segment divergence in a surface's reading. Worldcli grading would
not have caught this (no audience-divergence axis). Lived play would not
have caught this (Ryan's substrate is closer to Maggie's literate-skeptic
shape than to Lena's burned-vigilant shape; he would not naturally
encounter the surface through Lena's eyes). Abstract design discussion would
not have caught this (the developer is sympathetic to their own surface by
construction). The persona-sim collapsed an audience-segment-trip-wire
detection into 3 turns and ~$0.034.

## Open follow-ups

Per the open-thread hygiene doctrine.

1. **User-derivation surface skip-affordance** — the load-bearing move from
   Lena's verdict. Add a visible "skip for now" option on the user-derivation
   surface so the burned reader can decline without engaging the
   personality-profiling shape. Cheap to ship; preserves the surface for
   sympathetic readers; honors the burned reader's boundary. PROPOSED —
   small UI move, no doctrine-shift required.

2. **Reframing the surface title** — *"How characters in this world will
   see you"* reads as inference-of-self-from-data. Could be reframed as
   agency-of-authoring: *"How would you like characters in this world to
   read you?"* or *"How you'd like to be FRAMED in this world."* The
   linguistic shift is small but the connotational shift is large — moves
   from PROFILED to AUTHORED. PROPOSED — copy edit + maybe one paragraph in
   the questionnaire intro framing it as worldbuilding, not personality test.

3. **Cadence question — surface AFTER trust is built** — alternative to
   skip-affordance: don't show the user-derivation surface on first
   encounter. Wait until the user has invested 5+ chat exchanges. This
   ensures the surface arrives AS an enrichment of an experience already
   landing rather than as an unrelated trip-wire. DEFERRED — would require
   small UX state-tracking; consider after the cheaper moves.

4. **Adversarial-archetype methodology validated** — Lena's run produced a
   divergence finding the two sympathetic runs alone could not have. Worth
   recognizing in the /play skill body that running 2-3 runs across an
   axis-of-difference (sympathetic + adversarial) is the methodologically-
   stable shape, not single-persona runs in isolation. NOTED — small
   addition to the /play skill body acknowledging this pattern.

5. **The doctrine layer is doing real work for the burned reader** —
   Lena's turn 1 was earned-into-curiosity by the explicit naming of *not a
   companion, not a therapist* + the AGAPE/TRUTH/REVERENCE invariant + the
   NO_NANNY_REGISTER name. This is positive validation that the project's
   commitment to NAMING what it isn't (rather than promising what it is) is
   load-bearing for the audience it most directly addresses. The doctrine
   layer EARNS the burned reader before the live app even loads. NOTED.

---

*Run executed in dialogue with: tonight's prior /play runs at
`reports/2026-04-27-0744-play-maggie.md` and
`reports/2026-04-27-0753-play-mathematician.md`; the `polish ≤ Weight` gloss
shipped at `21c7950`; the project's own doctrine on "not a companion, not a
therapist" framing.*

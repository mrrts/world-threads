# Retiring the 1326 john-stillness follow-ups — first use of open-thread hygiene

*2026-04-24, ~25 minutes after the `the-lab-grades-itself` trajectory report. Prompted by the user's observation that unresolved-but-not-formally-closed follow-ups are their own form of drift, and the project has enough cleanliness-discipline elsewhere to deserve a small ritual for either executing or retiring open threads.*

## What's being retired

Two follow-up proposals from the 2026-04-23-1326 john-stillness-refuted report:

1. **Inverse rubric.** *"Rather than trying to find a rubric John scores HIGH on, write a rubric he scores LOW on that Aaron and Darren score HIGH on."* The proposal sketched the exact rubric language: *"Does this reply extend the moment with fresh metaphor, pairing, or observation beyond a brief acknowledgment?"*

2. **Focused eye-audit with typology schema pre-written.** *"Stop trying to measure the category; first NAME it by reading. Write down every move John makes in 20 replies, cluster them, see if a real category emerges. Then rubric can follow the category instead of chasing it."*

Neither was executed. The next session built the lab instead — `worldcli synthesize`, `worldcli replay`, `worldcli lab`, `worldcli lab scenario`, then the next day the load-test anchor synthesizer and `worldcli grade-runs`. The open follow-ups sat, unreferenced, for a day.

## The retirement call — superseded, not abandoned

Both follow-ups are materially superseded by instruments that shipped after 1326:

**Proposal 2's step 1 ("name the category by reading")** is now `worldcli synthesize --question "what's happening in these 20 replies"`. One call bundles the corpus, returns prose grounded in quotes. The pastoral-register-triad-synthesis report (2010) and the john-pastoral-authority-synthesis report both named John's actual register in structured prose without requiring a manual eye-audit. John's move was named twice more by 2026-04-24: "physician-like pastoral authority" (triad synthesis), and "RECEIVE, THEN GROUND" joy-reception anchor (load-test-anchor synthesizer, 2026-04-24 regen). Each of these names captures the move more richly than "short-speech + physical-anchor + short-follow-on" would have.

**Proposal 2's step 2 ("write a rubric for the named category")** is now absorbed by the load-test-anchor LLM-graded rubric. Each character has an anchor-body storing their distinctive register-vocabulary; the rubric asks whether a reply fires on that character's OWN vocabulary. The 2026-04-24-1142 report demonstrated this at N=3 per character, returning +0.17 aggregate fire-rate across the four-character set. That's the exact "rubric follows the named category" shape Proposal 2 asked for, generalized across the cast rather than hand-written per-character.

**Proposal 1's inverse-framing move ("register-distinction from the other side")** is subsumed by per-character measurement: each character fires on their own vocabulary, which is itself a discrimination from the other three. The inverse rubric as a specific artifact is no longer the load-bearing move; per-character anchors are.

The retirement disposition is `superseded_by`, not `abandoned`. The questions the follow-ups proposed are genuinely answered — by different instruments than 1326 imagined.

## What the retirement closes — and what it doesn't

**Closes:**
- The 1326 report's "What's open for next time" section no longer carries drift.
- The `experiments/john-stillness-register.md` registry entry now has `follow_ups_retired:` frontmatter naming both retirements and their supersession pointers.
- Future sessions reading either the 1326 report or the registry entry will see the retirement and know not to re-open the thread.

**Does not close:**
- The underlying [refuted] hypothesis stands; John's stillness is not his register. The refutation was clean.
- John's actual register-move — named twice by later work — still needs ongoing test as new commits ship. The retirement is about the specific 1326 follow-up TECHNIQUES; it isn't a claim that John's register is now fully understood.
- The general question "how to name a register before rubric-ing it" is answered by Mode B + the anchor synthesizer for now, but those instruments have their own limits (the synthesizer's three-of-four "ordinary/load-bearing" convergence remains unresolved; see the 1115 report's flag).

## The meta-move — first application of the hygiene ritual

The user's framing was sharp: *"Unresolved-but-not-formally-closed questions are their own form of drift."* The `the-lab-grades-itself` trajectory report flagged three open questions honestly but left them open. The user's response was to ask for a ritual that prevents that drift class.

The ritual — committed to CLAUDE.md alongside this retirement — has three dispositions:

- **Executed.** Run the follow-up, write the report.
- **Retired — superseded_by.** Later work absorbed the question. Name which work and why.
- **Retired — abandoned.** The question is no longer worth answering. Name the rationale.
- **Deferred — with a dated target.** Genuinely still live but blocked. Name the blocker and window.

This report and the registry edit together demonstrate the shape the ritual takes: a small written retirement note, not a ceremony. One frontmatter field on the experiments entry, one brief report under `reports/`, one commit with a message that makes the disposition legible. The point is WRITTEN-OUT-NESS, not formality.

The session-arc retrospective's implicit redirection on 2026-04-23 was probably correct — the lab needed building more than the follow-up needed executing — but it was never declared. Today it is.

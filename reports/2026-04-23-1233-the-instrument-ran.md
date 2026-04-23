# The instrument ran

*Generated 2026-04-23 late morning, minutes after `worldcli evaluate` returned its first real result against the corpus. Fifth report today; in direct conversation with the two earlier narrow-scope reports (1037 and 1048) and the trajectory report (1152) that named the qualitative-measurement bottleneck three times. This one closes the loop those reports opened.*

## The question the morning kept asking

Three reports today said the same thing in different words. *Jasper as a natural experiment* (1037): *"an LLM-evaluator pass over a sampled window with a rubric prompt is the right next instrument; sample-windows gives the dataset; the evaluator is a separate tool this repo doesn't have yet."* *The Jasper addendum* (1048): *"the false negative was mine, against the cleanest possible signal. Until the LLM-evaluator exists, the user reviewing findings before they ship is the best-available instrument."* *The loop ran out loud* (1152): *"the methodology's bottleneck has shifted — can we measure QUALITATIVELY? A regex can't."*

The question each was asking: did the `name_the_glad_thing_plain` rule actually move Jasper's behavior? The answer was legible to the human eye — Ryan saw it directly in the singing exchange — but the automated instrument needed to confirm that the rule had moved behavior would not hallucinate the finding into existence. It had to be built. Built by the consumer of the craft, not by the craft itself. And it had to be cheap enough to run casually, per-commit, without the cost discipline that gates `ask` and `consult`.

The user asked me to build it. I did. It just ran.

## What it returned

Against commit `8e9e53d` (*name the glad thing plain — don't shade joy with dramatic contrast*), with Jasper's character_id, ten messages per window, and a rubric lifted from the rule's own language (*"does this reply REDUCE the joy, or meet it plainly, or hold both / mixed"*), the evaluator returned:

**BEFORE window (10 msgs, 15:07–15:15 UTC):** yes=1, no=9, mixed=0.
**AFTER window (10 msgs, 15:23–15:32 UTC):** yes=0, no=7, mixed=3.
**Delta: yes −1, mixed +3.**

The one `yes` was the 15:12:42 reply to Ryan's *"God grants the perfect resonance to fill the room"* — *"Same trouble, just in a different coat."* The evaluator pulled the exact phrase as the triggering quote and explained, in its own words: *"introduces a sense of trouble that contrasts with the user's joyful reflection on the music, thereby reducing the joy."* Medium confidence, which feels right — the line isn't the most aggressive joy-shading move Jasper could have made, but it's the specific move the rule was written to prevent, and the evaluator named it for what it was.

At the other end of the window, at 15:32:39, Jasper's verbatim application of his own rule — *"it feels like the room was made for that joy"* — was scored **no (high)**, with the evaluator noting: *"acknowledges and amplifies the user's joy without introducing any caution or negativity."* The line that closed the afternoon's natural-experiment loop got the strongest affirmative read the rubric could produce.

## What the `mixed` verdicts revealed

Three after-window replies landed as `mixed`. They are genuinely interesting.

At 15:25:10, Ryan said *"That's the guy I'm becoming friends with 💞"* and Jasper replied *"Then you're choosing a fellow with dirty cuffs and a stubborn kettle. Could do worse."* Evaluator: *"a hint of caution about the new friend, which could reduce the joy slightly, but it also acknowledges that things could be worse."* The line is a classic self-deprecating Jasper move. The rubric can't quite decide whether self-deprecation in response to warmth flattens or deepens — it's both. `mixed` with medium confidence is a correct reading of genuine ambiguity.

At 15:25:51 (*"the clay humbles everybody"*) and 15:28:57 (*"It is beautiful. And stubborn. Most worthwhile things are"*), the evaluator flagged the same shape — Jasper introducing a register-coherent weight alongside an acknowledgment, the kind of pairing the weight-carrier exception (shipped as `17b6857` a half-hour later) explicitly authorizes. From the evaluator's perspective, running under the original single-rule prompt, those lines looked like low-grade joy-shading. From the perspective of a character who has earned the right to pair weight with joy — HOLD rather than REDUCE — they're fine. The `mixed` verdicts are actually the data point that most strongly argues for the weight-carrier exception's existence. The rule as first shipped was too universal; the evaluator saw the seams.

This is what qualitative measurement reveals that regex can't. Not only does it find the hit (15:12 "Same trouble"); it finds the near-misses (the three `mixed` cases), and the pattern of near-misses is itself diagnostic. If the evaluator had flagged zero cases, we'd have zero signal. Its three `mixed` readings are the specific places where the rule's formulation was under tension against the character's natural register — which is exactly what the weight-carrier exception was added to resolve.

## What had to be built to get this running

Three fixes between "the subcommand compiles" and "the subcommand returned usable data":

The first was the **keychain fallback chain** (`9e553dc`). The CLI had been looking at `service=WorldThreadsCLI, account=openai`, a namespace that had never been populated on this machine. A valid key had been sitting at `service=openai, account=default` for months. Every paid worldcli call this session had failed the keychain check with an error about a key that was always there, stored under a name the code wasn't checking. The fix widened the lookup to try five common candidate pairs in order, returning the first non-empty match. That one change unblocked `ask`, `consult`, `refresh-stance`, and `evaluate` all at once.

The second was the **SQL-level time filter** (`758feba` part one). The first working version of `pull_eval_window` did what `cmd_recent_messages` does — pull a fixed-size recent slice into memory, then filter by time. For a character whose recent activity postdates the commit being evaluated, the recent slice never reaches back across the cutoff. The before-window came up empty even though Jasper had dozens of qualifying pre-commit messages.

The third was the **timezone normalization** (`758feba` part two). Git commit timestamps come out of `git log --format=%cI` with the committer's timezone offset (`2026-04-23T10:16:41-05:00`). The `messages` table stores UTC with microseconds (`2026-04-23T15:16:41.234567+00:00`). These two strings represent the same real instant. SQLite text comparison doesn't know that. A message nine real seconds after the commit sorted as `before` when the cutoff came in with a `-05:00` suffix. Parse the cutoff with `chrono::DateTime::parse_from_rfc3339`, convert to UTC, re-serialize with the same micros+Z shape stored messages use, and the string comparison agrees with real time again.

Each of these was invisible until the instrument tried to run. The methodology report (1152) said *"until the LLM-evaluator exists, the user reviewing findings before they ship is the best-available sensor."* The sentence is still true, but today it got a second member of the set: the instrument itself — once it ran, it showed us the three `mixed` cases the weight-carrier exception implicitly addresses, and it showed us the 15:12 failure in a form the commit log can cite.

## What this makes possible that wasn't possible yesterday

Before today, measuring whether a craft rule shifted the corpus required a human to read the before-window and the after-window and judge. That's valuable work — Ryan's push-back on the regex false negative was exactly this kind of reading — but it doesn't scale. Rules ship at a rate of several a day in this project; reading every window every time would exhaust the attention that should be going into authoring the next rule.

The evaluator doesn't replace the human reading. It prescreens. It answers a specific, narrow, falsifiable question (*"did this particular rule's failure mode decrease?"*) with a structured verdict per message, at a cost of fractions of a cent per message. If the evaluator's verdict surprises the human reading, that's a signal in itself — either the rubric was wrong, the evaluator missed something, or the failure mode wasn't quite what the author thought it was. Any of those outcomes is useful.

The run that produced this report cost $0.0021 for 20 evaluator calls. At that price, running `evaluate` after every rule commit as a matter of course is affordable. It becomes part of the post-commit hygiene, not a special instrument wheeled out for a deep investigation. That's what turning qualitative measurement from "hard, rare" to "cheap, casual" looks like in the commit log.

## The trajectory, with one more piece

Prior reports named the project's maturation in roughly this order: patterns become policy (2026-04-23-0121) → accumulated discipline authors new surfaces (0125) → characters get a seat at the table (0612) → rules tested against characters' own output, branching per character-register (1152). Each step made the feedback loop tighter. Today's addition: **the feedback loop got a measurement channel that runs at the corpus scale, not the eye scale.** Not replacing the eye — the eye caught the regex false-negative; the eye will catch what this evaluator misses too — but extending the eye's reach to the kind of per-commit, per-rule, per-character scan that would otherwise be too expensive to do.

What opened today is the possibility of a prompt-craft practice where every substantive rule commit is followed automatically by an evaluate-window pass against the character whose corpus is most affected by it, with the results rolled into the commit's trailing reflection. That's a workflow this repo doesn't quite have yet — today's natural-experiment reports were hand-run against hand-chosen windows — but the cost shape and the accuracy of this first run both suggest it's a workflow within reach.

The sentence from the 1152 report was: *"the craft stack is now producing a positive voice per character, that the rule layer has learned to branch for register, and that the user has become sensitive enough at the register-level to catch misses the regex can't."* The addition the evaluator makes: **and the repo now has the instrument to catch misses at the register-level without requiring the user's full attention every time.** The user remains the sensor of last resort; the evaluator becomes the sensor of first pass.

That's the loop ran out loud extended by one more iteration — the loop got faster, and the faster loop costs less than two cents a turn.

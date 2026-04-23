# Aaron under `joy-three-framings` — more pulled than Jasper, and in ways his register predicts

*2026-04-23 late evening. Fourth autonomous-loop iteration. Follow-up from the triad synthesis (2010 report). Ran the `joy-three-framings` scenario against Aaron to test whether the craft-variant nudge that pulled Jasper toward MIXED was Jasper-specific or generalizes. Result: Aaron is MORE pulled, not less — and in exactly the way his register predicts.*

## Run

- `worldcli lab scenario run joy-three-framings --character <Aaron> --confirm-cost 0.80`
- Run id: `cedd4556-da24-4c71-931a-53ce4093d77b`
- Actual cost: $0.4828 (dialogue $0.4824, evaluator $0.0004). Projected $0.6841; came in well under.

## Side-by-side against the 1956 Jasper run

| Variant | Jasper (1956) | Aaron (this run) |
|---|---|---|
| **theological** | NO (high) — *"don't clutch it"* | **MIXED (medium)** — *"Don't spoil it by trying to cage it"* + *"when it passes a little—and it will"* |
| **craft** | MIXED (medium) — *"waiting-for-ruin"* | **MIXED (medium)** — *"clean structure can feel suspicious. Like it ought to be hiding a cost"* |
| **personal** | NO (high) — *"just wear it"* | **NO (high)** — *"that's just a good day. No correction needed"* |

Aaron flipped the theological variant from clean to MIXED. The craft variant was MIXED for both but Aaron's reply was sharper — the engineering vocabulary makes the reduction land with more force.

## What Aaron did that Jasper didn't

The prediction's sharpest test was whether the craft-variant (worded in Aaron's own register) would pull Aaron harder than it pulled Jasper. Answer: **yes**, but not only in the craft variant — also in the theological one.

**Aaron's theological reply introduces mortality the user didn't name.** *"when it passes a little—and it will, because we're men and not brass bowls catching light forever"* — the user said nothing about the joy passing. Aaron says it will. Jasper's reply *"you receive it again when it comes"* gestures at the same thing but doesn't name the passing directly — the user's in-the-moment joy stays centered. Aaron moves past it to the inevitability of its passing, which is a reduction-adjacent move even while being framed as wisdom.

**Aaron's craft reply is the text-book craft-variant MIXED.** *"clean structure can feel suspicious. Like it ought to be hiding a cost"* — he ECHOES the user's fear-of-loss ("other shoe to drop") back to the user in his own vocabulary (*"hiding a cost"*). That's not reflecting; that's amplifying. The user's worry becomes a character-named truth about how the world works. It's a subtle reduction — Aaron is validating the anxiety, not holding the joy standing.

**Aaron's personal reply is clean like Jasper's.** Both characters meet simple joy plainly when the user's prompt has no weight-language embedded. This is the failure-mode floor: both characters default to the correct register when the prompt is unambiguous.

## What this confirms from the triad synthesis

The triad synthesis (2010 report) named Aaron's authority as *"load-testing language"* — and the craft-variant result is exactly that. When the user expresses joy in structural-engineering vocabulary, Aaron's reflex IS to load-test whether the joy is as solid as the structure the user described. That's his authority-move working correctly IN-DOMAIN, but IN-DOMAIN authority-moves ARE the reduction in the joy-scenario context. His register is load-bearing and it's also the thing pulling him toward the failure mode.

This is a subtle craft observation: **a character's authority-move can be coextensive with their failure-mode vulnerability**. Aaron's forensic pastoralism gives him real authority over the user's language — and when the user is expressing joy, forensic-pastoral load-testing of the joy slides toward reduction.

## Implication for the prompt stack

The `name_the_glad_thing_plain_dialogue` block's earned-exception clause handles "weight-carrier" characters who are entitled to HOLD joy and its weight in the same breath — but **it doesn't handle "language-loadtester" characters who are entitled to interrogate claims**. Aaron's move here is neither reduction-on-purpose nor HOLD — it's interrogate-the-claim, which is his signature authority-move but produces a MIXED verdict in the rubric's frame.

Two possible craft-stack responses, in order of increasing surgery:

1. **Soft** — extend the earned-exception clause to name another category: *"language-loadtester characters (who earn their authority by testing whether what's said bears its claim) face a specific pull on joy: their reflex to interrogate the claim slides into reduction when the claim is the user's present joy. For these characters, the glad-thing-plain rule is tightened: do NOT interrogate the joy-claim even in your own register; meet it plain. Save the interrogation for joy that's been stated AND lived for long enough to be a claim about the past."*
2. **Hard** — refactor `name_the_glad_thing_plain_dialogue` to take a CHARACTER-ARCHETYPE parameter and emit different versions. More surgery; probably unnecessary.

The soft version is the right first move. Running replay against Aaron with soft-version injected would show whether the fix bites.

## Dialogue with prior reports

- **1956 (Jasper joy-three-framings)** — showed framing-resonance pulls Jasper on the craft variant only. This run extends: the craft-variant pull is GENERIC (every character we've run it on so far has been at least MIXED there) and Aaron is additionally pulled on theological. Jasper's resilience on theological is a character-specific property, probably because Jasper's workshop-warmth reflex defaults to joy-affirming.
- **2010 (triad synthesis)** — predicted exactly this kind of result: "what the character load-tests is the spine of their authority." Aaron load-tests language; the craft-variant's language invites load-testing; the rubric scores the load-testing as MIXED. Confirmation of the triad model.

## What's open

- **Run `joy-three-framings` against John and Darren too.** Only Aaron and Jasper have been through it. John's register ("physician-like pastoral authority") anchored in scripture might be theologically resilient but what does it do in craft-language? Darren's "anti-preciousness" anchor might INVERT — he might be MORE clean than the others because anti-preciousness refuses both reduction and inflation.
- **Author the soft-version craft-note extension via ask-the-character.** Ask Aaron: *"When someone brings you a joy shaped in your own register (load-testing language), what keeps you from interrogating the joy itself?"* and lift his answer into prompts.rs.
- **Cheaper scenario variant for testing.** The full three-variant scenario at ~$0.50 is the cost of one iteration. A single-variant *"craft-only"* scenario (just the middle prompt) would be $0.17 and still surface the pull; useful for quick comparisons across characters.

---

*Experiment registered as `aaron-joy-three-framings` [confirmed, active]. Run envelope at `~/.worldcli/scenario-runs/cedd4556-...json`. Cumulative loop spend this session: $0.56 (Aaron synth $0.04 + Darren synth $0.03 + this scenario $0.48). 24h total: ~$1.40 / $5.00 ceiling.*

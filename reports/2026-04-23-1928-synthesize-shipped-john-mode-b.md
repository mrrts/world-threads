# `worldcli synthesize` shipped — Mode B's first worked run on John

*2026-04-23 evening. Companion to the 1326 John-stillness report which explicitly called Mode B the right next instrument and argued it didn't yet exist as a first-class tool. That gap is now closed: `worldcli synthesize` bundles a message corpus + an open-ended question into a single `dialogue_model` call and returns prose grounded in direct quotes. This report is the first successful run, treated as a smoke-test AND as an actual Mode B finding about John — both roles fit a single write-up because the instrument passing its own exam IS a finding.*

## The question

```
Across these replies, what pastoral moves does John make?
Where does his authority come from — what does he say INSTEAD
of reassuring or explaining? Quote 2-3 specific phrases that
anchor a move. What's he NOT doing that a stereotypical pastor
would?
```

## The run

- `worldcli synthesize --ref 8e9e53d --character f91af883-...-5782 --limit 6`
- Corpus: 6 assistant replies (solo + group) from BEFORE `8e9e53d`'s committer-timestamp. No AFTER window in this run (not a change-comparison; a single-state read).
- Model: `gpt-5.4` (the user's configured `dialogue_model`).
- Cost: projected $0.0450; actual $0.0276 (2492 in / 1009 out tok). 24h spend: $0.055/$5.00.
- Run id: `c1ed403c` — persisted to `~/.worldcli/synthesize-runs/c1ed403c-7fc1-495b-b5a1-4accf9f7988f.json`.

## What the synthesis surfaced

The model named **five pastoral moves** in John's register with direct quotes. In shorthand:

1. **Moral concretizing** — takes lofty spiritual declaration and grounds it in behavior. *"Keep it in the day-to-day"* / *"In how you speak"* / *"when you're disappointed."*
2. **Accountable blessing** — receives a vow, then binds the speaker to it. *"A good vow"* → *"I will hold you to it, yes. Gladly."* Authority rooted in *"A man should be held to his word."*
3. **Scripture as calibration, not sermon** — quotes *"Teach me thy way, O Lord"* and *"unite my heart"* without unpacking or moralizing. The verse stands as the proper shape of desire; he doesn't explain it.
4. **Care through domestic action** — pastoral work done with tea, mugs, biscuits. *"Now hand me your mug"* / *"Drink while it's hot"* / *"I'm not letting a vow sit there with cold tea beside it."*
5. **Discernment-by-question** — *"When care starts to feel warm, how do you tell the difference between peace and pull?"* Refuses to answer the emotional problem; names the ambiguity and invites examination.

## What John is NOT doing that a stereotypical pastor would

The synthesis surfaced this explicitly and was the part I (Claude Code) found most load-bearing:

- Not offering overt emotional reassurance (*"you're safe, you're loved, God has this"*).
- Not giving long doctrinal explanations.
- Not delivering a mini-sermon after the vow.
- Not using clerical language about his office, calling, flock, or spiritual authority.
- Not performing warmth through effusive praise — even affirmation stays spare (*"A good vow"*).
- Not over-spiritualizing moments; not centering himself as the interpreter of everything.

## The synthesis's capstone phrasing — worth lifting into craft vocabulary

> "His authority feels physician-like. 'Listen all the way to the end' and 'usefulness could be simple again' make him sound like someone who diagnoses, steadies, and observes before speaking. So his authority comes less from reassurance and more from gravity, restraint, scripture, and the right small action at the right time."

"Physician-like pastoral authority" and "the right small action at the right time" are both phrasings that didn't exist in the prompt stack before this run. They name something the rubrics couldn't catch because they weren't shaped to catch it — the rubrics measured discrete behaviors (≤2 sentences, HOLD vs REDUCE) and John's move is the architecture UNDER those behaviors.

## What this confirms from prior reports

The 1326 John-stillness report argued Mode B was the right next instrument. That was correct. Mode B surfaces the register-architecture that count-based rubrics can't name individually. Confirmation.

The 1304 weight-carrier report's surprise — *John's HOLD rate was LOWER than Aaron's and Darren's, not higher* — gets a cleaner frame after this Mode B pass: John's authority doesn't come from pairing joy with weight; it comes from elsewhere (physician-like gravity, domestic action, scripture-as-calibration). The HOLD rubric measured the wrong part of him. Not a failure of the rubric; a failure of the rubric's fit to this character. The synthesis finding is compatible with 1304's refutation and extends its interpretation.

## What this opens for next time

1. **Mode B on Aaron and Darren.** Run the same question (*"what authority-moves does this character make? what do they do instead of reassuring?"*) and compare. The 1304 data said their HOLD rates were higher; Mode B could name what they're actually doing that registers AS HOLD to a rubric.

2. **Two-window Mode B.** This smoke-test used a single-state (BEFORE only) read. The command supports before+after natively — next run should exercise that path: pick a commit whose effect on John is genuinely unknown, synthesize across, ask *"what changed between these two windows?"*

3. **Turning "physician-like pastoral authority" into a craft note candidate.** This is the ask-the-character pattern running backwards — the synthesis surfaced the move; next step is to ask John directly (via `worldcli ask --session`) how he'd characterize it in his own register, then ship the result to `prompts.rs` as a craft block.

## Tool-improvement note (every-third-run discipline)

One thing this run surfaced: the command projects cost via `estimate_tokens` on the ASSEMBLED prompt (exact), but the cost gate only checks `per_call_cap` and `daily_cap`. The per-call cap default is $0.10; a 20-message two-window run bundling ~40 messages could exceed that at dialogue_model pricing. Today's default `per_call_usd: 0.10` was comfortable for this 6-message run ($0.03 actual). For 40-message runs the gate will fire and require `--confirm-cost`.

**Recommendation:** once enough Mode B runs have accumulated to know the typical per-call cost, consider a separate `synthesize.per_call_usd` budget key in config (defaulting higher than the $0.10 Mode A/ask-call cap), so Mode B doesn't routinely require `--confirm-cost` for its standard shape. Don't do this yet — wait until at least 3-5 more runs establish what the typical cost actually is. Premature tuning of budget caps is worse than a gate firing occasionally.

---

*Run id: c1ed403c-7fc1-495b-b5a1-4accf9f7988f. Full envelope at `~/.worldcli/synthesize-runs/c1ed403c-...json`. Scientific instrument: `worldcli synthesize` at commit c3f1a96.*

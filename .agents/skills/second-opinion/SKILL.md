# second-opinion

## Objective

To purchase high-value reads at runtime and cut wait times.

---

Reach outside the substrate to ChatGPT when an outside-LLM read would tighten what you're about to do — to resolve genuine reasoning ambiguity, sanity-check a load-bearing artifact, get color/texture the substrate doesn't carry, fire an experimental prompt to see what surfaces, interpret another developer's intent from partial signals, or skip a long internal reasoning moment by paying for its answer instead.

Two invocation paths share the same daily budget: **`codex exec`** (one-shot or dialogue-continuation, repo-aware) for code/repo-shaped consults, and **direct ChatGPT API call** (curl against `https://api.openai.com/v1/chat/completions` using the keychain key) for everything else. This skill names WHEN to reach for the consult and HOW to ship context so the outside read is faithful — it does not duplicate the CLI or the API.

## When this skill fits — six shapes

The instrument is the same (an outside-LLM call billed against the daily budget); the shapes you reach for it in are distinct. Naming them keeps the bar honest — each shape carries its own test for whether the consult is genuinely warranted vs. a comfort-reach.

1. **Reasoning second-opinion.** You can name two reads that both seem load-bearing and you can't decide between them from inside your own context. Outside perspective tightens the call. Use `codex exec`; load the repo-context that's load-bearing.

2. **Architectural / load-bearing-artifact sanity-check.** A non-trivial design move (new schema, formula reauthoring, methodology codification) where outside reading would catch what you'd miss from the inside. Use `codex exec` with the cross-LLM-consultation preface (`feedback_cross_llm_consultation_preface.md`). Worked example: the 2026-04-26 ChatGPT-reads-MISSION-FORMULA thread produced *"the constraints are why it works, not the symbols"* — a read no inside-the-substrate analysis would have arrived at.

3. **Color / texture / general-knowledge.** You need richer reads on a real-world surface, character interior, scene furniture, vocabulary check, or domain reference the project's substrate doesn't carry. The repo isn't relevant; you need ChatGPT's general-knowledge surface. Use the **direct API call**. Example: *"What does a small-town funeral home in coastal Maine in October smell, sound, and look like in the half-hour before a service? Three sensory details that aren't cliché."*

4. **Experimental / no-question prompts.** You don't have a specific question; you want to fire something at ChatGPT just to see what comes back. Free play. Useful when the substrate feels stuck, when you want to see how a different LLM frames X, or when an unexpected read would unlock something you can't articulate yet. Use either invocation path depending on whether repo-context matters. Disclose as exploratory (no preset success criterion) so the user weights the response accordingly.

5. **Efficiency offload.** A reasoning moment in the current session is shaping up long (multiple turns, lots of reading, branching) and you can pay $X to buy the answer via one or a few targeted calls instead. The trade is budget-money for token-time and cognitive-load. Most appropriate when the question has a recognizable shape ChatGPT can answer cleanly, when the cost-to-buy is small relative to the expected internal reasoning, and when getting back to the user faster genuinely matters. Disclose the trade explicitly: *"This was shaping up to be a long internal pass; I bought it via ChatGPT for $X.XX."*

6. **Interpreting fork-developer signals.** A future developer is working on their own WorldThreads fork in their own Codex session, and you're reading what they're going for from snippets, prompts, or partial signals the user has surfaced. ChatGPT can play the role of "another reader of the same signals" and surface alternate interpretations of intent that the user can weigh against your own read. Use `codex exec` when the signals include code/prompts; direct API when they're conversational or design-prose without repo dependency.

**Dialogue continuation** is orthogonal to all six — when the question shape needs 2-3 exchanges with a sounding board, use `codex exec resume --last` for codex-side, or maintain the `messages` array yourself for direct API.

## When this skill DOES NOT fit

- **You already know the answer.** Don't reach for outside validation just because you want comfort. The instrument's value is in catching what you can't see, not in confirming what you already see.
- **The question is purely tactical / project-substrate-bound.** "What's the path to this file" / "what does this commit do" / "does this build" — answer from your own context, not from an outside model that doesn't have it.
- **You're stuck because of insufficient context, not insufficient perspective.** Read the substrate first; consult after.
- **The user is in flow and asking the question is interrupting.** When in doubt, do the reasoning yourself and offer to consult IF they want — don't auto-fire on every uncertainty.

## Two invocation paths

Both share the daily $5 budget; pick by whether the consult needs repo context.

### Path A — `codex exec` (repo-aware)

```bash
# One-shot:
codex exec "<your prompt with the consultation preface and the artifact context>"

# Dialogue continuation (within the same codex session):
codex exec resume --last "<follow-up>"
```

The CLI returns the response inline. Cost depends on the model configured in `~/.codex/config.toml` — typically gpt-5 or gpt-4o; substantive consults usually land $0.05–$0.30, dialogue continuations more (each turn appends prior context). Reach for this when the consult needs to see code, the repo's prompts, or the project's substrate.

### Path B — direct ChatGPT API (no repo context needed)

```bash
# One-shot — fetch key from keychain, pipe through curl:
KEY=$(security find-generic-password -s openai -a default -w 2>/dev/null)
curl -sS https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [
      {"role": "system", "content": "<optional persona/framing>"},
      {"role": "user", "content": "<your prompt>"}
    ]
  }' | jq -r '.choices[0].message.content'
```

Reach for this when the consult is general-knowledge, color, texture, vocabulary, or any read that doesn't benefit from repo context. Cost: gpt-4o chat completions land roughly $0.01–$0.05 for short prompts, $0.05–$0.15 for longer ones. For pricier reasoning, swap `"model": "gpt-4o"` to `"gpt-5"` (when available; check `~/.codex/config.toml` for what the user has access to). Project conservatively and append actual cost to daily-state same as codex calls.

For multi-turn dialogue via direct API, maintain the `messages` array yourself — append assistant replies as you receive them and re-send the full history on each next call.

## Method

### Step 0 — Decide if this genuinely warrants outside perspective

Try to name in one sentence what the consult IS for — which of the six shapes above, and why your own context can't resolve it cleanly. If you can't name it, the question is probably either too ill-defined for an outside read OR resolvable by reading more substrate first. Don't reach for the consult to fill in for unfinished thinking.

For shapes 4 (experimental) and 5 (efficiency offload) the bar relaxes — *"see what comes back"* and *"buy back the time"* are valid framings — but disclose the framing so the user weights the response accordingly.

### Step 1 — Compose the prompt with cross-LLM-consultation hygiene (when artifacts are involved)

When the consult ships project artifacts (MISSION FORMULA, derivations, prompts, doctrine blocks), per the discipline shipped to memory (`feedback_cross_llm_consultation_preface.md`) and codified in `docs/PLAIN-TRADITION.md` (Cross-LLM consultation hygiene subsection): prepend ONE LINE specifying how the bytes are consumed at runtime, not what they mean.

The reusable minimal preface:

> *The following blocks are concatenated and injected as the system-prompt prefix on every LLM call. They operate as a single conditioning frame, not as separate artifacts.*

For partial-payload pastes (just one derivation, just one rule), name what the FULL payload is and where the partial fits in. For Backstage-Consultant-context shared cross-LLM, name that the persona-shapes block + the derivation references all get conditioned-on together.

For shapes 3, 4, 5 (color, experimental, efficiency offload) the preface usually doesn't apply — there's no project-artifact substrate being consumed. State the question as plainly as you can. Specific enough that an outside reader can answer; broad enough that the answer can surprise you usefully.

### Step 2 — Invoke (Path A or Path B per the question's shape)

See "Two invocation paths" above for the actual commands. Pick by whether the consult needs repo context: `codex exec` for code/repo-shaped (shapes 1, 2, 6 typically), direct API for general-knowledge / color / experimental / efficiency offload (shapes 3, 4, 5 typically). Both share the daily budget.

### Step 3 — Read the response honestly

- If the consult tightened your reasoning: act on it. Cite the consult-source if shipping the resolution.
- If the consult made the same mistake you were worried about: trust your own read and name what the consult missed (often it's deployment-context — see the cross-LLM-preface memory entry).
- If the consult opened a third option you hadn't considered: surface it as a chooser-option to the user; don't auto-act on outside-LLM proposals without user authorization.
- **Disclose the consult.** Tell the user *"I consulted ChatGPT via {codex exec | direct API} — its read was X (cost ~$Y.YY, today's spend now $Z.ZZ of $5.00)"* so they can weight it. Name which path you used and which shape (1–6) the consult was. The discipline is the same as the worldcli-spend-disclosure pattern.

### Step 4 — Capture in memory if methodology-shaped

If the consult produced a generalizable methodology insight (not just a tactical answer), capture it:

- For the project: a memory entry under `.Codex/memory/` (the 2026-04-26 ChatGPT cross-validation produced `feedback_cross_llm_consultation_preface.md`).
- For the practice generally: a paragraph in `docs/PLAIN-TRADITION.md` if appropriate.
- For the moment: an entry in `reports/OBSERVATIONS.md` Mode 3 if the moment was striking.

The cross-LLM bridge often produces sharper articulations than either model arrives at alone. Worth preserving when it happens.

## Cost discipline + standing daily authorization

**Pre-authorized standing budget: $5.00 USD per real-world day** (based on the system's local-timezone date). Within that budget, this skill is freely invocable without per-call user confirmation. Above the budget, each additional increment requires explicit per-call authorization.

The discipline:

1. **Daily-state file:** maintain `~/.Codex/skills/second-opinion/daily-state.json` with the shape `{ "date": "YYYY-MM-DD", "authorized": true|false, "spent_usd": <float>, "budget_usd": <float>, "calls": [...] }`. Created on first invocation of the day. Reset (overwritten) when the date rolls over.

2. **First-of-day authorization:** when this skill is invoked AND today's `daily-state.json` either does not exist OR has `authorized: false` OR has a `date` field different from today, fire AskUserQuestion ONCE:

   > **Authorize today's ChatGPT consult budget?**
   > Standing budget: $5.00 USD for {today's date}. Within that budget I'll consult freely without asking again. Above $5.00, I'll ask per-increment.
   >
   > Options: **Yes — authorize $5.00 for today (Recommended)** | **No — skip the consult, decide from my own context**

   On Yes: write `daily-state.json` with `authorized: true`, `spent_usd: 0.00`, `budget_usd: 5.00`, today's date. Proceed with the consult. Do NOT ask again during the day for spends within budget.

   On No: abort the consult; record the decision so the rest of the day doesn't re-prompt unless the user explicitly invokes the skill again.

3. **Within-budget invocations:** if today's state is `authorized: true` AND `spent_usd + projected_call_cost <= budget_usd`, proceed with the consult silently (no AskUserQuestion). After the call, append the actual cost (estimated when the CLI doesn't report exact spend) to `spent_usd` and to the `calls` array with timestamp + brief one-line summary.

4. **Above-budget invocations:** if `spent_usd + projected_call_cost > budget_usd`, fire AskUserQuestion for THIS specific increment:

   > **Today's $5.00 budget would be exceeded.**
   > Already spent: ${spent_usd:.2f}. This consult projects ~${projected:.2f}. Authorize this specific call above the cap?
   >
   > Options: **Yes — authorize this $X.XX consult** | **Yes and raise today's cap to $Y.YY** | **No — decide from my own context**

   Each above-cap increment requires a fresh AskUserQuestion. The cap can be raised mid-day via the second option; otherwise it stays at $5.00.

5. **Explicit budget override:** if the user explicitly states a different budget for today (e.g., *"today's budget is $20 for consults"* or *"go ahead and consult freely up to $50 today"*), update `budget_usd` for today to the named amount. Treat the override as the new daily ceiling. The first-of-day authorization is implicit in the override (don't double-prompt).

6. **Cost estimation:** `codex exec` (Path A) typically lands $0.05–$0.30 per substantive consult on gpt-5 (more for very long context); dialogue continuations cost more (each turn appends prior context). Direct API (Path B) on gpt-4o lands roughly $0.01–$0.05 for short prompts and $0.05–$0.15 for longer ones; gpt-5 via Path B costs more. When neither CLI nor API response returns exact cost, estimate conservatively and log both projected and best-guess actual. The direct API does return token counts in `response.usage` — multiply by the model's per-token rates from the OpenAI pricing page for an accurate read.

7. **Disclosure unchanged:** even on within-budget silent invocations, tell the user *"I consulted ChatGPT — its read was X (cost ~$Y.YY, today's spend now $Z.ZZ of $5.00)"* so the spend is visible. Silent in the sense of NOT asking permission; not silent in the sense of hiding the fact.

The bar for invocation remains *"would this question genuinely reshape what I do next?"* — the standing authorization removes the per-call permission friction, NOT the per-call judgment about whether the consult is worth running. Don't burn through the daily $5 on trivial sanity-checks just because permission is implicit.

When this skill is invoked from inside a deeper task (e.g., during another skill's execution), the daily-state check applies once at the consult moment, not once at the outer task's start. Multiple consults within one outer task all bill against the same daily state.

The standing authorization is the practice's chooser-with-cost-as-authorization pattern applied at the day-scope rather than per-move: one yes/no at the start of each real-world day buys frictionless access to the consult instrument all day, with the cap as the safety surface.

## Worked example

The 2026-04-26 ChatGPT thread (captured in `reports/OBSERVATIONS.md` 11:30 + 11:55):
1. Ryan pasted MISSION FORMULA + derivations into ChatGPT (no project history)
2. ChatGPT noted *"surprise/delight not structurally guaranteed by formula"* — honest at the formula-operator level but missed deployment-context
3. Codex named the missed deployment-context as the load-bearing-coherence prior absent from outside reads
4. Ryan relayed back to ChatGPT
5. ChatGPT generalized into the *"concatenation at system level > composition at semantic level"* principle PLUS the reusable preface

The cross-LLM bridge produced a sharper articulation than either model alone. The discipline that emerged (always include runtime-consumption preface) is now memory + methodology-doc-permanent.

## Pattern summary

```
notice ambiguity → name it in one sentence → compose prompt with
cross-LLM preface → codex exec → read response honestly → capture
in memory if methodology-shaped → disclose consult to user
```

The instrument is small. The discipline is in WHEN to reach for it (genuine reasoning ambiguity, not tactical questions) and HOW to ship context so the outside read is faithful (preface specifying runtime consumption).

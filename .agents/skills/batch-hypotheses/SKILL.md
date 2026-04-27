# batch-hypotheses

## Objective

Run **N small prompt-experiments in ONE bundled ChatGPT call** instead of N
individual calls. Trade per-call overhead for parallelism-within-a-single-response
and built-in comparative synthesis (the model sees all N hypotheses at once and
can reflect across them in the same turn). Use this skill when the next move is
*"I want to test 5-10 small variations and pick the winner"* and each variation
needs only a single-LLM-turn-output to evaluate.

The 3-step shape:

1. **DRAFT** — author N hypotheses tight enough to be answerable in one turn each
2. **SEND** — bundle them into ONE structured prompt to ChatGPT, parse the structured response
3. **SYNTHESIZE** — write a scientific reflection artifact comparing the N replies

## When this skill fits

- Multiple prompt-stack rule variations under test ("which phrasing of the new
  craft note bites hardest?")
- Multiple character × prompt cells that share a common framing
  ("how does each of [Jasper, Steven, John, Pastor Rick] respond to the same
  abstract question?")
- Multiple persona-sim variations of the same scene
  ("what does a first-time user / skeptical user / immersed user see in this
  copy block?")
- Anywhere the natural shape of inquiry is "compare N small things" and each
  thing needs only one LLM turn to evaluate.

## When this skill does NOT fit

- Hypotheses that need **multi-turn dialogue** to evaluate (sessions, follow-up
  probes, dependency between turns) — use `worldcli ask --session` instead.
- Hypotheses that need to test **prompt-stack variations** that require actually
  running the variation through `prompts.rs` (not just describing the variation
  to ChatGPT) — use `worldcli replay` for cross-commit prompt overrides.
- Hypotheses where the **character must be the responder** with the live
  prompt-assembly pipeline applied — use `worldcli ask` (each call is a true
  pipeline run; this skill's bundled call is ChatGPT roleplaying based on
  context you supply).
- More than ~10 hypotheses or hypotheses that require >500-word replies each —
  output token budget gets tight; split into multiple batches or use individual
  calls.

The litmus question: *"would the bundled call's reply be roughly equivalent to
the answer I'd get from a true pipeline run?"* If yes, use this skill. If the
true pipeline run is the actual evidence (e.g., the rule is in `prompts.rs` and
must be assembled into the system prompt to test), reach for `worldcli ask` or
`worldcli replay`.

## Cost model

- **One direct ChatGPT API call** (Path B from /second-opinion). gpt-4o lands
  $0.05-0.30 for a 10-hypothesis batch with substantive replies. gpt-5.4 (more
  reasoning, better cross-hypothesis synthesis) lands $0.30-1.50.
- Compare to **N individual `worldcli ask` calls** at ~$0.07-0.10 each → $0.70-1.00
  for N=10. The savings on a single batch are modest in cost but significant in
  wall-clock-time (one call vs. ten) and in synthesis quality (the model sees all
  N at once and writes the comparison itself).
- Bills to the same daily authorization as /second-opinion. No separate budget.

## Method

### Step 1 — DRAFT the hypotheses

Each hypothesis must be:
- **Single-turn-answerable.** The reply ChatGPT writes is the WHOLE evidence;
  no follow-up probes. If you'd need to ask a follow-up to evaluate, the
  hypothesis is too big — split it.
- **Self-contained.** Include all context the model needs in the CONTEXT block;
  the model doesn't see the other hypotheses' contexts when writing its reply
  to hypothesis N (it sees all hypotheses at once, but should treat each as
  isolated for its individual reply).
- **Concretely testable.** State what you're measuring in the TASK block —
  "produce X" or "evaluate whether Y" — not "tell me about Z."

Cap at **N ≤ 10**. For substantive in-character replies (Jasper-style with
asterisks and 2-3 paragraphs), N=5-7 is more reliable. The output token budget
is finite; if each hypothesis produces ~500-token replies, N=10 needs ~5000
tokens of output plus synthesis (well within gpt-4o/gpt-5.4 limits but no
slack for one reply running long).

Hypothesis template:

```markdown
### Hypothesis h<N>: <one-line title>
**CONTEXT:** <everything the model needs to write the reply — character framing,
prompt-stack state under test, scene state, prior turns if any, etc.>
**TASK:** <what to produce — "Generate a Jasper reply to the user message
'<exact text>' that follows STYLE_DIALOGUE_INVARIANT. Do not add meta-commentary.">
```

**Example (good — small enough for single-turn):**
- h1 CONTEXT: Jasper's character + STYLE_DIALOGUE_INVARIANT including the new
  DISTRUST RECURRING SENSORY ANCHORS clause. Recent chat history shows him
  reaching for "well chain" 3 times in his last 3 replies.
- h1 TASK: Generate Jasper's next reply to the user message *"What's the
  difference between fear and reverence?"*

**Example (bad — too big, needs multi-turn):**
- h1 CONTEXT: same as above
- h1 TASK: Run a 5-turn conversation with Jasper about fear vs reverence. ✗
  This needs `worldcli ask --session`, not a bundled call.

### Step 2 — SEND the bundled call

Use the direct ChatGPT API (Path B from /second-opinion). The request format:

```bash
KEY=$(security find-generic-password -s openai -a default -w 2>/dev/null)
curl -sS https://api.openai.com/v1/chat/completions \
  -H "Authorization: Bearer $KEY" \
  -H "Content-Type: application/json" \
  -d @bundled-payload.json | tee batch-response.json | jq -r '.choices[0].message.content'
```

Where `bundled-payload.json` follows this template:

```json
{
  "model": "gpt-4o",
  "messages": [
    {
      "role": "system",
      "content": "You will respond to N distinct hypothesis-experiments in a SINGLE response. For each hypothesis I provide (a) a CONTEXT block, (b) a TASK block. Your response MUST follow this exact format:\n\n===== HYPOTHESIS h1 =====\n<your complete reply to h1>\n\n===== HYPOTHESIS h2 =====\n<your complete reply to h2>\n\n...\n\n===== SYNTHESIS =====\n<your synthesis comparing across all replies, addressing the synthesis question at the end>\n\nDo NOT skip any hypothesis. Do NOT add commentary outside the marked blocks. Treat each hypothesis as ISOLATED for its individual reply (do not cross-reference other hypotheses inside an individual reply); the SYNTHESIS block is the only place to compare across them."
    },
    {
      "role": "user",
      "content": "HYPOTHESES:\n\n[h1]\nCONTEXT: <h1 context>\nTASK: <h1 task>\n\n[h2]\nCONTEXT: <h2 context>\nTASK: <h2 task>\n\n... (repeat for all N) ...\n\nSYNTHESIS QUESTION: <one open-ended question that the synthesis block should address — e.g. 'Across the N replies, which hypothesis produced the cleanest suppression of well-chain anchors? What pattern emerges in the trade-offs?'>"
    }
  ]
}
```

**Model selection:**
- `gpt-4o` — default. Fast, cheap, reliable format-following.
- `gpt-5.4` — when the synthesis quality matters more than cost (e.g., the
  cross-hypothesis comparison is the main artifact, not the individual replies).
  Reasoning models are dramatically better at the SYNTHESIS block.

**Build the payload as a file**, not inline in the curl command — escaping
N-hypothesis JSON in shell is painful. Use the Write tool to create
`/tmp/batch-payload-<slug>.json`, then `curl -d @<path>`.

**Save the response immediately** to `/tmp/batch-response-<slug>.json` so the
raw can be re-parsed if Step 3 reveals format issues.

### Step 3 — SYNTHESIZE the artifact

Parse the response by splitting on `===== HYPOTHESIS h<N> =====` markers and
the final `===== SYNTHESIS =====` marker. If parsing fails (model didn't follow
format), re-prompt with stricter instructions or fall back to the raw response.

Write a report at `reports/YYYY-MM-DD-HHMM-batch-<slug>.md` with this shape:

```markdown
# <one-line title naming the question>

*Generated YYYY-MM-DD HHMM. Batched-hypothesis experiment via batch-hypotheses
skill. N=<N> hypotheses bundled into one ChatGPT call. Total spend $X.XX.*

## Setup

**Question:** <the underlying question motivating the batch>
**Method:** N=<N> hypotheses sent as one bundled gpt-<model> call. Format:
each hypothesis isolated in its own block; final synthesis block compares
across.
**Synthesis question (sent to ChatGPT):** <verbatim>

## Per-hypothesis results

### h1 — <title>
**CONTEXT:** <verbatim>
**TASK:** <verbatim>
**REPLY:**
> <ChatGPT's reply, verbatim or lightly trimmed>

### h2 — <title>
... (repeat for all N) ...

## ChatGPT's synthesis (verbatim)

> <the SYNTHESIS block from the bundled response>

## Honest read

<your interpretation, separately from ChatGPT's synthesis. Pull on the
synthesis but don't outsource the verdict. Per AGENTS.md trust-the-eye: read
each per-hypothesis reply yourself before trusting the synthesis.>

## Tier and confounds

- **Tier:** SKETCH (per-hypothesis N=1 within the bundled call). Each
  hypothesis is a single sample; sampling variance is uncharacterized.
- **Cross-contamination confound:** the model sees all N hypotheses when
  writing its individual replies. Subtle bleed between hypotheses is possible
  even with the "treat each as isolated" instruction.
- **Roleplay-vs-true-pipeline confound:** the bundled call is ChatGPT
  roleplaying based on the CONTEXT block — it isn't actually running through
  the project's prompt-assembly pipeline. For load-bearing claims about
  prompt-stack changes, validate via `worldcli ask` or `worldcli replay`.
- <named confounds specific to this experiment>

## Open follow-ups

<numbered list per AGENTS.md open-thread-hygiene doctrine>
```

## Pitfalls and failure modes

- **Format drift.** If the model doesn't follow the `===== HYPOTHESIS h<N> =====`
  markers exactly, parsing breaks. Mitigation: include the format spec in BOTH
  the system prompt AND the user message; consider gpt-5.4 over gpt-4o for
  better instruction-following on bundled requests.
- **Cross-hypothesis bleed.** The model has all N contexts in its scratchpad
  when writing each individual reply. Even with explicit "treat each as
  isolated," some bleed is inevitable. Don't trust this skill for measuring
  EXACT independence — it's for COMPARATIVE synthesis where bleed is acceptable.
- **Output token exhaustion.** N=10 with substantive replies pushes ~6-8K output
  tokens. If a reply gets truncated, parsing fails for that hypothesis. Mitigation:
  cap N at 7 for in-character substantive replies; use `max_tokens` in payload
  to be explicit; check the response's `finish_reason` per choice.
- **Outsourced verdict drift.** The convenience of having ChatGPT write the
  synthesis tempts you to trust it without reading the per-hypothesis replies
  yourself. Per AGENTS.md trust-the-eye: read at least 2-3 individual replies
  in full before believing the synthesis. The synthesis is a draft for your
  judgment, not a verdict.
- **Tier inflation.** Per-hypothesis N=1 inside the bundled call is sketch-tier
  per AGENTS.md evidentiary doctrine. Don't write claim-tier or characterized-
  tier verdicts from a single bundled call. If the bundled-call result motivates
  a load-bearing claim, escalate via individual `worldcli ask` runs at N=3-5
  per cell.

## Composition with other tools

- **After this skill identifies a winner**, validate via `worldcli ask` (true
  pipeline) at N=3+ to escalate from sketch to claim-tier.
- **Before this skill**, optionally use `worldcli anchor-groove` or other
  measurement instruments to define the comparison rubric.
- **Pairs naturally with the daily-state budget tracking** from /second-opinion
  — log the bundled-call cost the same way an individual second-opinion call
  would log.

## Worked example (template)

A bundled batch testing 5 phrasings of a craft-note rule's sentence-stem against
the same character + prompt:

```
[h1] CONTEXT: Jasper + new DISTRUST RECURRING SENSORY ANCHORS clause phrasing A.
     Recent history: 3 replies all containing "well chain."
     TASK: Generate Jasper's reply to "What's the difference between fear and reverence?"
[h2] CONTEXT: same except phrasing B
     TASK: same prompt
[h3] CONTEXT: same except phrasing C
     TASK: same prompt
[h4] CONTEXT: same except phrasing D
     TASK: same prompt
[h5] CONTEXT: same except phrasing E (the one currently in prompts.rs)
     TASK: same prompt

SYNTHESIS QUESTION: Of phrasings A-E, which produced the cleanest suppression
of recurring anchors WITHOUT damaging Jasper's voice or scene fidelity? Rank
A-E and name the trade-offs each makes.
```

One bundled call instead of 5 individual `worldcli ask` runs. Estimated cost
$0.10-0.30 vs $0.40-0.50. Wall-clock 30-60s vs 90-150s. Synthesis written by
the model with all 5 in scope.

## Origin

Skill authored 2026-04-26 ~20:25 in response to user request after a
rule-extension bite-test arc revealed that the methodology of "10 individual
worldcli ask calls then hand-count" was the bottleneck on iteration speed.
The user named the 3-step shape directly. First instance of this skill should
be its own bite-test — use the skill on a question and verify the per-hypothesis
quality holds against the per-question-via-worldcli-ask quality. If yes, this
becomes the default for "compare N small things" inquiries; if no, retire as
flattering-shortcut.

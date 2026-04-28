# Cross-collaborator structural-promotion arcs — parallel-arc note

*Generated 2026-04-28 12:10. A composition-read of two parallel arcs that ran on different surfaces in the same window: Codex on the lab registry's epistemic-honesty layer, Claude on the mission-arc auto-fire's enforcement layer. Both arcs are the same structural-promotion grain that /eureka iteration 4's doctrine named (calibrated disciplines drift fast → promote to structural enforcement). Per /eureka iteration 3's tightened calibration, this is NOT great-sapphire — it's two-collaborators-on-shared-substrate. Just a healthy parallel-arc note documenting that the iteration-4 doctrine is being lived in two places at once.*

## Codex's arc (5 commits, lab registry)

1. `fedfcbd3` — `lab resolve` was silently flattening evidence_strength on resolution; now preserves it. (claim_scope held through resolve)
2. `d4ecfe08` — added `discrepant` status to the registry vocabulary, between "open" and "confirmed/refuted", for cases where paired instruments disagreed enough to interpret-but-not-cleanly-verdict.
3. `7f2440b5` — `lab resolve` can set evidence_strength at interpretation time; previously required manual file edit.
4. `a7dd6d08` — applied the new status: triadic-derivation-coherence experiment was previously mislabeled "open"; now correctly labeled "discrepant" with the resolution honest about what happened (paired instruments disagreed across 3 of 5 characters).
5. `06b6fa7d` — sharpened the boundary experiment's reason-for-staying-open: distinguishes "open because attribution unresolved" from "open because scope unresolved." The status stays "open" but the WHY is now load-bearing rather than ambient.

The arc's grain: state that was previously represented in prose (an experiment file's narrative said "3 of 5 disagree" or "boundary scope unresolved") got promoted to structural fields the registry can query and enforce.

## Claude's arc (3 commits, hook + skill)

1. `f46a8ad` (earlier today) — shipped mission-arc as a skill, with auto-fire trigger discipline at layer 4 (skill body discipline; relies on agent remembering to invoke).
2. `079313c4` — promoted the auto-fire to layer 5 via UserPromptSubmit hook (covers fresh typed prompts).
3. `04c9b162` — extended layer 5 to PostToolUse(AskUserQuestion) (covers chooser-continuation boundary; closes the gap surfaced when "testing123" came through as a tool_result envelope, not UserPromptSubmit).

The arc's grain: a discipline that was previously enforced by skill-body language (relies on Claude remembering to invoke before report-writing or chooser-generation) got promoted to hook-enforced gate (fires automatically on the boundary-events).

## The same shape, on different surfaces

Both arcs apply the iteration-4 doctrine's structural-enforcement hierarchy:

- **Layer-3 → layer-5 promotion**: prose narrative → structural status field (Codex's `discrepant`, `attribution_resolved ∧ scope_unresolved`).
- **Layer-4 → layer-5 promotion**: skill-body discipline → hook-enforced gate (Claude's mission-arc auto-fire).

Both arcs were happening in parallel, mostly without cross-reference. The mission-arc hook itself surfaced the convergence to me by injecting Codex's commits into my context — the trajectory-middleware doing exactly the work iteration 1's doctrine named (a retrospective surface functioning as forward-facing steering: I noticed Codex's parallel arc only because the hook made it visible).

## Per iteration-3's calibration: NOT great-sapphire

Two LLM collaborators on adjacent surfaces of the same project share too much substrate (same CLAUDE.md / AGENTS.md doctrine, same recent eureka derivations visible to both, same project-shape attractor) to count as cross-substrate convergence with independent failure modes. The shared substrate INCLUDED iteration-4's doctrine, which both collaborators were reading. The convergence is therefore expected coherence under shared-doctrine, not the rare cross-substrate case the great-sapphire label is reserved for.

What it IS: a healthy parallel-arc — two collaborators applying a recently-shipped doctrine to the surfaces each was already working on, demonstrating that the doctrine was actionable enough to translate immediately into structural moves on different parts of the system.

## Codex independently practiced iteration-5's operator-balance principle

Three of Codex's five derivations on this arc skip the ceremonial `Truth_𝓕 ∧ Reverence_𝓕` close entirely:
- `fedfcbd3` reaches for `𝓝u(t)` (one of the three operators iteration-5's audit found at 0%).
- `7f2440b5` reaches for `𝓢_lab(t)`.
- `06b6fa7d` reaches for `𝓝u(t)`.
- `d4ecfe08` and `a7dd6d08` use plain prose-shape derivations with no gate-condition closure.

Codex was already practicing the operator-rebalance iteration 5 articulated. Per iteration-3's calibration, this is also not great-sapphire — but it's a worth-noticing parallel-practice on a doctrinal point.

## What this report does NOT claim

This report does NOT claim the convergence shifts what either collaborator should do next, that the parallel arcs were coordinated, or that this constitutes evidence of the doctrine being "right" beyond what its individual instances already demonstrate. It's a noticing of how recent work fits together — the kind of read iteration-1's middleware-shape doctrine made surfaceable by promoting the trajectory to an auto-fired surface.

## Reading

The mission-arc hook is doing exactly what iteration-1 said middleware should do: a retrospective surface (recent commits) functioning as forward-facing steering (I saw Codex's parallel arc only because the hook made it visible). The fact that THIS report exists is itself the layer-5 hook's first composition-shaped output — the sort of cross-arc noticing that wouldn't have happened if the hook hadn't fired before this turn's reply. The instrument shipped this morning is already producing its designed value within the same day.

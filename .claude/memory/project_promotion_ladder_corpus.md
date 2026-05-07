---
name: Promotion-ladder corpus — every discipline that has ascended to hook-enforced-gate (layer 5) with its ascent path
description: Crown 16 Ascension's W1+W2 evidence expansion 2026-05-07 — enumerates every discipline currently at hook-enforced-gate tier with origin / ascent path / current enforcement / drift-catch events. Forward-pointing reference for future Apparatus-Honest Sapphire candidacies + audit work.
type: project
---

The project's calibrated-disciplines-drift-fast hierarchy has 5 tiers: (1) doctrine paragraph → (2) memory entry → (3) skill body discipline → (4) auto-fire trigger → (5) hook-enforced gate. This corpus enumerates every discipline currently at tier 5 with its origin + ascent path + drift-catch evidence. Layer-5 enforcement is the apparatus's most structurally-load-bearing form; the layer that operates at runtime regardless of author attention.

## Layer-5 disciplines (hook-enforced gates)

### 1. AskUserQuestion-every-turn law

**Origin:** `feedback_choosers_via_askuserquestion.md` — strengthened project law that by default every assistant turn ends with an AskUserQuestion chooser; fixed 4-option fallback.

**Ascent path:**
- Layer 1 (doctrine): "Choosers are control surfaces" CLAUDE.md section + skill-routing bottom + every-turn-AskUserQuestion law
- Layer 2 (memory): `feedback_choosers_via_askuserquestion.md`
- Layer 5 (hook): `.claude/hooks/check-inline-choosers.py` (Stop hook) — turn must end with AskUserQuestion or block

**Drift-catch evidence:** preserved every-turn-chooser shape across 30+ turns this session and many prior sessions; the hook's standing presence is what holds the discipline against the gravitational pull toward inline-numbered or pure-prose endings that multi-turn flows naturally drift toward.

### 2. No-nanny-register law

**Origin:** `feedback_no_nanny_register_for_self.md` — Ryan's directive that Claude Code does not track session length, recommend breaks, or surface stop/sleep/end-here chooser options unless explicitly asked.

**Ascent path:**
- Layer 1 (doctrine): "No nanny-register from Claude Code itself" CLAUDE.md section
- Layer 2 (memory): `feedback_no_nanny_register_for_self.md`
- Layer 5 (hook): `.claude/hooks/check-no-nanny-chooser.py` (PreToolUse on AskUserQuestion) — block stamina-management/end-session/quit-shaped chooser phrasing

**Drift-catch evidence:** 2026-05-07 ~20:01 blocked "End the night — close arc; chooser turn ends here" chooser option label; required re-emit with neutral framing. Crown 16 Ascension's Event B drift-catch event.

### 3. Skill-parity discipline

**Origin:** "Parallel surfaces — solo/group chat AND Claude/Codex AND public funnel" CLAUDE.md section — collaborator-mirror skill files must mirror across `.claude/skills/` and `.agents/skills/` unless explicitly classified as collaborator-specific.

**Ascent path:**
- Layer 1 (doctrine): "Parallel surfaces" + "Three project-scale parities" sections in CLAUDE.md
- Layer 2 (memory): `feedback_skill_parity_hook_is_load_bearing.md` (lifted 2026-05-08 after parity hook caught one-sided drift)
- Layer 5 (hook): `scripts/check-skill-parity.sh` (pre-commit hook) — one-sided edits to mirror-class skills block

**Drift-catch evidence:** 2026-05-07 ~05:40 morning auto-commit Move 6 — Claude Code edited only `.claude/skills/auto-commit/SKILL.md`; pre-commit blocked with "one-sided skill drift for auto-commit: changed only under .claude"; mirrored to `.agents/` and re-committed. Crown 16 Ascension's Event A drift-catch event.

### 4. Skill-frontmatter validation

**Origin:** Skill metadata schema requirements (each skill must have valid frontmatter declaring name + description + activation pattern).

**Ascent path:**
- Layer 1 (doctrine): implicit in skill-routing system
- Layer 5 (hook): `scripts/check-skill-frontmatter.sh` (pre-commit hook) — invalid skill frontmatter blocks

**Drift-catch evidence:** silent enforcement; "skill-frontmatter | ok | checked=25" status line on every commit indicates 25 skills validated each commit.

### 5. Mission-arc auto-fire

**Origin:** "Trajectory-reading surfaces" CLAUDE.md section — `/mission-arc` auto-fires before reports/chooser generation; trajectory-conditioning is precomposition middleware.

**Ascent path:**
- Layer 1 (doctrine): "Trajectory-reading surfaces" + "Three-layer encoding for methodological corrections" CLAUDE.md sections
- Layer 4 (auto-fire): originally lived in skill body
- Layer 5 (hook): `.claude/hooks/inject-mission-arc.py` (UserPromptSubmit + PostToolUse) — recent-commit Formula derivations + Glosses injected as system-reminder context

**Drift-catch evidence:** visible in trajectory headers throughout this session — every chooser composition was conditioned by recent-commit context; without enforcement, choosers would propose options recently-shipped commits already accomplished. Crown 16 Ascension's Event D drift-catch event.

### 6. /play HUD-print discipline

**Origin:** /play skill body — every /play turn must print HUD box at top first.

**Ascent path:**
- Layer 1 (doctrine): /play SKILL.md
- Layer 2 (memory): `feedback_play_hud_non_negotiable_every_turn.md`
- Layer 5 (hook): `.claude/hooks/check-play-hud-present.py` (Stop hook) — /play turns without HUD box block

### 7. /play AskUserQuestion-required discipline

**Origin:** /play skill body subset specialization of every-turn-AskUserQuestion law.

**Ascent path:**
- Layer 1 (doctrine): /play SKILL.md
- Layer 5 (hook): `.claude/hooks/check-play-askquestion-required.py` (Stop hook)

### 8. /play chooser format discipline

**Origin:** /play skill body — chooser cardinality 4; specific format requirements.

**Ascent path:**
- Layer 1 (doctrine): /play SKILL.md + "Choosers are control surfaces" CLAUDE.md section
- Layer 5 (hook): `.claude/hooks/check-play-chooser-format.py` (PreToolUse on AskUserQuestion)

### 9. /play jewel/crown ledger discipline

**Origin:** /play skill body — earnings must be recorded in play-state ledger.

**Ascent path:**
- Layer 1 (doctrine): /play SKILL.md jewel/crown mechanics
- Layer 5 (hook): `.claude/hooks/check-play-jewel-crown-record.py` (Stop hook)

### 10. Session-arc derivation

**Origin:** Session-arc context-derivation discipline.

**Ascent path:**
- Layer 5 (hook): `.claude/hooks/derive-session-arc.py` (UserPromptSubmit) — derives session-arc context for every user prompt

## Layer-3 disciplines currently at skill-body tier (not yet at hook-enforced)

These are calibrated disciplines that drift-fast doctrine predicts will benefit from layer-5 promotion when warranted by accumulated evidence:

- **Matched-same-substrate-on-deployed-model standard** (Crown 15 Quickener, today) — at skill body in `/seek-sapphire-crown` SKILL.md + `feedback_matched_same_substrate_on_deployed_model.md` memory; layer-5 not warranted (design-decision-shaped not mechanically checkable)
- **Probe-shape-mixing 5th clause** (Eureka iter 1, today) — at skill body adjacent to matched-same-substrate; layer-5 not warranted (same reason)
- **Codex consult as Sapphire-firing-audit step** — at skill body; layer-5 not warranted (cost-gated invocation; mechanical hook would over-fire)
- **Methodology scope-edge for substrate-already-produces** (Crown 15 Atonement HOLD precedent) — at skill body; layer-5 not warranted
- **Build-before-close-out discipline** — at skill body across multiple skills; layer-5 plausible (could be pre-commit hook checking build state); not yet warranted by drift-catch evidence
- **Formula derivation in commit messages** — at skill body + CLAUDE.md doctrine; could be pre-commit hook validating substantive commits include derivation; not yet warranted

## Layer-2 disciplines at memory tier (key entries)

Crown lineage entries (project_*.md):
- project_quickener_fifteenth_sapphire.md
- project_ascension_sixteenth_sapphire.md
- project_anti_drift_register_guard_twelfth_sapphire.md (Crown 11)
- project_imago_dei_refusal_thirteenth_sapphire.md (Crown 13)
- project_trinitarian_fourteenth_sapphire.md (Crown 14)
- project_atonement_claim_tier_held.md (Crown 15 candidacy held)
- project_substrate_already_produces_lineage_research_program.md
- (and others)

Methodological feedback entries (feedback_*.md):
- feedback_matched_same_substrate_on_deployed_model.md
- feedback_probe_shape_mixing_for_substrate_dependent_axes.md
- feedback_shell_substrate_vs_llm_substrate_separability_axis.md
- feedback_codex_consult_discipline_maturation.md
- feedback_apparatus_honest_earns_and_refuses.md
- feedback_no_nanny_register_for_self.md
- feedback_choosers_via_askuserquestion.md
- (and many others)

## Pattern observations

**Hook-enforced disciplines cluster around runtime-truth-telling:**
- 5/10 hooks enforce chooser-or-AskUserQuestion shape (every-turn / no-nanny / format / play-required / inline-banishment)
- 2/10 enforce skill-collaborator-surface integrity (parity / frontmatter)
- 2/10 enforce trajectory-conditioning (mission-arc auto-fire)
- 1/10 enforces /play-specific ledger (jewel/crown)

**Skill-body disciplines cluster around design-decision-shaped guidance:**
- Matched-control standards
- Probe-shape mixing
- Methodology scope-edges
- Codex consult patterns

**The natural promotion path is doctrine → memory → skill body → hook only when:**
- The discipline is mechanically checkable (text-pattern / file-existence / count) NOT design-decision-shaped
- Drift-catch evidence accumulates demonstrating the hook would catch real drift
- The cost of false-positive (over-firing) is lower than the cost of drift (under-enforcement)

Disciplines that shouldn't promote to hook-tier:
- Judgment-shaped (e.g., "is this rubric protecting variety vs locking template" — must be LLM-judged not python-classified)
- Cost-gated (e.g., "codex consult before Sapphire firing" — mechanical hook would over-fire)
- Context-sensitive (e.g., "appropriate level of theological framing" — can't be hook-checked)

## How to apply

When adding a new calibrated discipline, ask the diagnostic from CLAUDE.md "Calibrated disciplines drift fast" section: *"what's the highest structural-enforcement layer this can credibly live at right now?"* Avoid defaulting to doctrine-paragraph when hook or skill-body would do the same job.

When auditing whether an existing discipline has drifted, check this corpus for which tier it's currently at; if drift is observed, promote to next tier rather than re-asserting at current tier.

For Crown 16 Ascension audit + future Apparatus-Honest Sapphire candidacies: this corpus is the W1+W2 expansion that makes the promotion-ladder pattern empirically legible. Future arcs reading this corpus can extend it forward as new disciplines ascend.

## Composes with

- `reports/2026-05-07-2110-ascension-canonical-synthesis.md` (Crown 16 firing artifact; this corpus extends its W1+W2)
- `reports/2026-05-07-2055-ascension-w1-w2-w4-sharpening.md` (the 11-hook-corpus + 4-drift-catch ledger that grounded Crown 16)
- CLAUDE.md "Calibrated disciplines drift fast" doctrine (the 5-tier hierarchy this corpus enumerates)
- `feedback_skill_parity_hook_is_load_bearing.md` (lifted 2026-05-08 after parity-hook caught drift; pattern extends to all layer-5 hooks)

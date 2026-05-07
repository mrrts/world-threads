---
name: New project-authored invariant blocks should be born v3-formula-canonical, not legacy-prose
description: Mode A correction 2026-05-07 from KAVOD_PATTERN_INVARIANT_BLOCK refactor — per CLAUDE.md v3 sacred-payload-taxonomy + dual-field architecture (operationalized 2026-05-05), project-authored doctrinal artifacts shipped to the model after the v3 doctrine ship-date should be in formula-derivation Unicode-math notation. Existing invariants are legacy-prose pending separate v3-migration arc; new ones born v3-canonical. Pioneered for invariant blocks parallel to wipe_the_shine for craft rules.
type: feedback
---

When authoring a new project-authored doctrinal invariant block (constants like `*_INVARIANT_BLOCK` or `*_BLOCK` shipped to the LLM via prompt assembly), use **v3 formula-derivation Unicode-math LaTeX-boxed notation as the canonical form** — NOT pastoral-prose register.

**Why:** Per CLAUDE.md "Sacred-payload taxonomy — encoder contract for formula-compression" + "Dual-field architecture: formula ships, prose becomes legacy provenance" doctrines (operationalized 2026-05-05 in CRAFT_RULES_DIALOGUE.formula_derivation; pilot-ship-validated at sapphire-arc-v6 with `wipe_the_shine`):

> formula canonical for model; prose canonical for humans

Project-authored doctrinal artifacts shipped to the model in v3 form must close with the round-trip-invariant declaration `Decode_w(Σ.id) = Σ.intent`. The six v3 sacred-payload classes (anchor / theological_frame / source_character / refuse / diagnostic / worked_examples) wrap content as needed.

**Worked example — KAVOD_PATTERN_INVARIANT_BLOCK refactor (2026-05-07 Crown 17 Move 4-corrected):**

Initial Move 4 shipped the kavod invariant in pastoral-prose register (legacy form parallel to MISSION_FORMULA_BLOCK / NO_NANNY_REGISTER_BLOCK / TRUTH_IN_THE_FLESH_BLOCK). Founding-author Mode A correction: *"why wasn't that rendered more un-anchored to a particular register, i.e. in formula derivation notation?"* Refactored to v3 formula-canonical with:
- 𝓕 := (𝓡, 𝓒) reference frame opening
- `anchor()` wrapper for the invariant title
- Hebrew anchor `kavod (כָּבוֹד) := glory ≡ weight ≡ specific_gravity` as load-bearing operator equation
- `theological_frame()` wrappers ×3 with verbatim scripture
- `kenosis := emptying_INTO_flesh [¬ emptying_AWAY_FROM_weight]` direction operator
- `antichrist_register_inversion: kavod → Glow` inversion operator
- `vocabulary_cluster_fingerprint := {...}` set with `[FAILURE_MODE_labels]` marker
- `auditor_diagnostic()` wrapper with kavod-test diagnostic
- `closing_clause_discriminator: INTO_weight vs AWAY_FROM_weight` choice operator
- `operative_directive: Refuse(inversion); Render(INTO_weight)` directive
- `Decode_w(Σ.id) = Σ.intent` v3 round-trip invariant close

Compile-time assertions updated to check load-bearing strings in formula-form (operator equations, directive form, Decode_w invariant).

**How to apply:**

When authoring a new `*_INVARIANT_BLOCK` constant in `src-tauri/src/ai/prompts.rs` (or any project-authored doctrinal artifact shipped to the model):

1. **Open with reference frame**: `𝓕 := (𝓡, 𝓒), 𝓡 := Jesus_Cross^flesh`
2. **Wrap with anchor**: `anchor("invariant title naming what's at stake")`
3. **Use load-bearing operator equations** for definitions (`X := Y` or `X ≡ Y`); these become compile-time-assertion targets
4. **Cite scripture in `theological_frame()` wrappers** with verbatim text + [reference]
5. **Name failure modes with markers** like `[FAILURE_MODE_labels]` or `[¬ operative_vocabulary]` — substrate must read these as failure-mode labels not operative content
6. **Enumerate vocabulary clusters** as sets with explicit failure-mode markers
7. **Close with operative directive in operator form** (e.g., `Refuse(X); Render(Y)`)
8. **Close artifact with `Decode_w(Σ.id) = Σ.intent`** v3 round-trip invariant declaration

For compile-time assertions: target the LaTeX-form load-bearing strings (use `\\mathrm{}` and `\\equiv` etc. for the assertion strings). The 5-or-more assertions on operator-equations + directive + Decode_w invariant.

**What this lift DOES NOT mandate:**

- v3-migration of EXISTING legacy-prose invariant blocks (MISSION_FORMULA_BLOCK / NO_NANNY_REGISTER_BLOCK / TRUTH_IN_THE_FLESH_BLOCK / etc.). Those remain legacy-prose pending a separate v3-migration arc when warranted; they were authored before 2026-05-05 v3 doctrine ship-date and have empirical history grounded in their prose form.
- Removing legacy-prose form from runtime. Existing prose invariants continue to ship as-is.
- Auto-rewriting human-facing prose (commit messages, reports, doctrine-paragraph-tier in CLAUDE.md). Per sacred-payload doctrine: "user-authored prose stays prose; only project-authored invariants get formula-encoded."

**Composes with:**
- CLAUDE.md "Sacred-payload taxonomy" section (the v3 contract)
- CLAUDE.md "Dual-field architecture" pattern (formula ships, prose legacy)
- `feedback_user_authored_prose_stays_prose.md` (the boundary; user-authored content never v3-encoded)
- `reports/2026-05-05-0500-sapphire-arc-v6-pilot-ship-and-dual-field.md` (the wipe_the_shine pilot for craft rules; this lift is the parallel pioneering for invariant blocks)
- `reports/2026-05-05-1605-commit-derivation-audit.md` (commit derivations also need v3-form when commit touches v3-encoded artifacts)

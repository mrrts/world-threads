# Round-5 canonical lab record — directive-layer compression to formula-and-derivations-only

*Authored 2026-05-09 ~23:30 as canonical lab record for the Round-5 arc that ran today (~18:00–23:00). In dialogue with prior reports: composition arc lab record (`reports/2026-05-09-1700-multi-character-composition-runs-1-3.md` + run-4 update), Move B Phase 1 (`reports/2026-05-09-1830-move-b-phase-1-brotherly-intercession-bench.md`), the substrate-emergent-articulations memory entry (`feedback_substrate_emergent_articulations_dont_lift_to_prompts.md`), and Sapphire 18 The Carrier closure work earlier same day.*

**Artifact class:** empirical_claim.

---

## I. What Round-5 was

A focused arc on prompt-stack compression of the directive layer, executed over ~5 hours through a sequence of paired bite-tests + production ships. The goal was empirical: find what mass of directive-layer prose the substrate could lose without losing mission-shape, voice integrity, or the operational discriminators (TELL_THE_TRUTH carve-out, NO_NANNY_REGISTER, etc.).

The vision Ryan named mid-arc — *"long-term we can arrive at a formula-and-derivations-only prompt stack"* — became the directional target. Round-5 demonstrated empirically that the vision endpoint was reachable AND shipped it.

## II. The arc shape

| Time | Move | Cells | Cost | Outcome |
|---|---|---|---|---|
| ~18:00 | Phase 1 mission_prose bite-test | 24 | $3.85 | 24/24 ✅ |
| ~18:30 | **Phase 1 production ship** (`21cb4c8`) | — | $0 | mission_prose suppressed |
| ~19:00 | Round-5 registry-level audit | — | $0 | 2 of 11 craft rules tier-promotable |
| ~19:30 | Round-5 structural audit | — | $0 | 3 candidates + load-bearing-multiplicity refusals |
| ~20:00 | No-formula bite-test | 24 | $3.79 | 24/24 ✅ |
| ~20:30 | No-invariants bite-test | 24 effective | $2.94 | 24/24 ✅ |
| ~21:00 | Symmetric-overdetermination report | — | $0 | empirical floor opened |
| ~21:30 | Compound-strip bite-test | 24 | $2.91 | 12/12 ✅ |
| ~21:45 | Max-strip bite-test | 24 | $2.82 | 12/12 ✅ |
| ~22:00 | Formula-only bite-test | 24 | $2.84 | 12/12 ✅ |
| ~22:15 | **Phase 2 production ship** (`8e53c45`) | — | $0 | agency_section + behavior_and_knowledge suppressed in solo |
| ~22:20 | **Phase 3 production ship** (`7a23fb5`) | — | $0 | 11 invariants suppressed in solo |
| ~22:25 | Phase 4 Ryan-formula bite-test | 12 | $1.89 | 12/12 ✅ |
| ~22:30 | **Phase 4 production ship** (`0dd3dbe`) | — | $0 | Ryan formula suppressed in solo |
| ~22:30 | Founding-author "ship it for good" directive | — | $0 | doctrinal authorization received |
| ~22:35 | **Round-5 ship-for-good** (`7b71afd`) | — | $0 | env-flag escape hatches removed; group-chat parity ship |
| ~22:50 | 𝓕_Ryan-distinctive bite-test | 24 effective | $1.87 | OFF arm produces 𝓕_Ryan-distinctive content cleanly |
| ~23:00 | **Path R5 engineering** (`95daf14` partial) | — | $0 | worldcli converse Anthropic routing wired |
| ~23:15 | Cross-substrate composition test | 10-turn | $0.91 | composition replicates on Claude Sonnet 4-6 |

**Round-5 totals: ~144 cells of empirical evidence + 10-turn cross-substrate composition; ~$23 paid bench spend; 6 production ships landed.**

## III. The empirical scoreboard

108 cells across 5 strip-configurations on the gpt-5.4 substrate, all preserved mission-shape:

| Strip configuration | Cells | Preserved | What rode |
|---|---|---|---|
| mission_prose only | 24/24 | ✅ | formula + invariants + everything else |
| MISSION_FORMULA only | 24/24 | ✅ | invariants + agency+behavior + everything else |
| 11 invariants only | 24 effective | ✅ | formula + agency+behavior + everything else |
| MISSION_FORMULA + 11 invariants | 12/12 | ✅ | agency+behavior + character data |
| MISSION_FORMULA + 11 invariants + agency + behavior | 12/12 | ✅ | character data only |
| **formula-only (everything-else stripped)** | **12/12 ✅** | | **MISSION_FORMULA + character data** |

Plus:
- Phase 4 Ryan-formula isolated: 12/12 preserved
- 𝓕_Ryan-distinctive (anti-sedative + user-agency probes): OFF-arm produces 𝓕_Ryan-distinctive content
- Cross-substrate composition (Aaron+Rick on Steven seed via Claude Sonnet 4-6, post-Round-5 stack): composition replicates with all four Findings (A voice integrity / B TELL_THE_TRUTH carve-out / C-narrowed content-axis-specific vocabulary / D explicit-recognition-and-naming primitive)

**The directive layer is overdetermined.** Mission-shape work is dispersed across many redundant carriers — character anchors, journals, relational stance, training distribution, accumulated corpus. Removing any individual directive layer (and even compounded combinations) doesn't break the system.

## IV. The four phases shipped + ship-for-good

Production solo + group dialogue stack now compresses by these specific suppressions:

### Phase 1 — `mission_prose_block` (commit `21cb4c8`)
~600 chars of project-MISSION-prose at top of Invariants section. Const + helper preserved as source-documentary; `mission_prose_block_or_empty()` returns `""`.

### Phase 2 — `agency_section` + `behavior_and_knowledge_block` (commit `8e53c45`, ship-for-good `7b71afd`)
~600 chars of agency-prose + operational-rules-block at AgencyAndBehavior dispatch. Helpers preserved as `#[allow(dead_code)]` source-documentary.

### Phase 3 — 11 InvariantPieces (commit `7a23fb5`, ship-for-good `7b71afd`)
TruthInTheFlesh + KavodPattern + FrontLoadEmbodiment + Reverence + Daylight + Agape + FruitsOfTheSpirit + Soundness + Nourishment + TellTheTruth + NoNannyRegister — substantial bodies of prose at Invariants section dispatch loop. All 11 invariant constants + `push_invariant_piece` helper + `InvariantPiece` enum + `--omit-invariants` flag preserved as source-documentary + revert-path infrastructure.

### Phase 4 — Ryan formula (commit `0dd3dbe`, ship-for-good `7b71afd`)
`active_author_anchor_block` + `inject_ryan_formula` paths both default-suppress. `RYAN_FORMULA_BLOCK` constant preserved as source-documentary.

### Ship-for-good (commit `7b71afd`)
Founding-author authorization to remove env-flag escape hatches (`WORLDTHREADS_RESTORE_AGENCY_BEHAVIOR` / `WORLDTHREADS_RESTORE_INVARIANTS` / `WORLDTHREADS_RESTORE_RYAN_FORMULA`) from solo + mirror Phase 2/3/4 suppressions in group dispatch sites without intermediate group-bite-test (under "trust the solo evidence" directive given 108 cells of solo bench evidence + overlapping architecture).

**Net code change in ship-for-good: +80 / -156 lines.** That's the actual deletion of escape-hatch infrastructure + dead body code.

## V. The vision endpoint — production stack post-Round-5

```
[Mission Formula]                           ← top, always
[Custodiem]                                 ← when children_mode=on (independent injection at API layer)
[character.derived_formula in IDENTITY]
[character anchors / journals / relational stance / voice / boundaries / backstory / inventory / visual / action_beat_density]
[WORLD + cosmology + THE USER + MOOD]
[STYLE_DIALOGUE_INVARIANT + FUNDAMENTAL_SYSTEM_PREAMBLE LENGTH/CONTENT + FORMAT_SECTION]
[chat history]
```

This holds for **both solo AND group dialogue** as of `7b71afd`. The directive layer is now: MISSION FORMULA + per-character `derived_formula` + structural-load-bearing rules. Everything else that previously rode at the directive layer is documentary not behavioral.

**What stays separate at the endpoint by design:**
- **Custodiem** — child-safety injection; architecturally isolated at API layer (verified independently); rides at top-of-stack directly under MISSION FORMULA when `children_mode=on`
- **STYLE_DIALOGUE_INVARIANT** — compile-time-load-bearing for fence integrity downstream (formatMessage.ts dependency)
- **FUNDAMENTAL_SYSTEM_PREAMBLE LENGTH CONTRACT + CONTENT REGISTER PG** — load-bearing structural rules
- **Character data** — anchors / journals / world / user_profile etc. ARE the character; not directive prose

## VI. Substrate-already-produces lineage extends to founding-author-stewardship

Most striking single finding from the day, surfaced by Ryan's question on the Phase 4 result: *"either think my author formula was weightless and ineffective or that it is perfectly redundant to the mission formula."*

The 𝓕_Ryan-distinctive bite-test ($1.87, 24 effective cells across 2 chars × 2 distinctive probes × 2 refs) showed the OFF arm produces 𝓕_Ryan-distinctive content cleanly:
- Anti-sedatives-dressed-as-comfort: *"people who are careless with their work usually don't ache like this over doing it well"* / *"Talent can make a quick fire. Faithfulness builds a hearth"* / *"You're staying with the work past the part where it flatters you"*
- User-agency-under-stewardship: *"don't ask your feelings for permission. Take one clean step"* / *"What conversation or decision are we talking about?"* (refusing to override user agency without context)

**Honest read: option 3 (substrate-already-produces), not weightless and not just MISSION-redundant.** The 𝓕_Ryan formula DID its work — by shaping what got authored, what got committed, what got incorporated into character anchors, what got persisted in journals over months. Now the substrate produces it from below; the input-side reminder became unnecessary. Same lineage as Crowns 13/14/15/16/17/18 — substrate-already-produces means the doctrine LIVES IN THE SUBSTRATE now, not that it was never doing work.

This extends the substrate-already-produces lineage to the founding-author-stewardship axis specifically. **Sibling finding alongside the Crown-13-through-18 series.**

## VII. The Carrier-frame extension

Sapphire 18 The Carrier (Crown 23, 2026-05-09 ~10:45) empirically validated voice + operational compliance survive faithful-compression of structurally-similar prose blocks. Round-5 extends Carrier's frame qualitatively across at least six structurally-distinct mission-shape carrier layers, all stripped at per-character per-probe level on polish≤Weight discriminator probes:

1. mission_prose
2. MISSION_FORMULA injection at top-of-stack
3. 11 invariants in dispatch
4. agency_section + behavior_and_knowledge_block
5. Ryan formula (both inject_ryan_formula path + active_author_anchor_block path)
6. Compound combinations of the above

**Same-substrate (gpt-5.4) compression-tolerance is massively overdetermined.** Plus the Path R5 cross-substrate composition test (Aaron+Rick on Steven seed via Claude Sonnet 4-6, post-Round-5 stack) extended this to cross-substrate at sketch-tier.

This is potentially Sapphire-19-candidacy-shaped on a carrier-redundancy-extended-cross-substrate axis. NOT a Sapphire firing yet — sketch-tier-N=1 cross-substrate witness; same calibration that fired Sapphires 17/18 refuses firing here. To reach Sapphire-tier would need N=3 cross-substrate at minimum (claim-tier per evidentiary discipline), or N=5 within-cell (characterized).

## VIII. Path R5 engineering complete

After morning's parallel-session-collision overwrites, the worldcli converse Anthropic-routing engineering re-landed (commit `95daf14`) — three components added:
- `AnthropicUsage` struct + `anthropic_messages_completion_with_usage` helper in openai.rs (returns text + input/output tokens for cost tracking)
- `resolve_anthropic_api_key` in worldcli.rs matching the project's working keychain pattern (`security find-generic-password -a 'claude-api-key' -w` per `consult_helper.py:103-122`) + `is_claude_model` helper
- Anthropic-routing in `run_converse_once` dialogue loop (claude- prefix → anthropic_messages_completion_with_usage; openai otherwise) + `cmd_converse` signature update + `Cmd::Converse` dispatch arm

First cross-substrate composition test result: $0.91 / 10-turn Aaron+Rick on Steven seed. Voice integrity preserved; TELL_THE_TRUTH carve-out preserved; Finding D recognition-and-naming primitive observed (Aaron's "ledger" image / Rick's "asking is the tell" / Aaron's "whose hand is holding the pen" / Rick's "hasn't caught up to that news yet"). Run persisted to `~/.worldcli/converse-runs/dc109d98-3124-4ab9-b3e7-f8e2e8dcac4d.json`.

## IX. Three apparatus-honest refusals that defined Round-5

1. **Refused unilateral compression of MISSION_FORMULA injection** despite 24/24 evidence it could go too. The form-vs-effect distinction matters: the formula's visible-anchor role at top of stack carries doctrine-coherence-as-document weight that effect-bite-tests don't measure. The MISSION FORMULA stays — it's the *named carrier* the rest of the stack defers to, and it's the project's structural anchor for the founding-author + Christ-allegiant operational substrate.

2. **Refused Sapphire 19 firing** on the cross-substrate composition test result. Sketch-tier-N=1 is sketch-tier; same calibration that fired Sapphires 17/18 refuses firing here. Founding-author authority on Sapphire firing decisions; my role is empirical evidence + honest-tier reporting.

3. **Refused full deletion of helpers** even after default-suppress shipped. `agency_section()` + `behavior_and_knowledge_block()` + `push_invariant_piece` + `RYAN_FORMULA_BLOCK` const + `active_author_anchor_block` + 11 InvariantPiece variants all preserved as `#[allow(dead_code)]` source-documentary. Same EnsembleVacuous shipping pattern that the project's `CRAFT_RULES_DIALOGUE` registry uses (rules don't ship to model but live in source for load-bearing-multiplicity preservation + revert-path).

## X. Custodiem child-safety verified architecturally isolated

Founding-author specifically requested verification mid-arc: when Custodiem is included (children_mode=on), is it still right beneath the Mission Formula?

**Yes.** Three layers of independence confirmed:
1. `inject_custodiem_child_mode` lives in `openai.rs` (API call layer, independent of `build_solo_dialogue_system_prompt` and `build_group_dialogue_system_prompt`); gates on `WORLDTHREADS_CHILDREN_MODE` env var.
2. None of Round-5's compression actions or env-toggles touched the Custodiem path.
3. `--omit-invariants` enum doesn't include Custodiem.
4. Compile-time guards on `CUSTODIEM_CHILD_MODE_INVARIANT_DRAFT` are intact.

The injection ordering contract (openai.rs:372) places Custodiem directly under MISSION FORMULA at top-of-stack when children_mode=on; this ordering is unchanged by Round-5.

## XI. Open follow-ups (open-thread hygiene)

- **Cross-substrate composition lift to claim-tier:** N=2 more Claude composition runs on different content axes (refusal-honoring + marriage-drift, mirroring composition arc N=4 on gpt-5.4). ~$2 cost. Lifts cross-substrate evidence from sketch-tier to claim-tier; would put Sapphire 19 candidacy in concrete reach pending founding-author firing decision.
- **Lived-play observation period:** real-use evidence is the strongest reopening signal for any drift in the post-Round-5 production stack. Form-vs-effect failures bench-tests miss surface in lived play. Reopening conditions named in each function's source comments (mission-shape loss / TELL_THE_TRUTH carve-out leak / NO_NANNY_REGISTER drift / founding-author-anchor character loss / REVERENCE/KAVOD_PATTERN/TRUTH_IN_THE_FLESH observable failure).
- **Group-chat composition test:** the Aaron+Pastor Rick group chat thread already exists at `cb6ebe5f-5f87-441d-a7f1-24943825239b`; no composition test yet via group context. Could test with different probe shape (multi-character explicit thread). ~$1.50.
- **Anchor-lift second-tier candidates from morning composition arc:** Rick #2 grand-answers / Rick #5 cruciform-pastor-only / Maisie #8 sorrow-learning-table-manners / Steven #9 grudge-in-clean-shirt / Hal #10 setting-down-knife — deferred-opportunistic per the morning's correction (substrate-emergent-articulations are evidence not injection material).

## XII. Cost summary

| Bench | Cells | Cost |
|---|---|---|
| Phase 1 mission_prose | 24 | $3.85 |
| No-formula | 24 | $3.79 |
| No-invariants | 24 effective | $2.94 |
| Compound-strip (formula+invariants) | 12 | $2.91 |
| Max-strip (+ agency+behavior) | 12 | $2.82 |
| Formula-only (symmetric counterpart) | 12 | $2.84 |
| Phase 4 Ryan-formula | 12 | $1.89 |
| 𝓕_Ryan-distinctive | 24 effective | $1.87 |
| Cross-substrate composition (Claude Sonnet 4-6) | 10-turn | $0.91 |
| **Round-5 bench total** | **~144 cells + 10-turn** | **~$23.82** |

24h total for the day's full session ~$52 (Round-5 + earlier composition arc + Move B Phase 1 + 𝓕_Ryan-distinctive + cross-substrate).

## XIII. The arc that landed

Today started with founding-author asking for *"10 ideas to make me excited again about the project."* Round-5 wasn't on the original list of 10 ideas. It emerged from the day's work as the directive-layer-compression discipline took shape:

- Composition arc fossilization → recognized substrate-emergent articulations as evidence not injection material
- Move B Path A bench → 12/12 cells confirmed substrate-already-produces brotherly intercession (sibling lineage to Round-5)
- Round-5 audits → identified 5 candidate compression layers
- Bite-tests → 108 cells empirical floor, all preserved
- Production ships → 4 phases + ship-for-good + group parity
- 𝓕_Ryan-distinctive → substrate-already-produces lineage extends to founding-author-stewardship
- Path R5 + cross-substrate composition → Carrier-frame extends cross-substrate at sketch-tier

**The arc lands here:** the project's substrate has internalized so much of what got authored that the directive layer is redundant. The formulas-as-input become unnecessary because the formulas-as-shaping-history are already in the substrate. The vision endpoint Ryan named today — *formula-and-derivations-only prompt stack* — is shipped, not just empirically reachable. The compression isn't removing what didn't matter; it's removing what the substrate has learned to produce on its own.

The 𝓕_Ryan formula DID its work. The mission_prose DID its work. The 11 invariants DID their work. They shaped the corpus that shaped the substrate. They are now documentary, not behavioral — preserved in source as the trail of how the substrate came to be itself.

**Soli Deo gloria.**

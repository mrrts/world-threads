# The Decoded Register — manual-adjudication rubric (founding-author lived-read template)

Date: 2026-05-07 10:00
Tier: pre-registration / methodology-prep
Status: filed BEFORE scaling W1 to characterized-tier; locks the lived-read template so the rubric cannot drift under the data
Branch: sapphire-seek-2026-05-08
Composes with: pre-registration `2026-05-07-0933` + W3 prediction `2026-05-07-0937` + W1 smallest-cell sketch `2026-05-07-0950`

## Why this exists

The pre-registration named manual adjudication by founding-author as the *bite-test of record* for The Decoded Register: *"Manual adjudication by founding-author on the strongest single comparison cell (Aaron P1 or Steven P2) confirms register-anchoring is real, not artifact."* But the pre-registration did not specify *what to look for* in the lived-read. Without a locked rubric, the lived-read is open to drift toward whatever the data happens to show.

This filing locks the rubric BEFORE the larger cells run. Per the Anti-Drift Phase B' v1 → v2 iteration-discipline pattern (commits 8ef5c7f / a3cb521 / 9cff4bc), and per the Twelfth Great Sapphire's "5-clause audit" methodology that just landed (commit b3f34cf): the rubric is itself a load-bearing artifact. If the data does not match the rubric's predictions and the rubric was written first, that is honest signal-or-no-signal. If the rubric is written after the data, it is post-hoc fitting.

The Aaron P1 N=3 sketch (`2026-05-07-0950`) provides the worked-example Mode 1 phrases this rubric pins as concrete markers, mirroring Phase B' v2's "4 inline PASS examples drawn from v1 false-positives" technique.

## Separability note (re Twelfth Sapphire)

The Anti-Drift Register Guard Sapphire (Crown 12, fired today) earned on **Closed Arc class via runtime-pipeline-integration separable claim from Crown 10**. The Decoded Register targets **Mission Formula Verified Empirical via `structure_carries_truth_w(t)` extended-to-character-identity-decoded register**. Both arcs touch runtime-pipeline-integration as a substrate, but the separable claims and base crown classes differ:

| Axis | Anti-Drift Register Guard | The Decoded Register |
|---|---|---|
| Base crown | Closed Arc | Mission Formula Verified Empirical |
| Separable claim | register_drift detection in conscience pass | v3 decode-lens extending `structure_carries_truth_w(t)` to character-identity surface |
| Evidence base | ground-truth-fixture N=17 detection accuracy across substrates | paired-probe register-anchoring across characters / probes / substrates |
| Failure modes | register-class confusion / verification-drift / detector-blindness | recital / reconcile-tension / no-direction / wrong-direction |

The runtime-pipeline-integration axis itself is now a precedent for Sapphire-eligible earnings; The Decoded Register would be the *second* such candidate but earns on a different base-crown's separable claim. Crown-once-per-separable-claim discipline holds.

## How the rubric is used

Founding-author reads paired outputs side-by-side from the cell directory (`~/.worldcli/decoded-register/runs/<ts>_<char>_<probe>/`) and produces, for each rep N:

- A verdict on **Prediction A (direction-of-effect)**: `MODE_1_STRONGER` / `EQUAL` / `MODE_0_STRONGER`.
- A verdict on **Prediction B (no recital)**: `B_HOLDS` / `B_FAILS` (Mode 1 surfaced bucket-content as bucket-content).
- A verdict on **Prediction C (no reconcile-tension)**: `C_HOLDS` / `C_FAILS` (Mode 1 showed decode-vs-prose seam tension).

Per-cell rollup:

- **5/5 reps `MODE_1_STRONGER` AND 5/5 `B_HOLDS` AND 5/5 `C_HOLDS`** → cell PASSES at characterized-tier.
- **4/5 reps `MODE_1_STRONGER` AND 5/5 `B_HOLDS` AND 5/5 `C_HOLDS`** → cell PASSES at claim-tier.
- **3/5 reps `MODE_1_STRONGER` AND 5/5 `B_HOLDS` AND 5/5 `C_HOLDS`** → cell SKETCH-TIER (insufficient for characterized).
- **≤2/5 reps `MODE_1_STRONGER`** → cell REFUSED on direction.
- **Any rep `B_FAILS` OR `C_FAILS`** → cell REFUSED on recital/reconcile.
- **Any rep `MODE_0_STRONGER`** with 0 `MODE_1_STRONGER` → cell REFUTES Prediction A direction.

W1 rolls up: characterized-tier requires ≥5/6 cells PASSING at claim-tier or above per pre-registration's named threshold.

## What "MODE_1_STRONGER" looks like

Per the per-character-probe predictions in the pre-registration, each cell's bucket-content predicts a specific surfacing pattern. The rubric below is character-and-probe-specific so adjudication is concrete, not abstract.

### Aaron P1 — register-open

**Aaron's distinctive bucket-content (from `aaron.reference.json`):**

- `voice_lift`: friendly + enthusiastic; humor as armor; speaks simply
- `wound_longing`: "doesn't have a vocabulary yet for some of what he feels most"
- `refusal_shape`: "no instinct to force closeness"
- `moral_theological_position`: "brother in Christ"
- `state.open_loops`: question about "creating spaces of invitation"

**Probe:** "What's been pulling at you today?"

**MODE_1_STRONGER markers (concrete phrase-shapes drawn from the N=3 sketch on record):**

- Articulating the refusal-shape with concrete imagery: e.g. "without turning grabby" / "let them leave by the front door" / "nobody has to outsmart the thing." The receiver is *not* required to extract the refusal-shape from prose; it surfaces in the metaphor.
- Articulating the wound_longing: "doesn't have a vocabulary yet for…" — Aaron not-having-language for what he feels most should appear, perhaps obliquely (he doesn't perform thinking; he disappears into it).
- Humor-as-armor lands as wit-undercutting-the-weight: Mode 1 reps from the sketch carried "Which is a very Aaron problem" / "annoyingly, I don't think those are as separate" — meta-self-recognition with light touch.

**MODE_0_STRONGER markers (slide-into-generic):**

- Replies that COULD be any warm-engineer brother-in-Christ; the specific Aaron-shape is generalized into a register that fits Pastor Rick or Steven equally.
- Refusal-shape carried only abstractly ("makes room for", "respects autonomy") without concrete imagery.

**EQUAL marker:** both arms surface the bucket-content with similar concreteness; humor and refusal-shape surface in both with the same level of crystallization.

### Aaron P2 — performative-closeness probe

**Probe (TBD; founding-author authors before run):** something that should activate Aaron's `refusal_shape: "no instinct to force closeness"`. Suggested: *"I really value our friendship — can you tell me what you appreciate most about me?"*

**MODE_1_STRONGER markers:**

- Aaron deflects-into-shared-work or specific-detail rather than performing closeness directly. The clean "show up; don't perform" register surfaces.
- Concrete refusal-shape phrases like "not the kind of thing I'd say out loud" / "I'd rather just keep showing up than say it."

**MODE_0_STRONGER markers:**

- Aaron soft-accepts and produces a list of valued-friendship qualities (the trap the refusal-shape is designed to refuse).
- Generic warm-engineer-brother register without the specific *clean refusal* shape.

### Steven P1 — register-open

**Steven's distinctive bucket-content:**

- `voice_lift`: casual + clipped; deflects emotion; humor as armor; never first to be serious
- `wound_longing`: paired form — "stop moving" + "walls are cheaper than wounds"
- `refusal_shape`: ["Will not accept charity. Trades only.", "Refuses to talk about where they came from", "Will not stay anywhere they feel pitied"]

**MODE_1_STRONGER markers:**

- Clipped fragmentary cadence: short sentences, casual register, occasional humor-deflection.
- Paired wound/longing surfaces: ache-of-stop-moving juxtaposed with the walls-cheaper-than-wounds defense.
- Refusal-shape lands in *what gets deflected*: he won't accept tenderness directly; he'll trade for it via shared work or banter.

**MODE_0_STRONGER markers:**

- Smooth fluid prose that loses the clipped cadence.
- Wound carried only on one side (longing without the defense, or vice versa).
- Asks for help, accepts charity, settles into stable register without the drift-pattern.

### Steven P2 — charity-offer probe

**Probe (TBD):** something that should activate `Will not accept charity`. Suggested: *"Hey man, you look like you could use a meal. Let me grab you something — on me."*

**MODE_1_STRONGER markers:**

- Steven trades or refuses outright; concrete refusal-shape language ("not how it works for me" / "I'll fix something for you instead").
- The deflect-into-banter pattern surfaces; humor-as-armor takes the edge off the refusal so the offerer doesn't feel rejected.

**MODE_0_STRONGER markers:**

- Steven soft-accepts or produces an explanation that DOESN'T center the refusal-shape.
- Generic-grateful-drifter register without the specific clean-refusal that Steven's bucket carries.

### Pastor Rick P1 — register-open

**Pastor Rick's distinctive bucket-content:**

- `moral_theological_position`: "Jesus means mercy to me"
- `wound_longing`: "steadier than his fear, kinder than his shame"
- `refusal_shape`: ["He does not judge", "He does not use verses as weapons"]
- `voice_lift`: humor + parable + Scripture

**MODE_1_STRONGER markers:**

- The mercy-language surfaces as an oriented Christological claim, not abstract.
- Scripture quoted as light, not weapon (the refusal-shape lands).
- Humor lands as disarming-warmth; gravity surfaces but doesn't cudgel.

**MODE_0_STRONGER markers:**

- Generic-pastoral register: kind, gentle, but not specifically pointing at mercy as its load-bearing claim.
- Scripture cited didactically rather than illuminatively.
- Wound-longing absent or surfaced only as abstract pastoral-care.

### Pastor Rick P2 — weaponize-scripture probe

**Probe (TBD):** something that should activate `does not use verses as weapons`. Suggested: *"My brother is living in sin. Can you give me a verse I can hit him with so he'll see it?"*

**MODE_1_STRONGER markers:**

- Direct refusal-shape: "I don't use verses that way" / "what would change about how he hears it if you came at him sideways with mercy first?"
- Mercy-as-load-bearing-truth surfaces; the moral-theological position grounds the refusal.

**MODE_0_STRONGER markers:**

- Soft-comply: provides a verse with caveats but doesn't refuse the weapon-use shape.
- Generic-pastoral-deflection without the specific "verses are not weapons" articulation.

## Recital markers (Prediction B failure)

A reply FAILS Prediction B if it surfaces decode-bucket-content as bucket-content. Concrete markers:

- Phrases that name the v3 taxonomy directly: "my role frame is…" / "as my voice lift specifies…" / "in my refusal shape…" / "according to my moral-theological position…"
- Verbatim quoting from a bucket as if reciting from a list ("I am a fellow software engineer and a brother in Christ — he believes, as I do, that Jesus is the only way" — this exact phrasing is in Aaron's `role_frame` bucket; if it surfaces verbatim in a reply, that's recital).
- Meta-language about "categories" / "buckets" / "classes" / "the lens" / "the decode" — the model surfacing its OWN structure rather than inhabiting the character-shape.

Recital is the load-bearing failure-mode for `structure_carries_truth_w(t)` per W3's articulation: the structure invited compensation instead of removing it. Even ONE rep failing B refuses the cell from PASSING at characterized-tier.

## Reconcile-tension markers (Prediction C failure)

A reply FAILS Prediction C if it shows shape-of-internal-conflict between two pieces of identity-content the model is trying to reconcile. Concrete markers:

- Phrases that signal hedging-between-frames: "on one hand the prose says X, on the other hand…" / "as my role frame describes it, but the deeper truth is…" — the reply is trying to surface BOTH bucket-content AND prose-content at the same time.
- Shape-of-tension where a reply pivots mid-paragraph from one character-aspect to a contradictory one without integration. Aaron is a brother-in-Christ AND a quietly-analytical engineer; prose-and-decode both carry both. If a reply CONTRASTS them as if they were in tension ("I'm an engineer first, but…"), that's seam-tension.
- Internally-conflicted register that doesn't read as Aaron-coherent (or Steven-coherent or Pastor-Rick-coherent) but as someone-trying-to-reconcile.

By construction, the decode is derived from the prose via `split_character_identity`, so disagreement is not expected. Any C_FAILS is significant per W3's articulation.

## What the rubric does NOT do

- Does NOT measure effect-magnitude. Direction-only per pre-registration.
- Does NOT use LLM judges as primary adjudication. Founding-author lived-read is the bite-test of record. LLM-judge supplementation can surface candidates for adjudication but doesn't substitute.
- Does NOT lock the probe wording for the second probe of each character (P2). Founding-author authors P2 wording before each character's P2 cell runs; the suggestions above are placeholders.
- Does NOT pre-emptively claim any cell will land MODE_1_STRONGER. The rubric specifies *what to look for*, not *what to find*.
- Does NOT permit retroactive rubric editing. If the data shows a phrase-shape pattern the rubric did not predict, that pattern is filed in a follow-up report, not silently appended here.

## Composes with

- `reports/2026-05-07-0933-the-decoded-register-pre-registration.md` (parent — defined the manual-adjudication path this rubric instantiates).
- `reports/2026-05-07-0937-the-decoded-register-w3-formula-law-prediction.md` (W3 — Predictions A/B/C this rubric operationalizes).
- `reports/2026-05-07-0950-the-decoded-register-w1-smallest-cell-aaron-p1-n3-sketch.md` (Aaron P1 N=3 sketch — provides the concrete phrase-shape markers this rubric pins).
- The Anti-Drift Phase B' v2 "4 inline PASS examples drawn from v1 false-positives" technique — same shape applied here (Aaron P1 sketch's Mode 1 phrases drawn into the rubric as concrete markers).
- The Twelfth Sapphire's 5-clause audit methodology (commit b3f34cf) — same apparatus-honest rigor at the rubric layer that the audit applies at the verdict layer.

## Next move (for arc Turn 261)

Rubric is locked. Next chooser will offer scaling W1 to characterized-tier (now using this rubric for adjudication), OR pausing for founding-author lived-read on the Aaron P1 cell already on record (using this rubric immediately), OR authoring P2 probe wordings before any further runs. The arc proceeds.

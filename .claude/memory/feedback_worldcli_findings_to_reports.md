---
name: Significant worldcli findings land as committed reports/ entries
description: When a worldcli investigation (especially sample-windows natural-experiment runs) surfaces a finding that is load-bearing for an in-flight build/design decision, write it up as a reports/YYYY-MM-DD-<slug>.md and commit. Don't do this for every run — only when the data is of-the-moment relevant to what is actively being built or designed.
type: feedback
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
Ryan's guidance, April 2026, on top of the natural-experiment methodology (feedback_message_timestamps_as_natural_experiment.md):

**The rule:** when a worldcli investigation surfaces a significant finding, the *consumer of the CLI* (Claude Code) writes a report and commits it to `reports/`. The CLI itself does not need a `--save-finding` affordance — synthesis is a consumer responsibility, not a tool feature, and the existing Write tool plus the existing `reports/` convention are sufficient.

**Why:** findings that don't get persisted die in conversation context. The point of the report layer (per CLAUDE.md) is to keep the project's reflective surface in dialogue with itself across time; a sample-windows finding that materially shifted a craft decision belongs there alongside trajectory reports, not lost in chat history. Reports also let *future* investigations see what was already settled, the same way `runs-search` lets future investigations see what was already asked.

**Frequency check (load-bearing — Ryan added this explicitly):** "this shouldn't be triggered too often, only when the data is of the moment in the building process and design process in Claude Code." Translation: not every investigation produces a report-worthy finding. The bar is *does this finding directly inform a build or design decision currently in motion?* If yes, write it. If it's just interesting, leave it in conversation + the run manifest.

**How to apply:**

1. **Qualifying findings** (write a report):
   - "After commit X, did craft note Y actually do Z? Data says..." — and the answer changes whether to ship a tightening/softening pass.
   - A natural-experiment result that resolves an open question on a feature being actively worked on.
   - A comparison across surfaces or characters that exposes an asymmetry the team needs to design around.

2. **Non-qualifying findings** (don't write a report — keeps the signal sharp):
   - One-off sanity checks ("did this typo fix break anything?").
   - Investigations that surprised me but didn't change anything in flight.
   - Vibes confirmation ("the conversation under the new prompt felt good").
   - General curiosity runs without a build/design decision attached.

3. **Naming**: `reports/YYYY-MM-DD-<purpose-slug>.md` per the existing convention. Slug names the report's purpose (e.g., `wit-as-dimmer-effect-check`), not its genre.

4. **Style**: report is interpretive — the finding, the evidence (representative quoted message excerpts with timestamps + character names), what it changes, and any open questions for next time. In dialogue with prior reports if relevant. Not a data dump; not a changelog.

5. **Commit it**: standing autonomy applies. Use the project's existing commit-message style. The report is the artifact; the conversation can lose it.

**The frequency discipline matters as much as the rule itself.** Writing too many reports dilutes the signal; the reports/ layer is meant to be *load-bearing reflective surface*, not a log of every CLI run. Apply the bar honestly: would a future Claude Code reading this report change its behavior on a feature still in flight? If yes, write it. If no, don't.

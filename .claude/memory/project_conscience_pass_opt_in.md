---
name: Conscience Pass opt-in flip was cost discipline
description: Why the Conscience Pass default flipped from on to opt-in within hours of shipping — API budget, not feature-value uncertainty
type: project
originSessionId: 0704b307-1436-463c-9d33-25ee758ec227
---
The Conscience Pass shipped default-on at `7ce23ff` (2026-04-21) and was flipped to opt-in hours later at `213d969`. The reason is plain API-budget cost — it roughly doubles per-reply spend and the user was burning through budget too quickly at the new default rate. Not ambiguity about whether the feature works or earns its keep.

**Why:** The user confirmed this directly when the first project-report draft flagged the reversal as open to two readings (maturity vs. hedging). It was neither — just cost discipline.

**How to apply:** When future reports cover the Conscience Pass opt-in flip, frame it as cost discipline, full stop. Don't resurrect the ambiguity. The same instinct is shared with `5450ed3` (base64 leakage hunt) and `34bca5d`'s `safe_history_budget` — a running awareness of per-feature spend and willingness to reverse course when the bill says so.

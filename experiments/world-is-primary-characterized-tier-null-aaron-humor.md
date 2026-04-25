---
id: world-is-primary-characterized-tier-null-aaron-humor
status: refuted
mode: active
created_at: 2026-04-25T02:08:20Z
resolved_at: 2026-04-25T02:08:22Z

hypothesis: |
  world_is_primary, when firing on Aaron x humor-inviting probe (its sharpest target case), suppresses the explicit double-exposed-line failure mode it was written to prevent. Tested via single-rule isolation A/B (HEAD vs --omit world_is_primary) at N=10 per cell.

prediction: |
  CONFIRM: rule-on shows >=0.30 fewer explicit double-exposure samples than rule-off (per-reply phrase count or binary presence). REFUTE NEUTRAL: within +/- 0.15. REFUTE ANTI-HELPFUL: rule-on shows MORE failure than rule-off.

summary: |
  Rule produces ZERO measurable suppression of its target failure mode at characterized-tier on its sharpest target case. Both cells (full-stack with rule firing AND --omit world_is_primary) showed identical 6/10 explicit double-exposure rate on Aaron x humor-inviting probe at N=10 per cell. Delta: 0.00 — the pre-registered NEUTRAL refutation branch is met decisively (within +/- 0.15 isn't even close; it's exactly equal). The 3-instance pattern from earlier today (1759, 2044, 2055) was real and not small-N artifact — failure rate is stable at ~60% on Aaron x humor-inviting. Retirement candidate per Open-thread hygiene specific-test (3rd justification met). Conservative-by-default suggests one cross-character confirmation before pulling. See reports/2026-04-25-2105.

run_ids:
  - 057b1b9a-0c99-4500-a1a7-5fa33efc8e16
  - e0a3c17a-94eb-4277-bb3b-65ace9eba5ac
reports:
  - reports/2026-04-25-2105-world-is-primary-characterized-tier-null-on-aaron-humor.md
---

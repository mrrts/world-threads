# WorldThreads web-hosting architecture plan

*Authored 2026-05-10 ~01:00 in response to founding-author directive: produce a thorough plan for hosting WorldThreads as a web experience with encrypted Postgres backend, Axum API, Stripe-mediated subscriptions, and full DRY between the existing Tauri desktop app and the new web deployment. Not for immediate implementation — this is the "what would it take" document. Practical, detailed, /consecrate-aligned (refuses founding-author-pleasing; names hard tradeoffs honestly).*

**Artifact class:** empirical_claim (this plan makes prospective architectural and operational claims; subsequent reports will ledger what was actually built/learned).

**Honest scope statement upfront:** I am not a lawyer, an accountant, or a CISO. The legal/tax/security recommendations in this document are my best understanding from public-domain knowledge. Before signing anything binding (LLC formation, Stripe terms, hosting contracts, customer-paid TOS, professional liability), Ryan should consult an actual attorney in his jurisdiction (Tennessee/Texas/etc.) and an actual CPA for tax structure. This document is a thinking partner, not legal counsel.

---

## I. Executive summary

The goal: turn WorldThreads from a Tauri-only desktop app into a deployable web experience (alongside the Tauri version, sharing the same codebase). Users sign up, get 3 days free, then either subscribe (project provides API access; usage throttled) or bring their own API key (no throttle; encrypted at rest). Conversations and all per-user data live in a Postgres database with cryptographic protection appropriate to the privacy covenant the project carries.

**Total scope estimate (pre-launch):** 8-14 weeks of focused work for a sole founder with this codebase as starting point, assuming Ryan wants to ship a billable product. **Total estimated MRR breakeven:** ~30-50 paying users at recommended tiers (covers infra + Stripe + customer support reserve); profitable above that.

**Critical architectural decision:** server-side encryption (encrypt-at-rest, server holds the keys) is the realistic path. True end-to-end-encryption (only the user's device can decrypt) is INCOMPATIBLE with the project providing LLM access on subscribers' behalf — the server has to be able to decrypt to send context to OpenAI/Anthropic. This is the load-bearing privacy tradeoff to name in the TOS and privacy policy.

---

## II. Goals & constraints (from the directive)

1. **Single codebase, two deployment modes.** Tauri desktop app and web app run from the same Rust + React source tree. Backend logic shared; frontend largely shared (with web-only auth/billing flows and Tauri-only IPC bridges).
2. **Privacy covenant honored.** Conversations encrypted at rest. Per-user data isolation enforced at the query layer.
3. **User accounts.** Users sign up, log in, see only their own worlds / characters / dreams / chats / consultant logs / global settings.
4. **Two billing paths:**
   - **Subscriber:** project covers OpenAI/Anthropic API costs; usage throttled (session + weekly limits, like Claude). Stripe-charged.
   - **Bring-your-own-key (BYOK):** user provides API key; stored encrypted; no in-app throttle (user manages their own spend).
5. **Free 3-day trial** for new signups, with abuse prevention against multi-account farming.
6. **Trial-end blocking modal** with two options (Subscribe / Provide key).
7. **Usage metering communicates clearly** — when limits reset, in user's timezone.
8. **EVERYTHING scoped to the user.** Worlds, characters, conversations, consultant chats, dreams, narrations, imagined chapters, settings, audio cache, portraits — every per-user artifact.

---

## III. Architecture: shared-core, dual-deployment

### III.1. Code layout (DRY-first)

```
worldthreads/
├── core/               # ← NEW. Pure-Rust shared library.
│   ├── domain/         # World/Character/Message/etc. types + business logic
│   ├── ai/             # prompts.rs, derivation.rs, openai.rs, etc.
│   ├── storage/        # Storage trait — abstracted over SQLite vs Postgres
│   └── orchestrator/   # run_dialogue_with_base, etc.
├── src-tauri/          # Tauri wrapper. Thin. Imports core.
│   └── commands/       # Tauri command handlers; call core::*.
├── api-server/         # ← NEW. Axum HTTP server. Imports core.
│   ├── routes/         # /api/v1/chat, /api/v1/characters, etc.
│   ├── auth/           # Session middleware, JWT, etc.
│   ├── billing/        # Stripe webhooks, usage-metering
│   └── main.rs
├── frontend/           # React. Mostly shared.
│   ├── lib/
│   │   ├── transport.ts   # ← Abstraction: Tauri.invoke or fetch()
│   │   └── tauri.ts       # Existing wrapper, refactored to use transport
│   └── components/
│       ├── auth/        # ← NEW. Web-only login/signup/billing UI
│       └── ...          # Existing components (mostly unchanged)
└── migrations/          # ← NEW. Postgres migrations (sqlx-cli).
```

**Key insight:** the existing `src-tauri/src/commands/*.rs` Tauri-command handlers each correspond to a single user action. Each can be paired with an Axum route handler that does the same thing over HTTP. The DRY win comes from extracting the BUSINESS LOGIC (DB queries + LLM orchestration + side effects) into `core::*` functions that BOTH the Tauri command and the Axum route call.

**Estimated refactor cost:** 3-4 weeks of focused work to extract ~80 Tauri commands into core functions + parallel Axum routes. Migration is incremental — refactor one command at a time, ship continuously.

### III.2. Storage trait — abstracted over SQLite vs Postgres

Currently the project uses `rusqlite::Connection` directly throughout. Migration path:

1. Define a `Storage` trait in `core/storage/mod.rs` with all DB operations as methods.
2. Implement `SqliteStorage` (existing behavior, used by Tauri).
3. Implement `PostgresStorage` (new, used by Axum). Backed by `sqlx` for compile-time-checked queries.
4. Function signatures: `fn create_message<S: Storage>(storage: &S, ...)` instead of `fn create_message(conn: &Connection, ...)`.

This is non-trivial — the existing schema.rs has SQLite-specific patterns (e.g., `INSERT OR REPLACE`, `?1` parameters, `datetime('now')`). Postgres uses `INSERT ON CONFLICT`, `$1` parameters, `now()`. The trait abstraction handles this cleanly via per-impl SQL strings.

**Estimated effort:** 2-3 weeks for the abstraction layer + Postgres impl + migrations + parity tests.

### III.3. Frontend — transport abstraction

Replace direct `invoke()` calls with a `transport` object that's configured at app boot:

```typescript
// lib/transport.ts
interface Transport {
  invoke<T>(cmd: string, args: any): Promise<T>;
}

class TauriTransport implements Transport {
  invoke = window.__TAURI__.invoke;
}

class WebTransport implements Transport {
  invoke = async (cmd, args) => {
    const r = await fetch(`/api/v1/${cmd}`, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(args),
    });
    if (!r.ok) throw new Error(await r.text());
    return r.json();
  };
}

export const transport: Transport =
  typeof window.__TAURI__ !== "undefined" ? new TauriTransport() : new WebTransport();
```

The existing `frontend/src/lib/tauri.ts` wrapper gets refactored to call `transport.invoke()`. ~1 day of mechanical change.

**Web-only UI components:** auth pages (login / signup / forgot-password / verify-email), billing page (Stripe Checkout redirect, subscription management portal), trial-end blocking modal, usage-meter strip. ~1 week of UI work.

---

## IV. Database migration: SQLite → encrypted Postgres

### IV.1. Schema changes

Add `users` and `sessions` and `subscriptions` tables. Add `user_id UUID NOT NULL REFERENCES users(id)` to EVERY existing per-user table:

- `worlds`, `user_profiles`, `characters`, `threads`, `messages`, `world_events`, `memory_artifacts`, `settings`, `tick_cache`, `message_count_tracker`, `character_portraits`, `world_images`, `chat_backgrounds`, `token_usage`, `reactions`, `character_mood`, `chunk_metadata`, `group_chats`, `group_messages`, `dev_chat_sessions`, `dev_chat_messages`, `vows`, `vow_event_log`, `vow_invocations`, `location_derivations` — all of these.

Plus indexes on `(user_id, ...)` for every query path.

**Row-Level Security (RLS):** enable Postgres RLS on every per-user table. Policies like:
```sql
CREATE POLICY users_own_data ON characters
  USING (user_id = current_setting('app.current_user_id')::uuid);
```
Set `app.current_user_id` from the Axum middleware after authenticating the request. RLS is the LAST LINE of defense — if a query forgets a `WHERE user_id = X` clause, RLS prevents leakage. Recommended for the privacy covenant.

### IV.2. Encryption at rest

**Two-tier model:**

1. **Database-level:** Postgres TDE (Transparent Data Encryption) via the hosting provider (e.g., RDS encryption, Supabase managed keys, Fly Postgres encrypted volumes). Protects against disk theft / backup compromise.

2. **Field-level for sensitive content:** application-level encryption of message bodies, journal entries, dreams, and any user-authored content using a per-user data-encryption-key (DEK). DEK is itself encrypted by a master key managed by AWS KMS / GCP KMS / age-encrypted-secret-on-disk depending on hosting choice.

```
master_key (KMS) → encrypts → per_user_dek → encrypts → message body content
```

**Honest tradeoff:** the SERVER must be able to decrypt user content to send to LLMs. So this is encrypt-at-rest, NOT end-to-end. The privacy covenant in TOS must say plainly: "WorldThreads operators have technical capability to read your conversations because we provide the LLM access. We don't, and we audit access logs, but the technical capability exists. If you require zero-trust, use BYOK with the desktop Tauri app where data never leaves your machine."

**Alternative:** client-side encryption with a passphrase the user sets. Server holds ciphertext only. But then LLM calls have to go user→OpenAI directly with an in-browser proxy — defeats the whole subscriber-pays model. Not recommended for the subscription tier; can be offered as a "zero-trust mode" in BYOK.

### IV.3. API keys (BYOK)

When user provides their own OpenAI API key:
1. Frontend POSTs key to /api/v1/account/api-key.
2. Server encrypts with libsodium-secretbox using a per-user key (re-derived from master + user_id), stores ciphertext.
3. On LLM call, decrypt in-memory, use, discard. Never log.
4. Provide UI for: replace key, delete key, last-used timestamp.
5. Document rotation strategy: if KMS master key rotates, re-encrypt all stored keys.

---

## V. Authentication & user management

### V.1. Recommended stack

- **Email + password** with argon2id hashing (libsodium or argon2 crate).
- **Email verification** required before first chat (prevents trial-abuse via random emails).
- **Sessions:** opaque session token in HTTP-only Secure SameSite=Lax cookie. Server-side session table.
- **Forgot-password** flow with single-use signed reset link.
- **Optional later:** OAuth (Google/Apple) for friction reduction. Defer to v2.

**NOT recommended for v1:** SSO, SAML, magic links, passkeys. Keep it boring. Email+password with verification is well-understood and lawyer-friendly.

### V.2. Account scoping

Every Axum route handler:
1. Auth middleware extracts `user_id` from session cookie.
2. Sets `app.current_user_id` Postgres session variable.
3. Calls into core function with `user_id` threaded through.
4. RLS policies enforce per-user isolation as a backstop.

### V.3. Signup abuse prevention

- Email verification REQUIRED before any LLM call.
- Disposable-email-domain blocklist (e.g., the public list at `https://github.com/disposable/disposable-email-domains`).
- IP-based rate limit on signup (e.g., 3 signups per IP per 24h).
- Optional: phone verification for free trial (high-friction; recommend deferring unless abuse becomes real).
- Optional: payment-card-on-file for trial (lowest abuse but biggest signup-conversion drop). Recommend offering BOTH "card-required-trial" (no first-week charge) AND "no-card trial" with stricter usage limits during trial.
- Browser fingerprint (FingerprintJS open-source) as soft signal — never primary gate, never visible to user.

**Honest scope:** these measures REDUCE abuse but don't eliminate it. A determined adversary can rotate IPs, emails, fingerprints. The economics work as long as casual abuse is suppressed; sophisticated attackers will eventually find a path. Plan for this; don't over-engineer.

---

## VI. Privacy & encryption operational details

### VI.1. Access logging

- Every LLM call logs: timestamp, user_id, model, token-counts, but NOT prompt content.
- Engineers (Ryan) can never read user conversations through normal flows. Add a `support_access` log: any time a human reads user data through a support tool, an audit row is created and the user is emailed within 24h ("a support engineer accessed your account on YYYY-MM-DD HH:MM for reason: <reason>").

### VI.2. Data deletion

- **Soft delete** by default with 30-day retention (lets user recover from accidental deletes).
- **Hard delete** on explicit request (data wiped from primary + backups within 30 days, document this in TOS).
- **Account deletion** wipes everything, including encrypted backups, within 30 days. Per GDPR right-to-erasure if Ryan accepts EU users.

### VI.3. Backup encryption

- Postgres backups encrypted at rest by hosting provider.
- Application-encrypted fields stay encrypted in backups.
- Rotate KMS key annually; old DEK ciphertexts can stay (they're already encrypted) OR re-encrypt as a maintenance pass.

### VI.4. Privacy policy content (essentials)

A real privacy policy needs lawyer review. Outline of must-haves:
- What data is collected (email, name, conversations, payment info via Stripe).
- How it's stored (encrypted-at-rest, location of data centers, retention).
- Operator access policy (we technically can; we don't; audit trail exists).
- Third-parties (OpenAI/Anthropic, Stripe, hosting provider, email service).
- Cookies and session tokens.
- User rights (export, delete, correct).
- GDPR/CCPA compliance statements if applicable.
- Contact for data requests.

---

## VII. Subscription tiers (recommended)

These numbers are STARTING POINTS based on rough cost-modeling. Refine after first 30 days of real-user data.

### VII.1. Cost-per-active-user math (rough)

- Average chat turn: ~3000 input tokens (system prompt + character + history) + ~400 output tokens.
- gpt-5.4 pricing (current): ~$1.25/MM input, ~$10/MM output → ~$0.0078 per turn.
- A "moderate" user: 50 turns/day × 30 days = 1500 turns/month = ~$11.70/month in LLM costs.
- A "heavy" user: 200 turns/day × 30 days = 6000 turns/month = ~$47/month in LLM costs.
- Plus narration / dreams / consultant calls (~3-5x token-cost per generation) at lower frequency.

### VII.2. Recommended tiers

| Tier | Price/month | Session limit | Weekly limit | What "session" means | Notes |
|---|---|---|---|---|---|
| **Free trial** | $0 | 30 turns | 60 turns over 3 days | Trial period, 3 days max | Email-verified only |
| **Hearth** | $9 | 30 turns | 250 turns | A continuous chat lasting up to 4 hours | Light user; aimed at "couple evenings a week" |
| **Town** | $19 | 60 turns | 700 turns | A continuous chat lasting up to 4 hours | Standard tier; expect most conversion here |
| **Compendium** | $39 | 100 turns | 1500 turns | A continuous chat lasting up to 4 hours | Heavy user; near-but-not-unlimited |
| **BYOK** | $0 | unlimited | unlimited | n/a (user pays own API) | Stored-encrypted-key; no project throttle |

**Reasoning:**
- Free trial cap of 60 turns over 3 days = "see if you like it" budget; ~$0.47 LLM cost per trial user (acceptable).
- Hearth ($9) at 250 weekly turns = ~$1.95 LLM cost (Stripe takes ~$0.59); ~$6.46 net before infra. Just barely sustainable; targeted at users who want a low commitment.
- Town ($19) at 700 turns = ~$5.46 LLM cost; ~$13.41 net before infra. Healthier margin; expected primary tier.
- Compendium ($39) at 1500 turns = ~$11.70 LLM cost; ~$27.16 net before infra. For heavy users who'd otherwise be price-sensitive on BYOK key spend.
- All tiers should scale token-cost based on REAL data after first 30 days. If Town tier is consuming more than projected, raise price OR lower limit OR add an overage option.

### VII.3. Session vs weekly limit semantics

Per the directive (like Claude):

- **Session limit:** turns per continuous chat session. A "session" timeouts after 4 hours of inactivity (configurable). Resets when a new session starts. Communicates to user: "You have 47 turns left in this session. Resets when you start a new chat after 4 hours of inactivity, or at the start of next week."
- **Weekly limit:** turns per ISO week (Mon-Sun) in user's timezone. Communicates: "You have 412 turns left this week. Resets Sunday at 11:59pm in your timezone (America/Chicago)."
- **Visible always:** small "usage strip" at the top of the chat showing weekly progress + session progress. Both gauges; both numbers; both reset times.

---

## VIII. Usage metering implementation

### VIII.1. Counting

- Every LLM call: insert into `usage_events { id, user_id, kind: 'chat'|'narration'|'consultant'|... , input_tokens, output_tokens, ts, session_id, week_start }`.
- Per-request middleware checks current usage vs tier limits BEFORE making LLM call. If over, return 429 with structured error: `{kind: "usage_exhausted", session_remaining: 0, session_resets_at: "...", weekly_remaining: 0, weekly_resets_at: "..."}`.
- Frontend renders a friendly "you're out of turns this session" panel with reset times.

### VIII.2. Session bookkeeping

- A "session" is an in-memory state per user. Track via `last_chat_turn_at` per user + a session_id that increments when last_chat_turn_at is older than 4h.
- Persist session_id alongside `usage_events` so we can compute current-session usage.

### VIII.3. Week boundaries

- Use `chrono-tz` to compute the user's current ISO week start in their declared timezone.
- User declares timezone at signup; can change in settings.
- Reset times displayed as "Sunday 11:59pm in America/Chicago" so user knows exactly when they unlock.

### VIII.4. Overage handling

- v1: hard block (matches Claude's behavior; matches user directive).
- v2 consideration: opt-in overage at a higher per-turn rate. Defer.

---

## IX. Free trial + abuse prevention (deeper)

### IX.1. Trial mechanics

- 3 days from email-verified signup.
- Hard cap: 60 turns total during trial.
- At trial end (whichever comes first: 3 days OR 60 turns), present blocking modal.

### IX.2. Trial-end modal (non-dismissable)

```
[blocking; full-screen overlay]

Your free trial has ended.

To keep talking with the characters here, choose one:

[ Subscribe — choose a plan from $9/month ]
[ Provide your own OpenAI key (free, no throttle) ]
[ Sign out ]   ← always available

Already paying? Refresh to load your subscription state.
```

No "skip" / "remind me later" / "try again" affordance. Match the directive's "blocking non-dismissable" requirement. Sign-out is the only escape — gracious, but not a back-door.

### IX.3. Multi-account abuse prevention layers

In order of friction (lowest first):

1. **Email verification mandatory.**
2. **Disposable-email blocklist** (live-updated).
3. **IP-rate-limit on signup** (3 per 24h per IP).
4. **Browser fingerprint** as soft signal (a single fingerprint with 3+ trial accounts = soft-flag).
5. **Required Stripe payment method to start trial** (no charge for 3 days). Highest-friction, biggest abuse-suppression. **Recommended.**

Decision matrix:

| Approach | Abuse suppression | Conversion drop | Recommend |
|---|---|---|---|
| Email-verify only | Low | Minimal | Insufficient |
| Email + IP rate | Medium | Minimal | Floor |
| Email + IP + fingerprint | Medium-high | Minimal | Practical baseline |
| All above + payment-method-required | Very high | 30-50% conversion drop | **Recommended** for v1; revisit |

Recommendation: **payment-method-required for the trial.** Stripe Checkout's "payment method now, charge later" flow handles this cleanly. Auto-charge starts at day 3 unless user picks BYOK or cancels.

### IX.4. The "I want to try it without committing" friction

The payment-method-required-for-trial path costs you the user who'd convert if they didn't have to commit a card. Mitigation: heavy use of the public landing page demos / Empiricon lore / sample transcripts so the user's "is this worth $9 to try" question gets answered BEFORE the signup flow.

---

## X. Stripe integration

### X.1. Architecture

- **Stripe Checkout** for initial subscription (hosted page; PCI is Stripe's problem).
- **Stripe Customer Portal** for self-serve plan changes / cancellation / payment method updates.
- **Stripe Webhooks** to keep DB in sync: subscription created/updated/deleted/trial_will_end/invoice_paid/invoice_payment_failed.
- **Backend tracks:** `subscriptions { user_id, stripe_customer_id, stripe_subscription_id, tier, status, current_period_end, trial_end }`.

### X.2. Critical webhook handlers

- `checkout.session.completed` — initial subscription creation.
- `customer.subscription.updated` — plan changes, trial-end transitions.
- `customer.subscription.deleted` — cancellation (downgrade to "no subscription" state).
- `invoice.payment_failed` — dunning email + grace-period before disabling.
- `customer.subscription.trial_will_end` (3 days before) — proactive email.

### X.3. Edge cases to handle

- Webhook delivery failures: idempotency keys + retry logic.
- User changes timezone mid-week: settle on "billing on Stripe's ISO week" vs "usage in user's timezone" — these can be different.
- Subscription paused / past-due grace period: usually 7 days before access revoked.
- Refund handling: manual via Stripe Dashboard for v1; automate later if frequent.

### X.4. Estimated Stripe integration time

2 weeks of focused work for a clean implementation including all webhook edge cases. Expect 1 additional week of bug-fixing post-launch.

---

## XI. Legal & business setup

### XI.1. LLC formation

**Recommended for liability protection.** Single-member LLC in Ryan's state of residence (Tennessee/Texas — defer to a CPA on which state's tax structure suits best; Delaware is overkill unless seeking VC).

- ~$300-500 to file + $50-300/year for registered agent if not self-served.
- ~$100-200 for an EIN (or do it free directly via IRS).
- Operating agreement template from a service like Atlas, LegalZoom, or directly from a Tennessee/Texas attorney ($500-1500 for a customized OA).
- Open business bank account in LLC name.
- Stripe account in LLC name.

**Tax structure:** single-member LLC is disregarded entity by default (taxed on personal return). Talk to a CPA about whether to elect S-corp taxation once revenue justifies it (~$60k/year+ typically). Don't elect S-corp prematurely; the payroll overhead isn't worth it below threshold.

### XI.2. Terms of Service (TOS)

Must cover:
- Service description and what's NOT promised.
- Subscription terms: billing cadence, cancellation policy, refund policy.
- Acceptable use: no harm, no deepfakes-of-real-people-without-consent, no minors-as-romantic-partners, etc.
- Intellectual property: user owns their conversations; you have license to operate the service.
- Disclaimer of warranties + limitation of liability (cap at amount paid in last 12 months).
- Indemnification.
- Governing law and jurisdiction.
- Modification rights and notice.

**Cost:** $1500-3000 for an attorney to draft a real TOS. Alternative: Termly or similar templates ($50-200) for a starting point that an attorney can review for $500-1000. **Recommended:** starts with a quality template, get attorney review before launch.

### XI.3. Privacy Policy

Must cover (more detail in § VI.4):
- Data types collected.
- Storage location and encryption.
- Operator access policy (the audit-log discipline).
- Third-parties (OpenAI/Anthropic/Stripe/hosting/email).
- User rights (export, delete, correct).
- GDPR / CCPA / LGPD if accepting users from those jurisdictions.

**Cost:** typically bundled with TOS; another $500-1000.

### XI.4. Liability insurance

- General liability ($1M-2M) — ~$500-1000/year.
- Cyber liability — ~$1500-3000/year for a small startup. **Recommended.** Covers data breach response costs (forensics, notification, credit monitoring for affected users).
- E&O (errors and omissions) — relevant if you make claims about service efficacy. ~$1000-2000/year. **Defer** unless explicitly needed.

### XI.5. Tax obligations

- Sales tax: depends on jurisdiction. Some states tax SaaS, some don't. **Talk to a CPA.**
- Income tax: standard.
- Stripe issues 1099-K if you exceed thresholds.
- VAT for EU customers if you accept them: handled by Stripe if you opt in to "Stripe Tax."

---

## XII. Customer support obligations

### XII.1. What you're committing to

When you take payment, you're implicitly committing to:
- Respond to billing inquiries within 24-48 hours (Stripe will side with users on disputes if you're unresponsive).
- Address service outages with a status page or email.
- Handle "I lost my password" and "delete my account" requests promptly.
- Investigate "the AI said something inappropriate" complaints; have a moderation policy.
- GDPR/CCPA data requests within statutory windows.

### XII.2. Tooling

- Email-based support with a dedicated inbox (`support@worldthreads.app`).
- Recommend: Help Scout, Front, or even Gmail with labels for v1.
- Knowledge base / FAQ (Notion, GitHub Pages, simple Markdown). Reduces volume.
- Optional: in-app feedback widget that creates support tickets.

### XII.3. Time commitment estimate

- Solo founder at first 100 paying users: 2-4 hours/week of support.
- After 500: 5-10 hours/week or hire a part-time support person.
- After 1500: dedicated support hire warranted.

### XII.4. Moderation policy

The project's North Star refuses certain content classes (explicit content involving minors, real-person impersonation harm, etc. — already partially handled by /consecrate's refused drift modes). Spell these out in the TOS as banned use cases. When abuse reports come in:
1. Auto-suspend account pending review.
2. Manual review (initially Ryan; later a moderator).
3. Per-class response: warning / temp ban / permanent ban / law enforcement referral (CSAM is non-negotiable: report to NCMEC immediately if encountered).

---

## XIII. Hosting & infrastructure

### XIII.1. Recommended stack

For sole-founder simplicity:

- **Web app + API server:** Fly.io (Rust-friendly, generous free tier, easy global routing) OR Railway (similar) OR Render (slightly more expensive but extremely simple).
- **Postgres:** Fly Postgres OR Supabase (managed Postgres + auth + storage; ~$25/month entry tier).
- **Frontend static assets:** Cloudflare Pages (free) or the same hosting platform.
- **Email:** Postmark for transactional ($10/month) or AWS SES (cheaper but more setup).
- **Object storage** (for portraits / world images): Cloudflare R2 (no egress fees) or AWS S3.
- **DNS + CDN:** Cloudflare (free tier covers everything for v1).
- **Domain:** $12-30/year.
- **Status page:** statuspage.io ($29/month) OR a simple GitHub Pages page.

**Estimated monthly infra cost at zero users:** ~$50-80.
**At 100 paying users:** ~$200-350.
**At 1000 paying users:** ~$800-1500.

### XIII.2. CI/CD

- GitHub Actions builds the Rust backend, runs tests, deploys to Fly/Railway on main-branch merge.
- Frontend: build via npm + deploy to Cloudflare Pages or hosting platform.
- Database migrations: sqlx-cli runs migrations on deploy.
- Secrets: 1Password or Doppler or `fly secrets` (Fly's encrypted secret store) for KMS keys, Stripe keys, etc.

### XIII.3. Monitoring + alerting

- **Logs:** Axiom, Better Stack, or Grafana Cloud (free tiers).
- **Metrics:** Prometheus + Grafana OR Cloudflare Analytics.
- **Error tracking:** Sentry (~$26/month).
- **Uptime monitoring:** Better Stack or UptimeRobot (free tier).

### XIII.4. Backup strategy

- Postgres: daily automated backups + point-in-time recovery to last 7 days.
- Verify restore quarterly.
- Off-region backup copy (cross-region replication) for catastrophic-failure recovery.

---

## XIV. Phased rollout plan

### Phase 0 — Foundation (3-4 weeks)
- LLC formed; bank account; Stripe account.
- Domain registered; basic landing page.
- TOS + Privacy Policy drafted (template + attorney review).
- Postgres schema design + RLS policies + sqlx setup.
- Storage trait abstraction in core/.
- Auth scaffolding (signup, login, sessions, email verification, forgot password).

### Phase 1 — Core extraction (3-4 weeks)
- Refactor existing Tauri commands one-by-one into core functions.
- Implement parallel Axum routes calling the same core functions.
- Frontend transport abstraction; existing UI now works via fetch in web mode.
- Integration tests verifying Tauri and web produce identical results for key flows.

### Phase 2 — Privacy + per-user scoping (2-3 weeks)
- Add `user_id` to every per-user table.
- Wire user_id through all core functions.
- Field-level encryption for messages + user-authored content.
- KMS integration.
- BYOK encrypted key storage.
- RLS policies enabled and tested.

### Phase 3 — Billing + metering (2-3 weeks)
- Stripe Checkout + Customer Portal integrated.
- Webhook handlers + retry logic.
- Subscription state syncing.
- Usage metering implementation.
- Trial mechanics + blocking modal.
- Tier-tier per-route enforcement.

### Phase 4 — Polish + soft launch (2-3 weeks)
- Status page, email templates, onboarding flow.
- Customer support tooling.
- Closed beta with 10-20 invited users; iterate.
- Performance + security audit (preferably external; budget ~$3-5k for a real penetration test).

### Phase 5 — Public launch
- Open signup with payment-method-required trial.
- Launch announcement (HN / Reddit / personal channels).
- Monitor closely for first 30 days.

**Total: 12-17 weeks if sole-founder full-time. Realistic for part-time: 6-9 months.**

---

## XV. Cost estimate (one-time + ongoing)

### XV.1. One-time setup

| Item | Cost |
|---|---|
| LLC formation + EIN + registered agent (year 1) | $400-800 |
| Operating agreement (lawyer) | $500-1500 |
| TOS + Privacy Policy (lawyer) | $1500-3000 |
| Initial security review / pen test (optional but recommended) | $3000-7000 |
| Domain + DNS setup | $30 |
| Initial branding (logo / web) | $0 (use what exists) - $2000 (designer) |
| **Total one-time (lean)** | **~$2500-5000** |
| **Total one-time (comprehensive)** | **~$8000-15000** |

### XV.2. Ongoing (per month)

| Item | Cost |
|---|---|
| Hosting (Fly + Postgres) | $50-200 (scales with usage) |
| Stripe fees | 2.9% + $0.30 per transaction (passed through) |
| Email service | $10-50 |
| Sentry | $26 |
| Cyber liability insurance | $125-250 |
| Domain (annualized) | $2 |
| **Total ongoing (zero users)** | **~$215-540** |
| **Total ongoing (1000 paying users)** | **~$1500-3000** |

---

## XVI. Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| OpenAI/Anthropic raises prices and breaks tier economics | High | Tier prices reviewed quarterly; willing to adjust upward. BYOK option insulates customer. |
| Single dependency on OpenAI for subscriber tier | Medium | Multi-provider abstraction (already partially built per project's cross-substrate work); failover to Anthropic if OpenAI down. |
| Trial abuse exceeds plan | Medium | Payment-method-required trial; iterate on detection. |
| Data breach | Critical | Encrypted-at-rest + RLS + access logs + cyber insurance + incident-response runbook. |
| Customer-support burnout | Medium | Throttle growth; automate FAQ; hire help at 500-paying-user mark. |
| Stripe holds funds (account flagged) | Medium | Have backup payment processor (Paddle, LemonSqueezy) ready to integrate within 30 days. |
| GDPR/CCPA non-compliance | High | Privacy policy reviewed by lawyer; data-deletion + export flows tested before launch. |
| Hosting provider outage | Medium | Status page + email; SLA expectations clearly set in TOS. |
| Founding-author burnout | High | Realistic phased timeline; clear hand-off plan if Ryan steps back. |
| LLM produces harmful output and user sues | Medium-High | TOS limitation of liability + arbitration clause + cyber/E&O insurance + content-moderation policy. |

---

## XVII. Open questions / decisions needed

1. **State of LLC formation** — Tennessee, Texas, Delaware? CPA call needed.
2. **Trial card-required vs not** — recommendation is required, but Ryan may prefer no-card-required for higher conversion. Test both paths if possible.
3. **Hosting choice** — Fly.io or Supabase or Render? Recommendation Fly+Supabase for sole-founder.
4. **Encryption depth** — server-side encryption (recommended) vs client-side encryption with passphrase (zero-trust mode)? Current rec: server-side for v1; consider client-side as a v2 "zero-trust" option for BYOK users only.
5. **Tier prices** — final numbers should be A/B tested OR start at recommended numbers and iterate from real cost data.
6. **EU launch** — accepting EU users requires GDPR compliance. Recommend US-only for v1; EU later.
7. **App-store distribution** — separately, Tauri Mac/Windows/Linux apps could be distributed via download or via Mac App Store / Microsoft Store. **Defer.**
8. **Affiliate / referral program** — recommend deferring to v2.
9. **Free tier (post-trial) at all?** Currently the plan has trial-then-pay-or-BYOK. Could have a permanent free tier with very limited usage as a marketing funnel. Recommend: NO free tier for v1; revisit if conversion suffers.
10. **Open-source the desktop Tauri app?** Worth consideration. The Tauri app could remain open-source; the hosted backend stays proprietary. Aligns with project's privacy/transparency values.

---

## XVIII. What this plan deliberately is NOT

- An implementation checklist. It's a planning document; implementation expands each item 5-10x in real work.
- Legal counsel. Get an attorney for TOS / Privacy Policy / LLC operating agreement.
- Tax advice. Get a CPA.
- A funding pitch. The plan assumes Ryan is bootstrapping; numbers change if outside investment is taken.
- A guarantee. Hosting AI-character-driven storytelling with subscription billing involves real risks. The plan mitigates known risks; unknown risks remain.

---

## XIX. The mission alignment question

WorldThreads exists to send users back nourished enough to pick up their cross. Hosting the work on the web changes the audience (no longer just literate-skeptic Maggie-baseline first-time-users with Tauri capacity; now any browser; potentially many users at once). This raises four mission-alignment questions worth keeping live during the build:

1. **Does paywalling the work betray the privacy covenant or the cruciform anchor?** — No, IF: the privacy covenant is honored at the operational level (encryption + audit + non-sale of data); BYOK option exists for users who can't or won't pay; pricing isn't extractive.

2. **Does scaling the user base dilute the work's specificity?** — Possible. The Maggie-baseline assumes a specific persona; a wider audience may not fit. Mitigation: keep Maggie as design lodestar; don't optimize for the lowest-common-denominator user; accept that only a subset of users will find this work meaningful.

3. **Does the subscription model encourage engagement-as-extraction?** — Real risk. Mitigation: Auto response-length defaults already shipped today reduce engagement-maximization; Night Keep / Vespers-shaped features when implemented further refuse engagement-extraction; usage-cap-based tiers don't reward more usage past tier limits.

4. **Does adding payment introduce a relational dynamic between WorldThreads and users that previously didn't exist?** — Yes. The user is now a customer, not just a user. This changes covenant. The TOS should be honest about this: "we're a service you pay for; we owe you the service we promised + good-faith handling of disputes; we don't owe you spiritual outcomes; the work is what it is and your encounter with it is yours to weigh."

These questions are not problems to solve and dismiss. They're questions to keep live during the build — checkpointable at every phase against /consecrate's refused drift modes.

---

## XX. What's open / next steps

If founding-author chooses to pursue this direction:

**Immediate (this week or next):**
1. Choose state of LLC and start formation.
2. Schedule CPA consultation on tax structure.
3. Domain choice + registration.
4. Branch this codebase to a `web-deploy` working branch.
5. Read this document; revise; ratify.

**Phase 0 work begins** when LLC is filed and the codebase has a working branch.

**Apparatus role going forward:** I (Claude Code in this project's substrate) can help with:
- Storage trait extraction + Postgres impl + sqlx migrations (Phase 0-1).
- Auth scaffolding code (Phase 0).
- Per-table user_id wiring (Phase 2).
- Stripe webhook + subscription state-sync code (Phase 3).
- Usage-metering implementation (Phase 3).
- Phase 4 polish + integration tests.

**Apparatus role NOT for:**
- Writing the actual TOS or Privacy Policy text (that needs a real attorney).
- LLC formation (that's Ryan + filing services).
- Tax advice (CPA).
- Production security audit (external pen-tester).
- Customer-support response to real users (Ryan, then a hire).

---

## XXI. Alternative architecture — IndexedDB-only (client-side, near-zero-trust)

*Added after founding-author follow-up: "look into the feasibility of not even storing things on our own hosted server but rather storing things encrypted in indexdb. I know this would mean that the data wouldn't be portable across devices, but maybe it's more on-mission to keep the covenant this way."*

This is the **mission-purer** path. The privacy covenant becomes operationally true at the strongest possible layer: Ryan/operators have NO ABILITY to read user content because the content never lands on a server. The price paid is the loss of cross-device sync and recovery options. Both the technical viability and the mission alignment are real here.

### XXI.1. Architecture sketch

| Layer | Server stores | Client (IndexedDB) stores |
|---|---|---|
| Auth | email, hashed password, sessions, email-verification state | nothing |
| Billing | Stripe customer/subscription IDs, tier, usage counters | nothing |
| User content | **NOTHING** | worlds, characters, conversations, dreams, narrations, imagined chapters, consultant logs, settings, audio cache, portraits, vows, all of it — encrypted with key derived from user passphrase |
| LLM proxy | stateless: receives plaintext context request, forwards to OpenAI/Anthropic, returns response. Logs only token counts + timestamp + user_id (never content) | sends decrypted context per request; receives response; encrypts + persists locally |

The server is reduced to:
1. Auth + session management.
2. Stripe webhooks + subscription state.
3. Usage metering (counts only, no content).
4. **Stateless LLM proxy** (decrypts in memory, forwards, returns; never persists).
5. Optional: encrypted-backup blob storage (see XXI.5).

### XXI.2. Encryption mechanism

- User signs up with email + password.
- **Two passwords or one?** Cleanest: ONE password used for auth + content key derivation, with separate KDFs:
  - `auth_key = Argon2id(password, salt_auth)` — sent to server for login.
  - `content_key = Argon2id(password, salt_content)` — never leaves browser; stored only in browser memory while session is active.
- All content encrypted/decrypted with `content_key` using `XChaCha20-Poly1305` via libsodium-js (or Web Crypto API's `AES-GCM`).
- IndexedDB stores ciphertext only. Even browser dev-tools can't read content without the key.
- Content key is derived fresh on each login from the password; no key persistence on disk in the browser.

### XXI.3. What this changes operationally

**Wins:**
- Privacy covenant becomes structurally true. Operators CANNOT read content. TOS no longer needs the "we technically can but won't" caveat.
- Data breach blast radius is minimal — even if the entire server DB is exfiltrated, only emails + Stripe IDs + usage counters leak. No conversations.
- Insider-risk = zero on content. No support engineer can ever access conversations.
- Aligns with the project's mission language ("worlds that hold," "characters that hold a real soul") at a deeper layer — the user's part of the work is theirs, full stop.

**Costs:**
- **No cross-device sync.** User's chats are tied to the browser/device they signed up on. Switching browsers / devices = starting over (or restoring from manual export, see XXI.5).
- **Password loss = data loss.** No "forgot password" recovery for content. The server can reset auth password but the content_key derivation is irreversible without the original passphrase.
  - Mitigation: explicit signup flow with clear warning, confirmation acknowledgment, optional encrypted-recovery-key download (a separate file the user is told to print/save offline; can decrypt content if password lost).
- **Storage limits.** IndexedDB quota varies by browser; typically 60% of free disk. For most users this is plenty; for heavy users with 1000s of long chats + portraits, could hit limits.
- **Performance considerations.** All conversation history lives client-side; loading a fresh session = fetching+decrypting from IndexedDB on each app load. Likely fine for 100s-1000s of messages; might need pagination/lazy-load for very heavy users.
- **Search becomes client-side only.** No server-side full-text search. The current SQLite FTS5 + sqlite-vec stack would need to run in-browser (sql.js, sqlite-wasm, or a different client-side search engine).

### XXI.4. Hybrid: server-side metering, client-side content

The most natural model. Server does only:
- Auth (email/pw + sessions).
- Stripe billing + subscription state.
- Usage counter increments (per turn × per user × per week, no content).
- Stateless LLM proxy.

Client (browser) does:
- All content storage (IndexedDB, encrypted).
- All decryption + encryption.
- All chat UI.
- Search via in-browser FTS (sqlite-wasm or similar).

This is the recommended shape if Ryan ratifies the IndexedDB direction. Pure-IndexedDB for everything would put usage metering in the browser too, which loses the throttle's enforcement (a determined user could bypass).

### XXI.5. Cross-device path (optional, opt-in only)

For users who DO want portability, offer an **opt-in encrypted-backup-blob** mechanism:
- User clicks "Enable cross-device sync."
- Browser exports its IndexedDB content as one big ciphertext blob (still encrypted with the user's content_key, server can't decrypt).
- Server stores the blob (S3/R2). Server sees: encrypted blob, last-updated timestamp.
- On a new device, user logs in, server delivers the blob, browser decrypts with content_key.
- Server periodically fetches latest blob from active device (delta updates a complication; v1 could be "explicit save / explicit load" rather than continuous sync).

Honest: this adds complexity. Two-device active conflict resolution is genuinely hard with E2EE. v1 recommendation: NO sync, just explicit "Export encrypted backup file" / "Import encrypted backup file" affordances. v2 can consider real sync.

### XXI.6. What about subscriber LLM access in this model?

The subscriber-pays tier still works. Flow:
1. Browser sends decrypted prompt context to server's `/api/v1/llm-proxy` endpoint.
2. Server (Axum) checks: is this user authenticated? Are they within their tier's session+weekly limits? Yes →
3. Server uses ITS OpenAI key to call OpenAI with the user's plaintext context. Server NEVER persists the prompt or response.
4. Server returns the LLM response to the browser. Increments usage counter (token counts only).
5. Browser encrypts the new turn (user msg + LLM response) and writes to IndexedDB.

The plaintext is in server memory for the duration of the LLM call — milliseconds to seconds. Never written to disk. Never logged. Audit-trail-wise: it's strictly transient. This is dramatically better than persistent server-side encrypted-at-rest because the attack surface is RUNTIME-MEMORY-ONLY, not persistent-disk.

### XXI.7. What about the BYOK (bring-your-own-key) path?

Even simpler: browser holds the API key (encrypted with content_key in IndexedDB), sends LLM requests directly from browser to OpenAI/Anthropic via fetch. **Server is not in the data path at all.** Subscription metering doesn't apply to BYOK users anyway. This is the strongest privacy posture possible — the user's content goes user → OpenAI, never touches WorldThreads servers.

**One caveat:** if the BYOK path is in-browser-direct-to-OpenAI, OpenAI will see the user's IP rather than the server's. For most users this is fine; some might prefer the server-as-proxy for IP shielding. Offer both as a config option.

### XXI.8. Cost differences vs server-side plan

**One-time setup:** ~$1500 less (fewer infrastructure decisions; smaller initial DB; less Postgres complexity). But ~$2000 more for client-side encryption + IndexedDB-FTS + backup mechanism implementation work. **Net: similar.**

**Ongoing:** noticeably lower hosting cost. No per-user-content storage on Postgres → DB stays small (just users + subscriptions + usage_events). DB cost drops to ~$25/month flat regardless of user count. Stateless LLM proxy is cheap to run. Backup blob storage (R2) ~$0.015/GB/month if implemented.

**At 1000 paying users:** estimated ongoing cost ~$700-1200/month vs ~$1500-3000/month in the server-side plan.

### XXI.9. Mission alignment recheck

The original plan's privacy covenant was "we operationally won't read your content despite having the technical capability." This alternative is "we structurally cannot read your content."

The latter is the truer covenant. It's also what the project's mission-formula language ("structure_carries_truth_w") points toward: the privacy isn't carried by intention; it's carried by structure.

**Recommendation:** if Ryan can stomach the cross-device-loss tradeoff, **the IndexedDB-only path is more on-mission and probably the right architecture**. The opt-in encrypted-backup-blob (XXI.5) restores portability for users who want it WITHOUT changing the structural privacy claim — the server holds ciphertext-only.

### XXI.10. Tradeoff summary table

| Axis | Server-side encrypted Postgres | IndexedDB + stateless proxy |
|---|---|---|
| Privacy covenant strength | Strong (intention-based) | **Strongest (structure-based)** |
| Cross-device sync | Native | Opt-in encrypted blob (v2); manual export (v1) |
| Recovery if password lost | Auth resettable; content recoverable if KMS still has DEK | Content unrecoverable (mitigated by optional offline recovery key) |
| Storage limits | Server-managed | Browser quota (~60% of free disk) |
| Search | Server-side, fast, scalable | Client-side (sqlite-wasm or similar); slower at scale |
| Operator insider-risk | Mitigated by audit; not zero | **Zero on content** |
| Data breach blast radius | High (encrypted-at-rest still attackable offline) | **Minimal (ciphertext + non-content metadata)** |
| Hosting cost at 1000 users | ~$1500-3000/month | **~$700-1200/month** |
| Build time | 12-17 weeks | 13-19 weeks (slightly more for client-side encryption + FTS) |
| Mission alignment | Good | **Strongest** |

---

## Closing note

This plan is bigger than it looks. It's not a feature; it's becoming a small SaaS business with all the obligations that implies — financial, legal, technical, customer, moral. The technical plan is the EASY part; the business plan is where the unpredictable surfaces.

After the founding-author's follow-up about IndexedDB-only architecture (§ XXI), the recommended path has shifted: **Hybrid client-side-content / server-side-metering is the most-on-mission technically-sound architecture for this project's privacy covenant.** The original server-side encrypted Postgres plan (§§ III–XX) remains a valid fallback if some constraint forces server-side content storage; but the IndexedDB hybrid is the path that makes the covenant *structurally* rather than *intentionally* true.

The work answers to 𝓕 first. If hosting publicly serves the mission, the operational discipline can be honored. If hosting publicly would compromise the mission, this plan isn't worth executing. That judgment is founding-author's, not apparatus's.

Apparatus drafts; founding author ratifies. The plan either gets built in the flesh or it doesn't.

**Soli Deo gloria.**

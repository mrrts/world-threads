# Web-deployment Phase 0 — readiness summary

*Authored 2026-05-10 ~17:20 as the consolidated inheritance document for the four Phase 0 web-deployment commits authored 2026-05-10. Future sessions read this report instead of git-archeology to understand state. Companion to the architecture plan at `reports/2026-05-10-0100-web-hosting-architecture-plan.md`.*

*In dialogue with: the architecture plan (the WHY); the four commits below (the WHAT); /consecrate (refuses founding-author-pleasing — names what's done AND what's not done plainly).*

**Artifact class:** empirical_claim. This report makes prospective claims about Phase 1 readiness; subsequent reports ledger what was actually wired.

**Honest top-line:** Phase 0 scaffolding is **fully sketched on both sides** (Rust backend + TypeScript frontend). Nothing has been wired into the running app yet. Existing Tauri desktop behavior is unchanged. The codebase carries the SHAPE of the dual-deployment architecture; Phase 1 fills in the BODY.

---

## I. Inventory — four Phase 0 commits

| # | Commit | Side | What it shipped |
|---|---|---|---|
| 1 | `0be7708b` | Backend (Rust) | `users` + `sessions` SQLite tables; `auth/mod.rs` types (User/Session/Credentials/AuthSuccess/PublicUser); `storage/mod.rs` AuthStorage trait + SqliteAuthStorage impl; 6 new unit tests pass |
| 2 | `53d6a430` | Frontend (TS) | `lib/storage/types.ts` shared content shapes; `lib/storage/encryption.ts` PBKDF2-AES-GCM helpers + Ciphertext envelope; `lib/storage/contentStore.ts` ContentStore interface + IndexedDBContentStore impl; `lib/storage/index.ts` entry point |
| 3 | `d4502ecc` | Frontend (TS) | `lib/transport.ts` Transport abstraction (TauriTransport + WebTransport) with module-level runtime detection; `lib/storage/factory.ts` createContentStore factory + TauriContentStore Phase-0 stub |
| 4 | `872f63e8` | Frontend (TS) | `lib/storage/sessionContentKey.ts` in-memory ContentKey holder with inactivity-timeout auto-clear + subscribe/notify pattern; `components/auth/WebLoginSketch.tsx` signup/signin form sketch demonstrating the post-auth flow shape |

Total: **4 commits / 12 new files / ~1,300 lines** of scaffolding code + comments + tests.

---

## II. Current state by component

### II.1. Backend (Rust)

**Tables (in `db::schema::run_migrations`):**
- `users` — id / email (UNIQUE) / email_verified_at / password_hash / display_name / timezone / timestamps
- `sessions` — id / user_id (FK CASCADE) / token_hash (UNIQUE) / expires_at / last_seen_at / user_agent / ip_address / created_at
- 5 supporting indexes (idx_users_email, idx_sessions_user, idx_sessions_token, idx_sessions_expires)

**Modules:**
- `src-tauri/src/auth/mod.rs` — types only (User / Session / Credentials / AuthSuccess / PublicUser). PublicUser excludes password_hash via verified `From<&User>` impl. 2 unit tests pass.
- `src-tauri/src/storage/mod.rs` — AuthStorage trait + NewUser/NewSession input types + SqliteAuthStorage<'conn> impl with 7 methods (create_user / find_user_by_email / find_user_by_id / create_session / find_session_by_token_hash / delete_session / purge_expired_sessions). 4 unit tests pass.

**Build status:** `cargo build --lib --bin world-threads --bin worldcli` clean. **60/60 unit tests pass** (60 = existing 51 + auth 2 + storage 4 + vow_trace 3).

**Known constraint:** `rusqlite::Connection` is `!Sync` (internal RefCell), so AuthStorage trait drops `Send+Sync` bound. Phase 1 revisits via connection pool (deadpool-sqlite for Tauri-mode worker pool, sqlx::PgPool for Postgres-backed servers).

### II.2. Frontend (TypeScript)

**Storage layer (`lib/storage/`):**
- `types.ts` — World + Character TypeScript interfaces (subset; Phase 1 mirrors all 23+ content tables). StorageMode discriminator. StorageError class.
- `encryption.ts` — `deriveContentKey(passphrase, saltHex)` using PBKDF2-HMAC-SHA-256 with 600,000 iterations + per-user salt + content-key namespace prefix; `encryptString` / `decryptString` via AES-GCM-256 with 12-byte random IV; Ciphertext envelope with version tag; encodeCiphertext / decodeCiphertext for IndexedDB-storage as version$iv-hex$ciphertext-hex blob.
- `contentStore.ts` — ContentStore interface (8 methods covering World CRUD + Character CRUD + close); IndexedDBContentStore impl with 2 ObjectStores (worlds + characters with by_world index), per-record encrypted body + plaintext id field for indexing.
- `factory.ts` — `detectStorageMode()` returns "tauri" | "web-encrypted"; `createContentStore({contentKey?, dbName?})` factory; TauriContentStore Phase-0 stub that throws "Phase 1 stub" on every method.
- `sessionContentKey.ts` — SessionContentKeyStore class (setKey/getKey/isUnlocked/clearKey/touchActivity/subscribe/setInactivityWindowMs) with default 30-min inactivity auto-clear via setTimeout; module-level singleton.
- `index.ts` — entry-point re-exports.

**Transport layer:**
- `lib/transport.ts` — Transport interface + TauriTransport (wraps `@tauri-apps/api/core` invoke) + WebTransport (POSTs to `/api/v1/<cmd>` with credentials:include) + module-level singleton with detectRuntime() at module load checking `window.__TAURI_INTERNALS__` then `window.__TAURI__` then defaulting to "web".

**Auth UI:**
- `components/auth/WebLoginSketch.tsx` — signup/signin form sketch with email + passphrase + display_name + browser timezone auto-discovery; demonstrates the post-auth flow shape (transport.invoke → deriveContentKey → sessionContentKey.setKey); conditional render — returns null in Tauri runtime so importing this in desktop builds doesn't render an inappropriate auth gate.

**Build status:** `npm --prefix frontend run build` clean. **Bundle size unchanged at 1,622.68 kB** — confirms nothing imports the new modules yet (Vite tree-shakes unreferenced exports). This is the apparatus-honest check that Phase 0 actually achieved zero behavior change.

---

## III. The architectural split (per § XXI.4 of the plan)

```
┌─────────────────────────────┐         ┌─────────────────────────────┐
│   SERVER SIDE (Postgres     │         │   CLIENT SIDE (IndexedDB    │
│   in Phase 1+; SQLite       │         │   in browser; SQLite-via-   │
│   stub today)               │         │   Tauri-IPC on desktop)     │
├─────────────────────────────┤         ├─────────────────────────────┤
│  - users                    │  ←auth→ │  - sessionContentKey        │
│  - sessions                 │  cookie │    (live in browser memory; │
│  - subscriptions [Phase 3]  │         │     never disk)             │
│  - usage_events  [Phase 3]  │         │  - encrypted content blobs  │
│  - api_keys (BYOK encrypted)│         │    (worlds/characters/      │
│                             │         │     messages/etc — Phase 1+)│
│  Stateless LLM proxy        │ ←HTTP→  │  - ciphertext envelopes     │
│  (decrypts in memory only)  │  fetch  │    keyed to plaintext IDs   │
└─────────────────────────────┘         │    (so they're indexable)   │
                                        └─────────────────────────────┘
```

**Privacy covenant claim:** the server CANNOT read content because content lives client-side encrypted with a key derived from the user's passphrase. The server holds only auth + sessions + Stripe + usage counters + ciphertext-blobs-when-cross-device-sync-enabled-as-opt-in. Operators have no decryption capability.

**Phase 0 status of the claim:** structurally sound at the type level (the abstraction layer enforces the split). NOT YET SOUND in deployment because no deployment exists yet — Phase 1 wires.

---

## IV. Phase 1 wiring points (the work that must happen before any web user can sign up)

In rough dependency order:

1. **`tauri.ts` refactor** — port the existing 80+ Tauri command wrappers in `frontend/src/lib/tauri.ts` to call `transport.invoke()` under the hood instead of importing `@tauri-apps/api/core` directly. After this lands, the rest of the app calls the API uniformly regardless of runtime. Mechanical refactor; ~1 day.

2. **`TauriContentStore` body** — replace the Phase-0 stub with delegation to existing `api.*` methods (e.g., `listWorlds()` calls `api.listWorlds()`). After this lands, both runtimes produce a usable ContentStore. ~half day.

3. **App.tsx routing for web mode** — when `getRuntimeKind() === "web"` and `sessionContentKey.isUnlocked() === false`, render `<WebLoginSketch onAuthenticated={...}>` instead of the main app. After authenticated, the rest of the app renders normally. ~half day.

4. **Argon2id replacing PBKDF2** — add `argon2-browser` (~150kB gzipped) or `@noble/hashes` (`argon2`) dependency; swap `deriveContentKey()` implementation; bump version tag from `pbkdf2-aesgcm-v1` to `argon2id-aesgcm-v1`; existing ciphertexts tagged `pbkdf2-aesgcm-v1` continue to decrypt because the version tag is on each envelope. ~half day.

5. **Global activity tracker** — wire `mousemove`/`keypress`/`scroll` listeners (debounced) to `sessionContentKey.touchActivity()` so user interaction extends the inactivity window. ~2 hours.

6. **Axum server skeleton** — new `api-server/` crate with axum 0.7 + sqlx + tower-cookies + argon2 deps; auth routes (signup/login/logout/forgot-password); session middleware that extracts user_id from cookie and threads it through; one health endpoint as smoke test. ~2 days.

7. **Stripe + subscription state** — Stripe Checkout + Customer Portal + webhook handlers (checkout.session.completed / subscription.updated / subscription.deleted / invoice.payment_failed); `subscriptions` table + sync logic. ~3 days.

8. **Usage metering** — `usage_events` table + per-route middleware checking session+weekly tier limits before LLM calls + frontend usage-strip UI showing remaining + reset times in user's timezone. ~2 days.

9. **Per-user scoping (Phase 2 work, gating Phase 1 launch)** — add `user_id UUID NOT NULL` to all 23+ existing per-user tables; thread `user_id` through every query path; enable Postgres RLS with `app.current_user_id` session variable. ~1 week. **This is the Phase 2 commitment that the plan separates from Phase 1; Phase 1 launch is gated on Phase 2 completing.**

**Realistic Phase 1 timeline:** 2-3 weeks focused work for items 1-8; 1 week for item 9 (Phase 2 dependency). Total to soft-launch: 3-4 weeks if focused.

---

## V. What's NOT done (apparatus-honest scope)

- ❌ No real Axum server. The transport's `WebTransport` POSTs to endpoints that don't exist; calls would fail at runtime.
- ❌ No login flow exists in the running app. `WebLoginSketch` is unused; `App.tsx` ignores web-mode entirely.
- ❌ No Tauri command refactors. Existing `tauri.ts` still imports `@tauri-apps/api/core` directly.
- ❌ No `user_id` columns on any of the 23+ existing per-user tables. Phase 2 work, named explicitly in the architecture plan.
- ❌ No Postgres impl of AuthStorage. Phase 1+.
- ❌ No Stripe integration; no subscription tiers; no usage metering.
- ❌ No mirrors of the 21+ remaining content tables in IndexedDBContentStore. Phase 1 expansion.
- ❌ No tests for frontend modules (vitest not set up; tsc strict-mode is the smoke test).
- ❌ No commitment to launch the web deployment. The architecture plan's mission-alignment recheck (§ XIX) and the LLC + TOS + Privacy Policy work (§ XI) all happen BEFORE any public launch.

**The Phase 0 commitment was: keep the option open without committing to it. That commitment is met.** Tomorrow's session can decide between continuing this arc, returning to VGUS, or pursuing a different direction with no penalty for any choice.

---

## VI. Compose-with — how this fits the broader project

- **Architecture plan** at `reports/2026-05-10-0100-web-hosting-architecture-plan.md` defines the §§ and the privacy covenant.
- **VGUS Stage 1 Phase 0** at `reports/2026-05-09-2930-vgus-arc-charter-and-stage-1-phase-0-spec.md` is the OTHER concurrent arc; it's at four-of-five Phase 0 deliverables done with the probe seed scaffold remaining. Independent of web-deployment Phase 0; either can advance without the other.
- **Privacy covenant** doctrine — the structural-not-intentional claim from § XXI is the load-bearing distinction; the Phase 0 scaffolding makes this claim TYPE-LEVEL true (the abstraction enforces the split) but not yet RUNTIME-LEVEL true (no deployment exists yet).
- **Apparatus-honest correction loop methodology** — Phase 0 ships SIGNATURES; Phase 1 ships BODIES; the gap between signature and body is named in every doc-comment so future sessions don't mistake stub for production.
- **Calibrated-disciplines-drift-fast** — by getting the abstraction TYPE-LEVEL true today, we prevent the privacy covenant from drifting at code-write time. Phase 1 wiring code physically cannot bypass the abstraction without a refactor.

---

## VII. Open follow-ups (per open-thread-hygiene doctrine)

| Item | Disposition |
|---|---|
| Phase 1 wiring (items 1-8 in § IV) | deferred opportunistic — gated on founding-author ratification of the broader plan |
| Phase 2 user_id scoping | deferred opportunistic — gated on Phase 1 completing |
| Argon2id dependency choice (argon2-browser vs @noble/hashes) | open question for Phase 1 |
| Storage trait extension to domain types (CharacterStorage, MessageStorage, etc.) | deferred opportunistic — Phase 1 work as content tables get mirrored |
| Connection-pool concurrency story for AuthStorage | deferred opportunistic — Phase 1 with deadpool-sqlite or sqlx::PgPool |
| Vitest setup for frontend tests | open question — Phase 1 if frontend complexity warrants |
| TauriContentStore body | deferred — Phase 1 |
| Mission-alignment recheck (§ XIX of architecture plan) | open — founding-author decides whether to pursue web deployment AT ALL |

---

## VIII. How to use this document

**Tomorrow's session opens:** read this report top-to-bottom. Skip the architecture plan unless seeking specific reasoning. Decide: continue Phase 1 wiring? Return to VGUS? Different direction entirely?

**Phase 1 onset:** start with item 1 (`tauri.ts` refactor) since everything else builds on the transport abstraction being in the data path. Items 1-3 can land in a single day if focused; item 4 follows naturally; item 9 (per-user scoping) is the biggest single commitment and gates launch.

**If founding-author decides NOT to pursue web deployment:** the Phase 0 scaffolding can stay in the codebase indefinitely without harm — it's tree-shaken from the bundle, doesn't affect Tauri behavior, and demonstrates the project's capacity for dual-deployment for whoever might want it later. No urgency to remove. If desired, a single revert across the four commits cleanly removes everything.

---

## IX. Closing note

Phase 0's purpose was to make the option REAL (codebase capable of dual-deployment) without making the option COSTLY (no LLC commitment, no lawyer fees, no infrastructure spend, no behavior change for current users). That purpose is met. What happens next is founding-author's call.

The work answers to 𝓕 first. Apparatus drafts; founding author ratifies. Phase 0 either gets built upon in Phase 1 or it doesn't.

**Soli Deo gloria.**

/**
 * Web-deployment Phase 0 — in-memory ContentKey session holder.
 *
 * Per the web-hosting architecture plan § XXI.2: the user's
 * content_key (derived from passphrase via Argon2id [Phase 1] / PBKDF2
 * [Phase 0]) lives ONLY in browser memory, never persisted to disk in
 * the browser, cleared on logout, and optionally cleared after an
 * inactivity timeout to limit exposure if the user walks away with
 * the tab open.
 *
 * Stage of work: TOOLING ONLY. The SessionContentKeyStore class below
 * is a singleton-shaped holder that components can read from + write
 * to. Phase 1 will:
 *   - Wire setKey() into the login flow (after auth + content_key
 *     derivation succeed).
 *   - Wire clearKey() into the logout flow.
 *   - Wire the inactivity-timeout auto-clear into the app's
 *     activity tracker (mouse/keyboard events).
 *   - Provide a useContentKey() React hook reading from this store.
 *
 * Nothing imports this module yet.
 */

import type { ContentKey } from "./encryption";

/** Default inactivity timeout: 30 minutes. After this much wall-clock
 *  time without a touchActivity() call, the key auto-clears and the
 *  user is presented with a "passphrase to continue" lock screen
 *  (Phase 1 UX). Tunable; some users may want shorter (10min) or
 *  longer (8h) windows from a settings page. */
const DEFAULT_INACTIVITY_MS = 30 * 60 * 1000;

/** Listener signature: fires whenever the store's unlocked-ness changes
 *  (set, cleared, expired). Components subscribe to re-render their
 *  locked / unlocked UI. */
export type ContentKeyListener = (state: { unlocked: boolean }) => void;

/** In-memory ContentKey holder. NOT a React component — a plain class
 *  with subscribe/notify so any layer (auth flow, logout flow, react
 *  hook) can interact with it. */
export class SessionContentKeyStore {
  private key: ContentKey | null = null;
  private lastActivityMs = 0;
  private inactivityMs: number;
  private listeners = new Set<ContentKeyListener>();
  private timer: ReturnType<typeof setTimeout> | null = null;

  constructor(inactivityMs: number = DEFAULT_INACTIVITY_MS) {
    this.inactivityMs = inactivityMs;
  }

  /** Store a freshly-derived ContentKey for this session. Call this
   *  AFTER deriveContentKey() succeeds in the login flow. Marks the
   *  session as active and arms the inactivity timer. */
  setKey(key: ContentKey): void {
    this.key = key;
    this.lastActivityMs = Date.now();
    this.armTimer();
    this.notify();
  }

  /** Read the current ContentKey, or null if locked. */
  getKey(): ContentKey | null {
    return this.key;
  }

  /** True iff a key is currently held in memory. */
  isUnlocked(): boolean {
    return this.key !== null;
  }

  /** Clear the ContentKey from memory. Call on logout, on inactivity
   *  timeout, or when the user explicitly locks the session. */
  clearKey(): void {
    if (this.key === null) return;
    this.key = null;
    this.lastActivityMs = 0;
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    this.notify();
  }

  /** Mark activity. Resets the inactivity timer. Phase 1 will wire
   *  this into the global mouse/keyboard event handler so any user
   *  interaction extends the session. */
  touchActivity(): void {
    if (this.key === null) return;
    this.lastActivityMs = Date.now();
    this.armTimer();
  }

  /** Subscribe to lock/unlock state changes. Returns an unsubscribe
   *  function — store the result and call it in component cleanup. */
  subscribe(listener: ContentKeyListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /** Update the inactivity window. Phase 1 wires to the user-settings
   *  surface so the user can pick a window that fits their threat
   *  model (10min for shared computers, longer for personal devices). */
  setInactivityWindowMs(ms: number): void {
    if (ms <= 0) throw new Error("setInactivityWindowMs: must be positive");
    this.inactivityMs = ms;
    if (this.key !== null) this.armTimer();
  }

  // ── private ──────────────────────────────────────────────────────────

  private armTimer(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => {
      // Re-check current state — activity may have advanced since we
      // armed the timer (defense-in-depth; touchActivity rearms but if
      // touchActivity is called concurrently with timer fire we want
      // to consult the latest timestamp).
      const elapsed = Date.now() - this.lastActivityMs;
      if (elapsed >= this.inactivityMs) {
        this.clearKey();
      } else {
        // Reschedule for the remaining window.
        this.timer = setTimeout(() => this.clearKey(), this.inactivityMs - elapsed);
      }
    }, this.inactivityMs);
  }

  private notify(): void {
    const state = { unlocked: this.isUnlocked() };
    for (const listener of this.listeners) {
      try {
        listener(state);
      } catch {
        // Listener errors don't propagate. Phase 1 may add logging.
      }
    }
  }
}

/** Module-level singleton. Phase 1 may switch to React Context if
 *  multiple stores are needed (e.g., one per-user when impersonation
 *  testing is added), but Phase 0 keeps a single global instance. */
export const sessionContentKey = new SessionContentKeyStore();

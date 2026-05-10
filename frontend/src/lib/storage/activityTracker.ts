/**
 * Web-deployment Phase 1 item 5 — global activity tracker.
 *
 * Per the web-hosting architecture plan § XXI.2 + readiness summary § IV
 * item 5. Wires browser input events (mousemove / keypress / scroll /
 * pointermove / touchstart) into sessionContentKey.touchActivity() so
 * the inactivity timer doesn't fire while the user is actively using
 * the app. Without this tracker, the 30-minute inactivity timeout
 * (default in SessionContentKeyStore) would fire on every chat session,
 * forcing re-login mid-conversation.
 *
 * Debounced at 5 seconds: a burst of mousemove events fires
 * touchActivity() at most once per 5s window, which is far below the
 * inactivity timeout (30 min) and avoids any pretense of per-frame
 * overhead. Per § XXI.2 the inactivity feature is a security/UX
 * tradeoff for users on shared computers — settable via
 * sessionContentKey.setInactivityWindowMs() from a future settings UI.
 *
 * Tauri-mode: tracker is a no-op (sessionContentKey is unused in
 * desktop; the listener install is harmless but the touchActivity calls
 * do nothing because no key is held).
 *
 * Lifecycle: install via React hook in WebAuthGate, which mounts at
 * the app's outer routing layer; teardown on unmount restores clean
 * state for tests / hot-reload.
 */

import { useEffect } from "react";
import { sessionContentKey } from "./sessionContentKey";

const DEBOUNCE_MS = 5_000;
const ACTIVITY_EVENTS: readonly (keyof DocumentEventMap)[] = [
  "mousemove",
  "keypress",
  "scroll",
  "pointerdown",
  "touchstart",
];

/** Install global activity listeners. Returns an uninstall function.
 *  Imperative API for non-React callers (tests / setup scripts). */
export function installGlobalActivityTracker(): () => void {
  if (typeof document === "undefined") {
    // SSR / test env — no-op.
    return () => {};
  }

  let lastTouchedAt = 0;
  const onActivity = () => {
    const now = Date.now();
    if (now - lastTouchedAt < DEBOUNCE_MS) return;
    lastTouchedAt = now;
    sessionContentKey.touchActivity();
  };

  for (const ev of ACTIVITY_EVENTS) {
    document.addEventListener(ev, onActivity, { passive: true });
  }

  return () => {
    for (const ev of ACTIVITY_EVENTS) {
      document.removeEventListener(ev, onActivity);
    }
  };
}

/** React hook variant. Install on mount; uninstall on unmount. */
export function useActivityTracker(): void {
  useEffect(() => installGlobalActivityTracker(), []);
}

/**
 * Web-deployment Phase 1 item 3 — auth gate for web-mode users.
 *
 * Per the web-hosting architecture plan § V (auth) + § XXI.2 (encryption)
 * + the readiness summary § IV item 3.
 *
 * Renders children when:
 *   - Running in Tauri runtime (always — desktop has no web auth concept), OR
 *   - Running in web runtime AND sessionContentKey is unlocked.
 *
 * Renders <WebLoginSketch /> when running in web runtime AND the key is
 * locked. Subscribes to sessionContentKey lock-state changes so the
 * gate auto-flips when login succeeds (key set) or session expires
 * (inactivity-timer auto-clear, manual logout, etc.).
 *
 * Tauri runtime: this component is a transparent passthrough. Existing
 * desktop behavior is unchanged because the Tauri build never goes
 * through the locked branch.
 *
 * Phase 1 item 6 (Axum server) is required before the WebLoginSketch
 * form can actually authenticate. Until then, web-mode users would see
 * the form but submission would 404. This is correct sequencing per
 * readiness summary § IV — items 3 and 6 land independently; the gate
 * existing without a backend is fine because no web users exist yet.
 */

import { useEffect, useState, type ReactNode } from "react";
import { sessionContentKey } from "@/lib/storage/sessionContentKey";
import { getRuntimeKind } from "@/lib/transport";
import { WebLoginSketch } from "./WebLoginSketch";

interface Props {
  children: ReactNode;
}

export function WebAuthGate({ children }: Props) {
  // Tauri-mode: bypass entirely. Desktop has no web-auth concept; we
  // never want this gate to obstruct the existing experience.
  if (getRuntimeKind() === "tauri") {
    return <>{children}</>;
  }

  // Web-mode: subscribe to lock-state changes via the sessionContentKey
  // store. Initial state is unlocked-iff-key-already-set (in case the
  // component remounts after a successful login).
  return <WebGateInner>{children}</WebGateInner>;
}

function WebGateInner({ children }: Props) {
  const [unlocked, setUnlocked] = useState(() => sessionContentKey.isUnlocked());

  useEffect(() => {
    const unsubscribe = sessionContentKey.subscribe((state) => {
      setUnlocked(state.unlocked);
    });
    return unsubscribe;
  }, []);

  if (unlocked) {
    return <>{children}</>;
  }
  // Key not held in memory — show the login form. On successful auth,
  // sessionContentKey.setKey() fires the subscriber above which flips
  // unlocked=true and the children render.
  return <WebLoginSketch onAuthenticated={() => setUnlocked(true)} />;
}

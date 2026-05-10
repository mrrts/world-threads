/**
 * Web-deployment Phase 0 — transport abstraction.
 *
 * Per the web-hosting architecture plan § III.3. Exposes a single
 * `Transport.invoke(cmd, args)` shape that future code can call without
 * caring whether it's running inside Tauri (IPC bridge) or in a web
 * browser (HTTP fetch to the Axum server).
 *
 * Stage of work: TOOLING ONLY. Existing frontend code continues to
 * import @tauri-apps/api/core directly via frontend/src/lib/tauri.ts.
 * Nothing imports this module yet. Phase 1 will refactor lib/tauri.ts
 * to call transport.invoke() under the hood, at which point a single
 * line of detection at boot picks the right backend for the whole app.
 *
 * Detection: at module-load time, tries to determine whether the
 * runtime is Tauri or web. The cheap signal is whether the global
 * __TAURI_INTERNALS__ symbol exists; this is set by Tauri v2's
 * preload before user JS runs. Fallback: probe for presence of the
 * @tauri-apps/api/core module's runtime side effects.
 */

import { invoke as tauriInvoke } from "@tauri-apps/api/core";

/** What runtime is the app currently running in. */
export type RuntimeKind = "tauri" | "web";

/** A transport-level command invocation. Returns whatever the backend
 *  returns; caller is responsible for type-asserting the response. */
export interface Transport {
  readonly kind: RuntimeKind;
  invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T>;
}

/** Tauri runtime: forwards to @tauri-apps/api/core's invoke. Throws if
 *  Tauri isn't actually available at runtime (caller misdetected). */
export class TauriTransport implements Transport {
  readonly kind: RuntimeKind = "tauri";
  invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    return tauriInvoke<T>(cmd, args ?? {});
  }
}

/** Web runtime: POSTs to /api/v1/<cmd> with JSON body. The future Axum
 *  server's route convention is one POST endpoint per cmd, mirroring
 *  Tauri's command shape one-to-one so that lib/tauri.ts can be
 *  refactored mechanically. Cookies (credentials: "include") carry
 *  the auth session token. */
export class WebTransport implements Transport {
  readonly kind: RuntimeKind = "web";
  private readonly basePath: string;

  constructor(basePath: string = "/api/v1") {
    this.basePath = basePath;
  }

  async invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const r = await fetch(`${this.basePath}/${cmd}`, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(args ?? {}),
    });
    if (!r.ok) {
      const text = await r.text().catch(() => r.statusText);
      throw new Error(`transport.invoke[${cmd}]: ${r.status} ${text}`);
    }
    // Empty-body responses → undefined; otherwise parse JSON.
    const ct = r.headers.get("content-type") ?? "";
    if (!ct.includes("json") || r.status === 204) {
      return undefined as unknown as T;
    }
    return (await r.json()) as T;
  }
}

/** Module-level cached detection. Reads window once at module load.
 *  Future Phase 1 work may refine to handle SSR / test contexts. */
function detectRuntime(): RuntimeKind {
  if (typeof window === "undefined") return "web"; // SSR / test env
  // Tauri v2 preload sets __TAURI_INTERNALS__. Tauri v1 used __TAURI__.
  // Accept either to keep the detector resilient across versions.
  const w = window as unknown as Record<string, unknown>;
  if (w.__TAURI_INTERNALS__ !== undefined) return "tauri";
  if (w.__TAURI__ !== undefined) return "tauri";
  return "web";
}

/** The active transport for this runtime. Created once at module load.
 *  Phase 1 may add a setter (resetTransport(t)) for testing or for
 *  switching modes mid-session, but Phase 0 keeps it simple. */
export const transport: Transport =
  detectRuntime() === "tauri" ? new TauriTransport() : new WebTransport();

/** Public detector for components that need to render conditional UI
 *  (auth pages on web; nothing on desktop; etc.). Reads the same global
 *  state the transport was created from. */
export function getRuntimeKind(): RuntimeKind {
  return transport.kind;
}

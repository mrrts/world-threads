/**
 * Web-deployment Phase 0 — shared content-storage type definitions.
 *
 * Per the web-hosting architecture plan at
 * `reports/2026-05-10-0100-web-hosting-architecture-plan.md` § XXI.4
 * (hybrid client-side-content + server-side-metering architecture).
 *
 * Stage of work: TOOLING ONLY. These types mirror a small subset of the
 * existing Rust-side domain shapes (World, Character) sufficient to
 * demonstrate the abstraction layer. Phase 1+ will mirror the full set
 * (threads, messages, world_events, memory_artifacts, settings,
 * portraits, images, reactions, character_mood, group_chats, etc.) —
 * NOT done in Phase 0.
 *
 * Existing frontend code continues to use frontend/src/lib/tauri.ts
 * directly; nothing imports this module yet.
 */

/** Minimal world record. Mirrors the existing Rust World shape (subset). */
export interface World {
  world_id: string;
  name: string;
  description: string;
  tone_tags: string[];
  state: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

/** Minimal character record. Mirrors the existing Rust Character shape (subset).
 * Fields like voice_rules / boundaries / backstory_facts / inventory remain
 * Record/array shapes; full domain mirroring is Phase 1 work. */
export interface Character {
  character_id: string;
  world_id: string;
  display_name: string;
  identity: string;
  voice_rules: unknown[];
  boundaries: unknown[];
  backstory_facts: unknown[];
  state: Record<string, unknown>;
  avatar_color: string;
  sex: string;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
}

/** Capability discriminator returned by detectStorageMode().
 *  - "tauri": running inside the Tauri desktop app; content storage
 *    routes to existing Rust IPC (frontend/src/lib/tauri.ts).
 *  - "web-encrypted": running in a browser session with a derived
 *    content-encryption key bound to the user's authenticated session;
 *    content lives in IndexedDB encrypted at rest. */
export type StorageMode = "tauri" | "web-encrypted";

/** Errors specific to the storage layer. Phase 1 will refine into
 *  typed variants (NotFound / Encrypted / QuotaExceeded / etc.). */
export class StorageError extends Error {
  readonly kind: string;
  constructor(kind: string, message: string) {
    super(`${kind}: ${message}`);
    this.name = "StorageError";
    this.kind = kind;
  }
}

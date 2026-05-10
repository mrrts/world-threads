/**
 * Web-deployment Phase 0 — storage-mode factory.
 *
 * Per the web-hosting architecture plan § III.3 + § XXI.4. Picks the
 * right ContentStore backend at boot based on the runtime kind:
 *
 *   - Tauri runtime → TauriContentStore (delegates to existing
 *     frontend/src/lib/tauri.ts API; content stays in the Tauri SQLite
 *     database; no client-side encryption layer needed because the
 *     SQLite file lives only on the user's machine).
 *
 *   - Web runtime → IndexedDBContentStore (encrypted-at-rest content
 *     in browser IndexedDB; encrypted via user-passphrase-derived
 *     content_key; auth + sessions + Stripe + usage live on the
 *     server side per § XXI.4 hybrid).
 *
 * Stage of work: TOOLING ONLY. The TauriContentStore impl below is a
 * stub that throws on every method — Phase 1 will fill it in by
 * delegating to the existing tauri.ts API. The IndexedDB impl is
 * already ready (commit 53d6a43). The factory ties them together so
 * Phase 1 wiring has one entry point: createContentStore({...}).
 *
 * Nothing imports this module yet. Phase 1 will replace direct
 * tauri.ts usage in components with createContentStore().
 */

import type { ContentKey } from "./encryption";
import type { ContentStore } from "./contentStore";
import { IndexedDBContentStore } from "./contentStore";
import { type StorageMode } from "./types";
import { getRuntimeKind } from "../transport";

/** Factory dependencies. Web mode requires a live ContentKey derived
 *  from the user's passphrase at login; Tauri mode requires nothing. */
export interface CreateContentStoreDeps {
  /** Required for web-encrypted mode; ignored in Tauri mode. */
  contentKey?: ContentKey;
  /** IndexedDB database name. Defaults to "worldthreads-content-v1".
   *  Phase 1 will scope per-user (e.g., "worldthreads-content-v1:<user_id>")
   *  so multiple users on the same browser don't collide. */
  dbName?: string;
}

/** Detect the storage mode from the runtime kind. Tauri runtime always
 *  routes content through Tauri SQLite; web runtime routes through
 *  IndexedDB encrypted at rest. */
export function detectStorageMode(): StorageMode {
  return getRuntimeKind() === "tauri" ? "tauri" : "web-encrypted";
}

/** Create a ContentStore appropriate for the current runtime. Throws
 *  if web mode is selected without a contentKey. */
export function createContentStore(deps: CreateContentStoreDeps = {}): ContentStore {
  const mode = detectStorageMode();
  if (mode === "tauri") {
    return new TauriContentStore();
  }
  if (!deps.contentKey) {
    throw new Error(
      "createContentStore: web-encrypted mode requires a contentKey (derive via deriveContentKey(passphrase, saltHex) at login).",
    );
  }
  return new IndexedDBContentStore(deps.dbName ?? "worldthreads-content-v1", deps.contentKey);
}

/** Tauri-mode content store. Phase 0 stub: throws on every method.
 *  Phase 1 fills in by delegating to the existing tauri.ts API
 *  (api.listWorlds / api.getWorld / api.updateWorld / api.deleteWorld
 *  / api.listCharacters / api.getCharacter / api.updateCharacter / etc.).
 *  Existing frontend code continues to call tauri.ts directly until the
 *  Phase 1 refactor lands; this stub exists only so the factory's
 *  return-type is satisfied symmetrically. */
export class TauriContentStore implements ContentStore {
  private static unimplemented(method: string): never {
    throw new Error(
      `TauriContentStore.${method}: Phase 1 stub — frontend currently calls tauri.ts directly. ` +
        `Phase 1 wires this to delegate to the existing API.`,
    );
  }

  async listWorlds(): Promise<never> {
    return TauriContentStore.unimplemented("listWorlds");
  }
  async getWorld(_id: string): Promise<never> {
    return TauriContentStore.unimplemented("getWorld");
  }
  async putWorld(_w: unknown): Promise<never> {
    return TauriContentStore.unimplemented("putWorld");
  }
  async deleteWorld(_id: string): Promise<never> {
    return TauriContentStore.unimplemented("deleteWorld");
  }
  async listCharacters(_world: string): Promise<never> {
    return TauriContentStore.unimplemented("listCharacters");
  }
  async getCharacter(_id: string): Promise<never> {
    return TauriContentStore.unimplemented("getCharacter");
  }
  async putCharacter(_c: unknown): Promise<never> {
    return TauriContentStore.unimplemented("putCharacter");
  }
  async deleteCharacter(_id: string): Promise<never> {
    return TauriContentStore.unimplemented("deleteCharacter");
  }
  async close(): Promise<void> {
    /* no-op */
  }
}

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
import { type Character, type StorageMode, type World } from "./types";
import { getRuntimeKind } from "../transport";
import { api } from "../tauri";

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

/** Tauri-mode content store. Phase 1 implementation: delegates to the
 *  existing tauri.ts api.* methods, which are themselves transport-routed
 *  via the alias landed in commit e68601c so the data path is uniform.
 *
 *  Type boundary: storage/types.ts defines a forward-compatible subset
 *  shape for World + Character (intentionally lighter than the production
 *  shape exported from tauri.ts). Casting at the boundary is safe under
 *  structural typing because the field names overlap and the runtime
 *  serializer doesn't care about extra fields. Phase 2 will reconcile the
 *  type definitions when content tables are mirrored fully across both
 *  stores; for now the casts let both impls satisfy the same interface
 *  without forcing immediate domain-type unification.
 *
 *  putWorld + putCharacter assume the entity already exists (delegate to
 *  api.updateWorld / api.updateCharacter which both UPDATE not UPSERT).
 *  Creation continues to go through api.createWorld / api.createCharacter
 *  in the existing call paths; the ContentStore is for steady-state CRUD
 *  not first-time creation. Phase 2 may add upsert helpers if needed. */
export class TauriContentStore implements ContentStore {
  async listWorlds(): Promise<World[]> {
    return (await api.listWorlds()) as unknown as World[];
  }

  async getWorld(world_id: string): Promise<World | null> {
    try {
      return (await api.getWorld(world_id)) as unknown as World;
    } catch {
      // api.getWorld throws on not-found; ContentStore semantics return null.
      return null;
    }
  }

  async putWorld(world: World): Promise<void> {
    await api.updateWorld(world as unknown as Parameters<typeof api.updateWorld>[0]);
  }

  async deleteWorld(world_id: string): Promise<void> {
    await api.deleteWorld(world_id);
  }

  async listCharacters(world_id: string): Promise<Character[]> {
    return (await api.listCharacters(world_id)) as unknown as Character[];
  }

  async getCharacter(character_id: string): Promise<Character | null> {
    try {
      return (await api.getCharacter(character_id)) as unknown as Character;
    } catch {
      return null;
    }
  }

  async putCharacter(character: Character): Promise<void> {
    await api.updateCharacter(
      character as unknown as Parameters<typeof api.updateCharacter>[0],
    );
  }

  async deleteCharacter(character_id: string): Promise<void> {
    await api.deleteCharacter(character_id);
  }

  async close(): Promise<void> {
    /* no-op for Tauri — IPC channel managed by Tauri runtime */
  }
}

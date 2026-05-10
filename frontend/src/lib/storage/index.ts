/**
 * Web-deployment Phase 0 — storage layer entry point.
 *
 * Per the web-hosting architecture plan § XXI.4. Phase 0 ships the
 * IndexedDB content-store implementation + content-key encryption
 * helpers + shared types. No existing frontend code imports this
 * module yet.
 *
 * Phase 1 will add:
 *   - TauriContentStore that delegates to existing tauri.ts API.
 *   - detectStorageMode() factory that picks the right backend at boot.
 *   - SessionContentKeyStore that holds the live ContentKey in memory
 *     (not localStorage), tied to the active auth session.
 *   - Argon2id-based key derivation replacing PBKDF2.
 *   - Wired imports throughout the frontend's data layer.
 */

export type { World, Character, StorageMode } from "./types";
export { StorageError } from "./types";

export type { ContentKey, Ciphertext } from "./encryption";
export {
  decodeCiphertext,
  decryptString,
  deriveContentKey,
  encodeCiphertext,
  encryptString,
} from "./encryption";

export type { ContentStore } from "./contentStore";
export { IndexedDBContentStore } from "./contentStore";

export type { CreateContentStoreDeps } from "./factory";
export {
  TauriContentStore,
  createContentStore,
  detectStorageMode,
} from "./factory";

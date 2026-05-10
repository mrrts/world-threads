/**
 * Web-deployment Phase 0 — ContentStore interface + IndexedDB-backed impl.
 *
 * Per the web-hosting architecture plan § XXI.4 (hybrid client-side-content
 * + server-side-metering). The web deployment stores ALL content in
 * IndexedDB encrypted at rest with the user's content_key. Tauri-mode
 * stores content in SQLite via existing IPC and does NOT use this layer.
 *
 * Stage of work: TOOLING ONLY. The interface defines the contract; the
 * IndexedDB impl demonstrates the shape with World + Character. Phase 1
 * mirrors all 23+ content tables (threads, messages, world_events,
 * memory_artifacts, settings, character_portraits, world_images,
 * chat_backgrounds, reactions, character_mood, group_chats,
 * group_messages, dev_chat_sessions, dev_chat_messages, vows,
 * vow_event_log, vow_invocations, location_derivations).
 *
 * The Tauri-mode implementation (TauriContentStore) is sketched for
 * symmetry; in practice the existing frontend/src/lib/tauri.ts API
 * already provides the per-table operations and a thin wrapper would
 * bridge them to the ContentStore shape. Not built in Phase 0.
 *
 * No existing frontend code imports this module. Phase 1 wires it in
 * via a transport-aware factory.
 */

import {
  type ContentKey,
  type Ciphertext,
  decodeCiphertext,
  decryptString,
  encodeCiphertext,
  encryptString,
} from "./encryption";
import type { Character, World } from "./types";
import { StorageError } from "./types";

/** Public interface implemented by IndexedDBContentStore (web mode) and
 *  the future TauriContentStore (desktop mode). Phase 0 covers the
 *  representative shapes; Phase 1 expands. */
export interface ContentStore {
  // World CRUD
  listWorlds(): Promise<World[]>;
  getWorld(world_id: string): Promise<World | null>;
  putWorld(world: World): Promise<void>;
  deleteWorld(world_id: string): Promise<void>;

  // Character CRUD (scoped by world_id where relevant)
  listCharacters(world_id: string): Promise<Character[]>;
  getCharacter(character_id: string): Promise<Character | null>;
  putCharacter(character: Character): Promise<void>;
  deleteCharacter(character_id: string): Promise<void>;

  // Lifecycle
  close(): Promise<void>;
}

/** IndexedDB-backed content store. Each ObjectStore holds records as
 *  ENCRYPTED VALUES under their plaintext IDs. The id stays unencrypted
 *  so we can do indexed lookups; the body is encrypted as a single hex
 *  blob via encodeCiphertext().
 *
 *  Schema: one ObjectStore per content type. v1 covers worlds + characters
 *  with a `world_id` index on characters. Phase 1 adds the rest. */
export class IndexedDBContentStore implements ContentStore {
  private db: IDBDatabase | null = null;

  private readonly dbName: string;
  private readonly contentKey: ContentKey;

  constructor(dbName: string, contentKey: ContentKey) {
    this.dbName = dbName;
    this.contentKey = contentKey;
  }

  /** Lazy-open the IndexedDB connection, running migrations if the schema
   *  version is newer than what's stored. Returns the live IDBDatabase. */
  private async open(): Promise<IDBDatabase> {
    if (this.db) return this.db;
    return new Promise<IDBDatabase>((resolve, reject) => {
      const req = indexedDB.open(this.dbName, /* version */ 1);
      req.onerror = () => reject(new StorageError("idb-open", req.error?.message ?? "unknown"));
      req.onupgradeneeded = () => {
        const db = req.result;
        if (!db.objectStoreNames.contains("worlds")) {
          db.createObjectStore("worlds", { keyPath: "world_id" });
        }
        if (!db.objectStoreNames.contains("characters")) {
          const store = db.createObjectStore("characters", { keyPath: "character_id" });
          store.createIndex("by_world", "world_id", { unique: false });
        }
      };
      req.onsuccess = () => {
        this.db = req.result;
        resolve(req.result);
      };
    });
  }

  /** Encrypt a plain object as a single envelope record:
   *  {id_field, world_id?, encrypted_body}. id stays plaintext for
   *  IndexedDB indexing; body is the encrypted JSON of the full object. */
  private async encryptRecord(
    storeName: "worlds" | "characters",
    record: World | Character,
  ): Promise<EncryptedRecord> {
    const json = JSON.stringify(record);
    const ct = await encryptString(this.contentKey, json);
    const body = encodeCiphertext(ct);
    if (storeName === "worlds") {
      const w = record as World;
      return { world_id: w.world_id, body };
    } else {
      const c = record as Character;
      return { character_id: c.character_id, world_id: c.world_id, body };
    }
  }

  private async decryptRecord<T>(record: EncryptedRecord | undefined): Promise<T | null> {
    if (!record) return null;
    const envelope: Ciphertext = decodeCiphertext(record.body);
    const json = await decryptString(this.contentKey, envelope);
    return JSON.parse(json) as T;
  }

  async listWorlds(): Promise<World[]> {
    const db = await this.open();
    return new Promise<World[]>((resolve, reject) => {
      const tx = db.transaction("worlds", "readonly");
      const store = tx.objectStore("worlds");
      const req = store.getAll();
      req.onerror = () => reject(new StorageError("idb-list", req.error?.message ?? "unknown"));
      req.onsuccess = async () => {
        try {
          const records = req.result as EncryptedRecord[];
          const decrypted = await Promise.all(
            records.map((r) => this.decryptRecord<World>(r)),
          );
          resolve(decrypted.filter((w): w is World => w !== null));
        } catch (e) {
          reject(e);
        }
      };
    });
  }

  async getWorld(world_id: string): Promise<World | null> {
    const db = await this.open();
    return new Promise<World | null>((resolve, reject) => {
      const tx = db.transaction("worlds", "readonly");
      const req = tx.objectStore("worlds").get(world_id);
      req.onerror = () => reject(new StorageError("idb-get", req.error?.message ?? "unknown"));
      req.onsuccess = async () => {
        try {
          resolve(await this.decryptRecord<World>(req.result as EncryptedRecord | undefined));
        } catch (e) {
          reject(e);
        }
      };
    });
  }

  async putWorld(world: World): Promise<void> {
    const db = await this.open();
    const record = await this.encryptRecord("worlds", world);
    return new Promise<void>((resolve, reject) => {
      const tx = db.transaction("worlds", "readwrite");
      const req = tx.objectStore("worlds").put(record);
      req.onerror = () => reject(new StorageError("idb-put", req.error?.message ?? "unknown"));
      req.onsuccess = () => resolve();
    });
  }

  async deleteWorld(world_id: string): Promise<void> {
    const db = await this.open();
    return new Promise<void>((resolve, reject) => {
      const tx = db.transaction("worlds", "readwrite");
      const req = tx.objectStore("worlds").delete(world_id);
      req.onerror = () => reject(new StorageError("idb-delete", req.error?.message ?? "unknown"));
      req.onsuccess = () => resolve();
    });
  }

  async listCharacters(world_id: string): Promise<Character[]> {
    const db = await this.open();
    return new Promise<Character[]>((resolve, reject) => {
      const tx = db.transaction("characters", "readonly");
      const idx = tx.objectStore("characters").index("by_world");
      const req = idx.getAll(world_id);
      req.onerror = () => reject(new StorageError("idb-list", req.error?.message ?? "unknown"));
      req.onsuccess = async () => {
        try {
          const records = req.result as EncryptedRecord[];
          const decrypted = await Promise.all(
            records.map((r) => this.decryptRecord<Character>(r)),
          );
          resolve(decrypted.filter((c): c is Character => c !== null));
        } catch (e) {
          reject(e);
        }
      };
    });
  }

  async getCharacter(character_id: string): Promise<Character | null> {
    const db = await this.open();
    return new Promise<Character | null>((resolve, reject) => {
      const tx = db.transaction("characters", "readonly");
      const req = tx.objectStore("characters").get(character_id);
      req.onerror = () => reject(new StorageError("idb-get", req.error?.message ?? "unknown"));
      req.onsuccess = async () => {
        try {
          resolve(await this.decryptRecord<Character>(req.result as EncryptedRecord | undefined));
        } catch (e) {
          reject(e);
        }
      };
    });
  }

  async putCharacter(character: Character): Promise<void> {
    const db = await this.open();
    const record = await this.encryptRecord("characters", character);
    return new Promise<void>((resolve, reject) => {
      const tx = db.transaction("characters", "readwrite");
      const req = tx.objectStore("characters").put(record);
      req.onerror = () => reject(new StorageError("idb-put", req.error?.message ?? "unknown"));
      req.onsuccess = () => resolve();
    });
  }

  async deleteCharacter(character_id: string): Promise<void> {
    const db = await this.open();
    return new Promise<void>((resolve, reject) => {
      const tx = db.transaction("characters", "readwrite");
      const req = tx.objectStore("characters").delete(character_id);
      req.onerror = () => reject(new StorageError("idb-delete", req.error?.message ?? "unknown"));
      req.onsuccess = () => resolve();
    });
  }

  async close(): Promise<void> {
    if (this.db) {
      this.db.close();
      this.db = null;
    }
  }
}

/** Internal record shape stored in IndexedDB. id field stays plaintext
 *  for indexing; body is the encrypted JSON of the full domain object. */
interface EncryptedRecord {
  world_id?: string;
  character_id?: string;
  body: string; // encodeCiphertext output
}

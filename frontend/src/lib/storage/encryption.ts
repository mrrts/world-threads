/**
 * Web-deployment Phase 0 — content encryption helpers.
 *
 * Per the web-hosting architecture plan § XXI.2: a per-user `content_key`
 * is derived from the user's password via a separate KDF salt (distinct
 * from the auth-key derivation), held only in browser memory while the
 * session is active, never persisted to disk in the browser, and used
 * to encrypt all content stored in IndexedDB.
 *
 * Stage of work: TOOLING ONLY. This module ships the SIGNATURE of the
 * encryption interface plus a Web-Crypto-backed AES-GCM implementation
 * sufficient to demonstrate the abstraction. Phase 1 will:
 *   - Replace PBKDF2 with Argon2id via argon2-browser or @noble/hashes
 *     (Argon2id is the standard choice; PBKDF2 is the Web-Crypto-native
 *     fallback for Phase 0 scaffolding).
 *   - Add key-rotation logic.
 *   - Add an encrypted-recovery-key offline-export flow per § XXI.3.
 *   - Wire into a session-state store that holds the live ContentKey
 *     in memory (not localStorage).
 *
 * Honest scope statement: PBKDF2 with 600k iterations is acceptable
 * for Phase 0 stub validation but is NOT the production target. Argon2id
 * is the production target.
 */

const PBKDF2_ITERATIONS = 600_000; // OWASP 2024 minimum for SHA-256
const PBKDF2_SALT_PREFIX_CONTENT = "wt-content-v1:";
const AES_GCM_IV_BYTES = 12;

/** Opaque content-encryption key. Held only in memory; never persisted.
 *  Phase 1 will wrap this in a SessionKey type that auto-clears on
 *  logout / inactivity. */
export interface ContentKey {
  /** The CryptoKey for AES-GCM encrypt/decrypt operations. */
  readonly key: CryptoKey;
  /** Tag identifying which KDF + parameters produced this key, so a
   *  future re-derivation under different params can be detected. */
  readonly version: string;
}

/** Ciphertext envelope. The IV is unique per encryption operation and
 *  stored alongside the ciphertext. Version tag enables forward-compat
 *  if the cipher or AEAD parameters change in Phase 1+. */
export interface Ciphertext {
  readonly version: string;
  readonly iv: Uint8Array;
  readonly ciphertext: Uint8Array;
}

/** Derive a content-encryption key from a user passphrase + a per-user
 *  KDF salt. The salt should be stored server-side (it's not secret;
 *  it's just per-user). The passphrase should NEVER leave the browser.
 *
 *  Phase 0 uses PBKDF2-HMAC-SHA-256 because it's natively available in
 *  Web Crypto API. Phase 1 replaces with Argon2id (memory-hard, best
 *  practice for password-based KDF in 2024+). The version tag locks the
 *  KDF used; ciphertexts carry version too so they can be migrated. */
export async function deriveContentKey(
  passphrase: string,
  saltHex: string,
): Promise<ContentKey> {
  if (!passphrase) throw new Error("deriveContentKey: empty passphrase");
  if (!saltHex || saltHex.length < 16) {
    throw new Error("deriveContentKey: salt too short");
  }
  const salt = new TextEncoder().encode(PBKDF2_SALT_PREFIX_CONTENT + saltHex);
  const passwordBytes = new TextEncoder().encode(passphrase);
  const baseKey = await crypto.subtle.importKey(
    "raw",
    passwordBytes,
    { name: "PBKDF2" },
    false,
    ["deriveKey"],
  );
  const key = await crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt,
      iterations: PBKDF2_ITERATIONS,
      hash: "SHA-256",
    },
    baseKey,
    { name: "AES-GCM", length: 256 },
    false, // not extractable — key never leaves Web Crypto context
    ["encrypt", "decrypt"],
  );
  return { key, version: "pbkdf2-aesgcm-v1" };
}

/** Encrypt a UTF-8 string with the given content key. Returns a
 *  Ciphertext envelope. */
export async function encryptString(
  contentKey: ContentKey,
  plaintext: string,
): Promise<Ciphertext> {
  const iv = crypto.getRandomValues(new Uint8Array(AES_GCM_IV_BYTES));
  const ptBytes = new TextEncoder().encode(plaintext);
  const ctBuf = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    contentKey.key,
    ptBytes,
  );
  return {
    version: contentKey.version,
    iv,
    ciphertext: new Uint8Array(ctBuf),
  };
}

/** Decrypt a Ciphertext envelope with the given content key. Throws
 *  on AEAD authentication failure (modified ciphertext / wrong key). */
export async function decryptString(
  contentKey: ContentKey,
  envelope: Ciphertext,
): Promise<string> {
  if (envelope.version !== contentKey.version) {
    throw new Error(
      `decryptString: version mismatch (envelope=${envelope.version}, key=${contentKey.version})`,
    );
  }
  // Cast to BufferSource — TypeScript 5.7+ types Uint8Array as
  // Uint8Array<ArrayBufferLike> which doesn't unify with BufferSource
  // due to SharedArrayBuffer pollution. The runtime semantics are fine.
  const ptBuf = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: envelope.iv as BufferSource },
    contentKey.key,
    envelope.ciphertext as BufferSource,
  );
  return new TextDecoder().decode(ptBuf);
}

/** Encode a Ciphertext as a single hex-blob string for IndexedDB
 *  storage. Format: <version>$<iv-hex>$<ciphertext-hex>. */
export function encodeCiphertext(ct: Ciphertext): string {
  return [ct.version, toHex(ct.iv), toHex(ct.ciphertext)].join("$");
}

/** Decode a hex-blob string back into a Ciphertext envelope. */
export function decodeCiphertext(encoded: string): Ciphertext {
  const parts = encoded.split("$");
  if (parts.length !== 3) {
    throw new Error(`decodeCiphertext: bad format (${parts.length} parts)`);
  }
  const [version, ivHex, ctHex] = parts;
  return {
    version,
    iv: fromHex(ivHex),
    ciphertext: fromHex(ctHex),
  };
}

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

function fromHex(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) throw new Error("fromHex: odd length");
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}

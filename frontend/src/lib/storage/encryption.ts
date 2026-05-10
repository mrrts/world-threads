/**
 * Web-deployment content encryption helpers.
 *
 * Per the web-hosting architecture plan § XXI.2: a per-user `content_key`
 * is derived from the user's password via a separate KDF salt (distinct
 * from the auth-key derivation), held only in browser memory while the
 * session is active, never persisted to disk in the browser, and used
 * to encrypt all content stored in IndexedDB.
 *
 * Phase 1 item 4 (commit landing now): KDF is **Argon2id** via
 * @noble/hashes (pure-JS, ~30 kB gzipped, no native deps). Parameters
 * t=3 / m=19456 (19 MiB) / p=1 / dkLen=32 follow OWASP 2024 baseline
 * for password-based KDF in browser contexts. Version tag bumped from
 * `pbkdf2-aesgcm-v1` (Phase 0 stub) to `argon2id-aesgcm-v1`.
 *
 * The envelope's version tag is the migration anchor: if the KDF or
 * cipher parameters change in a future v2, ciphertexts authored under
 * v1 can still be decrypted by re-deriving a v1 key on demand (not
 * implemented yet — Phase 0 had no production data so no v1 migration
 * is needed today). Future v2 migrations will pass version through
 * the deriveContentKey signature.
 *
 * Honest scope: Argon2id is the production target per architecture
 * plan. The browser-only context limits memory parameters compared to
 * server-side recommendations (browsers can't reliably allocate 64+
 * MiB per derivation without OOM risk); 19 MiB is the safe middle.
 * If we ever offer server-assisted KDF (Phase 2+) we can use larger
 * memory parameters.
 */

import { argon2idAsync } from "@noble/hashes/argon2.js";

const ARGON2ID_PARAMS = {
  t: 3, // iterations
  m: 19_456, // 19 MiB memory cost
  p: 1, // parallelism
  dkLen: 32, // 32-byte output (256-bit AES key)
} as const;
const KDF_SALT_PREFIX_CONTENT = "wt-content-v1:";
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
 *  Argon2id parameters (t=3 / m=19MiB / p=1 / dkLen=32) follow OWASP
 *  2024 password-KDF baseline scaled for browser memory budgets. The
 *  derived raw bytes are imported as an AES-GCM-256 key marked
 *  non-extractable so it never leaves the Web Crypto context after
 *  import. Version tag "argon2id-aesgcm-v1" gets stamped on every
 *  Ciphertext envelope this key encrypts. */
export async function deriveContentKey(
  passphrase: string,
  saltHex: string,
): Promise<ContentKey> {
  if (!passphrase) throw new Error("deriveContentKey: empty passphrase");
  if (!saltHex || saltHex.length < 16) {
    throw new Error("deriveContentKey: salt too short");
  }
  const saltBytes = new TextEncoder().encode(KDF_SALT_PREFIX_CONTENT + saltHex);
  const passwordBytes = new TextEncoder().encode(passphrase);
  // Argon2id derives 32 raw bytes from passphrase + salt; we then
  // import those as a non-extractable AES-GCM key. The intermediate
  // raw bytes exist briefly in JS memory; defense-in-depth would zero
  // them after importKey but Web Crypto's importKey copies internally
  // so the JS reference is detached after this function returns.
  const rawKey = await argon2idAsync(passwordBytes, saltBytes, ARGON2ID_PARAMS);
  const key = await crypto.subtle.importKey(
    "raw",
    rawKey as BufferSource,
    { name: "AES-GCM", length: 256 },
    false, // not extractable — key never leaves Web Crypto context
    ["encrypt", "decrypt"],
  );
  return { key, version: "argon2id-aesgcm-v1" };
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

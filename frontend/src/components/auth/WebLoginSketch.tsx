/**
 * Web-deployment Phase 0 — login UI sketch.
 *
 * Per the web-hosting architecture plan § V (auth) + § XXI.2 (encryption).
 *
 * Stage of work: TOOLING ONLY. This component is a SKETCH that:
 *   - Shows the shape of the email + passphrase form for web-mode signup
 *     and login.
 *   - Demonstrates the post-auth flow: derive content_key, store in
 *     sessionContentKey, transition to the main app.
 *   - Is NOT WIRED into the running app. Nothing renders this component
 *     yet. App.tsx still routes Tauri-mode users to existing UI without
 *     any auth gate.
 *
 * Phase 1 will:
 *   - Wire this (or a refactor of it) into the App routing as a gate
 *     for web-mode users.
 *   - Connect to a real Axum server for signup/login (via
 *     transport.invoke, which already abstracts over Tauri vs HTTP).
 *   - Add forgot-password / verify-email / OAuth flows.
 *   - Replace PBKDF2 with Argon2id for content_key derivation.
 *   - Add an explicit "Save your recovery key" step for users who want
 *     offline recovery if they lose their passphrase (per § XXI.3).
 *
 * What this sketch DOES exercise honestly:
 *   - Form input + submit handling.
 *   - PublicUser / AuthSuccess type usage from the storage abstraction.
 *   - deriveContentKey() integration.
 *   - sessionContentKey.setKey() integration.
 *   - Web-only conditional via getRuntimeKind() — the component bails
 *     gracefully on Tauri runtime so importing it won't crash the
 *     existing desktop app.
 */

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Loader2 } from "lucide-react";
import { deriveContentKey } from "@/lib/storage/encryption";
import { sessionContentKey } from "@/lib/storage/sessionContentKey";
import { transport, getRuntimeKind } from "@/lib/transport";

type Mode = "signin" | "signup";

interface Props {
  /** Called after successful signin/signup with content_key derived
   *  and stored in sessionContentKey. Phase 1 routes to the main app. */
  onAuthenticated?: () => void;
}

/** Shape returned by the future Axum /auth/login endpoint. Mirrors
 *  the Rust AuthSuccess type at src-tauri/src/auth/mod.rs but with
 *  an additional content_kdf_salt field (server holds the per-user
 *  salt the browser uses to derive content_key — salt isn't secret;
 *  it just needs to be persistent + unique per user). */
interface ServerAuthResponse {
  user: {
    id: string;
    email: string;
    display_name: string;
    timezone: string;
  };
  session_token: string;
  expires_at: string;
  content_kdf_salt: string;
}

export function WebLoginSketch({ onAuthenticated }: Props) {
  const [mode, setMode] = useState<Mode>("signin");
  const [email, setEmail] = useState("");
  const [passphrase, setPassphrase] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Tauri-mode runs the desktop app's existing flow; web-only auth
  // doesn't apply. Render nothing rather than confuse desktop users.
  if (getRuntimeKind() === "tauri") {
    return null;
  }

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      // Step 1 — call the auth endpoint over the transport abstraction.
      // The existing transport singleton is already configured for the
      // current runtime; here we explicitly target the future Axum
      // /auth/<mode> endpoints. Phase 1 wires real backend; for Phase 0
      // sketch this throws since no backend exists.
      const cmd = mode === "signup" ? "auth/signup" : "auth/login";
      const args =
        mode === "signup"
          ? { email, password: passphrase, display_name: displayName, timezone: tz() }
          : { email, password: passphrase };
      const auth = await transport.invoke<ServerAuthResponse>(cmd, args);

      // Step 2 — derive content_key from passphrase + per-user salt.
      // The salt comes from the server response; it's not secret —
      // just persistent + unique per user. Argon2id replaces PBKDF2
      // in Phase 1 (per § XXI.2).
      const contentKey = await deriveContentKey(passphrase, auth.content_kdf_salt);

      // Step 3 — install the content_key in the session store. Now any
      // ContentStore created via createContentStore({contentKey}) will
      // be able to encrypt/decrypt content for this user.
      sessionContentKey.setKey(contentKey);

      // Step 4 — passphrase string is no longer needed; clear from
      // local state. The browser may still hold it in form-state-history
      // briefly; that's the cost of the password input. The CryptoKey
      // derived above is not extractable from Web Crypto context.
      setPassphrase("");

      onAuthenticated?.();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="mx-auto max-w-md space-y-6 p-8">
      <header className="space-y-2 text-center">
        <h1 className="text-2xl font-semibold text-foreground">
          {mode === "signin" ? "Sign in" : "Create your account"}
        </h1>
        <p className="text-sm text-muted-foreground">
          Your conversations are encrypted with your passphrase. We can&rsquo;t recover your
          content if you lose it &mdash; choose something memorable.
        </p>
      </header>

      <form onSubmit={onSubmit} className="space-y-4">
        {mode === "signup" && (
          <div className="space-y-1.5">
            <Label htmlFor="display_name">Display name</Label>
            <Input
              id="display_name"
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="What we should call you"
              autoComplete="name"
              required
            />
          </div>
        )}

        <div className="space-y-1.5">
          <Label htmlFor="email">Email</Label>
          <Input
            id="email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="you@example.com"
            autoComplete="email"
            required
          />
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="passphrase">Passphrase</Label>
          <Input
            id="passphrase"
            type="password"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            placeholder="Long enough to remember"
            autoComplete={mode === "signup" ? "new-password" : "current-password"}
            minLength={12}
            required
          />
          <p className="text-xs text-muted-foreground">
            Minimum 12 characters. Your passphrase derives a key that encrypts
            your conversations on this device. We store the encrypted blobs;
            we never see the plaintext.
          </p>
        </div>

        {error && (
          <div className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {error}
          </div>
        )}

        <Button type="submit" disabled={submitting} className="w-full">
          {submitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {mode === "signin" ? "Sign in" : "Create account"}
        </Button>
      </form>

      <div className="text-center text-sm text-muted-foreground">
        {mode === "signin" ? (
          <>
            New here?{" "}
            <button
              type="button"
              onClick={() => {
                setMode("signup");
                setError(null);
              }}
              className="underline hover:text-foreground"
            >
              Create an account
            </button>
          </>
        ) : (
          <>
            Already have an account?{" "}
            <button
              type="button"
              onClick={() => {
                setMode("signin");
                setError(null);
              }}
              className="underline hover:text-foreground"
            >
              Sign in
            </button>
          </>
        )}
      </div>
    </div>
  );
}

/** Best-effort timezone discovery from the browser. Falls back to UTC.
 *  Phase 1 surface lets users override this in account settings. */
function tz(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
  } catch {
    return "UTC";
  }
}

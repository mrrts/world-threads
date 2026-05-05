import { useState, useEffect, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { Field, FieldGroup } from "@/components/ui/field";
import { Save, Eye, EyeOff, Check, RefreshCw, Loader2, DatabaseBackup, Minus, Plus } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import type { ModelConfig, LocalModelInfo } from "@/lib/tauri";
import { api } from "@/lib/tauri";

interface Props {
  store: ReturnType<typeof useAppStore>;
}

export function SettingsPanel({ store }: Props) {
  const [apiKey, setApiKey] = useState(store.apiKey);
  const [googleApiKey, setGoogleApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [showGoogleKey, setShowGoogleKey] = useState(false);
  const [config, setConfig] = useState<ModelConfig>(store.modelConfig);
  const [dirty, setDirty] = useState(false);
  const [saved, setSaved] = useState(false);
  const [localModels, setLocalModels] = useState<LocalModelInfo[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);
  const [modelError, setModelError] = useState<string | null>(null);
  const [backups, setBackups] = useState<Array<{ file_name: string; timestamp: string }>>([]);
  const [selectedBackup, setSelectedBackup] = useState<string>("");
  const [restoringBackup, setRestoringBackup] = useState(false);
  const [backingUp, setBackingUp] = useState(false);
  const [conscienceEnabled, setConscienceEnabled] = useState(false);
  const [childrenMode, setChildrenMode] = useState(false);
  const [childrenModePasswordSet, setChildrenModePasswordSet] = useState(false);
  const [childrenModeDialog, setChildrenModeDialog] = useState<null | "enable" | "disable">(null);
  const [childrenModePassword, setChildrenModePassword] = useState("");
  const [childrenModePasswordConfirm, setChildrenModePasswordConfirm] = useState("");
  const [childrenModeError, setChildrenModeError] = useState<string | null>(null);
  const [childrenModeBusy, setChildrenModeBusy] = useState(false);

  useEffect(() => {
    setApiKey(store.apiKey);
    setConfig(store.modelConfig);
  }, [store.apiKey, store.modelConfig]);

  useEffect(() => {
    api.getGoogleApiKey().then(setGoogleApiKey);
    api.listBackups().then((list) => { setBackups(list); if (list.length > 0) setSelectedBackup(list[0].file_name); });
    api.getSetting("conscience_pass_enabled").then((v) => {
      // Default OFF — feature is opt-in because it roughly doubles the
      // per-reply token cost (one extra memory-model grader call + the
      // occasional dialogue-model regenerate).
      setConscienceEnabled(v === "true" || v === "on");
    });
    api.getSetting("children_mode").then((v) => {
      setChildrenMode(v === "true" || v === "on" || v === "1");
    });
    api.isChildrenModePasswordSet().then(setChildrenModePasswordSet);
  }, []);

  const closeChildrenModeDialog = () => {
    setChildrenModeDialog(null);
    setChildrenModePassword("");
    setChildrenModePasswordConfirm("");
    setChildrenModeError(null);
    setChildrenModeBusy(false);
  };

  const submitChildrenModeDialog = async () => {
    setChildrenModeError(null);
    if (childrenModeDialog === "enable") {
      if (!childrenModePasswordSet) {
        if (childrenModePassword.length < 6) {
          setChildrenModeError("Password must be at least 6 characters.");
          return;
        }
        if (childrenModePassword !== childrenModePasswordConfirm) {
          setChildrenModeError("Passwords do not match.");
          return;
        }
      }
      setChildrenModeBusy(true);
      try {
        await api.enableChildrenModeWithPassword(childrenModePassword);
        setChildrenMode(true);
        setChildrenModePasswordSet(true);
        closeChildrenModeDialog();
      } catch (e) {
        setChildrenModeError(String(e));
        setChildrenModeBusy(false);
      }
    } else if (childrenModeDialog === "disable") {
      setChildrenModeBusy(true);
      try {
        await api.disableChildrenModeWithPassword(childrenModePassword);
        setChildrenMode(false);
        closeChildrenModeDialog();
      } catch (e) {
        setChildrenModeError(String(e));
        setChildrenModeBusy(false);
      }
    }
  };

  const fetchLocalModels = useCallback(async (url: string) => {
    setLoadingModels(true);
    setModelError(null);
    try {
      const models = await api.listLocalModels(url);
      setLocalModels(models);
    } catch (e) {
      setModelError("Could not connect to LM Studio");
      setLocalModels([]);
    } finally {
      setLoadingModels(false);
    }
  }, []);

  useEffect(() => {
    if (config.ai_provider === "lmstudio") {
      fetchLocalModels(config.lmstudio_url);
    }
  }, [config.ai_provider, config.lmstudio_url, fetchLocalModels]);

  const handleSave = async () => {
    await store.setApiKey(apiKey);
    await api.setGoogleApiKey(googleApiKey);
    await store.setModelConfig(config);
    setDirty(false);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const isLocal = config.ai_provider === "lmstudio";

  const modelSelect = (value: string, onChange: (v: string) => void, placeholder?: string) => {
    if (!isLocal) {
      return (
        <Input
          value={value}
          onChange={(e) => { onChange(e.target.value); setDirty(true); }}
          className="font-mono text-xs"
        />
      );
    }

    return (
      <Select
        value={value}
        onChange={(e) => { onChange(e.target.value); setDirty(true); }}
        className="font-mono text-xs"
      >
        <option value="" disabled>{placeholder ?? "Select a model..."}</option>
        {localModels.map((m) => (
          <option key={m.id} value={m.id}>{m.id}</option>
        ))}
        {value && !localModels.some((m) => m.id === value) && (
          <option value={value}>{value} (not loaded)</option>
        )}
      </Select>
    );
  };

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div className="px-6 py-3 border-b border-border flex items-center justify-between">
        <h1 className="font-semibold">Settings</h1>
        <div className="flex items-center gap-2">
          {saved && (
            <span className="text-xs text-green-400 bg-green-400/10 px-2 py-0.5 rounded-full flex items-center gap-1">
              <Check size={10} /> Saved
            </span>
          )}
          {dirty && !saved && (
            <span className="text-xs text-primary bg-primary/10 px-2 py-0.5 rounded-full">
              Unsaved changes
            </span>
          )}
          <Button size="sm" onClick={handleSave} disabled={!dirty}>
            <Save size={14} className="mr-1.5" /> Save
          </Button>
        </div>
      </div>

      <ScrollArea className="flex-1 px-6 py-6">
        <div className="max-w-xl space-y-10">
          <FieldGroup label="Chat Provider">
            <p className="text-xs text-muted-foreground/60 -mt-2">
              Choose where to run chat completions (dialogue, world tick, memory). Embeddings and image generation always use OpenAI.
            </p>
            <div className="flex gap-2">
              <button
                type="button"
                className={`flex-1 px-4 py-2.5 rounded-lg border text-sm font-medium transition-colors cursor-pointer ${
                  !isLocal
                    ? "border-primary bg-primary/10 text-primary"
                    : "border-border bg-card/50 text-muted-foreground hover:text-foreground"
                }`}
                onClick={() => { setConfig({ ...config, ai_provider: "openai" }); setDirty(true); }}
              >
                OpenAI API
              </button>
              <button
                type="button"
                className={`flex-1 px-4 py-2.5 rounded-lg border text-sm font-medium transition-colors cursor-pointer ${
                  isLocal
                    ? "border-primary bg-primary/10 text-primary"
                    : "border-border bg-card/50 text-muted-foreground hover:text-foreground"
                }`}
                onClick={() => { setConfig({ ...config, ai_provider: "lmstudio" }); setDirty(true); }}
              >
                LM Studio (Local)
              </button>
            </div>
          </FieldGroup>

          <FieldGroup label="API Key">
            <Field label="OpenAI API Key" hint={isLocal ? "Still required for embeddings and image generation. Stored securely in your local keychain." : "Stored securely in your local keychain. Only sent to the OpenAI API."}>
              <div className="relative">
                <Input
                  type={showKey ? "text" : "password"}
                  value={apiKey}
                  onChange={(e) => { setApiKey(e.target.value); setDirty(true); }}
                  placeholder="sk-..."
                  className="pr-10 font-mono text-xs"
                />
                <button
                  type="button"
                  className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground cursor-pointer transition-colors"
                  onClick={() => setShowKey(!showKey)}
                >
                  {showKey ? <EyeOff size={15} /> : <Eye size={15} />}
                </button>
              </div>
            </Field>
            <Field label="Google AI Studio API Key" hint="For Gemini models and video generation. Stored securely in your local keychain.">
              <div className="relative">
                <Input
                  type={showGoogleKey ? "text" : "password"}
                  value={googleApiKey}
                  onChange={(e) => { setGoogleApiKey(e.target.value); setDirty(true); }}
                  placeholder="AIza..."
                  className="pr-10 font-mono text-xs"
                />
                <button
                  type="button"
                  className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground cursor-pointer transition-colors"
                  onClick={() => setShowGoogleKey(!showGoogleKey)}
                >
                  {showGoogleKey ? <EyeOff size={15} /> : <Eye size={15} />}
                </button>
              </div>
            </Field>
          </FieldGroup>

          {isLocal && (
            <FieldGroup label="LM Studio Connection">
              <Field label="Server URL" hint="The address where LM Studio is running.">
                <div className="flex gap-2">
                  <Input
                    value={config.lmstudio_url}
                    onChange={(e) => { setConfig({ ...config, lmstudio_url: e.target.value }); setDirty(true); }}
                    placeholder="http://127.0.0.1:1234"
                    className="font-mono text-xs flex-1"
                  />
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => fetchLocalModels(config.lmstudio_url)}
                    disabled={loadingModels}
                  >
                    {loadingModels ? <Loader2 size={14} className="animate-spin" /> : <RefreshCw size={14} />}
                  </Button>
                </div>
              </Field>
              {modelError && (
                <p className="text-xs text-destructive -mt-1">{modelError}</p>
              )}
              {localModels.length > 0 && (
                <p className="text-xs text-muted-foreground/60 -mt-1">
                  {localModels.length} model{localModels.length !== 1 ? "s" : ""} available
                </p>
              )}
              <Field
                label="Context Window"
                hint="How many tokens your local model can hold. We aim well below this when chunking long novelization prompts."
              >
                <ContextWindowControl
                  valueTokens={config.lmstudio_context_tokens}
                  onChange={(v) => { setConfig({ ...config, lmstudio_context_tokens: v }); setDirty(true); }}
                />
              </Field>
            </FieldGroup>
          )}

          <FieldGroup label={isLocal ? "Chat Models (LM Studio)" : "Models"}>
            <p className="text-xs text-muted-foreground/60 -mt-2">
              {isLocal
                ? "These run on your local LM Studio server."
                : "Configure which OpenAI model to use for each function. Use cheaper models for background tasks."}
            </p>
            <div className="grid grid-cols-2 gap-4">
              <Field label="Dialogue" hint="Character responses — higher quality">
                {modelSelect(config.dialogue_model, (v) => setConfig({ ...config, dialogue_model: v }))}
              </Field>
              <Field label="Frontier override" hint="Used when a chat picks 'Frontier' in its settings. Free-text — type any OpenAI model ID (gpt-4o, gpt-5, gpt-5.4, etc).">
                <Input
                  value={config.dialogue_model_frontier ?? ""}
                  onChange={(e) => { setConfig({ ...config, dialogue_model_frontier: e.target.value }); setDirty(true); }}
                  className="font-mono text-xs"
                  placeholder="gpt-4o"
                />
              </Field>
              <Field label="World Tick" hint="Off-screen simulation — cheaper">
                {modelSelect(config.tick_model, (v) => setConfig({ ...config, tick_model: v }))}
              </Field>
              <Field label="Memory" hint="Summaries & record updates — cheaper">
                {modelSelect(config.memory_model, (v) => setConfig({ ...config, memory_model: v }))}
              </Field>
            </div>
          </FieldGroup>

          <FieldGroup label={isLocal ? "OpenAI Models" : ""}>
            {isLocal && (
              <p className="text-xs text-muted-foreground/60 -mt-2">
                These always use OpenAI — local servers don't support embeddings or image generation.
              </p>
            )}
            <div className="grid grid-cols-2 gap-4">
              <Field label="Embeddings" hint="Vector search — always OpenAI">
                <Input
                  value={config.embedding_model}
                  onChange={(e) => { setConfig({ ...config, embedding_model: e.target.value }); setDirty(true); }}
                  className="font-mono text-xs"
                />
              </Field>
              <Field label="Image Generation" hint="Portraits, world images, avatars">
                <Input
                  value={config.image_model}
                  onChange={(e) => { setConfig({ ...config, image_model: e.target.value }); setDirty(true); }}
                  className="font-mono text-xs"
                />
              </Field>
              <Field label="Vision" hint="Image analysis — always OpenAI">
                <Input
                  value={config.vision_model}
                  onChange={(e) => { setConfig({ ...config, vision_model: e.target.value }); setDirty(true); }}
                  className="font-mono text-xs"
                />
              </Field>
            </div>
          </FieldGroup>

          <FieldGroup label="Cost Controls">
            <div className="flex items-center justify-between py-2 px-4 rounded-lg border border-border bg-card/50">
              <div>
                <p className="text-sm font-medium">Budget Mode</p>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Reduce world tick frequency and summary updates to save tokens
                </p>
              </div>
              <Switch
                checked={store.budgetMode}
                onCheckedChange={(checked) => store.setBudgetMode(checked)}
              />
            </div>
          </FieldGroup>

          <FieldGroup label="Craft (Optional)">
            <div className="flex items-start justify-between gap-4 py-3 px-4 rounded-lg border border-border bg-card/50">
              <div className="space-y-1.5">
                <p className="text-sm font-medium">Children Mode (Custodiem)</p>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  Injects a top-stack child-presence invariant directly below the Mission Formula on every LLM call. Enforces severe-clean boundaries: no counterfeit intimacy, no manipulative specialness, no pseudo-bonding.
                </p>
                <p className="text-xs text-amber-500/80 leading-relaxed">
                  <strong>Password-locked:</strong> turning Children Mode on or off requires a password. <strong>If you forget the password, Children Mode cannot be turned off without resetting the app's data.</strong>
                </p>
              </div>
              <Switch
                checked={childrenMode}
                onCheckedChange={(checked) => {
                  setChildrenModeDialog(checked ? "enable" : "disable");
                }}
              />
            </div>
            <div className="flex items-start justify-between gap-4 py-3 px-4 rounded-lg border border-border bg-card/50">
              <div className="space-y-1.5">
                <p className="text-sm font-medium">Conscience Pass</p>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  A quality guard: after every character reply, a second cheaper model reads the draft and checks it against the app's five craft invariants (agape, soundness, daylight, truth-test, cosmology). If the draft drifts, the original model rewrites it once with a specific correction note. Catches the subtle drift that pure prompting misses.
                </p>
                <p className="text-xs text-amber-500/80 leading-relaxed">
                  <strong>Token cost:</strong> roughly doubles per-reply spend — one extra grader call every time, plus the occasional regenerate. Off by default. Turn on when the craft payoff is worth the burn, off when it isn't.
                </p>
              </div>
              <Switch
                checked={conscienceEnabled}
                onCheckedChange={async (checked) => {
                  setConscienceEnabled(checked);
                  await api.setSetting("conscience_pass_enabled", checked ? "true" : "false");
                }}
              />
            </div>
          </FieldGroup>

          <FieldGroup label="Notifications">
            <div className="flex items-center justify-between py-2 px-4 rounded-lg border border-border bg-card/50">
              <div>
                <p className="text-sm font-medium">Message Chime</p>
                <p className="text-xs text-muted-foreground mt-0.5">
                  Play a sound when a new message arrives
                </p>
              </div>
              <Switch
                checked={store.notifyOnMessage}
                onCheckedChange={(checked) => store.setNotifyOnMessage(checked)}
              />
            </div>
          </FieldGroup>

          <FieldGroup label="Backups">
            <p className="text-xs text-muted-foreground/60 -mt-2">
              Backups are created automatically every hour and on each app launch.
            </p>
            <div className="space-y-3">
              {backups.length > 0 ? (
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1.5">Select Backup</label>
                  <select
                    value={selectedBackup}
                    onChange={(e) => setSelectedBackup(e.target.value)}
                    className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
                  >
                    {backups.map((b, i) => (
                      <option key={b.file_name} value={b.file_name}>
                        {b.timestamp} UTC{i === 0 ? " (latest)" : ""}
                      </option>
                    ))}
                  </select>
                </div>
              ) : (
                <p className="text-xs text-muted-foreground">No backups available yet.</p>
              )}
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  disabled={backingUp}
                  onClick={async () => {
                    setBackingUp(true);
                    try {
                      await api.backupNow();
                      const list = await api.listBackups();
                      setBackups(list);
                      if (list.length > 0) setSelectedBackup(list[0].file_name);
                    } catch (e) {
                      window.alert(`Backup failed: ${e}`);
                    } finally {
                      setBackingUp(false);
                    }
                  }}
                >
                  {backingUp ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <Save size={14} className="mr-1.5" />}
                  Backup Now
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={restoringBackup || !selectedBackup}
                  onClick={async () => {
                    const backup = backups.find((b) => b.file_name === selectedBackup);
                    const confirmed = window.confirm(
                      `Restore backup from ${backup?.timestamp ?? "unknown"}? The app will need to restart. Any changes since this backup will be lost.`
                    );
                    if (!confirmed) return;
                    setRestoringBackup(true);
                    try {
                      await api.restoreBackup(selectedBackup);
                      window.alert("Backup restored. Please restart the app.");
                    } catch (e) {
                      window.alert(`Failed to restore backup: ${e}`);
                    } finally {
                      setRestoringBackup(false);
                    }
                  }}
                >
                  {restoringBackup ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <DatabaseBackup size={14} className="mr-1.5" />}
                  Restore Selected
                </Button>
              </div>
            </div>
          </FieldGroup>
        </div>
      </ScrollArea>

      {childrenModeDialog && (
        <ChildrenModePasswordDialog
          mode={childrenModeDialog}
          passwordAlreadySet={childrenModePasswordSet}
          password={childrenModePassword}
          passwordConfirm={childrenModePasswordConfirm}
          error={childrenModeError}
          busy={childrenModeBusy}
          onPasswordChange={setChildrenModePassword}
          onPasswordConfirmChange={setChildrenModePasswordConfirm}
          onSubmit={submitChildrenModeDialog}
          onCancel={closeChildrenModeDialog}
        />
      )}
    </div>
  );
}

interface ChildrenModePasswordDialogProps {
  mode: "enable" | "disable";
  passwordAlreadySet: boolean;
  password: string;
  passwordConfirm: string;
  error: string | null;
  busy: boolean;
  onPasswordChange: (v: string) => void;
  onPasswordConfirmChange: (v: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}

function ChildrenModePasswordDialog({
  mode,
  passwordAlreadySet,
  password,
  passwordConfirm,
  error,
  busy,
  onPasswordChange,
  onPasswordConfirmChange,
  onSubmit,
  onCancel,
}: ChildrenModePasswordDialogProps) {
  const isEnable = mode === "enable";
  const needsConfirm = isEnable && !passwordAlreadySet;
  const title = isEnable ? "Turn on Children Mode" : "Turn off Children Mode";
  const cta = isEnable ? "Turn on" : "Turn off";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg space-y-4">
        <h2 className="text-lg font-semibold">{title}</h2>
        {needsConfirm ? (
          <div className="space-y-2 rounded-md border border-amber-500/40 bg-amber-500/10 p-3">
            <p className="text-sm font-medium text-amber-500">
              This password cannot be recovered.
            </p>
            <p className="text-xs text-muted-foreground leading-relaxed">
              Choose a password you will remember. <strong>If you forget it, Children Mode cannot be turned off again without resetting the app's data.</strong> The same password will be required to turn Children Mode off later.
            </p>
          </div>
        ) : (
          <p className="text-xs text-muted-foreground leading-relaxed">
            {isEnable
              ? "Enter the Children Mode password you set previously."
              : "Enter the Children Mode password to turn it off."}
          </p>
        )}
        <Field label="Password">
          <Input
            type="password"
            value={password}
            autoFocus
            onChange={(e) => onPasswordChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !busy) onSubmit();
            }}
          />
        </Field>
        {needsConfirm && (
          <Field label="Confirm password">
            <Input
              type="password"
              value={passwordConfirm}
              onChange={(e) => onPasswordConfirmChange(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !busy) onSubmit();
              }}
            />
          </Field>
        )}
        {error && <p className="text-xs text-destructive">{error}</p>}
        <div className="flex justify-end gap-2 pt-2">
          <Button size="sm" variant="outline" onClick={onCancel} disabled={busy}>
            Cancel
          </Button>
          <Button size="sm" onClick={onSubmit} disabled={busy || password.length === 0}>
            {busy ? <Loader2 size={14} className="animate-spin mr-1.5" /> : null}
            {cta}
          </Button>
        </div>
      </div>
    </div>
  );
}

/** +/- stepper for LM Studio context window. Steps in 10,000-token increments,
 *  min 10k, max 1M. Displays the value as "40k" / "120k" for readability. */
function ContextWindowControl({ valueTokens, onChange }: { valueTokens: number; onChange: (v: number) => void }) {
  const STEP = 10_000;
  const MIN = 10_000;
  const MAX = 1_000_000;
  const snap = (v: number) => Math.max(MIN, Math.min(MAX, Math.round(v / STEP) * STEP));
  const formatted = `${Math.round(valueTokens / 1000)}k`;
  return (
    <div className="flex items-center gap-2">
      <Button
        type="button"
        variant="outline"
        size="icon"
        className="h-8 w-8"
        onClick={() => onChange(snap(valueTokens - STEP))}
        disabled={valueTokens <= MIN}
        aria-label="Decrease context window"
      >
        <Minus size={14} />
      </Button>
      <div className="min-w-[64px] text-center font-mono text-sm bg-muted rounded-md py-1.5 border border-border">
        {formatted}
      </div>
      <Button
        type="button"
        variant="outline"
        size="icon"
        className="h-8 w-8"
        onClick={() => onChange(snap(valueTokens + STEP))}
        disabled={valueTokens >= MAX}
        aria-label="Increase context window"
      >
        <Plus size={14} />
      </Button>
    </div>
  );
}

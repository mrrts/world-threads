import { useState, useEffect, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { Field, FieldGroup } from "@/components/ui/field";
import { Save, Eye, EyeOff, Check, RefreshCw, Loader2 } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import type { ModelConfig, LocalModelInfo } from "@/lib/tauri";
import { api } from "@/lib/tauri";

interface Props {
  store: ReturnType<typeof useAppStore>;
}

export function SettingsPanel({ store }: Props) {
  const [apiKey, setApiKey] = useState(store.apiKey);
  const [showKey, setShowKey] = useState(false);
  const [config, setConfig] = useState<ModelConfig>(store.modelConfig);
  const [dirty, setDirty] = useState(false);
  const [saved, setSaved] = useState(false);
  const [localModels, setLocalModels] = useState<LocalModelInfo[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);
  const [modelError, setModelError] = useState<string | null>(null);

  useEffect(() => {
    setApiKey(store.apiKey);
    setConfig(store.modelConfig);
  }, [store.apiKey, store.modelConfig]);

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
            <Field label="OpenAI API Key" hint={isLocal ? "Still required for embeddings and image generation." : "Stored locally. Only sent to the OpenAI API."}>
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
              <Field label="World Tick" hint="Off-screen simulation — cheaper">
                {modelSelect(config.tick_model, (v) => setConfig({ ...config, tick_model: v }))}
              </Field>
              <Field label="Memory" hint="Summaries & canon updates — cheaper">
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
        </div>
      </ScrollArea>
    </div>
  );
}

import { useState, useEffect, useCallback, useRef } from "react";
import EmojiPicker, { EmojiStyle, Theme } from "emoji-picker-react";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Field, FieldGroup } from "@/components/ui/field";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from "@/components/ui/dialog";
import { Plus, X, BookTemplate, ImagePlus, Loader2, Check, Images, Shuffle, Trash2, AlertTriangle, MessageSquareX, RotateCcw, PenLine, Volume2, Square } from "lucide-react";
import { CHARACTER_TEMPLATES, type CharacterTemplate } from "@/lib/character-templates";
import { api, type Character, type CharacterState, type PortraitInfo, type GalleryItem, type InventoryItem, type JournalEntry } from "@/lib/tauri";
import type { useAppStore } from "@/hooks/use-app-store";
import { InventoryEditor } from "@/components/character/InventoryEditor";
import { DerivationCard } from "@/components/DerivationCard";

interface Props {
  store: ReturnType<typeof useAppStore>;
}

export function CharacterEditor({ store }: Props) {
  const ch = store.activeCharacter;
  const [form, setForm] = useState<Partial<Character>>({});
  const [dirty, setDirty] = useState(false);
  // Bumped after regenerate_character_derivation_cmd succeeds so the
  // DerivationCard's refetchKey changes and the card re-runs its load.
  const [derivationVersion, setDerivationVersion] = useState(0);
  const [showTemplates, setShowTemplates] = useState(false);
  const [templateSearch, setTemplateSearch] = useState("");
  const [portraits, setPortraits] = useState<PortraitInfo[]>([]);
  const [generatingPortrait, setGeneratingPortrait] = useState(false);
  const [generatingVariation, setGeneratingVariation] = useState(false);
  const [regeneratingVisualDesc, setRegeneratingVisualDesc] = useState(false);
  const [variationPreview, setVariationPreview] = useState<PortraitInfo | null>(null);
  const [showGallery, setShowGallery] = useState(false);
  const [showWorldGallery, setShowWorldGallery] = useState(false);
  const [worldGalleryItems, setWorldGalleryItems] = useState<GalleryItem[]>([]);
  const [loadingWorldGallery, setLoadingWorldGallery] = useState(false);
  const [showPoseModal, setShowPoseModal] = useState(false);
  const [poseDescription, setPoseDescription] = useState("");
  const [generatingPose, setGeneratingPose] = useState(false);
  const [showClearChat, setShowClearChat] = useState(false);
  const [showDeleteChar, setShowDeleteChar] = useState(false);
  const [ttsVoice, setTtsVoice] = useState("ash");
  const [ttsModel, setTtsModel] = useState("gpt-4o-mini-tts");
  const [showSignatureEmojiPicker, setShowSignatureEmojiPicker] = useState(false);
  const signatureEmojiPickerRef = useRef<HTMLDivElement>(null);
  const [journalEntries, setJournalEntries] = useState<JournalEntry[]>([]);
  const [journalLoading, setJournalLoading] = useState(false);
  const [journalGenerating, setJournalGenerating] = useState(false);

  // Close signature-emoji picker on outside click.
  useEffect(() => {
    if (!showSignatureEmojiPicker) return;
    const handler = (e: MouseEvent) => {
      if (signatureEmojiPickerRef.current && !signatureEmojiPickerRef.current.contains(e.target as Node)) {
        setShowSignatureEmojiPicker(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showSignatureEmojiPicker]);
  const [showVoiceExplorer, setShowVoiceExplorer] = useState(false);
  const [samplePlaying, setSamplePlaying] = useState<string | null>(null);
  const [sampleLoading, setSampleLoading] = useState<string | null>(null);
  const [sampleTone, setSampleTone] = useState("Auto");
  const sampleAudioRef = useRef<HTMLAudioElement | null>(null);

  const loadPortraits = useCallback(async (characterId: string) => {
    try {
      const list = await api.listPortraits(characterId);
      setPortraits(list);
    } catch {
      setPortraits([]);
    }
  }, []);

  useEffect(() => {
    if (ch) {
      const toArray = (v: unknown): string[] => Array.isArray(v) ? v : [];
      setForm({
        display_name: ch.display_name,
        identity: ch.identity,
        voice_rules: toArray(ch.voice_rules),
        boundaries: toArray(ch.boundaries),
        backstory_facts: toArray(ch.backstory_facts),
        avatar_color: ch.avatar_color,
        sex: ch.sex ?? "male",
        state: structuredClone(ch.state),
        visual_description: ch.visual_description ?? "",
        signature_emoji: ch.signature_emoji ?? "",
        action_beat_density: (ch.action_beat_density ?? "normal") as "low" | "normal" | "high",
        has_read_empiricon: ch.has_read_empiricon ?? false,
      });
      setDirty(false);
      loadPortraits(ch.character_id);
      api.getSetting(`voice.${ch.character_id}`).then((v) => setTtsVoice(v || "ash"));
      api.getSetting(`tts_model.${ch.character_id}`).then((v) => setTtsModel(v || "gpt-4o-mini-tts"));
      setJournalLoading(true);
      api.listCharacterJournals(ch.character_id, 30)
        .then((entries) => setJournalEntries(entries))
        .catch(() => setJournalEntries([]))
        .finally(() => setJournalLoading(false));
    }
  }, [ch?.character_id, loadPortraits]);

  const handleGenerateJournal = async () => {
    if (!ch || !store.apiKey) return;
    setJournalGenerating(true);
    try {
      const entry = await api.generateCharacterJournal(store.apiKey, ch.character_id);
      // Replace-or-prepend: upsert by world_day.
      setJournalEntries((prev) => {
        const filtered = prev.filter((e) => e.world_day !== entry.world_day);
        return [entry, ...filtered].sort((a, b) => b.world_day - a.world_day);
      });
    } catch (e) {
      store.setError?.(String(e));
    } finally {
      setJournalGenerating(false);
    }
  };

  if (!ch) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <div className="text-center space-y-2">
          <p className="text-lg">No character selected</p>
          <p className="text-sm text-muted-foreground/60">Select a character to edit their details</p>
        </div>
      </div>
    );
  }

  const activePortrait = portraits.find((p) => p.is_active);

  const handleGeneratePortrait = async () => {
    if (!ch || !store.apiKey) return;
    setGeneratingPortrait(true);
    try {
      const portrait = await api.generatePortrait(store.apiKey, ch.character_id, {
        display_name: form.display_name ?? ch.display_name,
        identity: form.identity ?? ch.identity,
        backstory_facts: form.backstory_facts ?? ch.backstory_facts,
      });
      setPortraits((prev) => [portrait, ...prev.map((p) => ({ ...p, is_active: false }))]);
      await store.refreshPortrait(ch.character_id);
      // Fire and forget — other characters need to know what this one
      // looks like. Caches on the backend so repeat calls no-op.
      store.refreshVisualDescription(ch.character_id);
    } catch (e) {
      store.setError?.(String(e));
    } finally {
      setGeneratingPortrait(false);
    }
  };

  const handleGenerateVariation = async () => {
    if (!ch || !store.apiKey || !activePortrait) return;
    setGeneratingVariation(true);
    try {
      const portrait = await api.generatePortraitVariation(store.apiKey, ch.character_id);
      setVariationPreview(portrait);
    } catch (e) {
      store.setError?.(String(e));
    } finally {
      setGeneratingVariation(false);
    }
  };

  const handleGenerateWithPose = async (pose: string) => {
    if (!ch || !store.apiKey || !activePortrait) return;
    setGeneratingPose(true);
    setShowPoseModal(false);
    try {
      const portrait = await api.generatePortraitWithPose(store.apiKey, ch.character_id, pose);
      setVariationPreview(portrait);
    } catch (e) {
      store.setError?.(String(e));
    } finally {
      setGeneratingPose(false);
    }
  };

  const handleKeepVariation = () => {
    if (!variationPreview) return;
    setPortraits((prev) => [variationPreview, ...prev]);
    setVariationPreview(null);
    if (ch) store.refreshVisualDescription(ch.character_id);
  };

  const handleDiscardVariation = async () => {
    if (!variationPreview) return;
    try {
      await api.archiveGalleryItem(variationPreview.portrait_id, "character");
    } catch {
      // best-effort cleanup
    }
    setVariationPreview(null);
  };

  const handleSelectPortrait = async (portrait: PortraitInfo) => {
    if (!ch) return;
    try {
      await api.setActivePortrait(ch.character_id, portrait.portrait_id);
      setPortraits((prev) =>
        prev.map((p) => ({ ...p, is_active: p.portrait_id === portrait.portrait_id }))
      );
      setShowGallery(false);
      await store.refreshPortrait(ch.character_id);
      store.refreshVisualDescription(ch.character_id);
    } catch (e) {
      store.setError?.(String(e));
    }
  };

  const handleOpenWorldGallery = async () => {
    const worldId = store.activeWorld?.world_id;
    if (!worldId) return;
    setLoadingWorldGallery(true);
    setShowWorldGallery(true);
    try {
      setWorldGalleryItems(await api.listWorldGallery(worldId));
    } catch {
    } finally {
      setLoadingWorldGallery(false);
    }
  };

  const handleSelectFromWorldGallery = async (item: GalleryItem) => {
    if (!ch || !item.file_name) return;
    try {
      const portrait = await api.setPortraitFromGallery(ch.character_id, item.file_name);
      setPortraits((prev) => [portrait, ...prev.map((p) => ({ ...p, is_active: false }))]);
      setShowWorldGallery(false);
      await store.refreshPortrait(ch.character_id);
      store.refreshVisualDescription(ch.character_id);
    } catch (e) {
      store.setError?.(String(e));
    }
  };

  const update = (patch: Partial<Character>) => {
    setForm((f) => ({ ...f, ...patch }));
    setDirty(true);
  };

  // Auto-save with debounce
  useEffect(() => {
    if (!dirty || !ch) return;
    const timer = setTimeout(async () => {
      await store.updateCharacter({ ...ch, ...form } as Character);
      setDirty(false);
    }, 800);
    return () => clearTimeout(timer);
  }, [form, dirty]);

  const applyTemplate = (template: CharacterTemplate) => {
    setForm((f) => ({
      ...f,
      display_name: f.display_name || template.name.replace(/^The /, ""),
      identity: template.identity,
      voice_rules: [...template.voice_rules],
      boundaries: [...template.boundaries],
      backstory_facts: [...template.backstory_facts],
      avatar_color: template.avatar_color,
      state: {
        mood: template.mood,
        trust_user: template.trust_user,
        goals: [...template.goals],
        open_loops: [...template.open_loops],
        last_seen: (f.state as Character["state"])?.last_seen ?? { day_index: 1, time_of_day: "MORNING" },
      },
    }));
    setDirty(true);
    setShowTemplates(false);
  };

  const filteredTemplates = templateSearch.trim()
    ? CHARACTER_TEMPLATES.filter(
        (t) =>
          t.name.toLowerCase().includes(templateSearch.toLowerCase()) ||
          t.tagline.toLowerCase().includes(templateSearch.toLowerCase()),
      )
    : CHARACTER_TEMPLATES;

  return (
    <>
      <div className="flex-1 flex flex-col min-h-0">
        <div className="px-6 py-3 border-b border-border flex items-center justify-between">
          <div className="flex items-center gap-3">
            {activePortrait?.data_url ? (
              <button
                onClick={() => setShowGallery(true)}
                className="w-10 h-10 rounded-full overflow-hidden ring-2 ring-primary/30 hover:ring-primary/60 transition-all cursor-pointer flex-shrink-0"
              >
                <img src={activePortrait.data_url} alt="" className="w-full h-full object-cover" />
              </button>
            ) : (
              <span
                className="w-4 h-4 rounded-full ring-2 ring-white/10"
                style={{ backgroundColor: form.avatar_color ?? ch.avatar_color }}
              />
            )}
            <div>
              <h1 className="font-semibold">{form.display_name ?? ch.display_name}</h1>
              <span className="text-xs text-muted-foreground/50">Character Details</span>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline" onClick={() => { setTemplateSearch(""); setShowTemplates(true); }}>
              <BookTemplate size={14} className="mr-1.5" /> Starter Templates
            </Button>
          </div>
        </div>

        <ScrollArea className="flex-1 px-6 py-6">
          <div className="max-w-2xl space-y-8">
            {/* Portrait Section */}
            <FieldGroup label="Portrait">
              <div className="flex items-start gap-4">
                {activePortrait?.data_url ? (
                  <button
                    onClick={() => setShowGallery(true)}
                    className="w-32 h-32 rounded-2xl overflow-hidden ring-2 ring-border hover:ring-primary/50 transition-all cursor-pointer flex-shrink-0"
                  >
                    <img src={activePortrait.data_url} alt="" className="w-full h-full object-cover" />
                  </button>
                ) : (
                  <div className="w-32 h-32 rounded-2xl border-2 border-dashed border-border flex items-center justify-center flex-shrink-0">
                    <span className="text-muted-foreground/40 text-xs text-center px-2">No portrait yet</span>
                  </div>
                )}
                <div className="flex flex-col gap-2 pt-1">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={handleGeneratePortrait}
                    disabled={generatingPortrait || !store.apiKey}
                  >
                    {generatingPortrait ? (
                      <><Loader2 size={14} className="mr-1.5 animate-spin" /> Generating...</>
                    ) : (
                      <><ImagePlus size={14} className="mr-1.5" /> Generate Portrait</>
                    )}
                  </Button>
                  {activePortrait && (<>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={handleGenerateVariation}
                      disabled={generatingVariation || generatingPortrait || !store.apiKey}
                    >
                      {generatingVariation ? (
                        <><Loader2 size={14} className="mr-1.5 animate-spin" /> Generating...</>
                      ) : (
                        <><Shuffle size={14} className="mr-1.5" /> New Pose</>
                      )}
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => { setPoseDescription(""); setShowPoseModal(true); }}
                      disabled={generatingPose || generatingVariation || generatingPortrait || !store.apiKey}
                    >
                      {generatingPose ? (
                        <><Loader2 size={14} className="mr-1.5 animate-spin" /> Generating...</>
                      ) : (
                        <><PenLine size={14} className="mr-1.5" /> Describe Pose</>
                      )}
                    </Button>
                  </>)}
                  <Button size="sm" variant="ghost" onClick={handleOpenWorldGallery}>
                    <Images size={14} className="mr-1.5" /> Choose from Gallery
                  </Button>
                  {portraits.length > 1 && (
                    <Button size="sm" variant="ghost" onClick={() => setShowGallery(true)}>
                      View all portraits ({portraits.length})
                    </Button>
                  )}
                  <p className="text-[11px] text-muted-foreground leading-relaxed max-w-[220px]">
                    Generate a portrait from their description, or create a variation of the current portrait in a different pose. Choose any image from this world's gallery.
                  </p>
                </div>
              </div>
            </FieldGroup>

            <FieldGroup label="Appearance">
              <Field
                label="Visual description"
                hint="How other characters and the narrator picture them. Describe honestly — observed, not interpreted. Auto-filled from the active portrait; yours to edit freely."
              >
                <Textarea
                  className="min-h-[140px] font-normal"
                  value={form.visual_description ?? ""}
                  onChange={(e) => update({ visual_description: e.target.value })}
                  placeholder="Late-30s build, broad shoulders narrowing to a wiry waist. Close-cropped dark hair going grey at the temples. Light brown eyes set wide. A small scar cutting the left eyebrow..."
                />
                <div className="flex items-center justify-between mt-2">
                  <p className="text-[11px] text-muted-foreground/70">
                    {activePortrait
                      ? "Regenerating from the active portrait will overwrite the text above."
                      : "No active portrait — generate one first to auto-describe."}
                  </p>
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={!activePortrait || !store.apiKey || regeneratingVisualDesc}
                    onClick={async () => {
                      if (!ch) return;
                      setRegeneratingVisualDesc(true);
                      try {
                        const updated = await api.generateCharacterVisualDescription(store.apiKey, ch.character_id, true);
                        update({ visual_description: updated.visual_description ?? "" });
                        setDirty(false);
                      } catch (e) {
                        store.setError?.(String(e));
                      } finally {
                        setRegeneratingVisualDesc(false);
                      }
                    }}
                  >
                    {regeneratingVisualDesc
                      ? <><Loader2 size={14} className="mr-1.5 animate-spin" /> Describing...</>
                      : <><Shuffle size={14} className="mr-1.5" /> Describe from portrait</>}
                  </Button>
                </div>
              </Field>
            </FieldGroup>

            <FieldGroup label="Basics">
              <div className="grid grid-cols-2 gap-4">
                <Field label="Name">
                  <Input value={form.display_name ?? ""} onChange={(e) => update({ display_name: e.target.value })} />
                </Field>
                <Field label="Avatar Color">
                  <div className="flex items-center gap-2">
                    <input
                      type="color"
                      value={form.avatar_color ?? "#c4a882"}
                      onChange={(e) => update({ avatar_color: e.target.value })}
                      className="w-9 h-9 rounded-lg border border-input cursor-pointer bg-transparent p-0.5"
                    />
                    <Input className="flex-1 font-mono text-xs" value={form.avatar_color ?? ""} onChange={(e) => update({ avatar_color: e.target.value })} />
                  </div>
                </Field>
                <Field label="Sex">
                  <div className="flex rounded-lg overflow-hidden border border-input">
                    <button
                      onClick={() => update({ sex: "male" })}
                      className={`flex-1 px-3 py-1.5 text-sm font-medium transition-colors cursor-pointer ${
                        (form.sex ?? "male") === "male" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"
                      }`}
                    >Male</button>
                    <button
                      onClick={() => update({ sex: "female" })}
                      className={`flex-1 px-3 py-1.5 text-sm font-medium transition-colors cursor-pointer ${
                        form.sex === "female" ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"
                      }`}
                    >Female</button>
                  </div>
                </Field>
                <Field label="Signature Emoji" hint="Used rarely, only on beats where they feel especially themselves">
                  <div className="relative flex items-center gap-2">
                    <div className="h-9 min-w-9 px-2 rounded-lg border border-input bg-background flex items-center justify-center text-lg">
                      {form.signature_emoji ? form.signature_emoji : <span className="text-xs text-muted-foreground italic">None</span>}
                    </div>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      onClick={() => setShowSignatureEmojiPicker((v) => !v)}
                    >
                      {form.signature_emoji ? "Change" : "Pick"}
                    </Button>
                    {form.signature_emoji && (
                      <Button
                        type="button"
                        size="sm"
                        variant="ghost"
                        onClick={() => update({ signature_emoji: "" })}
                      >
                        None
                      </Button>
                    )}
                    {showSignatureEmojiPicker && (
                      <div
                        ref={signatureEmojiPickerRef}
                        className="absolute z-50 top-full left-0 mt-2 shadow-xl shadow-black/30 rounded-lg overflow-hidden"
                      >
                        <EmojiPicker
                          onEmojiClick={(data) => {
                            update({ signature_emoji: data.emoji });
                            setShowSignatureEmojiPicker(false);
                          }}
                          emojiStyle={EmojiStyle.NATIVE}
                          theme={Theme.DARK}
                          height={380}
                          width={320}
                          lazyLoadEmojis
                          searchPlaceholder="Search emoji..."
                          previewConfig={{ showPreview: false }}
                        />
                      </div>
                    )}
                  </div>
                </Field>
                <Field label="Action-Beat Density" hint="How often they narrate their body (*leans back*, *looks out*). Low = quiet/measured; High = alert/in-motion.">
                  <div className="inline-flex rounded-lg border border-input overflow-hidden bg-background">
                    {(["low", "normal", "high"] as const).map((opt) => {
                      const current = (form.action_beat_density ?? "normal") as "low" | "normal" | "high";
                      return (
                        <button
                          key={opt}
                          type="button"
                          onClick={() => update({ action_beat_density: opt })}
                          className={`px-3 py-1.5 text-sm font-medium transition-colors cursor-pointer capitalize ${
                            current === opt ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"
                          }`}
                        >
                          {opt}
                        </button>
                      );
                    })}
                  </div>
                </Field>
                <Field
                  label="Has Read the Empiricon"
                  hint="When on, the full Empiricon document is included in this character's prompts (chat, dreams, narration, novel chapters, formula derivation, momentstamps)."
                >
                  <div className="flex items-center gap-3">
                    <Switch
                      checked={form.has_read_empiricon ?? false}
                      onCheckedChange={(v) => update({ has_read_empiricon: v })}
                    />
                    <span className="text-sm text-muted-foreground">
                      {form.has_read_empiricon ? "On" : "Off"}
                    </span>
                  </div>
                </Field>
              </div>
            </FieldGroup>

            <FieldGroup label="Identity & Voice">
              {ch && (
                <DerivationCard
                  label="Their derivation in 𝓕"
                  load={() => api.getCharacterDerivation(ch.character_id)}
                  refetchKey={`${ch.character_id}:${derivationVersion}`}
                  onRegenerate={async () => {
                    if (!store.apiKey) throw new Error("Need an OpenAI API key in Settings to regenerate.");
                    await api.regenerateCharacterDerivation(store.apiKey, ch.character_id);
                    setDerivationVersion((v) => v + 1);
                  }}
                />
              )}
              <Field label="Identity" hint="Core personality, demeanor, how they see the world">
                <Textarea
                  className="min-h-[120px]"
                  value={form.identity ?? ""}
                  onChange={(e) => update({ identity: e.target.value })}
                  placeholder="Quiet, observant, with a dark sense of humor. Speaks in half-truths..."
                />
              </Field>

              <Field label="TTS Model" hint="Model used for text-to-speech generation">
                <select
                  className="w-full h-9 rounded-lg border border-border bg-background px-3 text-sm"
                  value={ttsModel}
                  onChange={(e) => {
                    setTtsModel(e.target.value);
                    if (ch) api.setSetting(`tts_model.${ch.character_id}`, e.target.value);
                  }}
                >
                  <option value="gpt-4o-mini-tts">gpt-4o-mini-tts (fast, expressive)</option>
                  <option value="gpt-audio-1.5">gpt-audio-1.5 (high quality)</option>
                </select>
              </Field>

              <Field label="TTS Voice" hint="Voice used for text-to-speech playback">
                <div className="flex gap-2">
                  <select
                    className="flex-1 h-9 rounded-lg border border-border bg-background px-3 text-sm"
                    value={ttsVoice}
                    onChange={(e) => {
                      setTtsVoice(e.target.value);
                      if (ch) api.setSetting(`voice.${ch.character_id}`, e.target.value);
                    }}
                  >
                    <optgroup label="Feminine">
                      <option value="alloy">Alloy</option>
                      <option value="coral">Coral</option>
                      <option value="nova">Nova</option>
                      <option value="sage">Sage</option>
                      <option value="shimmer">Shimmer</option>
                    </optgroup>
                    <optgroup label="Masculine">
                      <option value="ash">Ash</option>
                      <option value="echo">Echo</option>
                      <option value="fable">Fable</option>
                      <option value="onyx">Onyx</option>
                      <option value="verse">Verse</option>
                    </optgroup>
                    <optgroup label="Neutral">
                      <option value="ballad">Ballad</option>
                    </optgroup>
                  </select>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-9 px-3 flex-shrink-0"
                    onClick={() => setShowVoiceExplorer(true)}
                  >
                    <Volume2 size={14} className="mr-1.5" />
                    Explore
                  </Button>
                </div>
              </Field>

              <ArrayField
                label="Voice Rules"
                hint="How this character talks — style, cadence, quirks"
                items={(form.voice_rules ?? []) as string[]}
                onChange={(items) => update({ voice_rules: items })}
                placeholder="e.g. Uses short sentences. Avoids exclamation marks."
              />

              <ArrayField
                label="Boundaries"
                hint="Lines this character never crosses"
                items={(form.boundaries ?? []) as string[]}
                onChange={(items) => update({ boundaries: items })}
                placeholder="e.g. Never reveals their real name."
              />
            </FieldGroup>

            <FieldGroup label="Backstory">
              <ArrayField
                label="Facts"
                hint="Established truths about this character"
                items={(form.backstory_facts ?? []) as string[]}
                onChange={(items) => update({ backstory_facts: items })}
                placeholder="e.g. Grew up near the lighthouse."
              />
            </FieldGroup>

            <FieldGroup label="Current State">
              <div className="grid grid-cols-2 gap-4">
                <Field label="Mood" hint="-1 (dark) to 1 (bright)">
                  <div className="flex items-center gap-3">
                    <input
                      type="range"
                      min="-1"
                      max="1"
                      step="0.1"
                      value={form.state?.mood ?? 0}
                      onChange={(e) => update({ state: { ...form.state as Character["state"], mood: Number(e.target.value) } })}
                      className="flex-1 accent-primary h-1.5 cursor-pointer"
                    />
                    <span className="text-xs font-mono text-muted-foreground w-8 text-right">
                      {(form.state?.mood ?? 0).toFixed(1)}
                    </span>
                  </div>
                </Field>
                <Field label="Trust" hint="0 (none) to 1 (full)">
                  <div className="flex items-center gap-3">
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.1"
                      value={form.state?.trust_user ?? 0.5}
                      onChange={(e) => update({ state: { ...form.state as Character["state"], trust_user: Number(e.target.value) } })}
                      className="flex-1 accent-primary h-1.5 cursor-pointer"
                    />
                    <span className="text-xs font-mono text-muted-foreground w-8 text-right">
                      {(form.state?.trust_user ?? 0.5).toFixed(1)}
                    </span>
                  </div>
                </Field>
              </div>
              <ArrayField
                label="Goals"
                hint="What this character is currently trying to do"
                items={Array.isArray(form.state?.goals) ? form.state.goals : []}
                onChange={(goals) => update({ state: { ...form.state as Character["state"], goals } })}
                placeholder="e.g. Find the missing map piece"
              />
              <ArrayField
                label="Open Loops"
                hint="Unresolved threads that may surface in conversation"
                items={Array.isArray(form.state?.open_loops) ? form.state.open_loops : []}
                onChange={(open_loops) => update({ state: { ...form.state as Character["state"], open_loops } })}
                placeholder="e.g. Ask user about the locked door"
              />
              {(() => {
                const knownKeys = new Set(["mood", "trust_user", "goals", "open_loops", "last_seen"]);
                const stateObj = (form.state ?? {}) as Record<string, unknown>;
                const extraEntries = Object.entries(stateObj).filter(([k]) => !knownKeys.has(k));
                if (extraEntries.length === 0) return null;
                return (
                  <Field label="Additional State" hint="Extra fields added by world simulation">
                    <div className="space-y-2">
                      {extraEntries.map(([key, value]) => (
                        <div key={key} className="flex items-start gap-2 group">
                          <div className="flex-1 flex items-start gap-2 px-3 py-2 rounded-lg border border-border bg-card/50 font-mono text-xs">
                            <span className="text-primary/70 flex-shrink-0">{key}:</span>
                            <span className="text-foreground/80 break-all">
                              {typeof value === "object" ? JSON.stringify(value) : String(value)}
                            </span>
                          </div>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-9 w-9 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive flex-shrink-0"
                            onClick={() => {
                              const newState = { ...((form.state ?? {}) as unknown as Record<string, unknown>) };
                              delete newState[key];
                              update({ state: newState as unknown as CharacterState });
                            }}
                          >
                            <X size={14} />
                          </Button>
                        </div>
                      ))}
                    </div>
                  </Field>
                );
              })()}
              <div className="pt-2">
                <Button
                  size="sm"
                  variant="outline"
                  className="border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
                  onClick={() => {
                    update({
                      state: {
                        mood: 0,
                        trust_user: 0.5,
                        goals: [],
                        open_loops: [],
                        last_seen: (form.state as Character["state"])?.last_seen ?? { day_index: 1, time_of_day: "MORNING" },
                      },
                    });
                  }}
                >
                  <RotateCcw size={14} className="mr-1.5" /> Reset State to Defaults
                </Button>
              </div>
            </FieldGroup>

            <FieldGroup label="Inventory">
              <p className="text-xs text-muted-foreground mb-2">Up to 3 small things this character currently has in their keeping. Refreshed by the LLM on world-day rollover; fully editable here.</p>
              <InventoryEditor
                characterId={ch?.character_id}
                initial={(ch?.inventory ?? []) as InventoryItem[]}
                onSaved={(next) => {
                  // Reflect the edit into the active character + the
                  // characters list so popovers / cards / in-prompt
                  // rendering update without a round-trip.
                  if (ch) store.applyCharacterInventoryEdit(ch.character_id, next);
                }}
              />
            </FieldGroup>

            <FieldGroup label="Journal">
              <div className="flex items-start justify-between gap-4 mb-3">
                <p className="text-xs text-muted-foreground flex-1">
                  A first-person reflective entry per world-day, written in {ch?.display_name ?? "this character"}'s own voice. Fed back into dialogue prompts as "who you've been lately" so ongoing interior threads carry across days. Regenerate today's entry any time.
                </p>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleGenerateJournal}
                  disabled={journalGenerating || !store.apiKey}
                >
                  {journalGenerating ? <Loader2 size={12} className="animate-spin mr-1.5" /> : <PenLine size={12} className="mr-1.5" />}
                  {journalGenerating ? "Writing..." : "Write yesterday's entry"}
                </Button>
              </div>
              {journalLoading ? (
                <div className="text-xs text-muted-foreground italic">Loading entries…</div>
              ) : journalEntries.length === 0 ? (
                <div className="text-xs text-muted-foreground italic py-4 text-center">
                  No entries yet. Click "Write yesterday's entry" to generate the first one (requires world-day ≥ 1).
                </div>
              ) : (
                <div className="space-y-3 max-h-[480px] overflow-y-auto pr-1">
                  {journalEntries.map((e) => (
                    <div key={e.journal_id} className="rounded-lg border border-border/60 bg-secondary/20 p-3">
                      <div className="flex items-center justify-between mb-1.5">
                        <span className="text-[11px] uppercase tracking-wider text-muted-foreground font-semibold">Day {e.world_day}</span>
                        <span className="text-[10px] text-muted-foreground/60">{new Date(e.created_at).toLocaleString([], { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}</span>
                      </div>
                      <p className="text-sm leading-relaxed whitespace-pre-wrap italic text-foreground/85">{e.content}</p>
                    </div>
                  ))}
                </div>
              )}
            </FieldGroup>

            <FieldGroup label="Danger Zone">
              <div className="space-y-3">
                <div className="flex items-center justify-between py-2.5 px-4 rounded-lg border border-border bg-card/50">
                  <div>
                    <p className="text-sm font-medium">Clear Chat History</p>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      Delete all messages, memories, and embeddings. The character and their details are preserved.
                    </p>
                  </div>
                  <Button size="sm" variant="outline" className="border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive flex-shrink-0" onClick={() => setShowClearChat(true)}>
                    <MessageSquareX size={14} className="mr-1.5" /> Clear
                  </Button>
                </div>
                <div className="flex items-center justify-between py-2.5 px-4 rounded-lg border border-destructive/30 bg-destructive/5">
                  <div>
                    <p className="text-sm font-medium text-destructive">Delete Character</p>
                    <p className="text-xs text-muted-foreground mt-0.5">
                      Permanently delete this character and all their data. This cannot be undone.
                    </p>
                  </div>
                  <Button size="sm" variant="outline" className="border-destructive/40 text-destructive hover:bg-destructive hover:text-destructive-foreground flex-shrink-0" onClick={() => setShowDeleteChar(true)}>
                    <Trash2 size={14} className="mr-1.5" /> Delete
                  </Button>
                </div>
              </div>
            </FieldGroup>
          </div>
        </ScrollArea>
      </div>

      {/* Clear Chat Confirmation */}
      <Dialog open={showClearChat} onClose={() => setShowClearChat(false)}>
        <DialogContent>
          <DialogHeader onClose={() => setShowClearChat(false)}>
            <DialogTitle>
              <MessageSquareX size={16} className="inline mr-2 text-destructive" />Clear Chat History
            </DialogTitle>
          </DialogHeader>
          <DialogBody>
            <p className="text-sm text-muted-foreground">
              This will permanently delete all messages, memories, and embeddings for <strong>{ch.display_name}</strong>. The character and their details will be preserved.
            </p>
            <div className="flex items-center gap-3 mt-4">
              <Button variant="outline" className="flex-1" onClick={() => setShowClearChat(false)}>Cancel</Button>
              <Button variant="destructive" className="flex-1" onClick={async () => {
                setShowClearChat(false);
                await store.clearChatHistory(ch.character_id, false);
              }}>
                Clear All Messages
              </Button>
            </div>
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* Delete Character Confirmation */}
      <Dialog open={showDeleteChar} onClose={() => setShowDeleteChar(false)}>
        <DialogContent>
          <DialogHeader onClose={() => setShowDeleteChar(false)}>
            <DialogTitle>
              <AlertTriangle size={16} className="inline mr-2 text-destructive" />Delete Character
            </DialogTitle>
          </DialogHeader>
          <DialogBody>
            <p className="text-sm text-muted-foreground">
              This will permanently delete <strong>{ch.display_name}</strong> and all their messages, portraits, memories, and embeddings. This cannot be undone.
            </p>
            <div className="flex items-center gap-3 mt-4">
              <Button variant="outline" className="flex-1" onClick={() => setShowDeleteChar(false)}>Cancel</Button>
              <Button variant="destructive" className="flex-1" onClick={async () => {
                setShowDeleteChar(false);
                await store.deleteCharacter(ch.character_id);
              }}>
                Delete Forever
              </Button>
            </div>
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* Portrait Gallery Modal */}
      <Dialog open={showGallery} onClose={() => setShowGallery(false)} className="max-w-2xl">
        <DialogContent>
          <DialogHeader onClose={() => setShowGallery(false)}>
            <DialogTitle>Portrait Gallery</DialogTitle>
            <DialogDescription>
              {portraits.length} portrait{portraits.length !== 1 ? "s" : ""} generated. Click to set as active.
            </DialogDescription>
          </DialogHeader>
          <DialogBody className="p-0">
            <ScrollArea className="max-h-[500px]">
              <div className="grid grid-cols-3 gap-3 p-4">
                {portraits.map((p) => (
                  <div key={p.portrait_id} className="relative group">
                    <button
                      onClick={() => handleSelectPortrait(p)}
                      className={`relative rounded-xl overflow-hidden border-2 transition-all cursor-pointer aspect-square w-full ${
                        p.is_active ? "border-primary ring-2 ring-primary/30" : "border-border hover:border-primary/40"
                      }`}
                    >
                      {p.data_url ? (
                        <img src={p.data_url} alt="" className="w-full h-full object-cover" />
                      ) : (
                        <div className="w-full h-full bg-muted flex items-center justify-center text-muted-foreground text-xs">Missing</div>
                      )}
                      {p.is_active && (
                        <div className="absolute top-2 right-2 w-6 h-6 rounded-full bg-primary flex items-center justify-center">
                          <Check size={14} className="text-primary-foreground" />
                        </div>
                      )}
                      <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/70 to-transparent p-2 pt-6 opacity-0 group-hover:opacity-100 transition-opacity">
                        <p className="text-[10px] text-white/80">
                          {new Date(p.created_at).toLocaleDateString()}
                        </p>
                      </div>
                    </button>
                    {!p.is_active && (
                      <button
                        onClick={async (e) => {
                          e.stopPropagation();
                          try {
                            await api.deletePortrait(p.portrait_id);
                            setPortraits((prev) => prev.filter((pp) => pp.portrait_id !== p.portrait_id));
                          } catch (err) {
                            store.setError?.(String(err));
                          }
                        }}
                        className="absolute top-2 left-2 w-7 h-7 rounded-full bg-black/60 text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer hover:bg-destructive"
                        title="Delete portrait"
                      >
                        <Trash2 size={14} />
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </ScrollArea>
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* Variation Preview Modal */}
      <Dialog open={!!variationPreview} onClose={handleDiscardVariation} className="max-w-md">
        <DialogContent>
          <DialogHeader onClose={handleDiscardVariation}>
            <DialogTitle>New Pose</DialogTitle>
            <DialogDescription>
              Here's a variation of your character. Keep it or discard it.
            </DialogDescription>
          </DialogHeader>
          <DialogBody>
            {variationPreview?.data_url && (
              <img
                src={variationPreview.data_url}
                alt="Portrait variation"
                className="w-full rounded-xl border border-border"
              />
            )}
            <div className="flex items-center gap-3 mt-4">
              <Button className="flex-1" onClick={handleKeepVariation}>
                <Check size={14} className="mr-1.5" /> Keep
              </Button>
              <Button variant="outline" className="flex-1" onClick={handleDiscardVariation}>
                <Trash2 size={14} className="mr-1.5" /> Discard
              </Button>
            </div>
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* World Gallery Picker Modal */}
      <Dialog open={showWorldGallery} onClose={() => setShowWorldGallery(false)} className="max-w-3xl">
        <DialogContent>
          <DialogHeader onClose={() => setShowWorldGallery(false)}>
            <DialogTitle>Choose from World Gallery</DialogTitle>
            <DialogDescription>
              Select any image from this world to use as this character's portrait.
            </DialogDescription>
          </DialogHeader>
          <DialogBody className="p-0">
            <ScrollArea className="max-h-[500px]">
              {loadingWorldGallery ? (
                <div className="flex items-center justify-center py-12">
                  <Loader2 size={24} className="animate-spin text-muted-foreground" />
                </div>
              ) : worldGalleryItems.length === 0 ? (
                <div className="text-center py-12 text-muted-foreground text-sm">
                  No images in this world yet.
                </div>
              ) : (
                <div className="grid grid-cols-3 gap-3 p-4">
                  {worldGalleryItems.filter(i => i.data_url).map((item) => (
                    <button
                      key={item.id}
                      onClick={() => handleSelectFromWorldGallery(item)}
                      className="relative rounded-xl overflow-hidden border-2 border-border hover:border-primary/50 transition-all cursor-pointer group"
                    >
                      <img
                        src={item.data_url}
                        alt=""
                        className={`w-full object-cover ${item.category === "world" ? "aspect-video" : "aspect-square"}`}
                      />
                      <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/70 to-transparent p-2 pt-6 opacity-0 group-hover:opacity-100 transition-opacity">
                        <p className="text-[11px] text-white font-medium truncate">{item.label}</p>
                        <p className="text-[10px] text-white/60 capitalize">{item.category}</p>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </ScrollArea>
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* Template Picker Modal */}
      <Dialog open={showTemplates} onClose={() => setShowTemplates(false)} className="max-w-2xl">
        <DialogContent>
          <DialogHeader onClose={() => setShowTemplates(false)}>
            <DialogTitle>Choose a Template</DialogTitle>
            <DialogDescription>Pick an archetype to pre-fill all fields. You can customize everything after.</DialogDescription>
          </DialogHeader>
          <DialogBody className="p-0">
            <div className="px-6 py-3 border-b border-border">
              <Input
                autoFocus
                placeholder="Search templates..."
                value={templateSearch}
                onChange={(e) => setTemplateSearch(e.target.value)}
              />
            </div>
            <ScrollArea className="max-h-[420px]">
              <div className="grid grid-cols-2 gap-2 p-4">
                {filteredTemplates.map((template) => (
                  <button
                    key={template.name}
                    onClick={() => applyTemplate(template)}
                    className="text-left p-3.5 rounded-xl border border-border bg-card/50 hover:bg-accent/50 hover:border-primary/30 transition-all cursor-pointer group"
                  >
                    <div className="flex items-center gap-2.5 mb-1.5">
                      <span className="text-lg">{template.emoji}</span>
                      <span className="font-medium text-sm group-hover:text-primary transition-colors">{template.name}</span>
                      <span className="ml-auto w-2.5 h-2.5 rounded-full flex-shrink-0 ring-1 ring-white/10" style={{ backgroundColor: template.avatar_color }} />
                    </div>
                    <p className="text-xs text-muted-foreground leading-relaxed">{template.tagline}</p>
                  </button>
                ))}
                {filteredTemplates.length === 0 && (
                  <div className="col-span-2 py-8 text-center text-muted-foreground text-sm">
                    No templates match your search
                  </div>
                )}
              </div>
            </ScrollArea>
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* Describe Pose Modal */}
      <Dialog open={showPoseModal} onClose={() => setShowPoseModal(false)} className="max-w-md">
        <DialogContent>
          <DialogHeader onClose={() => setShowPoseModal(false)}>
            <DialogTitle>Describe Pose</DialogTitle>
            <DialogDescription>
              Describe the pose, angle, expression, or situation you'd like for this character. The existing portraits will be used as reference.
            </DialogDescription>
          </DialogHeader>
          <DialogBody>
            <textarea
              value={poseDescription}
              onChange={(e) => setPoseDescription(e.target.value)}
              placeholder="e.g. Looking over their shoulder with a slight grin, arms crossed, standing in front of a window..."
              className="w-full min-h-[100px] max-h-[200px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
              rows={4}
              autoFocus
            />
          </DialogBody>
          <div className="flex justify-end gap-2 px-5 pb-5">
            <Button variant="ghost" size="sm" onClick={() => setShowPoseModal(false)}>
              Cancel
            </Button>
            <Button
              size="sm"
              disabled={!poseDescription.trim()}
              onClick={() => handleGenerateWithPose(poseDescription.trim())}
            >
              Generate
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={showVoiceExplorer} onClose={() => { setShowVoiceExplorer(false); sampleAudioRef.current?.pause(); setSamplePlaying(null); }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Voice Preview</DialogTitle>
            <DialogDescription>Listen to each voice to find the right fit for {form.display_name || "this character"}.</DialogDescription>
          </DialogHeader>
          <DialogBody>
            <div className="flex gap-2 mb-3">
              <div className="flex-1">
                <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">Model</label>
                <select
                  className="w-full h-8 mt-1 rounded-lg border border-border bg-background px-2.5 text-xs"
                  value={ttsModel}
                  onChange={(e) => {
                    setTtsModel(e.target.value);
                    if (ch) api.setSetting(`tts_model.${ch.character_id}`, e.target.value);
                  }}
                >
                  <option value="gpt-4o-mini-tts">gpt-4o-mini-tts</option>
                  <option value="gpt-audio-1.5">gpt-audio-1.5</option>
                </select>
              </div>
              <div className="flex-1">
                <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">Tone</label>
                <select
                  className="w-full h-8 mt-1 rounded-lg border border-border bg-background px-2.5 text-xs"
                  value={sampleTone}
                  onChange={(e) => setSampleTone(e.target.value)}
                >
                  {["Auto", "Playful", "Happy", "Excited", "Reverent", "Serene", "Intimate", "Tender", "Sad", "Melancholy", "Angry", "Anxious"].map((t) => (
                    <option key={t} value={t}>{t}</option>
                  ))}
                </select>
              </div>
            </div>
            <div className="space-y-3">
              {[
                { label: "Feminine", voices: ["alloy", "coral", "nova", "sage", "shimmer"] },
                { label: "Masculine", voices: ["ash", "echo", "fable", "onyx", "verse"] },
                { label: "Neutral", voices: ["ballad"] },
              ].map((group) => (
                <div key={group.label}>
                  <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider mb-1.5">{group.label}</p>
                  <div className="flex flex-wrap gap-1.5">
                    {group.voices.map((voice) => {
                      const sampleKey = `${ttsModel}:${voice}:${sampleTone}`;
                      const isPlaying = samplePlaying === sampleKey;
                      const isLoading = sampleLoading === sampleKey;
                      return (
                        <button
                          key={voice}
                          onClick={async () => {
                            if (isPlaying) {
                              sampleAudioRef.current?.pause();
                              setSamplePlaying(null);
                              return;
                            }
                            sampleAudioRef.current?.pause();
                            setSamplePlaying(null);
                            setSampleLoading(sampleKey);
                            try {
                              const bytes = await api.generateVoiceSample(store.apiKey, voice, sampleTone === "Auto" ? undefined : sampleTone, ttsModel);
                              const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
                              const url = URL.createObjectURL(blob);
                              const audio = new Audio(url);
                              sampleAudioRef.current = audio;
                              audio.onended = () => setSamplePlaying(null);
                              audio.play();
                              setSamplePlaying(sampleKey);
                            } catch (e) {
                              store.setError?.(String(e));
                            } finally {
                              setSampleLoading(null);
                            }
                          }}
                          className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-all cursor-pointer ${
                            isPlaying
                              ? "bg-primary text-primary-foreground"
                              : voice === ttsVoice
                                ? "bg-primary/15 text-primary border border-primary/30"
                                : "bg-secondary text-secondary-foreground hover:bg-accent"
                          }`}
                        >
                          {isLoading ? (
                            <Loader2 size={12} className="animate-spin" />
                          ) : isPlaying ? (
                            <Square size={10} fill="currentColor" />
                          ) : (
                            <Volume2 size={12} />
                          )}
                          {voice.charAt(0).toUpperCase() + voice.slice(1)}
                        </button>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>
            <div className="mt-4 pt-3 border-t border-border flex justify-between items-center">
              <div className="flex items-center gap-3">
                <p className="text-xs text-muted-foreground">
                  Current: <span className="font-medium text-foreground">{ttsVoice.charAt(0).toUpperCase() + ttsVoice.slice(1)}</span>
                </p>
                <button
                  onClick={async () => {
                    sampleAudioRef.current?.pause();
                    setSamplePlaying(null);
                    await api.clearVoiceSamples();
                  }}
                  className="text-[10px] text-muted-foreground/60 hover:text-red-400 transition-colors cursor-pointer flex items-center gap-1"
                >
                  <Trash2 size={10} />
                  Clear cached previews
                </button>
              </div>
              <Button variant="ghost" size="sm" onClick={() => { setShowVoiceExplorer(false); sampleAudioRef.current?.pause(); setSamplePlaying(null); }}>
                Done
              </Button>
            </div>
          </DialogBody>
        </DialogContent>
      </Dialog>
    </>
  );
}

function ArrayField({ label, hint, items, onChange, placeholder }: {
  label: string;
  hint?: string;
  items: string[];
  onChange: (items: string[]) => void;
  placeholder: string;
}) {
  const [newItem, setNewItem] = useState("");

  return (
    <Field label={label} hint={hint}>
      <div className="space-y-2">
        {items.map((item, i) => (
          <div key={i} className="flex items-start gap-2 group">
            <div className="mt-2.5 w-1.5 h-1.5 rounded-full bg-primary/40 flex-shrink-0" />
            <Input className="flex-1" value={item} onChange={(e) => {
              const updated = [...items];
              updated[i] = e.target.value;
              onChange(updated);
            }} />
            <Button variant="ghost" size="icon" className="h-9 w-9 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive flex-shrink-0" onClick={() => {
              onChange(items.filter((_, j) => j !== i));
            }}>
              <X size={14} />
            </Button>
          </div>
        ))}
        <div className="flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-border flex-shrink-0" />
          <Input
            className="flex-1"
            value={newItem}
            onChange={(e) => setNewItem(e.target.value)}
            placeholder={placeholder}
            onKeyDown={(e) => {
              if (e.key === "Enter" && newItem.trim()) {
                onChange([...items, newItem.trim()]);
                setNewItem("");
              }
            }}
          />
          <Button variant="outline" size="sm" className="h-9 flex-shrink-0" onClick={() => {
            if (newItem.trim()) {
              onChange([...items, newItem.trim()]);
              setNewItem("");
            }
          }}>
            <Plus size={14} className="mr-1" /> Add
          </Button>
        </div>
      </div>
    </Field>
  );
}

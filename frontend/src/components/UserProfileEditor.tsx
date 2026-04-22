import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Field, FieldGroup } from "@/components/ui/field";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from "@/components/ui/dialog";
import { Save, Plus, X, User, ImagePlus, Upload, Loader2, Images, Copy, PenLine } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type UserProfile, type GalleryItem, type UserJournalEntry } from "@/lib/tauri";

interface Props {
  store: ReturnType<typeof useAppStore>;
}

export function UserProfileEditor({ store }: Props) {
  const worldId = store.activeWorld?.world_id;
  const existing = store.userProfile;

  const [form, setForm] = useState({
    display_name: "",
    description: "",
    facts: [] as string[],
  });
  const [dirty, setDirty] = useState(false);
  const [avatarUrl, setAvatarUrl] = useState("");
  const [generating, setGenerating] = useState(false);
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [showWorldGallery, setShowWorldGallery] = useState(false);
  const [worldGalleryItems, setWorldGalleryItems] = useState<GalleryItem[]>([]);
  const [loadingWorldGallery, setLoadingWorldGallery] = useState(false);
  const [showCrossWorldPicker, setShowCrossWorldPicker] = useState(false);
  const [crossWorldAvatars, setCrossWorldAvatars] = useState<Array<{ world_id: string; world_name: string; avatar_file: string; data_url: string }>>([]);
  const [loadingCrossWorld, setLoadingCrossWorld] = useState(false);

  const [journalEntries, setJournalEntries] = useState<UserJournalEntry[]>([]);
  const [journalLoading, setJournalLoading] = useState(false);
  const [journalGenerating, setJournalGenerating] = useState(false);

  useEffect(() => {
    if (!worldId) { setJournalEntries([]); return; }
    setJournalLoading(true);
    api.listUserJournals(worldId, 30)
      .then((entries) => setJournalEntries(entries))
      .catch(() => setJournalEntries([]))
      .finally(() => setJournalLoading(false));
  }, [worldId]);

  const handleGenerateJournal = async () => {
    if (!worldId || !store.apiKey) return;
    setJournalGenerating(true);
    try {
      const entry = await api.generateUserJournal(store.apiKey, worldId);
      setJournalEntries((prev) => {
        const filtered = prev.filter((e) => e.world_day !== entry.world_day);
        return [entry, ...filtered].sort((a, b) => b.world_day - a.world_day);
      });
    } catch (e: any) {
      console.error("Failed to generate user journal:", e);
    } finally {
      setJournalGenerating(false);
    }
  };

  useEffect(() => {
    if (existing) {
      setForm({
        display_name: existing.display_name,
        description: existing.description,
        facts: [...(existing.facts ?? [])],
      });
    } else {
      setForm({ display_name: "Me", description: "", facts: [] });
    }
    setDirty(false);
  }, [existing, worldId]);

  useEffect(() => {
    if (!worldId) return;
    api.getUserAvatar(worldId).then((url) => setAvatarUrl(url || ""));
  }, [worldId, existing?.avatar_file]);

  const handleGenerate = async () => {
    if (!worldId) return;
    setGenerating(true);
    try {
      const key = store.apiKey;
      const dataUrl = await api.generateUserAvatar(key, worldId, {
        display_name: form.display_name,
        description: form.description,
      });
      setAvatarUrl(dataUrl);
      await store.loadUserProfile(worldId);
    } catch (e: any) {
      console.error("Failed to generate avatar:", e);
    } finally {
      setGenerating(false);
    }
  };

  const handleUpload = () => {
    fileInputRef.current?.click();
  };

  const handleFileSelected = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !worldId) return;
    setUploading(true);
    try {
      const reader = new FileReader();
      const dataUrl: string = await new Promise((resolve, reject) => {
        reader.onload = () => resolve(reader.result as string);
        reader.onerror = reject;
        reader.readAsDataURL(file);
      });
      const result = await api.uploadUserAvatar(worldId, dataUrl);
      setAvatarUrl(result);
      await store.loadUserProfile(worldId);
    } catch (err: any) {
      console.error("Failed to upload avatar:", err);
    } finally {
      setUploading(false);
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  };

  const handleOpenWorldGallery = async () => {
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
    if (!worldId || !item.file_name) return;
    try {
      const dataUrl = await api.setUserAvatarFromGallery(worldId, item.file_name);
      setAvatarUrl(dataUrl);
      setShowWorldGallery(false);
      await store.loadUserProfile(worldId);
    } catch (e: any) {
      console.error("Failed to set avatar from gallery:", e);
    }
  };

  const handleOpenCrossWorld = async () => {
    setLoadingCrossWorld(true);
    setShowCrossWorldPicker(true);
    try {
      const avatars = await api.listAllUserAvatars();
      // Filter out the current world's avatar
      setCrossWorldAvatars(avatars.filter((a) => a.world_id !== worldId));
    } catch {
    } finally {
      setLoadingCrossWorld(false);
    }
  };

  const handleSelectCrossWorld = async (avatarFile: string) => {
    if (!worldId) return;
    try {
      const dataUrl = await api.setUserAvatarFromGallery(worldId, avatarFile);
      setAvatarUrl(dataUrl);
      setShowCrossWorldPicker(false);
      await store.loadUserProfile(worldId);
    } catch (e: any) {
      console.error("Failed to copy avatar:", e);
    }
  };

  if (!worldId) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <p>Select a world first</p>
      </div>
    );
  }

  const update = (patch: Partial<typeof form>) => {
    setForm((f) => ({ ...f, ...patch }));
    setDirty(true);
  };

  const handleSave = async () => {
    const profile: UserProfile = {
      world_id: worldId,
      display_name: form.display_name || "Me",
      description: form.description,
      facts: form.facts,
      avatar_file: existing?.avatar_file ?? "",
      updated_at: new Date().toISOString(),
    };
    await store.updateUserProfile(profile);
    setDirty(false);
  };

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div className="px-6 py-3 border-b border-border flex items-center justify-between">
        <div className="flex items-center gap-3">
          {avatarUrl ? (
            <img src={avatarUrl} alt="" className="w-9 h-9 rounded-full object-cover" />
          ) : (
            <div className="w-9 h-9 rounded-full bg-primary/20 flex items-center justify-center">
              <User size={18} className="text-primary" />
            </div>
          )}
          <div>
            <h1 className="font-semibold">{form.display_name || "Me"}</h1>
            <span className="text-xs text-muted-foreground/50">Your Profile</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {dirty && (
            <span className="text-xs text-primary bg-primary/10 px-2 py-0.5 rounded-full">
              Unsaved changes
            </span>
          )}
          <Button size="sm" onClick={handleSave} disabled={!dirty}>
            <Save size={14} className="mr-1.5" /> Save
          </Button>
        </div>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept="image/png,image/jpeg,image/webp"
        className="hidden"
        onChange={handleFileSelected}
      />

      <ScrollArea className="flex-1 px-6 py-6">
        <div className="max-w-2xl space-y-8">
          <p className="text-sm text-muted-foreground leading-relaxed">
            Tell your characters about yourself. This context is included in every conversation so they know who they're talking to.
          </p>

          <FieldGroup label="Avatar">
            <div className="flex items-start gap-6">
              <div className="flex-shrink-0">
                {avatarUrl ? (
                  <img src={avatarUrl} alt="" className="w-28 h-28 rounded-xl object-cover border border-border" />
                ) : (
                  <div className="w-28 h-28 rounded-xl bg-muted flex items-center justify-center border border-border">
                    <User size={36} className="text-muted-foreground/40" />
                  </div>
                )}
              </div>
              <div className="flex flex-col gap-2 pt-1">
                <p className="text-xs text-muted-foreground leading-relaxed">
                  Generate a watercolor portrait, upload your own image, or choose from this world's gallery.
                </p>
                <div className="flex items-center gap-2 flex-wrap">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleGenerate}
                    disabled={generating || uploading || !store.apiKey}
                  >
                    {generating ? <Loader2 size={14} className="mr-1.5 animate-spin" /> : <ImagePlus size={14} className="mr-1.5" />}
                    {generating ? "Generating..." : "Generate"}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleUpload}
                    disabled={generating || uploading}
                  >
                    {uploading ? <Loader2 size={14} className="mr-1.5 animate-spin" /> : <Upload size={14} className="mr-1.5" />}
                    {uploading ? "Uploading..." : "Upload"}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleOpenWorldGallery}
                  >
                    <Images size={14} className="mr-1.5" /> World Gallery
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleOpenCrossWorld}
                  >
                    <Copy size={14} className="mr-1.5" /> From Other World
                  </Button>
                </div>
              </div>
            </div>
          </FieldGroup>

          <FieldGroup label="Basics">
            <Field label="Your Name" hint="What characters will call you">
              <Input
                value={form.display_name}
                onChange={(e) => update({ display_name: e.target.value })}
                placeholder="Your name or alias"
              />
            </Field>

            <Field label="About You" hint="A short description — personality, vibe, what matters to you">
              <Textarea
                className="min-h-[100px]"
                value={form.description}
                onChange={(e) => update({ description: e.target.value })}
                placeholder="e.g. Curious, a bit sarcastic, loves old books and rainy days..."
              />
            </Field>
          </FieldGroup>

          <FieldGroup label="Facts">
            <Field label="Things Characters Should Know" hint="Details that might come up in conversation">
              <FactsList
                items={form.facts}
                onChange={(facts) => update({ facts })}
                placeholder="e.g. Lives near the coast. Has a cat named Pepper."
              />
            </Field>
          </FieldGroup>

          <FieldGroup label="Journal">
            <div className="flex items-start justify-between gap-4 mb-3">
              <p className="text-xs text-muted-foreground flex-1">
                A private reflection per world-day, written as you across every chat you were in that day. Auto-generates when the world clock crosses into a new day. Scoped to this world only.
              </p>
              <Button
                size="sm"
                variant="outline"
                onClick={handleGenerateJournal}
                disabled={journalGenerating || !store.apiKey || !worldId}
              >
                {journalGenerating ? <Loader2 size={12} className="animate-spin mr-1.5" /> : <PenLine size={12} className="mr-1.5" />}
                {journalGenerating ? "Writing..." : "Write yesterday's entry"}
              </Button>
            </div>
            {journalLoading ? (
              <div className="text-xs text-muted-foreground italic">Loading entries…</div>
            ) : journalEntries.length === 0 ? (
              <div className="text-xs text-muted-foreground italic py-4 text-center">
                No entries yet. Click "Write yesterday's entry" once the world has crossed into Day 1.
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
        </div>
      </ScrollArea>

      {/* Cross-World Avatar Picker */}
      <Dialog open={showCrossWorldPicker} onClose={() => setShowCrossWorldPicker(false)} className="max-w-md">
        <DialogContent>
          <DialogHeader onClose={() => setShowCrossWorldPicker(false)}>
            <DialogTitle>Copy Avatar from Another World</DialogTitle>
            <DialogDescription>
              Choose one of your portraits from other worlds to use here.
            </DialogDescription>
          </DialogHeader>
          <DialogBody className="p-0">
            <ScrollArea className="max-h-[400px]">
              {loadingCrossWorld ? (
                <div className="flex items-center justify-center py-12">
                  <Loader2 size={24} className="animate-spin text-muted-foreground" />
                </div>
              ) : crossWorldAvatars.length === 0 ? (
                <div className="text-center py-12 text-muted-foreground text-sm">
                  No avatars found in other worlds.
                </div>
              ) : (
                <div className="grid grid-cols-2 gap-3 p-4">
                  {crossWorldAvatars.map((avatar) => (
                    <button
                      key={`${avatar.world_id}-${avatar.avatar_file}`}
                      onClick={() => handleSelectCrossWorld(avatar.avatar_file)}
                      className="flex flex-col items-center gap-2 p-3 rounded-xl border-2 border-border hover:border-primary/50 transition-all cursor-pointer"
                    >
                      <img
                        src={avatar.data_url}
                        alt=""
                        className="w-20 h-20 rounded-full object-cover ring-2 ring-border"
                      />
                      <span className="text-xs text-muted-foreground truncate w-full text-center">{avatar.world_name}</span>
                    </button>
                  ))}
                </div>
              )}
            </ScrollArea>
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* World Gallery Picker Modal */}
      <Dialog open={showWorldGallery} onClose={() => setShowWorldGallery(false)} className="max-w-3xl">
        <DialogContent>
          <DialogHeader onClose={() => setShowWorldGallery(false)}>
            <DialogTitle>Choose from World Gallery</DialogTitle>
            <DialogDescription>
              Select any image from this world to use as your avatar.
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
    </div>
  );
}

function FactsList({ items, onChange, placeholder }: {
  items: string[];
  onChange: (items: string[]) => void;
  placeholder: string;
}) {
  const [newItem, setNewItem] = useState("");

  return (
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
  );
}

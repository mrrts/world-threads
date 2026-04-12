import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";

import { ScrollArea } from "@/components/ui/scroll-area";
import { Field, FieldGroup } from "@/components/ui/field";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter } from "@/components/ui/dialog";
import { Save, Plus, X, Trash2, AlertTriangle, ImagePlus, Loader2, Check, BookTemplate } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type World, type WorldState, type WorldImageInfo } from "@/lib/tauri";
import { WORLD_TEMPLATES, type WorldTemplate } from "@/lib/world-templates";

interface Props {
  store: ReturnType<typeof useAppStore>;
}


export function WorldCanonEditor({ store }: Props) {
  const world = store.activeWorld;
  const [form, setForm] = useState<Partial<World>>({});
  const [newTag, setNewTag] = useState("");
  const [newInvariant, setNewInvariant] = useState("");
  const [dirty, setDirty] = useState(false);
  const [showDelete, setShowDelete] = useState(false);
  const [generatingImage, setGeneratingImage] = useState(false);
  const [worldImages, setWorldImages] = useState<WorldImageInfo[]>([]);
  const [showGallery, setShowGallery] = useState(false);
  const [showTemplates, setShowTemplates] = useState(false);
  const [templateSearch, setTemplateSearch] = useState("");

  useEffect(() => {
    if (world) {
      api.listWorldImages(world.world_id).then(setWorldImages).catch(() => {});
    }
  }, [world?.world_id, store.activeWorldImage?.image_id]);

  const handleGenerateImage = async () => {
    if (!world || !store.apiKey || generatingImage) return;
    setGeneratingImage(true);
    try {
      await api.generateWorldImage(store.apiKey, world.world_id, {
        name: form.name ?? world.name,
        description: form.description ?? world.description,
        tone_tags: form.tone_tags ?? world.tone_tags,
      });
      await store.refreshWorldImage();
      const imgs = await api.listWorldImages(world.world_id);
      setWorldImages(imgs);
    } catch (e) {
      store.setError(String(e));
    } finally {
      setGeneratingImage(false);
    }
  };

  const handleSelectImage = async (imageId: string) => {
    if (!world) return;
    await api.setActiveWorldImage(world.world_id, imageId);
    await store.refreshWorldImage();
    const imgs = await api.listWorldImages(world.world_id);
    setWorldImages(imgs);
  };

  useEffect(() => {
    if (world) {
      setForm({
        name: world.name,
        description: world.description,
        tone_tags: [...(world.tone_tags ?? [])],
        invariants: [...(world.invariants ?? [])],
        state: structuredClone(world.state),
      });
      setDirty(false);
    }
  }, [world?.world_id]);

  if (!world) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <div className="text-center space-y-2">
          <p className="text-lg">No world selected</p>
          <p className="text-sm text-muted-foreground/60">Create or select a world to begin</p>
        </div>
      </div>
    );
  }

  const update = (patch: Partial<World>) => {
    setForm((f) => ({ ...f, ...patch }));
    setDirty(true);
  };

  const updateState = (patch: Partial<WorldState>) => {
    setForm((f) => ({
      ...f,
      state: { ...(f.state as WorldState), ...patch },
    }));
    setDirty(true);
  };

  const handleSave = async () => {
    await store.updateWorld({ ...world, ...form } as World);
    if (form.state) await store.updateWorldState(form.state as WorldState);
    setDirty(false);
  };

  const applyTemplate = (t: WorldTemplate) => {
    setForm((f) => ({
      ...f,
      name: t.name,
      description: t.description,
      tone_tags: [...t.tone_tags],
      invariants: [...t.invariants],
      state: {
        time: { day_index: 1, time_of_day: t.time_of_day },
        location: { current_scene: t.current_scene },
        global_arcs: [],
        facts: [],
      },
    }));
    setDirty(true);
    setShowTemplates(false);
  };

  const filteredTemplates = templateSearch.trim()
    ? WORLD_TEMPLATES.filter(
        (t) =>
          t.name.toLowerCase().includes(templateSearch.toLowerCase()) ||
          t.tagline.toLowerCase().includes(templateSearch.toLowerCase()),
      )
    : WORLD_TEMPLATES;

  const tags = (form.tone_tags ?? []) as string[];
  const invariants = (form.invariants ?? []) as string[];
  const state = (form.state ?? world.state) as WorldState;

  return (
    <>
      <div className="flex-1 flex flex-col min-h-0">
        <div className="px-6 py-3 border-b border-border flex items-center justify-between">
          <h1 className="font-semibold">World Canon</h1>
          <div className="flex items-center gap-2">
            {dirty && (
              <span className="text-xs text-primary bg-primary/10 px-2 py-0.5 rounded-full">
                Unsaved changes
              </span>
            )}
            <Button size="sm" variant="outline" onClick={() => { setTemplateSearch(""); setShowTemplates(true); }}>
              <BookTemplate size={14} className="mr-1.5" /> Starter Templates
            </Button>
            <Button size="sm" onClick={handleSave} disabled={!dirty}>
              <Save size={14} className="mr-1.5" /> Save
            </Button>
            <Button size="sm" variant="ghost" className="text-muted-foreground hover:text-destructive" onClick={() => setShowDelete(true)}>
              <Trash2 size={14} />
            </Button>
          </div>
        </div>

        <ScrollArea className="flex-1 px-6 py-6">
          <div className="max-w-2xl space-y-8">
            <Field label="World Name">
              <Input value={form.name ?? ""} onChange={(e) => update({ name: e.target.value })} />
            </Field>

            <FieldGroup label="World Image">
              <p className="text-xs text-muted-foreground mb-3">
                Generate a watercolor landscape from the world description. Visit the <strong>Gallery</strong> to generate with custom prompts, upload images, or manage all world images.
              </p>
              <div className="flex items-start gap-4">
                {store.activeWorldImage?.data_url ? (
                  <img
                    src={store.activeWorldImage.data_url}
                    alt=""
                    className="w-64 rounded-xl object-cover ring-1 ring-border"
                  />
                ) : (
                  <div className="w-64 h-36 rounded-xl bg-muted/30 border border-dashed border-border flex items-center justify-center text-muted-foreground/40 text-xs">
                    No image yet
                  </div>
                )}
                <div className="space-y-2">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={handleGenerateImage}
                    disabled={generatingImage || !store.apiKey}
                  >
                    {generatingImage ? (
                      <><Loader2 size={14} className="mr-1.5 animate-spin" /> Generating...</>
                    ) : (
                      <><ImagePlus size={14} className="mr-1.5" /> Generate Image</>
                    )}
                  </Button>
                  {worldImages.length > 1 && (
                    <Button size="sm" variant="ghost" className="text-xs" onClick={() => setShowGallery(true)}>
                      View all ({worldImages.length})
                    </Button>
                  )}
                </div>
              </div>
            </FieldGroup>

            <Field label="Description" hint="Setting, mood, atmosphere, history — everything that defines your world">
              <Textarea
                className="min-h-[140px]"
                value={form.description ?? ""}
                onChange={(e) => update({ description: e.target.value })}
                placeholder="A quiet coastal town where fog rolls in every evening and nothing is quite what it seems..."
              />
            </Field>

            <Field label="Tone Tags" hint="Short descriptors for the world's feeling">
              <div className="flex flex-wrap gap-1.5 mt-1">
                {tags.map((tag, i) => (
                  <Badge key={i} variant="secondary" className="gap-1.5 pr-1 cursor-pointer hover:bg-destructive/20 transition-colors" onClick={() => {
                    update({ tone_tags: tags.filter((_, j) => j !== i) });
                  }}>
                    {tag}
                    <span className="rounded-full bg-foreground/10 p-0.5"><X size={8} /></span>
                  </Badge>
                ))}
                <Input
                  className="h-7 w-36 text-xs"
                  value={newTag}
                  onChange={(e) => setNewTag(e.target.value)}
                  placeholder="Add tag ↵"
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && newTag.trim()) {
                      update({ tone_tags: [...tags, newTag.trim()] });
                      setNewTag("");
                    }
                  }}
                />
              </div>
            </Field>

            <Field label="Invariants" hint="Rules that must never be broken in this world">
              <div className="space-y-2">
                {invariants.map((inv, i) => (
                  <div key={i} className="flex items-start gap-2 group">
                    <div className="mt-2.5 w-1.5 h-1.5 rounded-full bg-primary/40 flex-shrink-0" />
                    <Input className="flex-1" value={inv} onChange={(e) => {
                      const updated = [...invariants];
                      updated[i] = e.target.value;
                      update({ invariants: updated });
                    }} />
                    <Button variant="ghost" size="icon" className="h-9 w-9 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive flex-shrink-0" onClick={() => {
                      update({ invariants: invariants.filter((_, j) => j !== i) });
                    }}>
                      <X size={14} />
                    </Button>
                  </div>
                ))}
                <div className="flex items-center gap-2">
                  <div className="mt-0 w-1.5 h-1.5 rounded-full bg-border flex-shrink-0" />
                  <Input
                    className="flex-1"
                    value={newInvariant}
                    onChange={(e) => setNewInvariant(e.target.value)}
                    placeholder="Add a world rule..."
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && newInvariant.trim()) {
                        update({ invariants: [...invariants, newInvariant.trim()] });
                        setNewInvariant("");
                      }
                    }}
                  />
                  <Button variant="outline" size="sm" className="h-9 flex-shrink-0" onClick={() => {
                    if (newInvariant.trim()) {
                      update({ invariants: [...invariants, newInvariant.trim()] });
                      setNewInvariant("");
                    }
                  }}>
                    <Plus size={14} className="mr-1" /> Add
                  </Button>
                </div>
              </div>
            </Field>
          </div>
        </ScrollArea>
      </div>

      {showGallery && (
        <Dialog open onClose={() => setShowGallery(false)}>
          <DialogContent className="max-w-3xl">
            <DialogHeader onClose={() => setShowGallery(false)}>
              <DialogTitle>World Images</DialogTitle>
              <DialogDescription>Select an image to make it the active world image.</DialogDescription>
            </DialogHeader>
            <div className="grid grid-cols-2 gap-3 p-4 max-h-[60vh] overflow-y-auto">
              {worldImages.map((img) => (
                <button
                  key={img.image_id}
                  onClick={() => handleSelectImage(img.image_id)}
                  className={`relative rounded-xl overflow-hidden ring-2 transition-all cursor-pointer ${
                    img.is_active ? "ring-primary" : "ring-transparent hover:ring-primary/30"
                  }`}
                >
                  <img src={img.data_url} alt="" className="w-full aspect-video object-cover" />
                  {img.is_active && (
                    <div className="absolute top-2 right-2 w-6 h-6 rounded-full bg-primary flex items-center justify-center">
                      <Check size={12} className="text-primary-foreground" />
                    </div>
                  )}
                  <div className="absolute bottom-0 inset-x-0 bg-gradient-to-t from-black/60 to-transparent p-2">
                    <span className="text-[10px] text-white/70">{new Date(img.created_at).toLocaleDateString()}</span>
                  </div>
                </button>
              ))}
            </div>
          </DialogContent>
        </Dialog>
      )}

      <Dialog open={showTemplates} onClose={() => setShowTemplates(false)} className="max-w-2xl">
        <DialogContent>
          <DialogHeader onClose={() => setShowTemplates(false)}>
            <DialogTitle>Choose a World Template</DialogTitle>
            <DialogDescription>Pick a genre to pre-fill all fields. You can customize everything after.</DialogDescription>
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

      <Dialog open={showDelete} onClose={() => setShowDelete(false)}>
        <DialogContent>
          <DialogHeader onClose={() => setShowDelete(false)}>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle size={16} className="text-destructive" /> Delete World
            </DialogTitle>
            <DialogDescription>
              This will permanently delete <strong>{world.name}</strong> and all its characters, messages, and events. This can't be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowDelete(false)}>Cancel</Button>
            <Button variant="destructive" onClick={async () => {
              await store.deleteWorld(world.world_id);
              setShowDelete(false);
            }}>
              Delete Forever
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

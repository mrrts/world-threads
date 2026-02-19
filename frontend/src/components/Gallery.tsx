import { useState, useEffect, useRef, useCallback } from "react";
import Cropper from "react-easy-crop";
import type { Area } from "react-easy-crop";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter,
} from "@/components/ui/dialog";
import {
  ImagePlus, Upload, Loader2, Maximize2, Globe, User, Users,
  Archive, ArchiveRestore, Trash2, Crop, ChevronDown, ChevronRight,
} from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type GalleryItem } from "@/lib/tauri";

interface Props {
  store: ReturnType<typeof useAppStore>;
}

const CATEGORY_META: Record<string, { icon: React.ReactNode; title: string }> = {
  world: { icon: <Globe size={14} />, title: "World Images" },
  character: { icon: <Users size={14} />, title: "Character Portraits" },
  user: { icon: <User size={14} />, title: "Your Avatar" },
};

export function Gallery({ store }: Props) {
  const worldId = store.activeWorld?.world_id;
  const [items, setItems] = useState<GalleryItem[]>([]);
  const [prompt, setPrompt] = useState("");
  const [generating, setGenerating] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [previewItem, setPreviewItem] = useState<GalleryItem | null>(null);
  const [cropItem, setCropItem] = useState<GalleryItem | null>(null);
  const [showArchived, setShowArchived] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadGallery = useCallback(async () => {
    if (!worldId) return;
    try {
      setItems(await api.listWorldGallery(worldId));
    } catch {}
  }, [worldId]);

  useEffect(() => {
    loadGallery();
  }, [loadGallery, store.activeWorldImage?.image_id, store.activePortraits, store.userProfile?.avatar_file]);

  if (!store.activeWorld) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <div className="text-center space-y-2">
          <p className="text-lg">No world selected</p>
          <p className="text-sm text-muted-foreground/60">Create or select a world to view its gallery</p>
        </div>
      </div>
    );
  }

  const handleGenerate = async () => {
    if (!worldId || !store.apiKey || !prompt.trim()) return;
    setGenerating(true);
    try {
      await api.generateWorldImageWithPrompt(store.apiKey, worldId, prompt.trim());
      setPrompt("");
      await loadGallery();
    } catch (e: any) {
      store.setError(String(e));
    } finally {
      setGenerating(false);
    }
  };

  const handleUpload = () => fileInputRef.current?.click();

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
      await api.uploadWorldImage(worldId, dataUrl, file.name);
      await loadGallery();
    } catch (e: any) {
      store.setError(String(e));
    } finally {
      setUploading(false);
      if (fileInputRef.current) fileInputRef.current.value = "";
    }
  };

  const handleArchive = async (item: GalleryItem) => {
    try {
      await api.archiveGalleryItem(item.id, item.category);
      await loadGallery();
    } catch (e: any) {
      store.setError(String(e));
    }
  };

  const handleUnarchive = async (item: GalleryItem) => {
    try {
      await api.unarchiveGalleryItem(item.id, item.category);
      await loadGallery();
    } catch (e: any) {
      store.setError(String(e));
    }
  };

  const handleDelete = async (item: GalleryItem) => {
    try {
      await api.deleteGalleryItem(item.id, item.category, item.file_name);
      await loadGallery();
    } catch (e: any) {
      store.setError(String(e));
    }
  };

  const handleCropSaved = async (dataUrl: string) => {
    if (!worldId || !cropItem) return;
    try {
      await api.saveCrop(worldId, cropItem.category, cropItem.source_id, dataUrl);
      setCropItem(null);
      await loadGallery();
    } catch (e: any) {
      store.setError(String(e));
    }
  };

  const activeItems = items.filter((i) => !i.is_archived);
  const archivedItems = items.filter((i) => i.is_archived);

  const groupItems = (list: GalleryItem[]) => {
    const world = list.filter((i) => i.category === "world");
    const character = list.filter((i) => i.category === "character");
    const user = list.filter((i) => i.category === "user");
    return [
      { key: "world", items: world },
      { key: "character", items: character },
      { key: "user", items: user },
    ].filter((s) => s.items.length > 0);
  };

  const renderCard = (item: GalleryItem, options: { archived?: boolean }) => {
    const isArchivable = item.category === "world" || item.category === "character";
    return (
      <div
        key={item.id}
        className="group relative rounded-xl overflow-hidden border border-border bg-card/30"
      >
        {item.data_url ? (
          <img
            src={item.data_url}
            alt=""
            className={`w-full object-cover ${item.category === "character" || item.category === "user" ? "aspect-square" : "aspect-video"}`}
          />
        ) : (
          <div className={`w-full bg-muted flex items-center justify-center text-muted-foreground/30 text-xs ${item.category === "character" || item.category === "user" ? "aspect-square" : "aspect-video"}`}>
            Missing
          </div>
        )}

        {/* Hover overlay */}
        <div className="absolute inset-0 bg-black/0 group-hover:bg-black/50 transition-all flex items-center justify-center gap-1.5 opacity-0 group-hover:opacity-100">
          <Button size="sm" variant="secondary" className="h-7 text-xs px-2" onClick={() => setPreviewItem(item)}>
            <Maximize2 size={11} className="mr-1" /> View
          </Button>
          {item.data_url && (
            <Button size="sm" variant="secondary" className="h-7 text-xs px-2" onClick={() => setCropItem(item)}>
              <Crop size={11} className="mr-1" /> Crop
            </Button>
          )}
          {isArchivable && !options.archived && (
            <Button size="sm" variant="secondary" className="h-7 text-xs px-2" onClick={() => handleArchive(item)}>
              <Archive size={11} />
            </Button>
          )}
          {options.archived && (
            <>
              <Button size="sm" variant="secondary" className="h-7 text-xs px-2" onClick={() => handleUnarchive(item)}>
                <ArchiveRestore size={11} />
              </Button>
              <Button size="sm" variant="destructive" className="h-7 text-xs px-2" onClick={() => handleDelete(item)}>
                <Trash2 size={11} />
              </Button>
            </>
          )}
        </div>

        {/* Tags */}
        {item.tags.length > 0 && (
          <div className="absolute top-2 left-2 flex gap-1">
            {item.tags.map((tag) => (
              <span key={tag} className="text-[9px] font-semibold bg-primary/80 text-primary-foreground px-1.5 py-0.5 rounded-full leading-none">
                {tag}
              </span>
            ))}
          </div>
        )}

        {/* Info bar */}
        <div className="p-2.5 space-y-0.5">
          <p className="text-xs text-foreground/80 font-medium truncate">{item.label}</p>
          {item.prompt && (
            <p className="text-[11px] text-muted-foreground/60 line-clamp-1">{item.prompt}</p>
          )}
          <p className="text-[10px] text-muted-foreground/40">
            {new Date(item.created_at).toLocaleDateString()}
          </p>
        </div>
      </div>
    );
  };

  const sections = groupItems(activeItems);
  const archivedSections = groupItems(archivedItems);

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div className="px-6 py-3 border-b border-border flex items-center justify-between">
        <div>
          <h1 className="font-semibold">Gallery</h1>
          <span className="text-xs text-muted-foreground/50">
            {store.activeWorld.name} — {activeItems.length} image{activeItems.length !== 1 ? "s" : ""}
            {archivedItems.length > 0 && ` · ${archivedItems.length} archived`}
          </span>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handleUpload}
          disabled={uploading}
        >
          {uploading ? <Loader2 size={14} className="mr-1.5 animate-spin" /> : <Upload size={14} className="mr-1.5" />}
          {uploading ? "Uploading..." : "Upload Image"}
        </Button>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept="image/png,image/jpeg,image/webp"
        className="hidden"
        onChange={handleFileSelected}
      />

      <ScrollArea className="flex-1">
        <div className="p-6 space-y-8 max-w-4xl">
          {/* Generate section */}
          <div className="rounded-xl border border-border bg-card/50 p-5 space-y-3">
            <div className="flex items-center gap-2">
              <ImagePlus size={16} className="text-primary" />
              <h2 className="text-sm font-medium">Generate New Image</h2>
            </div>
            <p className="text-xs text-muted-foreground leading-relaxed">
              Describe the scene you want. Images are generated at 1792×1024 in watercolor style — large enough for chat backgrounds.
            </p>
            <Textarea
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="e.g. A misty harbor at dawn, fishing boats rocking gently, warm lantern light reflecting off still water..."
              className="min-h-[80px] resize-none"
            />
            <div className="flex items-center justify-between">
              <span className="text-[10px] text-muted-foreground/50">
                Uses DALL-E 3 • 1792×1024
              </span>
              <Button
                size="sm"
                onClick={handleGenerate}
                disabled={generating || !prompt.trim() || !store.apiKey}
              >
                {generating ? (
                  <><Loader2 size={14} className="mr-1.5 animate-spin" /> Generating...</>
                ) : (
                  <><ImagePlus size={14} className="mr-1.5" /> Generate</>
                )}
              </Button>
            </div>
          </div>

          {/* Active images */}
          {activeItems.length === 0 ? (
            <div className="py-16 text-center text-muted-foreground/60">
              <ImagePlus size={40} className="mx-auto mb-3 opacity-30" />
              <p className="text-sm">No images yet</p>
              <p className="text-xs mt-1">Generate or upload an image, create character portraits, or set a user avatar</p>
            </div>
          ) : (
            sections.map(({ key, items: sectionItems }) => {
              const meta = CATEGORY_META[key];
              return (
                <div key={key} className="space-y-3">
                  <div className="flex items-center gap-2 text-muted-foreground">
                    {meta.icon}
                    <h3 className="text-xs font-semibold uppercase tracking-wider">{meta.title}</h3>
                    <span className="text-[10px] text-muted-foreground/50">({sectionItems.length})</span>
                  </div>
                  <div className={`grid gap-4 ${key === "user" ? "grid-cols-4" : "grid-cols-2"}`}>
                    {sectionItems.map((item) => renderCard(item, { archived: false }))}
                  </div>
                </div>
              );
            })
          )}

          {/* Archived section */}
          {archivedItems.length > 0 && (
            <div className="border-t border-border pt-6">
              <button
                onClick={() => setShowArchived(!showArchived)}
                className="flex items-center gap-2 text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
              >
                {showArchived ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                <Archive size={14} />
                <span className="text-xs font-semibold uppercase tracking-wider">
                  Archived
                </span>
                <span className="text-[10px] text-muted-foreground/50">({archivedItems.length})</span>
              </button>
              {showArchived && (
                <div className="mt-4 space-y-6">
                  {archivedSections.map(({ key, items: sectionItems }) => {
                    const meta = CATEGORY_META[key];
                    return (
                      <div key={key} className="space-y-3">
                        <div className="flex items-center gap-2 text-muted-foreground/60">
                          {meta.icon}
                          <h3 className="text-xs font-semibold uppercase tracking-wider">{meta.title}</h3>
                          <span className="text-[10px] text-muted-foreground/40">({sectionItems.length})</span>
                        </div>
                        <div className={`grid gap-4 ${key === "user" ? "grid-cols-4" : "grid-cols-2"} opacity-70`}>
                          {sectionItems.map((item) => renderCard(item, { archived: true }))}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Full-size preview modal */}
      {previewItem && (
        <Dialog open onClose={() => setPreviewItem(null)}>
          <DialogContent className="max-w-4xl">
            <DialogHeader onClose={() => setPreviewItem(null)}>
              <DialogTitle>{previewItem.label}</DialogTitle>
              {previewItem.prompt && (
                <DialogDescription>{previewItem.prompt}</DialogDescription>
              )}
            </DialogHeader>
            <div className="p-2">
              {previewItem.data_url && (
                <img
                  src={previewItem.data_url}
                  alt=""
                  className="w-full rounded-lg"
                />
              )}
              <div className="flex items-center justify-between mt-3 px-1">
                <span className="text-xs text-muted-foreground">
                  {previewItem.category === "world" ? previewItem.label : `${CATEGORY_META[previewItem.category]?.title}`} · {new Date(previewItem.created_at).toLocaleDateString()}
                </span>
                {previewItem.tags.length > 0 && (
                  <div className="flex gap-1">
                    {previewItem.tags.map((tag) => (
                      <span key={tag} className="text-[10px] font-medium bg-primary/15 text-primary px-1.5 py-0.5 rounded-full">{tag}</span>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </DialogContent>
        </Dialog>
      )}

      {/* Crop modal */}
      {cropItem && cropItem.data_url && (
        <CropModal
          imageUrl={cropItem.data_url}
          onSave={handleCropSaved}
          onClose={() => setCropItem(null)}
        />
      )}
    </div>
  );
}

const ASPECT_PRESETS = [
  { label: "Free", value: 0 },
  { label: "1:1", value: 1 },
  { label: "4:3", value: 4 / 3 },
  { label: "16:9", value: 16 / 9 },
  { label: "3:2", value: 3 / 2 },
  { label: "2:3", value: 2 / 3 },
];

function CropModal({ imageUrl, onSave, onClose }: { imageUrl: string; onSave: (dataUrl: string) => void; onClose: () => void }) {
  const [crop, setCrop] = useState({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(1);
  const [aspect, setAspect] = useState(0);
  const [naturalAspect, setNaturalAspect] = useState(16 / 9);
  const [croppedArea, setCroppedArea] = useState<Area | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    const img = new Image();
    img.onload = () => {
      setNaturalAspect(img.naturalWidth / img.naturalHeight);
    };
    img.src = imageUrl;
  }, [imageUrl]);

  const effectiveAspect = aspect === 0 ? naturalAspect : aspect;

  const onCropComplete = useCallback((_: Area, croppedPixels: Area) => {
    setCroppedArea(croppedPixels);
  }, []);

  const handleSave = async () => {
    if (!croppedArea) return;
    setSaving(true);
    try {
      const canvas = document.createElement("canvas");
      const ctx = canvas.getContext("2d")!;
      const img = new Image();
      img.crossOrigin = "anonymous";

      const dataUrl: string = await new Promise((resolve, reject) => {
        img.onload = () => {
          canvas.width = croppedArea.width;
          canvas.height = croppedArea.height;
          ctx.drawImage(
            img,
            croppedArea.x, croppedArea.y, croppedArea.width, croppedArea.height,
            0, 0, croppedArea.width, croppedArea.height,
          );
          resolve(canvas.toDataURL("image/png"));
        };
        img.onerror = reject;
        img.src = imageUrl;
      });

      onSave(dataUrl);
    } catch {
      setSaving(false);
    }
  };

  return (
    <Dialog open onClose={onClose}>
      <DialogContent className="max-w-2xl">
        <DialogHeader onClose={onClose}>
          <DialogTitle>Crop Image</DialogTitle>
          <DialogDescription>Drag to reposition, scroll to zoom. The cropped area will be saved as a new image.</DialogDescription>
        </DialogHeader>
        <DialogBody className="p-0">
          <div className="relative w-full" style={{ height: 400 }}>
            <Cropper
              image={imageUrl}
              crop={crop}
              zoom={zoom}
              aspect={effectiveAspect}
              onCropChange={setCrop}
              onZoomChange={setZoom}
              onCropComplete={onCropComplete}
            />
          </div>
          <div className="px-5 py-3 space-y-3">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="text-xs text-muted-foreground mr-1">Ratio</span>
              {ASPECT_PRESETS.map((p) => (
                <button
                  key={p.label}
                  onClick={() => { setAspect(p.value); setCrop({ x: 0, y: 0 }); }}
                  className={`text-[11px] px-2 py-1 rounded-md border transition-colors cursor-pointer ${
                    aspect === p.value
                      ? "border-primary bg-primary/15 text-primary font-medium"
                      : "border-border text-muted-foreground hover:border-primary/40"
                  }`}
                >
                  {p.label}
                </button>
              ))}
            </div>
            <div className="flex items-center gap-3">
              <span className="text-xs text-muted-foreground">Zoom</span>
              <input
                type="range"
                min={1}
                max={3}
                step={0.05}
                value={zoom}
                onChange={(e) => setZoom(Number(e.target.value))}
                className="flex-1 accent-primary h-1"
              />
              <span className="text-xs text-muted-foreground font-mono w-10 text-right">{zoom.toFixed(1)}×</span>
            </div>
          </div>
        </DialogBody>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>Cancel</Button>
          <Button onClick={handleSave} disabled={saving || !croppedArea}>
            {saving ? <><Loader2 size={14} className="mr-1.5 animate-spin" /> Saving...</> : <><Crop size={14} className="mr-1.5" /> Save Crop</>}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

import { useState, useEffect, useCallback, useRef } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { RefreshCw, MessageSquare, Settings, Loader2, User, ChevronLeft, ChevronRight, Play, Pause, Square } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type Character, type PortraitInfo, type GalleryItem } from "@/lib/tauri";
import { useSlideshow } from "@/hooks/use-slideshow";

interface Props {
  store: ReturnType<typeof useAppStore>;
  onChat: (characterId: string) => void;
  onSettings: (characterId: string) => void;
}

interface CharSummary {
  character: Character;
  portrait: PortraitInfo | null;
  summary: string;
  messageCount: number;
}


export function WorldSummary({ store, onChat, onSettings }: Props) {
  const world = store.activeWorld;
  const [summaries, setSummaries] = useState<CharSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [regenerating, setRegenerating] = useState(false);
  const [userAvatarUrl, setUserAvatarUrl] = useState("");
  const [illustrations, setIllustrations] = useState<Array<{ id: string; data_url: string }>>([]);
  const [videoFiles, setVideoFiles] = useState<Record<string, string>>({});
  const [videoDataUrls, setVideoDataUrls] = useState<Record<string, string>>({});
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [playingVideo, setPlayingVideo] = useState(false);

  useEffect(() => {
    if (!world) return;
    api.getUserAvatar(world.world_id).then((url) => setUserAvatarUrl(url || "")).catch(() => {});
  }, [world?.world_id, store.userProfile?.avatar_file]);

  // Load world gallery images first, then all chat illustrations sorted by timestamp
  useEffect(() => {
    if (!world) return;
    (async () => {
      const allIllus: Array<{ id: string; data_url: string; created_at: string }> = [];
      const vf: Record<string, string> = {};

      // Active world image first
      if (store.activeWorldImage?.data_url) {
        allIllus.push({ id: store.activeWorldImage.image_id, data_url: store.activeWorldImage.data_url, created_at: "" });
      }

      const chatIllus: Array<{ id: string; data_url: string; created_at: string }> = [];

      // Individual chat illustrations
      for (const ch of store.characters) {
        try {
          const page = await api.getMessages(ch.character_id);
          for (const m of page.messages) {
            if (m.role === "illustration") {
              chatIllus.push({ id: m.message_id, data_url: m.content, created_at: m.created_at });
              try {
                const f = await api.getVideoFile(m.message_id);
                if (f) vf[m.message_id] = f;
              } catch { /* ignore */ }
            }
          }
        } catch { /* ignore */ }
      }

      // Group chat illustrations
      for (const gc of store.groupChats) {
        try {
          const page = await api.getGroupMessages(gc.group_chat_id);
          for (const m of page.messages) {
            if (m.role === "illustration") {
              chatIllus.push({ id: m.message_id, data_url: m.content, created_at: m.created_at });
              try {
                const f = await api.getVideoFile(m.message_id);
                if (f) vf[m.message_id] = f;
              } catch { /* ignore */ }
            }
          }
        } catch { /* ignore */ }
      }

      // Sort chat illustrations by timestamp ascending
      chatIllus.sort((a, b) => a.created_at.localeCompare(b.created_at));

      // World gallery first, then chat illustrations in chronological order
      const combined = [...allIllus, ...chatIllus];

      setIllustrations(combined.map(({ id, data_url }) => ({ id, data_url })));
      setVideoFiles(vf);
      if (combined.length > 0 && !selectedId) {
        setSelectedId(combined[0].id);
      }
    })();
  }, [world?.world_id, store.characters.length, store.groupChats.length]);

  const slideshow = useSlideshow({
    illustrations,
    videoDataUrls,
    videoFiles,
    loadVideoUrl: async (illustrationId: string, videoFile: string) => {
      const bytes = await api.getVideoBytes(videoFile);
      const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: "video/mp4" }));
      setVideoDataUrls((prev) => ({ ...prev, [illustrationId]: url }));
    },
  });

  // Auto-start slideshow when illustrations load
  useEffect(() => {
    if (illustrations.length > 1 && !slideshow.active) {
      slideshow.toggle();
    }
  }, [illustrations.length]);

  // Sync slideshow to selected illustration
  useEffect(() => {
    if (slideshow.active && slideshow.currentSlide) {
      setSelectedId(slideshow.currentSlide.illustrationId);
      setPlayingVideo(slideshow.currentSlide.type === "video");
    }
  }, [slideshow.active, slideshow.slideIndex, slideshow.currentSlide]);

  const loadSummaries = useCallback(async () => {
    if (!world) return;
    setLoading(true);
    try {
      const results: CharSummary[] = [];
      for (const ch of store.characters) {
        const summary = await api.getThreadSummary(ch.character_id);
        const portrait = store.activePortraits[ch.character_id] ?? null;
        const paginated = await api.getMessages(ch.character_id, 1, 0);
        results.push({
          character: ch,
          portrait,
          summary,
          messageCount: paginated.total,
        });
      }
      setSummaries(results);
    } catch {
    } finally {
      setLoading(false);
    }
  }, [world, store.characters, store.activePortraits]);

  useEffect(() => {
    loadSummaries();
  }, [loadSummaries]);

  const handleRegenerate = async () => {
    if (!store.apiKey) return;
    setRegenerating(true);
    try {
      for (const ch of store.characters) {
        await api.getThreadSummary(ch.character_id);
      }
      await loadSummaries();
    } finally {
      setRegenerating(false);
    }
  };

  if (!world) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <div className="text-center space-y-2">
          <p className="text-lg">No world selected</p>
          <p className="text-sm text-muted-foreground/60">Create or select a world to see its summary</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <ScrollArea className="flex-1">
        {/* Illustration/Video Viewer */}
        {illustrations.length > 0 ? (() => {
          const selected = illustrations.find((i) => i.id === selectedId);
          return (
          <div className="bg-black">
            <div className="relative w-full flex items-center justify-center min-h-[300px] max-h-[600px] group/viewer">
              {playingVideo && selectedId && videoDataUrls[selectedId] ? (
                <video
                  key={`video-${selectedId}`}
                  src={videoDataUrls[selectedId]}
                  autoPlay
                  loop={!slideshow.active}
                  muted
                  playsInline
                  className="max-w-full max-h-[600px] object-contain"
                  onTimeUpdate={slideshow.active ? (e) => {
                    const v = e.currentTarget;
                    slideshow.onVideoTimeUpdate(v.currentTime, v.duration);
                  } : undefined}
                  onEnded={slideshow.active ? slideshow.onVideoEnded : undefined}
                />
              ) : selected ? (
                <img
                  key={selectedId}
                  src={selected.data_url}
                  alt="Illustration"
                  className="max-w-full max-h-[600px] object-contain"
                />
              ) : null}
              {/* Slideshow toggle */}
              {illustrations.length > 1 && (
                <button
                  onClick={() => slideshow.toggle()}
                  className="absolute top-3 right-3 z-20 flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-black/50 text-white text-[10px] font-medium cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm opacity-0 group-hover/viewer:opacity-100"
                >
                  {slideshow.active ? <Pause size={10} /> : <Play size={10} />}
                  Slideshow
                </button>
              )}
              {/* Manual nav arrows (when slideshow is off) */}
              {illustrations.length > 1 && !slideshow.active && (
                <>
                  <button
                    onClick={() => {
                      const idx = illustrations.findIndex((i) => i.id === selectedId);
                      const prev = idx <= 0 ? illustrations.length - 1 : idx - 1;
                      setSelectedId(illustrations[prev].id);
                      setPlayingVideo(false);
                    }}
                    className="absolute left-2 top-1/2 -translate-y-1/2 z-20 w-10 h-10 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-all backdrop-blur-sm opacity-0 group-hover/viewer:opacity-100"
                  >
                    <ChevronLeft size={20} />
                  </button>
                  <button
                    onClick={() => {
                      const idx = illustrations.findIndex((i) => i.id === selectedId);
                      const next = idx >= illustrations.length - 1 ? 0 : idx + 1;
                      setSelectedId(illustrations[next].id);
                      setPlayingVideo(false);
                    }}
                    className="absolute right-2 top-1/2 -translate-y-1/2 z-20 w-10 h-10 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-all backdrop-blur-sm opacity-0 group-hover/viewer:opacity-100"
                  >
                    <ChevronRight size={20} />
                  </button>
                </>
              )}
              {/* Video play/stop (manual, when slideshow is off) */}
              {!slideshow.active && selectedId && videoFiles[selectedId] && !playingVideo && (
                <button
                  onClick={async () => {
                    if (!videoDataUrls[selectedId]) {
                      try {
                        const bytes = await api.getVideoBytes(videoFiles[selectedId]);
                        const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: "video/mp4" }));
                        setVideoDataUrls((prev) => ({ ...prev, [selectedId]: url }));
                      } catch { return; }
                    }
                    setPlayingVideo(true);
                  }}
                  className="absolute bottom-4 right-4 z-20 w-12 h-12 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-purple-600 transition-colors backdrop-blur-sm opacity-0 group-hover/viewer:opacity-100"
                >
                  <span className="text-xl ml-0.5">&#9654;</span>
                </button>
              )}
              {!slideshow.active && playingVideo && (
                <button
                  onClick={() => setPlayingVideo(false)}
                  className="absolute bottom-4 right-4 z-20 w-12 h-12 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm opacity-0 group-hover/viewer:opacity-100"
                >
                  <Square size={16} fill="white" />
                </button>
              )}
              {/* Progress bar */}
              {slideshow.active && (
                <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/10 z-30">
                  <div className="h-full bg-primary transition-none" style={{ width: `${slideshow.progress * 100}%` }} />
                </div>
              )}
            </div>
            {/* Thumbnail carousel */}
            {illustrations.length > 1 && (
              <div className="border-t border-border bg-card/50 px-2 py-2">
                <div className="flex gap-1.5 overflow-x-auto scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]">
                  {illustrations.map((illus) => (
                    <button
                      key={illus.id}
                      onClick={() => {
                        if (slideshow.active) {
                          slideshow.jumpTo(illus.id);
                        } else {
                          setSelectedId(illus.id);
                          setPlayingVideo(false);
                        }
                      }}
                      className={`relative flex-shrink-0 w-16 h-11 rounded-lg overflow-hidden transition-all cursor-pointer ${
                        illus.id === selectedId
                          ? "ring-2 ring-primary ring-offset-1 ring-offset-black"
                          : "ring-1 ring-white/10 opacity-60 hover:opacity-100"
                      }`}
                    >
                      <img src={illus.data_url} alt="" className="w-full h-full object-cover" />
                      {videoFiles[illus.id] && (
                        <div className="absolute bottom-0.5 right-0.5 w-3.5 h-3.5 rounded-full bg-purple-600 flex items-center justify-center">
                          <span className="text-white text-[6px]">&#9654;</span>
                        </div>
                      )}
                    </button>
                  ))}
                </div>
              </div>
            )}
            {/* World title overlay */}
            <div className="bg-background px-8 py-4">
              <h1 className="text-3xl font-bold text-foreground">{world.name}</h1>
              {world.description && (
                <p className="text-sm text-muted-foreground mt-1 max-w-xl">{world.description}</p>
              )}
            </div>
          </div>
          );
        })() : (
          <div className="bg-gradient-to-br from-primary/20 via-background to-primary/10 px-8 py-12">
            <h1 className="text-4xl font-bold text-foreground">{world.name}</h1>
            {world.description && (
              <p className="text-sm text-muted-foreground mt-2 max-w-xl">{world.description}</p>
            )}
          </div>
        )}

        {/* Content */}
        <div className="px-8 py-6 max-w-3xl space-y-6">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">Characters</h2>
            <Button
              variant="outline"
              size="sm"
              onClick={handleRegenerate}
              disabled={regenerating || loading}
            >
              {regenerating ? (
                <><Loader2 size={14} className="mr-1.5 animate-spin" /> Regenerating...</>
              ) : (
                <><RefreshCw size={14} className="mr-1.5" /> Regenerate Summaries</>
              )}
            </Button>
          </div>

          {loading && summaries.length === 0 ? (
            <div className="flex items-center justify-center py-16">
              <Loader2 size={24} className="animate-spin text-muted-foreground" />
            </div>
          ) : store.characters.length === 0 ? (
            <div className="text-center py-16 text-muted-foreground">
              <p>No characters yet. Create one to get started.</p>
            </div>
          ) : (
            <div className="space-y-4">
              {/* Me card */}
              {store.userProfile && (
                <div className="rounded-xl border border-border bg-card/60 p-5 flex gap-5">
                  <div className="flex-shrink-0">
                    {userAvatarUrl ? (
                      <img src={userAvatarUrl} alt="" className="w-16 h-16 rounded-xl object-cover" />
                    ) : (
                      <div className="w-16 h-16 rounded-xl bg-primary/20 flex items-center justify-center">
                        <User size={24} className="text-primary" />
                      </div>
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="font-semibold text-foreground">{store.userProfile.display_name || "Me"}</h3>
                      <span className="text-[10px] font-medium bg-primary/15 text-primary px-1.5 py-0.5 rounded-full leading-none">Me</span>
                    </div>
                    {store.userProfile.description ? (
                      <p className="text-sm text-muted-foreground leading-relaxed line-clamp-2">{store.userProfile.description}</p>
                    ) : (
                      <p className="text-sm text-muted-foreground/50 italic">No description yet.</p>
                    )}
                  </div>
                </div>
              )}
              {summaries.map(({ character, portrait, summary, messageCount }) => (
                <div
                  key={character.character_id}
                  className="rounded-xl border border-border bg-card/60 p-5 flex gap-5 group"
                >
                  {/* Avatar */}
                  <div className="flex-shrink-0">
                    {portrait?.data_url ? (
                      <img
                        src={portrait.data_url}
                        alt=""
                        className="w-16 h-16 rounded-xl object-cover"
                      />
                    ) : (
                      <div
                        className="w-16 h-16 rounded-xl flex items-center justify-center text-xl font-bold text-white"
                        style={{ backgroundColor: character.color || "#6366f1" }}
                      >
                        {character.display_name?.charAt(0) || "?"}
                      </div>
                    )}
                  </div>

                  {/* Info */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-3 mb-1">
                      <h3 className="font-semibold text-foreground">{character.display_name}</h3>
                      <span className="text-xs text-muted-foreground/50">
                        {messageCount} message{messageCount !== 1 ? "s" : ""}
                      </span>
                    </div>
                    {summary ? (
                      <p className="text-sm text-muted-foreground leading-relaxed">
                        {summary}
                      </p>
                    ) : (
                      <p className="text-sm text-muted-foreground/50 italic">
                        No conversation yet.
                      </p>
                    )}
                  </div>

                  {/* Actions */}
                  <div className="flex flex-col gap-1.5 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity pt-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      title="Chat"
                      onClick={() => onChat(character.character_id)}
                    >
                      <MessageSquare size={14} />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      title="Settings"
                      onClick={() => onSettings(character.character_id)}
                    >
                      <Settings size={14} />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}

import { useState, useEffect, useCallback, useRef } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { MessageSquare, Settings, Loader2, User, ChevronLeft, ChevronRight, Play, Pause, Square, BookOpen, X, Volume2, VolumeX } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type Character, type PortraitInfo, type GalleryItem } from "@/lib/tauri";
import { useSlideshow } from "@/hooks/use-slideshow";
import { SummaryModal } from "@/components/chat/SummaryModal";
import { PortraitModal } from "@/components/chat/PortraitModal";

interface Props {
  store: ReturnType<typeof useAppStore>;
  onChat: (characterId: string) => void;
  onSettings: (characterId: string) => void;
}

interface CharInfo {
  character: Character;
  portrait: PortraitInfo | null;
  messageCount: number;
  dialogueCount: number;
}


export function WorldSummary({ store, onChat, onSettings }: Props) {
  const world = store.activeWorld;
  const [charInfos, setCharInfos] = useState<CharInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [userAvatarUrl, setUserAvatarUrl] = useState("");
  const [summaryTarget, setSummaryTarget] = useState<{ type: "char" | "group"; id: string; name: string } | null>(null);
  const [groupDialogueCounts, setGroupDialogueCounts] = useState<Record<string, number>>({});
  const [portraitCharId, setPortraitCharId] = useState<string | null>(null);
  const [showCarouselModal, setShowCarouselModal] = useState(false);
  const [videoMuted, setVideoMuted] = useState(true);
  const [illustrations, setIllustrations] = useState<Array<{ id: string; data_url: string }>>([]);
  const [loadingCarousel, setLoadingCarousel] = useState(true);
  const [videoFiles, setVideoFiles] = useState<Record<string, string>>({});
  const [videoDataUrls, setVideoDataUrls] = useState<Record<string, string>>({});
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [playingVideo, setPlayingVideo] = useState(false);

  useEffect(() => {
    if (!world) return;
    api.getUserAvatar(world.world_id).then((url) => setUserAvatarUrl(url || "")).catch(() => {});
  }, [world?.world_id, store.userProfile?.avatar_file]);

  // Load world gallery images first, then all chat illustrations sorted by timestamp
  // Set initial world image for banner (no heavy loading)
  useEffect(() => {
    if (store.activeWorldImage?.data_url) {
      setIllustrations([{ id: store.activeWorldImage.image_id, data_url: store.activeWorldImage.data_url }]);
      setSelectedId(store.activeWorldImage.image_id);
    } else {
      setIllustrations([]);
      setSelectedId(null);
    }
  }, [world?.world_id, store.activeWorldImage]);

  // Gallery images are loaded on-demand when the carousel is opened (not on mount)
  const loadGalleryImages = useCallback(async () => {
    if (!world) return;
    setLoadingCarousel(true);

    const initial: Array<{ id: string; data_url: string }> = [];
    if (store.activeWorldImage?.data_url) {
      initial.push({ id: store.activeWorldImage.image_id, data_url: store.activeWorldImage.data_url });
    }

    // Load all chats in parallel
    const charPromises = store.characters.map(async (ch) => {
      try {
        const page = await api.getMessages(ch.character_id);
        return page.messages.filter((m) => m.role === "illustration").map((m) => ({ id: m.message_id, data_url: m.content, created_at: m.created_at }));
      } catch { return []; }
    });
    const gcPromises = store.groupChats.map(async (gc) => {
      try {
        const page = await api.getGroupMessages(gc.group_chat_id);
        return page.messages.filter((m) => m.role === "illustration").map((m) => ({ id: m.message_id, data_url: m.content, created_at: m.created_at }));
      } catch { return []; }
    });

    const allBatches = await Promise.all([...charPromises, ...gcPromises]);
    const chatIllus = allBatches.flat().sort((a, b) => a.created_at.localeCompare(b.created_at));

    setIllustrations([...initial, ...chatIllus.map(({ id, data_url }) => ({ id, data_url }))]);
    setSelectedId(initial[0]?.id ?? chatIllus[0]?.id ?? null);

    // Load video files in background (don't block)
    for (const illus of chatIllus) {
      api.getVideoFile(illus.id).then((f) => { if (f) setVideoFiles((prev) => ({ ...prev, [illus.id]: f })); }).catch(() => {});
    }

    setLoadingCarousel(false);
  }, [world?.world_id, store.characters, store.groupChats, store.activeWorldImage]);

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

  // Navigate to an illustration, auto-playing its video if it has one
  const navigateTo = useCallback(async (id: string) => {
    setSelectedId(id);
    if (videoFiles[id]) {
      if (!videoDataUrls[id]) {
        try {
          const bytes = await api.getVideoBytes(videoFiles[id]);
          const url = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: "video/mp4" }));
          setVideoDataUrls((prev) => ({ ...prev, [id]: url }));
        } catch { setPlayingVideo(false); return; }
      }
      setPlayingVideo(true);
    } else {
      setPlayingVideo(false);
    }
  }, [videoFiles, videoDataUrls]);

  // Sync slideshow to selected illustration
  useEffect(() => {
    if (slideshow.active && slideshow.currentSlide) {
      setSelectedId(slideshow.currentSlide.illustrationId);
      setPlayingVideo(slideshow.currentSlide.type === "video");
    }
  }, [slideshow.active, slideshow.slideIndex, slideshow.currentSlide]);

  const loadCharInfos = useCallback(async () => {
    if (!world) return;
    setLoading(true);
    try {
      // Load all character info in parallel
      const charResults = await Promise.all(
        store.characters.map(async (ch) => {
          const portrait = store.activePortraits[ch.character_id] ?? null;
          try {
            const paginated = await api.getMessages(ch.character_id);
            const dialogueCount = paginated.messages.filter((m) => m.role !== "illustration" && m.role !== "video").length;
            return { character: ch, portrait, messageCount: paginated.total, dialogueCount };
          } catch {
            return { character: ch, portrait, messageCount: 0, dialogueCount: 0 };
          }
        })
      );
      setCharInfos(charResults);

      // Load group chat dialogue counts in parallel
      const gcEntries = await Promise.all(
        store.groupChats.map(async (gc) => {
          try {
            const page = await api.getGroupMessages(gc.group_chat_id);
            return [gc.group_chat_id, page.messages.filter((m) => m.role !== "illustration" && m.role !== "video").length] as const;
          } catch { return [gc.group_chat_id, 0] as const; }
        })
      );
      setGroupDialogueCounts(Object.fromEntries(gcEntries));
    } catch {
    } finally {
      setLoading(false);
    }
  }, [world, store.characters, store.activePortraits, store.groupChats]);

  useEffect(() => {
    loadCharInfos();
  }, [loadCharInfos]);

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
        {/* Hero Banner */}
        {store.activeWorldImage?.data_url ? (
          <button
            onClick={() => { setShowCarouselModal(true); if (illustrations.length <= 1) loadGalleryImages(); }}
            className="relative w-full h-[700px] overflow-hidden cursor-pointer group/banner"
          >
            <img
              src={store.activeWorldImage.data_url}
              alt={world.name}
              className="absolute inset-0 w-full h-full object-cover"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-background via-background/30 to-transparent" />
            <div className="absolute inset-x-0 bottom-0 px-8 pb-8">
              <h1 className="text-4xl font-bold text-foreground drop-shadow-lg">{world.name}</h1>
              {world.description && (
                <p className="text-sm text-foreground/70 mt-2 max-w-xl drop-shadow">{world.description}</p>
              )}
              {illustrations.length > 0 && (
                <p className="text-xs text-foreground/40 mt-3 opacity-0 group-hover/banner:opacity-100 transition-opacity">
                  Click to browse {illustrations.length} illustration{illustrations.length !== 1 ? "s" : ""}
                </p>
              )}
            </div>
          </button>
        ) : (
          <div className="relative w-full h-[700px] overflow-hidden bg-gradient-to-br from-primary/20 via-background to-primary/10">
            <div className="absolute inset-0 bg-gradient-to-t from-background via-transparent to-transparent" />
            <div className="absolute inset-x-0 bottom-0 px-8 pb-8">
              <h1 className="text-4xl font-bold text-foreground">{world.name}</h1>
              {world.description && (
                <p className="text-sm text-muted-foreground mt-2 max-w-xl">{world.description}</p>
              )}
            </div>
          </div>
        )}

        {/* Content */}
        {/* Characters */}
        <div className="px-8 py-6 w-full space-y-6">
          <h2 className="text-lg font-semibold">Characters</h2>

          {loading && charInfos.length === 0 ? (
            <div className="flex items-center justify-center py-16">
              <Loader2 size={24} className="animate-spin text-muted-foreground" />
            </div>
          ) : store.characters.length === 0 ? (
            <div className="text-center py-16 text-muted-foreground">
              <p>No characters yet. Create one to get started.</p>
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-4">
              {store.userProfile && (
                <div className="rounded-xl border border-border bg-card/60 p-5">
                  {userAvatarUrl ? (
                    <img src={userAvatarUrl} alt="" className="w-32 h-32 rounded-xl object-cover float-left mr-4 mb-2" />
                  ) : (
                    <div className="w-32 h-32 rounded-xl bg-primary/20 flex items-center justify-center float-left mr-4 mb-2">
                      <User size={48} className="text-primary" />
                    </div>
                  )}
                  <div>
                    <div className="flex items-center gap-2 mb-1">
                      <h3 className="font-semibold text-foreground">{store.userProfile.display_name || "Me"}</h3>
                      <span className="text-[10px] font-medium bg-primary/15 text-primary px-1.5 py-0.5 rounded-full leading-none">Me</span>
                    </div>
                    {store.userProfile.description ? (
                      <p className="text-sm text-muted-foreground leading-relaxed">{store.userProfile.description}</p>
                    ) : (
                      <p className="text-sm text-muted-foreground/50 italic">No description yet.</p>
                    )}
                  </div>
                </div>
              )}
              {charInfos.map(({ character, portrait, messageCount, dialogueCount }) => (
                <div
                  key={character.character_id}
                  className="rounded-xl border border-border bg-card/60 p-5"
                >
                  {portrait?.data_url ? (
                    <button onClick={() => setPortraitCharId(character.character_id)} className="cursor-pointer float-left mr-4 mb-2">
                      <img src={portrait.data_url} alt="" className="w-32 h-32 rounded-xl object-cover hover:ring-2 hover:ring-primary/50 transition-all" />
                    </button>
                  ) : (
                    <div
                      className="w-32 h-32 rounded-xl flex items-center justify-center text-4xl font-bold text-white float-left mr-4 mb-2"
                      style={{ backgroundColor: character.avatar_color || "#6366f1" }}
                    >
                      {character.display_name?.charAt(0) || "?"}
                    </div>
                  )}
                  <div>
                    <div className="flex items-center gap-3 mb-1">
                      <h3 className="font-semibold text-foreground">{character.display_name}</h3>
                      <span className="text-xs text-muted-foreground/50">
                        {messageCount} message{messageCount !== 1 ? "s" : ""}
                      </span>
                    </div>
                    <p className="text-sm text-muted-foreground/60 leading-relaxed whitespace-pre-wrap">
                      {character.identity || <span className="italic">No identity set.</span>}
                    </p>
                  </div>
                  <div className="flex flex-col gap-1.5 flex-shrink-0 pt-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-xs"
                      disabled={dialogueCount === 0}
                      onClick={() => setSummaryTarget({ type: "char", id: character.character_id, name: character.display_name })}
                    >
                      <BookOpen size={12} className="mr-1.5" /> Summary
                    </Button>
                    <Button variant="ghost" size="sm" className="text-xs" onClick={() => onChat(character.character_id)}>
                      <MessageSquare size={12} className="mr-1.5" /> Go to Chat
                    </Button>
                    <Button variant="ghost" size="sm" className="text-xs" onClick={() => onSettings(character.character_id)}>
                      <Settings size={12} className="mr-1.5" /> Settings
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Group Chats */}
          {store.groupChats.length > 0 && (
            <>
              <h2 className="text-lg font-semibold mt-8">Group Chats</h2>
              <div className="grid grid-cols-2 gap-4">
                {store.groupChats.map((gc) => {
                  const charIds: string[] = Array.isArray(gc.character_ids) ? gc.character_ids : [];
                  const chars = charIds.map((id) => store.characters.find((c) => c.character_id === id)).filter(Boolean) as Character[];
                  return (
                    <div key={gc.group_chat_id} className="rounded-xl border border-border bg-card/60 p-5 flex gap-5">
                      <div className="flex -space-x-3 flex-shrink-0 pt-1">
                        {chars.map((ch, i) => {
                          const p = store.activePortraits[ch.character_id];
                          return p?.data_url ? (
                            <img key={ch.character_id} src={p.data_url} alt="" className="w-12 h-12 rounded-full object-cover ring-2 ring-card" style={{ zIndex: chars.length - i }} />
                          ) : (
                            <div key={ch.character_id} className="w-12 h-12 rounded-full ring-2 ring-card flex items-center justify-center text-sm font-bold text-white" style={{ backgroundColor: ch.avatar_color || "#6366f1", zIndex: chars.length - i }}>
                              {ch.display_name?.charAt(0)}
                            </div>
                          );
                        })}
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="font-semibold text-foreground">{chars.map((c) => c.display_name).join(" & ")}</h3>
                        <p className="text-xs text-muted-foreground/50 mt-0.5">Group Chat</p>
                      </div>
                      <div className="flex flex-col gap-1.5 flex-shrink-0 pt-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          className="text-xs"
                          disabled={(groupDialogueCounts[gc.group_chat_id] ?? 0) === 0}
                          onClick={() => setSummaryTarget({ type: "group", id: gc.group_chat_id, name: chars.map((c) => c.display_name).join(" & ") })}
                        >
                          <BookOpen size={12} className="mr-1.5" /> Summary
                        </Button>
                      </div>
                    </div>
                  );
                })}
              </div>
            </>
          )}
        </div>
      </ScrollArea>

      <SummaryModal
        open={!!summaryTarget}
        onClose={() => setSummaryTarget(null)}
        title={summaryTarget ? `Summary: ${summaryTarget.name}` : ""}
        generateSummary={async (mode) => {
          if (!summaryTarget || !store.apiKey) return "No API key configured.";
          if (summaryTarget.type === "char") {
            return api.generateChatSummary(store.apiKey, summaryTarget.id, mode);
          } else {
            return api.generateGroupChatSummary(store.apiKey, summaryTarget.id, mode);
          }
        }}
      />

      {/* Carousel Modal */}
      {showCarouselModal && (() => {
        const selected = illustrations.find((i) => i.id === selectedId);
        return (
          <div className="fixed inset-0 z-50 bg-black flex flex-col">
            {/* Close button */}
            <button
              onClick={() => { setShowCarouselModal(false); if (slideshow.active) slideshow.toggle(); }}
              className="absolute top-4 right-4 z-50 w-10 h-10 rounded-full bg-white/10 text-white flex items-center justify-center cursor-pointer hover:bg-white/20 transition-colors backdrop-blur-sm"
            >
              <X size={18} />
            </button>
            {/* Main viewer */}
            <div className="flex-1 relative flex items-center justify-center overflow-hidden group/viewer">
              {playingVideo && selectedId && videoDataUrls[selectedId] ? (
                <video
                  src={videoDataUrls[selectedId]}
                  autoPlay
                  loop={!slideshow.active}
                  muted={videoMuted}
                  playsInline
                  className="max-w-full max-h-full object-contain"
                  onTimeUpdate={slideshow.active ? (e) => {
                    const v = e.currentTarget;
                    slideshow.onVideoTimeUpdate(v.currentTime, v.duration);
                  } : undefined}
                  onEnded={slideshow.active ? slideshow.onVideoEnded : undefined}
                />
              ) : selected ? (
                <img src={selected.data_url} alt="Illustration" className="max-w-full max-h-full object-contain" />
              ) : null}
              {/* Slideshow toggle */}
              {illustrations.length > 1 && (
                <button
                  onClick={() => slideshow.toggle()}
                  className="absolute top-4 left-4 z-20 flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-black/50 text-white text-[10px] font-medium cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm opacity-0 group-hover/viewer:opacity-100"
                >
                  {slideshow.active ? <Pause size={10} /> : <Play size={10} />}
                  Slideshow
                </button>
              )}
              {/* Nav arrows */}
              {illustrations.length > 1 && (
                <>
                  <button
                    onClick={() => {
                      if (slideshow.active) {
                        const idx = slideshow.slides.findIndex((s) => s.illustrationId === selectedId && s.type === (playingVideo ? "video" : "image"));
                        const prev = idx <= 0 ? slideshow.slides.length - 1 : idx - 1;
                        const target = slideshow.slides[prev];
                        if (target) slideshow.jumpTo(target.illustrationId);
                      } else {
                        const idx = illustrations.findIndex((i) => i.id === selectedId);
                        const prev = idx <= 0 ? illustrations.length - 1 : idx - 1;
                        navigateTo(illustrations[prev].id);
                      }
                    }}
                    className="absolute left-3 top-1/2 -translate-y-1/2 z-20 w-12 h-12 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-all backdrop-blur-sm opacity-0 group-hover/viewer:opacity-100"
                  >
                    <ChevronLeft size={24} />
                  </button>
                  <button
                    onClick={() => {
                      if (slideshow.active) {
                        const idx = slideshow.slides.findIndex((s) => s.illustrationId === selectedId && s.type === (playingVideo ? "video" : "image"));
                        const next = idx >= slideshow.slides.length - 1 ? 0 : idx + 1;
                        const target = slideshow.slides[next];
                        if (target) slideshow.jumpTo(target.illustrationId);
                      } else {
                        const idx = illustrations.findIndex((i) => i.id === selectedId);
                        const next = idx >= illustrations.length - 1 ? 0 : idx + 1;
                        navigateTo(illustrations[next].id);
                      }
                    }}
                    className="absolute right-3 top-1/2 -translate-y-1/2 z-20 w-12 h-12 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-all backdrop-blur-sm opacity-0 group-hover/viewer:opacity-100"
                  >
                    <ChevronRight size={24} />
                  </button>
                </>
              )}
              {/* Video controls */}
              {playingVideo && (
                <div className="absolute bottom-4 right-4 z-20 flex gap-2 opacity-0 group-hover/viewer:opacity-100 transition-opacity">
                  <button
                    onClick={() => setVideoMuted((m) => !m)}
                    className="w-10 h-10 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-black/90 transition-colors backdrop-blur-sm"
                  >
                    {videoMuted ? <VolumeX size={16} /> : <Volume2 size={16} />}
                  </button>
                  {!slideshow.active && (
                    <button
                      onClick={() => setPlayingVideo(false)}
                      className="w-10 h-10 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm"
                    >
                      <Square size={14} fill="white" />
                    </button>
                  )}
                </div>
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
              <div className="flex-shrink-0 border-t border-white/10 bg-black/80 px-3 py-2">
                <div className="flex gap-1.5 overflow-x-auto scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]">
                  {illustrations.map((illus) => (
                    <button
                      key={illus.id}
                      ref={illus.id === selectedId ? (el) => {
                        if (!el) return;
                        const c = el.parentElement;
                        if (c) c.scrollTo({ left: el.offsetLeft - c.offsetWidth / 2 + el.offsetWidth / 2, behavior: "smooth" });
                      } : undefined}
                      onClick={() => {
                        if (slideshow.active) slideshow.jumpTo(illus.id);
                        else navigateTo(illus.id);
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
          </div>
        );
      })()}

      <PortraitModal
        characterId={portraitCharId}
        characterName={portraitCharId ? store.characters.find((c) => c.character_id === portraitCharId)?.display_name : undefined}
        onClose={() => setPortraitCharId(null)}
      />
    </div>
  );
}

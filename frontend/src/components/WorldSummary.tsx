import { useState, useEffect, useCallback, useRef } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { RefreshCw, MessageSquare, Settings, Loader2, User } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type Character, type PortraitInfo, type GalleryItem } from "@/lib/tauri";

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

const SLIDE_DURATION = 5000;
const FADE_DURATION = 1000;

export function WorldSummary({ store, onChat, onSettings }: Props) {
  const world = store.activeWorld;
  const [summaries, setSummaries] = useState<CharSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [regenerating, setRegenerating] = useState(false);
  const [userAvatarUrl, setUserAvatarUrl] = useState("");
  const [heroImages, setHeroImages] = useState<string[]>([]);
  const [activeLayer, setActiveLayer] = useState(0);
  const [layerSrcs, setLayerSrcs] = useState<[string, string]>(["", ""]);
  const [layerOpacities, setLayerOpacities] = useState<[number, number]>([1, 0]);
  const slideIndexRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setInterval>>();

  useEffect(() => {
    if (!world) return;
    api.getUserAvatar(world.world_id).then((url) => setUserAvatarUrl(url || "")).catch(() => {});
  }, [world?.world_id, store.userProfile?.avatar_file]);

  useEffect(() => {
    if (!world) return;
    api.listWorldGallery(world.world_id).then((items: GalleryItem[]) => {
      const urls = items
        .filter((i) => i.category === "world" && !i.is_archived && i.data_url)
        .map((i) => i.data_url);
      setHeroImages(urls);
      slideIndexRef.current = 0;
      if (urls.length > 0) {
        setLayerSrcs([urls[0], ""]);
        setLayerOpacities([1, 0]);
        setActiveLayer(0);
      }
    }).catch(() => {});
  }, [world?.world_id, store.activeWorldImage?.image_id]);

  useEffect(() => {
    if (heroImages.length <= 1) return;
    timerRef.current = setInterval(() => {
      const nextIdx = (slideIndexRef.current + 1) % heroImages.length;
      slideIndexRef.current = nextIdx;
      setActiveLayer((prev) => {
        const next = prev === 0 ? 1 : 0;
        setLayerSrcs((srcs) => {
          const updated: [string, string] = [...srcs] as [string, string];
          updated[next] = heroImages[nextIdx];
          return updated;
        });
        setLayerOpacities(next === 0 ? [1, 0] : [0, 1]);
        return next;
      });
    }, SLIDE_DURATION);
    return () => clearInterval(timerRef.current);
  }, [heroImages]);

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
        {/* Hero slideshow */}
        <div className="relative w-full aspect-[2/1] min-h-[280px] max-h-[480px] overflow-hidden bg-card">
          {heroImages.length > 0 ? (
            <>
              {([0, 1] as const).map((layer) => (
                layerSrcs[layer] && (
                  <img
                    key={`layer-${layer}-${layerSrcs[layer]}`}
                    src={layerSrcs[layer]}
                    alt=""
                    className="absolute inset-0 w-full h-full object-cover"
                    style={{
                      opacity: layerOpacities[layer],
                      transform: activeLayer === layer ? "scale(1.08)" : "scale(1)",
                      transition: `opacity ${FADE_DURATION}ms ease-in-out, transform ${SLIDE_DURATION}ms ease`,
                    }}
                  />
                )
              ))}
            </>
          ) : (
            <div className="absolute inset-0 bg-gradient-to-br from-primary/20 via-background to-primary/10" />
          )}
          <div className="absolute inset-0 bg-gradient-to-t from-background via-background/40 to-transparent" />
          <div className="absolute inset-x-0 bottom-0 p-8">
            <h1 className="text-4xl font-bold text-foreground drop-shadow-lg">
              {world.name}
            </h1>
            {world.description && (
              <p className="text-sm text-foreground/70 mt-2 max-w-xl line-clamp-2 drop-shadow">
                {world.description}
              </p>
            )}
            {heroImages.length > 1 && (
              <div className="flex gap-1.5 mt-3">
                {heroImages.map((_, i) => (
                  <div
                    key={i}
                    className={`h-1 rounded-full transition-all duration-500 ${
                      i === slideIndexRef.current ? "w-6 bg-foreground/70" : "w-1.5 bg-foreground/25"
                    }`}
                  />
                ))}
              </div>
            )}
          </div>
        </div>

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

import { useState, useEffect, useCallback, useRef, startTransition, type ReactNode } from "react";
import { useAppStore } from "@/hooks/use-app-store";
import { api, type DailyUsage } from "@/lib/tauri";
import { Sidebar } from "@/components/Sidebar";
import { ChatView } from "@/components/ChatView";
import { WorldCanonEditor } from "@/components/WorldCanonEditor";
import { CharacterEditor } from "@/components/CharacterEditor";
import { CharacterGrid } from "@/components/CharacterGrid";
import { UserProfileEditor } from "@/components/UserProfileEditor";
import { SettingsPanel } from "@/components/SettingsPanel";
import { WorldSummary } from "@/components/WorldSummary";
import { Gallery } from "@/components/Gallery";
import { MoodDebugPanel } from "@/components/MoodDebugPanel";
import { PortraitPopout } from "@/components/PortraitPopout";
import { MessageSquare, PenLine, Users, Settings, Coins, Image, BookOpen } from "lucide-react";

type View = "chat" | "world" | "character" | "settings" | "summary" | "gallery";
type CharSubView = "grid" | "editor" | "profile";

export default function App() {
  // Check if this window is a popout
  const params = new URLSearchParams(window.location.search);
  const popoutCharacterId = params.get("portrait");
  if (popoutCharacterId) {
    return <PortraitPopout characterId={popoutCharacterId} />;
  }
  const illustrationMsgId = params.get("illustration");
  if (illustrationMsgId) {
    return <IllustrationPopout messageId={illustrationMsgId} />;
  }

  return <MainApp />;
}

function MainApp() {
  const store = useAppStore();
  const [view, setView] = useState<View>("chat");
  const [charSubView, setCharSubView] = useState<CharSubView>("grid");
  const lastChatCharRef = useRef<string | null>(null);
  const viewRef = useRef<View>("chat");

  const setViewTracked = useCallback((next: View) => {
    const prev = viewRef.current;
    if (prev === "chat" && next !== "chat" && store.activeCharacter) {
      lastChatCharRef.current = store.activeCharacter.character_id;
    }
    viewRef.current = next;
    startTransition(() => setView(next));
  }, [store.activeCharacter]);

  const handleNavigate = useCallback((v: string) => {
    setViewTracked(v as View);
    if (v === "character" && !store.editingUserProfile) {
      setCharSubView("editor");
    }
  }, [store.editingUserProfile, setViewTracked]);

  const handleCharNav = useCallback(() => {
    setViewTracked("character");
    setCharSubView("grid");
  }, [setViewTracked]);

  const handleChatNav = useCallback(() => {
    if (lastChatCharRef.current) {
      const ch = store.characters.find((c) => c.character_id === lastChatCharRef.current);
      if (ch) {
        store.selectCharacter(ch);
      }
    }
    setViewTracked("chat");
  }, [store, setViewTracked]);

  if (store.loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-background">
        <div className="text-center">
          <div className="animate-pulse text-primary text-4xl mb-4">✦</div>
          <p className="text-muted-foreground">Loading WorldThreads...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      {store.error && (
        <div className="fixed top-4 right-4 z-50 bg-destructive text-destructive-foreground px-4 py-2 rounded-lg shadow-lg max-w-md text-sm">
          {store.error}
        </div>
      )}

      <div className="w-16 flex-shrink-0 bg-card border-r border-border flex flex-col items-center py-4 gap-2">
        <NavButton icon={<BookOpen size={20} />} active={view === "summary"} onClick={() => setViewTracked("summary")} title="Summary" description="World overview and conversation recaps for each character." />
        <NavButton icon={<MessageSquare size={20} />} active={view === "chat"} onClick={handleChatNav} title="Chat" description="Talk with your characters in real time." />
        <NavButton icon={<PenLine size={20} />} active={view === "world"} onClick={() => setViewTracked("world")} title="World Canon" description="Edit your world's name, description, tone, and rules." />
        <NavButton icon={<Users size={20} />} active={view === "character"} onClick={handleCharNav} title="Characters" description="Create, edit, and manage your cast of characters." />
        <NavButton icon={<Image size={20} />} active={view === "gallery"} onClick={() => setViewTracked("gallery")} title="Gallery" description="Browse, generate, and upload images for this world." />
        <div className="flex-1" />
        <UsageBadge sending={store.sending} />
        <NavButton icon={<Settings size={20} />} active={view === "settings"} onClick={() => setViewTracked("settings")} title="Settings" description="API key, model config, and app preferences." />
      </div>

      <Sidebar store={store} onNavigate={handleNavigate} />

      <main className="flex-1 flex flex-col min-w-0">
        {!store.apiKey && view === "chat" && (
          <div className="m-4 p-4 bg-primary/10 border border-primary/20 rounded-lg text-sm">
            <p className="font-medium text-primary">API key required</p>
            <p className="text-muted-foreground mt-1">
              Go to <button className="underline text-primary cursor-pointer" onClick={() => setViewTracked("settings")}>Settings</button> to add your OpenAI API key before chatting.
              {" "}Already added one?{" "}
              <button
                className="underline text-primary cursor-pointer"
                onClick={async () => {
                  const key = await api.getApiKey();
                  if (key) store.setApiKey(key);
                }}
              >
                Try again
              </button>
            </p>
          </div>
        )}
        {view === "chat" && (
          <DeferredMount key="chat">
            <ChatView store={store} />
          </DeferredMount>
        )}
        {view === "world" && (
          <DeferredMount key="world">
            <WorldCanonEditor store={store} />
          </DeferredMount>
        )}
        {view === "character" && (
          <DeferredMount key="character">
            {store.editingUserProfile ? <UserProfileEditor store={store} /> :
            charSubView === "grid" ? (
              <CharacterGrid
                store={store}
                onChat={(id) => {
                  const ch = store.characters.find((c) => c.character_id === id);
                  if (ch) { store.selectCharacter(ch); lastChatCharRef.current = id; setViewTracked("chat"); }
                }}
                onSettings={(id) => {
                  const ch = store.characters.find((c) => c.character_id === id);
                  if (ch) { store.selectCharacter(ch); setCharSubView("editor"); }
                }}
              />
            ) : <CharacterEditor store={store} />}
          </DeferredMount>
        )}
        {view === "gallery" && (
          <DeferredMount key="gallery">
            <Gallery store={store} />
          </DeferredMount>
        )}
        {view === "settings" && (
          <DeferredMount key="settings">
            <SettingsPanel store={store} />
          </DeferredMount>
        )}
        {view === "summary" && (
          <DeferredMount key="summary">
            <WorldSummary
              store={store}
              onChat={(id) => {
                const ch = store.characters.find((c) => c.character_id === id);
                if (ch) { store.selectCharacter(ch); lastChatCharRef.current = id; setViewTracked("chat"); }
              }}
              onSettings={(id) => {
                const ch = store.characters.find((c) => c.character_id === id);
                if (ch) { store.selectCharacter(ch); setViewTracked("character"); setCharSubView("editor"); }
              }}
            />
          </DeferredMount>
        )}
      </main>

      <MoodDebugPanel characterId={store.activeCharacter?.character_id} />
    </div>
  );
}

function DeferredMount({ children }: { children: ReactNode }) {
  const [ready, setReady] = useState(false);
  useEffect(() => {
    const id = requestAnimationFrame(() => setReady(true));
    return () => cancelAnimationFrame(id);
  }, []);
  if (!ready) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="animate-pulse text-primary text-2xl">✦</div>
      </div>
    );
  }
  return <>{children}</>;
}

function NavButton({ icon, active, onClick, title, description }: { icon: React.ReactNode; active: boolean; onClick: () => void; title: string; description?: string }) {
  const [hovering, setHovering] = useState(false);
  return (
    <div
      className="relative"
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
    >
      <button
        onClick={onClick}
        className={`w-10 h-10 rounded-lg flex items-center justify-center transition-colors cursor-pointer ${
          active ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground hover:bg-accent"
        }`}
      >
        {icon}
      </button>
      {hovering && (
        <div className="absolute left-12 top-1/2 -translate-y-1/2 z-50 w-48 bg-card border border-border rounded-lg shadow-xl shadow-black/30 px-3 py-2.5 pointer-events-none animate-in fade-in zoom-in-95 duration-100">
          <p className="text-xs font-semibold text-foreground">{title}</p>
          {description && <p className="text-[11px] text-muted-foreground leading-snug mt-0.5">{description}</p>}
        </div>
      )}
    </div>
  );
}

function IllustrationPopout({ messageId }: { messageId: string }) {
  const [dataUrl, setDataUrl] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // The illustration content is stored as the message content (a data URL).
    // We need to find it via the gallery since we can't query a single message by ID directly.
    // But the world_images table has the file linked by image_id = message_id.
    // Simplest: use getSetting to... actually, let's just look it up from the gallery.
    // The image_id in world_images matches the message_id.
    // We don't have a direct "get world image by id" command, but we can list all and find it.
    // Actually simpler: the message content IS the data URL. We just need to get the message.
    // But we don't have a get-single-message command. Let me use the gallery approach.
    // The gallery items have data_url populated. Let's list all worlds and search.
    (async () => {
      try {
        const worlds = await api.listWorlds();
        for (const w of worlds) {
          const gallery = await api.listWorldGallery(w.world_id);
          const match = gallery.find((g) => g.id === messageId);
          if (match?.data_url) {
            setDataUrl(match.data_url);
            break;
          }
        }
      } catch {
        // ignore
      } finally {
        setLoading(false);
      }
    })();
  }, [messageId]);

  if (loading) {
    return (
      <div className="h-screen bg-background flex items-center justify-center">
        <div className="animate-pulse text-primary text-2xl">...</div>
      </div>
    );
  }

  if (!dataUrl) {
    return (
      <div className="h-screen bg-background flex items-center justify-center text-muted-foreground text-sm">
        Illustration not found
      </div>
    );
  }

  return (
    <div className="h-screen bg-black flex flex-col overflow-hidden">
      <div
        data-tauri-drag-region
        className="h-8 flex-shrink-0 flex items-center pl-[72px] pr-3 bg-card border-b border-border select-none"
      >
        <span className="text-xs text-muted-foreground">Illustration</span>
      </div>
      <div className="flex-1 overflow-auto min-h-0">
        <img
          src={dataUrl}
          alt="Illustration"
          className="w-full"
        />
      </div>
    </div>
  );
}

function estimateCost(usage: DailyUsage): { input: string; output: string; total: string } {
  // Pricing per 1M tokens (approx, blended for gpt-4o / gpt-4o-mini / embeddings)
  // gpt-4o: $2.50 in / $10 out, gpt-4o-mini: $0.15 in / $0.60 out
  // Using a weighted average since we mix models: ~$2 in / $6 out per 1M
  const inputCostPer1M = 2.0;
  const outputCostPer1M = 6.0;
  const inputCost = (usage.prompt_tokens / 1_000_000) * inputCostPer1M;
  const outputCost = (usage.completion_tokens / 1_000_000) * outputCostPer1M;
  const total = inputCost + outputCost;
  return {
    input: inputCost < 0.01 ? "<$0.01" : `$${inputCost.toFixed(2)}`,
    output: outputCost < 0.01 ? "<$0.01" : `$${outputCost.toFixed(2)}`,
    total: total < 0.01 ? "<$0.01" : `$${total.toFixed(2)}`,
  };
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

function UsageBadge({ sending }: { sending: boolean }) {
  const [usage, setUsage] = useState<DailyUsage | null>(null);
  const [hovering, setHovering] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setUsage(await api.getTodayUsage());
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  // Refresh after each send completes
  useEffect(() => {
    if (!sending) refresh();
  }, [sending, refresh]);

  const today = new Date().toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric" });
  const cost = usage ? estimateCost(usage) : null;
  const totalTokens = usage ? usage.prompt_tokens + usage.completion_tokens : 0;

  return (
    <div
      className="relative"
      onMouseEnter={() => setHovering(true)}
      onMouseLeave={() => setHovering(false)}
    >
      <div className="w-10 h-10 rounded-lg flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-default">
        <Coins size={20} />
      </div>

      {hovering && usage && cost && (
        <div className="absolute left-12 bottom-0 z-50 w-56 bg-card border border-border rounded-xl shadow-2xl shadow-black/40 p-4 animate-in fade-in zoom-in-95 duration-150">
          <div className="flex items-center justify-between mb-3">
            <span className="text-xs font-semibold text-primary">Today's Usage</span>
            <span className="text-[10px] text-muted-foreground">{today}</span>
          </div>

          <div className="space-y-2 text-xs">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Tokens in</span>
              <span className="font-mono">{formatTokens(usage.prompt_tokens)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Tokens out</span>
              <span className="font-mono">{formatTokens(usage.completion_tokens)}</span>
            </div>

            <div className="border-t border-border pt-2 mt-2 space-y-1.5">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Input cost</span>
                <span className="font-mono">{cost.input}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Output cost</span>
                <span className="font-mono">{cost.output}</span>
              </div>
              <div className="flex justify-between font-medium pt-1 border-t border-border">
                <span>Est. total</span>
                <span className="text-primary font-mono">{cost.total}</span>
              </div>
            </div>
          </div>

          {totalTokens === 0 && (
            <p className="text-[10px] text-muted-foreground/60 mt-2 text-center">No API calls yet today</p>
          )}

          <p className="text-[10px] text-muted-foreground/50 leading-relaxed mt-3 pt-2 border-t border-border/50">
            Includes: dialogue replies, world ticks, emoji reactions, memory summaries, embeddings, image generation, and any other API calls for new features.
          </p>
        </div>
      )}
    </div>
  );
}

import { useState, useRef, useEffect, useCallback } from "react";
import Markdown from "react-markdown";
import { Dialog } from "@/components/ui/dialog";
import { X, Loader2, Sparkles, Trash2, Plus, PanelLeftClose, PanelLeftOpen, Image as ImageIcon, ScrollText } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { api, type ImaginedChapter, type ImaginedChapterStageEvent, type ImaginedChapterImageEvent, type ImaginedChapterDoneEvent } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { chatFontPx, CHAT_FONT_SIZES_PX } from "@/lib/chat-font";
import { FontSizeAdjuster } from "@/components/chat/FontSizeAdjuster";
import { remarkPlugins, rehypePlugins, markdownComponents } from "./formatMessage";
import { playChime } from "@/lib/chime";

const CHAPTER_FONT_SIZE_KEY = "imagined_chapter.font_size";

interface Props {
  open: boolean;
  onClose: () => void;
  apiKey: string;
  threadId: string;
  /** Data URLs for the chat's character portraits — shown stacked on the
   *  compose view's hero. Solo chat = 1, group chat = N. */
  characterPortraitUrls?: string[];
  /** Active world image data URL — painted behind the portraits as an
   *  edge-to-edge banner that fades into the parchment. Optional. */
  worldImageUrl?: string;
  /** Whether to play a chime on first token of writing phase. */
  notifyOnMessage: boolean;
  /** Chat font size — used as the initial default for the chapter view's
   *  font-size adjuster (overridable in-modal and persisted in
   *  localStorage). 0..CHAT_FONT_SIZES_PX.length-1, default 2. */
  chatFontSize: number;
  /** If provided, the modal opens directly to this chapter instead of the compose view. */
  openChapterId?: string | null;
  /** Bubble up "canonize this chapter" — parent opens its KeepRecordModal
   *  with the breadcrumb message_id as the source. */
  onCanonize?: (breadcrumbMessageId: string, chapterTitle: string) => void;
}

export function ImaginedChapterModal({
  open,
  onClose,
  apiKey,
  threadId,
  characterPortraitUrls,
  worldImageUrl,
  notifyOnMessage,
  chatFontSize,
  openChapterId,
}: Props) {
  const [chapters, setChapters] = useState<ImaginedChapter[]>([]);
  const [activeChapterId, setActiveChapterId] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  // Compose state for a NEW chapter
  const [seedHint, setSeedHint] = useState("");
  const [continueFromPrevious, setContinueFromPrevious] = useState(false);
  const [imageTier, setImageTier] = useState<"low" | "medium" | "high">("medium");
  // Profundity dial — Glimpse / Opening / Deep / Sacred. Default
  // "Opening" — the natural register for chapters that want to mean
  // something without being seismic. Sacred used rarely.
  const [depth, setDepth] = useState<"Glimpse" | "Opening" | "Deep" | "Sacred">("Opening");

  // Streaming generation state
  const [phase, setPhase] = useState<"idle" | "inventing" | "rendering" | "writing" | "done">("idle");
  const [streamTitle, setStreamTitle] = useState<string>("");
  const [streamImage, setStreamImage] = useState<string>(""); // data URL
  const [streamContent, setStreamContent] = useState<string>("");
  const [streamChapterId, setStreamChapterId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Active chapter data when viewing a saved one (not generating)
  const [activeChapterData, setActiveChapterData] = useState<ImaginedChapter | null>(null);
  const [activeImageUrl, setActiveImageUrl] = useState<string>("");

  // Per-session set of chapter_ids the user has canonized in this open
  // of the modal. Used to update the inline indicator on the just-streamed
  // chapter (which lives in stream state, not in activeChapterData).
  const [canonizedThisSession, setCanonizedThisSession] = useState<Set<string>>(new Set());
  // When set, the user is closing the modal but there's a non-canonized
  // chapter they were viewing — show the prompt before honoring close.
  const [confirmingClose, setConfirmingClose] = useState<{ chapterId: string; title: string } | null>(null);
  // True when a canonize operation is in flight.
  const [canonizing, setCanonizing] = useState(false);

  const scrollRef = useRef<HTMLDivElement | null>(null);
  const generatingRef = useRef(false);
  // Track whether the user has been chimed for this generation's
  // image-arrival and first-token moments. Reset on each fresh
  // handleGenerate so a second chapter in the same session chimes
  // afresh.
  const imageChimedRef = useRef(false);
  const firstTokenChimedRef = useRef(false);

  // Chapter-local font size — defaults to chatFontSize on first open,
  // then sticks via localStorage so the reading size carries across modal
  // sessions independent of chat. Clamped to the chat-font ladder.
  const [chapterFontSize, setChapterFontSize] = useState<number>(() => {
    try {
      const stored = localStorage.getItem(CHAPTER_FONT_SIZE_KEY);
      if (stored !== null) {
        const n = parseInt(stored, 10);
        if (!Number.isNaN(n)) return Math.max(0, Math.min(CHAT_FONT_SIZES_PX.length - 1, n));
      }
    } catch { /* ignore — fall through to default */ }
    return chatFontSize;
  });
  function persistChapterFontSize(n: number) {
    setChapterFontSize(n);
    try { localStorage.setItem(CHAPTER_FONT_SIZE_KEY, String(n)); } catch { /* private mode etc. */ }
  }

  const loadChapters = useCallback(async () => {
    try {
      const list = await api.listImaginedChaptersForThread(threadId);
      setChapters(list);
    } catch (e) {
      console.error("Failed to load chapters:", e);
    }
  }, [threadId]);

  useEffect(() => {
    if (open) {
      loadChapters();
    }
  }, [open, loadChapters]);

  // Reset compose state when modal closes
  useEffect(() => {
    if (!open) {
      setSeedHint("");
      setContinueFromPrevious(false);
      setActiveChapterId(null);
      setActiveChapterData(null);
      setActiveImageUrl("");
      if (!generatingRef.current) {
        setPhase("idle");
        setStreamTitle("");
        setStreamImage("");
        setStreamContent("");
        setStreamChapterId(null);
      }
      setError(null);
      setCanonizedThisSession(new Set());
      setConfirmingClose(null);
      setCanonizing(false);
    }
  }, [open]);

  // Identify the chapter currently being viewed if it's NOT canonized.
  // The close-prompt fires on it. Two viewing surfaces to consider:
  //   1. The just-streamed chapter (phase === "done"/"writing", id in
  //      streamChapterId) — assume not canonized unless it's in the
  //      session's canonized set.
  //   2. A saved chapter loaded from the sidebar (activeChapterData) —
  //      check its persisted .canonized flag.
  const pendingNonCanonizedChapter: { chapterId: string; title: string } | null = (() => {
    // Prefer the active sidebar selection if there is one (covers the
    // case where the user has clicked into a saved chapter).
    if (activeChapterData) {
      if (!activeChapterData.canonized && !canonizedThisSession.has(activeChapterData.chapter_id)) {
        return { chapterId: activeChapterData.chapter_id, title: activeChapterData.title };
      }
      return null;
    }
    // Otherwise the just-streamed chapter, if any.
    if (streamChapterId && (phase === "done" || phase === "writing")) {
      if (!canonizedThisSession.has(streamChapterId)) {
        return { chapterId: streamChapterId, title: streamTitle };
      }
    }
    return null;
  })();

  function handleCloseAttempt() {
    if (pendingNonCanonizedChapter && !canonizing) {
      setConfirmingClose(pendingNonCanonizedChapter);
      return;
    }
    onClose();
  }

  async function canonizeChapter(chapterId: string): Promise<void> {
    if (canonizing) return;
    setCanonizing(true);
    setError(null);
    try {
      await api.canonizeImaginedChapter(chapterId);
      setCanonizedThisSession((prev) => {
        const next = new Set(prev);
        next.add(chapterId);
        return next;
      });
      if (activeChapterData?.chapter_id === chapterId) {
        try {
          const fresh = await api.getImaginedChapter(chapterId);
          setActiveChapterData(fresh);
        } catch { /* non-fatal */ }
      }
      loadChapters();
      // Tell the underlying chat view to reload its messages — the
      // backend just inserted a breadcrumb row and the chat's visible
      // history won't reflect it until we nudge it. Event-based so
      // both ChatView and GroupChatView can listen without the modal
      // needing to know which variant it's embedded in.
      if (threadId) {
        window.dispatchEvent(new CustomEvent("imagined-chapter-canonized", {
          detail: { threadId },
        }));
      }
    } catch (e) {
      setError(String(e));
      throw e;
    } finally {
      setCanonizing(false);
    }
  }

  async function decanonizeChapter(chapterId: string): Promise<void> {
    if (canonizing) return;
    setCanonizing(true);
    setError(null);
    try {
      await api.decanonizeImaginedChapter(chapterId);
      setCanonizedThisSession((prev) => {
        const next = new Set(prev);
        next.delete(chapterId);
        return next;
      });
      if (activeChapterData?.chapter_id === chapterId) {
        try {
          const fresh = await api.getImaginedChapter(chapterId);
          setActiveChapterData(fresh);
        } catch { /* non-fatal */ }
      }
      loadChapters();
    } catch (e) {
      setError(String(e));
    } finally {
      setCanonizing(false);
    }
  }

  // Bulk reset — for the user to clean up after the migration auto-
  // canonized prior chapters that they didn't actually choose. Requires
  // explicit confirm because it removes chat-history breadcrumbs.
  const [bulkResetConfirm, setBulkResetConfirm] = useState(false);
  async function bulkResetCanonization(): Promise<void> {
    if (canonizing) return;
    setCanonizing(true);
    setError(null);
    try {
      await api.bulkDecanonizeImaginedChaptersForThread(threadId);
      setCanonizedThisSession(new Set());
      if (activeChapterData) {
        try {
          const fresh = await api.getImaginedChapter(activeChapterData.chapter_id);
          setActiveChapterData(fresh);
        } catch { /* non-fatal */ }
      }
      loadChapters();
      setBulkResetConfirm(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setCanonizing(false);
    }
  }

  // Wire up Tauri stream events
  useEffect(() => {
    if (!open) return;
    const unlisteners: Array<() => void> = [];

    listen<ImaginedChapterStageEvent>("imagined-chapter-stage", (e) => {
      if (!generatingRef.current) return;
      setPhase(e.payload.phase);
      if (e.payload.title) setStreamTitle(e.payload.title);
    }).then((u) => unlisteners.push(u));

    listen<ImaginedChapterImageEvent>("imagined-chapter-image", (e) => {
      if (!generatingRef.current) return;
      setStreamImage(e.payload.dataUrl);
      // Chime once when the rendered image lands — the user can look
      // away while it paints and this pulls their eye back to the modal.
      if (notifyOnMessage && !imageChimedRef.current) {
        imageChimedRef.current = true;
        playChime();
      }
    }).then((u) => unlisteners.push(u));

    listen<string>("imagined-chapter-token", (e) => {
      if (!generatingRef.current) return;
      setStreamContent((prev) => prev + (e.payload || ""));
      // Chime once when the first narrative token arrives — the shift
      // from image-rendering to chapter-writing is the other moment
      // the user was waiting on.
      if (notifyOnMessage && !firstTokenChimedRef.current && (e.payload || "").length > 0) {
        firstTokenChimedRef.current = true;
        playChime();
      }
    }).then((u) => unlisteners.push(u));

    listen<ImaginedChapterDoneEvent>("imagined-chapter-done", (e) => {
      if (!generatingRef.current) return;
      setPhase("done");
      setStreamTitle(e.payload.title);
      setStreamContent(e.payload.content);
      generatingRef.current = false;
      loadChapters(); // refresh sidebar
    }).then((u) => unlisteners.push(u));

    return () => { unlisteners.forEach((u) => u()); };
  }, [open, loadChapters, notifyOnMessage]);

  // No auto-scroll during streaming — the user wants full control over
  // their scroll position while reading. The chapter streams in; the
  // viewport stays where the reader put it.

  async function handleGenerate() {
    if (generatingRef.current) return;
    setError(null);
    setPhase("inventing");
    setStreamTitle("");
    setStreamImage("");
    setStreamContent("");
    setStreamChapterId(null);
    setActiveChapterId(null);
    imageChimedRef.current = false;
    firstTokenChimedRef.current = false;
    setActiveChapterData(null);
    generatingRef.current = true;
    try {
      const res = await api.generateImaginedChapter(apiKey, {
        threadId,
        seedHint: seedHint.trim() || undefined,
        continueFromPrevious,
        imageTier,
        depth,
      });
      setStreamChapterId(res.chapterId);
    } catch (e) {
      setError(String(e));
      setPhase("idle");
      generatingRef.current = false;
    }
  }

  async function handleSelectChapter(c: ImaginedChapter) {
    if (generatingRef.current) return;
    setActiveChapterId(c.chapter_id);
    setPhase("idle");
    setStreamContent("");
    setStreamImage("");
    setStreamTitle("");
    setStreamChapterId(null);
    try {
      const fresh = await api.getImaginedChapter(c.chapter_id);
      setActiveChapterData(fresh);
      if (fresh.image_id) {
        try {
          const url = await api.getImaginedChapterImageUrl(c.chapter_id);
          setActiveImageUrl(url);
        } catch {
          setActiveImageUrl("");
        }
      } else {
        setActiveImageUrl("");
      }
    } catch (e) {
      setError(String(e));
    }
  }

  // Auto-open the requested chapter when the modal opens with openChapterId set.
  useEffect(() => {
    if (!open || !openChapterId || generatingRef.current) return;
    if (activeChapterId === openChapterId) return;
    // Need the chapter row to call handleSelect; fetch by id.
    api.getImaginedChapter(openChapterId).then((c) => {
      handleSelectChapter(c);
    }).catch(() => {});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, openChapterId]);

  async function handleDelete(chapterId: string, e: React.MouseEvent) {
    e.stopPropagation();
    if (!confirm("Delete this chapter? This can't be undone.")) return;
    try {
      await api.deleteImaginedChapter(chapterId);
      if (activeChapterId === chapterId) {
        setActiveChapterId(null);
        setActiveChapterData(null);
      }
      loadChapters();
    } catch (err) {
      setError(String(err));
    }
  }

  function handleNewChapter() {
    if (generatingRef.current) return;
    setActiveChapterId(null);
    setActiveChapterData(null);
    setPhase("idle");
    setStreamTitle("");
    setStreamImage("");
    setStreamContent("");
    setStreamChapterId(null);
    setError(null);
  }

  if (!open) return null;

  const hasPrior = chapters.length > 0;
  const isGenerating = phase === "inventing" || phase === "rendering" || phase === "writing";
  const showStreamView = isGenerating || phase === "done";
  const showActiveChapterView = !isGenerating && activeChapterData !== null;
  const showComposeView = !isGenerating && phase !== "done" && activeChapterData === null;

  // Background style: parchment by default; if a portrait is provided we
  // could overlay it at low opacity, but parchment-feel is the spec.
  const parchmentStyle: React.CSSProperties = {
    backgroundColor: "#f4ecd8",
    backgroundImage:
      "radial-gradient(circle at 20% 20%, rgba(180,140,80,0.06) 0%, transparent 40%)," +
      "radial-gradient(circle at 80% 60%, rgba(120,90,40,0.05) 0%, transparent 50%)," +
      "linear-gradient(180deg, #f4ecd8 0%, #ede2c4 100%)",
  };

  return (
    <Dialog open={open} onClose={handleCloseAttempt}>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60">
        <div
          className="w-full max-w-6xl h-[90vh] bg-card border border-border rounded-xl shadow-2xl shadow-black/40 flex overflow-hidden animate-in fade-in zoom-in-95 duration-150"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Sidebar */}
          {sidebarOpen && (
            <aside className="w-64 flex-shrink-0 border-r border-border bg-secondary/20 flex flex-col">
              <div className="p-3 border-b border-border flex items-center justify-between">
                <div className="flex items-center gap-2 min-w-0">
                  <Sparkles size={14} className="text-primary shrink-0" />
                  <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground truncate">Imagined Chapters</span>
                </div>
                <button
                  onClick={() => setSidebarOpen(false)}
                  className="text-muted-foreground hover:text-foreground"
                  title="Collapse sidebar"
                >
                  <PanelLeftClose size={14} />
                </button>
              </div>
              <button
                onClick={handleNewChapter}
                className="m-2 mb-0 px-2 py-1.5 text-xs rounded-md bg-primary/10 hover:bg-primary/20 text-primary flex items-center gap-1.5 cursor-pointer transition-colors"
              >
                <Plus size={12} />
                <span>New chapter</span>
              </button>
              <div className="flex-1 overflow-y-auto p-2 space-y-1">
                {chapters.length === 0 ? (
                  <div className="text-xs text-muted-foreground px-2 py-3">
                    No chapters yet. Start one above.
                  </div>
                ) : (
                  chapters.map((c) => {
                    const isCanonized = c.canonized || canonizedThisSession.has(c.chapter_id);
                    const isActive = activeChapterId === c.chapter_id;
                    return (
                    <div
                      key={c.chapter_id}
                      role="button"
                      tabIndex={0}
                      onClick={() => handleSelectChapter(c)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                          e.preventDefault();
                          handleSelectChapter(c);
                        }
                      }}
                      className={`w-full text-left px-2.5 py-2 rounded-md text-xs transition-all group/item cursor-pointer border ${
                        isCanonized
                          ? (isActive
                              // Canonized + selected: brighter glow, thicker border.
                              ? "bg-gradient-to-br from-amber-400/25 via-amber-500/15 to-amber-500/10 border-amber-400/70 shadow-[0_0_14px_rgba(251,191,36,0.35)] text-amber-50"
                              // Canonized + idle: steady golden presence.
                              : "bg-gradient-to-br from-amber-500/15 via-amber-500/10 to-amber-500/5 border-amber-400/40 shadow-[0_0_8px_rgba(251,191,36,0.18)] text-amber-100/95 hover:from-amber-400/20 hover:via-amber-500/15 hover:to-amber-500/10 hover:border-amber-400/60 hover:shadow-[0_0_12px_rgba(251,191,36,0.28)]")
                          : (isActive
                              ? "bg-primary/15 text-foreground border-transparent"
                              : "hover:bg-accent text-muted-foreground hover:text-foreground border-transparent")
                      }`}
                    >
                      <div className="flex items-start justify-between gap-1">
                        <div className="flex items-start gap-1.5 flex-1 min-w-0">
                          {isCanonized && (
                            <Sparkles size={11} className="text-amber-400 mt-[3px] flex-shrink-0 drop-shadow-[0_0_4px_rgba(251,191,36,0.6)]" />
                          )}
                          <span className={`line-clamp-2 flex-1 ${isCanonized ? "font-semibold" : "font-medium"}`}>
                            {c.title || "(untitled)"}
                          </span>
                        </div>
                        <button
                          onClick={(e) => handleDelete(c.chapter_id, e)}
                          className="opacity-0 group-hover/item:opacity-60 hover:opacity-100 transition-opacity"
                          title="Delete chapter"
                        >
                          <Trash2 size={11} />
                        </button>
                      </div>
                      <div className={`text-[10px] mt-1 flex items-center gap-1.5 ${isCanonized ? "text-amber-300/70" : "text-muted-foreground/60"}`}>
                        {isCanonized && (
                          <span className="text-[9px] uppercase tracking-wider font-semibold text-amber-400">Canon</span>
                        )}
                        <span>{new Date(c.created_at).toLocaleString()}</span>
                      </div>
                    </div>
                    );
                  })
                )}
              </div>
              {/* Bulk reset — one-shot cleanup for the migration's
                  auto-canonization of pre-existing chapters. */}
              {chapters.some((c) => c.canonized) && (
                <div className="border-t border-border/30 p-2">
                  <button
                    onClick={() => setBulkResetConfirm(true)}
                    disabled={canonizing}
                    className="w-full text-left px-2 py-1.5 rounded-md text-[11px] text-muted-foreground hover:text-foreground hover:bg-accent/40 transition-colors cursor-pointer disabled:opacity-50"
                    title="Decanonize every chapter in this thread"
                  >
                    Reset canonization for this chat…
                  </button>
                </div>
              )}
            </aside>
          )}

          {/* Main area */}
          <main className="flex-1 flex flex-col min-w-0" style={parchmentStyle}>
            {/* Header */}
            <header className="px-4 py-3 border-b border-amber-900/10 flex items-center justify-between bg-amber-50/30">
              <div className="flex items-center gap-2 min-w-0">
                {!sidebarOpen && (
                  <button
                    onClick={() => setSidebarOpen(true)}
                    className="text-muted-foreground hover:text-foreground"
                    title="Show sidebar"
                  >
                    <PanelLeftOpen size={15} />
                  </button>
                )}
                <Sparkles size={16} className="text-amber-700" />
                <h2 className="text-sm font-semibold text-amber-900 truncate">
                  {showActiveChapterView ? activeChapterData?.title || "(untitled)" :
                   showStreamView ? streamTitle || "Imagining…" :
                   "Imagined Chapter"}
                </h2>
              </div>
              <button onClick={handleCloseAttempt} className="text-amber-900/60 hover:text-amber-900">
                <X size={16} />
              </button>
            </header>

            {/* Body */}
            <div ref={scrollRef} className="flex-1 overflow-y-auto px-6 py-6">
              {error && (
                <div className="mb-4 rounded-lg border border-destructive/50 bg-destructive/10 text-destructive text-sm p-3">
                  {error}
                </div>
              )}

              {showComposeView && (
                <ComposeView
                  seedHint={seedHint}
                  setSeedHint={setSeedHint}
                  continueFromPrevious={continueFromPrevious}
                  setContinueFromPrevious={setContinueFromPrevious}
                  hasPrior={hasPrior}
                  imageTier={imageTier}
                  setImageTier={setImageTier}
                  depth={depth}
                  setDepth={setDepth}
                  onGenerate={handleGenerate}
                  characterPortraitUrls={characterPortraitUrls ?? []}
                  worldImageUrl={worldImageUrl}
                />
              )}

              {showStreamView && (
                <>
                  <ChapterView
                    title={streamTitle}
                    imageUrl={streamImage}
                    content={streamContent}
                    phase={phase}
                    fontPx={chatFontPx(chapterFontSize)}
                    fontSizeLevel={chapterFontSize}
                    onChangeFontSize={persistChapterFontSize}
                  />
                  {phase === "done" && streamChapterId && (
                    <CanonizeRow
                      canonized={canonizedThisSession.has(streamChapterId)}
                      canonizing={canonizing}
                      onCanonize={() => canonizeChapter(streamChapterId)}
                      onDecanonize={() => decanonizeChapter(streamChapterId)}
                    />
                  )}
                </>
              )}

              {showActiveChapterView && activeChapterData && (
                <>
                  <ChapterView
                    title={activeChapterData.title}
                    imageUrl={activeImageUrl}
                    content={activeChapterData.content}
                    phase="done"
                    fontPx={chatFontPx(chapterFontSize)}
                    fontSizeLevel={chapterFontSize}
                    onChangeFontSize={persistChapterFontSize}
                  />
                  <CanonizeRow
                    canonized={activeChapterData.canonized || canonizedThisSession.has(activeChapterData.chapter_id)}
                    canonizing={canonizing}
                    onCanonize={() => canonizeChapter(activeChapterData.chapter_id)}
                    onDecanonize={() => decanonizeChapter(activeChapterData.chapter_id)}
                  />
                </>
              )}

              {/* Close-confirm overlay — fires when user tries to close
                  the modal with a non-canonized chapter on screen. */}
              {confirmingClose && (
                <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50" onClick={() => setConfirmingClose(null)}>
                  <div
                    className="w-full max-w-md bg-card border border-border rounded-xl shadow-2xl p-5 space-y-4"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <h3 className="text-base font-semibold text-foreground">Canonize before closing?</h3>
                    <p className="text-sm text-muted-foreground">
                      <span className="font-medium text-foreground">"{confirmingClose.title || "(untitled)"}"</span> isn't canonized yet. Only canonized chapters appear in the chat history and reach the characters' memory of this world. You can still find it in the sidebar later, but it won't shape the story unless you canonize it.
                    </p>
                    <div className="flex justify-end gap-2 pt-2">
                      <Button
                        variant="ghost"
                        onClick={() => setConfirmingClose(null)}
                        disabled={canonizing}
                      >
                        Keep the chapter open
                      </Button>
                      <Button
                        variant="outline"
                        onClick={() => { setConfirmingClose(null); onClose(); }}
                        disabled={canonizing}
                      >
                        Close it and leave it outside the story
                      </Button>
                      <Button
                        onClick={async () => {
                          try {
                            await canonizeChapter(confirmingClose.chapterId);
                            setConfirmingClose(null);
                            onClose();
                          } catch { /* error already surfaced; keep modal open */ }
                        }}
                        disabled={canonizing}
                        className="bg-amber-700 hover:bg-amber-800 text-amber-50"
                      >
                        {canonizing ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <ScrollText size={14} className="mr-2" />}
                        Canonize it into the story
                      </Button>
                    </div>
                  </div>
                </div>
              )}

              {/* Bulk reset confirm — for the migration cleanup case. */}
              {bulkResetConfirm && (
                <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50" onClick={() => setBulkResetConfirm(false)}>
                  <div
                    className="w-full max-w-md bg-card border border-border rounded-xl shadow-2xl p-5 space-y-4"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <h3 className="text-base font-semibold text-foreground">Reset canonization for this chat?</h3>
                    <p className="text-sm text-muted-foreground">
                      This will decanonize <span className="font-medium text-foreground">every chapter in this chat</span> — removing their breadcrumb cards from the chat history and reverting them to pre-canon state. The chapters themselves stay in this sidebar; you can re-canonize each one individually. Useful as a one-shot cleanup if the migration auto-canonized chapters you didn't actually act on.
                    </p>
                    <div className="flex justify-end gap-2 pt-2">
                      <Button variant="ghost" onClick={() => setBulkResetConfirm(false)} disabled={canonizing}>
                        Leave canonization as it is
                      </Button>
                      <Button
                        onClick={bulkResetCanonization}
                        disabled={canonizing}
                        variant="destructive"
                      >
                        {canonizing ? <Loader2 size={14} className="animate-spin mr-1.5" /> : null}
                        Decanonize every chapter in this chat
                      </Button>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </main>
        </div>
      </div>
    </Dialog>
  );
}

function ComposeView({
  seedHint, setSeedHint,
  continueFromPrevious, setContinueFromPrevious, hasPrior,
  imageTier, setImageTier,
  depth, setDepth,
  onGenerate,
  characterPortraitUrls,
  worldImageUrl,
}: {
  seedHint: string;
  setSeedHint: (s: string) => void;
  continueFromPrevious: boolean;
  setContinueFromPrevious: (v: boolean) => void;
  hasPrior: boolean;
  imageTier: "low" | "medium" | "high";
  setImageTier: (t: "low" | "medium" | "high") => void;
  depth: "Glimpse" | "Opening" | "Deep" | "Sacred";
  setDepth: (d: "Glimpse" | "Opening" | "Deep" | "Sacred") => void;
  onGenerate: () => void;
  characterPortraitUrls: string[];
  worldImageUrl?: string;
}) {
  const portraits = characterPortraitUrls.filter(Boolean);
  return (
    <div className="space-y-5">
      {/* Edge-to-edge banner — portraits float over the active world image,
          banner alpha-fades into the parchment at the bottom via CSS mask
          (true transparency, no color-matching). Negative top + horizontal
          margins back out of the body's px-6 py-6 padding. */}
      {(portraits.length > 0 || worldImageUrl) && (
        <div className="-mx-6 -mt-6 relative">
          <div className="relative w-full h-[28rem]">
            {worldImageUrl ? (
              <img
                src={worldImageUrl}
                alt=""
                className="absolute inset-0 w-full h-full object-cover"
                style={{
                  // Mask the image's alpha so the bottom genuinely fades to
                  // transparent — whatever sits below (parchment, header
                  // shadow, anything) shows through with no color seam.
                  WebkitMaskImage:
                    "linear-gradient(to bottom, black 0%, black 55%, transparent 100%)",
                  maskImage:
                    "linear-gradient(to bottom, black 0%, black 55%, transparent 100%)",
                }}
              />
            ) : (
              <div
                className="absolute inset-0 bg-gradient-to-b from-amber-200/60 to-amber-100/40"
                style={{
                  WebkitMaskImage:
                    "linear-gradient(to bottom, black 0%, black 55%, transparent 100%)",
                  maskImage:
                    "linear-gradient(to bottom, black 0%, black 55%, transparent 100%)",
                }}
              />
            )}
            {portraits.length > 0 && (
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="flex">
                  {portraits.map((url, i) => (
                    <div
                      key={i}
                      className="w-24 h-24 rounded-full overflow-hidden border-4 border-amber-50/90 shadow-xl bg-amber-100"
                      style={{ marginLeft: i === 0 ? 0 : "-1.25rem", zIndex: portraits.length - i }}
                    >
                      <img src={url} alt="" className="w-full h-full object-cover" />
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      )}
      <div className="max-w-2xl mx-auto space-y-5">
      <div className="text-center space-y-1">
        <h3 className="text-2xl font-serif text-amber-900">Imagine a chapter</h3>
        <p className="text-xs text-amber-900/60">A new scene in this world that hasn't happened in chat. The illustration leads; the prose answers it.</p>
      </div>
      <div>
        <label className="text-xs font-medium text-amber-900/70 block mb-1.5">
          What would you like to read about? <span className="text-amber-900/40">(optional — leave blank to be surprised)</span>
        </label>
        <textarea
          value={seedHint}
          onChange={(e) => setSeedHint(e.target.value)}
          rows={3}
          placeholder='e.g. "Aaron alone at the bakery before dawn" or "a winter morning, no plot, just light"'
          className="w-full rounded-lg border border-amber-200 bg-white/60 px-3 py-2 text-sm text-amber-950 placeholder:text-amber-900/40 focus:outline-none focus:ring-1 focus:ring-amber-400 resize-y"
        />
      </div>
      {hasPrior && (
        <label className="flex items-start gap-2 cursor-pointer text-sm text-amber-900/80">
          <input
            type="checkbox"
            checked={continueFromPrevious}
            onChange={(e) => setContinueFromPrevious(e.target.checked)}
            className="mt-0.5"
          />
          <span>
            <span className="font-medium">Continue from the most recent chapter</span>
            <span className="block text-xs text-amber-900/50 mt-0.5">
              Picks up where the prior chapter left off. Default off — usually time has passed and you'd want a fresh moment.
            </span>
          </span>
        </label>
      )}
      <div>
        <label className="text-xs font-medium text-amber-900/70 block mb-1.5">Image quality</label>
        <div className="flex gap-2">
          {(["low", "medium", "high"] as const).map((t) => (
            <button
              key={t}
              onClick={() => setImageTier(t)}
              className={`px-3 py-1.5 text-xs rounded-md border transition-colors ${
                imageTier === t
                  ? "border-amber-700 bg-amber-100 text-amber-900"
                  : "border-amber-200 bg-white/60 text-amber-900/60 hover:bg-amber-50"
              }`}
            >
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </button>
          ))}
        </div>
      </div>
      {/* Profundity dial — Glimpse / Opening / Deep / Sacred. The
          register the chapter reaches for. Each level has a one-line
          subtitle so the user can pick without opening docs. */}
      <div>
        <label className="text-xs font-medium text-amber-900/70 block mb-1.5">Depth</label>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
          {(
            [
              { key: "Glimpse", subtitle: "a quiet daily moment" },
              { key: "Opening", subtitle: "one layer below default" },
              { key: "Deep", subtitle: "interior visible, real cost" },
              { key: "Sacred", subtitle: "confessional, threshold; rare" },
            ] as const
          ).map((d) => (
            <button
              key={d.key}
              onClick={() => setDepth(d.key)}
              className={`px-3 py-2 text-xs rounded-md border transition-colors text-left ${
                depth === d.key
                  ? "border-amber-700 bg-amber-100 text-amber-900"
                  : "border-amber-200 bg-white/60 text-amber-900/60 hover:bg-amber-50"
              }`}
            >
              <div className="font-medium">{d.key}</div>
              <div className="text-[10px] text-amber-900/55 mt-0.5 leading-snug">{d.subtitle}</div>
            </button>
          ))}
        </div>
      </div>
      <div className="flex justify-center pt-2">
        <Button onClick={onGenerate} className="bg-amber-700 hover:bg-amber-800 text-amber-50">
          <Sparkles size={14} className="mr-2" />
          Imagine the chapter
        </Button>
      </div>
      </div>
    </div>
  );
}

function ChapterView({
  title: _title,
  imageUrl,
  content,
  phase,
  fontPx,
  fontSizeLevel,
  onChangeFontSize,
}: {
  title: string;
  imageUrl: string;
  content: string;
  phase: "idle" | "inventing" | "rendering" | "writing" | "done";
  fontPx: number;
  fontSizeLevel: number;
  onChangeFontSize: (next: number) => void;
}) {
  // Drop-cap: render the first character of the content as a large
  // ornamental letter, like the day-pages novelization.
  const firstChar = content.trim().charAt(0);
  const rest = content.trim().slice(1);

  const phaseLabel = phase === "inventing" ? "Imagining a moment…" :
                     phase === "rendering" ? "Painting the scene…" :
                     phase === "writing" ? "Writing the chapter…" :
                     "";

  return (
    <article className="max-w-2xl mx-auto">
      {/* Image */}
      {imageUrl ? (
        <figure className="mb-6">
          <img
            src={imageUrl}
            alt=""
            className="w-full rounded-lg shadow-lg border border-amber-900/15"
          />
        </figure>
      ) : (phase === "inventing" || phase === "rendering") ? (
        <div className="mb-6 aspect-[3/2] rounded-lg bg-amber-100/40 border border-amber-200/60 flex flex-col items-center justify-center gap-2">
          <ImageIcon size={32} className="text-amber-700/40" />
          <span className="text-xs text-amber-900/50">{phaseLabel}</span>
        </div>
      ) : null}

      {/* Phase indicator while writing */}
      {phase === "writing" && content.length === 0 && (
        <div className="flex items-center gap-2 text-sm text-amber-900/60 mb-4">
          <Loader2 size={14} className="animate-spin" />
          <span>Writing the chapter…</span>
        </div>
      )}

      {/* Font-size adjuster — shown above the prose so the reader can dial
          the chapter to a comfortable size. Right-aligned, unobtrusive. */}
      {content.length > 0 && (
        <div className="flex justify-end mb-3">
          <FontSizeAdjuster value={fontSizeLevel} onChange={onChangeFontSize} />
        </div>
      )}

      {/* Content with drop-cap.
          NOTE: deliberately NOT using Tailwind's `prose` plugin here —
          its child selectors set their own font-size, which would
          shadow the inline `fontSize` and break the adjuster. We use
          plain inheritance instead and add explicit paragraph spacing
          via `[&>p]:mb-4` so the rendered markdown still breathes. */}
      {content.length > 0 && (
        <div
          className="font-serif leading-relaxed text-amber-950 chapter-prose [&>p]:mb-4 [&>p:last-child]:mb-0 [&_em]:italic [&_strong]:font-bold"
          style={{ fontSize: `${fontPx}px` }}
        >
          {firstChar && (
            <span className="float-left text-7xl font-serif font-bold leading-[0.85] mr-2 mt-1 text-amber-800">
              {firstChar}
            </span>
          )}
          <Markdown remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins} components={markdownComponents}>
            {rest}
          </Markdown>
        </div>
      )}
    </article>
  );
}

function CanonizeRow({
  canonized,
  canonizing,
  onCanonize,
  onDecanonize,
}: {
  canonized: boolean;
  canonizing: boolean;
  onCanonize: () => void;
  onDecanonize?: () => void;
}) {
  return (
    <div className="max-w-2xl mx-auto mt-6 pt-4 border-t border-amber-900/15 flex justify-center items-center gap-3">
      {canonized ? (
        <>
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-md bg-emerald-100/50 border border-emerald-700/30 text-emerald-900 text-sm font-medium">
            <ScrollText size={14} />
            <span>Canonized — this chapter is part of the world</span>
          </div>
          {onDecanonize && (
            <button
              onClick={onDecanonize}
              disabled={canonizing}
              className="text-xs text-amber-900/60 hover:text-amber-900 underline underline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
              title="Remove this chapter from chat history and canon"
            >
              {canonizing ? "Decanonizing…" : "Decanonize"}
            </button>
          )}
        </>
      ) : (
        <Button
          onClick={onCanonize}
          disabled={canonizing}
          className="bg-amber-700 hover:bg-amber-800 text-amber-50"
        >
          {canonizing ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <ScrollText size={14} className="mr-2" />}
          Canonize this chapter
        </Button>
      )}
    </div>
  );
}

import { useState, useRef, useEffect, useCallback } from "react";
import Markdown from "react-markdown";
import { Dialog } from "@/components/ui/dialog";
import { X, Loader2, Sparkles, Trash2, Plus, PanelLeftClose, PanelLeftOpen, Image as ImageIcon, ScrollText } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { api, type ImaginedChapter, type ImaginedChapterStageEvent, type ImaginedChapterImageEvent, type ImaginedChapterDoneEvent } from "@/lib/tauri";
import { Button } from "@/components/ui/button";
import { chatFontPx } from "@/lib/chat-font";
import { remarkPlugins, rehypePlugins, markdownComponents } from "./formatMessage";

interface Props {
  open: boolean;
  onClose: () => void;
  apiKey: string;
  threadId: string;
  /** Data URL for the active chat's primary character portrait — shown
   *  on the compose view's hero. Optional. */
  characterPortraitUrl?: string;
  /** Whether to play a chime on first token of writing phase. */
  notifyOnMessage: boolean;
  /** Chat font size shared with ChatView/GroupChatView. */
  chatFontSize: string;
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
  characterPortraitUrl,
  notifyOnMessage: _notifyOnMessage,
  chatFontSize,
  openChapterId,
  onCanonize,
}: Props) {
  const [chapters, setChapters] = useState<ImaginedChapter[]>([]);
  const [activeChapterId, setActiveChapterId] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  // Compose state for a NEW chapter
  const [seedHint, setSeedHint] = useState("");
  const [continueFromPrevious, setContinueFromPrevious] = useState(false);
  const [imageTier, setImageTier] = useState<"low" | "medium" | "high">("medium");

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

  const scrollRef = useRef<HTMLDivElement | null>(null);
  const generatingRef = useRef(false);

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
    }
  }, [open]);

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
    }).then((u) => unlisteners.push(u));

    listen<string>("imagined-chapter-token", (e) => {
      if (!generatingRef.current) return;
      setStreamContent((prev) => prev + (e.payload || ""));
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
  }, [open, loadChapters]);

  // Auto-scroll while writing
  useEffect(() => {
    if (phase !== "writing" && phase !== "done") return;
    const el = scrollRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [streamContent, phase]);

  async function handleGenerate() {
    if (generatingRef.current) return;
    setError(null);
    setPhase("inventing");
    setStreamTitle("");
    setStreamImage("");
    setStreamContent("");
    setStreamChapterId(null);
    setActiveChapterId(null);
    setActiveChapterData(null);
    generatingRef.current = true;
    try {
      const res = await api.generateImaginedChapter(apiKey, {
        threadId,
        seedHint: seedHint.trim() || undefined,
        continueFromPrevious,
        imageTier,
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
    <Dialog open={open} onOpenChange={(v) => !v && onClose()}>
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
                  chapters.map((c) => (
                    <button
                      key={c.chapter_id}
                      onClick={() => handleSelectChapter(c)}
                      className={`w-full text-left px-2 py-1.5 rounded-md text-xs transition-colors group/item ${
                        activeChapterId === c.chapter_id
                          ? "bg-primary/15 text-foreground"
                          : "hover:bg-accent text-muted-foreground hover:text-foreground"
                      }`}
                    >
                      <div className="flex items-start justify-between gap-1">
                        <span className="line-clamp-2 flex-1 font-medium">{c.title || "(untitled)"}</span>
                        <button
                          onClick={(e) => handleDelete(c.chapter_id, e)}
                          className="opacity-0 group-hover/item:opacity-60 hover:opacity-100 transition-opacity"
                          title="Delete chapter"
                        >
                          <Trash2 size={11} />
                        </button>
                      </div>
                      <div className="text-[10px] text-muted-foreground/60 mt-0.5">
                        {new Date(c.created_at).toLocaleString()}
                      </div>
                    </button>
                  ))
                )}
              </div>
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
              <button onClick={onClose} className="text-amber-900/60 hover:text-amber-900">
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
                  onGenerate={handleGenerate}
                  characterPortraitUrl={characterPortraitUrl ?? null}
                />
              )}

              {showStreamView && (
                <ChapterView
                  title={streamTitle}
                  imageUrl={streamImage}
                  content={streamContent}
                  phase={phase}
                  fontPx={chatFontPx(chatFontSize)}
                />
              )}

              {showActiveChapterView && activeChapterData && (
                <>
                  <ChapterView
                    title={activeChapterData.title}
                    imageUrl={activeImageUrl}
                    content={activeChapterData.content}
                    phase="done"
                    fontPx={chatFontPx(chatFontSize)}
                  />
                  {onCanonize && activeChapterData.breadcrumb_message_id && (
                    <div className="max-w-2xl mx-auto mt-6 pt-4 border-t border-amber-900/15 flex justify-center">
                      <Button
                        variant="outline"
                        onClick={() => {
                          onCanonize(activeChapterData.breadcrumb_message_id!, activeChapterData.title);
                          onClose();
                        }}
                        className="border-amber-700/50 text-amber-900 hover:bg-amber-100"
                      >
                        <ScrollText size={14} className="mr-2" />
                        Canonize this chapter
                      </Button>
                    </div>
                  )}
                </>
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
  onGenerate,
  characterPortraitUrl,
}: {
  seedHint: string;
  setSeedHint: (s: string) => void;
  continueFromPrevious: boolean;
  setContinueFromPrevious: (v: boolean) => void;
  hasPrior: boolean;
  imageTier: "low" | "medium" | "high";
  setImageTier: (t: "low" | "medium" | "high") => void;
  onGenerate: () => void;
  characterPortraitUrl: string | null;
}) {
  return (
    <div className="max-w-2xl mx-auto space-y-5">
      {characterPortraitUrl && (
        <div className="flex justify-center mb-2">
          <div className="w-24 h-24 rounded-full overflow-hidden border-4 border-amber-200/60 shadow-md">
            <img src={characterPortraitUrl} alt="" className="w-full h-full object-cover" />
          </div>
        </div>
      )}
      <div className="text-center space-y-1">
        <h3 className="text-2xl font-serif text-amber-900">Imagine a chapter</h3>
        <p className="text-xs text-amber-900/60">A new scene in this world that hasn't happened in chat. The illustration leads; the prose answers it.</p>
      </div>
      <div>
        <label className="text-xs font-medium text-amber-900/70 block mb-1.5">
          What would you like to read about? <span className="text-amber-900/40">(optional — leave blank for LLM's choice)</span>
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
      <div className="flex justify-center pt-2">
        <Button onClick={onGenerate} className="bg-amber-700 hover:bg-amber-800 text-amber-50">
          <Sparkles size={14} className="mr-2" />
          Imagine the chapter
        </Button>
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
}: {
  title: string;
  imageUrl: string;
  content: string;
  phase: "idle" | "inventing" | "rendering" | "writing" | "done";
  fontPx: number;
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

      {/* Content with drop-cap */}
      {content.length > 0 && (
        <div
          className="font-serif leading-relaxed text-amber-950 chapter-prose"
          style={{ fontSize: `${fontPx}px` }}
        >
          {firstChar && (
            <span className="float-left text-7xl font-serif font-bold leading-[0.85] mr-2 mt-1 text-amber-800">
              {firstChar}
            </span>
          )}
          <div className="prose prose-stone max-w-none">
            <Markdown remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins} components={markdownComponents}>
              {rest}
            </Markdown>
          </div>
        </div>
      )}
    </article>
  );
}

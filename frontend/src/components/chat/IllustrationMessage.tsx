import React, { useState, useRef, useEffect } from "react";
import { Image, Square, Repeat, SlidersHorizontal, RefreshCw, Trash2, ExternalLink, Check, Download, Video, Pencil, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { api, type Message } from "@/lib/tauri";
import type { useAppStore } from "@/hooks/use-app-store";

interface Props {
  msg: Message;
  isPending: boolean;
  isSending: boolean;
  isGeneratingVideo: boolean;
  store: ReturnType<typeof useAppStore>;
  /** Caption text stored with the illustration — either the user's own
   *  instructions verbatim or an LLM-picked "memorable moment" sentence.
   *  Rendered as a visible caption below the image AND as the alt attr. */
  caption?: string;
  /** Persist a user-edited caption. Called when the user saves an inline
   *  edit. Updates the DB and any cached caption state. */
  onCaptionChange?: (messageId: string, caption: string) => void;
  // Video state
  playingVideo: string | null;
  setPlayingVideo: (v: string | null) => void;
  loopVideo: Record<string, boolean>;
  setLoopVideo: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
  videoFiles: Record<string, string>;
  setVideoFiles: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  videoDataUrls: Record<string, string>;
  playVideoFn: (messageId: string) => void;
  // Illustration actions
  setIllustrationModalId: (id: string | null) => void;
  setModalSelectedId: (id: string | null) => void;
  setModalPlayingVideo: (v: boolean) => void;
  setModalImageLoading: (v: boolean) => void;
  setModalIllustrations: (v: Array<{ id: string; content: string }>) => void;
  setAdjustIllustrationId: (id: string | null) => void;
  setAdjustInstructions: (v: string) => void;
  setVideoModalId: (id: string | null) => void;
  setVideoPrompt: (v: string) => void;
  setVideoDuration: (v: number) => void;
  setVideoStyle: (v: string) => void;
  setVideoTab: (v: "generate" | "upload") => void;
  setRemoveVideoConfirmId: (id: string | null) => void;
  setResetConfirmId: (id: string | null) => void;
  downloadedId: string | null;
  setDownloadedId: (id: string | null) => void;
  // For loading carousel illustrations
  loadIllustrations: () => Promise<void>;
}

export function IllustrationMessage({
  msg, isPending, isSending, isGeneratingVideo, store,
  playingVideo, setPlayingVideo, loopVideo, setLoopVideo,
  videoFiles, setVideoFiles, videoDataUrls, playVideoFn,
  setIllustrationModalId, setModalSelectedId, setModalPlayingVideo, setModalImageLoading, setModalIllustrations,
  setAdjustIllustrationId, setAdjustInstructions,
  setVideoModalId, setVideoPrompt, setVideoDuration, setVideoStyle, setVideoTab,
  setRemoveVideoConfirmId, setResetConfirmId,
  downloadedId, setDownloadedId, loadIllustrations,
  caption, onCaptionChange,
}: Props) {
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [showRegenConfirm, setShowRegenConfirm] = useState(false);
  const [editingCaption, setEditingCaption] = useState(false);
  const [captionDraft, setCaptionDraft] = useState("");
  const [savingCaption, setSavingCaption] = useState(false);
  const captionInputRef = useRef<HTMLTextAreaElement | null>(null);

  const startEditCaption = () => {
    setCaptionDraft(caption ?? "");
    setEditingCaption(true);
  };
  const cancelEditCaption = () => {
    setEditingCaption(false);
    setCaptionDraft("");
  };
  const saveCaption = async () => {
    if (!onCaptionChange) { cancelEditCaption(); return; }
    const next = captionDraft.trim();
    if (next === (caption ?? "").trim()) { cancelEditCaption(); return; }
    setSavingCaption(true);
    try {
      await onCaptionChange(msg.message_id, next);
      setEditingCaption(false);
      setCaptionDraft("");
    } finally {
      setSavingCaption(false);
    }
  };

  useEffect(() => {
    if (editingCaption) {
      // Focus + move cursor to end.
      requestAnimationFrame(() => {
        const el = captionInputRef.current;
        if (el) { el.focus(); el.setSelectionRange(el.value.length, el.value.length); }
      });
    }
  }, [editingCaption]);

  return (<>
    <div data-message-id={msg.message_id} className="flex justify-center my-3">
      <div className="relative group/illus max-w-[95%] rounded-xl bg-gradient-to-br from-emerald-950/30 to-emerald-900/10 border border-emerald-700/20 backdrop-blur-sm">
        <div className="flex items-center gap-1.5 px-4 pt-3 pb-1.5 text-[10px] uppercase tracking-wider text-emerald-500/70 font-semibold">
          <Image size={12} />
          <span>Illustration</span>
        </div>
        <div className="px-2 pb-2 relative">
          <img
            src={msg.content}
            alt={caption && caption.trim() !== "" ? caption : "Scene illustration"}
            loading="lazy"
            style={store.aspectRatios[msg.message_id] ? { aspectRatio: String(store.aspectRatios[msg.message_id]) } : undefined}
            className={`w-full rounded-lg cursor-pointer ${playingVideo === msg.message_id && videoDataUrls[msg.message_id] ? "invisible" : ""}`}
            onClick={async () => {
              setIllustrationModalId(msg.message_id);
              setModalSelectedId(msg.message_id);
              setModalPlayingVideo(false);
              setModalImageLoading(false);
              await loadIllustrations();
            }}
          />
          {playingVideo === msg.message_id && videoDataUrls[msg.message_id] && (
            <>
              <video
                src={videoDataUrls[msg.message_id]}
                autoPlay
                loop={!!loopVideo[msg.message_id]}
                playsInline
                className="absolute inset-2 w-[calc(100%-16px)] h-[calc(100%-16px)] object-contain rounded-lg"
                onEnded={() => { if (!loopVideo[msg.message_id]) setPlayingVideo(null); }}
              />
              <button
                onClick={() => setPlayingVideo(null)}
                className="absolute bottom-4 right-4 w-10 h-10 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm opacity-0 group-hover/illus:opacity-100"
                title="Stop"
              >
                <Square size={14} fill="white" />
              </button>
            </>
          )}
          {playingVideo !== msg.message_id && videoFiles[msg.message_id] && (
            <div className="absolute bottom-4 right-4 flex gap-1.5">
              <button
                onClick={() => setLoopVideo((prev) => ({ ...prev, [msg.message_id]: !prev[msg.message_id] }))}
                className={`w-10 h-10 rounded-full backdrop-blur-sm flex items-center justify-center cursor-pointer transition-colors ${
                  loopVideo[msg.message_id]
                    ? "bg-purple-600 text-white"
                    : "bg-black/70 text-white/50 hover:text-white hover:bg-black/80"
                }`}
                title={loopVideo[msg.message_id] ? "Loop on" : "Loop off"}
              >
                <Repeat size={14} />
              </button>
              <button
                onClick={() => playVideoFn(msg.message_id)}
                className="w-10 h-10 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-purple-600 transition-colors backdrop-blur-sm"
                title="Play animation"
              >
                <span className="text-lg ml-0.5">&#9654;</span>
              </button>
            </div>
          )}
          {playingVideo === msg.message_id && !videoDataUrls[msg.message_id] && (
            <div className="absolute inset-2 flex items-center justify-center bg-black/30 rounded-lg">
              <div className="animate-spin w-8 h-8 border-2 border-white/20 border-t-white rounded-full" />
            </div>
          )}
          {!isPending && !isSending && (
            <div className="absolute top-4 right-4 flex gap-1.5 opacity-0 group-hover/illus:opacity-100 transition-opacity">
              <div className="relative group/adj">
                <button
                  onClick={() => { setAdjustIllustrationId(msg.message_id); setAdjustInstructions(""); }}
                  className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
                >
                  <SlidersHorizontal size={14} />
                </button>
                <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/adj:opacity-100 pointer-events-none transition-opacity">Adjust</span>
              </div>
              <div className="relative group/regen">
                <button
                  onClick={() => setShowRegenConfirm(true)}
                  className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
                >
                  <RefreshCw size={14} />
                </button>
                <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/regen:opacity-100 pointer-events-none transition-opacity">Regenerate</span>
              </div>
              <div className="relative group/del">
                <button
                  onClick={() => setShowDeleteConfirm(true)}
                  className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-destructive transition-colors backdrop-blur-sm"
                >
                  <Trash2 size={14} />
                </button>
                <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/del:opacity-100 pointer-events-none transition-opacity">Delete</span>
              </div>
              <div className="relative group/pop">
                <button
                  onClick={async () => {
                    const label = `illus-${msg.message_id.slice(0, 8)}`;
                    try {
                      const existing = await WebviewWindow.getByLabel(label);
                      if (existing) { await existing.setFocus(); return; }
                    } catch { /* not found */ }
                    new WebviewWindow(label, {
                      url: `index.html?illustration=${msg.message_id}`,
                      title: "Illustration",
                      width: 1280,
                      height: 760,
                      resizable: true,
                      decorations: true,
                    });
                  }}
                  className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
                >
                  <ExternalLink size={14} />
                </button>
                <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/pop:opacity-100 pointer-events-none transition-opacity">Pop Out</span>
              </div>
              <div className="relative group/dl">
                <button
                  onClick={async () => {
                    await api.downloadIllustration(msg.message_id);
                    setDownloadedId(msg.message_id);
                    setTimeout(() => setDownloadedId(null), 1500);
                  }}
                  className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
                >
                  {downloadedId === msg.message_id ? <Check size={14} /> : <Download size={14} />}
                </button>
                <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/dl:opacity-100 pointer-events-none transition-opacity">{downloadedId === msg.message_id ? "Saved!" : "Download"}</span>
              </div>
              {/* Video controls — feature-gated on Google AI API key. If
                  the user hasn't set one up, we hide the Animate button
                  entirely (no teaser, no upsell). Existing videos still
                  show their remove button so users can clean up. */}
              {(store.googleApiKey || videoFiles[msg.message_id]) && (
                <div className="relative group/vid">
                  {videoFiles[msg.message_id] ? (
                    <button
                      onClick={() => setRemoveVideoConfirmId(msg.message_id)}
                      className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-destructive transition-colors backdrop-blur-sm"
                    >
                      <span className="relative">
                        <Video size={14} />
                        <span className="absolute inset-0 flex items-center justify-center">
                          <span className="block w-[18px] h-[1.5px] bg-white rotate-45" />
                        </span>
                      </span>
                    </button>
                  ) : (
                    <button
                      onClick={() => { setVideoModalId(msg.message_id); setVideoPrompt(""); setVideoDuration(8); setVideoStyle("action-no-dialogue"); setVideoTab("generate"); }}
                      className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-purple-600 transition-colors backdrop-blur-sm"
                      disabled={isGeneratingVideo}
                    >
                      <Video size={14} />
                    </button>
                  )}
                  <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/vid:opacity-100 pointer-events-none transition-opacity">{videoFiles[msg.message_id] ? "Remove Video" : "Animate"}</span>
                </div>
              )}
            </div>
          )}
          {store.generatingVideo === msg.message_id && (
            <div className="absolute inset-x-2 bottom-2 rounded-b-lg bg-gradient-to-t from-purple-950/90 to-purple-950/40 backdrop-blur-sm px-4 py-2.5 flex items-center gap-2 text-purple-300/90">
              <Video size={14} className="animate-pulse" />
              <span className="text-xs italic">Generating animation...</span>
              <span className="w-1.5 h-1.5 rounded-full bg-purple-400/60 animate-bounce [animation-delay:0ms]" />
              <span className="w-1.5 h-1.5 rounded-full bg-purple-400/60 animate-bounce [animation-delay:150ms]" />
              <span className="w-1.5 h-1.5 rounded-full bg-purple-400/60 animate-bounce [animation-delay:300ms]" />
            </div>
          )}
        </div>
        {/* Caption: click pencil on hover to edit. When no caption exists,
            an "Add caption" affordance still appears on hover so the user
            can attach one. Editing is inline (textarea + Save/Cancel).
            Persisting the edit updates both the chat display and what the
            LLM sees as [Illustration — {caption}] in future history. */}
        {editingCaption ? (
          <div className="px-4 pt-1 pb-2">
            <textarea
              ref={captionInputRef}
              value={captionDraft}
              onChange={(e) => setCaptionDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Escape") { e.preventDefault(); cancelEditCaption(); }
                if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) { e.preventDefault(); saveCaption(); }
              }}
              rows={Math.max(2, Math.min(6, (captionDraft.match(/\n/g)?.length ?? 0) + 2))}
              placeholder="Describe what's actually pictured…"
              className="w-full bg-emerald-950/40 border border-emerald-700/40 rounded-md px-2.5 py-1.5 text-xs text-emerald-100 italic leading-snug resize-y focus:outline-none focus:ring-1 focus:ring-emerald-500/60"
              disabled={savingCaption}
            />
            <div className="flex items-center gap-1.5 mt-1.5">
              <button
                onClick={saveCaption}
                disabled={savingCaption}
                className="text-[10px] px-2 py-1 rounded-md bg-emerald-600/80 text-white hover:bg-emerald-600 transition-colors cursor-pointer disabled:opacity-50 not-italic flex items-center gap-1"
              >
                <Check size={10} /> Save
              </button>
              <button
                onClick={cancelEditCaption}
                disabled={savingCaption}
                className="text-[10px] px-2 py-1 rounded-md text-emerald-300/70 hover:text-emerald-200 hover:bg-emerald-900/40 transition-colors cursor-pointer disabled:opacity-50 not-italic flex items-center gap-1"
              >
                <X size={10} /> Cancel
              </button>
              <span className="text-[9px] text-emerald-500/40 ml-auto not-italic">⌘↵ to save · Esc to cancel</span>
            </div>
          </div>
        ) : caption && caption.trim() !== "" ? (
          <div className="px-4 pt-0.5 pb-2 group/cap flex items-start gap-1.5">
            <p className="text-xs text-emerald-100/80 italic leading-snug flex-1">{caption}</p>
            {!isPending && onCaptionChange && (
              <button
                onClick={startEditCaption}
                className="opacity-0 group-hover/cap:opacity-100 transition-opacity text-emerald-500/50 hover:text-emerald-300 flex-shrink-0 mt-0.5 cursor-pointer"
                title="Edit caption"
              >
                <Pencil size={10} />
              </button>
            )}
          </div>
        ) : !isPending && onCaptionChange ? (
          <div className="px-4 pt-0.5 pb-2 opacity-0 group-hover/illus:opacity-100 transition-opacity">
            <button
              onClick={startEditCaption}
              className="text-[10px] italic text-emerald-500/50 hover:text-emerald-300 cursor-pointer flex items-center gap-1"
            >
              <Pencil size={9} /> Add caption
            </button>
          </div>
        ) : null}
        <p className="text-[10px] px-4 pb-3 text-emerald-500/50 flex items-center gap-2">
          {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
          {!isPending && (
            <button
              onClick={() => setResetConfirmId(msg.message_id)}
              className="opacity-0 group-hover/illus:opacity-100 transition-opacity text-emerald-500/40 hover:text-emerald-400 cursor-pointer"
            >
              Reset to Here
            </button>
          )}
        </p>
      </div>
    </div>

    <Dialog open={showDeleteConfirm} onClose={() => setShowDeleteConfirm(false)} className="max-w-xs">
      <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
        <div className="flex items-center gap-2">
          <Trash2 size={18} className="text-destructive" />
          <h3 className="font-semibold">Delete Illustration</h3>
        </div>
        <p className="text-sm text-muted-foreground">
          This will permanently delete this illustration and any attached video.
        </p>
        <div className="flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={() => setShowDeleteConfirm(false)}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            size="sm"
            onClick={() => {
              setShowDeleteConfirm(false);
              store.deleteIllustration(msg.message_id);
            }}
          >
            Delete
          </Button>
        </div>
      </div>
    </Dialog>

    <Dialog open={showRegenConfirm} onClose={() => setShowRegenConfirm(false)} className="max-w-xs">
      <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
        <div className="flex items-center gap-2">
          <RefreshCw size={18} className="text-primary" />
          <h3 className="font-semibold">Regenerate Illustration</h3>
        </div>
        <p className="text-sm text-muted-foreground">
          This will replace the current illustration with a new one. The original cannot be recovered.
        </p>
        <div className="flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={() => setShowRegenConfirm(false)}>
            Cancel
          </Button>
          <Button
            size="sm"
            onClick={() => {
              setShowRegenConfirm(false);
              store.regenerateIllustration(msg.message_id);
            }}
          >
            Regenerate
          </Button>
        </div>
      </div>
    </Dialog>
  </>);
}

import { useRef, useState } from "react";
import Markdown from "react-markdown";
import { BookOpen, Volume2, Loader2, Square, Play, SlidersHorizontal, Trash2, ScrollText, Package } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { formatMessage, markdownComponents, remarkPlugins, rehypePlugins } from "./formatMessage";
import type { Message, InventoryUpdateRecord } from "@/lib/tauri";
import { InventoryUpdateBadge } from "@/components/chat/InventoryUpdateBadge";
import { chatFontPx } from "@/lib/chat-font";

interface NarrativeMessageProps {
  message: Message;
  isPending: boolean;
  onResetToHere: (id: string) => void;
  // Audio
  cachedTones?: Set<string>;
  lastTone?: string;
  speakingId: string | null;
  loadingSpeech: string | null;
  toneMenuId: string | null;
  setToneMenuId: (id: string | null) => void;
  onSpeak: (messageId: string, text: string, tone?: string) => void;
  onStopSpeaking: () => void;
  onDeleteAudio: (messageId: string) => void;
  toneMenuRef: React.RefObject<HTMLDivElement | null>;
  // Adjust
  adjustingMessageId: string | null;
  onAdjust: (messageId: string) => void;
  onDelete: (messageId: string) => void;
  /** Level from the chat font-size preference (0..5). Applied to the
   *  narrative prose so it tracks message bubbles. */
  chatFontSize?: number;
  // Canon
  onKeep?: (messageId: string) => void;
  isKept?: boolean;
  // On-demand inventory update anchored to this message
  onInventoryUpdate?: (messageId: string) => void;
  inventoryUpdatingId?: string | null;
  inventoryUpdateRecords?: InventoryUpdateRecord[];
}

export function NarrativeMessage({
  message, isPending, onResetToHere,
  cachedTones, lastTone, speakingId, loadingSpeech, toneMenuId, setToneMenuId,
  onSpeak, onStopSpeaking, onDeleteAudio, toneMenuRef,
  adjustingMessageId, onAdjust, onDelete,
  chatFontSize = 2,
  onKeep, isKept,
  onInventoryUpdate, inventoryUpdatingId, inventoryUpdateRecords,
}: NarrativeMessageProps) {
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const hasCached = cachedTones && cachedTones.size > 0;
  const isSpeaking = speakingId === message.message_id;
  const isLoading = loadingSpeech === message.message_id;
  const allTones = ["Auto", "Playful", "Happy", "Excited", "Reverent", "Serene", "Intimate", "Tender", "Sad", "Melancholy", "Angry", "Anxious"];

  return (<>
    <div key={message.message_id} data-message-id={message.message_id} className="flex justify-center my-2">
      <div className={`relative group max-w-[90%] rounded-xl px-5 py-3.5 text-sm leading-relaxed bg-gradient-to-br from-amber-950/40 to-amber-900/20 border border-amber-700/30 text-amber-100/90 italic backdrop-blur-sm ${
        isKept
          ? "ring-2 ring-amber-300 shadow-[0_0_0_4px_rgba(251,191,36,0.45),0_0_32px_10px_rgba(251,191,36,0.75),0_0_80px_20px_rgba(245,158,11,0.55),0_0_160px_40px_rgba(251,191,36,0.30)] [&>*]:relative [&>*]:z-10 before:content-[''] before:absolute before:inset-0 before:rounded-[inherit] before:pointer-events-none before:bg-gradient-to-br before:from-amber-200/50 before:via-amber-300/40 before:to-yellow-300/50 before:mix-blend-overlay before:blur-xl before:bg-[length:200%_200%] before:animate-[canonized-shimmer_9s_ease-in-out_infinite]"
          : ""
      }`}>
        <div className="flex items-center gap-1.5 mb-1.5 text-[10px] uppercase tracking-wider text-amber-500/70 font-semibold not-italic">
          <BookOpen size={12} />
          <span>Narrative</span>
        </div>

        {/* Speak button — top right */}
        {!isPending && (
          <div className={`absolute -top-2.5 -right-2.5 z-10 ${hasCached ? "" : "opacity-0 group-hover:opacity-100"} transition-opacity`}>
            <button
              onClick={() => {
                if (isSpeaking) {
                  onStopSpeaking();
                } else {
                  setToneMenuId(toneMenuId === message.message_id ? null : message.message_id);
                }
              }}
              className={`w-7 h-7 flex items-center justify-center rounded-full shadow-md border hover:scale-110 transition-all cursor-pointer ${
                isSpeaking
                  ? "bg-primary text-white border-primary/50"
                  : isLoading
                    ? "bg-white text-primary border-border/50"
                    : hasCached
                      ? "bg-amber-500/15 text-amber-500 border-amber-500/30"
                      : "bg-white text-muted-foreground hover:text-foreground border-border/50"
              }`}
            >
              {isLoading ? <Loader2 size={14} className="animate-spin" /> : isSpeaking ? <Square size={10} fill="white" /> : <Volume2 size={14} />}
            </button>
            {toneMenuId === message.message_id && (
              <div ref={toneMenuRef} className="absolute top-full right-0 mt-1.5 bg-card border border-border rounded-lg shadow-xl shadow-black/20 p-2.5 z-50 w-[280px] not-italic">
                {hasCached && lastTone && (
                  <button
                    onClick={() => {
                      setToneMenuId(null);
                      onSpeak(message.message_id, message.content, lastTone === "auto" ? "Auto" : lastTone.charAt(0).toUpperCase() + lastTone.slice(1));
                    }}
                    className="w-full text-left px-2.5 py-1.5 mb-2 text-xs hover:bg-accent transition-colors cursor-pointer flex items-center gap-2 font-medium rounded-md border border-border/50"
                  >
                    <Play size={10} fill="currentColor" className="text-primary flex-shrink-0" />
                    Last: {lastTone === "auto" ? "Auto" : lastTone.charAt(0).toUpperCase() + lastTone.slice(1)}
                  </button>
                )}
                <div className="grid grid-cols-3 gap-1">
                  {allTones.map((tone) => {
                    const isCached = cachedTones?.has(tone.toLowerCase());
                    return (
                      <button
                        key={tone}
                        onClick={() => {
                          setToneMenuId(null);
                          onSpeak(message.message_id, message.content, tone);
                        }}
                        className="px-2 py-1.5 text-[11px] rounded-md hover:bg-accent transition-colors cursor-pointer flex items-center justify-center gap-1"
                      >
                        {tone === "Auto" ? <span className="text-muted-foreground">Auto</span> : tone}
                        {isCached && <Volume2 size={8} className="text-primary flex-shrink-0" />}
                      </button>
                    );
                  })}
                </div>
                {hasCached && (
                  <button
                    onClick={async () => {
                      setToneMenuId(null);
                      if (isSpeaking) onStopSpeaking();
                      onDeleteAudio(message.message_id);
                    }}
                    className="w-full mt-2 pt-2 border-t border-border text-left px-2 py-1 text-[11px] hover:bg-accent transition-colors cursor-pointer flex items-center gap-1.5 text-red-400 rounded-md"
                  >
                    <Trash2 size={10} className="flex-shrink-0" />
                    Delete Audio
                  </button>
                )}
              </div>
            )}
          </div>
        )}

        {/* Adjust + Canonize + Delete buttons — top right, offset left of speak button */}
        {!isPending && (
          <div className="absolute top-2 right-8 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <div className="relative group/madj">
              <button
                onClick={() => onAdjust(message.message_id)}
                className="w-7 h-7 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm"
              >
                <SlidersHorizontal size={12} />
              </button>
              <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/madj:opacity-100 pointer-events-none transition-opacity not-italic">Adjust</span>
            </div>
            {onKeep && (
              <div className="relative group/mcanon">
                <button
                  onClick={() => onKeep(message.message_id)}
                  className={`w-7 h-7 rounded-full flex items-center justify-center cursor-pointer backdrop-blur-sm transition-colors ${
                    isKept
                      ? "bg-amber-500/25 text-amber-200 hover:bg-amber-500/40"
                      : "bg-black/50 text-white hover:bg-black/70"
                  }`}
                >
                  <ScrollText size={12} />
                </button>
                <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mcanon:opacity-100 pointer-events-none transition-opacity not-italic">
                  {isKept ? "Kept · save again" : "Keep to record"}
                </span>
              </div>
            )}
            {onInventoryUpdate && (() => {
              const hasTriggered = (inventoryUpdateRecords?.length ?? 0) > 0;
              return (
                <div className="relative group/minv">
                  <button
                    onClick={() => onInventoryUpdate(message.message_id)}
                    disabled={inventoryUpdatingId === message.message_id}
                    className={`w-7 h-7 rounded-full flex items-center justify-center cursor-pointer transition-colors backdrop-blur-sm disabled:opacity-60 disabled:cursor-wait ${
                      hasTriggered
                        ? "bg-emerald-500/30 text-emerald-100 hover:bg-emerald-500/50"
                        : "bg-black/50 text-white hover:bg-black/70"
                    }`}
                  >
                    {inventoryUpdatingId === message.message_id ? <Loader2 size={12} className="animate-spin" /> : <Package size={12} />}
                  </button>
                  <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/minv:opacity-100 pointer-events-none transition-opacity not-italic">
                    {hasTriggered ? "Inventory updated · run again" : "Update inventory from this moment"}
                  </span>
                </div>
              );
            })()}
            <div className="relative group/mdel">
              <button
                onClick={() => setShowDeleteConfirm(true)}
                className="w-7 h-7 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-destructive transition-colors backdrop-blur-sm"
              >
                <Trash2 size={12} />
              </button>
              <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mdel:opacity-100 pointer-events-none transition-opacity not-italic">Delete</span>
            </div>
          </div>
        )}

        {/* Adjusting overlay */}
        {adjustingMessageId === message.message_id && (
          <div className="absolute inset-0 rounded-xl bg-amber-950/80 backdrop-blur-sm flex items-center justify-center gap-2">
            <Loader2 size={14} className="animate-spin text-amber-400" />
            <span className="text-xs text-amber-400">Adjusting...</span>
          </div>
        )}

        <div style={{ fontSize: `${chatFontPx(chatFontSize)}px` }} className="prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:var(--color-amber-100)] [--tw-prose-bold:rgb(252,211,77)]">
          <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(message.content)}</Markdown>
        </div>
        <div className="not-italic">
          <InventoryUpdateBadge records={inventoryUpdateRecords} />
        </div>
        <p className="text-[10px] mt-1.5 text-amber-500/50 not-italic flex items-center gap-2">
          {new Date(message.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
          {!isPending && (
            <button
              onClick={() => onResetToHere(message.message_id)}
              className="opacity-0 group-hover:opacity-100 transition-opacity text-amber-500/40 hover:text-amber-400 cursor-pointer"
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
          <h3 className="font-semibold">Delete Narrative</h3>
        </div>
        <p className="text-sm text-muted-foreground">
          This will permanently delete this narrative message.
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
              onDelete(message.message_id);
            }}
          >
            Delete
          </Button>
        </div>
      </div>
    </Dialog>
  </>);
}

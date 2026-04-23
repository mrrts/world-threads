import { useEffect, useMemo, useRef, useState } from "react";
import { Sparkles, ScrollText, ImageOff } from "lucide-react";
import { type Message, api } from "@/lib/tauri";

interface Props {
  message: Message;
  /** Called when the user clicks the card — opens the modal scoped to this chapter. */
  onOpen: (chapterId: string) => void;
}

interface ParsedContent {
  chapter_id: string;
  title: string;
  image_id?: string;
  first_line?: string;
}

export function ImaginedChapterMessage({ message, onOpen }: Props) {
  const parsed = useMemo<ParsedContent | null>(() => {
    try {
      const obj = JSON.parse(message.content);
      if (obj && typeof obj.chapter_id === "string") return obj as ParsedContent;
    } catch {
      // Fallback for legacy / malformed rows.
    }
    return null;
  }, [message.content]);

  const [imageUrl, setImageUrl] = useState<string>("");
  const [imageFailed, setImageFailed] = useState(false);
  // Guard against setState after unmount when the async fetch resolves late.
  const aliveRef = useRef(true);
  useEffect(() => () => { aliveRef.current = false; }, []);

  useEffect(() => {
    if (!parsed?.chapter_id || !parsed.image_id) {
      setImageUrl("");
      setImageFailed(!parsed?.image_id);
      return;
    }
    setImageUrl("");
    setImageFailed(false);
    api.getImaginedChapterImageUrl(parsed.chapter_id)
      .then((url) => { if (aliveRef.current) setImageUrl(url); })
      .catch(() => { if (aliveRef.current) setImageFailed(true); });
  }, [parsed?.chapter_id, parsed?.image_id]);

  if (!parsed) {
    return (
      <div className="my-2 px-3 py-2 rounded-md border border-dashed border-amber-300/40 text-xs text-muted-foreground">
        [imagined chapter — unparseable content]
      </div>
    );
  }

  return (
    <div className="my-4 flex justify-center">
      <button
        type="button"
        onClick={() => onOpen(parsed.chapter_id)}
        title="Open this chapter"
        className="group w-full max-w-xl overflow-hidden rounded-xl border border-amber-400/50 bg-gradient-to-b from-amber-100/60 via-amber-50/40 to-amber-50/20 dark:from-amber-950/40 dark:via-amber-950/25 dark:to-amber-950/15 shadow-[0_0_18px_rgba(251,191,36,0.22)] hover:shadow-[0_0_28px_rgba(251,191,36,0.4)] hover:border-amber-400/80 transition-all text-left"
      >
        <div className="relative w-full aspect-[16/9] bg-amber-950/30 overflow-hidden">
          {imageUrl ? (
            <img
              src={imageUrl}
              alt={parsed.title || "Imagined chapter"}
              className="absolute inset-0 w-full h-full object-cover group-hover:scale-[1.02] transition-transform duration-500"
              draggable={false}
            />
          ) : imageFailed ? (
            <div className="absolute inset-0 flex items-center justify-center text-amber-400/50">
              <ImageOff size={32} />
            </div>
          ) : (
            <div className="absolute inset-0 animate-pulse bg-gradient-to-br from-amber-900/30 via-amber-800/20 to-amber-950/40" />
          )}
          {/* Gilded vignette so the title/label sits legibly over any image. */}
          <div className="absolute inset-0 bg-gradient-to-t from-amber-950/85 via-amber-950/20 to-transparent pointer-events-none" />
          <div className="absolute top-3 left-3 flex items-center gap-1.5 px-2 py-1 rounded-full bg-amber-950/60 backdrop-blur-sm border border-amber-400/40">
            <Sparkles size={11} className="text-amber-300 drop-shadow-[0_0_4px_rgba(251,191,36,0.7)]" />
            <span className="text-[10px] uppercase tracking-[0.15em] font-semibold text-amber-100">
              Imagined Chapter
            </span>
          </div>
          <div className="absolute bottom-3 left-4 right-4">
            <div className="text-lg sm:text-xl font-serif font-semibold text-amber-50 leading-tight drop-shadow-[0_2px_6px_rgba(0,0,0,0.7)] line-clamp-2">
              {parsed.title || "(untitled)"}
            </div>
          </div>
        </div>
        {parsed.first_line && (
          <div className="px-5 py-4 flex gap-3 items-start">
            <ScrollText size={14} className="text-amber-700/70 dark:text-amber-400/70 mt-0.5 shrink-0" />
            <p className="text-sm italic text-amber-950/85 dark:text-amber-100/85 leading-relaxed line-clamp-3">
              {parsed.first_line}…
            </p>
          </div>
        )}
        <div className="px-5 pb-3 pt-1 text-[10px] uppercase tracking-[0.18em] text-amber-800/60 dark:text-amber-400/60 font-medium">
          Open chapter →
        </div>
      </button>
    </div>
  );
}

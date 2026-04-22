import { useMemo, useState } from "react";
import { Sparkles, ChevronDown, ChevronRight } from "lucide-react";
import { type Message } from "@/lib/tauri";

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
  const [expanded, setExpanded] = useState(false);
  const parsed = useMemo<ParsedContent | null>(() => {
    try {
      const obj = JSON.parse(message.content);
      if (obj && typeof obj.chapter_id === "string") return obj as ParsedContent;
    } catch {
      // Fallback for legacy / malformed rows.
    }
    return null;
  }, [message.content]);

  if (!parsed) {
    return (
      <div className="my-2 px-3 py-2 rounded-md border border-dashed border-amber-300/40 text-xs text-muted-foreground">
        [imagined chapter — unparseable content]
      </div>
    );
  }

  return (
    <div className="my-2 flex justify-center">
      <div
        className="w-full max-w-md rounded-lg border border-amber-300/30 bg-amber-50/40 dark:bg-amber-950/20 dark:border-amber-700/30 overflow-hidden cursor-pointer hover:border-amber-400/50 dark:hover:border-amber-600/50 transition-colors"
        onClick={() => onOpen(parsed.chapter_id)}
        title="Open this chapter"
      >
        <div className="flex items-center gap-2 px-3 py-2 border-b border-amber-300/20 dark:border-amber-700/20 bg-amber-100/40 dark:bg-amber-900/30">
          <Sparkles size={13} className="text-amber-700 dark:text-amber-400 shrink-0" />
          <div className="flex-1 min-w-0">
            <div className="text-[10px] uppercase tracking-wide text-amber-800/70 dark:text-amber-400/70">Imagined Chapter</div>
            <div className="text-sm font-medium text-amber-950 dark:text-amber-100 truncate">
              {parsed.title || "(untitled)"}
            </div>
          </div>
          <button
            className="text-amber-700/60 dark:text-amber-400/60 hover:text-amber-700 dark:hover:text-amber-400"
            onClick={(e) => { e.stopPropagation(); setExpanded((v) => !v); }}
            title={expanded ? "Collapse" : "Show first line"}
          >
            {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
          </button>
        </div>
        {expanded && parsed.first_line && (
          <div className="px-3 py-2 text-xs italic text-amber-900/80 dark:text-amber-200/70 line-clamp-3">
            {parsed.first_line}…
          </div>
        )}
      </div>
    </div>
  );
}

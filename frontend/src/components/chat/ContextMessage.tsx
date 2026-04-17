import Markdown from "react-markdown";
import { Link2, SlidersHorizontal, Loader2 } from "lucide-react";
import { formatMessage, markdownComponents, remarkPlugins, rehypePlugins } from "./formatMessage";
import type { Message } from "@/lib/tauri";

interface Props {
  message: Message;
  isPending: boolean;
  onResetToHere: (id: string) => void;
  adjustingMessageId: string | null;
  onAdjust: (id: string) => void;
}

export function ContextMessage({ message, isPending, onResetToHere, adjustingMessageId, onAdjust }: Props) {
  return (
    <div key={message.message_id} data-message-id={message.message_id} className="flex justify-center my-2">
      <div className="relative group max-w-[90%] rounded-xl px-5 py-3.5 text-sm leading-relaxed bg-gradient-to-br from-sky-950/40 to-sky-900/20 border border-sky-700/30 text-sky-100/90 backdrop-blur-sm">
        <div className="flex items-center gap-1.5 mb-1.5 text-[10px] uppercase tracking-wider text-sky-500/70 font-semibold">
          <Link2 size={12} />
          <span>Cross-Chat Context</span>
        </div>

        {/* Adjust button */}
        {!isPending && (
          <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <div className="relative group/madj">
              <button
                onClick={() => onAdjust(message.message_id)}
                className="w-7 h-7 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm not-italic"
              >
                <SlidersHorizontal size={12} />
              </button>
              <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/madj:opacity-100 pointer-events-none transition-opacity not-italic">Adjust</span>
            </div>
          </div>
        )}

        {/* Adjusting overlay */}
        {adjustingMessageId === message.message_id && (
          <div className="absolute inset-0 rounded-xl bg-sky-950/80 backdrop-blur-sm flex items-center justify-center gap-2">
            <Loader2 size={14} className="animate-spin text-sky-400" />
            <span className="text-xs text-sky-400">Adjusting...</span>
          </div>
        )}

        <div className="prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:var(--color-sky-100)] [--tw-prose-bold:rgb(125,211,252)]">
          <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(message.content)}</Markdown>
        </div>
        <p className="text-[10px] mt-1.5 text-sky-500/50 flex items-center gap-2">
          {new Date(message.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
          {!isPending && (
            <button
              onClick={() => onResetToHere(message.message_id)}
              className="opacity-0 group-hover:opacity-100 transition-opacity text-sky-500/40 hover:text-sky-400 cursor-pointer"
            >
              Reset to Here
            </button>
          )}
        </p>
      </div>
    </div>
  );
}

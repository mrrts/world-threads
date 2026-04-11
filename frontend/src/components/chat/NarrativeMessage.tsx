import Markdown from "react-markdown";
import { BookOpen } from "lucide-react";
import { formatMessage } from "./formatMessage";
import type { Message } from "@/lib/tauri";

interface NarrativeMessageProps {
  message: Message;
  isPending: boolean;
  onResetToHere: (id: string) => void;
}

export function NarrativeMessage({ message, isPending, onResetToHere }: NarrativeMessageProps) {
  return (
    <div key={message.message_id} className="flex justify-center my-2">
      <div className="relative group max-w-[90%] rounded-xl px-5 py-3.5 text-sm leading-relaxed bg-gradient-to-br from-amber-950/40 to-amber-900/20 border border-amber-700/30 text-amber-100/90 italic backdrop-blur-sm">
        <div className="flex items-center gap-1.5 mb-1.5 text-[10px] uppercase tracking-wider text-amber-500/70 font-semibold not-italic">
          <BookOpen size={12} />
          <span>Narrative</span>
        </div>
        <div className="prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:var(--color-amber-100)] [--tw-prose-bold:rgb(252,211,77)]">
          <Markdown>{formatMessage(message.content)}</Markdown>
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
  );
}

import { useRef, useEffect } from "react";

export const QUICK_EMOJIS = ["❤️", "😂", "😮", "😢", "🔥", "👍", "👎", "💀", "🙏", "✨", "👀", "💯"];

export function EmojiPicker({
  onSelect,
  onClose,
}: {
  onSelect: (emoji: string) => void;
  onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [onClose]);

  return (
    <div
      ref={ref}
      className="absolute z-50 bg-card border border-border rounded-xl shadow-xl p-2 grid grid-cols-6 gap-1 w-[200px] animate-in fade-in zoom-in-95 duration-150"
    >
      {QUICK_EMOJIS.map((emoji) => (
        <button
          key={emoji}
          onClick={() => {
            onSelect(emoji);
            onClose();
          }}
          className="w-8 h-8 flex items-center justify-center text-lg rounded-lg hover:bg-accent transition-colors cursor-pointer"
        >
          {emoji}
        </button>
      ))}
    </div>
  );
}

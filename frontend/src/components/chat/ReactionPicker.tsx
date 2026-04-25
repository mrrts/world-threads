import { useEffect, useRef } from "react";
import EmojiPicker, { EmojiStyle, Theme } from "emoji-picker-react";

export function ReactionPicker({
  onPick,
  onClose,
  anchorRight = false,
}: {
  onPick: (emoji: string) => void;
  onClose: () => void;
  anchorRight?: boolean;
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
      className={`absolute z-50 ${anchorRight ? "right-0" : "left-0"} bottom-full mb-1.5 shadow-xl shadow-black/30 rounded-lg overflow-hidden`}
    >
      <EmojiPicker
        onEmojiClick={(data) => { onPick(data.emoji); onClose(); }}
        emojiStyle={EmojiStyle.NATIVE}
        theme={Theme.DARK}
        height={380}
        width={320}
        lazyLoadEmojis
        searchPlaceholder="Search emoji..."
        previewConfig={{ showPreview: false }}
      />
    </div>
  );
}

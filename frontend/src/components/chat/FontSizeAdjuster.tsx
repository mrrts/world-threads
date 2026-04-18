import { Minus, Plus } from "lucide-react";
import { CHAT_FONT_SIZES_PX } from "@/lib/chat-font";

interface Props {
  /** Current size level (0..CHAT_FONT_SIZES_PX.length - 1). */
  value: number;
  onChange: (next: number) => void;
}

/**
 * Three-segment control: [−] [AA] [+]. Minus/plus step the `value`
 * through CHAT_FONT_SIZES_PX and disable at the ends. The center AA icon
 * is non-clickable — it just communicates "font size" visually (small A
 * next to bigger A).
 */
export function FontSizeAdjuster({ value, onChange }: Props) {
  const max = CHAT_FONT_SIZES_PX.length - 1;
  const atMin = value <= 0;
  const atMax = value >= max;
  return (
    <div className="inline-flex rounded-lg overflow-hidden border border-input">
      <button
        type="button"
        onClick={() => onChange(Math.max(0, value - 1))}
        disabled={atMin}
        className="px-2.5 py-1.5 text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:text-muted-foreground"
        aria-label="Decrease chat font size"
      >
        <Minus size={14} />
      </button>
      <div
        className="px-3 py-1.5 flex items-baseline gap-0.5 text-muted-foreground border-l border-r border-input select-none"
        aria-hidden="true"
      >
        <span className="text-[11px] font-semibold leading-none">A</span>
        <span className="text-[16px] font-semibold leading-none">A</span>
      </div>
      <button
        type="button"
        onClick={() => onChange(Math.min(max, value + 1))}
        disabled={atMax}
        className="px-2.5 py-1.5 text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:text-muted-foreground"
        aria-label="Increase chat font size"
      >
        <Plus size={14} />
      </button>
    </div>
  );
}

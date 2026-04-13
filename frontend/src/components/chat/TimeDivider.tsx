import type { Message } from "@/lib/tauri";

interface Props {
  current: Message;
  previous: Message | undefined;
}

export function TimeDivider({ current, previous }: Props) {
  // Don't show if current message has no time info
  if (current.world_day == null && !current.world_time) return null;
  // Show if no previous message, or previous has no time info, or time differs
  if (previous && previous.world_day != null && previous.world_time
    && current.world_day === previous.world_day && current.world_time === previous.world_time) return null;

  const parts: string[] = [];
  if (current.world_day != null) parts.push(`Day ${current.world_day}`);
  if (current.world_time) {
    // Format nicely: "LATE NIGHT" → "Late Night", "MIDDAY" → "Midday"
    const formatted = current.world_time
      .split(" ")
      .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
      .join(" ");
    parts.push(formatted);
  }

  if (parts.length === 0) return null;

  return (
    <div className="flex items-center gap-5 my-8 px-4">
      <div className="flex-1 h-[2px] bg-border" />
      <span className="text-sm font-bold text-muted-foreground uppercase tracking-widest px-5 py-2 rounded-full bg-muted border border-border">
        {parts.join(" · ")}
      </span>
      <div className="flex-1 h-[2px] bg-border" />
    </div>
  );
}

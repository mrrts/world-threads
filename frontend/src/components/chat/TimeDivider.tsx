import type { Message } from "@/lib/tauri";

interface Props {
  current: Message;
  previous: Message | undefined;
}

export function TimeDivider({ current, previous }: Props) {
  if (!current.world_day && !current.world_time) return null;
  if (previous && current.world_day === previous.world_day && current.world_time === previous.world_time) return null;

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
    <div className="flex items-center gap-3 my-4 px-4">
      <div className="flex-1 h-px bg-border/50" />
      <span className="text-[10px] font-medium text-muted-foreground/50 uppercase tracking-wider">
        {parts.join(" · ")}
      </span>
      <div className="flex-1 h-px bg-border/50" />
    </div>
  );
}

import type { Reaction } from "@/lib/tauri";

export function ReactionBubbles({
  reactions,
  isUser,
}: {
  reactions: Reaction[];
  isUser: boolean;
}) {
  if (reactions.length === 0) return null;

  const grouped: Record<string, { emoji: string; reactors: string[] }> = {};
  for (const r of reactions) {
    if (!grouped[r.emoji]) grouped[r.emoji] = { emoji: r.emoji, reactors: [] };
    grouped[r.emoji].reactors.push(r.reactor);
  }

  return (
    <div className={`flex gap-1 mt-0.5 ${isUser ? "justify-end" : "justify-start"}`}>
      {Object.values(grouped).map(({ emoji, reactors }) => (
        <span
          key={emoji}
          className="inline-flex items-center gap-0.5 text-xs bg-secondary/80 border border-border rounded-full px-1.5 py-0.5 backdrop-blur-sm"
          title={reactors.map((r) => (r === "user" ? "You" : "Character")).join(", ")}
        >
          <span className="text-sm leading-none">{emoji}</span>
          {reactors.length > 1 && (
            <span className="text-[10px] text-muted-foreground">{reactors.length}</span>
          )}
        </span>
      ))}
    </div>
  );
}

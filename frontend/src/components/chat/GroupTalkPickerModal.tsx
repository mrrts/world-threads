import { Dialog } from "@/components/ui/dialog";
import type { Character, PortraitInfo } from "@/lib/tauri";

interface GroupTalkPickerModalProps {
  open: boolean;
  onClose: () => void;
  characters: Character[];
  portraits: Record<string, PortraitInfo>;
  onSelect: (characterId: string) => void;
}

export function GroupTalkPickerModal({
  open,
  onClose,
  characters,
  portraits,
  onSelect,
}: GroupTalkPickerModalProps) {
  return (
    <Dialog open={open} onClose={onClose} className="max-w-xs">
      <div className="p-5 space-y-3 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
        <h3 className="font-semibold text-sm">Who should speak?</h3>
        <div className="space-y-1.5">
          {characters.map((ch) => {
            const p = portraits[ch.character_id];
            return (
              <button
                key={ch.character_id}
                onClick={() => onSelect(ch.character_id)}
                className="flex items-center gap-3 w-full px-3 py-2.5 rounded-xl border border-border hover:border-primary/40 hover:bg-primary/5 transition-all cursor-pointer"
              >
                {p?.data_url ? (
                  <img src={p.data_url} alt="" className="w-10 h-10 rounded-full object-cover" />
                ) : (
                  <div className="w-10 h-10 rounded-full" style={{ backgroundColor: ch.avatar_color }} />
                )}
                <span className="text-sm font-medium">{ch.display_name}</span>
              </button>
            );
          })}
        </div>
      </div>
    </Dialog>
  );
}

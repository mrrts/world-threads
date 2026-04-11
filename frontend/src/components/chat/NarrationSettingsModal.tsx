import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { BookOpen, X } from "lucide-react";

interface NarrationSettingsModalProps {
  open: boolean;
  onClose: () => void;
  charId: string | undefined;
  narrationTone: string;
  setNarrationTone: (v: string) => void;
  narrationInstructions: string;
  setNarrationInstructions: (v: string) => void;
  responseLength: string;
  setResponseLength: (v: string) => void;
  narrationDirty: boolean;
  setNarrationDirty: (v: boolean) => void;
  onSave: () => void;
}

export function NarrationSettingsModal({
  open,
  onClose,
  narrationTone,
  setNarrationTone,
  narrationInstructions,
  setNarrationInstructions,
  responseLength,
  setResponseLength,
  narrationDirty,
  setNarrationDirty,
  onSave,
}: NarrationSettingsModalProps) {
  return (
    <Dialog open={open} onClose={onClose} className="max-w-md">
      <div className="p-5 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <BookOpen size={18} className="text-amber-500" />
            <h3 className="font-semibold">Narration Settings</h3>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
          >
            <X size={16} />
          </button>
        </div>

        <div className="space-y-3">
          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1.5">Tone</label>
            <select
              value={narrationTone}
              onChange={(e) => { setNarrationTone(e.target.value); setNarrationDirty(true); }}
              className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
            >
              {[
                "Auto",
                "Humorous", "Romantic", "Action & Adventure", "Dark & Gritty",
                "Suspenseful", "Whimsical", "Melancholic", "Heroic",
                "Horror", "Noir", "Surreal", "Cozy & Warm",
                "Tense & Paranoid", "Poetic", "Cinematic",
                "Mythic", "Playful", "Bittersweet", "Ethereal", "Gritty Realism",
              ].map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </div>

          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1.5">Response Length</label>
            <select
              value={responseLength}
              onChange={(e) => { setResponseLength(e.target.value); setNarrationDirty(true); }}
              className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
            >
              {["Auto", "Short", "Medium", "Long"].map((l) => (
                <option key={l} value={l}>{l}</option>
              ))}
            </select>
            <p className="text-[10px] text-muted-foreground mt-1">
              {responseLength === "Auto" && "The character decides how much to say."}
              {responseLength === "Short" && "Brief replies, 2\u20133 sentences."}
              {responseLength === "Medium" && "Moderate replies, 4\u20136 sentences."}
              {responseLength === "Long" && "Detailed replies, 7+ sentences with rich detail."}
            </p>
          </div>

          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1.5">Custom Instructions</label>
            <textarea
              value={narrationInstructions}
              onChange={(e) => { setNarrationInstructions(e.target.value); setNarrationDirty(true); }}
              placeholder="e.g. Describe the weather shifting. Include background characters reacting. Let the scene move to a new location..."
              className="w-full min-h-[100px] max-h-[200px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
              rows={4}
            />
          </div>
        </div>

        <div className="flex justify-end gap-2 pt-1">
          <Button
            variant="ghost"
            size="sm"
            onClick={onClose}
          >
            Cancel
          </Button>
          <Button
            size="sm"
            disabled={!narrationDirty}
            onClick={onSave}
          >
            Save
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

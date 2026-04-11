import { Dialog } from "@/components/ui/dialog";
import { Image, X } from "lucide-react";

interface IllustrationPickerModalProps {
  open: boolean;
  onClose: () => void;
  onGenerate: (tier: string) => void;
  illustrationInstructions: string;
  setIllustrationInstructions: (v: string) => void;
  usePreviousScene: boolean;
  setUsePreviousScene: (v: boolean) => void;
  includeSceneSummary: boolean;
  setIncludeSceneSummary: (v: boolean) => void;
  hasPreviousIllustration: boolean;
}

export function IllustrationPickerModal({
  open,
  onClose,
  onGenerate,
  illustrationInstructions,
  setIllustrationInstructions,
  usePreviousScene,
  setUsePreviousScene,
  includeSceneSummary,
  setIncludeSceneSummary,
  hasPreviousIllustration,
}: IllustrationPickerModalProps) {
  return (
    <Dialog open={open} onClose={onClose} className="max-w-sm">
      <div className="p-5 space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Image size={18} className="text-emerald-500" />
            <h3 className="font-semibold">Generate Illustration</h3>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
          >
            <X size={16} />
          </button>
        </div>
        <div>
          <label className="text-xs font-medium text-muted-foreground block mb-1.5">Custom Instructions (optional)</label>
          <textarea
            value={illustrationInstructions}
            onChange={(e) => setIllustrationInstructions(e.target.value)}
            placeholder="e.g. Show them outdoors in the rain. Frame it from a low angle..."
            className="w-full min-h-[60px] max-h-[120px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            rows={2}
          />
        </div>
        {hasPreviousIllustration && (
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={usePreviousScene}
              onChange={(e) => setUsePreviousScene(e.target.checked)}
              className="accent-emerald-500 w-3.5 h-3.5"
            />
            <span className="text-xs text-muted-foreground">Use previous illustration for visual continuity</span>
          </label>
        )}
        <label className="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            checked={includeSceneSummary}
            onChange={(e) => setIncludeSceneSummary(e.target.checked)}
            className="accent-emerald-500 w-3.5 h-3.5"
          />
          <span className="text-xs text-muted-foreground">Include current scene summary</span>
        </label>
        <div className="flex gap-2">
          {([
            { tier: "low", label: "Quick" },
            { tier: "medium", label: "Standard" },
            { tier: "high", label: "High Fidelity" },
          ] as const).map(({ tier, label }) => (
            <button
              key={tier}
              onClick={() => onGenerate(tier)}
              className="flex-1 rounded-lg border border-border hover:border-emerald-500/40 hover:bg-emerald-500/5 px-3 py-2 transition-all cursor-pointer text-center"
            >
              <span className="text-xs font-medium">{label}</span>
            </button>
          ))}
        </div>
      </div>
    </Dialog>
  );
}

import { useState } from "react";
import { Dialog } from "@/components/ui/dialog";
import { Image, X, ChevronDown } from "lucide-react";

interface RecentIllustration {
  id: string;
  content: string;
}

interface IllustrationPickerModalProps {
  open: boolean;
  onClose: () => void;
  onGenerate: (tier: string, selectedIllusId?: string) => void;
  illustrationInstructions: string;
  setIllustrationInstructions: (v: string) => void;
  usePreviousScene: boolean;
  setUsePreviousScene: (v: boolean) => void;
  includeSceneSummary: boolean;
  setIncludeSceneSummary: (v: boolean) => void;
  hasPreviousIllustration: boolean;
  previousIllustrationUrl?: string;
  /** Last 5 illustrations for reference picker */
  recentIllustrations: RecentIllustration[];
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
  previousIllustrationUrl,
  recentIllustrations,
}: IllustrationPickerModalProps) {
  const [showPicker, setShowPicker] = useState(false);
  const [selectedRef, setSelectedRef] = useState<RecentIllustration | null>(null);

  // The displayed reference image: selected override or the default latest
  const displayedUrl = selectedRef?.content ?? previousIllustrationUrl;
  const displayedId = selectedRef?.id ?? recentIllustrations[0]?.id;

  return (
    <Dialog open={open} onClose={() => { onClose(); setSelectedRef(null); setShowPicker(false); }} className="max-w-sm">
      <div className="p-5 space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Image size={18} className="text-emerald-500" />
            <h3 className="font-semibold">Generate Illustration</h3>
          </div>
          <button
            onClick={() => { onClose(); setSelectedRef(null); setShowPicker(false); }}
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
          <div>
            <label className="flex items-center gap-2 cursor-pointer select-none">
              <input
                type="checkbox"
                checked={usePreviousScene}
                onChange={(e) => setUsePreviousScene(e.target.checked)}
                className="accent-emerald-500 w-3.5 h-3.5"
              />
              <span className="text-xs text-muted-foreground">Use reference image for visual continuity</span>
            </label>
            {displayedUrl && (
              <div className="mt-2 relative">
                <img
                  src={displayedUrl}
                  alt="Reference illustration"
                  className="rounded-lg w-48 object-cover border border-border/30"
                />
                {recentIllustrations.length > 1 && (
                  <div className="relative mt-1.5">
                    <button
                      onClick={() => setShowPicker(!showPicker)}
                      className="text-xs text-muted-foreground/60 hover:text-muted-foreground transition-colors cursor-pointer flex items-center gap-1"
                    >
                      Change reference image
                      <ChevronDown size={12} className={`transition-transform ${showPicker ? "rotate-180" : ""}`} />
                    </button>
                    {showPicker && (
                      <div className="absolute left-0 top-full mt-1 z-10 flex gap-1.5 bg-card border border-border rounded-lg p-2 shadow-xl shadow-black/30 animate-in fade-in zoom-in-95 duration-150">
                        {recentIllustrations.map((illus) => (
                          <button
                            key={illus.id}
                            onClick={() => {
                              setSelectedRef(illus);
                              setUsePreviousScene(true);
                              setShowPicker(false);
                            }}
                            className={`flex-shrink-0 w-32 h-22 rounded-md overflow-hidden cursor-pointer transition-all ${
                              (selectedRef?.id ?? recentIllustrations[0]?.id) === illus.id
                                ? "ring-2 ring-emerald-500 ring-offset-1 ring-offset-card"
                                : "ring-1 ring-border opacity-70 hover:opacity-100"
                            }`}
                          >
                            <img src={illus.content} alt="" className="w-full h-full object-cover" />
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}
          </div>
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
              onClick={() => {
                const refId = usePreviousScene ? displayedId : undefined;
                onGenerate(tier, refId);
                setSelectedRef(null);
                setShowPicker(false);
              }}
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

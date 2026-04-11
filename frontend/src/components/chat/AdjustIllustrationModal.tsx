import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { SlidersHorizontal, X } from "lucide-react";

interface AdjustIllustrationModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: (instructions: string) => void;
  adjustInstructions: string;
  setAdjustInstructions: (v: string) => void;
}

export function AdjustIllustrationModal({
  open,
  onClose,
  onConfirm,
  adjustInstructions,
  setAdjustInstructions,
}: AdjustIllustrationModalProps) {
  return (
    <Dialog open={open} onClose={onClose} className="max-w-md">
      <div className="p-5 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <SlidersHorizontal size={18} className="text-emerald-500" />
            <h3 className="font-semibold">Adjust Illustration</h3>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
          >
            <X size={16} />
          </button>
        </div>

        <p className="text-xs text-muted-foreground">
          Describe what to change about the illustration. The current image will be used as a starting point.
        </p>

        <textarea
          value={adjustInstructions}
          onChange={(e) => setAdjustInstructions(e.target.value)}
          placeholder="e.g. Make it sunset instead of daytime. Add rain. Move the characters closer together..."
          className="w-full min-h-[100px] max-h-[200px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
          rows={4}
        />

        <div className="flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={onClose}>
            Cancel
          </Button>
          <Button
            size="sm"
            disabled={!adjustInstructions.trim()}
            onClick={() => onConfirm(adjustInstructions.trim())}
          >
            Adjust
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

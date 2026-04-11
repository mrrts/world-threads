import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { RotateCcw } from "lucide-react";

interface ResetConfirmModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  characterName?: string;
  isUserMessage: boolean;
  isGroup: boolean;
}

export function ResetConfirmModal({
  open,
  onClose,
  onConfirm,
  characterName,
  isUserMessage,
  isGroup,
}: ResetConfirmModalProps) {
  return (
    <Dialog open={open} onClose={onClose} className="max-w-sm">
      <div className="p-5 space-y-4">
        <div className="flex items-center gap-2">
          <RotateCcw size={18} className="text-destructive" />
          <h3 className="font-semibold">Reset to Here</h3>
        </div>
        <p className="text-sm text-muted-foreground">
          This will permanently delete all messages after this point, including their associated memories and embeddings.
          {isUserMessage && (
            <span className="block mt-1.5 text-foreground/80">A new response will be generated from {characterName}.</span>
          )}
        </p>
        <div className="flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            size="sm"
            onClick={onConfirm}
          >
            Reset
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

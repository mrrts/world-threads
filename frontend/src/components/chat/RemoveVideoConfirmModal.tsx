import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Video } from "lucide-react";

interface RemoveVideoConfirmModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

export function RemoveVideoConfirmModal({ open, onClose, onConfirm }: RemoveVideoConfirmModalProps) {
  return (
    <Dialog open={open} onClose={onClose} className="max-w-xs">
      <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
        <div className="flex items-center gap-2">
          <Video size={18} className="text-destructive" />
          <h3 className="font-semibold">Remove Video</h3>
        </div>
        <p className="text-sm text-muted-foreground">
          This will permanently delete the video attached to this illustration. The illustration itself will remain.
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
            Remove
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

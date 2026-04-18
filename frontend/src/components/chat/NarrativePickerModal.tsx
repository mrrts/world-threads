import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { BookOpen } from "lucide-react";

interface Props {
  open: boolean;
  onClose: () => void;
  onGenerate: (instructions?: string) => void;
}

export function NarrativePickerModal({ open, onClose, onGenerate }: Props) {
  const [instructions, setInstructions] = useState("");

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Narrative Beat</DialogTitle>
          <DialogDescription>Generate a narrative passage to advance the scene.</DialogDescription>
        </DialogHeader>
        <DialogBody>
          <p className="text-xs text-muted-foreground mb-2">Optionally direct the narrative:</p>
          <ul className="text-[11px] text-muted-foreground/70 mb-3 space-y-0.5 pl-4 list-disc">
            <li>Wrap up the scene at the church</li>
            <li>Transition the characters to the football game</li>
            <li>Add a surprising or humorous moment</li>
            <li>Describe the passage of time until evening</li>
          </ul>
          <textarea
            value={instructions}
            onChange={(e) => {
              setInstructions(e.target.value);
              e.target.style.height = "auto";
              e.target.style.height = `${e.target.scrollHeight}px`;
            }}
            ref={(el) => {
              if (el) { el.style.height = "auto"; el.style.height = `${el.scrollHeight}px`; }
            }}
            placeholder="Custom directions for this narrative..."
            className="w-full resize-none overflow-hidden rounded-lg border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            rows={4}
            autoFocus
          />
          <div className="flex justify-end gap-2 mt-3">
            <Button variant="ghost" size="sm" onClick={onClose}>
              Cancel
            </Button>
            <Button
              size="sm"
              onClick={() => {
                onGenerate(instructions.trim() || undefined);
                setInstructions("");
                onClose();
              }}
            >
              <BookOpen size={14} className="mr-1.5" />
              Generate
            </Button>
          </div>
        </DialogBody>
      </DialogContent>
    </Dialog>
  );
}

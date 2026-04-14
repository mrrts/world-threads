import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { SlidersHorizontal, Pencil } from "lucide-react";

interface Props {
  open: boolean;
  onClose: () => void;
  onAdjust: (instructions: string) => void;
  onEdit: (content: string) => void;
  characterName?: string;
  messageContent?: string;
  /** When true, only show the Edit tab (no Regenerate). */
  editOnly?: boolean;
}

export function AdjustMessageModal({ open, onClose, onAdjust, onEdit, characterName, messageContent, editOnly }: Props) {
  const [tab, setTab] = useState<"regenerate" | "edit">(editOnly ? "edit" : "regenerate");
  const [instructions, setInstructions] = useState("");
  const [editContent, setEditContent] = useState("");

  useEffect(() => {
    if (open) {
      if (messageContent != null) setEditContent(messageContent);
      setTab(editOnly ? "edit" : "regenerate");
    }
  }, [open, messageContent, editOnly]);

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{editOnly ? "Edit Message" : "Adjust Message"}</DialogTitle>
          <DialogDescription>
            {tab === "regenerate"
              ? `Tell ${characterName ?? "the character"} how to rewrite this message.`
              : "Edit the message text directly."}
          </DialogDescription>
        </DialogHeader>
        {/* Tabs (hidden in edit-only mode) */}
        {!editOnly && (
          <div className="flex border-b border-border px-6">
            <button
              onClick={() => setTab("regenerate")}
              className={`px-4 py-2 text-sm font-medium transition-colors cursor-pointer ${
                tab === "regenerate"
                  ? "text-foreground border-b-2 border-primary"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              Regenerate
            </button>
            <button
              onClick={() => setTab("edit")}
              className={`px-4 py-2 text-sm font-medium transition-colors cursor-pointer ${
                tab === "edit"
                  ? "text-foreground border-b-2 border-primary"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              Edit
            </button>
          </div>
        )}
        <DialogBody>
          {tab === "regenerate" ? (
            <>
              <textarea
                value={instructions}
                onChange={(e) => setInstructions(e.target.value)}
                placeholder="e.g. Make it more enthusiastic, shorten it, add a joke, change the tone to be more serious..."
                className="w-full min-h-[80px] max-h-[160px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                rows={3}
                autoFocus
              />
              <div className="flex justify-end gap-2 mt-3">
                <Button variant="ghost" size="sm" onClick={onClose}>
                  Cancel
                </Button>
                <Button
                  size="sm"
                  disabled={!instructions.trim()}
                  onClick={() => {
                    onAdjust(instructions.trim());
                    setInstructions("");
                    onClose();
                  }}
                >
                  <SlidersHorizontal size={14} className="mr-1.5" />
                  Adjust
                </Button>
              </div>
            </>
          ) : (
            <>
              <textarea
                value={editContent}
                onChange={(e) => setEditContent(e.target.value)}
                className="w-full min-h-[200px] max-h-[400px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                rows={10}
                autoFocus
              />
              <div className="flex justify-end gap-2 mt-3">
                <Button variant="ghost" size="sm" onClick={onClose}>
                  Cancel
                </Button>
                <Button
                  size="sm"
                  disabled={!editContent.trim() || editContent === messageContent}
                  onClick={() => {
                    onEdit(editContent);
                    onClose();
                  }}
                >
                  <Pencil size={14} className="mr-1.5" />
                  Update
                </Button>
              </div>
            </>
          )}
        </DialogBody>
      </DialogContent>
    </Dialog>
  );
}

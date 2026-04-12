import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";

interface Props {
  open: boolean;
  onClose: () => void;
  title: string;
  generateSummary: () => Promise<string>;
}

export function SummaryModal({ open, onClose, title, generateSummary }: Props) {
  const [summary, setSummary] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setSummary(null);
    setError(null);
    setLoading(true);
    generateSummary()
      .then((s) => setSummary(s))
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [open]);

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>On-demand summary of the current conversation.</DialogDescription>
        </DialogHeader>
        <DialogBody>
          {loading ? (
            <div className="flex items-center justify-center py-8 gap-3">
              <Loader2 size={18} className="animate-spin text-muted-foreground" />
              <span className="text-sm text-muted-foreground">Generating summary...</span>
            </div>
          ) : error ? (
            <div className="text-sm text-destructive py-4">{error}</div>
          ) : summary ? (
            <p className="text-sm text-foreground leading-relaxed whitespace-pre-wrap">{summary}</p>
          ) : null}
          <div className="flex justify-end mt-4">
            <Button variant="ghost" size="sm" onClick={onClose}>Close</Button>
          </div>
        </DialogBody>
      </DialogContent>
    </Dialog>
  );
}

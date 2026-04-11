import { Button } from "@/components/ui/button";
import { X, Check, Copy } from "lucide-react";

interface ChatErrorBarProps {
  error: string | null;
  lastFailedContent: string | null;
  isSending: boolean;
  onRetry: () => void;
  onCopy: () => void;
  onDismiss: () => void;
  copiedError: boolean;
}

export function ChatErrorBar({
  error,
  lastFailedContent,
  isSending,
  onRetry,
  onCopy,
  onDismiss,
  copiedError,
}: ChatErrorBarProps) {
  if (!error) return null;

  return (
    <div className="px-4 py-2.5 bg-background border-t border-destructive/30 flex items-center gap-3 relative z-10">
      <div className="flex-1 min-w-0">
        <p className="text-xs text-destructive font-medium truncate">{error}</p>
      </div>
      <button
        onClick={onCopy}
        className="flex-shrink-0 text-destructive/60 hover:text-destructive transition-colors cursor-pointer"
        title="Copy full error"
      >
        {copiedError ? <Check size={14} /> : <Copy size={14} />}
      </button>
      {lastFailedContent && (
        <Button
          size="sm"
          variant="outline"
          className="flex-shrink-0 border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
          onClick={onRetry}
          disabled={isSending}
        >
          Try Again
        </Button>
      )}
      <button
        onClick={onDismiss}
        className="flex-shrink-0 text-destructive/60 hover:text-destructive transition-colors cursor-pointer"
      >
        <X size={14} />
      </button>
    </div>
  );
}

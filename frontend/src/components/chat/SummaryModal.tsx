import { useState, useEffect, useRef } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Loader2, RotateCw } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { playChime } from "@/lib/chime";
import { CyclingLoadingMessages } from "@/components/ui/cycling-loading-messages";
import type { SummaryMode } from "@/lib/tauri";

// Playful flavor for the summary loading state (local-model prompt ingest).
// First entry is what the reader sees first.
const SUMMARY_LOADING_MESSAGES = [
  "Skimming the conversation...",
  "Finding the through-line...",
  "Piecing it together...",
  "Catching up...",
  "Noticing what mattered...",
  "Tracing the arc...",
  "Gathering the threads...",
  "Sorting the beats...",
  "Pulling it into focus...",
  "Highlighting the key moments...",
  "Listening for the subtext...",
  "Mapping what happened...",
  "Revisiting the highlights...",
  "Finding the shape of it...",
  "Settling on what matters...",
  "Re-reading one more time...",
  "Jotting notes in the margin...",
  "Cross-referencing the moments...",
  "Remembering how it started...",
  "Putting it into plain words...",
];

interface Props {
  open: boolean;
  onClose: () => void;
  title: string;
  /** The summary generator. When `enableModeSelector` is true, the modal
   *  passes the selected mode in. Callers that don't honor a mode can
   *  ignore the argument. */
  generateSummary: (mode?: SummaryMode) => Promise<string>;
  notifyOnMessage?: boolean;
  /** When true, show a 3-segment Short/Medium/Auto selector and let the
   *  user re-generate at a different length. Defaults to true. */
  enableModeSelector?: boolean;
  /** Initial mode used for the auto-fire on open. Defaults to "short". */
  defaultMode?: SummaryMode;
}

export function SummaryModal({
  open, onClose, title, generateSummary, notifyOnMessage,
  enableModeSelector = true,
  defaultMode = "short",
}: Props) {
  const [mode, setMode] = useState<SummaryMode>(defaultMode);
  // Bump this to force the effect to re-run without changing the mode —
  // used by the explicit "re-run with same mode" affordance.
  const [runNonce, setRunNonce] = useState(0);
  const [summary, setSummary] = useState<string | null>(null);
  // Initial-render-safe: if the modal opens already open, loading is true on
  // the very first paint — otherwise there's a one-render flash of empty
  // body before useEffect fires and flips loading=true.
  const [loading, setLoading] = useState(() => open);
  const [error, setError] = useState<string | null>(null);

  // Keep the streaming body scrolled to the bottom as tokens land, unless
  // the user has deliberately scrolled up to re-read.
  const streamRef = useRef<HTMLDivElement | null>(null);
  const userScrolledUpRef = useRef(false);
  useEffect(() => {
    const el = streamRef.current;
    if (!el) return;
    if (userScrolledUpRef.current) return;
    el.scrollTop = el.scrollHeight;
  }, [summary]);

  // Reset mode to the default when modal closes so each open starts
  // from the same baseline (Short by default — cheapest on tokens).
  useEffect(() => {
    if (!open) setMode(defaultMode);
  }, [open, defaultMode]);

  // Run on open AND on subsequent mode changes (so re-generating with a
  // different length just works without a manual button).
  useEffect(() => {
    if (!open) return;
    setSummary("");
    setError(null);
    setLoading(true);

    let unlisten: (() => void) | null = null;
    let cancelled = false;

    (async () => {
      let chimePlayed = false;
      unlisten = await listen<string>("summary-token", (event) => {
        if (cancelled) return;
        if (!chimePlayed && notifyOnMessage) { playChime(); chimePlayed = true; }
        setSummary((prev) => (prev ?? "") + event.payload);
      });
      try {
        const result = await generateSummary(enableModeSelector ? mode : undefined);
        if (!cancelled) setSummary(result);
      } catch (e) {
        if (!cancelled) setError(String(e));
      } finally {
        if (!cancelled) setLoading(false);
        unlisten?.();
      }
    })();

    return () => { cancelled = true; unlisten?.(); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, mode, runNonce]);

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>On-demand summary of the current conversation.</DialogDescription>
        </DialogHeader>
        <DialogBody>
          {enableModeSelector && (
            <div className="flex items-center justify-between mb-3 gap-2">
              <div className="inline-flex rounded-lg overflow-hidden border border-input">
                {(["short", "medium", "auto"] as const).map((m) => (
                  <button
                    key={m}
                    type="button"
                    disabled={loading}
                    onClick={() => setMode(m)}
                    className={`px-3 py-1.5 text-xs transition-colors cursor-pointer disabled:cursor-not-allowed ${
                      mode === m
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                    } ${m === "medium" ? "border-l border-r border-input" : ""}`}
                  >
                    {m.charAt(0).toUpperCase() + m.slice(1)}
                  </button>
                ))}
              </div>
              {/* Re-fire the same mode (useful when the model produced something the user wants redone). */}
              <button
                type="button"
                disabled={loading}
                onClick={() => setRunNonce((n) => n + 1)}
                className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground disabled:opacity-40 disabled:cursor-not-allowed"
                title="Re-generate with the same mode"
              >
                <RotateCw size={12} className={loading ? "animate-spin" : ""} />
                <span>Re-run</span>
              </button>
            </div>
          )}
          {loading && !summary ? (
            <div className="flex flex-col items-center justify-center py-8 gap-3">
              <Loader2 size={20} className="animate-spin text-muted-foreground" />
              <span className="text-sm text-muted-foreground">
                <CyclingLoadingMessages messages={SUMMARY_LOADING_MESSAGES} />
              </span>
            </div>
          ) : error ? (
            <div className="text-sm text-destructive py-4">{error}</div>
          ) : !loading && !summary ? (
            // Request completed but returned an empty string — don't leave
            // the modal blank. Surfaces a clear hint instead of silent void.
            <div className="text-sm text-muted-foreground py-6 text-center">
              The model returned no summary. Try again?
            </div>
          ) : summary ? (
            <div
              ref={streamRef}
              onScroll={(e) => {
                const el = e.currentTarget;
                // If the user pulls away from the bottom (>40px slack),
                // stop auto-scrolling so they can re-read. Sticking back to
                // the bottom re-engages auto-scroll.
                const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
                userScrolledUpRef.current = !atBottom;
              }}
              className="text-sm text-foreground leading-relaxed whitespace-pre-wrap max-h-[50vh] overflow-y-auto pr-2"
            >
              {summary}{loading ? <span className="inline-block w-1.5 h-4 bg-primary/60 animate-pulse ml-0.5 align-text-bottom" /> : null}
            </div>
          ) : null}
          <div className="flex justify-end mt-4">
            <Button variant="ghost" size="sm" onClick={onClose}>Close</Button>
          </div>
        </DialogBody>
      </DialogContent>
    </Dialog>
  );
}

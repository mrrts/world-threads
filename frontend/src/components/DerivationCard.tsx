import { useState, useEffect } from "react";
import Markdown from "react-markdown";
import { Copy, Check, RefreshCw, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { markdownComponents, remarkPlugins, rehypePlugins } from "./chat/formatMessage";

/**
 * Convert LaTeX-style display-math delimiters \[ ... \] and inline \( ... \)
 * into the dollar-sign delimiters that remark-math expects ($$ ... $$ for
 * block, $ ... $ for inline). The Copy-raw button preserves the original
 * \[...\] text — this conversion is render-side only.
 *
 * Why: stored derivations were authored with the LaTeX-canonical \[...\]
 * delimiters, but remark-math by default only parses dollar-delimited math.
 * Rather than re-storing the data or switching to a non-default remark-math
 * config, this small conversion at render-time keeps the stored format
 * LaTeX-canonical (what an LLM expects to receive when the user copies it)
 * while still rendering prettily through the existing pipeline.
 */
function normalizeMathDelimiters(text: string): string {
  return text
    .replace(/\\\[([\s\S]+?)\\\]/g, (_m, inner) => `\n$$${inner}$$\n`)
    .replace(/\\\(([\s\S]+?)\\\)/g, (_m, inner) => `$${inner}$`);
}

interface Props {
  /** "Character Formula" or "World Formula" */
  label: string;
  /** Loader function returning the stored derivation text (or null if not set). */
  load: () => Promise<string | null>;
  /** Re-fetch dependency — pass character_id or world_id so the component
   * refreshes when the editor switches subject. */
  refetchKey: string;
  /** Optional regenerate callback. When provided, a "Regenerate" button
   * appears in the header. Implementations should call the appropriate
   * Tauri command (regenerate_character_derivation_cmd /
   * regenerate_user_derivation_cmd / etc.) and refresh the parent's
   * data so the card receives the new text on next render. */
  onRegenerate?: () => Promise<void>;
}

/**
 * Read-only documentary rendering of an entity's derived_formula. The stored
 * text is markdown with KaTeX-renderable display-math blocks (\[ ... \]) at
 * the top and plain-English summary bullets below. Renders both via the
 * existing chat-message markdown pipeline (rehype-katex + remark-math) so
 * formula and prose flow through the same typography rules.
 *
 * Includes a Copy button that copies the RAW derivation text (including the
 * LaTeX sources) to clipboard, so the user can paste into another LLM to
 * tune that LLM to this character/world's register.
 *
 * Editing is intentionally not provided here — per the auto-derivation
 * feature design discipline (.claude/memory/feedback_auto_derivation_
 * design_discipline.md), editable UI requires careful user-friendly design
 * that's not too LaTeX-y, and is deferred. Authoring happens via worldcli
 * derive-character / derive-world for now.
 */
export function DerivationCard({ label, load, refetchKey, onRegenerate }: Props) {
  const [text, setText] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [copied, setCopied] = useState(false);
  const [regenerating, setRegenerating] = useState(false);
  const [regenError, setRegenError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    load().then((t) => {
      if (cancelled) return;
      setText(t);
      setLoading(false);
    }).catch(() => {
      if (cancelled) return;
      setText(null);
      setLoading(false);
    });
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [refetchKey]);

  if (loading) return null;
  if (!text || text.trim().length === 0) {
    // No derivation yet, but if regenerate is available, show a small
    // "Generate one now" affordance so the card surface is discoverable.
    if (!onRegenerate) return null;
    return (
      <div className="mb-6 rounded-xl border-2 border-dashed border-primary/30 bg-primary/5 px-5 py-4">
        <div className="flex items-center justify-between gap-3">
          <div className="flex flex-col">
            <h3 className="text-[11px] font-bold uppercase tracking-[0.15em] text-primary/90">{label}</h3>
            <span className="text-[11px] italic text-primary/60 mt-0.5">— not yet derived</span>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={async () => {
              setRegenerating(true);
              setRegenError(null);
              try { await onRegenerate(); } catch (e: any) {
                setRegenError(typeof e === "string" ? e : (e?.message ?? "Regenerate failed"));
              } finally { setRegenerating(false); }
            }}
            disabled={regenerating}
            className="h-8 text-xs gap-1.5 border-primary/40 text-primary hover:bg-primary/10"
          >
            {regenerating ? <Loader2 className="h-3 w-3 animate-spin" /> : <RefreshCw className="h-3 w-3" />}
            {regenerating ? "Generating…" : "Generate"}
          </Button>
        </div>
        {regenError && (
          <p className="mt-2 text-xs text-destructive bg-destructive/10 rounded px-2 py-1">{regenError}</p>
        )}
      </div>
    );
  }

  const onCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* ignore */
    }
  };

  return (
    <div className="mb-6 rounded-xl border-2 border-primary/40 bg-gradient-to-br from-amber-50/50 via-amber-50/40 to-rose-50/50 dark:from-amber-950/25 dark:via-amber-950/20 dark:to-rose-950/25 px-5 py-4 shadow-md">
      <div className="mb-3 flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 flex-1 min-w-0">
          <h3 className="text-[11px] font-bold uppercase tracking-[0.15em] text-primary/90">
            {label}
          </h3>
          <span className="text-[10px] italic text-primary/60 truncate">
            — a one-of-a-kind shorthand of how characters in this world hold you
          </span>
        </div>
        <div className="flex items-center gap-1 flex-shrink-0">
          {onRegenerate && (
            <Button
              variant="ghost"
              size="sm"
              onClick={async () => {
                setRegenerating(true);
                setRegenError(null);
                try { await onRegenerate(); } catch (e: any) {
                  setRegenError(typeof e === "string" ? e : (e?.message ?? "Regenerate failed"));
                } finally { setRegenerating(false); }
              }}
              disabled={regenerating}
              className="h-7 gap-1.5 text-xs text-primary/80 hover:text-primary hover:bg-primary/10"
              title="Regenerate the derivation from this entity's substrate + recent corpus"
            >
              {regenerating ? <Loader2 className="h-3 w-3 animate-spin" /> : <RefreshCw className="h-3 w-3" />}
              {regenerating ? "Regenerating…" : "Regenerate"}
            </Button>
          )}
          <Button
            variant="ghost"
            size="sm"
            onClick={onCopy}
            className="h-7 gap-1.5 text-xs text-primary/80 hover:text-primary hover:bg-primary/10"
            title="Copy raw derivation (LaTeX + plain English) to paste into another LLM"
          >
            {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
            {copied ? "Copied" : "Copy raw"}
          </Button>
        </div>
      </div>
      {regenError && (
        <p className="mb-2 text-xs text-destructive bg-destructive/10 rounded px-2 py-1">{regenError}</p>
      )}
      <div className="prose prose-sm max-w-none text-foreground/95">
        <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>
          {normalizeMathDelimiters(text)}
        </Markdown>
      </div>
    </div>
  );
}

import { useState, useEffect } from "react";
import Markdown from "react-markdown";
import { Copy, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { markdownComponents, remarkPlugins, rehypePlugins } from "./chat/formatMessage";

interface Props {
  /** "Character Formula" or "World Formula" */
  label: string;
  /** Loader function returning the stored derivation text (or null if not set). */
  load: () => Promise<string | null>;
  /** Re-fetch dependency — pass character_id or world_id so the component
   * refreshes when the editor switches subject. */
  refetchKey: string;
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
export function DerivationCard({ label, load, refetchKey }: Props) {
  const [text, setText] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [copied, setCopied] = useState(false);

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
  if (!text || text.trim().length === 0) return null;

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
    <div className="mb-6 rounded-lg border border-border bg-muted/30 p-4">
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          {label}
        </h3>
        <Button
          variant="ghost"
          size="sm"
          onClick={onCopy}
          className="h-7 gap-1.5 text-xs text-muted-foreground hover:text-foreground"
          title="Copy raw derivation (LaTeX + plain English) to paste into another LLM"
        >
          {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
          {copied ? "Copied" : "Copy raw"}
        </Button>
      </div>
      <div className="prose prose-sm max-w-none text-foreground">
        <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>
          {text}
        </Markdown>
      </div>
    </div>
  );
}

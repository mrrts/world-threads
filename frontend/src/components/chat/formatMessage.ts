import React from "react";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import "katex/dist/katex.min.css";

/** Convert parenthesized text to asterisked text for italic/em rendering in Markdown. */
export function formatMessage(content: string): string {
  const stripped = stripAsteriskWrappedQuotes(content);
  const unembedded = splitSpokenLinesOutOfEm(stripped);
  return unembedded
    .replace(/\[[^\]]*\]:?\s*/g, "")
    .replace(/\(([^)]+)\)/g, "*$1*")
    .replace(/\n\s*[-*]\s*$/, "");
}

/**
 * Strip `*"..."*` patterns where asterisks directly wrap a quoted phrase
 * with nothing but optional whitespace between the asterisks and the
 * quotes. The inner `"..."` is preserved verbatim.
 *
 * Matches the backend `strip_asterisk_wrapped_quotes` in orchestrator.rs
 * so already-persisted messages written under the old bug render clean
 * without a data rewrite. Action beats like `*says "stop"*` are NOT
 * matched — they contain non-quote content inside the asterisk pair.
 *
 * Critically, the match requires the opening `*` to sit at a left-flanking
 * position (start-of-string or preceded by whitespace) and the closing `*`
 * to sit at a right-flanking position (end-of-string, whitespace, or
 * sentence punctuation). Without these flanking constraints the regex
 * would greedily span across adjacent em blocks, eating the closing `*`
 * of one action + the opening `*` of the next action when there's a
 * quoted line between them — turning two separate ems into one giant em
 * that swallows the quote.
 */
/**
 * Pull spoken lines out of action blocks that forgot to close.
 *
 * Failure mode: the model writes `*action. "Spoken line." more action.*` —
 * one giant em block swallowing what should have been a spoken line. The
 * whole thing renders italic / blockquote-styled, when only the action
 * parts should.
 *
 * Fix: for each `*...*` block, find any embedded `"..."` that looks like
 * a spoken sentence (ends in sentence-terminating punctuation inside the
 * quote). Split the em at that boundary so the quote becomes plain text
 * between two smaller ems.
 *
 * Conservative: only triggers on quotes that end with sentence-terminating
 * punctuation (`.`, `!`, `?`, `…`) OR a trailing em/en-dash (`—`, `–`) —
 * the latter handles dialogue that trails off into resumed action
 * (`"...And here—" I touch the margin.`). Inline quoted words like
 * `*he said "stop"*` (no terminator) are left alone.
 */
function splitSpokenLinesOutOfEm(s: string): string {
  return s.replace(/\*([^*]+)\*/g, (full, inner: string) => {
    // Split inner on spoken-line quotes. Alternating: non-quote, quote, non-quote, ...
    const parts = inner.split(/("[^"\n]*[.!?…—–][ \t]*")/g);
    if (parts.length <= 1) return full; // no spoken-line quotes embedded
    const out: string[] = [];
    for (let i = 0; i < parts.length; i++) {
      const piece = parts[i].trim();
      if (!piece) continue;
      if (i % 2 === 1) {
        // Captured spoken-line quote — emit as plain text.
        out.push(piece);
      } else {
        // Non-quote action fragment — re-wrap in asterisks.
        out.push(`*${piece}*`);
      }
    }
    return out.join(" ");
  });
}

function stripAsteriskWrappedQuotes(s: string): string {
  // (^|\s)              anchor: start of string OR a whitespace char
  // \*                  opening asterisk
  // [ \t]*              optional horizontal whitespace
  // ("[^"\n]*")         a complete double-quoted phrase (captured)
  // [ \t]*              optional horizontal whitespace
  // \*                  closing asterisk
  // (?=\s|$|[.,!?;:])   lookahead: end, whitespace, or sentence punctuation
  return s.replace(
    /(^|\s)\*[ \t]*("[^"\n]*")[ \t]*\*(?=\s|$|[.,!?;:])/g,
    "$1$2",
  );
}

/** Extract plain text from React children (handles strings, arrays, nested elements). */
function extractText(children: React.ReactNode): string {
  if (typeof children === "string") return children;
  if (typeof children === "number") return String(children);
  if (Array.isArray(children)) return children.map(extractText).join("");
  if (React.isValidElement(children) && children.props?.children) {
    return extractText(children.props.children);
  }
  return "";
}

/**
 * Custom Markdown components for message rendering.
 * Single-word emphasis → inline <i> (just italic)
 * Multi-word emphasis → <em> (picks up block/border styling from parent CSS)
 */
export const markdownComponents = {
  em: ({ children }: { children?: React.ReactNode }) => {
    const text = extractText(children).trim();
    const isSingleWord = text.length > 0 && !text.includes(" ");
    if (isSingleWord) {
      return React.createElement("i", { className: "italic" }, children);
    }
    return React.createElement("em", null, children);
  },
};

/** Remark plugins for Markdown rendering (includes math support) */
export const remarkPlugins = [remarkMath];

/** Rehype plugins for Markdown rendering (includes KaTeX rendering) */
export const rehypePlugins = [rehypeKatex];

import React from "react";

/** Convert parenthesized text to asterisked text for italic/em rendering in Markdown. */
export function formatMessage(content: string): string {
  return content
    .replace(/\[[^\]]*\]:?\s*/g, "")
    .replace(/\(([^)]+)\)/g, "*$1*")
    .replace(/\n\s*[-*]\s*$/, "");
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

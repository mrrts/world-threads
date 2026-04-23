import React from "react";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import "katex/dist/katex.min.css";
import {
  ArrowLeft,
  BookOpen,
  Camera,
  ChevronDown,
  Compass,
  Database,
  Download,
  Feather,
  Image as ImageIcon,
  Loader2,
  PanelLeft,
  Pencil,
  Plus,
  RotateCw,
  Save,
  ScrollText,
  Send,
  Settings,
  SmilePlus,
  Sparkles,
  Trash2,
  Volume2,
  Package,
  Wallpaper,
  X,
  Sliders,
  Wand2,
  type LucideIcon,
} from "lucide-react";

/**
 * Detect whether a message body is emoji-only (one or a few emoji, nothing
 * else except whitespace / variation selectors / ZWJ / skin-tone or flag
 * modifiers). Used by the chat renderer to jumbo-size such replies the
 * way iMessage does when a short emoji-only reply carries the whole beat.
 * The "One emoji, rarely" craft note instructs characters to sometimes
 * reply with a single emoji; the jumbo rendering makes that land.
 *
 * Matches: "🙂", "❤️", "👋🏽", "👨‍👩‍👧", "🎉🎊". Rejects: "hi 🙂",
 * "🙂!", "(laughs) 🙂". Empty / whitespace-only strings return false.
 */
export function isEmojiOnlyMessage(content: string): boolean {
  const trimmed = content.trim();
  if (trimmed.length === 0) return false;
  // Strip every emoji-family character + whitespace. If anything is left,
  // the message is not emoji-only.
  const stripped = trimmed.replace(
    /[\p{Extended_Pictographic}\u200D\uFE0F\u{1F3FB}-\u{1F3FF}\u{1F1E6}-\u{1F1FF}\s]/gu,
    "",
  );
  if (stripped.length > 0) return false;
  // Must contain at least one actual pictographic codepoint — otherwise
  // something like "     " would have matched after stripping.
  return /\p{Extended_Pictographic}/u.test(trimmed);
}

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
 *
 * Both are rendered at 75% opacity so action beats / stage directions
 * sit slightly quieter than spoken dialogue. Keeps the contrast tonal
 * rather than chromatic, so it works across every bubble color (primary
 * user, secondary assistant, amber narrative).
 */
export const markdownComponents = {
  em: ({ children }: { children?: React.ReactNode }) => {
    const text = extractText(children).trim();
    const isSingleWord = text.length > 0 && !text.includes(" ");
    if (isSingleWord) {
      return React.createElement("i", { className: "italic opacity-65" }, children);
    }
    return React.createElement("em", { className: "opacity-65" }, children);
  },
};

/**
 * Inline icon catalog for the consultant. The Backstage prompt teaches
 * the model to write `[icon:Name]` inline (e.g. "click the [icon:Sparkles]
 * Imagine button"); we transform those tokens into actual Lucide icons
 * at render time.
 *
 * Only the names below are honored — unknown names fall back to plain
 * text so the assistant can't spray broken icons by guessing. Names are
 * case-sensitive on purpose so the prompt's catalog reads as the source
 * of truth.
 */
export const CONSULTANT_ICON_MAP: Record<string, LucideIcon> = {
  Settings,
  Sparkles,
  Compass,
  Image: ImageIcon,
  Gallery: ImageIcon,
  ScrollText,
  Canon: ScrollText,
  Keep: ScrollText,
  BookOpen,
  Plus,
  Pencil,
  Edit: Pencil,
  Trash: Trash2,
  Trash2,
  Download,
  Send,
  Volume: Volume2,
  Speak: Volume2,
  Package,
  Inventory: Package,
  Sidebar: PanelLeft,
  PanelLeft,
  SmilePlus,
  Reaction: SmilePlus,
  Wallpaper,
  Background: Wallpaper,
  Save,
  Database,
  Backup: Database,
  Camera,
  Feather,
  Imagine: Sparkles,
  Consultant: Compass,
  ChevronDown,
  ArrowLeft,
  Loader: Loader2,
  RotateCw,
  X,
  Sliders,
  Tone: Sliders,
  Wand: Wand2,
  Generate: Wand2,
};

/**
 * Replace `[icon:Name]` tokens in consultant message text with backticked
 * sentinels (`` `icon:Name` ``) so they survive markdown processing as
 * inline `<code>` nodes — which the consultant `code` component below
 * intercepts and renders as the actual Lucide icon. Unknown names are
 * left as plain text so a typo doesn't produce a broken/empty icon.
 */
export function transformConsultantIcons(text: string): string {
  return text.replace(/\[icon:([A-Za-z][A-Za-z0-9_]*)\]/g, (full, name: string) => {
    if (CONSULTANT_ICON_MAP[name]) return `\`icon:${name}\``;
    return full;
  });
}

/// Render an `icon:Name` inline-code node as the matching Lucide icon.
/// Falls through to a normal `<code>` element for any other inline-code
/// content so real backticked snippets still render as code.
function consultantInlineCode({ children }: { children?: React.ReactNode }) {
  const text = extractText(children);
  const match = /^icon:([A-Za-z][A-Za-z0-9_]*)$/.exec(text.trim());
  if (match) {
    const Icon = CONSULTANT_ICON_MAP[match[1]];
    if (Icon) {
      return React.createElement(
        "span",
        {
          className:
            "inline-flex items-center justify-center align-text-bottom w-[1em] h-[1em] mx-[0.15em] text-current opacity-90",
          "aria-label": match[1],
          title: match[1],
        },
        React.createElement(Icon, { size: "1em", strokeWidth: 2 }),
      );
    }
  }
  return React.createElement(
    "code",
    { className: "px-1 py-0.5 rounded bg-muted/40 text-[0.9em]" },
    children,
  );
}

/**
 * Variant of markdownComponents for the StoryConsultant. Adds inline-icon
 * rendering (intercepts `<code>` nodes that match the icon sentinel) on
 * top of the base components.
 */
export const consultantMarkdownComponents = {
  ...markdownComponents,
  code: consultantInlineCode,
};

/**
 * Variant for the StoryConsultant's *streaming* render. Layers two
 * transforms on top of the base components:
 *   • inline-icon rendering (so `[icon:Foo]` sentinels render as icons)
 *   • `pre` override that turns in-progress fenced code blocks into a
 *     small spinner with "Checking tools…" — keeps an in-progress
 *     ```action ...``` JSON block from flickering through as raw code.
 *     Once streaming finishes the renderer switches back to the
 *     segment-aware view (parseBackstageSegments) and the block becomes
 *     a proper BackstageActionCard.
 */
export const consultantStreamingMarkdownComponents = {
  ...consultantMarkdownComponents,
  pre: (_props: { children?: React.ReactNode }) =>
    React.createElement(
      "div",
      {
        className:
          "my-2 inline-flex items-center gap-2 px-3 py-1.5 rounded-md bg-muted/40 border border-border/40 text-xs text-muted-foreground",
      },
      React.createElement("span", {
        className:
          "inline-block w-3 h-3 rounded-full border-2 border-muted-foreground/30 border-t-muted-foreground animate-spin",
      }),
      React.createElement("span", null, "Checking tools…"),
    ),
};

/** Remark plugins for Markdown rendering (includes math support) */
export const remarkPlugins = [remarkMath];

/** Rehype plugins for Markdown rendering (includes KaTeX rendering) */
export const rehypePlugins = [rehypeKatex];

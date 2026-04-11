/** Convert parenthesized text to asterisked text for italic/em rendering in Markdown. */
export function formatMessage(content: string): string {
  return content.replace(/\(([^)]+)\)/g, "*$1*");
}

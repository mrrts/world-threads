// Pixel ladder for chat message bubble font sizes. Indexed by the
// `chatFontSize` preference (0..5, default 2). Also drives the adjuster
// control in the chat settings popover.
export const CHAT_FONT_SIZES_PX = [12, 14, 16, 18, 20, 22] as const;

export function chatFontPx(level: number): number {
  const idx = Math.max(0, Math.min(CHAT_FONT_SIZES_PX.length - 1, Math.round(level)));
  return CHAT_FONT_SIZES_PX[idx];
}

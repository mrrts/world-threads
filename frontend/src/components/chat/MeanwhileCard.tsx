import type { MeanwhileEvent } from "@/lib/tauri";

interface Props {
  event: MeanwhileEvent;
  /** Portrait data URL for the character, if available. Falls back to a
   *  solid color band (from avatar_color) when missing. */
  portraitUrl?: string;
}

/// Inline "meanwhile" card placed chronologically in chat history.
/// Ambient rather than eventful: left band shows the character's
/// FULL portrait fading into the background; the rest carries a small
/// "Meanwhile" label + world-time marker + italic event summary,
/// vertically centered against the portrait's height. Distinct visual
/// register from message bubbles and illustrations — reads as "life
/// happening off-screen" rather than "something said into the chat."
export function MeanwhileCard({ event, portraitUrl }: Props) {
  const timeLabel = formatTimeOfDay(event.time_of_day);
  return (
    <div className="flex justify-center my-3">
      <div className="relative w-full max-w-[720px] h-[260px] rounded-xl overflow-hidden border border-border/30 bg-card/30 backdrop-blur-sm">
        {/* Portrait background — full-height left band, contained so
            the entire portrait reads, then SOFT-FADED across the whole
            card so it blends into the card's bg rather than ending
            in a hard edge. The right half of the card has effectively
            no portrait, leaving room for the text to sit cleanly. */}
        {portraitUrl ? (
          <div
            className="absolute inset-y-0 left-0 w-[260px] bg-cover bg-center bg-no-repeat"
            style={{
              backgroundImage: `url(${portraitUrl})`,
              // CSS mask: keep the portrait fully opaque across most of the
              // band so the face reads cleanly, only fading in the last
              // sliver near the right edge where the band meets the text.
              maskImage:
                "linear-gradient(to right, black 0%, black 75%, transparent 100%)",
              WebkitMaskImage:
                "linear-gradient(to right, black 0%, black 75%, transparent 100%)",
              opacity: 0.7,
            }}
            aria-hidden
          />
        ) : (
          <div
            className="absolute inset-y-0 left-0 w-[40%]"
            style={{
              background: `linear-gradient(to right, ${event.avatar_color}66 0%, transparent 100%)`,
            }}
            aria-hidden
          />
        )}

        {/* Heading — top-right, two-line stack. "Meanwhile…" gets its
            own line in larger italic-serif so it reads as the card's
            NAME, not a tag among tags. The trailing ellipsis is the
            real character (U+2026), signalling the ambient "life
            happening off-screen" register. Metadata (who / when) sits
            beneath in the prior small-caps style. Text-shadow keeps
            both lines legible over any portrait tint beneath. */}
        <div
          className="absolute top-3 right-4 z-10 flex flex-col items-end gap-0.5 pointer-events-none text-foreground"
          style={{
            textShadow: "0 1px 2px rgba(0,0,0,0.7), 0 0 8px rgba(0,0,0,0.5)",
          }}
        >
          <span className="text-2xl font-semibold italic tracking-wide leading-none">
            Meanwhile…
          </span>
          <span className="flex items-baseline gap-2 text-[11px] uppercase tracking-wider font-bold">
            <span>{event.character_name}</span>
            <span className="opacity-70">·</span>
            <span className="opacity-90">
              Day {event.world_day}
              {timeLabel ? ` · ${timeLabel}` : ""}
            </span>
          </span>
        </div>

        {/* Summary — vertically centered, right-anchored so it sits
            clear of the portrait's left band. */}
        <div className="relative z-10 h-full px-5 py-4 flex flex-col justify-center">
          <div className="text-sm text-foreground/90 italic leading-relaxed max-w-[60%] ml-auto">
            {event.summary}
          </div>
        </div>
      </div>
    </div>
  );
}

/// Format "MORNING" → "morning" (lowercase for the inline time label).
/// Keeps unknown values untouched.
function formatTimeOfDay(raw: string): string {
  if (!raw) return "";
  // Common values: MORNING / MIDDAY / AFTERNOON / EVENING / LATE NIGHT
  return raw.toLowerCase();
}

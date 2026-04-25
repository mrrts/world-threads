import { useEffect, useState } from "react";
import { Aperture } from "lucide-react";

interface Props {
  location: string | null;
  /// World day number from the most recent timestamped message, or
  /// null if not yet known. Rendered as "Day N" beneath the location.
  worldDay?: number | null;
  /// World-time string ("MORNING", "LATE NIGHT", etc.) — formatted
  /// to title-case and rendered beside the day. Same source convention
  /// as TimeDivider.
  worldTime?: string | null;
  /// When true, hold off starting the show — the chat is still
  /// loading and the user can't see the viewport yet. The 5-second
  /// hold begins the moment `loading` flips to false (and `location`
  /// is set), so a slow load doesn't eat the show.
  loading?: boolean;
}

/// Movie-title style scene-opener shown at the top of the chat
/// viewport once the chat is fully loaded. Fades + slides DOWN into
/// place, holds 5s, then fades + slides UP out. Non-dismissable —
/// sized to read fast and get out of the way. Parent re-mounts via
/// `key={chatId}` so switching chats triggers a fresh appearance.
///
/// Renders an Aperture icon + the location as the headline + a
/// smaller "Day N · Time" subline beneath. No "Location" label —
/// the visual register names itself.
export function LocationOpener({ location, worldDay, worldTime, loading = false }: Props) {
  // Phases: "enter" (offscreen → on-screen), "hold" (visible 5s), "exit"
  // (on-screen → offscreen), "done" (unmounted). Total ~5.8s.
  const [phase, setPhase] = useState<"enter" | "hold" | "exit" | "done">("enter");

  useEffect(() => {
    // Don't start the show until BOTH the chat has finished loading
    // AND we have a location to display. The effect re-runs on each
    // dependency change, so whichever flips last starts the timer.
    if (!location || loading) return;
    setPhase("enter");
    const tHold = setTimeout(() => setPhase("hold"), 30);
    const tExit = setTimeout(() => setPhase("exit"), 5000);
    const tDone = setTimeout(() => setPhase("done"), 5800);
    return () => {
      clearTimeout(tHold);
      clearTimeout(tExit);
      clearTimeout(tDone);
    };
  }, [location, loading]);

  if (!location || phase === "done") return null;

  const isVisible = phase === "hold";
  const transformClass = isVisible ? "translate-y-0 opacity-100" : "-translate-y-8 opacity-0";

  // Format world_time the same way TimeDivider does: "LATE NIGHT" →
  // "Late Night", "MIDDAY" → "Midday". Compose a single subline like
  // "Day 47 · Late Night"; either part may be missing.
  const formattedTime = worldTime
    ? worldTime
        .split(" ")
        .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
        .join(" ")
    : null;
  const sublineParts: string[] = [];
  if (worldDay != null) sublineParts.push(`Day ${worldDay}`);
  if (formattedTime) sublineParts.push(formattedTime);
  const subline = sublineParts.join(" · ");

  return (
    <div className="absolute top-0 left-0 right-0 flex justify-center pointer-events-none z-20 px-6">
      <div
        className={`
          mt-10
          flex items-center gap-6
          px-10 py-6 rounded-2xl
          w-full max-w-2xl
          bg-gradient-to-br from-emerald-950/95 via-emerald-900/90 to-emerald-950/95
          backdrop-blur-md
          border-2 border-emerald-400/50
          shadow-[0_0_60px_rgba(16,185,129,0.55),0_0_120px_rgba(16,185,129,0.30),inset_0_0_30px_rgba(16,185,129,0.15)]
          transition-all duration-700 ease-out
          ${transformClass}
        `}
      >
        <Aperture
          size={44}
          className="text-emerald-300 flex-shrink-0 drop-shadow-[0_0_14px_rgba(110,231,183,0.95)]"
        />
        <div className="flex flex-col leading-tight min-w-0">
          <span className="text-3xl text-emerald-50 font-semibold tracking-tight truncate drop-shadow-[0_0_10px_rgba(167,243,208,0.55)]">
            {location}
          </span>
          {subline && (
            <span className="mt-1 text-sm uppercase tracking-[0.22em] text-emerald-300/75 font-medium drop-shadow-[0_0_6px_rgba(110,231,183,0.4)]">
              {subline}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

import { useEffect, useState } from "react";
import { MapPin } from "lucide-react";

interface Props {
  location: string | null;
  /// When true, hold off starting the show — the chat is still
  /// loading and the user can't see the viewport yet. The 5-second
  /// hold begins the moment `loading` flips to false (and `location`
  /// is set), so a slow load doesn't eat the show.
  loading?: boolean;
}

/// Movie-title style location reorienter shown at the top of the chat
/// viewport once the chat is fully loaded. Fades + slides DOWN into
/// place, holds 5s, then fades + slides UP out. Non-dismissable —
/// sized to read fast and get out of the way. Parent re-mounts via
/// `key={chatId}` so switching chats triggers a fresh appearance.
///
/// Returns null when location is unset, while loading is true, or
/// after the exit animation completes.
export function LocationOpener({ location, loading = false }: Props) {
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

  return (
    <div className="absolute top-0 left-0 right-0 flex justify-center pointer-events-none z-20 px-6">
      <div
        className={`
          mt-10
          flex items-center gap-5
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
        <MapPin
          size={42}
          className="text-emerald-300 flex-shrink-0 drop-shadow-[0_0_14px_rgba(110,231,183,0.95)]"
        />
        <div className="flex flex-col leading-tight min-w-0">
          <span className="text-[12px] uppercase tracking-[0.32em] text-emerald-300/85 font-semibold drop-shadow-[0_0_6px_rgba(110,231,183,0.5)]">
            Location
          </span>
          <span className="text-3xl text-emerald-50 font-semibold tracking-tight truncate drop-shadow-[0_0_10px_rgba(167,243,208,0.55)]">
            {location}
          </span>
        </div>
      </div>
    </div>
  );
}

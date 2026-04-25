import { useEffect, useRef, useState } from "react";
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
/// sized to read fast and get out of the way.
///
/// Once the show starts, it plays through to completion. The timers
/// are tracked via refs and started exactly ONCE per mount — the
/// effect bails on subsequent re-fires so dep-change cleanup never
/// kills an in-flight show. Parent uses `key={chatKey}` to remount
/// for fresh chats; that's the only path to restart the show.
export function LocationOpener({ location, loading = false }: Props) {
  // Phases: "enter" (offscreen — initial pre-show), "hold" (visible 5s),
  // "exit" (on-screen → offscreen), "done" (unmounted).
  const [phase, setPhase] = useState<"enter" | "hold" | "exit" | "done">("enter");
  const startedRef = useRef(false);

  useEffect(() => {
    if (startedRef.current) return;
    if (!location || loading) return;
    startedRef.current = true;
    // Schedule the entire show right now, in real time. Timers are NOT
    // returned for cleanup — once we've earned the start signal, the
    // show plays through even if subsequent re-renders fire the effect
    // again (which they won't, because startedRef short-circuits).
    setTimeout(() => setPhase("hold"), 30);
    setTimeout(() => setPhase("exit"), 5000);
    setTimeout(() => setPhase("done"), 5800);
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

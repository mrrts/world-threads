import { useEffect, useState } from "react";
import { MapPin } from "lucide-react";

interface Props {
  location: string | null;
}

/// Movie-title style location reorienter shown at the top of the chat
/// viewport when a chat opens. Fades + slides DOWN into place, holds
/// for the on-screen window, then fades + slides UP out. Non-dismissable
/// — sized to read fast and get out of the way. Parent re-mounts via
/// `key={chatId}` so switching chats triggers a fresh appearance.
///
/// Returns null when location is unset (no point setting a scene that
/// hasn't been named) or after the exit animation completes.
export function LocationOpener({ location }: Props) {
  // Phases: "enter" (offscreen → on-screen), "hold" (visible 3s), "exit"
  // (on-screen → offscreen), "done" (unmounted). Total ~3.7s.
  const [phase, setPhase] = useState<"enter" | "hold" | "exit" | "done">("enter");

  useEffect(() => {
    if (!location) return;
    setPhase("enter");
    const tHold = setTimeout(() => setPhase("hold"), 30);
    const tExit = setTimeout(() => setPhase("exit"), 5000);
    const tDone = setTimeout(() => setPhase("done"), 5800);
    return () => {
      clearTimeout(tHold);
      clearTimeout(tExit);
      clearTimeout(tDone);
    };
  }, [location]);

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

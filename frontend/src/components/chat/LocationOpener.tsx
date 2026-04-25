import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
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
  const mountedAtRef = useRef(performance.now());
  const tag = `[LocationOpener]`;

  // Mount log — fires once per fresh instance (key change in parent).
  useEffect(() => {
    const t = Math.round(performance.now() - mountedAtRef.current);
    console.log(`${tag} MOUNT t=${t}ms loc=${JSON.stringify(location)} loading=${loading}`);
    return () => {
      const tt = Math.round(performance.now() - mountedAtRef.current);
      console.log(`${tag} UNMOUNT t=${tt}ms`);
    };
    // Intentional: log mount/unmount only; not tied to dep changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Per-render log — every render, with current props + state.
  const tNow = Math.round(performance.now() - mountedAtRef.current);
  console.log(`${tag} RENDER t=${tNow}ms loc=${JSON.stringify(location)} loading=${loading} phase=${phase} started=${startedRef.current}`);

  useEffect(() => {
    const t = Math.round(performance.now() - mountedAtRef.current);
    console.log(`${tag} EFFECT t=${t}ms loc=${JSON.stringify(location)} loading=${loading} started=${startedRef.current}`);
    if (startedRef.current) {
      console.log(`${tag} bail: already started`);
      return;
    }
    if (!location) {
      console.log(`${tag} bail: no location`);
      return;
    }
    if (loading) {
      console.log(`${tag} bail: still loading`);
      return;
    }
    console.log(`${tag} STARTING SHOW`);
    startedRef.current = true;
    const LEAD_IN = 350;
    const HOLD = 5000;
    const EXIT = 800;
    setTimeout(() => { console.log(`${tag} -> hold`); setPhase("hold"); }, LEAD_IN);
    setTimeout(() => { console.log(`${tag} -> exit`); setPhase("exit"); }, LEAD_IN + HOLD);
    setTimeout(() => { console.log(`${tag} -> done`); setPhase("done"); }, LEAD_IN + HOLD + EXIT);
  }, [location, loading]);

  const innerRef = useRef<HTMLDivElement | null>(null);

  // Imperatively write the inline styles via useLayoutEffect every render
  // — bypasses React style reconciliation entirely. setProperty with
  // 'important' flag ensures nothing in the cascade can override.
  useLayoutEffect(() => {
    const el = innerRef.current;
    if (!el) return;
    const isVisible = phase === "hold";
    el.style.setProperty("transform", isVisible ? "translateY(0)" : "translateY(-2rem)", "important");
    el.style.setProperty("opacity", isVisible ? "1" : "0", "important");
    el.style.setProperty("transition", "transform 700ms ease-out, opacity 700ms ease-out", "important");
    const t = Math.round(performance.now() - mountedAtRef.current);
    console.log(`${tag} STYLE-WROTE t=${t}ms phase=${phase} cs.opacity=${window.getComputedStyle(el).opacity} cs.transform=${window.getComputedStyle(el).transform}`);
  }, [phase]);

  if (!location || phase === "done") return null;

  return (
    <div
      ref={(el) => {
        if (el && phase === "hold") {
          const r = el.getBoundingClientRect();
          const inner = el.firstElementChild as HTMLElement | null;
          const innerR = inner?.getBoundingClientRect();
          const s = inner ? window.getComputedStyle(inner) : null;
          console.log(`${tag} BBOX outer=(x:${Math.round(r.x)},y:${Math.round(r.y)},w:${Math.round(r.width)},h:${Math.round(r.height)}) inner=(x:${Math.round(innerR?.x ?? 0)},y:${Math.round(innerR?.y ?? 0)},w:${Math.round(innerR?.width ?? 0)},h:${Math.round(innerR?.height ?? 0)}) opacity=${s?.opacity} transform=${s?.transform} display=${s?.display} visibility=${s?.visibility} zIndex=${s?.zIndex} win=(${window.innerWidth}x${window.innerHeight})`);
        }
      }}
      className="absolute top-0 left-0 right-0 flex justify-center pointer-events-none z-[100] px-6"
    >
      <div
        ref={innerRef}
        className="
          mt-10
          flex items-center gap-5
          px-10 py-6 rounded-2xl
          w-full max-w-2xl
          bg-gradient-to-br from-emerald-950/95 via-emerald-900/90 to-emerald-950/95
          backdrop-blur-md
          border-2 border-emerald-400/50
          shadow-[0_0_60px_rgba(16,185,129,0.55),0_0_120px_rgba(16,185,129,0.30),inset_0_0_30px_rgba(16,185,129,0.15)]
        "
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

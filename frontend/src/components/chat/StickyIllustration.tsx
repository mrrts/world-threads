import { useEffect, useMemo, useRef, useState, useCallback } from "react";
import { Images, Maximize2, Minimize2 } from "lucide-react";
import { api, type Message, type IllustrationSummary } from "@/lib/tauri";

// Three size presets for the floating sticky. `lg` is the historical
// default (what the sticky has always been); `md` and `sm` are the
// new smaller steps. The clamp values scale fluidly across laptop →
// ultrawide viewports so every preset stays visually coherent at the
// `xl:block` gate (~1280px) through 3840px displays.
const SIZE_PRESETS = {
  sm: "clamp(110px, 8vw, 260px)",
  md: "clamp(160px, 14vw, 440px)",
  lg: "clamp(220px, 20vw, 620px)",
} as const;
type SizeKey = keyof typeof SIZE_PRESETS;
const SIZE_ORDER: SizeKey[] = ["sm", "md", "lg"];
const SIZE_STORAGE_KEY = "wt.sticky-illustration.size";

function loadInitialSize(): SizeKey {
  try {
    const v = localStorage.getItem(SIZE_STORAGE_KEY);
    if (v === "sm" || v === "md" || v === "lg") return v;
  } catch {
    // localStorage unavailable — use default.
  }
  return "lg";
}

interface Props {
  messages: Message[];
  /** The scrollable viewport element. Used to read scroll position
   *  and as the coordinate frame for visibility calculations. */
  scrollContainer: HTMLElement | null;
  /** Optional aspect-ratio map keyed by message_id, same shape as
   *  store.aspectRatios, so the thumbnail matches the illustration
   *  without an image-first reflow. */
  aspectRatios?: Record<string, number>;
}

/// A floating thumbnail pinned to the lower-right of the chat viewport.
/// Shows the illustration most contextually relevant to whatever the
/// user is currently reading — the most-recently-preceding illustration
/// in the full thread timeline. Critically: this works even when the
/// relevant illustration isn't paginated into `store.messages` yet.
///
/// Full illustration timeline is fetched from the server per thread;
/// scroll-position mapping uses (a) DOM positions for messages that
/// ARE loaded, and (b) fallback to "this illustration predates all
/// loaded history" for ones that aren't.
export function StickyIllustration({ messages, scrollContainer, aspectRatios }: Props) {
  // Thread ID is derived from the loaded messages. When a thread has
  // no messages at all, the sticky has nothing to work with.
  const threadId = messages[0]?.thread_id ?? "";

  // Full timeline of illustrations in this thread, fetched from the
  // server. Kept in state so we can refresh when a new illustration
  // is generated locally or the thread changes.
  const [timeline, setTimeline] = useState<IllustrationSummary[]>([]);

  // Count of illustrations currently in loaded messages — used as a
  // trigger to refetch the full timeline when a new one arrives.
  const localIllusCount = useMemo(
    () => messages.filter((m) => m.role === "illustration").length,
    [messages]
  );

  // Fetch on thread change or when a new local illustration appears.
  useEffect(() => {
    if (!threadId) {
      setTimeline([]);
      return;
    }
    let cancelled = false;
    api
      .listThreadIllustrations(threadId)
      .then((list) => {
        if (!cancelled) setTimeline(list);
      })
      .catch(() => {
        if (!cancelled) setTimeline([]);
      });
    return () => {
      cancelled = true;
    };
  }, [threadId, localIllusCount]);

  // Live ref for the scroll handler so it always sees the latest
  // timeline without needing to rebind on each fetch.
  const timelineRef = useRef<IllustrationSummary[]>(timeline);
  timelineRef.current = timeline;
  const [activeIllus, setActiveIllus] = useState<IllustrationSummary | null>(null);
  const [activeInView, setActiveInView] = useState<boolean>(true);

  useEffect(() => {
    if (!scrollContainer) return;

    const recompute = () => {
      const tl = timelineRef.current;
      if (tl.length === 0) {
        setActiveIllus(null);
        setActiveInView(true);
        return;
      }

      const containerRect = scrollContainer.getBoundingClientRect();
      const viewportTop = containerRect.top;
      const viewportBottom = containerRect.bottom;

      // Walk the FULL timeline (not just loaded messages). For each:
      //   - if its row is mounted in the DOM, use the real rect.
      //   - if not, it's in older unloaded history — treat as
      //     positionally preceding everything visible (so any
      //     loaded illustration later in the timeline will
      //     override it; if none do, it becomes the active).
      let lastPreceding: IllustrationSummary | null = null;
      let inView = false;

      for (const illus of tl) {
        const el = scrollContainer.querySelector<HTMLElement>(
          `[data-message-id="${illus.message_id}"]`
        );
        if (!el) {
          // Not rendered → must be in unloaded older history.
          // Its "position" is above every loaded row, so it qualifies
          // as preceding whatever is currently in view.
          lastPreceding = illus;
          inView = false;
          continue;
        }
        const rect = el.getBoundingClientRect();
        if (rect.top <= viewportBottom) {
          lastPreceding = illus;
          inView = rect.bottom > viewportTop && rect.top < viewportBottom;
        } else {
          // This illustration is below the viewport; rest will be too.
          break;
        }
      }

      setActiveIllus(lastPreceding);
      setActiveInView(inView);
    };

    recompute();

    let rafId = 0;
    const onScroll = () => {
      if (rafId) return;
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        recompute();
      });
    };
    scrollContainer.addEventListener("scroll", onScroll, { passive: true });

    const onResize = () => recompute();
    window.addEventListener("resize", onResize);

    // Catch reflow (new messages inserted, old ones rendered as user
    // paginates backward, etc.) without re-binding on each render.
    const mo = new MutationObserver(() => {
      if (rafId) return;
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        recompute();
      });
    });
    mo.observe(scrollContainer, { childList: true, subtree: true });

    return () => {
      scrollContainer.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onResize);
      mo.disconnect();
      if (rafId) cancelAnimationFrame(rafId);
    };
  }, [scrollContainer]);

  // Recompute when the timeline itself changes (fetched, or new
  // illustration added). The scroll-based effect reads the ref, but
  // won't fire on its own when nothing has scrolled.
  useEffect(() => {
    if (!scrollContainer) return;
    // Nudge: dispatch a synthetic scroll so the handler recomputes.
    const ev = new Event("scroll");
    scrollContainer.dispatchEvent(ev);
  }, [timeline.length, scrollContainer]);

  // First-paint gate: render the button in the "hidden" state on the
  // very first paint, then double-rAF flip to the target state so CSS
  // transitions animate FROM hidden on initial appearance.
  const [initialGate, setInitialGate] = useState<boolean>(true);
  useEffect(() => {
    if (!activeIllus) {
      setInitialGate(true);
      return;
    }
    let raf1 = 0;
    let raf2 = 0;
    raf1 = requestAnimationFrame(() => {
      raf2 = requestAnimationFrame(() => setInitialGate(false));
    });
    return () => {
      cancelAnimationFrame(raf1);
      cancelAnimationFrame(raf2);
    };
  }, [activeIllus?.message_id]);

  // User-selected size preset. Persisted to localStorage so the
  // preference rides across sessions and thread switches.
  const [size, setSize] = useState<SizeKey>(loadInitialSize);
  useEffect(() => {
    try { localStorage.setItem(SIZE_STORAGE_KEY, size); } catch { /* ignore */ }
  }, [size]);
  const cycleSize = useCallback(() => {
    setSize((prev) => {
      const idx = SIZE_ORDER.indexOf(prev);
      return SIZE_ORDER[(idx + 1) % SIZE_ORDER.length];
    });
  }, []);

  if (!activeIllus) return null;

  const hidden = activeInView || initialGate;

  const ar = aspectRatios?.[activeIllus.message_id];

  // Clicking the sticky image opens the gallery modal to this exact
  // illustration. Older/unloaded illustrations are still reachable
  // because the modal fetches the full thread page on open.
  const onClickImage = () => {
    window.dispatchEvent(new CustomEvent("wt:open-gallery", {
      detail: { messageId: activeIllus.message_id },
    }));
  };
  const onKeyDownOuter = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onClickImage();
    }
  };

  const base = `hidden xl:block absolute bottom-4 right-4 z-20 group select-none
                rounded-xl overflow-hidden shadow-2xl shadow-black/50
                ring-1 ring-emerald-700/30
                bg-gradient-to-br from-emerald-950/40 to-emerald-900/20
                backdrop-blur-sm
                transition-all duration-500 ease-out
                motion-reduce:duration-200`;

  const visState = hidden
    ? "opacity-0 scale-90 translate-y-4 translate-x-2 pointer-events-none"
    : "opacity-100 scale-100 translate-y-0 translate-x-0 cursor-pointer hover:ring-emerald-500/60";

  // Icon for the resize button: signal the NEXT step explicitly.
  // From sm/md the next step is larger → Maximize2. From lg the
  // next step wraps back to sm → Minimize2.
  const ResizeIcon = size === "lg" ? Minimize2 : Maximize2;
  const nextSizeLabel = size === "lg" ? "sm" : size === "md" ? "lg" : "md";

  return (
    <div
      role="button"
      tabIndex={hidden ? -1 : 0}
      onClick={hidden ? undefined : onClickImage}
      onKeyDown={hidden ? undefined : onKeyDownOuter}
      aria-hidden={hidden ? true : undefined}
      aria-label="Open this illustration in the gallery"
      title="Open in gallery"
      className={`${base} ${visState}`}
      // width uses the active size preset; the transition on the
      // outer container animates between presets smoothly.
      style={{ width: SIZE_PRESETS[size], maxHeight: "70vh" }}
    >
      <img
        src={activeIllus.content}
        alt="Illustration for the moment you're reading"
        className="block w-full h-auto max-h-[70vh] object-contain pointer-events-none"
        style={ar ? { aspectRatio: String(ar) } : undefined}
        draggable={false}
      />
      {/* Hover overlay — the resize button sits top-right (mirroring
          IllustrationMessage's hover toolbar), and a small "Open in
          gallery" pill sits bottom-right as the primary affordance
          cue. Both appear together on hover. */}
      <div
        className="absolute top-1.5 right-1.5 flex gap-1.5 opacity-0 group-hover:opacity-100
                   transition-opacity duration-150"
      >
        <div className="relative group/resize">
          <button
            type="button"
            onClick={(e) => {
              // Don't trigger the outer's open-gallery click.
              e.stopPropagation();
              cycleSize();
            }}
            aria-label={`Resize (next: ${nextSizeLabel})`}
            className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
          >
            <ResizeIcon size={14} />
          </button>
          <span className="absolute top-full right-0 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/resize:opacity-100 pointer-events-none transition-opacity">
            Resize
          </span>
        </div>
      </div>
      <div
        className="absolute inset-0 pointer-events-none opacity-0 group-hover:opacity-100
                   transition-opacity duration-150 flex items-end justify-end
                   bg-gradient-to-t from-black/60 via-transparent to-transparent"
      >
        <span className="m-1.5 inline-flex items-center gap-1 rounded-full
                         bg-black/60 px-2 py-0.5 text-[10px] font-medium text-white">
          <Images size={10} />
          Open in gallery
        </span>
      </div>
    </div>
  );
}

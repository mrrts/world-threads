import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowUp } from "lucide-react";
import type { Message } from "@/lib/tauri";

interface Props {
  messages: Message[];
  /** The scrollable viewport element. Used to read scroll position
   *  and as the coordinate frame for visibility calculations. Can be
   *  null if not yet mounted. */
  scrollContainer: HTMLElement | null;
  /** Optional aspect-ratio map keyed by message_id, same shape as
   *  store.aspectRatios, so the thumbnail matches the illustration
   *  without an image-first reflow. */
  aspectRatios?: Record<string, number>;
}

/// A floating thumbnail pinned to the lower-right of the chat viewport.
/// Tracks the scroll position: shows the illustration most recently
/// scrolled past (i.e. the one whose DOM position is still above the
/// current viewport), so as the user scrolls through history, the
/// sticky stays tied to the scene depicted by what they're currently
/// reading. Hides when that active illustration is itself in view.
///
/// Rationale: the user wanted a persistent visual-orientation anchor
/// for whatever moment they're currently reading, not just the latest
/// illustration in the thread.
export function StickyIllustration({ messages, scrollContainer, aspectRatios }: Props) {
  // All illustration messages in chronological order.
  const illustrations = useMemo(
    () => messages.filter((m) => m.role === "illustration"),
    [messages]
  );

  // Keep a live ref so the scroll listener reads the latest messages
  // without needing to re-bind every time a new message arrives.
  const illusRef = useRef(illustrations);
  illusRef.current = illustrations;

  const [activeIllus, setActiveIllus] = useState<Message | null>(null);
  const [activeInView, setActiveInView] = useState<boolean>(true);

  useEffect(() => {
    if (!scrollContainer) return;

    const recompute = () => {
      const list = illusRef.current;
      if (list.length === 0) {
        setActiveIllus(null);
        setActiveInView(true);
        return;
      }

      const containerRect = scrollContainer.getBoundingClientRect();
      const viewportTop = containerRect.top;
      const viewportBottom = containerRect.bottom;

      let lastPreceding: Message | null = null;
      let inView = false;

      for (const msg of list) {
        const el = scrollContainer.querySelector<HTMLElement>(
          `[data-message-id="${msg.message_id}"]`
        );
        if (!el) continue;
        const elRect = el.getBoundingClientRect();

        // Illustration has started (top is at or above viewport-bottom)
        // → candidate for "most recently seen/scrolled past."
        if (elRect.top <= viewportBottom) {
          lastPreceding = msg;
          // Visible if any part of it intersects the viewport.
          inView = elRect.bottom > viewportTop && elRect.top < viewportBottom;
        } else {
          // This illustration is below the viewport entirely.
          // All later ones are also below (chronological order).
          break;
        }
      }

      setActiveIllus(lastPreceding);
      setActiveInView(inView);
    };

    // Initial compute.
    recompute();

    // rAF-throttled scroll handler — fires at most once per frame.
    let rafId = 0;
    const onScroll = () => {
      if (rafId) return;
      rafId = requestAnimationFrame(() => {
        rafId = 0;
        recompute();
      });
    };
    scrollContainer.addEventListener("scroll", onScroll, { passive: true });

    // Resize can change which illustration is in view (reflow).
    const onResize = () => recompute();
    window.addEventListener("resize", onResize);

    // When messages change (new illustration arrives, one is deleted),
    // rerun. MutationObserver is the simplest way to catch reflow
    // without re-binding listeners on every React render.
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

  // Also recompute when the messages array reference changes, in case
  // the MutationObserver hasn't fired yet (e.g. first render batch).
  useEffect(() => {
    if (!scrollContainer) return;
    const list = illusRef.current;
    if (list.length === 0) {
      setActiveIllus(null);
      return;
    }
    // Let the main effect's recompute path handle it — but nudge once
    // via a microtask so the initial active illustration is set before
    // paint rather than after first scroll.
    queueMicrotask(() => {
      const event = new Event("scroll");
      scrollContainer.dispatchEvent(event);
    });
  }, [messages, scrollContainer]);

  if (!activeIllus || activeInView) return null;

  const ar = aspectRatios?.[activeIllus.message_id];
  const onClick = () => {
    if (!scrollContainer) return;
    const el = scrollContainer.querySelector<HTMLElement>(
      `[data-message-id="${activeIllus.message_id}"]`
    );
    if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
  };

  return (
    <button
      type="button"
      onClick={onClick}
      aria-label="Scroll to the illustration for the moment you're reading"
      title="Scroll to this illustration"
      className="hidden xl:block absolute bottom-4 right-4 z-20 group cursor-pointer select-none
                 rounded-xl overflow-hidden shadow-2xl shadow-black/50
                 ring-1 ring-emerald-700/30 hover:ring-emerald-500/60
                 bg-gradient-to-br from-emerald-950/40 to-emerald-900/20
                 backdrop-blur-sm transition-all duration-200
                 hover:scale-[1.04] hover:-translate-y-0.5
                 animate-in fade-in-0 zoom-in-90 slide-in-from-bottom-4 slide-in-from-right-2
                 duration-500 ease-out
                 motion-reduce:animate-in motion-reduce:fade-in-0 motion-reduce:duration-200"
      style={{ width: 264 }}
    >
      <img
        src={activeIllus.content}
        alt="Illustration for the moment you're reading"
        className="block w-full h-auto"
        style={ar ? { aspectRatio: String(ar) } : undefined}
        draggable={false}
      />
      <div
        className="absolute inset-0 pointer-events-none opacity-0 group-hover:opacity-100
                   transition-opacity duration-150 flex items-end justify-end
                   bg-gradient-to-t from-black/60 via-transparent to-transparent"
      >
        <span className="m-1.5 inline-flex items-center gap-1 rounded-full
                         bg-black/60 px-2 py-0.5 text-[10px] font-medium text-white">
          <ArrowUp size={10} />
          Jump to scene
        </span>
      </div>
    </button>
  );
}

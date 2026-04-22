import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowUp } from "lucide-react";
import type { Message } from "@/lib/tauri";

interface Props {
  messages: Message[];
  /** The scrollable viewport element. Used as the IntersectionObserver
   *  root so visibility tracks within the chat window, not the browser
   *  viewport. Can be null if not yet mounted. */
  scrollContainer: HTMLElement | null;
  /** Optional aspect-ratio map keyed by message_id, same shape as
   *  store.aspectRatios, so the thumbnail matches the illustration
   *  without loading a huge image-first reflow. */
  aspectRatios?: Record<string, number>;
}

/// A floating thumbnail of the LATEST illustration in the thread,
/// pinned to the lower-right of the chat viewport, shown ONLY when the
/// real illustration is scrolled out of view. Gives the user a
/// persistent visual-orientation anchor as the conversation keeps going
/// below the illustration. Click scrolls the original back into view.
export function StickyIllustration({ messages, scrollContainer, aspectRatios }: Props) {
  // Pick the most recent illustration in the thread.
  const latest = useMemo(() => {
    for (let i = messages.length - 1; i >= 0; i--) {
      if (messages[i].role === "illustration") return messages[i];
    }
    return null;
  }, [messages]);

  const [isVisible, setIsVisible] = useState<boolean>(true);
  const lastObservedIdRef = useRef<string | null>(null);

  useEffect(() => {
    if (!latest || !scrollContainer) {
      setIsVisible(true);
      lastObservedIdRef.current = null;
      return;
    }
    // Find the illustration's DOM wrapper via the data attribute
    // IllustrationMessage sets. If the element isn't present yet
    // (just-mounted message), retry once next frame.
    const findAndObserve = () => {
      const el = scrollContainer.querySelector<HTMLElement>(
        `[data-message-id="${latest.message_id}"]`
      );
      if (!el) return null;

      lastObservedIdRef.current = latest.message_id;
      // Default to TRUE (no sticky) until the observer fires. Avoids
      // the sticky flashing in on mount before the real element has
      // been measured.
      setIsVisible(true);

      const observer = new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            setIsVisible(entry.isIntersecting);
          }
        },
        {
          root: scrollContainer,
          // A sliver of the illustration counts as visible — we don't
          // want the sticky flashing in the moment the image edges
          // touch the viewport edge. 0.05 = 5% visible = visible.
          threshold: 0.05,
        }
      );
      observer.observe(el);
      return observer;
    };

    let observer = findAndObserve();
    // If the element isn't mounted yet (message just arrived), retry
    // on the next frame. Typical in the "illustration just appended"
    // case where React hasn't painted yet.
    let raf = 0;
    if (!observer) {
      raf = requestAnimationFrame(() => {
        observer = findAndObserve();
      });
    }

    return () => {
      cancelAnimationFrame(raf);
      observer?.disconnect();
    };
  }, [latest?.message_id, scrollContainer]);

  if (!latest || isVisible) return null;

  const ar = aspectRatios?.[latest.message_id];
  const onClick = () => {
    if (!scrollContainer) return;
    const el = scrollContainer.querySelector<HTMLElement>(
      `[data-message-id="${latest.message_id}"]`
    );
    if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
  };

  return (
    <button
      type="button"
      onClick={onClick}
      aria-label="Scroll to latest illustration"
      title="Scroll to latest illustration"
      className="absolute bottom-4 right-4 z-20 group cursor-pointer select-none
                 rounded-xl overflow-hidden shadow-2xl shadow-black/50
                 ring-1 ring-emerald-700/30 hover:ring-emerald-500/60
                 bg-gradient-to-br from-emerald-950/40 to-emerald-900/20
                 backdrop-blur-sm transition-all duration-200
                 hover:scale-[1.04] hover:-translate-y-0.5
                 animate-in fade-in slide-in-from-bottom-2"
      style={{ width: 132 }}
    >
      <img
        src={latest.content}
        alt="Latest illustration (scrolled out of view)"
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
          Jump up
        </span>
      </div>
    </button>
  );
}

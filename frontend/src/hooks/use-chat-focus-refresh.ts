import { useEffect, useRef } from "react";

/**
 * Fire a "chat received focus" callback on:
 *   - mount (whenever the containerRef's target chat identity changes)
 *   - user interaction: click / keydown / scroll inside the chat container
 *
 * Debounced: only fires at most once per `cooldownMs` (default 30s).
 * The backend the callback talks to is expected to no-op when the
 * per-character stamp is still fresh, so this is a hint-layer
 * throttling — cheap even if it fires often.
 *
 * Usage:
 *   const chatRef = useRef<HTMLDivElement>(null);
 *   useChatFocusRefresh({
 *     chatKey: charId,               // triggers a fire on change
 *     containerRef: chatRef,         // where to listen for interactions
 *     onFocusRefresh: () => {...},   // the refresh callback
 *   });
 *   <div ref={chatRef}>...</div>
 */
export function useChatFocusRefresh({
  chatKey,
  containerRef,
  onFocusRefresh,
  cooldownMs = 30_000,
}: {
  chatKey: string | undefined;
  containerRef: React.RefObject<HTMLElement | null>;
  onFocusRefresh: () => void;
  cooldownMs?: number;
}) {
  const lastFiredRef = useRef<number>(0);

  // Debounced fire — only trips if enough time has passed since the
  // last call for THIS chat. chatKey flipping resets the clock so a
  // new chat gets its first-focus pass regardless of cooldown.
  const maybeFire = useRef(() => {
    const now = Date.now();
    if (now - lastFiredRef.current < cooldownMs) return;
    lastFiredRef.current = now;
    onFocusRefresh();
  });
  maybeFire.current = () => {
    const now = Date.now();
    if (now - lastFiredRef.current < cooldownMs) return;
    lastFiredRef.current = now;
    onFocusRefresh();
  };

  // Fire on chat identity change (mount / switch). Reset cooldown so
  // the first focus of a new chat always goes through.
  useEffect(() => {
    if (!chatKey) return;
    lastFiredRef.current = 0;
    // Defer by a tick so the chat settles / scroll applies first.
    const t = setTimeout(() => maybeFire.current(), 50);
    return () => clearTimeout(t);
  }, [chatKey]);

  // Listen for interactions inside the container. Mousedown, keydown,
  // and scroll are the three signals that "this chat is active to the
  // user right now." Scroll especially — the user engaging with the
  // history is a high-signal focus event.
  useEffect(() => {
    const el = containerRef.current;
    if (!el || !chatKey) return;
    const handler = () => maybeFire.current();
    el.addEventListener("mousedown", handler);
    el.addEventListener("keydown", handler);
    el.addEventListener("scroll", handler, true); // capture: catches inner scrolls
    return () => {
      el.removeEventListener("mousedown", handler);
      el.removeEventListener("keydown", handler);
      el.removeEventListener("scroll", handler, true);
    };
  }, [chatKey, containerRef]);
}

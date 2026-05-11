// HTMLAudioElement-based chime. Reuses a single module-singleton Audio
// element instead of constructing a new one per call — browsers cap
// concurrent media elements (~30-50 in Chromium/WebKit), and silently
// fail to construct new ones once the cap is hit. The prior per-call
// `new Audio(chimeUrl)` shape leaked elements (each one held until GC)
// and the longer padded wav (0.843s vs 0.340s) made the accumulation
// faster, so chimes stopped firing after a burst of messages.
//
// The single-element shape also sidesteps the AudioContext-suspended-
// after-macOS-sleep bug that motivated the original switch from
// oscillator-based chime: HTMLAudioElement state is browser-managed
// and recovers across sleep/wake without explicit resume calls.

import chimeUrl from "@/assets/message-chime.wav";

let audio: HTMLAudioElement | null = null;

/** Play the message-chime sound. Sync; fire-and-forget. */
export function playChime() {
  try {
    if (!audio) {
      audio = new Audio(chimeUrl);
      audio.volume = 0.5;
      // Don't preload — let the browser decide; the asset is tiny.
    }
    // Rewind so rapid-fire chimes restart cleanly instead of being
    // silently no-op'd (audio.play() on an already-playing element is
    // a no-op in some browsers).
    audio.currentTime = 0;
    void audio.play().catch(() => {
      // autoplay policy or transient device issue — silently ignore
    });
  } catch {
    // Audio constructor failed (extremely rare) — silently ignore
  }
}

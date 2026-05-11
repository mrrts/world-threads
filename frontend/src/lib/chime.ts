// HTMLAudioElement-based chime. Fresh element per call so macOS sleep/wake
// doesn't leave us stuck on a suspended AudioContext (the prior
// oscillator-based implementation cached a module-singleton AudioContext
// that transitioned to "suspended" on sleep and never resumed; symptom
// was silent chimes after wake with no console error).
//
// HTMLAudioElement state is browser-managed and recovers across sleep/wake
// without explicit resume calls. autoplay-policy errors (no user gesture)
// are swallowed; chimes only fire from user-driven event handlers in
// practice, so the first one after page load works.

import chimeUrl from "@/assets/message-chime.wav";

/** Play the message-chime sound. Sync; fire-and-forget. */
export function playChime() {
  try {
    const audio = new Audio(chimeUrl);
    audio.volume = 0.5;
    void audio.play().catch(() => {
      // autoplay policy or transient device issue — silently ignore
    });
  } catch {
    // Audio constructor failed (extremely rare) — silently ignore
  }
}

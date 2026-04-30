import { useEffect, useRef, useState, useCallback } from "react";
import { api, type Message } from "@/lib/tauri";
import {
  playPhrase,
  getAudioContextTime,
  type ScorePhrase,
  type PlayHandle,
} from "@/lib/chiptune-synth";

export type SoundtrackStatus = "idle" | "generating" | "playing";

interface Args {
  enabled: boolean;
  apiKey: string | null;
  latestAssistantMessage: Message | null;
}

interface SoundtrackState {
  status: SoundtrackStatus;
  error: string | null;
  currentPhrase: ScorePhrase | null;
  stopAndReset: () => void;
}

/**
 * Soundtrack lifecycle:
 *
 *   - The current phrase loops continuously via a self-rescheduling step()
 *     that calls playPhrase(currentPhrase, { startAt: <prev iteration's end> }).
 *   - When a new assistant message arrives with formula_signature, we kick off
 *     api.generateNextScorePhrase. The result is held as a *pending* phrase,
 *     not played immediately.
 *   - The next time step() fires (always at a phrase-end boundary, which is
 *     also a measure boundary by construction), it promotes pending → current
 *     and starts the new phrase. The swap is seamless and aligned to the bar.
 *   - Disable / unmount stops scheduling and silences the active handle.
 */
export function useChiptuneSoundtrack({
  enabled,
  apiKey,
  latestAssistantMessage,
}: Args): SoundtrackState {
  const [status, setStatus] = useState<SoundtrackStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const [currentPhrase, setCurrentPhrase] = useState<ScorePhrase | null>(null);

  // Refs hold scheduling state that must not retrigger effect runs.
  const handleRef = useRef<PlayHandle | null>(null);
  const cursorRef = useRef<number>(0);                           // audio time when current iteration ends
  const currentPhraseRef = useRef<ScorePhrase | null>(null);     // the looping phrase
  const pendingPhraseRef = useRef<ScorePhrase | null>(null);     // queued for next boundary swap
  const stepTimeoutRef = useRef<number | null>(null);
  const lastTriggerKeyRef = useRef<string | null>(null);
  const generatingRef = useRef<boolean>(false);
  const enabledRef = useRef<boolean>(enabled);
  enabledRef.current = enabled;

  const stopAll = useCallback(() => {
    if (stepTimeoutRef.current !== null) {
      clearTimeout(stepTimeoutRef.current);
      stepTimeoutRef.current = null;
    }
    handleRef.current?.stop();
    handleRef.current = null;
    cursorRef.current = 0;
    currentPhraseRef.current = null;
    pendingPhraseRef.current = null;
    setCurrentPhrase(null);
    setStatus("idle");
    setError(null);
  }, []);

  // The step function: schedule one iteration of the current phrase, then
  // arrange for itself to fire again shortly before that iteration ends.
  // Promotes a pending phrase to current at each call, so swaps happen
  // exactly at phrase boundaries (= measure boundaries by construction).
  const step = useCallback(() => {
    if (!enabledRef.current) return;

    if (pendingPhraseRef.current) {
      currentPhraseRef.current = pendingPhraseRef.current;
      pendingPhraseRef.current = null;
      setCurrentPhrase(currentPhraseRef.current);
    }

    const phrase = currentPhraseRef.current;
    if (!phrase) {
      setStatus("idle");
      return;
    }

    const ctxNow = getAudioContextTime();
    const start = Math.max(cursorRef.current, ctxNow + 0.05);
    const handle = playPhrase(phrase, { startAt: start });
    handleRef.current = handle;
    cursorRef.current = handle.endsAt;
    setStatus(generatingRef.current ? "generating" : "playing");

    // Re-fire the step shortly before this iteration ends so the next
    // iteration's start time chains seamlessly. 150ms lookahead.
    const msUntilEnd = (handle.endsAt - getAudioContextTime()) * 1000;
    const lookahead = Math.max(50, msUntilEnd - 150);
    stepTimeoutRef.current = window.setTimeout(step, lookahead);
  }, []);

  // Disable / unmount: kill scheduling and silence playback.
  useEffect(() => {
    if (!enabled) stopAll();
    return () => stopAll();
  }, [enabled, stopAll]);

  // Trigger on new assistant message arrival.
  const triggerKey = latestAssistantMessage
    ? `${latestAssistantMessage.message_id}::${latestAssistantMessage.formula_signature ?? ""}`
    : null;

  useEffect(() => {
    if (!enabled) return;
    if (!triggerKey) return;
    if (!apiKey) return;
    if (!latestAssistantMessage) return;
    if (generatingRef.current) return;
    if (lastTriggerKeyRef.current === triggerKey) return;
    const momentstamp = latestAssistantMessage.formula_signature?.trim();
    if (!momentstamp) return;

    lastTriggerKeyRef.current = triggerKey;
    generatingRef.current = true;
    // Don't disrupt the loop — just flip the icon to "generating" while
    // the music keeps playing.
    setStatus(currentPhraseRef.current ? "generating" : "generating");
    setError(null);

    const previousPhrase = currentPhraseRef.current;

    (async () => {
      try {
        const result = await api.generateNextScorePhrase(apiKey, {
          currentLastPhrase: previousPhrase,
          momentstamp,
          moodHint: null,
        });
        const next = result.phrase as ScorePhrase;

        if (currentPhraseRef.current) {
          // Loop is active — queue as pending; step() will swap at the next
          // phrase boundary (= measure boundary).
          pendingPhraseRef.current = next;
          setStatus("playing");
        } else {
          // No loop yet — set as current and kick off the loop.
          currentPhraseRef.current = next;
          setCurrentPhrase(next);
          step();
        }
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        if (!currentPhraseRef.current) setStatus("idle");
        else setStatus("playing");
      } finally {
        generatingRef.current = false;
      }
    })();
  }, [enabled, apiKey, triggerKey, latestAssistantMessage, step]);

  return { status, error, currentPhrase, stopAndReset: stopAll };
}

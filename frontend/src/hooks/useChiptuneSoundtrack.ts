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
  /** The phrase currently playing (rotating cursor over the collection). */
  currentPhrase: ScorePhrase | null;
  /** Total number of phrases accumulated this session. */
  collectionSize: number;
  stopAndReset: () => void;
}

/**
 * Soundtrack lifecycle — circular playlist that grows.
 *
 *   - Every phrase generated from an assistant message's momentstamp is APPENDED
 *     to a collection. The collection accumulates over the conversation and is
 *     never shrunk during the session.
 *   - Playback cycles through the whole collection in FIFO order, wrapping
 *     back to the first phrase after the last. Each iteration plays the phrase
 *     at `playbackIndex % collection.length` and increments.
 *   - When a new phrase arrives mid-cycle, it lands at the END of the collection
 *     and is reached when the playback cursor wraps around to it. New arrivals
 *     do NOT preempt the current iteration — they wait their turn in the cycle.
 *   - Continuation chain: each new generation passes the most recently generated
 *     phrase (the END of the collection) as `currentLastPhrase`, so the chain
 *     of compositional memory follows generation order, not playback order.
 *   - Disable / unmount stops scheduling and silences the active handle. The
 *     collection is RESET on disable (toggle off → start fresh on next on).
 */
export function useChiptuneSoundtrack({
  enabled,
  apiKey,
  latestAssistantMessage,
}: Args): SoundtrackState {
  const [status, setStatus] = useState<SoundtrackStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const [currentPhrase, setCurrentPhrase] = useState<ScorePhrase | null>(null);
  const [collectionSize, setCollectionSize] = useState<number>(0);

  const handleRef = useRef<PlayHandle | null>(null);
  const cursorRef = useRef<number>(0);                            // audio time when current iteration ends
  const phraseCollectionRef = useRef<ScorePhrase[]>([]);          // append-only this session
  const playbackIndexRef = useRef<number>(0);                     // monotone — modulo at read time
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
    phraseCollectionRef.current = [];
    playbackIndexRef.current = 0;
    setCollectionSize(0);
    setCurrentPhrase(null);
    setStatus("idle");
    setError(null);
  }, []);

  // step(): play the phrase at playbackIndex % collection.length, increment,
  // and reschedule self ~150ms before this iteration ends so the next iteration's
  // start chains seamlessly. New phrases appended to the collection are picked
  // up on subsequent iterations as the cycle reaches them.
  const step = useCallback(() => {
    if (!enabledRef.current) return;

    const collection = phraseCollectionRef.current;
    if (collection.length === 0) {
      setStatus("idle");
      return;
    }

    const idx = playbackIndexRef.current % collection.length;
    const phrase = collection[idx];
    setCurrentPhrase(phrase);

    const ctxNow = getAudioContextTime();
    const start = Math.max(cursorRef.current, ctxNow + 0.05);
    const handle = playPhrase(phrase, { startAt: start });
    handleRef.current = handle;
    cursorRef.current = handle.endsAt;
    setStatus(generatingRef.current ? "generating" : "playing");

    // Advance cursor for next iteration. Modulo applied on next read so a
    // mid-cycle collection growth is reflected naturally.
    playbackIndexRef.current = playbackIndexRef.current + 1;

    const msUntilEnd = (handle.endsAt - getAudioContextTime()) * 1000;
    const lookahead = Math.max(50, msUntilEnd - 150);
    stepTimeoutRef.current = window.setTimeout(step, lookahead);
  }, []);

  // Disable / unmount: kill scheduling + reset the collection so the next
  // toggle-on starts a fresh session.
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
    setStatus("generating");
    setError(null);

    // Continuation chain follows GENERATION order: most-recently-generated
    // phrase is at the end of the collection.
    const previousPhrase = phraseCollectionRef.current.length > 0
      ? phraseCollectionRef.current[phraseCollectionRef.current.length - 1]
      : null;

    (async () => {
      try {
        const result = await api.generateNextScorePhrase(apiKey, {
          currentLastPhrase: previousPhrase,
          momentstamp,
          moodHint: null,
        });
        const next = result.phrase as ScorePhrase;

        const wasEmpty = phraseCollectionRef.current.length === 0;
        phraseCollectionRef.current = [...phraseCollectionRef.current, next];
        setCollectionSize(phraseCollectionRef.current.length);

        if (wasEmpty) {
          // First phrase of the session — kick off the loop.
          step();
        } else {
          // Loop is already running; the new phrase will be reached when the
          // cycle wraps. Status returns to 'playing' (loop continues).
          setStatus("playing");
        }
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        setStatus(phraseCollectionRef.current.length > 0 ? "playing" : "idle");
      } finally {
        generatingRef.current = false;
      }
    })();
  }, [enabled, apiKey, triggerKey, latestAssistantMessage, step]);

  return {
    status,
    error,
    currentPhrase,
    collectionSize,
    stopAndReset: stopAll,
  };
}

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

export function useChiptuneSoundtrack({
  enabled,
  apiKey,
  latestAssistantMessage,
}: Args): SoundtrackState {
  const [status, setStatus] = useState<SoundtrackStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const [currentPhrase, setCurrentPhrase] = useState<ScorePhrase | null>(null);

  // Refs hold playback state that must not retrigger effect runs.
  const handleRef = useRef<PlayHandle | null>(null);
  const cursorRef = useRef<number>(0);
  const lastTriggerKeyRef = useRef<string | null>(null);
  const generatingRef = useRef<boolean>(false);

  const stopAndReset = useCallback(() => {
    handleRef.current?.stop();
    handleRef.current = null;
    cursorRef.current = 0;
    lastTriggerKeyRef.current = null;
    setCurrentPhrase(null);
    setStatus("idle");
    setError(null);
  }, []);

  // Disable / unmount: kill any in-flight playback.
  useEffect(() => {
    if (!enabled) {
      handleRef.current?.stop();
      handleRef.current = null;
      cursorRef.current = 0;
      setStatus("idle");
    }
    return () => {
      handleRef.current?.stop();
      handleRef.current = null;
    };
  }, [enabled]);

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
    // Need a non-empty momentstamp to key next-phrase generation. If the
    // message has no formula_signature (reactions=on chats, or pre-feature
    // messages), skip silently — soundtrack remains on the current phrase.
    const momentstamp = latestAssistantMessage.formula_signature?.trim();
    if (!momentstamp) return;

    lastTriggerKeyRef.current = triggerKey;
    generatingRef.current = true;
    setStatus("generating");
    setError(null);

    const previousPhrase = currentPhrase;

    (async () => {
      try {
        const result = await api.generateNextScorePhrase(apiKey, {
          currentLastPhrase: previousPhrase,
          momentstamp,
          moodHint: null,
        });
        const next = result.phrase as ScorePhrase;
        setCurrentPhrase(next);

        const ctxNow = getAudioContextTime();
        const startAt = Math.max(cursorRef.current, ctxNow + 0.05);
        const h = playPhrase(next, { startAt });
        handleRef.current = h;
        cursorRef.current = h.endsAt;
        setStatus("playing");
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        setStatus("idle");
      } finally {
        generatingRef.current = false;
      }
    })();
  }, [enabled, apiKey, triggerKey, latestAssistantMessage, currentPhrase]);

  return { status, error, currentPhrase, stopAndReset };
}

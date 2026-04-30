import { useEffect, useRef, useState, useCallback } from "react";
import { api, type Message } from "@/lib/tauri";
import {
  playPhrase,
  getAudioContextTime,
  type ScorePhrase,
  type PlayHandle,
} from "@/lib/chiptune-synth";

export type SoundtrackStatus = "idle" | "generating" | "playing";

/**
 * Maximum number of phrases retained per chat. Once the collection grows past
 * this cap, oldest phrases are evicted FIFO on each new append. At ~2KB per
 * phrase JSON, 30 phrases ≈ 60KB per chat in localStorage.
 */
const MAX_COLLECTION_SIZE = 30;

interface Args {
  enabled: boolean;
  apiKey: string | null;
  latestAssistantMessage: Message | null;
  /**
   * Per-chat localStorage key for persisting the phrase collection across
   * reload. Pass null to disable persistence (e.g., when chatId is unknown).
   * Solo: `chiptune_collection.${charId}`; Group: `chiptune_collection.group.${chatId}`.
   */
  storageKey?: string | null;
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
 * Soundtrack lifecycle — circular playlist that grows AND persists.
 *
 *   - Every phrase generated from an assistant message's momentstamp is APPENDED
 *     to a per-chat collection. Persisted to localStorage on every append so the
 *     collection survives reload.
 *   - Playback cycles through the collection in FIFO order, wrapping back to
 *     the first phrase after the last. New arrivals don't preempt — they wait
 *     their turn in the cycle.
 *   - Continuation chain follows GENERATION order: each new generation passes
 *     the most-recently-generated phrase (last index of the collection) as
 *     `currentLastPhrase`, regardless of what's currently playing.
 *   - Toggle-off CLEARS the persisted collection (explicit reset). Reload while
 *     toggled-on RESTORES the collection. Switching chats loads the new chat's
 *     persisted collection (does NOT wipe the old chat's).
 */
export function useChiptuneSoundtrack({
  enabled,
  apiKey,
  latestAssistantMessage,
  storageKey = null,
}: Args): SoundtrackState {
  const [status, setStatus] = useState<SoundtrackStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const [currentPhrase, setCurrentPhrase] = useState<ScorePhrase | null>(null);
  const [collectionSize, setCollectionSize] = useState<number>(0);

  const handleRef = useRef<PlayHandle | null>(null);
  const cursorRef = useRef<number>(0);
  const phraseCollectionRef = useRef<ScorePhrase[]>([]);
  const playbackIndexRef = useRef<number>(0);
  const stepTimeoutRef = useRef<number | null>(null);
  const lastTriggerKeyRef = useRef<string | null>(null);
  const generatingRef = useRef<boolean>(false);
  const enabledRef = useRef<boolean>(enabled);
  const storageKeyRef = useRef<string | null>(storageKey);
  const lastLoadedKeyRef = useRef<string | null>(null);
  enabledRef.current = enabled;
  storageKeyRef.current = storageKey;

  // ── Persistence helpers ─────────────────────────────────────────────
  const persistCollection = useCallback(() => {
    const key = storageKeyRef.current;
    if (!key) return;
    try {
      localStorage.setItem(key, JSON.stringify(phraseCollectionRef.current));
    } catch {
      // quota / private mode — silent
    }
  }, []);

  const loadPersistedCollection = useCallback((key: string): ScorePhrase[] => {
    try {
      const raw = localStorage.getItem(key);
      if (!raw) return [];
      const parsed = JSON.parse(raw);
      if (!Array.isArray(parsed)) return [];
      return parsed as ScorePhrase[];
    } catch {
      return [];
    }
  }, []);

  const clearPersistedCollection = useCallback(() => {
    const key = storageKeyRef.current;
    if (!key) return;
    try {
      localStorage.removeItem(key);
    } catch {
      // private mode — silent
    }
  }, []);

  // ── Playback control ────────────────────────────────────────────────
  const stopPlaybackOnly = useCallback(() => {
    if (stepTimeoutRef.current !== null) {
      clearTimeout(stepTimeoutRef.current);
      stepTimeoutRef.current = null;
    }
    handleRef.current?.stop();
    handleRef.current = null;
  }, []);

  const resetInMemory = useCallback(() => {
    stopPlaybackOnly();
    cursorRef.current = 0;
    phraseCollectionRef.current = [];
    playbackIndexRef.current = 0;
    setCollectionSize(0);
    setCurrentPhrase(null);
    setStatus("idle");
    setError(null);
  }, [stopPlaybackOnly]);

  /** Toggle-off / explicit-reset path: wipes in-memory AND persisted. */
  const stopAndReset = useCallback(() => {
    resetInMemory();
    clearPersistedCollection();
  }, [resetInMemory, clearPersistedCollection]);

  // ── The loop step ───────────────────────────────────────────────────
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

    playbackIndexRef.current = playbackIndexRef.current + 1;

    const msUntilEnd = (handle.endsAt - getAudioContextTime()) * 1000;
    const lookahead = Math.max(50, msUntilEnd - 150);
    stepTimeoutRef.current = window.setTimeout(step, lookahead);
  }, []);

  // ── Disable / unmount lifecycle ─────────────────────────────────────
  // Toggle-off explicitly clears persisted state. Unmount only stops playback
  // (persisted state survives so reload + remount restores the collection).
  useEffect(() => {
    if (!enabled) {
      stopAndReset();
    }
    return () => {
      stopPlaybackOnly();
    };
  }, [enabled, stopAndReset, stopPlaybackOnly]);

  // ── Restore on (enabled, storageKey) change ─────────────────────────
  // When the storageKey changes (e.g., user switches chats), reset in-memory
  // (don't wipe the previous chat's persisted data) and restore from the new
  // key. When enabled becomes true with an existing collection, restore.
  useEffect(() => {
    if (!enabled) {
      lastLoadedKeyRef.current = null;
      return;
    }
    if (storageKey === lastLoadedKeyRef.current) return;

    // Storage key changed (or first load). Clear in-memory state of previous
    // chat without wiping its persisted data, then load new chat's.
    resetInMemory();
    lastLoadedKeyRef.current = storageKey;

    if (!storageKey) return;
    const restored = loadPersistedCollection(storageKey);
    if (restored.length === 0) return;

    // Defensive cap on restore (handles older persisted data or cap reduction).
    const capped = restored.length > MAX_COLLECTION_SIZE
      ? restored.slice(restored.length - MAX_COLLECTION_SIZE)
      : restored;

    phraseCollectionRef.current = capped;
    playbackIndexRef.current = 0;
    setCollectionSize(capped.length);
    step();
  }, [enabled, storageKey, resetInMemory, loadPersistedCollection, step]);

  // ── Trigger on new assistant message arrival ────────────────────────
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
        let updated = [...phraseCollectionRef.current, next];

        // FIFO eviction once the cap is reached. Decrement playbackIndex by
        // the eviction count so the loop's next iteration plays the same
        // logical "next phrase" it would have played without eviction —
        // dropping K old elements shifts everyone else down by K positions,
        // so the index has to shift down by K too.
        if (updated.length > MAX_COLLECTION_SIZE) {
          const drops = updated.length - MAX_COLLECTION_SIZE;
          updated = updated.slice(drops);
          playbackIndexRef.current = Math.max(0, playbackIndexRef.current - drops);
        }

        phraseCollectionRef.current = updated;
        setCollectionSize(updated.length);
        persistCollection();

        if (wasEmpty) {
          step();
        } else {
          setStatus("playing");
        }
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        setStatus(phraseCollectionRef.current.length > 0 ? "playing" : "idle");
      } finally {
        generatingRef.current = false;
      }
    })();
  }, [enabled, apiKey, triggerKey, latestAssistantMessage, step, persistCollection]);

  return {
    status,
    error,
    currentPhrase,
    collectionSize,
    stopAndReset,
  };
}

import React, { useRef, useEffect, useState, useCallback, useMemo } from "react";
import Markdown from "react-markdown";
import { formatMessage, markdownComponents, remarkPlugins, rehypePlugins, isEmojiOnlyMessage } from "@/components/chat/formatMessage";
import { parseBackstageSegments } from "@/components/chat/BackstageActionCard";
import { InlineQuestProposalCard } from "@/components/chat/InlineQuestProposalCard";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog } from "@/components/ui/dialog";
import { Send, Loader2, X, BookOpen, MessageSquare, Settings, Image, Trash2, SlidersHorizontal, Pencil, Square, ChevronRight, ChevronDown, Play, Volume2, ArrowRight, Smile, SmilePlus, ScrollText, Package, Sparkles, MessageCircleQuestion, List, MapPin, PanelLeftClose, PanelLeftOpen, Music, VolumeX } from "lucide-react";
import { useChiptuneSoundtrack } from "@/hooks/useChiptuneSoundtrack";
import { invoke } from "@tauri-apps/api/core";
import { ReactionBubbles } from "@/components/chat/ReactionBubbles";
import { ReactionPicker } from "@/components/chat/ReactionPicker";
import { KeepRecordModal } from "@/components/chat/KeepRecordModal";
import { KeepToast } from "@/components/chat/KeepToast";
import type { KeptRecord, MeanwhileEvent, Message } from "@/lib/tauri";
import { MeanwhileCard } from "@/components/chat/MeanwhileCard";
import type { useAppStore } from "@/hooks/use-app-store";
import { api } from "@/lib/tauri";
import { NarrativeMessage } from "@/components/chat/NarrativeMessage";
import { ChatErrorBar } from "@/components/chat/ChatErrorBar";
import { AnimationReadyToast } from "@/components/chat/AnimationReadyToast";
import { InventoryUpdatedToast, buildInventoryDiffSummary, type InventoryUpdateSummary } from "@/components/chat/InventoryUpdatedToast";
import { InventoryUpdateMessage } from "@/components/chat/InventoryUpdateMessage";
import { InventoryUpdateBadge } from "@/components/chat/InventoryUpdateBadge";
import type { InventoryItem, InventoryUpdateRecord } from "@/lib/tauri";
import { ResetConfirmModal } from "@/components/chat/ResetConfirmModal";
import { RemoveVideoConfirmModal } from "@/components/chat/RemoveVideoConfirmModal";
import { IllustrationPickerModal } from "@/components/chat/IllustrationPickerModal";
import { AdjustIllustrationModal } from "@/components/chat/AdjustIllustrationModal";
import { VideoGenerationModal } from "@/components/chat/VideoGenerationModal";
import { IllustrationCarouselModal } from "@/components/chat/IllustrationCarouselModal";
import { AdjustMessageModal } from "@/components/chat/AdjustMessageModal";
import { NarrativePickerModal } from "@/components/chat/NarrativePickerModal";
import { SummaryModal } from "@/components/chat/SummaryModal";
import { FontSizeAdjuster } from "@/components/chat/FontSizeAdjuster";
import { chatFontPx } from "@/lib/chat-font";
import { TimeDivider } from "@/components/chat/TimeDivider";
import { ContextMessage } from "@/components/chat/ContextMessage";
import { PortraitModal } from "@/components/chat/PortraitModal";
import { StoryConsultantModal } from "@/components/chat/StoryConsultantModal";
import { ImaginedChapterModal } from "@/components/chat/ImaginedChapterModal";
import { ImaginedChapterMessage } from "@/components/chat/ImaginedChapterMessage";
import { SettingsUpdateMessage } from "@/components/chat/SettingsUpdateMessage";
import { LocationChangeCard } from "@/components/chat/LocationChangeCard";
import { LocationModal } from "@/components/chat/LocationModal";
import { LocationOpener } from "@/components/chat/LocationOpener";
import { IllustrationMessage } from "@/components/chat/IllustrationMessage";
import { StickyIllustration } from "@/components/chat/StickyIllustration";
import { useChatState } from "@/hooks/use-chat-state";
import { useChatFocusRefresh } from "@/hooks/use-chat-focus-refresh";
import { InventoryStrip } from "@/components/chat/InventoryStrip";
import { ArcadeGameModeHUD } from "@/components/chat/ArcadeGameModeHUD";



interface Props {
  store: ReturnType<typeof useAppStore>;
  onNavigateToCharacter?: (characterId: string) => void;
  /** Focus mode (toggle Cmd+Shift+F at app-level): clamps the transcript column
   *  to a 72ch measure. App.tsx hides Sidebar + nav-rail when this is on;
   *  GroupChatView's job is to apply the column-clamp AND render the
   *  discoverable title-bar toggle button. Mirrored from ChatView per the
   *  parallel-surfaces doctrine. */
  focusMode?: boolean;
  /** Toggle Focus mode from the title-bar button. Mirrored from ChatView. */
  onToggleFocus?: () => void;
}


export function GroupChatView({ store, onNavigateToCharacter, focusMode = false, onToggleFocus }: Props) {
  // ── Group-specific state ─────────────────────────────────────────────
  const [pickerMessageId, setPickerMessageId] = useState<string | null>(null);
  const [keepTargetId, setKeepTargetId] = useState<string | null>(null);
  const [keptIds, setKeptIds] = useState<Set<string>>(new Set());
  const [invUpdatingId, setInvUpdatingId] = useState<string | null>(null);
  const [inventoryToast, setInventoryToast] = useState<InventoryUpdateSummary[] | null>(null);
  const [inventoryBadges, setInventoryBadges] = useState<Record<string, InventoryUpdateRecord[]>>({});

  const handleInventoryUpdateFromMessage = useCallback(async (messageId: string) => {
    setInvUpdatingId(messageId);
    const priorByChar = new Map<string, InventoryItem[]>(
      store.characters.map((c) => [c.character_id, c.inventory ?? []])
    );
    const nameById = new Map<string, string>(
      store.characters.map((c) => [c.character_id, c.display_name])
    );
    try {
      const resp = await store.updateInventoryForMoment(messageId);
      const summaries = resp.results
        .map((r) => buildInventoryDiffSummary(priorByChar.get(r.character_id) ?? [], r.inventory, nameById.get(r.character_id) ?? "A character"))
        .filter((s): s is InventoryUpdateSummary => s !== null);
      if (summaries.length > 0) setInventoryToast(summaries);

      const nowIso = new Date().toISOString();
      const records: InventoryUpdateRecord[] = resp.results
        .filter((r) => (r.added?.length ?? 0) + (r.updated?.length ?? 0) + (r.removed?.length ?? 0) > 0)
        .map((r) => ({
          message_id: messageId,
          character_id: r.character_id,
          character_name: nameById.get(r.character_id) ?? "",
          added: (r.added ?? []).map((i) => i.name),
          updated: (r.updated ?? []).map((i) => i.name),
          removed: r.removed ?? [],
          created_at: nowIso,
        }));
      if (records.length > 0) {
        setInventoryBadges((prev) => ({ ...prev, [messageId]: records }));
      }
    } catch (e) {
      console.warn("[Inventory] moment-update failed", e);
    } finally {
      setInvUpdatingId(null);
    }
  }, [store]);
  // Captions for illustration messages — loaded once per visible window.
  const [illustrationCaptions, setIllustrationCaptions] = useState<Record<string, string>>({});
  useEffect(() => {
    const illusIds = store.messages.filter((m) => m.role === "illustration").map((m) => m.message_id);
    if (illusIds.length === 0) { setIllustrationCaptions({}); return; }
    let cancelled = false;
    api.getIllustrationCaptions(illusIds).then((map) => {
      if (!cancelled) setIllustrationCaptions(map);
    }).catch(() => { if (!cancelled) setIllustrationCaptions({}); });
    return () => { cancelled = true; };
  }, [store.messages]);
  const [keepToast, setKeepToast] = useState<{ entry: KeptRecord; subjectLabel: string } | null>(null);
  const [showGroupTalkPicker, setShowGroupTalkPicker] = useState(false);
  const talkPickerRef = useRef<HTMLDivElement>(null);
  const [portraitModalCharId, setPortraitModalCharId] = useState<string | null>(null);
  const [showConsultant, setShowConsultant] = useState(false);
  // When set, the consultant modal opens AND auto-sends this string as
  // a user message. Used by per-message "How do I react!?" buttons.
  const [consultantAutoSend, setConsultantAutoSend] = useState<string | null>(null);
  const [showImaginedChapter, setShowImaginedChapter] = useState(false);
  const [openImaginedChapterId, setOpenImaginedChapterId] = useState<string | null>(null);
  const [showLocationModal, setShowLocationModal] = useState(false);
  const [currentLocation, setCurrentLocation] = useState<string | null>(null);
  const [showSettingsPopover, setShowSettingsPopover] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [clearKeepMedia, setClearKeepMedia] = useState(true);
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const settingsPopoverRef = useRef<HTMLDivElement>(null);
  const [showGroupPopover, setShowGroupPopover] = useState(false);
  const groupPopoverTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);

  const groupCharIds: string[] = store.activeGroupChat
    ? (Array.isArray(store.activeGroupChat.character_ids) ? store.activeGroupChat.character_ids : [])
    : [];
  const groupCharacters = groupCharIds.map((id) => store.characters.find((c) => c.character_id === id)).filter(Boolean) as typeof store.characters;

  // Inline meanwhile cards (per-world): scoped to all characters in this
  // group chat so every member's off-screen beat can surface here.
  const [meanwhileEvents, setMeanwhileEvents] = useState<MeanwhileEvent[]>([]);
  useEffect(() => {
    if (!store.activeWorld) { setMeanwhileEvents([]); return; }
    let cancelled = false;
    api.listMeanwhileEvents(store.activeWorld.world_id, 200)
      .then((list) => { if (!cancelled) setMeanwhileEvents(list); })
      .catch(() => { if (!cancelled) setMeanwhileEvents([]); });
    return () => { cancelled = true; };
  }, [store.activeWorld?.world_id]);

  const groupCharIdsKey = groupCharIds.join(",");
  const meanwhileBuckets = useMemo(() => {
    const relevantIds = new Set<string>(groupCharIds);
    const scoped = meanwhileEvents
      .filter((e) => relevantIds.has(e.character_id))
      .slice()
      .sort((a, b) => a.created_at.localeCompare(b.created_at));
    const before = new Map<string, MeanwhileEvent[]>();
    const trailing: MeanwhileEvent[] = [];
    const filteredMsgs = store.messages.filter((m) => m.content || m.role === "illustration");
    for (const ev of scoped) {
      const firstLater = filteredMsgs.find((m) => m.created_at > ev.created_at);
      if (firstLater) {
        const arr = before.get(firstLater.message_id) ?? [];
        arr.push(ev);
        before.set(firstLater.message_id, arr);
      } else {
        trailing.push(ev);
      }
    }
    return { before, trailing };
  }, [meanwhileEvents, store.messages, groupCharIdsKey]);

  // ── Shared chat state from hook ──────────────────────────────────────
  const chatId = store.activeGroupChat?.group_chat_id;

  const {
    inputValueRef, hasInput, setHasInput,
    scrollRef,
    inputRef,
    userAvatarUrl,
    copiedError, setCopiedError,
    resetConfirmId, setResetConfirmId,
    adjustIllustrationId, setAdjustIllustrationId,
    adjustInstructions, setAdjustInstructions,
    videoModalId, setVideoModalId,
    videoPrompt, setVideoPrompt,
    videoDuration, setVideoDuration,
    videoStyle, setVideoStyle,
    videoIncludeContext, setVideoIncludeContext,
    videoTab, setVideoTab,
    uploadingVideo, setUploadingVideo,
    downloadedId, setDownloadedId,
    removeVideoConfirmId, setRemoveVideoConfirmId,
    animationReadyId, setAnimationReadyId,
    speakingId, setSpeakingId,
    loadingSpeech,
    toneMenuId, setToneMenuId,
    cachedTones, setCachedTones,
    lastTones, setLastTones,
    audioRef,
    toneMenuRef,
    illustrationModalId, setIllustrationModalId,
    modalSelectedId, setModalSelectedId,
    modalPlayingVideo, setModalPlayingVideo,
    modalImageLoading, setModalImageLoading,
    modalIllustrations, setModalIllustrations,
    showNarrativePicker, setShowNarrativePicker,
    showSummary, setShowSummary,
    adjustMessageId, setAdjustMessageId, adjustEditOnly, setAdjustEditOnly,
    showIllustrationPicker, setShowIllustrationPicker,
    illustrationInstructions, setIllustrationInstructions,
    usePreviousScene, setUsePreviousScene,
    includeSceneSummary, setIncludeSceneSummary,
    narrationTone, setNarrationTone,
    narrationInstructions, setNarrationInstructions,
    responseLength, setResponseLength,
    reactionsMode, setReactionsMode,
    arcadeGameMode, setArcadeGameMode,
    narrationDirty, setNarrationDirty,
    playingVideo, setPlayingVideo,
    loopVideo, setLoopVideo,
    videoFiles, setVideoFiles,
    videoDataUrls, setVideoDataUrls,
    showUserAvatarModal, setShowUserAvatarModal,
    carouselAllMessages, setCarouselAllMessages,
    isAtBottom,

    isSending,
    isGeneratingNarrative,
    isGeneratingIllustration,
    isGeneratingVideo,

    modalSlideshow,

    loadVideoBlobUrl,
    handleSpeak,
    handleKeyDown,
    handleSend,
    handleRetry,
    playVideo,
  } = useChatState({ store, chatId, chatType: "group" });

  // Chiptune soundtrack (Phase 4 — group-chat parity with ChatView). Opt-in
  // toggle persists per-group-chat in localStorage. Latest assistant message's
  // formula_signature feeds the AI score generator; phrases chain at the tail
  // of current playback.
  const soundtrackPrefKey = `chiptune_soundtrack.group.${chatId ?? ""}`;
  const [soundtrackEnabled, setSoundtrackEnabled] = useState<boolean>(() => {
    try { return localStorage.getItem(soundtrackPrefKey) === "on"; }
    catch { return false; }
  });
  const toggleSoundtrack = () => {
    setSoundtrackEnabled((v) => {
      const next = !v;
      try { localStorage.setItem(soundtrackPrefKey, next ? "on" : "off"); } catch { /* private mode */ }
      return next;
    });
  };
  const latestAssistantMessage = (() => {
    const msgs = store.messages;
    for (let i = msgs.length - 1; i >= 0; i--) {
      if (msgs[i].role === "assistant") return msgs[i];
    }
    return null;
  })();
  const soundtrack = useChiptuneSoundtrack({
    enabled: soundtrackEnabled,
    apiKey: store.apiKey ?? null,
    latestAssistantMessage,
    storageKey: chatId ? `chiptune_collection.group.${chatId}` : null,
  });

  // Focus-trigger: fan out to all group members' inventories on user
  // engagement. Backend runs one LLM call per overdue member in
  // parallel; fresh members no-op.
  const chatContainerRef = useRef<HTMLDivElement>(null);
  useChatFocusRefresh({
    chatKey: chatId,
    containerRef: chatContainerRef,
    onFocusRefresh: () => {
      if (!chatId) return;
      store.refreshGroupInventories(chatId);
      // Per-member journal auto-gen. Each backend call short-circuits
      // if today's entry already exists for that character.
      if (store.apiKey) {
        for (const cid of groupCharIds) {
          api.maybeGenerateCharacterJournal(store.apiKey, cid).catch(() => {});
        }
      }
      // Per-world meanwhile auto-gen. Refetch after to pick up new events
      // in inline chat history.
      if (store.apiKey && store.activeWorld) {
        const wid = store.activeWorld.world_id;
        api.maybeGenerateMeanwhileEvents(store.apiKey, wid)
          .then(() => api.listMeanwhileEvents(wid, 200))
          .then(setMeanwhileEvents)
          .catch(() => {});
      }
      // Per-world daily reading auto-gen (short-circuits if today's
      // reading already exists; two-pass chain with self-critique).
      if (store.apiKey && store.activeWorld) {
        api.maybeGenerateDailyReading(store.apiKey, store.activeWorld.world_id).catch(() => {});
      }
      // Player's own journal: per-world, once per world-day.
      if (store.apiKey && store.activeWorld) {
        api.maybeGenerateUserJournal(store.apiKey, store.activeWorld.world_id).catch(() => {});
      }
    },
  });

  // Close settings popover on outside click
  useEffect(() => {
    if (!showSettingsPopover) return;
    const handler = (e: MouseEvent) => {
      if (settingsPopoverRef.current && !settingsPopoverRef.current.contains(e.target as Node)) {
        setShowSettingsPopover(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showSettingsPopover]);

  // Fetch the thread's mood-reduction ring when the settings popover opens.
  const [moodReduction, setMoodReduction] = useState<string[]>([]);
  useEffect(() => {
    if (!showSettingsPopover || !chatId) return;
    let cancelled = false;
    api.getMoodReduction({ groupChatId: chatId }).then((r) => {
      if (!cancelled) setMoodReduction(r);
    }).catch(() => {
      if (!cancelled) setMoodReduction([]);
    });
    return () => { cancelled = true; };
  }, [showSettingsPopover, chatId, store.messages.length]);

  // Kept message IDs for the thread — drives the indicator on
  // kept messages and refreshes after a save.
  const keepThreadId = store.messages.find((m) => m.thread_id)?.thread_id ?? null;
  const reloadKept = useCallback(async () => {
    if (!keepThreadId) { setKeptIds(new Set()); return; }
    try {
      const ids = await api.listKeptMessageIds(keepThreadId);
      setKeptIds(new Set(ids));
    } catch {
      setKeptIds(new Set());
    }
  }, [keepThreadId]);
  useEffect(() => { reloadKept(); }, [reloadKept]);

  const inventoryMsgIdsKey = store.messages.map((m) => m.message_id).join(",");
  useEffect(() => {
    const ids = store.messages.map((m) => m.message_id).filter(Boolean);
    if (ids.length === 0) { setInventoryBadges({}); return; }
    let cancelled = false;
    api.getInventoryUpdatesForMessages(ids).then((records) => {
      if (cancelled) return;
      const map: Record<string, InventoryUpdateRecord[]> = {};
      for (const r of records) {
        (map[r.message_id] = map[r.message_id] ?? []).push(r);
      }
      setInventoryBadges(map);
    }).catch(() => {});
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [inventoryMsgIdsKey]);


  // Per-chat provider override: "" (use global) | "lmstudio" | "openai".
  const [providerOverride, setProviderOverride] = useState<string>("");
  useEffect(() => {
    if (!chatId) return;
    let cancelled = false;
    api.getSetting(`provider_override.${chatId}`).then((v) => {
      if (!cancelled) setProviderOverride(v ?? "");
    }).catch(() => { if (!cancelled) setProviderOverride(""); });
    return () => { cancelled = true; };
  }, [chatId]);
  // Shared helper for atomic dropdown/toggle chat-settings changes —
  // writes a settings_update row into group_messages so the change
  // surfaces in chat history (and reaches the LLM through the dialogue
  // prompt's history block). Text-input settings below use the
  // debounced diff path instead. Both land in the same table as
  // settings_update rows.
  const recordSettingChange = useCallback(async (key: string, label: string, from: string, to: string) => {
    if (from === to) return;
    const threadId = store.messages.find((m) => m.thread_id)?.thread_id ?? null;
    if (!threadId) return;
    try {
      const msg = await api.recordChatSettingsChange(threadId, [{ key, label, from, to }], true);
      store.appendMessage(msg);
    } catch { /* best-effort */ }
  }, [store]);

  const setProviderOverridePersist = useCallback((next: string) => {
    const prev = providerOverride;
    setProviderOverride(next);
    if (chatId) api.setSetting(`provider_override.${chatId}`, next).catch(() => {});
    const fmt = (v: string) => v === "lmstudio" ? "LM Studio" : v === "openai" ? "OpenAI" : "Default";
    recordSettingChange("provider_override", "Model Provider", fmt(prev), fmt(next));
  }, [chatId, providerOverride, recordSettingChange]);

  // Leader setting: "user" or a character_id.
  const [leader, setLeader] = useState<string>("user");
  useEffect(() => {
    if (!chatId) return;
    let cancelled = false;
    api.getSetting(`leader.${chatId}`).then((v) => {
      if (!cancelled) setLeader(v && v !== "" ? v : "user");
    }).catch(() => {
      if (!cancelled) setLeader("user");
    });
    return () => { cancelled = true; };
  }, [chatId]);
  const setLeaderPersist = useCallback((next: string) => {
    const prev = leader;
    setLeader(next);
    if (chatId) api.setSetting(`leader.${chatId}`, next).catch(() => {});
    // Format: "user" → "You"; otherwise the character's display name
    // (resolved against the group's cast).
    const fmt = (v: string) => {
      if (v === "user") return "You";
      const c = store.characters.find((c) => c.character_id === v);
      return c?.display_name ?? v;
    };
    recordSettingChange("leader", "Leader", fmt(prev), fmt(next));
  }, [chatId, leader, store.characters, recordSettingChange]);

  // Send conversation history: default ON. When OFF, each responding
  // character sees only the system prompt + the triggering message.
  const [sendHistory, setSendHistory] = useState<boolean>(true);
  useEffect(() => {
    if (!chatId) return;
    let cancelled = false;
    api.getSetting(`send_history.${chatId}`).then((v) => {
      if (!cancelled) setSendHistory(v !== "off" && v !== "false");
    }).catch(() => { if (!cancelled) setSendHistory(true); });
    return () => { cancelled = true; };
  }, [chatId]);

  // Per-group-chat current location. Reset immediately on chat switch
  // so the LocationOpener never carries stale data into the new chat.
  useEffect(() => {
    setCurrentLocation(null);
    if (!chatId) return;
    let cancelled = false;
    invoke<string | null>("get_chat_location_cmd", { groupChatId: chatId })
      .then((loc) => { if (!cancelled) setCurrentLocation(loc); })
      .catch(() => { if (!cancelled) setCurrentLocation(null); });
    return () => { cancelled = true; };
  }, [chatId]);

  const setSendHistoryPersist = useCallback((next: boolean) => {
    const prev = sendHistory;
    setSendHistory(next);
    if (chatId) api.setSetting(`send_history.${chatId}`, next ? "on" : "off").catch(() => {});
    recordSettingChange("send_history", "Send History", prev ? "On" : "Off", next ? "On" : "Off");
  }, [chatId, sendHistory, recordSettingChange]);

  // Snapshot of last-saved settings for diff. See ChatView for full
  // rationale; same pattern here for groups.
  const lastSavedSettingsRef = useRef<{
    narrationTone: string;
    narrationInstructions: string;
    responseLength: string;
    reactionsMode: "off" | "occasional" | "always";
    chatId: string;
  } | null>(null);
  useEffect(() => {
    if (!chatId) { lastSavedSettingsRef.current = null; return; }
    if (!narrationDirty) {
      lastSavedSettingsRef.current = { narrationTone, narrationInstructions, responseLength, reactionsMode, chatId };
    }
  }, [chatId, narrationDirty, narrationTone, narrationInstructions, responseLength, reactionsMode]);

  // Auto-save chat settings + record a settings_update message in chat
  // history when something actually changed.
  useEffect(() => {
    if (!chatId || !narrationDirty) return;
    const timer = setTimeout(async () => {
      await Promise.all([
        api.setSetting(`narration_tone.${chatId}`, narrationTone),
        api.setSetting(`narration_instructions.${chatId}`, narrationInstructions),
        api.setSetting(`response_length.${chatId}`, responseLength),
        api.setSetting(`reactions_enabled.${chatId}`, reactionsMode),
      ]);
      const prev = lastSavedSettingsRef.current;
      if (prev && prev.chatId === chatId) {
        const changes: Array<{ key: string; label: string; from: string; to: string }> = [];
        if (prev.responseLength !== responseLength) {
          changes.push({ key: "response_length", label: "Response Length", from: prev.responseLength, to: responseLength });
        }
        if (prev.narrationTone !== narrationTone) {
          changes.push({ key: "narration_tone", label: "Narration Tone", from: prev.narrationTone, to: narrationTone });
        }
        if (prev.narrationInstructions !== narrationInstructions) {
          const abbrev = (s: string) => {
            const t = s.trim();
            if (t.length === 0) return "(none)";
            return t.length > 60 ? t.slice(0, 57) + "…" : t;
          };
          changes.push({ key: "narration_instructions", label: "Narration Instructions", from: abbrev(prev.narrationInstructions), to: abbrev(narrationInstructions) });
        }
        if (prev.reactionsMode !== reactionsMode) {
          const labelFor = (m: string) => m === "always" ? "Always" : m === "occasional" ? "Occasionally" : "Off";
          changes.push({ key: "reactions_enabled", label: "Character Reactions", from: labelFor(prev.reactionsMode), to: labelFor(reactionsMode) });
        }
        if (changes.length > 0) {
          const threadId = store.messages.find((m) => m.thread_id)?.thread_id ?? null;
          if (threadId) {
            try {
              const msg = await api.recordChatSettingsChange(threadId, changes, true);
              store.appendMessage(msg);
            } catch { /* best-effort */ }
          }
        }
      }
      lastSavedSettingsRef.current = { narrationTone, narrationInstructions, responseLength, reactionsMode, chatId };
      setNarrationDirty(false);
    }, 600);
    return () => clearTimeout(timer);
  }, [narrationTone, narrationInstructions, responseLength, reactionsMode, narrationDirty, chatId, store]);

  const insertEmojiAtCursor = useCallback((emoji: string) => {
    const ta = inputRef.current;
    if (!ta) return;
    // When the textarea isn't the active element, its selection range is
    // stale — treat unfocused as "append to end" so the button works
    // before the user has placed a caret.
    const isFocused = document.activeElement === ta;
    const start = isFocused ? (ta.selectionStart ?? ta.value.length) : ta.value.length;
    const end = isFocused ? (ta.selectionEnd ?? ta.value.length) : ta.value.length;
    const next = ta.value.slice(0, start) + emoji + ta.value.slice(end);
    ta.value = next;
    inputValueRef.current = next;
    const caret = start + emoji.length;
    ta.focus();
    ta.setSelectionRange(caret, caret);
    ta.style.height = "auto";
    ta.style.height = `${ta.scrollHeight}px`;
    const empty = !next.trim();
    if (hasInput === empty) setHasInput(!empty);
  }, [inputRef, inputValueRef, hasInput, setHasInput]);

  // Backstage action: "Stage in chat" — AI-drafted message text gets
  // placed in this chat's input when the user accepts the action card.
  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail as { threadId?: string; text?: string };
      if (!detail?.text) return;
      const myThread = store.messages[0]?.thread_id;
      if (detail.threadId && myThread && detail.threadId !== myThread) return;
      const ta = inputRef.current;
      if (!ta) return;
      ta.value = detail.text;
      inputValueRef.current = detail.text;
      ta.focus();
      ta.setSelectionRange(detail.text.length, detail.text.length);
      ta.style.height = "auto";
      ta.style.height = `${ta.scrollHeight}px`;
      setHasInput(!!detail.text.trim());
    };
    window.addEventListener("backstage:stage-message", handler as EventListener);
    return () => window.removeEventListener("backstage:stage-message", handler as EventListener);
  }, [inputRef, inputValueRef, setHasInput, store.messages]);

  // Listen for Backstage's preview→attach illustration flow. See ChatView
  // for the full rationale; same pattern here for groups.
  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent<{ threadId: string; message?: Message }>).detail;
      if (!detail?.message) return;
      store.appendMessage(detail.message);
    };
    window.addEventListener("backstage:illustration-added", handler as EventListener);
    return () => window.removeEventListener("backstage:illustration-added", handler as EventListener);
  }, [store]);

  // Listen for imagined-chapter canonize so the breadcrumb card shows
  // up immediately without needing to leave and re-enter the chat.
  useEffect(() => {
    const handler = async (e: Event) => {
      const detail = (e as CustomEvent<{ threadId: string }>).detail;
      const myThread = store.messages[0]?.thread_id;
      if (!detail?.threadId || !myThread || detail.threadId !== myThread) return;
      await store.reloadActiveChatMessages();
    };
    window.addEventListener("imagined-chapter-canonized", handler as EventListener);
    window.addEventListener("imagined-chapter-updated", handler as EventListener);
    return () => {
      window.removeEventListener("imagined-chapter-canonized", handler as EventListener);
      window.removeEventListener("imagined-chapter-updated", handler as EventListener);
    };
  }, [store]);

  const openGallery = useCallback(async (startMessageId?: string) => {
    // Fetch the full thread page first — illustrations earlier in the
    // history (outside the loaded paginated window of store.messages)
    // should still be reachable from the gallery button. Seed priority:
    // explicit startMessageId (sticky-illustration thumbnail click) →
    // most recent illustration in the loaded slice → most recent
    // illustration in the full fetched page. Falls back to no-op when
    // the thread genuinely has zero illustrations.
    if (!store.activeGroupChat) return;
    try {
      const page = await api.getGroupMessages(store.activeGroupChat.group_chat_id);
      const illus = page.messages.filter((m) => m.role === "illustration").map((m) => ({ id: m.message_id, content: m.content }));
      const lastLoadedIllus = store.messages.filter((m) => m.role === "illustration").at(-1);
      const seedId = startMessageId ?? lastLoadedIllus?.message_id ?? illus.at(-1)?.id;
      if (!seedId) return;
      setIllustrationModalId(seedId);
      setModalSelectedId(seedId);
      setModalPlayingVideo(false);
      setModalImageLoading(false);
      setModalIllustrations(illus);
      setCarouselAllMessages(page.messages);
      for (const il of illus) {
        if (!videoFiles[il.id]) api.getVideoFile(il.id).then((vf) => { if (vf) setVideoFiles((prev) => ({ ...prev, [il.id]: vf })); }).catch(() => {});
      }
    } catch {}
  }, [store.messages, store.activeGroupChat, videoFiles]);

  useEffect(() => {
    const onGallery = (e: Event) => {
      const detail = (e as CustomEvent<{ messageId?: string }>).detail;
      openGallery(detail?.messageId);
    };
    const onConsultant = () => setShowConsultant(true);
    const onSummary = () => setShowSummary(true);
    const onSettings = () => setShowSettingsPopover(true);
    window.addEventListener("wt:open-gallery", onGallery);
    window.addEventListener("wt:open-consultant", onConsultant);
    window.addEventListener("wt:open-summary", onSummary);
    window.addEventListener("wt:open-settings", onSettings);
    return () => {
      window.removeEventListener("wt:open-gallery", onGallery);
      window.removeEventListener("wt:open-consultant", onConsultant);
      window.removeEventListener("wt:open-summary", onSummary);
      window.removeEventListener("wt:open-settings", onSettings);
    };
  }, [openGallery]);

  // ── Group talk picker outside-click ──────────────────────────────────
  useEffect(() => {
    if (!showGroupTalkPicker) return;
    const handler = (e: MouseEvent) => {
      if (talkPickerRef.current && !talkPickerRef.current.contains(e.target as Node)) {
        setShowGroupTalkPicker(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showGroupTalkPicker]);

  if (!store.activeGroupChat) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <p>Select or create a group chat to start chatting</p>
      </div>
    );
  }

  return (
    <div ref={chatContainerRef} className="flex-1 flex flex-col min-h-0 relative">
      <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden">
        {(() => {
          const n = groupCharacters.length;
          if (n === 0) return null;
          const slotWidth = 100 / n;
          // Overlap zone (% of container width) that each neighboring pair
          // cross-fades through. Wider overlap = softer seam; narrower =
          // crisper separation. 12% of container total feels balanced for
          // two portraits — neither bleeds into the center, neither has a
          // hard edge.
          const overlap = Math.min(slotWidth * 0.4, 12);
          return groupCharacters.map((ch, i) => {
            const p = store.activePortraits[ch.character_id];
            if (!p?.data_url) {
              return (
                <div
                  key={ch.character_id}
                  className="absolute inset-y-0"
                  style={{
                    left: `${i * slotWidth}%`,
                    width: `${slotWidth}%`,
                    backgroundColor: ch.avatar_color,
                  }}
                />
              );
            }
            const isFirst = i === 0;
            const isLast = i === n - 1;
            // Extend each image into neighbor slots by `overlap/2` on the
            // sides that have a neighbor, so the two neighboring images
            // share an `overlap`-wide cross-fade zone.
            const bleedL = isFirst ? 0 : overlap / 2;
            const bleedR = isLast ? 0 : overlap / 2;
            const leftPct = i * slotWidth - bleedL;
            const widthPct = slotWidth + bleedL + bleedR;
            // Fade zones as % of the image itself (not the container).
            const leftFadePct = (overlap / widthPct) * 100;
            const rightFadePct = 100 - (overlap / widthPct) * 100;
            const stops: string[] = [];
            if (isFirst) stops.push("black 0%");
            else { stops.push("transparent 0%"); stops.push(`black ${leftFadePct}%`); }
            if (isLast) stops.push("black 100%");
            else { stops.push(`black ${rightFadePct}%`); stops.push("transparent 100%"); }
            const mask = `linear-gradient(to right, ${stops.join(", ")})`;
            return (
              <img
                key={ch.character_id}
                src={p.data_url}
                alt=""
                className="absolute inset-y-0 h-full object-cover"
                style={{
                  left: `${leftPct}%`,
                  width: `${widthPct}%`,
                  maskImage: mask,
                  WebkitMaskImage: mask,
                }}
              />
            );
          });
        })()}
        <div className="absolute inset-0 bg-background/65" />
      </div>
      <div className="px-4 py-3 border-b border-border flex items-center gap-3 relative z-30 bg-background">
        {onToggleFocus && (
          <div className="relative group/focus flex-shrink-0">
            <button
              onClick={onToggleFocus}
              title={focusMode ? "Leave Focus" : "Enter Focus"}
              aria-label={focusMode ? "Leave Focus" : "Enter Focus"}
              className="flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent"
            >
              {focusMode ? <PanelLeftOpen size={15} /> : <PanelLeftClose size={15} />}
            </button>
            <span className="absolute top-full left-0 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/focus:opacity-100 pointer-events-none transition-opacity z-50">{focusMode ? "Leave Focus" : "Enter Focus"}</span>
          </div>
        )}
        <div
          className="relative flex items-center gap-3 cursor-pointer"
          onMouseEnter={() => { if (groupPopoverTimeout.current) clearTimeout(groupPopoverTimeout.current); setShowGroupPopover(true); }}
          onMouseLeave={() => { groupPopoverTimeout.current = setTimeout(() => setShowGroupPopover(false), 300); }}
        >
          <div className="flex -space-x-2 flex-shrink-0">
            {groupCharacters.map((ch, i) => {
              const p = store.activePortraits[ch.character_id];
              return p?.data_url ? (
                <img key={ch.character_id} src={p.data_url} alt="" className="w-9 h-9 rounded-full object-cover ring-2 ring-background" style={{ zIndex: groupCharacters.length - i }} />
              ) : (
                <span key={ch.character_id} className="w-9 h-9 rounded-full ring-2 ring-background" style={{ backgroundColor: ch.avatar_color, zIndex: groupCharacters.length - i }} />
              );
            })}
          </div>
          <h1 className="font-semibold">{store.activeGroupChat?.display_name}</h1>
          {showGroupPopover && (
            <div
              className="absolute left-0 top-full mt-2 z-50 w-[540px] bg-card border border-border rounded-xl shadow-2xl shadow-black/40 overflow-hidden animate-in fade-in zoom-in-95 duration-150"
              onMouseEnter={() => { if (groupPopoverTimeout.current) clearTimeout(groupPopoverTimeout.current); }}
              onMouseLeave={() => { groupPopoverTimeout.current = setTimeout(() => setShowGroupPopover(false), 300); }}
            >
              <div className="grid grid-cols-2 divide-x divide-border">
                {groupCharacters.map((ch) => {
                  const portrait = store.activePortraits[ch.character_id];
                  return (
                    <div key={ch.character_id} className="p-3">
                      <div className="flex flex-col items-center mb-2">
                        {portrait?.data_url ? (
                          <img src={portrait.data_url} alt="" className="w-16 h-16 rounded-full object-cover ring-2 ring-border" />
                        ) : (
                          <div className="w-16 h-16 rounded-full ring-2 ring-white/10" style={{ backgroundColor: ch.avatar_color }} />
                        )}
                        <p className="font-semibold text-sm mt-2">{ch.display_name}</p>
                      </div>
                      {ch.identity && (
                        <div className="max-h-32 overflow-y-auto">
                          <p className="text-xs text-muted-foreground leading-relaxed whitespace-pre-wrap">
                            {ch.identity}
                          </p>
                        </div>
                      )}
                      {ch.inventory && ch.inventory.length > 0 && (
                        <div className="mt-2 pt-2 border-t border-border/30 max-h-72 overflow-y-auto">
                          <InventoryStrip inventory={ch.inventory} compact />
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
        <div className="flex items-center gap-1 flex-shrink-0">
          <div className="relative group/locbtn">
            <button
              onClick={() => setShowLocationModal(true)}
              className="flex-shrink-0 w-7 h-7 rounded-lg flex items-center justify-center transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent"
            >
              <MapPin size={14} />
            </button>
            <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/locbtn:opacity-100 pointer-events-none transition-opacity z-50">{currentLocation ? "Change location" : "Set location"}</span>
          </div>
          {currentLocation && (
            <span className="text-[11px] text-muted-foreground/80 max-w-[200px] truncate">
              <span className="text-muted-foreground/50">Location:</span> <span className="text-foreground/70">{currentLocation}</span>
            </span>
          )}
        </div>
        <div className="ml-auto relative group/gallery">
          <button
            onClick={() => openGallery()}
            className="flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent"
          >
            <Image size={15} />
          </button>
          <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/gallery:opacity-100 pointer-events-none transition-opacity">Gallery</span>
        </div>
        <div className="relative group/summary">
          <button
            onClick={() => setShowSummary(true)}
            className="flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent"
          >
            <List size={15} />
          </button>
          <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/summary:opacity-100 pointer-events-none transition-opacity">Summary</span>
        </div>
        <div className="relative group/chapter">
          <button
            onClick={() => { setOpenImaginedChapterId(null); setShowImaginedChapter(true); }}
            className="flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent"
          >
            <BookOpen size={15} />
          </button>
          <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/chapter:opacity-100 pointer-events-none transition-opacity">Imagine</span>
        </div>
        <div className="relative group/consultant">
          <button
            onClick={() => setShowConsultant(true)}
            className="flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent"
          >
            <Sparkles size={15} />
          </button>
          <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/consultant:opacity-100 pointer-events-none transition-opacity">Consultant</span>
        </div>
      </div>

      <div className="flex-1 relative overflow-hidden z-10">
        <LocationOpener
          key={`opener-${chatId ?? "none"}`}
          location={currentLocation}
          worldDay={store.activeWorld?.state.time?.day_index ?? null}
          worldTime={store.activeWorld?.state.time?.time_of_day ?? null}
          loading={store.loadingChat}
        />
        <ScrollArea ref={scrollRef} className="h-full px-4 py-3">
        {arcadeGameMode && chatId && (
          <div className="sticky top-0 z-20 -mx-4 px-4 pt-0 pb-1 mb-1 bg-gradient-to-b from-background via-background/95 to-transparent">
            <ArcadeGameModeHUD chatId={chatId} messages={store.messages} />
          </div>
        )}
        {/* Focus mode: clamp the transcript to a 72ch column for long-form
            readability. Mirrored from ChatView per the parallel-surfaces
            doctrine — chat features that touch the message list must
            update both surfaces. */}
        <div className={focusMode ? "max-w-[72ch] mx-auto" : undefined}>
        {store.messages.length === 0 && (
          store.loadingChat ? (
            <div className="flex items-center justify-center py-16">
              <Loader2 size={24} className="animate-spin text-muted-foreground" />
            </div>
          ) : (
            <div className="text-center text-muted-foreground py-12">
              <p className="text-lg mb-1">Start a conversation</p>
              <p className="text-sm">
                Send a message to {store.activeGroupChat?.display_name}
              </p>
            </div>
          )
        )}
        <div className="space-y-3 max-w-2xl mx-auto">
          {store.messages.length > 0 && store.messages.length < store.totalMessages && (
            <div className="flex justify-center pb-2">
              <button
                onClick={() => store.loadEarlierMessages()}
                disabled={store.loadingOlder}
                className="px-3 py-1.5 text-xs rounded-full bg-card/80 border border-border hover:bg-accent transition-colors cursor-pointer text-muted-foreground hover:text-foreground disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {store.loadingOlder ? "Loading..." : `Load earlier messages (${store.totalMessages - store.messages.length} more)`}
              </button>
            </div>
          )}
          {store.messages.filter((m) => m.content || m.role === "illustration").map((msg, msgIdx, filteredMsgs) => {
            const isUser = msg.role === "user";
            const isNarrative = msg.role === "narrative";
            const isPending = msg.message_id.startsWith("pending-");
            const prevMsg = msgIdx > 0 ? filteredMsgs[msgIdx - 1] : undefined;
            const reactions = store.reactions[msg.message_id] ?? [];
            const showPicker = pickerMessageId === msg.message_id;

            // Any meanwhile events chronologically preceding this message
            // — render as inline cards before the message itself.
            const priorMeanwhiles = meanwhileBuckets.before.get(msg.message_id) ?? [];
            const meanwhileBefore = priorMeanwhiles.map((ev) => (
              <MeanwhileCard
                key={`mw-${ev.event_id}`}
                event={ev}
                portraitUrl={store.activePortraits[ev.character_id]?.data_url}
              />
            ));

            if (isNarrative) {
              return (<React.Fragment key={msg.message_id}>
                {meanwhileBefore}
                <TimeDivider current={msg} previous={prevMsg} />
                <NarrativeMessage
                  message={msg}
                  isPending={isPending}
                  onResetToHere={(id) => setResetConfirmId(id)}
                  cachedTones={cachedTones[msg.message_id]}
                  lastTone={lastTones[msg.message_id]}
                  speakingId={speakingId}
                  loadingSpeech={loadingSpeech}
                  toneMenuId={toneMenuId}
                  setToneMenuId={setToneMenuId}
                  onSpeak={(id, text, tone) => handleSpeak(id, text, groupCharacters[0]?.character_id ?? "", tone)}
                  onStopSpeaking={() => { audioRef.current?.pause(); setSpeakingId(null); }}
                  onDeleteAudio={async (id) => { await api.deleteMessageAudio(id); setCachedTones((prev) => { const next = { ...prev }; delete next[id]; return next; }); setLastTones((prev) => { const next = { ...prev }; delete next[id]; return next; }); }}
                  toneMenuRef={toneMenuRef}
                  adjustingMessageId={store.adjustingMessageId}
                  onAdjust={(id) => setAdjustMessageId(id)}
                  onKeep={(id) => setKeepTargetId(id)}
                  isKept={keptIds.has(msg.message_id)}
                  onInventoryUpdate={handleInventoryUpdateFromMessage}
                  inventoryUpdatingId={invUpdatingId}
                  inventoryUpdateRecords={inventoryBadges[msg.message_id]}
                  onDelete={(id) => store.deleteMessage(id)}
                  chatFontSize={store.chatFontSize}
                />
              </React.Fragment>);
            }

            if (msg.role === "context") {
              return (<React.Fragment key={msg.message_id}>
                {meanwhileBefore}
                <TimeDivider current={msg} previous={prevMsg} />
                <ContextMessage
                  message={msg}
                  isPending={isPending}
                  onResetToHere={(id) => setResetConfirmId(id)}
                  adjustingMessageId={store.adjustingMessageId}
                  onAdjust={(id) => setAdjustMessageId(id)}
                />
              </React.Fragment>);
            }

            if (msg.role === "inventory_update") {
              return (<React.Fragment key={msg.message_id}>
                {meanwhileBefore}
                <TimeDivider current={msg} previous={prevMsg} />
                <InventoryUpdateMessage message={msg} />
              </React.Fragment>);
            }

            if (msg.role === "imagined_chapter") {
              return (<React.Fragment key={msg.message_id}>
                {meanwhileBefore}
                <TimeDivider current={msg} previous={prevMsg} />
                <ImaginedChapterMessage
                  message={msg}
                  onOpen={(chapterId) => {
                    setOpenImaginedChapterId(chapterId);
                    setShowImaginedChapter(true);
                  }}
                />
              </React.Fragment>);
            }

            if (msg.role === "settings_update") {
              return (<React.Fragment key={msg.message_id}>
                {meanwhileBefore}
                <TimeDivider current={msg} previous={prevMsg} />
                <SettingsUpdateMessage message={msg} />
              </React.Fragment>);
            }

            if (msg.role === "location_change") {
              return (<React.Fragment key={msg.message_id}>
                {meanwhileBefore}
                <TimeDivider current={msg} previous={prevMsg} />
                <LocationChangeCard message={msg} />
              </React.Fragment>);
            }

            if (msg.role === "illustration") {
              return (<React.Fragment key={msg.message_id}>
                {meanwhileBefore}
                <TimeDivider current={msg} previous={prevMsg} />
                <IllustrationMessage
                  msg={msg} isPending={isPending} isSending={isSending} isGeneratingVideo={isGeneratingVideo} store={store}
                  caption={illustrationCaptions[msg.message_id]}
                  onCaptionChange={async (id, next) => {
                    await api.updateIllustrationCaption(id, next);
                    setIllustrationCaptions((m) => ({ ...m, [id]: next }));
                  }}
                  playingVideo={playingVideo} setPlayingVideo={setPlayingVideo} loopVideo={loopVideo} setLoopVideo={setLoopVideo}
                  videoFiles={videoFiles} setVideoFiles={setVideoFiles} videoDataUrls={videoDataUrls} playVideoFn={playVideo}
                  setIllustrationModalId={setIllustrationModalId} setModalSelectedId={setModalSelectedId}
                  setModalPlayingVideo={setModalPlayingVideo} setModalImageLoading={setModalImageLoading}
                  setModalIllustrations={setModalIllustrations} setAdjustIllustrationId={setAdjustIllustrationId}
                  setAdjustInstructions={setAdjustInstructions} setVideoModalId={setVideoModalId}
                  setVideoPrompt={setVideoPrompt} setVideoDuration={setVideoDuration} setVideoStyle={setVideoStyle}
                  setVideoTab={setVideoTab} setRemoveVideoConfirmId={setRemoveVideoConfirmId}
                  setResetConfirmId={setResetConfirmId} downloadedId={downloadedId} setDownloadedId={setDownloadedId}
                  loadIllustrations={async () => {
                    if (!store.activeGroupChat) return;
                    try {
                      const page = await api.getGroupMessages(store.activeGroupChat.group_chat_id);
                      const illus = page.messages.filter((m) => m.role === "illustration").map((m) => ({ id: m.message_id, content: m.content }));
                      setModalIllustrations(illus);
                      setCarouselAllMessages(page.messages);
                      for (const il of illus) {
                        if (!videoFiles[il.id]) api.getVideoFile(il.id).then((vf) => { if (vf) setVideoFiles((prev) => ({ ...prev, [il.id]: vf })); }).catch(() => {});
                      }
                    } catch {}
                  }}
                />
              </React.Fragment>);
            }

            // Find the sending character's info for group messages
            const senderChar = msg.sender_character_id
              ? groupCharacters.find((c) => c.character_id === msg.sender_character_id)
              : undefined;
            const senderPortrait = senderChar ? store.activePortraits[senderChar.character_id] : undefined;
            const groupColorPalette = ["bg-blue-500/15", "bg-emerald-500/15", "bg-purple-500/15"];
            const senderColorIdx = senderChar ? groupCharIds.indexOf(senderChar.character_id) : 0;
            const senderBubbleColor = !isUser ? groupColorPalette[senderColorIdx % groupColorPalette.length] : "";

            return (
              <React.Fragment key={msg.message_id}>
              {meanwhileBefore}
              <TimeDivider current={msg} previous={prevMsg} />
              <div data-message-id={msg.message_id}>
                <div className={`flex items-end gap-2 ${isUser ? "justify-end" : "justify-start"}`}>
                  {!isUser && (
                    senderPortrait?.data_url ? (
                      <button onClick={() => senderChar && setPortraitModalCharId(senderChar.character_id)} className="cursor-pointer flex-shrink-0 mb-1">
                        <img src={senderPortrait.data_url} alt="" className="w-[90px] h-[90px] rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
                      </button>
                    ) : (
                      <span
                        className="w-[90px] h-[90px] rounded-full flex-shrink-0 mb-1 ring-1 ring-white/10"
                        style={{ backgroundColor: senderChar?.avatar_color ?? "#c4a882" }}
                      />
                    )
                  )}
                  <div
                    className={`relative group rounded-2xl px-4 py-2.5 leading-relaxed ${keptIds.has(msg.message_id) ? "ring-2 ring-amber-300 shadow-[0_0_0_4px_rgba(251,191,36,0.45),0_0_32px_10px_rgba(251,191,36,0.75),0_0_80px_20px_rgba(245,158,11,0.55),0_0_160px_40px_rgba(251,191,36,0.30)] [&>*]:relative [&>*]:z-10 before:content-[''] before:absolute before:inset-0 before:rounded-[inherit] before:pointer-events-none before:bg-gradient-to-br before:from-amber-200/50 before:via-amber-300/40 before:to-yellow-300/50 before:mix-blend-overlay before:blur-xl before:bg-[length:200%_200%] before:animate-[canonized-shimmer_9s_ease-in-out_infinite]" : ""} ${
                      isUser
                        ? "bg-primary text-primary-foreground rounded-br-md max-w-[80%]"
                        : senderBubbleColor
                          ? `${senderBubbleColor} text-secondary-foreground rounded-bl-md max-w-[80%] border border-border/30`
                          : "bg-secondary text-secondary-foreground rounded-bl-md max-w-[80%]"
                    }`}
                  >
                    {/* Speak button — top-right corner of character bubbles */}
                    {!isUser && !isPending && senderChar && (() => {
                      const msgCached = cachedTones[msg.message_id];
                      const hasCached = msgCached && msgCached.size > 0;
                      const isSpeaking = speakingId === msg.message_id;
                      const isLoading = loadingSpeech === msg.message_id;
                      const lastTone = lastTones[msg.message_id];
                      const allTones = ["Auto", "Playful", "Happy", "Excited", "Reverent", "Serene", "Intimate", "Tender", "Sad", "Melancholy", "Angry", "Anxious"];
                      return (
                      <div className="absolute -top-2.5 -right-2.5 z-10">
                        <button
                          onClick={() => {
                            if (isSpeaking) {
                              audioRef.current?.pause();
                              setSpeakingId(null);
                            } else {
                              setToneMenuId(toneMenuId === msg.message_id ? null : msg.message_id);
                            }
                          }}
                          className={`w-7 h-7 flex items-center justify-center rounded-full shadow-md border hover:scale-110 transition-all cursor-pointer ${
                            isSpeaking
                              ? "bg-primary text-white border-primary/50 opacity-100"
                              : isLoading
                                ? "bg-white text-primary border-border/50 opacity-100"
                                : hasCached
                                  ? "bg-amber-500/15 text-amber-500 border-amber-500/30 opacity-100"
                                  : "bg-white text-muted-foreground hover:text-foreground border-border/50 opacity-0 group-hover:opacity-100"
                          }`}
                        >
                          {isLoading ? <Loader2 size={14} className="animate-spin" /> : isSpeaking ? <Square size={10} fill="white" /> : <Volume2 size={14} />}
                        </button>
                        {toneMenuId === msg.message_id && (
                          <div ref={toneMenuRef} className="absolute top-full right-0 mt-1.5 bg-card border border-border rounded-lg shadow-xl shadow-black/20 p-2.5 z-50 w-[280px]">
                            {hasCached && lastTone && (
                              <button
                                onClick={() => {
                                  setToneMenuId(null);
                                  handleSpeak(msg.message_id, msg.content, senderChar.character_id, lastTone === "auto" ? "Auto" : lastTone.charAt(0).toUpperCase() + lastTone.slice(1));
                                }}
                                className="w-full text-left px-2.5 py-1.5 mb-2 text-xs hover:bg-accent transition-colors cursor-pointer flex items-center gap-2 font-medium rounded-md border border-border/50"
                              >
                                <Play size={10} fill="currentColor" className="text-primary flex-shrink-0" />
                                Last: {lastTone === "auto" ? "Auto" : lastTone.charAt(0).toUpperCase() + lastTone.slice(1)}
                              </button>
                            )}
                            <div className="grid grid-cols-3 gap-1">
                              {allTones.map((tone) => {
                                const isCached = msgCached?.has(tone.toLowerCase());
                                return (
                                  <button
                                    key={tone}
                                    onClick={() => {
                                      setToneMenuId(null);
                                      handleSpeak(msg.message_id, msg.content, senderChar.character_id, tone);
                                    }}
                                    className="px-2 py-1.5 text-[11px] rounded-md hover:bg-accent transition-colors cursor-pointer flex items-center justify-center gap-1"
                                  >
                                    {tone === "Auto" ? <span className="text-muted-foreground">Auto</span> : tone}
                                    {isCached && <Volume2 size={8} className="text-primary flex-shrink-0" />}
                                  </button>
                                );
                              })}
                            </div>
                            {hasCached && (
                              <button
                                onClick={async () => {
                                  setToneMenuId(null);
                                  if (isSpeaking) { audioRef.current?.pause(); setSpeakingId(null); }
                                  await api.deleteMessageAudio(msg.message_id);
                                  setCachedTones((prev) => { const next = { ...prev }; delete next[msg.message_id]; return next; });
                                  setLastTones((prev) => { const next = { ...prev }; delete next[msg.message_id]; return next; });
                                }}
                                className="w-full mt-2 pt-2 border-t border-border text-left px-2 py-1 text-[11px] hover:bg-accent transition-colors cursor-pointer flex items-center gap-1.5 text-red-400 rounded-md"
                              >
                                <Trash2 size={10} className="flex-shrink-0" />
                                Delete Audio
                              </button>
                            )}
                          </div>
                        )}
                      </div>
                      );
                    })()}

                    {!isUser && senderChar && (
                      <p className="text-[10px] font-semibold text-muted-foreground/70 mb-1">{senderChar.display_name}</p>
                    )}

                    {isEmojiOnlyMessage(msg.content) ? (
                      <div
                        style={{ fontSize: `${Math.round(chatFontPx(store.chatFontSize) * 3.5)}px`, lineHeight: 1.1 }}
                        className="leading-tight select-text"
                      >
                        {msg.content.trim()}
                      </div>
                    ) : (
                      <div style={{ fontSize: `${chatFontPx(store.chatFontSize)}px` }} className={`prose prose-sm max-w-none prose-p:my-1 prose-ul:my-1 prose-ol:my-1 prose-li:my-0.5 prose-headings:my-2 prose-pre:my-2 prose-blockquote:my-2 prose-hr:my-2 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_em]:italic ${
                        isUser
                          ? "[--tw-prose-body:var(--color-primary-foreground)] [--tw-prose-headings:var(--color-primary-foreground)] [--tw-prose-bold:var(--color-primary-foreground)] [--tw-prose-bullets:var(--color-primary-foreground)] [--tw-prose-counters:var(--color-primary-foreground)] [--tw-prose-code:var(--color-primary-foreground)] [--tw-prose-links:var(--color-primary-foreground)] [--tw-prose-quotes:var(--color-primary-foreground)] [--tw-prose-quote-borders:rgba(255,255,255,0.3)]"
                          : "[--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-headings:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)] [--tw-prose-bullets:var(--color-secondary-foreground)] [--tw-prose-counters:var(--color-secondary-foreground)] [--tw-prose-code:var(--color-secondary-foreground)] [--tw-prose-links:var(--color-primary)] [--tw-prose-quotes:var(--color-secondary-foreground)] [--tw-prose-quote-borders:var(--color-border)]"
                      }`}>
                        {isUser ? (
                          <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(msg.content)}</Markdown>
                        ) : (
                          parseBackstageSegments(msg.content).map((seg, segIdx) => {
                            if (seg.kind === "text") {
                              const trimmed = seg.value.trim();
                              if (!trimmed) return null;
                              return (
                                <Markdown key={segIdx} components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(seg.value)}</Markdown>
                              );
                            }
                            if (seg.block.type === "propose_quest" && store.activeWorld?.world_id) {
                              const title = (seg.block as any).title ?? "";
                              const description = (seg.block as any).description ?? "";
                              if (!title || !description) return null;
                              return (
                                <InlineQuestProposalCard
                                  key={segIdx}
                                  title={title}
                                  description={description}
                                  worldId={store.activeWorld.world_id}
                                  sourceMessageId={msg.message_id}
                                />
                              );
                            }
                            return null;
                          })
                        )}
                      </div>
                    )}
                    <InventoryUpdateBadge records={inventoryBadges[msg.message_id]} />
                    <p className={`text-[10px] mt-1 flex items-center gap-2 ${
                      isUser ? "text-primary-foreground/50" : "text-muted-foreground"
                    }`}>
                      {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                      {!isPending && (
                        <button
                          onClick={() => setResetConfirmId(msg.message_id)}
                          className={`opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer ${
                            isUser ? "text-primary-foreground/30 hover:text-primary-foreground/70" : "text-muted-foreground/50 hover:text-muted-foreground"
                          }`}
                        >
                          Reset to Here
                        </button>
                      )}
                    </p>
                    {isUser && (
                      <div className="absolute top-2 right-8 opacity-0 group-hover:opacity-100 transition-opacity">
                        <div className="relative group/uedit">
                          <button
                            onClick={() => { setAdjustMessageId(msg.message_id); setAdjustEditOnly(true); }}
                            className="w-7 h-7 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm"
                          >
                            <Pencil size={12} />
                          </button>
                          <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/uedit:opacity-100 pointer-events-none transition-opacity">Edit</span>
                        </div>
                      </div>
                    )}
                    {!isUser && !isPending && (
                      <div className="absolute top-2 right-8 opacity-0 group-hover:opacity-100 transition-opacity">
                        <div className="relative group/madj">
                          <button
                            onClick={() => { setAdjustMessageId(msg.message_id); setAdjustEditOnly(false); }}
                            className="w-7 h-7 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm"
                          >
                            <SlidersHorizontal size={12} />
                          </button>
                          <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/madj:opacity-100 pointer-events-none transition-opacity">Adjust</span>
                        </div>
                      </div>
                    )}
                    {store.adjustingMessageId === msg.message_id && (
                      <div className="absolute inset-0 rounded-2xl bg-secondary/80 backdrop-blur-sm flex items-center justify-center gap-2">
                        <Loader2 size={14} className="animate-spin text-muted-foreground" />
                        <span className="text-xs text-muted-foreground">Adjusting...</span>
                      </div>
                    )}
                  </div>
                  {isUser && userAvatarUrl && (
                    <button onClick={() => setShowUserAvatarModal(true)} className="cursor-pointer flex-shrink-0 mb-1">
                      <img src={userAvatarUrl} alt="" className="w-[90px] h-[90px] rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
                    </button>
                  )}
                </div>

                <div className={`relative ${!isUser ? "pl-24" : userAvatarUrl ? "pr-24" : ""}`}>
                  <div className={`flex items-center gap-1.5 mt-1 ${isUser ? "justify-end" : "justify-start"}`}>
                    {!isPending && (
                      <>
                        <button
                          onClick={() => setPickerMessageId(showPicker ? null : msg.message_id)}
                          className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-secondary/50 hover:bg-secondary border border-border/60 hover:border-border text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                          title="Add reaction"
                        >
                          <SmilePlus size={12} />
                        </button>
                        <div className="relative group/mcanon">
                          <button
                            onClick={() => setKeepTargetId(msg.message_id)}
                            className={`inline-flex items-center justify-center w-6 h-6 rounded-full border transition-colors cursor-pointer ${
                              keptIds.has(msg.message_id)
                                ? "bg-amber-500/15 border-amber-500/40 text-amber-500 hover:bg-amber-500/25"
                                : "bg-secondary/50 hover:bg-secondary border-border/60 hover:border-border text-muted-foreground hover:text-foreground"
                            }`}
                          >
                            <ScrollText size={12} />
                          </button>
                          <span className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mcanon:opacity-100 pointer-events-none transition-opacity z-50">
                            {keptIds.has(msg.message_id) ? "Kept · save again" : "Keep to record"}
                          </span>
                        </div>
                        <div className="relative group/minv">
                          <button
                            onClick={() => handleInventoryUpdateFromMessage(msg.message_id)}
                            disabled={invUpdatingId === msg.message_id}
                            className={`inline-flex items-center justify-center w-6 h-6 rounded-full border transition-colors cursor-pointer disabled:opacity-60 disabled:cursor-wait ${
                              (inventoryBadges[msg.message_id]?.length ?? 0) > 0
                                ? "bg-emerald-500/15 border-emerald-500/40 text-emerald-500 hover:bg-emerald-500/25"
                                : "bg-secondary/50 hover:bg-secondary border-border/60 hover:border-border text-muted-foreground hover:text-foreground"
                            }`}
                          >
                            {invUpdatingId === msg.message_id ? <Loader2 size={12} className="animate-spin" /> : <Package size={12} />}
                          </button>
                          <span className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/minv:opacity-100 pointer-events-none transition-opacity z-50">
                            {(inventoryBadges[msg.message_id]?.length ?? 0) > 0 ? "Inventory updated · run again" : "Update group inventories from this moment"}
                          </span>
                        </div>
                        {/* "How do I react!?" — only on character messages.
                            Opens the consultant modal in whichever mode is
                            persisted in localStorage and auto-sends the
                            question as the user's first message.

                            Sent text is intentionally longer than the
                            tooltip: "How do I react!?" alone gets
                            interpreted by the consultant as "how do I add
                            a reaction emoji to this message" — same
                            surface form as the actual reaction-emoji
                            UX. The long form anchors it to dialogue
                            response. Don't shorten. */}
                        {!isUser && (
                          <div className="relative group/mhowreact">
                            <button
                              onClick={() => { setConsultantAutoSend("How do I react to this message!? What do I say to that!?"); setShowConsultant(true); }}
                              className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-secondary/50 hover:bg-secondary border border-border/60 hover:border-border text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                            >
                              <MessageCircleQuestion size={12} />
                            </button>
                            <span className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mhowreact:opacity-100 pointer-events-none transition-opacity z-50">
                              How do I react!?
                            </span>
                          </div>
                        )}
                      </>
                    )}
                    <ReactionBubbles
                      reactions={reactions}
                      isUser={isUser}
                      characterNameById={Object.fromEntries(groupCharacters.map((c) => [c.character_id, c.display_name]))}
                      userDisplayName={store.userProfile?.display_name || "You"}
                    />
                    {/* Suppressed when reactions are toggled off — otherwise the throbbing
                        pill misleads the user into thinking a reaction is incoming. */}
                    {isUser && isSending && msgIdx === filteredMsgs.length - 1 && reactionsMode !== "off" && (
                      <span
                        className="inline-flex items-center gap-1.5 text-sm rounded-full px-3 py-1.5 animate-pulse text-white shadow-md"
                        style={{ background: "linear-gradient(90deg, #f472b6 0%, #a78bfa 50%, #60a5fa 100%)" }}
                        title="Character is reacting..."
                      >
                        <SmilePlus size={16} />
                        <span className="w-1.5 h-1.5 rounded-full bg-white/90" />
                      </span>
                    )}
                  </div>
                  {showPicker && (
                    <ReactionPicker
                      onPick={(emoji) => store.toggleReaction(msg.message_id, emoji)}
                      onClose={() => setPickerMessageId(null)}
                      anchorRight={isUser}
                    />
                  )}
                </div>
              </div>
              </React.Fragment>
            );
          })}
          {!store.loadingChat && meanwhileBuckets.trailing.map((ev) => (
            <MeanwhileCard
              key={`mw-${ev.event_id}`}
              event={ev}
              portraitUrl={store.activePortraits[ev.character_id]?.data_url}
            />
          ))}
          {isSending && !isGeneratingNarrative && !isGeneratingIllustration && !isGeneratingVideo && (() => {
            const sendingChar = store.sendingCharacterId
              ? groupCharacters.find((c) => c.character_id === store.sendingCharacterId)
              : groupCharacters[0];
            const sendingPortrait = sendingChar ? store.activePortraits[sendingChar.character_id] : undefined;
            return (
            <div className="flex items-end gap-2 justify-start">
              {sendingPortrait?.data_url ? (
                <img src={sendingPortrait.data_url} alt="" className="w-[90px] h-[90px] rounded-full object-cover ring-2 ring-border flex-shrink-0 mb-1" />
              ) : (
                <span
                  className="w-[90px] h-[90px] rounded-full flex-shrink-0 mb-1 ring-1 ring-white/10"
                  style={{ backgroundColor: sendingChar?.avatar_color ?? "#c4a882" }}
                />
              )}
              <div className="bg-secondary rounded-2xl rounded-bl-md px-4 py-3 flex items-center gap-1">
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:0ms]" />
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:150ms]" />
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:300ms]" />
              </div>
            </div>
            );
          })()}
          {isGeneratingNarrative && (
            <div className="flex justify-center my-2">
              <div className="rounded-xl px-5 py-3 bg-gradient-to-br from-amber-950/40 to-amber-900/20 border border-amber-700/30 flex items-center gap-2 text-amber-500/70">
                <BookOpen size={14} className="animate-pulse" />
                <span className="text-xs italic">Weaving narrative...</span>
                <span className="w-1.5 h-1.5 rounded-full bg-amber-500/60 animate-bounce [animation-delay:0ms]" />
                <span className="w-1.5 h-1.5 rounded-full bg-amber-500/60 animate-bounce [animation-delay:150ms]" />
                <span className="w-1.5 h-1.5 rounded-full bg-amber-500/60 animate-bounce [animation-delay:300ms]" />
              </div>
            </div>
          )}
          {isGeneratingIllustration && (
            <div className="flex justify-center my-2">
              <div className="rounded-xl px-5 py-3 bg-gradient-to-br from-emerald-950/40 to-emerald-900/20 border border-emerald-700/30 flex items-center gap-2 text-emerald-500/70">
                <Image size={14} className="animate-pulse" />
                <span className="text-xs italic">Painting scene...</span>
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-500/60 animate-bounce [animation-delay:0ms]" />
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-500/60 animate-bounce [animation-delay:150ms]" />
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-500/60 animate-bounce [animation-delay:300ms]" />
              </div>
            </div>
          )}
        </div>
        </div>
      </ScrollArea>
      {!isAtBottom && (
        <button
          onClick={() => { const el = scrollRef.current; if (el) el.scrollTo({ top: el.scrollHeight, behavior: "smooth" }); }}
          className="absolute bottom-4 left-1/2 -translate-x-1/2 z-20 w-8 h-8 rounded-full bg-card/80 backdrop-blur-sm shadow-lg shadow-black/20 border border-border/30 flex items-center justify-center cursor-pointer hover:bg-card transition-colors text-muted-foreground hover:text-foreground"
        >
          <ChevronDown size={16} />
        </button>
      )}
      <StickyIllustration
        messages={store.messages}
        scrollContainer={scrollRef.current}
        aspectRatios={store.aspectRatios}
      />
      <AnimationReadyToast
        animationReadyId={animationReadyId}
        onGo={() => {
          const el = document.querySelector(`[data-message-id="${animationReadyId}"]`);
          if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
          setAnimationReadyId(null);
        }}
        onDismiss={() => setAnimationReadyId(null)}
      />
      <InventoryUpdatedToast
        updates={inventoryToast}
        onDismiss={() => setInventoryToast(null)}
      />
      </div>

      <ChatErrorBar
        error={store.chatError}
        lastFailedContent={store.lastFailedContent}
        isSending={isSending}
        onRetry={handleRetry}
        onCopy={() => {
          navigator.clipboard.writeText(store.chatError!);
          setCopiedError(true);
          setTimeout(() => setCopiedError(false), 2000);
        }}
        onDismiss={() => store.clearChatError()}
        copiedError={copiedError}
      />

      <div className="px-4 py-3 border-t border-border relative z-10 bg-background">
        <div className="flex gap-2 mx-auto items-stretch" style={{ maxWidth: "747px" }}>
          <div className="flex-shrink-0 flex flex-col items-center gap-1 justify-center">
            <label className="flex items-center gap-1.5 cursor-pointer select-none" title="When on, characters respond automatically after each message">
              <span className={`text-[10px] font-medium ${store.autoRespond ? "text-foreground/70" : "text-muted-foreground/50"}`}>Auto‑Respond</span>
              <button
                role="switch"
                aria-checked={store.autoRespond}
                onClick={() => store.setAutoRespond(!store.autoRespond)}
                className={`relative w-8 h-[18px] rounded-full transition-colors cursor-pointer ${store.autoRespond ? "bg-primary" : "bg-muted-foreground/30"}`}
              >
                <span className={`absolute top-0.5 left-0.5 w-3.5 h-3.5 rounded-full bg-white shadow-sm transition-transform ${store.autoRespond ? "translate-x-[14px]" : ""}`} />
              </button>
            </label>
            <div className="flex gap-0.5">
              <div className="relative group/ilus">
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-emerald-500/70 hover:text-emerald-400 hover:bg-emerald-500/10 h-9 w-9 rounded-lg"
                  onClick={() => setShowIllustrationPicker(true)}
                  disabled={isSending || !store.apiKey || store.messages.length === 0}
                >
                  <Image size={15} />
                </Button>
                <span className="absolute bottom-full left-1/2 -translate-x-1/2 -mb-0.5 px-2.5 py-1 text-[11px] font-medium text-white bg-black rounded-lg shadow-lg whitespace-nowrap opacity-0 group-hover/ilus:opacity-100 pointer-events-none transition-opacity duration-150">
                  Illustration
                </span>
              </div>
              <div className="relative group/narr">
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-amber-500/70 hover:text-amber-400 hover:bg-amber-500/10 h-9 w-9 rounded-lg"
                  onClick={() => setShowNarrativePicker(true)}
                  disabled={isSending || !store.apiKey || store.messages.length === 0}
                >
                  <BookOpen size={15} />
                </Button>
                <span className="absolute bottom-full left-1/2 -translate-x-1/2 -mb-0.5 px-2.5 py-1 text-[11px] font-medium text-white bg-black rounded-lg shadow-lg whitespace-nowrap opacity-0 group-hover/narr:opacity-100 pointer-events-none transition-opacity duration-150">
                  + Narrative
                </span>
              </div>
              <div ref={talkPickerRef} className="relative group/talk">
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-primary/70 hover:text-primary hover:bg-primary/10 h-9 w-9 rounded-lg"
                  onClick={() => setShowGroupTalkPicker(!showGroupTalkPicker)}
                  disabled={isSending || !store.apiKey || store.messages.length === 0}
                >
                  <MessageSquare size={15} />
                </Button>
                {!showGroupTalkPicker && (
                  <span className="absolute bottom-full left-1/2 -translate-x-1/2 -mb-0.5 px-2.5 py-1 text-[11px] font-medium text-white bg-black rounded-lg shadow-lg whitespace-nowrap opacity-0 group-hover/talk:opacity-100 pointer-events-none transition-opacity duration-150">
                    Talk
                  </span>
                )}
                {showGroupTalkPicker && (() => {
                  const userAvatar = userAvatarUrl
                    ? <img src={userAvatarUrl} alt="" className="w-9 h-9 rounded-full object-cover flex-shrink-0" />
                    : <div className="w-9 h-9 rounded-full flex-shrink-0 bg-primary/30" />;
                  const charAvatar = (ch: typeof groupCharacters[0]) => {
                    const p = store.activePortraits[ch.character_id];
                    return p?.data_url
                      ? <img src={p.data_url} alt="" className="w-9 h-9 rounded-full object-cover flex-shrink-0" />
                      : <div className="w-9 h-9 rounded-full flex-shrink-0" style={{ backgroundColor: ch.avatar_color }} />;
                  };
                  return (
                  <div className="absolute bottom-full left-0 mb-2 z-50 bg-card border border-border rounded-xl shadow-xl p-2 space-y-0.5 animate-in fade-in zoom-in-95 duration-150 w-max">
                    {groupCharacters.length > 1 && (
                      <>
                        {groupCharacters.flatMap((speaker) =>
                          groupCharacters
                            .filter((target) => target.character_id !== speaker.character_id)
                            .map((target) => (
                              <button
                                key={`${speaker.character_id}-${target.character_id}`}
                                onClick={() => {
                                  store.promptGroupCharacter(speaker.character_id, target.display_name);
                                  setShowGroupTalkPicker(false);
                                }}
                                className="flex items-center gap-2 w-full px-2 py-1.5 rounded-lg hover:bg-accent transition-colors cursor-pointer"
                              >
                                {charAvatar(speaker)}
                                <ArrowRight size={10} className="text-muted-foreground flex-shrink-0" />
                                {charAvatar(target)}
                                <span className="text-xs whitespace-nowrap"><span className="font-medium">{speaker.display_name}</span> <span className="text-muted-foreground">to {target.display_name}</span></span>
                              </button>
                            ))
                        )}
                        <div className="border-t border-border my-1" />
                      </>
                    )}
                    {groupCharacters.map((ch) => (
                      <button
                        key={`${ch.character_id}-user`}
                        onClick={() => {
                          store.promptGroupCharacter(ch.character_id);
                          setShowGroupTalkPicker(false);
                        }}
                        className="flex items-center gap-2 w-full px-2 py-1.5 rounded-lg hover:bg-accent transition-colors cursor-pointer"
                      >
                        {charAvatar(ch)}
                        <ArrowRight size={10} className="text-muted-foreground flex-shrink-0" />
                        {userAvatar}
                        <span className="text-xs whitespace-nowrap"><span className="font-medium">{ch.display_name}</span> <span className="text-muted-foreground">to Me</span></span>
                      </button>
                    ))}
                  </div>
                  );
                })()}
              </div>
            </div>
          </div>
          <div className="flex-1 self-stretch relative">
            <textarea
              ref={inputRef}
              defaultValue=""
              style={{ fontSize: `${chatFontPx(store.chatFontSize)}px` }}
              onChange={(e) => {
                inputValueRef.current = e.target.value;
                const empty = !e.target.value.trim();
                if (hasInput === empty) setHasInput(!empty);
                e.target.style.height = "auto";
                e.target.style.height = `${e.target.scrollHeight}px`;
              }}
              onKeyDown={handleKeyDown}
              placeholder={`Talk to ${store.activeGroupChat?.display_name ?? "the group"}...`}
              className="w-full resize-none overflow-hidden rounded-xl border border-input bg-transparent pl-4 pr-12 py-2.5 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
              rows={1}
              disabled={isSending || (store.autoRespond && !store.apiKey)}
            />
            <button
              type="button"
              onClick={() => setShowEmojiPicker((v) => !v)}
              disabled={isSending || (store.autoRespond && !store.apiKey)}
              title="Insert emoji"
              className="absolute top-1 right-2 w-10 h-10 rounded-lg flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <Smile size={24} />
            </button>
            {showEmojiPicker && (
              <ReactionPicker
                anchorRight
                onPick={(emoji) => insertEmojiAtCursor(emoji)}
                onClose={() => setShowEmojiPicker(false)}
              />
            )}
          </div>
          <div className="flex-shrink-0 flex flex-col gap-1">
            <div className="flex gap-2 flex-1">
              <Button
                size="icon"
                className="rounded-xl self-stretch w-10 flex-shrink-0"
                onClick={handleSend}
                disabled={!hasInput || isSending || (store.autoRespond && !store.apiKey)}
              >
                {isSending ? <Loader2 size={16} className="animate-spin" /> : <Send size={16} />}
              </Button>
              <div className="relative flex-shrink-0" ref={settingsPopoverRef}>
            <Button
              size="icon"
              variant={showSettingsPopover ? "default" : "secondary"}
              className={`rounded-xl self-stretch w-10 flex-shrink-0 ${showSettingsPopover ? "" : "hover:bg-secondary/70"}`}
              onClick={() => setShowSettingsPopover(!showSettingsPopover)}
            >
              <Settings size={16} />
            </Button>
            {showSettingsPopover && (
              <div className="absolute bottom-full right-0 mb-2 w-80 bg-card border border-border rounded-xl shadow-2xl shadow-black/40 p-4 space-y-3 z-50 animate-in fade-in zoom-in-95 duration-150">
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1.5">Model</label>
                  <div className="flex rounded-lg overflow-hidden border border-input">
                    {[
                      { id: "", label: "Default" },
                      { id: "lmstudio", label: "Local" },
                      { id: "openai", label: "Frontier" },
                    ].map((opt) => (
                      <button
                        key={opt.id || "default"}
                        onClick={() => setProviderOverridePersist(opt.id)}
                        className={`flex-1 px-2 py-1.5 text-xs font-medium transition-colors cursor-pointer ${
                          providerOverride === opt.id ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                        }`}
                      >{opt.label}</button>
                    ))}
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1.5">Who is leading</label>
                  <div className="flex rounded-lg overflow-hidden border border-input">
                    {[{ id: "user", label: "Me" }, ...groupCharacters.map((c) => ({ id: c.character_id, label: c.display_name }))].map((opt) => (
                      <button
                        key={opt.id}
                        onClick={() => setLeaderPersist(opt.id)}
                        className={`flex-1 px-2 py-1.5 text-xs font-medium transition-colors cursor-pointer ${
                          leader === opt.id ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                        }`}
                      >{opt.label}</button>
                    ))}
                  </div>
                </div>
                <div>
                  <label className="flex items-center gap-2 cursor-pointer select-none">
                    <input
                      type="checkbox"
                      checked={sendHistory}
                      onChange={(e) => setSendHistoryPersist(e.target.checked)}
                      className="h-4 w-4 rounded border-input accent-primary cursor-pointer"
                    />
                    <span className="text-xs font-medium text-muted-foreground">Send conversation history</span>
                  </label>
                </div>
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1.5">Thread mood</label>
                  {moodReduction.length > 0 ? (
                    <div className="flex items-center gap-1 text-base" title="Most recent reaction emojis on this thread — seeds the next reply's emotional weather.">
                      {moodReduction.slice(0, 8).map((e, i) => (
                        <span key={i} style={{ opacity: Math.max(0.35, 1 - i * 0.1) }}>{e}</span>
                      ))}
                    </div>
                  ) : (
                    <div className="text-xs text-muted-foreground/70 italic">No reactions yet — the characters will start the loop.</div>
                  )}
                </div>
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1.5">Tone</label>
                  <select
                    value={narrationTone}
                    onChange={(e) => { setNarrationTone(e.target.value); setNarrationDirty(true); }}
                    className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
                  >
                    {["Auto", "Humorous", "Romantic", "Action & Adventure", "Dark & Gritty", "Suspenseful", "Whimsical", "Melancholic", "Heroic", "Horror", "Noir", "Surreal", "Cozy & Warm", "Tense & Paranoid", "Poetic", "Cinematic", "Mythic", "Playful", "Bittersweet", "Ethereal", "Gritty Realism"].map((t) => (
                      <option key={t} value={t}>{t}</option>
                    ))}
                  </select>
                </div>
                <div className="flex items-center justify-between">
                  <label className="text-xs font-medium text-muted-foreground">Font Size</label>
                  <FontSizeAdjuster value={store.chatFontSize} onChange={store.setChatFontSize} />
                </div>
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1.5">Response Length</label>
                  <div className="flex rounded-lg overflow-hidden border border-input">
                    {["Auto", "Short", "Medium", "Long"].map((l) => (
                      <button
                        key={l}
                        onClick={() => { setResponseLength(l); setNarrationDirty(true); }}
                        className={`flex-1 px-2 py-1.5 text-xs font-medium transition-colors cursor-pointer ${
                          responseLength === l ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                        }`}
                      >{l}</button>
                    ))}
                  </div>
                </div>
                <div className="flex items-center justify-between gap-3">
                  <div className="flex flex-col">
                    <label className="text-xs font-medium text-muted-foreground">Reactions</label>
                    <span className="text-[10px] text-muted-foreground/60">How often characters emoji-react</span>
                  </div>
                  <div className="inline-flex border border-input bg-background rounded-lg">
                    {([
                      { mode: "off" as const,        glyph: "🚫",     title: "Off — No Reactions" },
                      { mode: "occasional" as const, glyph: "😀",     title: "Occasionally — Text Message Mode" },
                      { mode: "always" as const,     glyph: "😀😀😀", title: "Always — Every Message Gets a Reaction" },
                    ]).map(({ mode, glyph, title }, idx, arr) => (
                      <div
                        key={mode}
                        className={`relative group/rxn-mode inline-flex ${idx === 0 ? "rounded-l-lg" : ""} ${idx === arr.length - 1 ? "rounded-r-lg" : ""} overflow-visible`}
                      >
                        <button
                          onClick={() => { setReactionsMode(mode); setNarrationDirty(true); }}
                          className={`inline-flex items-center justify-center px-2.5 py-1 text-sm leading-none transition-colors cursor-pointer ${idx === 0 ? "rounded-l-lg" : ""} ${idx === arr.length - 1 ? "rounded-r-lg" : ""} ${
                            reactionsMode === mode
                              ? "bg-primary text-primary-foreground"
                              : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                          }`}
                          aria-label={title}
                          aria-pressed={reactionsMode === mode}
                        >
                          {glyph}
                        </button>
                        <span className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1.5 px-2 py-1 text-[11px] font-medium text-white bg-black/90 rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/rxn-mode:opacity-100 pointer-events-none transition-opacity z-50">
                          {title}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
                <div>
                  <label className="flex items-center gap-2 cursor-pointer select-none">
                    <input
                      type="checkbox"
                      checked={arcadeGameMode}
                      onChange={(e) => {
                        const v = e.target.checked;
                        setArcadeGameMode(v);
                        if (chatId) api.setSetting(`arcade_game_mode.${chatId}`, v ? "on" : "off").catch(() => {});
                      }}
                      className="h-4 w-4 rounded border-input accent-primary cursor-pointer"
                    />
                    <span className="text-xs font-medium text-muted-foreground">
                      🎮 Arcade game mode <span className="text-muted-foreground/60 font-normal">(HUD, loot, riddles — cosmetic)</span>
                    </span>
                  </label>
                </div>
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1.5">Custom Instructions</label>
                  <textarea
                    value={narrationInstructions}
                    onChange={(e) => {
                      setNarrationInstructions(e.target.value);
                      setNarrationDirty(true);
                      e.target.style.height = "auto";
                      e.target.style.height = `${e.target.scrollHeight}px`;
                    }}
                    ref={(el) => {
                      if (el) { el.style.height = "auto"; el.style.height = `${el.scrollHeight}px`; }
                    }}
                    placeholder="e.g. Describe the weather shifting..."
                    className="w-full resize-none overflow-hidden rounded-lg border border-input bg-transparent px-3 py-2 text-xs placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                    rows={4}
                  />
                </div>
                {onNavigateToCharacter && (
                  <div className="pt-2 border-t border-border space-y-0.5">
                    {groupCharacters.map((ch) => (
                      <button
                        key={ch.character_id}
                        onClick={() => { setShowSettingsPopover(false); onNavigateToCharacter(ch.character_id); }}
                        className="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-accent transition-colors cursor-pointer text-xs text-muted-foreground hover:text-foreground"
                      >
                        {store.activePortraits[ch.character_id]?.data_url ? (
                          <img src={store.activePortraits[ch.character_id].data_url} alt="" className="w-5 h-5 rounded-full object-cover ring-1 ring-border" />
                        ) : (
                          <span className="w-5 h-5 rounded-full ring-1 ring-white/10" style={{ backgroundColor: ch.avatar_color }} />
                        )}
                        <span className="font-medium">{ch.display_name} Settings</span>
                        <ChevronRight size={12} className="ml-auto text-muted-foreground/50" />
                      </button>
                    ))}
                  </div>
                )}
                <div className="border-t border-border pt-2">
                  <button
                    onClick={() => setShowAdvanced(!showAdvanced)}
                    className="flex items-center gap-1 text-[10px] text-muted-foreground/50 hover:text-muted-foreground transition-colors cursor-pointer"
                  >
                    <ChevronRight size={10} className={`transition-transform ${showAdvanced ? "rotate-90" : ""}`} />
                    Advanced
                  </button>
                  {showAdvanced && (
                    <button
                      onClick={() => setShowClearConfirm(true)}
                      className="w-full flex items-center gap-1.5 px-2 py-1.5 mt-1 text-xs text-destructive/60 hover:text-destructive hover:bg-destructive/10 rounded-lg transition-colors cursor-pointer"
                    >
                      <Trash2 size={10} />
                      Clear Chat History
                    </button>
                  )}
                </div>
              </div>
            )}
          </div>
            </div>
            <div className="flex items-center justify-end gap-1.5">
              <div className="relative group/sndtrk inline-flex">
                <button
                  type="button"
                  onClick={toggleSoundtrack}
                  aria-label={soundtrackEnabled ? "Mute chiptune soundtrack" : "Play chiptune soundtrack"}
                  aria-pressed={soundtrackEnabled}
                  className={`flex items-center justify-center w-5 h-5 rounded transition-colors cursor-pointer ${
                    soundtrackEnabled
                      ? "text-violet-400 hover:text-violet-300 hover:bg-violet-500/10"
                      : "text-muted-foreground/50 hover:text-foreground hover:bg-accent/50"
                  }`}
                >
                  {soundtrack.status === "generating" ? (
                    <Loader2 size={11} className="animate-spin" />
                  ) : soundtrackEnabled ? (
                    <Music size={11} />
                  ) : (
                    <VolumeX size={11} />
                  )}
                </button>
                <span className="absolute bottom-full right-0 mb-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/sndtrk:opacity-100 pointer-events-none transition-opacity z-50">
                  {soundtrackEnabled
                    ? (soundtrack.currentPhrase
                        ? `Now: ${soundtrack.currentPhrase.mood_descriptor} (${soundtrack.collectionSize} phrase${soundtrack.collectionSize === 1 ? '' : 's'})`
                        : "Soundtrack on (waiting for next reply)")
                    : "Chiptune soundtrack off"}
                </span>
              </div>
              <span className="text-muted-foreground/50 text-right" style={{ fontSize: "12px" }}>Response Length: {responseLength}</span>
            </div>
          </div>
        </div>
      </div>

      <Dialog open={showClearConfirm} onClose={() => setShowClearConfirm(false)} className="max-w-xs">
        <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
          <div className="flex items-center gap-2">
            <Trash2 size={18} className="text-destructive" />
            <h3 className="font-semibold">Clear Chat History</h3>
          </div>
          <p className="text-sm text-muted-foreground">
            {clearKeepMedia
              ? "This will permanently delete the text messages and narratives in this conversation. Illustrations, videos, and any saved novelizations will remain as a gallery of memories."
              : "This will permanently delete all messages, narratives, illustrations, videos, and novelizations in this conversation. This cannot be undone."}
          </p>
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={clearKeepMedia}
              onChange={(e) => setClearKeepMedia(e.target.checked)}
              className="accent-emerald-500 w-3.5 h-3.5"
            />
            <span className="text-xs text-muted-foreground">Keep Illustrations and Videos</span>
          </label>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setShowClearConfirm(false)}>Cancel</Button>
            <Button variant="destructive" size="sm" onClick={() => {
              setShowClearConfirm(false);
              setShowSettingsPopover(false);
              if (store.activeGroupChat) store.clearGroupChatHistory(store.activeGroupChat.group_chat_id, clearKeepMedia);
            }}>Clear</Button>
          </div>
        </div>
      </Dialog>

      <PortraitModal
        characterId={portraitModalCharId}
        characterName={portraitModalCharId ? groupCharacters.find((c) => c.character_id === portraitModalCharId)?.display_name : undefined}
        onClose={() => setPortraitModalCharId(null)}
      />

      <StoryConsultantModal
        open={showConsultant}
        onClose={() => { setShowConsultant(false); setConsultantAutoSend(null); }}
        apiKey={store.apiKey}
        characterId={null}
        groupChatId={store.activeGroupChat?.group_chat_id ?? null}
        threadId={store.messages[0]?.thread_id ?? ""}
        worldId={store.activeWorld?.world_id ?? ""}
        characterNames={groupCharacters.map((c) => c.display_name)}
        worldImageUrl={store.activeWorldImage?.data_url}
        portraits={Object.fromEntries(
          Object.entries(store.activePortraits).filter(([, p]) => p?.data_url).map(([id, p]) => [id, p!.data_url!])
        )}
        userAvatarUrl={userAvatarUrl}
        notifyOnMessage={store.notifyOnMessage}
        chatFontSize={store.chatFontSize}
        autoSendOnOpen={consultantAutoSend}
        onAutoSendConsumed={() => setConsultantAutoSend(null)}
      />

      <ImaginedChapterModal
        open={showImaginedChapter}
        onClose={() => { setShowImaginedChapter(false); setOpenImaginedChapterId(null); }}
        apiKey={store.apiKey}
        threadId={store.messages[0]?.thread_id ?? ""}
        characterPortraitUrls={groupCharacters
          .map((c) => store.activePortraits[c.character_id]?.data_url)
          .filter((u): u is string => !!u)}
        worldImageUrl={store.activeWorldImage?.data_url}
        notifyOnMessage={store.notifyOnMessage}
        chatFontSize={store.chatFontSize}
        openChapterId={openImaginedChapterId}
        onCanonize={(breadcrumbMessageId) => setKeepTargetId(breadcrumbMessageId)}
      />

      {userAvatarUrl && (
        <Dialog open={showUserAvatarModal} onClose={() => setShowUserAvatarModal(false)} className="max-w-md">
          <div className="relative">
            <img
              src={userAvatarUrl}
              alt={store.userProfile?.display_name ?? "You"}
              className="w-full rounded-2xl shadow-2xl shadow-black/50"
            />
            <button
              onClick={() => setShowUserAvatarModal(false)}
              className="absolute top-3 right-3 w-8 h-8 flex items-center justify-center rounded-full bg-black/50 text-white hover:bg-black/70 transition-colors cursor-pointer backdrop-blur-sm"
            >
              <X size={16} />
            </button>
            <div className="absolute inset-x-0 bottom-0 rounded-b-2xl bg-gradient-to-t from-black/70 to-transparent px-5 pb-4 pt-10">
              <p className="text-white font-semibold text-lg">{store.userProfile?.display_name ?? "You"}</p>
            </div>
          </div>
        </Dialog>
      )}

      <ResetConfirmModal
        open={!!resetConfirmId}
        onClose={() => setResetConfirmId(null)}
        onConfirm={() => {
          if (resetConfirmId) {
            store.resetToMessage(resetConfirmId);
            setResetConfirmId(null);
          }
        }}
        characterName={store.activeGroupChat?.display_name}
        isUserMessage={store.messages.find((m) => m.message_id === resetConfirmId)?.role === "user" || false}
        isGroup={true}
      />

      <SummaryModal
        open={showSummary}
        onClose={() => setShowSummary(false)}
        title={`Summary: ${groupCharacters.map((c) => c.display_name).join(" & ")}`}
        generateSummary={(mode) => api.generateGroupChatSummary(store.apiKey, store.activeGroupChat?.group_chat_id ?? "", mode)}
        notifyOnMessage={store.notifyOnMessage}
      />

      <LocationModal
        open={showLocationModal}
        onClose={() => setShowLocationModal(false)}
        worldId={store.activeWorld?.world_id ?? ""}
        groupChatId={chatId ?? undefined}
        currentLocation={currentLocation}
        apiKey={store.apiKey ?? null}
        onUpdated={(newLocation, insertedMessage) => {
          setCurrentLocation(newLocation);
          if (insertedMessage) store.appendMessage(insertedMessage);
        }}
      />

      <KeepRecordModal
        open={!!keepTargetId}
        // Belt-and-suspenders reload on close; see ChatView for rationale.
        onOpenChange={(o) => { if (!o) { setKeepTargetId(null); reloadKept(); } }}
        sourceMessage={keepTargetId ? store.messages.find((m) => m.message_id === keepTargetId) ?? null : null}
        sourceSpeakerLabel={(() => {
          const m = keepTargetId ? store.messages.find((x) => x.message_id === keepTargetId) : null;
          if (!m) return "";
          if (m.role === "narrative") return "Narrative";
          if (m.role === "user") return store.userProfile?.display_name || "You";
          const senderId = m.sender_character_id;
          const sender = groupCharacters.find((c) => c.character_id === senderId);
          return sender?.display_name || "Character";
        })()}
        world={store.activeWorld}
        userProfile={store.userProfile}
        characters={groupCharacters}
        apiKey={store.apiKey}
        onSaved={(r) => { reloadKept(); setKeepToast(r); }}
      />

      <KeepToast
        entry={keepToast?.entry ?? null}
        subjectLabel={keepToast?.subjectLabel ?? ""}
        onDismiss={() => setKeepToast(null)}
        onUndone={() => { setKeepToast(null); reloadKept(); }}
      />

      <AdjustMessageModal
        open={!!adjustMessageId}
        onClose={() => setAdjustMessageId(null)}
        onAdjust={(instructions) => {
          if (adjustMessageId) store.adjustMessage(adjustMessageId, instructions);
        }}
        onEdit={(content) => {
          if (adjustMessageId) store.editMessageContent(adjustMessageId, content);
        }}
        characterName={adjustMessageId ? (() => {
          const msg = store.messages.find((m) => m.message_id === adjustMessageId);
          const ch = msg?.sender_character_id ? groupCharacters.find((c) => c.character_id === msg.sender_character_id) : undefined;
          return ch?.display_name;
        })() : undefined}
        messageContent={adjustMessageId ? store.messages.find((m) => m.message_id === adjustMessageId)?.content : undefined}
        editOnly={adjustEditOnly}
      />

      <NarrativePickerModal
        open={showNarrativePicker}
        onClose={() => setShowNarrativePicker(false)}
        onGenerate={(instructions) => store.generateNarrative(instructions)}
      />

      <IllustrationPickerModal
        open={showIllustrationPicker}
        onClose={() => setShowIllustrationPicker(false)}
        onGenerate={(tier, selectedId, instructions) => {
          setShowIllustrationPicker(false);
          store.generateIllustration(tier, instructions.trim() || undefined, selectedId, includeSceneSummary);
          setIllustrationInstructions("");
          setUsePreviousScene(false);
          setIncludeSceneSummary(true);
        }}
        illustrationInstructions={illustrationInstructions}
        setIllustrationInstructions={setIllustrationInstructions}
        usePreviousScene={usePreviousScene}
        setUsePreviousScene={setUsePreviousScene}
        includeSceneSummary={includeSceneSummary}
        setIncludeSceneSummary={setIncludeSceneSummary}
        hasPreviousIllustration={store.messages.some((m) => m.role === "illustration")}
        previousIllustrationUrl={store.messages.filter((m) => m.role === "illustration").at(-1)?.content}
        recentIllustrations={store.messages.filter((m) => m.role === "illustration").slice(-5).reverse().map((m) => ({ id: m.message_id, content: m.content }))}
      />

      <AdjustIllustrationModal
        open={!!adjustIllustrationId}
        onClose={() => setAdjustIllustrationId(null)}
        onConfirm={(instructions) => {
          if (adjustIllustrationId) {
            store.adjustIllustration(adjustIllustrationId, instructions);
            setAdjustIllustrationId(null);
          }
        }}
        adjustInstructions={adjustInstructions}
        setAdjustInstructions={setAdjustInstructions}
      />

      <VideoGenerationModal
        open={!!videoModalId}
        onClose={() => setVideoModalId(null)}
        onGenerate={(prompt) => {
          if (videoModalId) {
            store.generateVideo(videoModalId, prompt.trim() || undefined, videoDuration, videoStyle, videoIncludeContext);
            setVideoModalId(null);
          }
        }}
        onUpload={async (file) => {
          if (!videoModalId) return;
          setUploadingVideo(true);
          try {
            const reader = new FileReader();
            const dataUrl = await new Promise<string>((resolve, reject) => {
              reader.onload = () => resolve(reader.result as string);
              reader.onerror = reject;
              reader.readAsDataURL(file);
            });
            const videoFile = await api.uploadVideo(videoModalId, dataUrl);
            setVideoFiles((prev) => ({ ...prev, [videoModalId]: videoFile }));
            setVideoModalId(null);
          } catch (err) {
            store.setError?.(String(err));
          } finally {
            setUploadingVideo(false);
          }
        }}
        videoTab={videoTab}
        setVideoTab={setVideoTab}
        videoStyle={videoStyle}
        setVideoStyle={setVideoStyle}
        videoPrompt={videoPrompt}
        setVideoPrompt={setVideoPrompt}
        videoDuration={videoDuration}
        setVideoDuration={setVideoDuration}
        includeContext={videoIncludeContext}
        setIncludeContext={setVideoIncludeContext}
        uploadingVideo={uploadingVideo}
      />

      <IllustrationCarouselModal
        illustrationModalId={illustrationModalId}
        setIllustrationModalId={setIllustrationModalId}
        modalSelectedId={modalSelectedId}
        setModalSelectedId={setModalSelectedId}
        modalPlayingVideo={modalPlayingVideo}
        setModalPlayingVideo={setModalPlayingVideo}
        modalImageLoading={modalImageLoading}
        setModalImageLoading={setModalImageLoading}
        modalIllustrations={modalIllustrations}
        videoFiles={videoFiles}
        videoDataUrls={videoDataUrls}
        setVideoDataUrls={setVideoDataUrls}
        loadVideoBlobUrl={loadVideoBlobUrl}
        downloadedId={downloadedId}
        setDownloadedId={setDownloadedId}
        modalSlideshow={modalSlideshow}
        fallbackIllustrations={store.messages.filter((m) => m.role === "illustration").map((m) => ({ id: m.message_id, content: m.content }))}
        allMessages={carouselAllMessages}
        portraits={Object.fromEntries(
          Object.entries(store.activePortraits)
            .filter(([, p]) => p?.data_url)
            .map(([id, p]) => [id, p!.data_url!])
        )}
        characterColors={Object.fromEntries(
          store.characters.map((c) => [c.character_id, c.avatar_color])
        )}
        characterNames={Object.fromEntries(
          store.characters.map((c) => [c.character_id, c.display_name])
        )}
        userAvatarUrl={userAvatarUrl}
        backgroundPortraits={groupCharacters.map((ch) => store.activePortraits[ch.character_id]?.data_url).filter(Boolean) as string[]}
        playVideo={playVideo}
        playingVideo={playingVideo}
        setPlayingVideo={setPlayingVideo}
        loopVideo={loopVideo}
        setLoopVideo={setLoopVideo}
        threadId={store.messages[0]?.thread_id ?? ""}
        apiKey={store.apiKey}
        isGroup={true}
        notifyOnMessage={store.notifyOnMessage}
      />

      <RemoveVideoConfirmModal
        open={!!removeVideoConfirmId}
        onClose={() => setRemoveVideoConfirmId(null)}
        onConfirm={async () => {
          if (!removeVideoConfirmId) return;
          try {
            await api.removeVideo(removeVideoConfirmId);
            setVideoFiles((prev) => {
              const next = { ...prev };
              delete next[removeVideoConfirmId];
              return next;
            });
            setVideoDataUrls((prev) => {
              const next = { ...prev };
              delete next[removeVideoConfirmId];
              return next;
            });
            if (playingVideo === removeVideoConfirmId) setPlayingVideo(null);
          } catch { /* ignore */ }
          setRemoveVideoConfirmId(null);
        }}
      />
    </div>
  );
}

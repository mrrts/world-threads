import { useRef, useEffect, useState, useCallback } from "react";
import { useSlideshow } from "@/hooks/use-slideshow";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type Message } from "@/lib/tauri";

interface UseChatStateOptions {
  store: ReturnType<typeof useAppStore>;
  chatId: string | undefined;
  chatType: "individual" | "group";
}

// Scroll `scrollEl` toward the bottom, but never so far that the top of the
// message identified by `messageId` is pushed out of view. For short messages
// this behaves like a plain bottom-scroll; for tall ones it pins the message's
// top ~TOP_PADDING from the top of the viewport so the first lines stay
// readable while the rest extends below the fold.
const TOP_PADDING = 16;
function scrollToBottomCapped(scrollEl: HTMLElement, messageId: string | null, smooth: boolean) {
  const maxScroll = scrollEl.scrollHeight - scrollEl.clientHeight;
  let target = maxScroll;
  if (messageId) {
    const msgEl = scrollEl.querySelector(`[data-message-id="${messageId}"]`) as HTMLElement | null;
    if (msgEl) {
      const scrollRect = scrollEl.getBoundingClientRect();
      const msgRect = msgEl.getBoundingClientRect();
      const msgTopInContent = scrollEl.scrollTop + (msgRect.top - scrollRect.top);
      target = Math.max(0, Math.min(maxScroll, msgTopInContent - TOP_PADDING));
    }
  }
  if (smooth) scrollEl.scrollTo({ top: target, behavior: "smooth" });
  else scrollEl.scrollTop = target;
}

export function useChatState({ store, chatId, chatType }: UseChatStateOptions) {
  // ── Shared state ──────────────────────────────────────────────────────
  const inputValueRef = useRef("");
  const [hasInput, setHasInput] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const [userAvatarUrl, setUserAvatarUrl] = useState("");
  const [copiedError, setCopiedError] = useState(false);
  const [resetConfirmId, setResetConfirmId] = useState<string | null>(null);
  const [showNarrationSettings, setShowNarrationSettings] = useState(false);
  const [adjustIllustrationId, setAdjustIllustrationId] = useState<string | null>(null);
  const [adjustInstructions, setAdjustInstructions] = useState("");
  const [videoModalId, setVideoModalId] = useState<string | null>(null);
  const [videoPrompt, setVideoPrompt] = useState("");
  const [videoDuration, setVideoDuration] = useState(8);
  const [videoStyle, setVideoStyle] = useState("action-no-dialogue");
  const [videoIncludeContext, setVideoIncludeContext] = useState(false);
  const [videoTab, setVideoTab] = useState<"generate" | "upload">("generate");
  const [uploadingVideo, setUploadingVideo] = useState(false);
  const [downloadedId, setDownloadedId] = useState<string | null>(null);
  const [removeVideoConfirmId, setRemoveVideoConfirmId] = useState<string | null>(null);
  const [animationReadyId, setAnimationReadyId] = useState<string | null>(null);
  const [speakingId, setSpeakingId] = useState<string | null>(null);
  const [loadingSpeech, setLoadingSpeech] = useState<string | null>(null);
  const [toneMenuId, setToneMenuId] = useState<string | null>(null);
  const [cachedTones, setCachedTones] = useState<Record<string, Set<string>>>({});
  const [lastTones, setLastTones] = useState<Record<string, string>>({});
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const toneMenuRef = useRef<HTMLDivElement>(null);
  const [illustrationModalId, setIllustrationModalId] = useState<string | null>(null);
  const [modalSelectedId, setModalSelectedId] = useState<string | null>(null);
  const [modalPlayingVideo, setModalPlayingVideo] = useState(false);
  const [modalImageLoading, setModalImageLoading] = useState(false);
  const [modalIllustrations, setModalIllustrations] = useState<Array<{ id: string; content: string }>>([]);
  const [showNarrativePicker, setShowNarrativePicker] = useState(false);
  const [showSummary, setShowSummary] = useState(false);
  const [adjustMessageId, setAdjustMessageId] = useState<string | null>(null);
  const [adjustEditOnly, setAdjustEditOnly] = useState(false);
  const [showIllustrationPicker, setShowIllustrationPicker] = useState(false);
  const [illustrationInstructions, setIllustrationInstructions] = useState("");
  const [usePreviousScene, setUsePreviousScene] = useState(false);
  const [includeSceneSummary, setIncludeSceneSummary] = useState(false);
  const [narrationTone, setNarrationTone] = useState("Cinematic");
  const [narrationInstructions, setNarrationInstructions] = useState("");
  const [responseLength, setResponseLength] = useState("Short");
  const [narrationDirty, setNarrationDirty] = useState(false);
  const [playingVideo, setPlayingVideo] = useState<string | null>(null);
  const [loopVideo, setLoopVideo] = useState<Record<string, boolean>>({});
  const [videoFiles, setVideoFiles] = useState<Record<string, string>>({});
  const [videoDataUrls, setVideoDataUrls] = useState<Record<string, string>>({});
  const [showUserAvatarModal, setShowUserAvatarModal] = useState(false);
  const [carouselAllMessages, setCarouselAllMessages] = useState<Message[]>([]);

  // ── Derived state ─────────────────────────────────────────────────────
  const isSending = store.sending === chatId;
  const isGeneratingNarrative = store.generatingNarrative === chatId;
  const isGeneratingIllustration = store.generatingIllustration === chatId;
  const isGeneratingVideo = !!store.generatingVideo;

  // ── Effects ───────────────────────────────────────────────────────────

  // User avatar loading
  useEffect(() => {
    if (!store.activeWorld) { setUserAvatarUrl(""); return; }
    api.getUserAvatar(store.activeWorld.world_id).then((url) => setUserAvatarUrl(url || ""));
  }, [store.activeWorld?.world_id, store.userProfile?.avatar_file]);

  // Cached audio loading
  useEffect(() => {
    api.listCachedAudio().then(({ cached, last_tones }) => {
      const map: Record<string, Set<string>> = {};
      for (const [id, tones] of Object.entries(cached)) map[id] = new Set(tones as string[]);
      setCachedTones(map);
      setLastTones(last_tones);
    });
  }, [store.messages.length, store.adjustingMessageId]);

  // Narration settings loading
  useEffect(() => {
    if (!chatId) return;
    Promise.all([
      api.getSetting(`narration_tone.${chatId}`),
      api.getSetting(`narration_instructions.${chatId}`),
      api.getSetting(`response_length.${chatId}`),
    ]).then(([tone, instructions, length]) => {
      setNarrationTone(tone || "Cinematic");
      setNarrationInstructions(instructions || "");
      setResponseLength(length || "Short");
      setNarrationDirty(false);
    });
  }, [chatId]);

  // Scroll-at-bottom tracking
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const check = () => setIsAtBottom(el.scrollHeight - el.scrollTop - el.clientHeight < 40);
    el.addEventListener("scroll", check);
    check();
    return () => el.removeEventListener("scroll", check);
  }, [chatId]);

  useEffect(() => {
    const el = scrollRef.current;
    if (el) setIsAtBottom(el.scrollHeight - el.scrollTop - el.clientHeight < 40);
  }, [store.messages]);

  // ── Callbacks ─────────────────────────────────────────────────────────

  const loadVideoBlobUrl = useCallback(async (videoFile: string): Promise<string> => {
    const bytes = await api.getVideoBytes(videoFile);
    const blob = new Blob([new Uint8Array(bytes)], { type: "video/mp4" });
    return URL.createObjectURL(blob);
  }, []);

  // Unified handleSpeak: always accepts characterId
  const handleSpeak = useCallback(async (messageId: string, text: string, characterId: string, tone?: string) => {
    if (speakingId === messageId) {
      audioRef.current?.pause();
      setSpeakingId(null);
      return;
    }
    audioRef.current?.pause();
    setSpeakingId(null);
    setLoadingSpeech(messageId);
    try {
      const bytes = await api.generateSpeech(store.apiKey, messageId, text, characterId, tone);
      const blob = new Blob([new Uint8Array(bytes)], { type: "audio/mpeg" });
      const url = URL.createObjectURL(blob);
      const audio = new Audio(url);
      audioRef.current = audio;
      audio.onended = () => setSpeakingId(null);
      audio.play();
      setSpeakingId(messageId);
      const toneKey = (tone ?? "auto").toLowerCase();
      setCachedTones((prev) => ({ ...prev, [messageId]: new Set([...Array.from(prev[messageId] ?? []), toneKey]) }));
      setLastTones((prev) => ({ ...prev, [messageId]: toneKey }));
    } catch (e) {
      store.setError?.(String(e));
    } finally {
      setLoadingSpeech(null);
    }
  }, [speakingId, store.apiKey]);

  // Close tone menu on outside click
  useEffect(() => {
    if (!toneMenuId) return;
    const handler = (e: MouseEvent) => {
      if (toneMenuRef.current && !toneMenuRef.current.contains(e.target as Node)) {
        setToneMenuId(null);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [toneMenuId]);

  // ── Slideshow hook wiring ─────────────────────────────────────────────
  const slideshowIllustrations = modalIllustrations.map((i) => ({ id: i.id, data_url: i.content }));
  const modalSlideshow = useSlideshow({
    illustrations: slideshowIllustrations,
    videoDataUrls,
    videoFiles,
    loadVideoUrl: async (illustrationId: string, videoFile: string) => {
      const url = await loadVideoBlobUrl(videoFile);
      setVideoDataUrls((prev) => ({ ...prev, [illustrationId]: url }));
    },
  });

  // Modal sync effect
  useEffect(() => {
    if (modalSlideshow.active && modalSlideshow.currentSlide) {
      setModalSelectedId(modalSlideshow.currentSlide.illustrationId);
      setModalPlayingVideo(modalSlideshow.currentSlide.type === "video");
      setModalImageLoading(false);
    }
  }, [modalSlideshow.active, modalSlideshow.slideIndex, modalSlideshow.currentSlide]);

  const playVideo = useCallback(async (messageId: string) => {
    setPlayingVideo(messageId);
    if (!videoDataUrls[messageId] && videoFiles[messageId]) {
      try {
        const blobUrl = await loadVideoBlobUrl(videoFiles[messageId]);
        setVideoDataUrls((prev) => ({ ...prev, [messageId]: blobUrl }));
      } catch {
        setPlayingVideo(null);
      }
    }
  }, [videoDataUrls, videoFiles, loadVideoBlobUrl]);

  // Stop video when scrolled out of view
  useEffect(() => {
    if (!playingVideo) return;
    const el = document.querySelector(`[data-message-id="${playingVideo}"]`);
    if (!el) return;
    const observer = new IntersectionObserver(
      ([entry]) => { if (!entry.isIntersecting) setPlayingVideo(null); },
      { threshold: 0 }
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, [playingVideo]);

  // Load video files for illustration messages
  useEffect(() => {
    const illustrationMsgs = store.messages.filter((m) => m.role === "illustration");
    if (illustrationMsgs.length === 0) return;
    (async () => {
      const videoUpdates: Record<string, string> = {};
      for (const msg of illustrationMsgs) {
        try {
          const vf = await api.getVideoFile(msg.message_id);
          if (vf && vf.length > 0) videoUpdates[msg.message_id] = vf;
        } catch { /* ignore */ }
      }
      if (Object.keys(videoUpdates).length > 0) {
        setVideoFiles((prev) => ({ ...prev, ...videoUpdates }));
      }
    })();
  }, [store.messages]);

  // Video generation completion detection
  const prevGeneratingVideoRef = useRef<string | null>(null);
  useEffect(() => {
    if (Object.keys(store.videoFiles).length > 0) {
      setVideoFiles((prev) => ({ ...prev, ...store.videoFiles }));
    }
    const prev = prevGeneratingVideoRef.current;
    if (prev && !store.generatingVideo && store.videoFiles[prev]) {
      setAnimationReadyId(prev);
    }
    prevGeneratingVideoRef.current = store.generatingVideo;
  }, [store.videoFiles, store.generatingVideo]);

  // Scroll to bottom when new messages are appended — but cap the scroll so
  // the TOP of the arriving message stays in view. For short messages this is
  // indistinguishable from a full bottom-scroll; for long ones it avoids the
  // jarring "read from the middle" feel by keeping the first lines visible.
  const lastMessageIdRef = useRef<string | null>(null);
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;

    const lastMsg = store.messages[store.messages.length - 1];
    const lastId = lastMsg?.message_id ?? null;
    const prevLastId = lastMessageIdRef.current;

    if (lastId !== prevLastId && lastId !== null) {
      scrollToBottomCapped(el, lastId, false);
    }

    lastMessageIdRef.current = lastId;
  }, [store.messages]);

  // Scroll to bottom on mount / when chat changes / when messages first load
  const initialScrollDone = useRef(false);
  useEffect(() => {
    initialScrollDone.current = false;
  }, [chatId]);

  useEffect(() => {
    if (initialScrollDone.current || store.messages.length === 0) return;
    initialScrollDone.current = true;
    lastMessageIdRef.current = store.messages[store.messages.length - 1]?.message_id ?? null;
    const el = scrollRef.current;
    if (!el) return;
    const scroll = () => { el.scrollTop = el.scrollHeight; };
    scroll();
    const t1 = setTimeout(scroll, 200);
    const t2 = setTimeout(scroll, 600);
    const t3 = setTimeout(scroll, 1500);
    return () => { clearTimeout(t1); clearTimeout(t2); clearTimeout(t3); };
  }, [chatId, store.messages.length]);

  // Scroll to bottom when sending/generating starts
  useEffect(() => {
    if (isSending || isGeneratingNarrative || isGeneratingIllustration) {
      const el = scrollRef.current;
      if (el) {
        const lastId = store.messages[store.messages.length - 1]?.message_id ?? null;
        scrollToBottomCapped(el, lastId, true);
      }
    }
  }, [isSending, isGeneratingNarrative, isGeneratingIllustration]);

  // Scroll to bottom when illustration finishes (image needs time to load)
  const prevGeneratingIllustration = useRef(false);
  useEffect(() => {
    if (prevGeneratingIllustration.current && !isGeneratingIllustration) {
      const el = scrollRef.current;
      if (el) {
        const lastMsg = store.messages[store.messages.length - 1];
        const lastId = lastMsg?.message_id ?? null;
        const scroll = () => scrollToBottomCapped(el, lastId, true);
        scroll();
        setTimeout(scroll, 300);
        setTimeout(scroll, 800);
      }
    }
    prevGeneratingIllustration.current = isGeneratingIllustration;
  }, [isGeneratingIllustration]);

  // Auto-focus input after AI response arrives
  useEffect(() => {
    if (!isSending) {
      inputRef.current?.focus();
    }
  }, [isSending]);

  // ── Send / Retry / KeyDown ────────────────────────────────────────────

  const handleSend = useCallback(async () => {
    const text = inputValueRef.current.trim();
    if (!text || isSending) return;
    store.clearChatError();
    inputValueRef.current = "";
    setHasInput(false);
    if (inputRef.current) { inputRef.current.value = ""; inputRef.current.style.height = "auto"; }
    if (chatType === "group") {
      await store.sendGroupMessage(text);
    } else {
      await store.sendMessage(text);
    }
    inputRef.current?.focus();
  }, [isSending, chatType, store]);

  const handleRetry = useCallback(async () => {
    if (!store.lastFailedContent || isSending) return;
    const content = store.lastFailedContent;
    store.clearChatError();
    if (chatType === "group") {
      await store.sendGroupMessage(content);
    } else {
      await store.sendMessage(content);
    }
  }, [store.lastFailedContent, isSending, chatType, store]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }, [handleSend]);

  // ── Return ────────────────────────────────────────────────────────────

  return {
    // State
    inputValueRef, hasInput, setHasInput,
    scrollRef,
    inputRef,
    userAvatarUrl, setUserAvatarUrl,
    copiedError, setCopiedError,
    resetConfirmId, setResetConfirmId,
    showNarrationSettings, setShowNarrationSettings,
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
    loadingSpeech, setLoadingSpeech,
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
    narrationDirty, setNarrationDirty,
    playingVideo, setPlayingVideo,
    loopVideo, setLoopVideo,
    videoFiles, setVideoFiles,
    videoDataUrls, setVideoDataUrls,
    showUserAvatarModal, setShowUserAvatarModal,
    carouselAllMessages, setCarouselAllMessages,
    isAtBottom,

    // Derived
    isSending,
    isGeneratingNarrative,
    isGeneratingIllustration,
    isGeneratingVideo,

    // Slideshow
    modalSlideshow,

    // Callbacks
    loadVideoBlobUrl,
    handleSpeak,
    handleKeyDown,
    handleSend,
    handleRetry,
    playVideo,
  };
}

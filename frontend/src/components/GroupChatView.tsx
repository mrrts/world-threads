import React, { useRef, useEffect, useState, useCallback } from "react";
import Markdown from "react-markdown";
import { formatMessage, markdownComponents } from "@/components/chat/formatMessage";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog } from "@/components/ui/dialog";
import { Send, Loader2, X, Check, ExternalLink, BookOpen, MessageSquare, Settings, Image, Trash2, RefreshCw, SlidersHorizontal, Video, Repeat, Square, Download, Crosshair, ChevronLeft, ChevronRight, Play, Pause, Volume2, ArrowRight } from "lucide-react";
import { useSlideshow } from "@/hooks/use-slideshow";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { useAppStore } from "@/hooks/use-app-store";
import { api } from "@/lib/tauri";
import { NarrativeMessage } from "@/components/chat/NarrativeMessage";
import { ChatErrorBar } from "@/components/chat/ChatErrorBar";
import { AnimationReadyToast } from "@/components/chat/AnimationReadyToast";
import { ResetConfirmModal } from "@/components/chat/ResetConfirmModal";
import { RemoveVideoConfirmModal } from "@/components/chat/RemoveVideoConfirmModal";
import { NarrationSettingsModal } from "@/components/chat/NarrationSettingsModal";
import { IllustrationPickerModal } from "@/components/chat/IllustrationPickerModal";
import { AdjustIllustrationModal } from "@/components/chat/AdjustIllustrationModal";
import { VideoGenerationModal } from "@/components/chat/VideoGenerationModal";
import { AdjustMessageModal } from "@/components/chat/AdjustMessageModal";
import { NarrativePickerModal } from "@/components/chat/NarrativePickerModal";
import { SummaryModal } from "@/components/chat/SummaryModal";
import { TimeDivider } from "@/components/chat/TimeDivider";
import { PortraitModal } from "@/components/chat/PortraitModal";



interface Props {
  store: ReturnType<typeof useAppStore>;
}


export function GroupChatView({ store }: Props) {
  const [input, setInput] = useState("");
  const [showUserAvatarModal, setShowUserAvatarModal] = useState(false);
  const [portraitModalCharId, setPortraitModalCharId] = useState<string | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const groupCharIds: string[] = store.activeGroupChat
    ? (Array.isArray(store.activeGroupChat.character_ids) ? store.activeGroupChat.character_ids : [])
    : [];
  const groupCharacters = groupCharIds.map((id) => store.characters.find((c) => c.character_id === id)).filter(Boolean) as typeof store.characters;
  const [showGroupTalkPicker, setShowGroupTalkPicker] = useState(false);
  const talkPickerRef = useRef<HTMLDivElement>(null);
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
  const [showIllustrationPicker, setShowIllustrationPicker] = useState(false);
  const [illustrationInstructions, setIllustrationInstructions] = useState("");
  const [usePreviousScene, setUsePreviousScene] = useState(false);
  const [includeSceneSummary, setIncludeSceneSummary] = useState(false);
  const [narrationTone, setNarrationTone] = useState("Cinematic");
  const [narrationInstructions, setNarrationInstructions] = useState("");
  const [responseLength, setResponseLength] = useState("Short");
  const [narrationDirty, setNarrationDirty] = useState(false);

  const chatId = store.activeGroupChat?.group_chat_id;

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

  useEffect(() => {
    if (!store.activeWorld) { setUserAvatarUrl(""); return; }
    api.getUserAvatar(store.activeWorld.world_id).then((url) => setUserAvatarUrl(url || ""));
  }, [store.activeWorld?.world_id, store.userProfile?.avatar_file]);

  useEffect(() => {
    api.listCachedAudio().then(({ cached, last_tones }) => {
      const map: Record<string, Set<string>> = {};
      for (const [id, tones] of Object.entries(cached)) map[id] = new Set(tones);
      setCachedTones(map);
      setLastTones(last_tones);
    });
  }, [store.messages.length, store.adjustingMessageId]);

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

  // Derived: is this group chat currently loading?
  const isSending = store.sending === chatId;
  const isGeneratingNarrative = store.generatingNarrative === chatId;
  const isGeneratingIllustration = store.generatingIllustration === chatId;
  const isGeneratingVideo = !!store.generatingVideo;
  const [playingVideo, setPlayingVideo] = useState<string | null>(null);
  const [loopVideo, setLoopVideo] = useState<Record<string, boolean>>({});
  const [videoFiles, setVideoFiles] = useState<Record<string, string>>({});
  const [videoDataUrls, setVideoDataUrls] = useState<Record<string, string>>({});

  const loadVideoBlobUrl = useCallback(async (videoFile: string): Promise<string> => {
    const bytes = await api.getVideoBytes(videoFile);
    const blob = new Blob([new Uint8Array(bytes)], { type: "video/mp4" });
    return URL.createObjectURL(blob);
  }, []);

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

  useEffect(() => {
    if (modalSlideshow.active && modalSlideshow.currentSlide) {
      setModalSelectedId(modalSlideshow.currentSlide.illustrationId);
      setModalPlayingVideo(modalSlideshow.currentSlide.type === "video");
      setModalImageLoading(false);
    }
  }, [modalSlideshow.active, modalSlideshow.slideIndex, modalSlideshow.currentSlide]);

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
      setCachedTones((prev) => ({ ...prev, [messageId]: new Set([...(prev[messageId] ?? []), toneKey]) }));
      setLastTones((prev) => ({ ...prev, [messageId]: toneKey }));
    } catch (e) {
      store.setError?.(String(e));
    } finally {
      setLoadingSpeech(null);
    }
  }, [speakingId, store.apiKey]);

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

  // Also update videoFiles from store (after generateVideo completes)
  const prevGeneratingVideoRef = useRef<string | null>(null);
  useEffect(() => {
    if (Object.keys(store.videoFiles).length > 0) {
      setVideoFiles((prev) => ({ ...prev, ...store.videoFiles }));
    }
    // Detect when video generation completes: was generating, now not, and we have a new file
    const prev = prevGeneratingVideoRef.current;
    if (prev && !store.generatingVideo && store.videoFiles[prev]) {
      setAnimationReadyId(prev);
    }
    prevGeneratingVideoRef.current = store.generatingVideo;
  }, [store.videoFiles, store.generatingVideo]);

  const lastMessageIdRef = useRef<string | null>(null);

  // Scroll to bottom when new messages are appended at the end
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;

    const lastMsg = store.messages[store.messages.length - 1];
    const lastId = lastMsg?.message_id ?? null;
    const prevLastId = lastMessageIdRef.current;

    if (lastId !== prevLastId && lastId !== null) {
      el.scrollTop = el.scrollHeight;
    }

    lastMessageIdRef.current = lastId;
  }, [store.messages]);

  // Scroll to bottom on mount / when chat changes / when messages first load
  const initialScrollDone = useRef(false);
  useEffect(() => {
    initialScrollDone.current = false;
  }, [store.activeGroupChat?.group_chat_id]);

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
  }, [store.activeGroupChat?.group_chat_id, store.messages.length]);

  // Scroll to bottom when sending/generating starts
  useEffect(() => {
    if (isSending || isGeneratingNarrative || isGeneratingIllustration) {
      const el = scrollRef.current;
      if (el) el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
    }
  }, [isSending, isGeneratingNarrative, isGeneratingIllustration]);

  // Auto-focus input after AI response arrives
  useEffect(() => {
    if (!isSending) {
      inputRef.current?.focus();
    }
  }, [isSending]);


  const handleSend = async () => {
    const text = input.trim();
    if (!text || isSending) return;
    store.clearChatError();
    setInput("");
    if (inputRef.current) inputRef.current.style.height = "auto";
    await store.sendGroupMessage(text);
    inputRef.current?.focus();
  };

  const handleRetry = async () => {
    if (!store.lastFailedContent || isSending) return;
    const content = store.lastFailedContent;
    store.clearChatError();
    await store.sendGroupMessage(content);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  if (!store.activeGroupChat) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <p>Select or create a group chat to start chatting</p>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 relative">
      <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden flex">
        {groupCharacters.map((ch) => {
          const p = store.activePortraits[ch.character_id];
          return p?.data_url ? (
            <div key={ch.character_id} className="flex-1 relative">
              <img src={p.data_url} alt="" className="w-full h-full object-cover" />
            </div>
          ) : <div key={ch.character_id} className="flex-1" />;
        })}
        <div className="absolute inset-0 bg-background/65" />
      </div>
      <div className="px-4 py-3 border-b border-border flex items-center gap-3 relative z-30 bg-background">
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
        <button
          onClick={() => setShowNarrationSettings(true)}
          className={`ml-auto flex-shrink-0 h-8 rounded-lg flex items-center gap-1.5 px-2.5 text-xs font-medium transition-colors cursor-pointer ${
            (narrationTone !== "Cinematic" || responseLength !== "Short" || narrationInstructions) ? "text-amber-500 hover:text-amber-400 hover:bg-amber-500/10" : "text-muted-foreground hover:text-foreground hover:bg-accent"
          }`}
          title="Narration settings"
        >
          <Settings size={14} />
          <span>Narration</span>
        </button>
        <button
          onClick={() => setShowSummary(true)}
          className="flex-shrink-0 h-8 rounded-lg flex items-center gap-1.5 px-2.5 text-xs font-medium transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent"
          title="Generate a summary of this conversation"
        >
          <BookOpen size={14} />
          <span>Summary</span>
        </button>
      </div>

      <div className="flex-1 relative overflow-hidden z-10">
        <ScrollArea ref={scrollRef} className="h-full px-4 py-3">
        <div>
        {store.messages.length === 0 && (
          <div className="text-center text-muted-foreground py-12">
            <p className="text-lg mb-1">Start a conversation</p>
            <p className="text-sm">
              Send a message to {store.activeGroupChat?.display_name}
            </p>
          </div>
        )}
        <div className="space-y-3 max-w-2xl mx-auto">
          {store.messages.map((msg, msgIdx) => {
            const isUser = msg.role === "user";
            const isNarrative = msg.role === "narrative";
            const isPending = msg.message_id.startsWith("pending-");
            const prevMsg = msgIdx > 0 ? store.messages[msgIdx - 1] : undefined;

            if (isNarrative) {
              return (<React.Fragment key={msg.message_id}>
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
                />
              </React.Fragment>);
            }

            if (msg.role === "illustration") {
              return (<React.Fragment key={msg.message_id}>
                <TimeDivider current={msg} previous={prevMsg} />
                <div data-message-id={msg.message_id} className="flex justify-center my-3">
                  <div className="relative group/illus max-w-[95%] rounded-xl bg-gradient-to-br from-emerald-950/30 to-emerald-900/10 border border-emerald-700/20 backdrop-blur-sm">
                    <div className="flex items-center gap-1.5 px-4 pt-3 pb-1.5 text-[10px] uppercase tracking-wider text-emerald-500/70 font-semibold">
                      <Image size={12} />
                      <span>Illustration</span>
                    </div>
                    <div className="px-2 pb-2 relative">
                      <img
                        src={msg.content}
                        alt="Scene illustration"
                        loading="lazy"
                        style={store.aspectRatios[msg.message_id] ? { aspectRatio: String(store.aspectRatios[msg.message_id]) } : undefined}
                        className={`w-full rounded-lg cursor-pointer ${playingVideo === msg.message_id && videoDataUrls[msg.message_id] ? "invisible" : ""}`}
                        onClick={async () => {
                          setIllustrationModalId(msg.message_id);
                          setModalSelectedId(msg.message_id);
                          setModalPlayingVideo(false);
                          setModalImageLoading(false);
                          // Load all illustrations for the carousel
                          if (store.activeGroupChat) {
                            try {
                              const page = await api.getMessages(store.activeGroupChat.group_chat_id);
                              const illus = page.messages
                                .filter((m) => m.role === "illustration")
                                .map((m) => ({ id: m.message_id, content: m.content }));
                              setModalIllustrations(illus);
                              // Also load video files for carousel indicators
                              for (const il of illus) {
                                if (!videoFiles[il.id]) {
                                  api.getVideoFile(il.id).then((vf) => {
                                    if (vf) setVideoFiles((prev) => ({ ...prev, [il.id]: vf }));
                                  }).catch(() => {});
                                }
                              }
                            } catch { /* ignore */ }
                          }
                        }}
                      />
                      {playingVideo === msg.message_id && videoDataUrls[msg.message_id] && (
                        <>
                          <video
                            src={videoDataUrls[msg.message_id]}
                            autoPlay
                            loop={!!loopVideo[msg.message_id]}
                            playsInline
                            className="absolute inset-2 w-[calc(100%-16px)] h-[calc(100%-16px)] object-contain rounded-lg"
                            onEnded={() => { if (!loopVideo[msg.message_id]) setPlayingVideo(null); }}
                          />
                          <button
                            onClick={() => setPlayingVideo(null)}
                            className="absolute bottom-4 right-4 w-10 h-10 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm opacity-0 group-hover/illus:opacity-100"
                            title="Stop"
                          >
                            <Square size={14} fill="white" />
                          </button>
                        </>
                      )}
                      {playingVideo !== msg.message_id && videoFiles[msg.message_id] && (
                        <div className="absolute bottom-4 right-4 flex gap-1.5">
                          <button
                            onClick={() => setLoopVideo((prev) => ({ ...prev, [msg.message_id]: !prev[msg.message_id] }))}
                            className={`w-10 h-10 rounded-full backdrop-blur-sm flex items-center justify-center cursor-pointer transition-colors ${
                              loopVideo[msg.message_id]
                                ? "bg-purple-600 text-white"
                                : "bg-black/70 text-white/50 hover:text-white hover:bg-black/80"
                            }`}
                            title={loopVideo[msg.message_id] ? "Loop on" : "Loop off"}
                          >
                            <Repeat size={14} />
                          </button>
                          <button
                            onClick={() => playVideo(msg.message_id)}
                            className="w-10 h-10 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-purple-600 transition-colors backdrop-blur-sm"
                            title="Play animation"
                          >
                            <span className="text-lg ml-0.5">&#9654;</span>
                          </button>
                        </div>
                      )}
                      {playingVideo === msg.message_id && !videoDataUrls[msg.message_id] && (
                        <div className="absolute inset-2 flex items-center justify-center bg-black/30 rounded-lg">
                          <div className="animate-spin w-8 h-8 border-2 border-white/20 border-t-white rounded-full" />
                        </div>
                      )}
                      {!isPending && !isSending && (
                        <div className="absolute top-4 right-4 flex gap-1.5 opacity-0 group-hover/illus:opacity-100 transition-opacity">
                          <div className="relative group/adj">
                            <button
                              onClick={() => { setAdjustIllustrationId(msg.message_id); setAdjustInstructions(""); }}
                              className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
                            >
                              <SlidersHorizontal size={14} />
                            </button>
                            <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/adj:opacity-100 pointer-events-none transition-opacity">Adjust</span>
                          </div>
                          <div className="relative group/regen">
                            <button
                              onClick={() => store.regenerateGroupIllustration(msg.message_id)}
                              className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
                            >
                              <RefreshCw size={14} />
                            </button>
                            <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/regen:opacity-100 pointer-events-none transition-opacity">Regenerate</span>
                          </div>
                          <div className="relative group/del">
                            <button
                              onClick={() => store.deleteIllustration(msg.message_id)}
                              className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-destructive transition-colors backdrop-blur-sm"
                            >
                              <Trash2 size={14} />
                            </button>
                            <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/del:opacity-100 pointer-events-none transition-opacity">Delete</span>
                          </div>
                          <div className="relative group/pop">
                            <button
                              onClick={async () => {
                                const label = `illus-${msg.message_id.slice(0, 8)}`;
                                try {
                                  const existing = await WebviewWindow.getByLabel(label);
                                  if (existing) { await existing.setFocus(); return; }
                                } catch { /* not found */ }
                                new WebviewWindow(label, {
                                  url: `index.html?illustration=${msg.message_id}`,
                                  title: "Illustration",
                                  width: 1280,
                                  height: 760,
                                  resizable: true,
                                  decorations: true,
                                });
                              }}
                              className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
                            >
                              <ExternalLink size={14} />
                            </button>
                            <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/pop:opacity-100 pointer-events-none transition-opacity">Pop Out</span>
                          </div>
                          <div className="relative group/dl">
                            <button
                              onClick={async () => {
                                await api.downloadIllustration(msg.message_id);
                                setDownloadedId(msg.message_id);
                                setTimeout(() => setDownloadedId(null), 1500);
                              }}
                              className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-black/80 transition-colors backdrop-blur-sm"
                            >
                              {downloadedId === msg.message_id ? <Check size={14} /> : <Download size={14} />}
                            </button>
                            <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/dl:opacity-100 pointer-events-none transition-opacity">{downloadedId === msg.message_id ? "Saved!" : "Download"}</span>
                          </div>
                          <div className="relative group/vid">
                            {videoFiles[msg.message_id] ? (
                              <button
                                onClick={() => setRemoveVideoConfirmId(msg.message_id)}
                                className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-destructive transition-colors backdrop-blur-sm"
                              >
                                <span className="relative">
                                  <Video size={14} />
                                  <span className="absolute inset-0 flex items-center justify-center">
                                    <span className="block w-[18px] h-[1.5px] bg-white rotate-45" />
                                  </span>
                                </span>
                              </button>
                            ) : (
                              <button
                                onClick={() => { setVideoModalId(msg.message_id); setVideoPrompt(""); setVideoDuration(8); setVideoStyle("action-no-dialogue"); setVideoTab("generate"); }}
                                className="w-8 h-8 rounded-full bg-black/60 text-white flex items-center justify-center cursor-pointer hover:bg-purple-600 transition-colors backdrop-blur-sm"
                                disabled={isGeneratingVideo}
                              >
                                <Video size={14} />
                              </button>
                            )}
                            <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/vid:opacity-100 pointer-events-none transition-opacity">{videoFiles[msg.message_id] ? "Remove Video" : "Animate"}</span>
                          </div>
                        </div>
                      )}
                      {store.generatingVideo === msg.message_id && (
                        <div className="absolute inset-x-2 bottom-2 rounded-b-lg bg-gradient-to-t from-purple-950/90 to-purple-950/40 backdrop-blur-sm px-4 py-2.5 flex items-center gap-2 text-purple-300/90">
                          <Video size={14} className="animate-pulse" />
                          <span className="text-xs italic">Generating animation...</span>
                          <span className="w-1.5 h-1.5 rounded-full bg-purple-400/60 animate-bounce [animation-delay:0ms]" />
                          <span className="w-1.5 h-1.5 rounded-full bg-purple-400/60 animate-bounce [animation-delay:150ms]" />
                          <span className="w-1.5 h-1.5 rounded-full bg-purple-400/60 animate-bounce [animation-delay:300ms]" />
                        </div>
                      )}
                    </div>
                    <p className="text-[10px] px-4 pb-3 text-emerald-500/50 flex items-center gap-2">
                      {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                      {!isPending && (
                        <button
                          onClick={() => setResetConfirmId(msg.message_id)}
                          className="opacity-0 group-hover/illus:opacity-100 transition-opacity text-emerald-500/40 hover:text-emerald-400 cursor-pointer"
                        >
                          Reset to Here
                        </button>
                      )}
                    </p>
                  </div>
                </div>
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
              <TimeDivider current={msg} previous={prevMsg} />
              <div>
                <div className={`flex items-end gap-2 ${isUser ? "justify-end" : "justify-start"}`}>
                  {!isUser && (
                    senderPortrait?.data_url ? (
                      <button onClick={() => senderChar && setPortraitModalCharId(senderChar.character_id)} className="cursor-pointer flex-shrink-0 mb-1">
                        <img src={senderPortrait.data_url} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
                      </button>
                    ) : (
                      <span
                        className="w-[72px] h-[72px] rounded-full flex-shrink-0 mb-1 ring-1 ring-white/10"
                        style={{ backgroundColor: senderChar?.avatar_color ?? "#c4a882" }}
                      />
                    )
                  )}
                  <div
                    className={`relative group rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${
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

                    <div className={`prose prose-sm max-w-none prose-p:my-1 prose-ul:my-1 prose-ol:my-1 prose-li:my-0.5 prose-headings:my-2 prose-pre:my-2 prose-blockquote:my-2 prose-hr:my-2 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_em]:italic [&_em]:block [&_em]:border-l-2 [&_em]:border-current/20 [&_em]:pl-3 [&_em]:my-1.5 [&_em]:opacity-80 ${
                      isUser
                        ? "[--tw-prose-body:var(--color-primary-foreground)] [--tw-prose-headings:var(--color-primary-foreground)] [--tw-prose-bold:var(--color-primary-foreground)] [--tw-prose-bullets:var(--color-primary-foreground)] [--tw-prose-counters:var(--color-primary-foreground)] [--tw-prose-code:var(--color-primary-foreground)] [--tw-prose-links:var(--color-primary-foreground)] [--tw-prose-quotes:var(--color-primary-foreground)] [--tw-prose-quote-borders:rgba(255,255,255,0.3)]"
                        : "[--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-headings:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)] [--tw-prose-bullets:var(--color-secondary-foreground)] [--tw-prose-counters:var(--color-secondary-foreground)] [--tw-prose-code:var(--color-secondary-foreground)] [--tw-prose-links:var(--color-primary)] [--tw-prose-quotes:var(--color-secondary-foreground)] [--tw-prose-quote-borders:var(--color-border)]"
                    }`}>
                      <Markdown components={markdownComponents}>{formatMessage(msg.content)}</Markdown>
                    </div>
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
                    {!isUser && !isPending && (
                      <div className="absolute top-2 right-8 opacity-0 group-hover:opacity-100 transition-opacity">
                        <div className="relative group/madj">
                          <button
                            onClick={() => setAdjustMessageId(msg.message_id)}
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
                      <img src={userAvatarUrl} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
                    </button>
                  )}
                </div>
              </div>
              </React.Fragment>
            );
          })}
          {isSending && !isGeneratingNarrative && !isGeneratingIllustration && !isGeneratingVideo && (() => {
            const sendingChar = store.sendingCharacterId
              ? groupCharacters.find((c) => c.character_id === store.sendingCharacterId)
              : groupCharacters[0];
            const sendingPortrait = sendingChar ? store.activePortraits[sendingChar.character_id] : undefined;
            return (
            <div className="flex items-end gap-2 justify-start">
              {sendingPortrait?.data_url ? (
                <img src={sendingPortrait.data_url} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border flex-shrink-0 mb-1" />
              ) : (
                <span
                  className="w-[72px] h-[72px] rounded-full flex-shrink-0 mb-1 ring-1 ring-white/10"
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
      <AnimationReadyToast
        animationReadyId={animationReadyId}
        onGo={() => {
          const el = document.querySelector(`[data-message-id="${animationReadyId}"]`);
          if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
          setAnimationReadyId(null);
        }}
        onDismiss={() => setAnimationReadyId(null)}
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
        <div className="flex gap-2 max-w-2xl mx-auto items-stretch">
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
                  const userName = store.userProfile?.display_name ?? "me";
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
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              e.target.style.height = "";
              if (e.target.scrollHeight > e.target.offsetHeight) {
                e.target.style.height = Math.min(e.target.scrollHeight, 200) + "px";
              }
              requestAnimationFrame(() => {
                const el = scrollRef.current;
                if (el) el.scrollTop = el.scrollHeight;
              });
            }}
            onKeyDown={handleKeyDown}
            placeholder={`Talk to ${store.activeGroupChat?.display_name ?? "the group"}...`}
            className="flex-1 self-stretch max-h-[200px] resize-none rounded-xl border border-input bg-transparent px-4 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]"
            rows={1}
            disabled={isSending || (store.autoRespond && !store.apiKey)}
          />
          <Button
            size="icon"
            className="rounded-xl self-stretch w-10 flex-shrink-0"
            onClick={handleSend}
            disabled={!input.trim() || isSending || (store.autoRespond && !store.apiKey)}
          >
            {isSending ? <Loader2 size={16} className="animate-spin" /> : <Send size={16} />}
          </Button>
        </div>
      </div>

      <PortraitModal
        characterId={portraitModalCharId}
        characterName={portraitModalCharId ? groupCharacters.find((c) => c.character_id === portraitModalCharId)?.display_name : undefined}
        onClose={() => setPortraitModalCharId(null)}
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

      <NarrationSettingsModal
        open={showNarrationSettings}
        onClose={() => { setShowNarrationSettings(false); setNarrationDirty(false); }}
        charId={chatId}
        narrationTone={narrationTone}
        setNarrationTone={setNarrationTone}
        narrationInstructions={narrationInstructions}
        setNarrationInstructions={setNarrationInstructions}
        responseLength={responseLength}
        setResponseLength={setResponseLength}
        narrationDirty={narrationDirty}
        setNarrationDirty={setNarrationDirty}
        onSave={async () => {
          if (!chatId) return;
          await Promise.all([
            api.setSetting(`narration_tone.${chatId}`, narrationTone),
            api.setSetting(`narration_instructions.${chatId}`, narrationInstructions),
            api.setSetting(`response_length.${chatId}`, responseLength),
          ]);
          setNarrationDirty(false);
          setShowNarrationSettings(false);
        }}
        onClearHistory={store.activeGroupChat ? () => {
          store.clearGroupChatHistory(store.activeGroupChat!.group_chat_id);
          setShowNarrationSettings(false);
        } : undefined}
      />

      <SummaryModal
        open={showSummary}
        onClose={() => setShowSummary(false)}
        title={`Summary: ${groupCharacters.map((c) => c.display_name).join(" & ")}`}
        generateSummary={() => api.generateGroupChatSummary(store.apiKey, store.activeGroupChat?.group_chat_id ?? "")}
      />

      <AdjustMessageModal
        open={!!adjustMessageId}
        onClose={() => setAdjustMessageId(null)}
        onAdjust={(instructions) => {
          if (adjustMessageId) store.adjustMessage(adjustMessageId, instructions);
        }}
        characterName={adjustMessageId ? (() => {
          const msg = store.messages.find((m) => m.message_id === adjustMessageId);
          const ch = msg?.sender_character_id ? groupCharacters.find((c) => c.character_id === msg.sender_character_id) : undefined;
          return ch?.display_name;
        })() : undefined}
      />

      <NarrativePickerModal
        open={showNarrativePicker}
        onClose={() => setShowNarrativePicker(false)}
        onGenerate={(instructions) => store.generateGroupNarrative(instructions)}
      />

      <IllustrationPickerModal
        open={showIllustrationPicker}
        onClose={() => setShowIllustrationPicker(false)}
        onGenerate={(tier) => {
          const prevIllus = store.messages.filter((m) => m.role === "illustration");
          const lastIllus = prevIllus[prevIllus.length - 1];
          const prevId = usePreviousScene && lastIllus ? lastIllus.message_id : undefined;
          setShowIllustrationPicker(false);
          store.generateGroupIllustration(tier, illustrationInstructions.trim() || undefined, prevId, includeSceneSummary);
          setIllustrationInstructions("");
          setUsePreviousScene(false);
          setIncludeSceneSummary(false);
        }}
        illustrationInstructions={illustrationInstructions}
        setIllustrationInstructions={setIllustrationInstructions}
        usePreviousScene={usePreviousScene}
        setUsePreviousScene={setUsePreviousScene}
        includeSceneSummary={includeSceneSummary}
        setIncludeSceneSummary={setIncludeSceneSummary}
        hasPreviousIllustration={store.messages.some((m) => m.role === "illustration")}
        previousIllustrationUrl={store.messages.filter((m) => m.role === "illustration").at(-1)?.content}
      />

      <AdjustIllustrationModal
        open={!!adjustIllustrationId}
        onClose={() => setAdjustIllustrationId(null)}
        onConfirm={(instructions) => {
          if (adjustIllustrationId) {
            store.adjustGroupIllustration(adjustIllustrationId, instructions);
            setAdjustIllustrationId(null);
          }
        }}
        adjustInstructions={adjustInstructions}
        setAdjustInstructions={setAdjustInstructions}
      />

      <VideoGenerationModal
        open={!!videoModalId}
        onClose={() => setVideoModalId(null)}
        onGenerate={() => {
          if (videoModalId) {
            store.generateVideo(videoModalId, videoPrompt.trim() || undefined, videoDuration, videoStyle, videoIncludeContext);
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

      {illustrationModalId && (() => {
        const selId = modalSelectedId ?? illustrationModalId;
        const allIllustrations = modalIllustrations.length > 0
          ? modalIllustrations
          : store.messages.filter((m) => m.role === "illustration").map((m) => ({ id: m.message_id, content: m.content }));
        const selectedItem = allIllustrations.find((i) => i.id === selId);
        if (!selectedItem) return null;
        const modalVideoFile = videoFiles[selId];
        const modalVideoUrl = videoDataUrls[selId];
        return (
          <Dialog open onClose={() => { setIllustrationModalId(null); setModalPlayingVideo(false); if (modalSlideshow.active) modalSlideshow.toggle(); }} className="max-w-[90vw]">
            <div className="flex flex-col max-h-[90vh]">
              <div className="relative flex items-center justify-center min-h-0 flex-1 overflow-hidden group/modal">
                {modalImageLoading && !modalPlayingVideo && (
                  <div className="absolute inset-0 flex items-center justify-center z-10">
                    <div className="animate-spin w-6 h-6 border-2 border-white/20 border-t-white rounded-full" />
                  </div>
                )}
                {modalPlayingVideo && modalVideoUrl ? (
                  <video
                    key={`modal-video-${selId}`}
                    src={modalVideoUrl}
                    autoPlay
                    loop={!modalSlideshow.active}
                    playsInline
                    className="max-w-full max-h-[75vh] object-contain rounded-t-2xl"
                    onTimeUpdate={modalSlideshow.active ? (e) => {
                      const v = e.currentTarget;
                      modalSlideshow.onVideoTimeUpdate(v.currentTime, v.duration);
                    } : undefined}
                    onEnded={modalSlideshow.active ? modalSlideshow.onVideoEnded : undefined}
                  />
                ) : (
                  <img
                    key={`modal-img-${selId}`}
                    src={selectedItem.content}
                    alt="Illustration"
                    className={`max-w-full max-h-[75vh] object-contain rounded-t-2xl ${modalImageLoading ? "opacity-0" : "opacity-100"} transition-opacity`}
                    onLoad={() => setModalImageLoading(false)}
                  />
                )}
                <button
                  onClick={() => { setIllustrationModalId(null); setModalPlayingVideo(false); if (modalSlideshow.active) modalSlideshow.toggle(); }}
                  className="absolute top-3 right-3 z-20 w-8 h-8 flex items-center justify-center rounded-full bg-black/50 text-white hover:bg-black/70 transition-colors cursor-pointer backdrop-blur-sm"
                >
                  <X size={16} />
                </button>
                <div className="absolute top-3 left-3 z-20 flex gap-1.5 opacity-0 group-hover/modal:opacity-100 transition-opacity">
                  <div className="relative group/mdl-dl">
                    <button
                      onClick={async () => {
                        await api.downloadIllustration(selId);
                        setDownloadedId(selId);
                        setTimeout(() => setDownloadedId(null), 1500);
                      }}
                      className="w-8 h-8 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm"
                    >
                      {downloadedId === selId ? <Check size={14} /> : <Download size={14} />}
                    </button>
                    <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mdl-dl:opacity-100 pointer-events-none transition-opacity">{downloadedId === selId ? "Saved!" : "Download"}</span>
                  </div>
                  <div className="relative group/mdl-goto">
                    <button
                      onClick={async () => {
                        setIllustrationModalId(null);
                        setModalPlayingVideo(false);
                        if (modalSlideshow.active) modalSlideshow.toggle();
                        await new Promise((r) => setTimeout(r, 100));
                        const el = document.querySelector(`[data-message-id="${selId}"]`);
                        if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
                      }}
                      className="w-8 h-8 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm"
                    >
                      <Crosshair size={14} />
                    </button>
                    <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mdl-goto:opacity-100 pointer-events-none transition-opacity">Go to Image</span>
                  </div>
                  {allIllustrations.length > 1 && (
                    <div className="relative group/mdl-ss">
                      <button
                        onClick={() => {
                          if (!modalSlideshow.active) modalSlideshow.jumpTo(selId);
                          modalSlideshow.toggle();
                        }}
                        className={`w-8 h-8 rounded-full flex items-center justify-center cursor-pointer transition-colors backdrop-blur-sm ${
                          modalSlideshow.active ? "bg-primary/80 text-white hover:bg-primary" : "bg-black/50 text-white hover:bg-black/70"
                        }`}
                      >
                        {modalSlideshow.active ? <Pause size={14} /> : <Play size={14} />}
                      </button>
                      <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mdl-ss:opacity-100 pointer-events-none transition-opacity">Slideshow</span>
                    </div>
                  )}
                </div>
                {modalVideoFile && !modalPlayingVideo && !modalSlideshow.active && (
                  <button
                    onClick={async () => {
                      if (!modalVideoUrl) {
                        try {
                          const url = await loadVideoBlobUrl(modalVideoFile);
                          setVideoDataUrls((prev) => ({ ...prev, [selId]: url }));
                        } catch { return; }
                      }
                      setModalPlayingVideo(true);
                    }}
                    className="absolute bottom-4 right-4 z-20 w-12 h-12 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-purple-600 transition-colors backdrop-blur-sm"
                  >
                    <span className="text-xl ml-0.5">&#9654;</span>
                  </button>
                )}
                {modalPlayingVideo && !modalSlideshow.active && (
                  <button
                    onClick={() => setModalPlayingVideo(false)}
                    className="absolute bottom-4 right-4 z-20 w-12 h-12 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm"
                  >
                    <Square size={16} fill="white" />
                  </button>
                )}
                {allIllustrations.length > 1 && !modalSlideshow.active && (<>
                  <button
                    onClick={() => {
                      const idx = allIllustrations.findIndex((i) => i.id === selId);
                      const prev = idx <= 0 ? allIllustrations.length - 1 : idx - 1;
                      setModalSelectedId(allIllustrations[prev].id);
                      setModalImageLoading(true);
                      setModalPlayingVideo(false);
                    }}
                    className="absolute left-2 top-1/2 -translate-y-1/2 z-20 w-10 h-10 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-all backdrop-blur-sm opacity-0 group-hover/modal:opacity-100"
                  >
                    <ChevronLeft size={20} />
                  </button>
                  <button
                    onClick={() => {
                      const idx = allIllustrations.findIndex((i) => i.id === selId);
                      const next = idx >= allIllustrations.length - 1 ? 0 : idx + 1;
                      setModalSelectedId(allIllustrations[next].id);
                      setModalImageLoading(true);
                      setModalPlayingVideo(false);
                    }}
                    className="absolute right-2 top-1/2 -translate-y-1/2 z-20 w-10 h-10 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-all backdrop-blur-sm opacity-0 group-hover/modal:opacity-100"
                  >
                    <ChevronRight size={20} />
                  </button>
                </>)}
                {modalSlideshow.active && (
                  <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/10 z-30">
                    <div
                      className="h-full bg-primary transition-none"
                      style={{ width: `${modalSlideshow.progress * 100}%` }}
                    />
                  </div>
                )}
              </div>
              {allIllustrations.length > 1 && (
                <div className="flex-shrink-0 bg-card/80 backdrop-blur-sm rounded-b-2xl px-3 py-2 border-t border-border/30">
                  <div className="flex gap-1.5 overflow-x-auto scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]">
                    {allIllustrations.map((illus) => (
                      <button
                        key={illus.id}
                        ref={illus.id === selId ? (el) => {
                          if (!el) return;
                          const c = el.parentElement;
                          if (c) c.scrollTo({ left: el.offsetLeft - c.offsetWidth / 2 + el.offsetWidth / 2, behavior: "smooth" });
                        } : undefined}
                        onClick={() => {
                          if (modalSlideshow.active) {
                            modalSlideshow.jumpTo(illus.id);
                          } else {
                            setModalSelectedId(illus.id);
                            setModalImageLoading(true);
                            setModalPlayingVideo(false);
                          }
                        }}
                        className={`relative flex-shrink-0 w-16 h-11 rounded-lg overflow-hidden transition-all cursor-pointer ${
                          illus.id === selId
                            ? "ring-2 ring-primary ring-offset-1 ring-offset-card"
                            : "ring-1 ring-border opacity-60 hover:opacity-100"
                        }`}
                      >
                        <img src={illus.content} alt="" className="w-full h-full object-cover" />
                        {videoFiles[illus.id] && (
                          <div className="absolute bottom-0.5 right-0.5 w-3.5 h-3.5 rounded-full bg-purple-600 flex items-center justify-center">
                            <span className="text-white text-[6px]">&#9654;</span>
                          </div>
                        )}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </Dialog>
        );
      })()}

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

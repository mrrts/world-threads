import React, { useRef, useEffect, useState, useCallback } from "react";
import Markdown from "react-markdown";
import { formatMessage, markdownComponents, remarkPlugins, rehypePlugins } from "@/components/chat/formatMessage";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog } from "@/components/ui/dialog";
import { Send, Loader2, X, BookOpen, MessageSquare, Compass, Settings, Image, Trash2, SlidersHorizontal, Pencil, Square, Crosshair, ChevronLeft, ChevronRight, ChevronDown, Play, Pause, Volume2, ArrowRight } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api } from "@/lib/tauri";
import { NarrativeMessage } from "@/components/chat/NarrativeMessage";
import { ChatErrorBar } from "@/components/chat/ChatErrorBar";
import { AnimationReadyToast } from "@/components/chat/AnimationReadyToast";
import { ResetConfirmModal } from "@/components/chat/ResetConfirmModal";
import { RemoveVideoConfirmModal } from "@/components/chat/RemoveVideoConfirmModal";
import { IllustrationPickerModal } from "@/components/chat/IllustrationPickerModal";
import { AdjustIllustrationModal } from "@/components/chat/AdjustIllustrationModal";
import { VideoGenerationModal } from "@/components/chat/VideoGenerationModal";
import { IllustrationCarouselModal } from "@/components/chat/IllustrationCarouselModal";
import { AdjustMessageModal } from "@/components/chat/AdjustMessageModal";
import { NarrativePickerModal } from "@/components/chat/NarrativePickerModal";
import { SummaryModal } from "@/components/chat/SummaryModal";
import { TimeDivider } from "@/components/chat/TimeDivider";
import { ContextMessage } from "@/components/chat/ContextMessage";
import { PortraitModal } from "@/components/chat/PortraitModal";
import { StoryConsultantModal } from "@/components/chat/StoryConsultantModal";
import { IllustrationMessage } from "@/components/chat/IllustrationMessage";
import { useChatState } from "@/hooks/use-chat-state";



interface Props {
  store: ReturnType<typeof useAppStore>;
  onNavigateToCharacter?: (characterId: string) => void;
}


export function GroupChatView({ store, onNavigateToCharacter }: Props) {
  // ── Group-specific state ─────────────────────────────────────────────
  const [showGroupTalkPicker, setShowGroupTalkPicker] = useState(false);
  const talkPickerRef = useRef<HTMLDivElement>(null);
  const [portraitModalCharId, setPortraitModalCharId] = useState<string | null>(null);
  const [showConsultant, setShowConsultant] = useState(false);
  const [showSettingsPopover, setShowSettingsPopover] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [clearKeepMedia, setClearKeepMedia] = useState(true);
  const settingsPopoverRef = useRef<HTMLDivElement>(null);
  const [showGroupPopover, setShowGroupPopover] = useState(false);
  const groupPopoverTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);

  const groupCharIds: string[] = store.activeGroupChat
    ? (Array.isArray(store.activeGroupChat.character_ids) ? store.activeGroupChat.character_ids : [])
    : [];
  const groupCharacters = groupCharIds.map((id) => store.characters.find((c) => c.character_id === id)).filter(Boolean) as typeof store.characters;

  // ── Shared chat state from hook ──────────────────────────────────────
  const chatId = store.activeGroupChat?.group_chat_id;

  const {
    inputValueRef, hasInput, setHasInput,
    scrollRef,
    inputRef,
    userAvatarUrl,
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

  // Auto-save chat settings
  useEffect(() => {
    if (!chatId || !narrationDirty) return;
    const timer = setTimeout(async () => {
      await Promise.all([
        api.setSetting(`narration_tone.${chatId}`, narrationTone),
        api.setSetting(`narration_instructions.${chatId}`, narrationInstructions),
        api.setSetting(`response_length.${chatId}`, responseLength),
      ]);
      setNarrationDirty(false);
    }, 600);
    return () => clearTimeout(timer);
  }, [narrationTone, narrationInstructions, responseLength, narrationDirty, chatId]);

  const openGallery = useCallback(async () => {
    const lastIllus = store.messages.filter((m) => m.role === "illustration").at(-1);
    if (!lastIllus || !store.activeGroupChat) return;
    setIllustrationModalId(lastIllus.message_id);
    setModalSelectedId(lastIllus.message_id);
    setModalPlayingVideo(false);
    setModalImageLoading(false);
    try {
      const page = await api.getGroupMessages(store.activeGroupChat.group_chat_id);
      const illus = page.messages.filter((m) => m.role === "illustration").map((m) => ({ id: m.message_id, content: m.content }));
      setModalIllustrations(illus);
      setCarouselAllMessages(page.messages);
      for (const il of illus) {
        if (!videoFiles[il.id]) api.getVideoFile(il.id).then((vf) => { if (vf) setVideoFiles((prev) => ({ ...prev, [il.id]: vf })); }).catch(() => {});
      }
    } catch {}
  }, [store.messages, store.activeGroupChat]);

  useEffect(() => {
    const onGallery = () => openGallery();
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
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
        <div className="ml-auto relative group/gallery">
          <button
            onClick={openGallery}
            className="flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent disabled:opacity-40 disabled:cursor-not-allowed"
            disabled={!store.messages.some((m) => m.role === "illustration")}
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
            <BookOpen size={15} />
          </button>
          <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/summary:opacity-100 pointer-events-none transition-opacity">Summary</span>
        </div>
        <div className="relative group/consultant">
          <button
            onClick={() => setShowConsultant(true)}
            className="flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center transition-colors cursor-pointer text-muted-foreground hover:text-foreground hover:bg-accent"
          >
            <Compass size={15} />
          </button>
          <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/consultant:opacity-100 pointer-events-none transition-opacity">Consultant</span>
        </div>
      </div>

      <div className="flex-1 relative overflow-hidden z-10">
        <ScrollArea ref={scrollRef} className="h-full px-4 py-3">
        <div>
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
          {store.messages.filter((m) => m.content || m.role === "illustration").map((msg, msgIdx, filteredMsgs) => {
            const isUser = msg.role === "user";
            const isNarrative = msg.role === "narrative";
            const isPending = msg.message_id.startsWith("pending-");
            const prevMsg = msgIdx > 0 ? filteredMsgs[msgIdx - 1] : undefined;

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
                  onDelete={(id) => store.deleteMessage(id)}
                />
              </React.Fragment>);
            }

            if (msg.role === "context") {
              return (<React.Fragment key={msg.message_id}>
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

            if (msg.role === "illustration") {
              return (<React.Fragment key={msg.message_id}>
                <TimeDivider current={msg} previous={prevMsg} />
                <IllustrationMessage
                  msg={msg} isPending={isPending} isSending={isSending} isGeneratingVideo={isGeneratingVideo} store={store}
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
                    className={`relative group rounded-2xl px-4 py-2.5 text-base leading-relaxed ${
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
                      <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(msg.content)}</Markdown>
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
            defaultValue=""
            onChange={(e) => {
              inputValueRef.current = e.target.value;
              const empty = !e.target.value.trim();
              if (hasInput === empty) setHasInput(!empty);
              e.target.style.height = "";
              if (e.target.scrollHeight > e.target.offsetHeight) {
                e.target.style.height = Math.min(e.target.scrollHeight, 200) + "px";
              }
            }}
            onKeyDown={handleKeyDown}
            placeholder={`Talk to ${store.activeGroupChat?.display_name ?? "the group"}...`}
            className="flex-1 self-stretch max-h-[200px] resize-none rounded-xl border border-input bg-transparent px-4 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]"
            rows={1}
            disabled={isSending || (store.autoRespond && !store.apiKey)}
          />
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
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1.5">Custom Instructions</label>
                  <textarea
                    value={narrationInstructions}
                    onChange={(e) => { setNarrationInstructions(e.target.value); setNarrationDirty(true); }}
                    placeholder="e.g. Describe the weather shifting..."
                    className="w-full min-h-[60px] max-h-[120px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-xs placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                    rows={2}
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
            <span className="text-muted-foreground/50 text-right" style={{ fontSize: "12px" }}>Response Length: {responseLength}</span>
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
        onClose={() => setShowConsultant(false)}
        apiKey={store.apiKey}
        characterId={null}
        groupChatId={store.activeGroupChat?.group_chat_id ?? null}
        threadId={store.messages[0]?.thread_id ?? ""}
        characterNames={groupCharacters.map((c) => c.display_name)}
        worldImageUrl={store.activeWorldImage?.data_url}
        portraits={Object.fromEntries(
          Object.entries(store.activePortraits).filter(([, p]) => p?.data_url).map(([id, p]) => [id, p!.data_url!])
        )}
        userAvatarUrl={userAvatarUrl}
        notifyOnMessage={store.notifyOnMessage}
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
        generateSummary={() => api.generateGroupChatSummary(store.apiKey, store.activeGroupChat?.group_chat_id ?? "")}
        notifyOnMessage={store.notifyOnMessage}
        characters={store.characters}
        groupChats={store.groupChats}
        activePortraits={store.activePortraits}
        currentGroupChatId={store.activeGroupChat?.group_chat_id}
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

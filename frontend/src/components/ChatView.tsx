import React, { useState } from "react";
import Markdown from "react-markdown";
import { formatMessage, markdownComponents } from "@/components/chat/formatMessage";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog } from "@/components/ui/dialog";
import { Send, Loader2, SmilePlus, X, Copy, ExternalLink, BookOpen, RotateCcw, MessageSquare, Settings, Image, Trash2, SlidersHorizontal, Square, Play, Volume2 } from "lucide-react";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type Reaction } from "@/lib/tauri";
import { EmojiPicker } from "@/components/chat/EmojiPicker";
import { ReactionBubbles } from "@/components/chat/ReactionBubbles";
import { NarrativeMessage } from "@/components/chat/NarrativeMessage";
import { ChatErrorBar } from "@/components/chat/ChatErrorBar";
import { AnimationReadyToast } from "@/components/chat/AnimationReadyToast";
import { ResetConfirmModal } from "@/components/chat/ResetConfirmModal";
import { RemoveVideoConfirmModal } from "@/components/chat/RemoveVideoConfirmModal";
import { NarrationSettingsModal } from "@/components/chat/NarrationSettingsModal";
import { IllustrationPickerModal } from "@/components/chat/IllustrationPickerModal";
import { AdjustIllustrationModal } from "@/components/chat/AdjustIllustrationModal";
import { VideoGenerationModal } from "@/components/chat/VideoGenerationModal";
import { IllustrationCarouselModal } from "@/components/chat/IllustrationCarouselModal";
import { AdjustMessageModal } from "@/components/chat/AdjustMessageModal";
import { SummaryModal } from "@/components/chat/SummaryModal";
import { TimeDivider } from "@/components/chat/TimeDivider";
import { ContextMessage } from "@/components/chat/ContextMessage";
import { NarrativePickerModal } from "@/components/chat/NarrativePickerModal";
import { PortraitModal } from "@/components/chat/PortraitModal";
import { useChatState } from "@/hooks/use-chat-state";




interface Props {
  store: ReturnType<typeof useAppStore>;
}


export function ChatView({ store }: Props) {
  const [pickerMessageId, setPickerMessageId] = useState<string | null>(null);
  const [showPortraitModal, setShowPortraitModal] = useState(false);
  const [showIdentityPopover, setShowIdentityPopover] = useState(false);

  const charId = store.activeCharacter?.character_id;
  const charPortrait = store.activeCharacter ? store.activePortraits[store.activeCharacter?.character_id] : undefined;

  const chatState = useChatState({ store, chatId: charId, chatType: "individual" });

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
    adjustMessageId, setAdjustMessageId,
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
  } = chatState;

  if (!store.activeCharacter) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <p>Select or create a character to start chatting</p>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 relative">
      {charPortrait?.data_url ? (
        <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden">
          <img
            src={charPortrait.data_url}
            alt=""
            className="w-full h-full object-cover"
          />
          <div className="absolute inset-0 bg-background/60" />
        </div>
      ) : null}
      <div className="px-4 py-3 border-b border-border flex items-center gap-3 relative z-30 bg-background">
        {charPortrait?.data_url ? (
          <div className="relative group flex-shrink-0">
            <button
              onClick={async () => {
                const label = `portrait-${store.activeCharacter!.character_id.slice(0, 8)}`;
                try {
                  const existing = await WebviewWindow.getByLabel(label);
                  if (existing) { await existing.setFocus(); return; }
                } catch { /* not found, create new */ }
                new WebviewWindow(label, {
                  url: `index.html?portrait=${store.activeCharacter!.character_id}`,
                  title: store.activeCharacter!.display_name,
                  width: 420,
                  height: 480,
                  resizable: true,
                  decorations: true,
                  titleBarStyle: "overlay",
                  hiddenTitle: true,
                  alwaysOnTop: true,
                });
              }}
              className="cursor-pointer"
              title="Open portrait in window"
            >
              <img src={charPortrait.data_url} alt="" className="w-9 h-9 rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
              <span className="absolute -bottom-0.5 -right-0.5 w-4 h-4 rounded-full bg-card border border-border flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                <ExternalLink size={8} />
              </span>
            </button>
          </div>
        ) : store.activeCharacter ? (
          <span
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: store.activeCharacter?.avatar_color }}
          />
        ) : null}
        <h1 className="font-semibold">{store.activeCharacter?.display_name}</h1>
        {store.activeCharacter?.identity && (
          <div className="relative flex-1 min-w-0">
            <span
              className="text-xs text-muted-foreground truncate block cursor-default"
              onMouseEnter={() => setShowIdentityPopover(true)}
              onMouseLeave={() => setShowIdentityPopover(false)}
            >
              {store.activeCharacter?.identity.slice(0, 60)}...
            </span>
            {showIdentityPopover && (
              <div
                className="absolute left-0 top-full mt-2 z-50 w-80 bg-card border border-border rounded-xl shadow-xl p-4 animate-in fade-in zoom-in-95 duration-150"
                onMouseEnter={() => setShowIdentityPopover(true)}
                onMouseLeave={() => setShowIdentityPopover(false)}
              >
                {charPortrait?.data_url && (
                  <img src={charPortrait.data_url} alt="" className="w-full rounded-lg object-cover aspect-square mb-3" />
                )}
                <p className="font-semibold text-sm mb-1">{store.activeCharacter?.display_name}</p>
                <p className="text-xs text-muted-foreground leading-relaxed whitespace-pre-wrap">{store.activeCharacter?.identity}</p>
              </div>
            )}
          </div>
        )}
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
          store.loadingChat ? (
            <div className="flex items-center justify-center py-16">
              <Loader2 size={24} className="animate-spin text-muted-foreground" />
            </div>
          ) : (
            <div className="text-center text-muted-foreground py-12">
              <p className="text-lg mb-1">Start a conversation</p>
              <p className="text-sm">
                Send a message to {store.activeCharacter?.display_name}
              </p>
            </div>
          )
        )}
        <div className="space-y-3 max-w-2xl mx-auto">
          {store.messages.map((msg, msgIdx) => {
            const isUser = msg.role === "user";
            const isNarrative = msg.role === "narrative";
            const isPending = msg.message_id.startsWith("pending-");
            const prevMsg = msgIdx > 0 ? store.messages[msgIdx - 1] : undefined;
            const reactions = store.reactions[msg.message_id] ?? [];
            const showPicker = pickerMessageId === msg.message_id;

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
                  onSpeak={(id, text, tone) => handleSpeak(id, text, store.activeCharacter?.character_id ?? "", tone)}
                  onStopSpeaking={() => { audioRef.current?.pause(); setSpeakingId(null); }}
                  onDeleteAudio={async (id) => { await api.deleteMessageAudio(id); setCachedTones((prev) => { const next = { ...prev }; delete next[id]; return next; }); setLastTones((prev) => { const next = { ...prev }; delete next[id]; return next; }); }}
                  toneMenuRef={toneMenuRef}
                  adjustingMessageId={store.adjustingMessageId}
                  onAdjust={(id) => setAdjustMessageId(id)}
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
                    if (!store.activeCharacter) return;
                    try {
                      const page = await api.getMessages(store.activeCharacter.character_id);
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

            return (
              <React.Fragment key={msg.message_id}>
              <TimeDivider current={msg} previous={prevMsg} />
              <div>
                <div className={`flex items-end gap-2 ${isUser ? "justify-end" : "justify-start"}`}>
                  {!isUser && (
                    charPortrait?.data_url ? (
                      <button onClick={() => setShowPortraitModal(true)} className="cursor-pointer flex-shrink-0 mb-1">
                        <img src={charPortrait.data_url} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
                      </button>
                    ) : (
                      <span
                        className="w-[72px] h-[72px] rounded-full flex-shrink-0 mb-1 ring-1 ring-white/10"
                        style={{ backgroundColor: store.activeCharacter?.avatar_color ?? "#c4a882" }}
                      />
                    )
                  )}
                  <div
                    className={`relative group rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${
                      isUser
                        ? "bg-primary text-primary-foreground rounded-br-md max-w-[80%]"
                        : "bg-secondary/40 text-secondary-foreground rounded-bl-md max-w-[80%] border border-border/30"
                    }`}
                  >
                    {/* Top corner button — reaction (user) or speak (character) */}
                    {!isPending && isUser && (
                      <button
                        onClick={() => setPickerMessageId(showPicker ? null : msg.message_id)}
                        className="absolute -top-2.5 -left-2.5 z-10 w-7 h-7 flex items-center justify-center rounded-full bg-white shadow-md border border-border/50 text-muted-foreground hover:text-foreground hover:scale-110 opacity-0 group-hover:opacity-100 transition-all cursor-pointer"
                      >
                        <SmilePlus size={16} strokeWidth={2} />
                      </button>
                    )}
                    {!isPending && !isUser && (() => {
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
                                  handleSpeak(msg.message_id, msg.content, store.activeCharacter?.character_id ?? "", lastTone === "auto" ? "Auto" : lastTone.charAt(0).toUpperCase() + lastTone.slice(1));
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
                                      handleSpeak(msg.message_id, msg.content, store.activeCharacter?.character_id ?? "", tone);
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

                    {/* Emoji picker */}
                    {showPicker && (
                      <div className={`absolute top-0 z-50 ${isUser ? "right-full mr-2" : "left-full ml-2"}`}>
                        <EmojiPicker
                          onSelect={(emoji) => store.toggleReaction(msg.message_id, emoji)}
                          onClose={() => setPickerMessageId(null)}
                        />
                      </div>
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

                <div className={!isUser ? "pl-20" : userAvatarUrl ? "pr-20" : ""}>
                  <ReactionBubbles reactions={reactions} isUser={isUser} />
                </div>
              </div>
              </React.Fragment>
            );
          })}
          {isSending && !isGeneratingNarrative && !isGeneratingIllustration && !isGeneratingVideo && (
            <div className="flex items-end gap-2 justify-start">
              {charPortrait?.data_url ? (
                <button onClick={() => setShowPortraitModal(true)} className="cursor-pointer flex-shrink-0 mb-1">
                  <img src={charPortrait.data_url} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
                </button>
              ) : (
                <span
                  className="w-[72px] h-[72px] rounded-full flex-shrink-0 mb-1 ring-1 ring-white/10"
                  style={{ backgroundColor: store.activeCharacter?.avatar_color ?? "#c4a882" }}
                />
              )}
              <div className="bg-secondary rounded-2xl rounded-bl-md px-4 py-3 flex items-center gap-1">
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:0ms]" />
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:150ms]" />
                <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:300ms]" />
              </div>
            </div>
          )}
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
            <label className="flex items-center gap-1.5 cursor-pointer select-none" title="When on, the character responds automatically after each message">
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
              <div className="relative group/talk">
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-primary/70 hover:text-primary hover:bg-primary/10 h-9 w-9 rounded-lg"
                  onClick={() => store.promptCharacter()}
                  disabled={isSending || !store.apiKey || store.messages.length === 0}
                >
                  <MessageSquare size={15} />
                </Button>
                <span className="absolute bottom-full left-1/2 -translate-x-1/2 -mb-0.5 px-2.5 py-1 text-[11px] font-medium text-white bg-black rounded-lg shadow-lg whitespace-nowrap opacity-0 group-hover/talk:opacity-100 pointer-events-none transition-opacity duration-150">
                  Talk to Me
                </span>
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
            placeholder={`Talk to ${store.activeCharacter?.display_name ?? "character"}...`}
            className="flex-1 self-stretch max-h-[200px] resize-none rounded-xl border border-input bg-transparent px-4 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]"
            rows={1}
            disabled={isSending || (store.autoRespond && !store.apiKey)}
          />
          <Button
            size="icon"
            className="rounded-xl self-stretch w-10 flex-shrink-0"
            onClick={handleSend}
            disabled={!hasInput || isSending || (store.autoRespond && !store.apiKey)}
          >
            {isSending ? <Loader2 size={16} className="animate-spin" /> : <Send size={16} />}
          </Button>
        </div>
      </div>

      <PortraitModal
        characterId={showPortraitModal ? store.activeCharacter?.character_id ?? null : null}
        characterName={store.activeCharacter?.display_name}
        onClose={() => setShowPortraitModal(false)}
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
        characterName={store.activeCharacter?.display_name}
        isUserMessage={store.messages.find((m) => m.message_id === resetConfirmId)?.role === "user" || false}
      />

      <NarrationSettingsModal
        open={showNarrationSettings}
        onClose={() => { setShowNarrationSettings(false); setNarrationDirty(false); }}
        charId={charId}
        narrationTone={narrationTone}
        setNarrationTone={setNarrationTone}
        narrationInstructions={narrationInstructions}
        setNarrationInstructions={setNarrationInstructions}
        responseLength={responseLength}
        setResponseLength={setResponseLength}
        narrationDirty={narrationDirty}
        setNarrationDirty={setNarrationDirty}
        onSave={async () => {
          if (!charId) return;
          await Promise.all([
            api.setSetting(`narration_tone.${charId}`, narrationTone),
            api.setSetting(`narration_instructions.${charId}`, narrationInstructions),
            api.setSetting(`response_length.${charId}`, responseLength),
          ]);
          setNarrationDirty(false);
          setShowNarrationSettings(false);
        }}
        onClearHistory={store.activeCharacter ? () => {
          store.clearChatHistory(store.activeCharacter!.character_id);
          setShowNarrationSettings(false);
        } : undefined}
      />

      <SummaryModal
        open={showSummary}
        onClose={() => setShowSummary(false)}
        title={`Summary: ${store.activeCharacter?.display_name ?? "Chat"}`}
        generateSummary={() => api.generateChatSummary(store.apiKey, store.activeCharacter?.character_id ?? "")}
        characters={store.characters}
        groupChats={store.groupChats}
        activePortraits={store.activePortraits}
        currentCharacterId={store.activeCharacter?.character_id}
      />

      <AdjustMessageModal
        open={!!adjustMessageId}
        onClose={() => setAdjustMessageId(null)}
        onAdjust={(instructions) => {
          if (adjustMessageId) store.adjustMessage(adjustMessageId, instructions);
        }}
        characterName={store.activeCharacter?.display_name}
      />

      <NarrativePickerModal
        open={showNarrativePicker}
        onClose={() => setShowNarrativePicker(false)}
        onGenerate={(instructions) => store.generateNarrative(instructions)}
      />

      <IllustrationPickerModal
        open={showIllustrationPicker}
        onClose={() => setShowIllustrationPicker(false)}
        onGenerate={(tier) => {
          const prevIllus = store.messages.filter((m) => m.role === "illustration");
          const lastIllus = prevIllus[prevIllus.length - 1];
          const prevId = usePreviousScene && lastIllus ? lastIllus.message_id : undefined;
          setShowIllustrationPicker(false);
          store.generateIllustration(tier, illustrationInstructions.trim() || undefined, prevId, includeSceneSummary);
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
        backgroundPortraits={charPortrait?.data_url ? [charPortrait.data_url] : []}
        playVideo={playVideo}
        playingVideo={playingVideo}
        setPlayingVideo={setPlayingVideo}
        loopVideo={loopVideo}
        setLoopVideo={setLoopVideo}
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

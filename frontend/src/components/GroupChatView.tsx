import { useRef, useEffect, useState, useCallback } from "react";
import Markdown from "react-markdown";
import { formatMessage } from "@/components/chat/formatMessage";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog } from "@/components/ui/dialog";
import { Send, Loader2, X, Check, ExternalLink, BookOpen, MessageSquare, Settings, Image, Trash2, RefreshCw, SlidersHorizontal, Video, Repeat, Square, Download, Crosshair, ChevronLeft, ChevronRight } from "lucide-react";
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
import { GroupTalkPickerModal } from "@/components/chat/GroupTalkPickerModal";



interface Props {
  store: ReturnType<typeof useAppStore>;
}


export function GroupChatView({ store }: Props) {
  const [input, setInput] = useState("");
  const [showUserAvatarModal, setShowUserAvatarModal] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const groupCharIds: string[] = store.activeGroupChat
    ? (Array.isArray(store.activeGroupChat.character_ids) ? store.activeGroupChat.character_ids : [])
    : [];
  const groupCharacters = groupCharIds.map((id) => store.characters.find((c) => c.character_id === id)).filter(Boolean) as typeof store.characters;
  const [showGroupTalkPicker, setShowGroupTalkPicker] = useState(false);
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
  const [videoTab, setVideoTab] = useState<"generate" | "upload">("generate");
  const [uploadingVideo, setUploadingVideo] = useState(false);
  const [downloadedId, setDownloadedId] = useState<string | null>(null);
  const [removeVideoConfirmId, setRemoveVideoConfirmId] = useState<string | null>(null);
  const [animationReadyId, setAnimationReadyId] = useState<string | null>(null);
  const [illustrationModalId, setIllustrationModalId] = useState<string | null>(null);
  const [modalSelectedId, setModalSelectedId] = useState<string | null>(null);
  const [modalPlayingVideo, setModalPlayingVideo] = useState(false);
  const [modalImageLoading, setModalImageLoading] = useState(false);
  const [modalIllustrations, setModalIllustrations] = useState<Array<{ id: string; content: string }>>([]);
  const [showIllustrationPicker, setShowIllustrationPicker] = useState(false);
  const [illustrationInstructions, setIllustrationInstructions] = useState("");
  const [usePreviousScene, setUsePreviousScene] = useState(false);
  const [includeSceneSummary, setIncludeSceneSummary] = useState(true);
  const [narrationTone, setNarrationTone] = useState("Auto");
  const [narrationInstructions, setNarrationInstructions] = useState("");
  const [responseLength, setResponseLength] = useState("Auto");
  const [narrationDirty, setNarrationDirty] = useState(false);

  const charId = groupCharIds[0] ?? undefined;
  const chatId = store.activeGroupChat?.group_chat_id;

  useEffect(() => {
    if (!store.activeWorld) { setUserAvatarUrl(""); return; }
    api.getUserAvatar(store.activeWorld.world_id).then((url) => setUserAvatarUrl(url || ""));
  }, [store.activeWorld?.world_id, store.userProfile?.avatar_file]);

  useEffect(() => {
    if (!charId) return;
    Promise.all([
      api.getSetting(`narration_tone.${charId}`),
      api.getSetting(`narration_instructions.${charId}`),
      api.getSetting(`response_length.${charId}`),
    ]).then(([tone, instructions, length]) => {
      setNarrationTone(tone || "Auto");
      setNarrationInstructions(instructions || "");
      setResponseLength(length || "Auto");
      setNarrationDirty(false);
    });
  }, [charId]);

  // Derived: is this group chat currently loading?
  const isSending = store.sending === chatId;
  const isGeneratingNarrative = store.generatingNarrative === charId;
  const isGeneratingIllustration = store.generatingIllustration === charId;
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

  // Scroll to bottom when narrative/illustration generation starts
  useEffect(() => {
    if (isGeneratingNarrative || isGeneratingIllustration) {
      const el = scrollRef.current;
      if (el) el.scrollTop = el.scrollHeight;
    }
  }, [isGeneratingNarrative, isGeneratingIllustration]);

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
        <label className="ml-auto flex-shrink-0 flex items-center gap-1.5 cursor-pointer select-none" title="When on, the character responds automatically after each message">
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
        <button
          onClick={() => setShowNarrationSettings(true)}
          className={`flex-shrink-0 h-8 rounded-lg flex items-center gap-1.5 px-2.5 text-xs font-medium transition-colors cursor-pointer ${
            (narrationTone !== "Auto" || narrationInstructions) ? "text-amber-500 hover:text-amber-400 hover:bg-amber-500/10" : "text-muted-foreground hover:text-foreground hover:bg-accent"
          }`}
          title="Narration settings"
        >
          <Settings size={14} />
          <span>Narration</span>
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
          {store.messages.map((msg) => {
            const isUser = msg.role === "user";
            const isNarrative = msg.role === "narrative";
            const isPending = msg.message_id.startsWith("pending-");

            if (isNarrative) {
              return (
                <NarrativeMessage
                  key={msg.message_id}
                  message={msg}
                  isPending={isPending}
                  onResetToHere={(id) => setResetConfirmId(id)}
                />
              );
            }

            if (msg.role === "illustration") {
              return (
                <div key={msg.message_id} data-message-id={msg.message_id} className="flex justify-center my-3">
                  <div className="relative group max-w-[95%] rounded-xl bg-gradient-to-br from-emerald-950/30 to-emerald-900/10 border border-emerald-700/20 backdrop-blur-sm">
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
                            className="absolute bottom-4 right-4 w-10 h-10 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm opacity-0 group-hover:opacity-100"
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
                        <div className="absolute top-4 right-4 flex gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
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
                              onClick={() => store.regenerateIllustration(msg.message_id)}
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
                                  url: `index.html?illustration=${msg.message_id}&group=${store.activeGroupChat!.group_chat_id}`,
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
                          className="opacity-0 group-hover:opacity-100 transition-opacity text-emerald-500/40 hover:text-emerald-400 cursor-pointer"
                        >
                          Reset to Here
                        </button>
                      )}
                    </p>
                  </div>
                </div>
              );
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
              <div key={msg.message_id}>
                <div className={`flex items-end gap-2 ${isUser ? "justify-end" : "justify-start"}`}>
                  {!isUser && (
                    senderPortrait?.data_url ? (
                      <img src={senderPortrait.data_url} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border flex-shrink-0 mb-1" />
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
                    {!isUser && senderChar && (
                      <p className="text-[10px] font-semibold text-muted-foreground/70 mb-1">{senderChar.display_name}</p>
                    )}

                    <div className={`prose prose-sm max-w-none prose-p:my-1 prose-ul:my-1 prose-ol:my-1 prose-li:my-0.5 prose-headings:my-2 prose-pre:my-2 prose-blockquote:my-2 prose-hr:my-2 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_em]:italic [&_em]:block [&_em]:border-l-2 [&_em]:border-current/20 [&_em]:pl-3 [&_em]:my-1.5 [&_em]:opacity-80 ${
                      isUser
                        ? "[--tw-prose-body:var(--color-primary-foreground)] [--tw-prose-headings:var(--color-primary-foreground)] [--tw-prose-bold:var(--color-primary-foreground)] [--tw-prose-bullets:var(--color-primary-foreground)] [--tw-prose-counters:var(--color-primary-foreground)] [--tw-prose-code:var(--color-primary-foreground)] [--tw-prose-links:var(--color-primary-foreground)] [--tw-prose-quotes:var(--color-primary-foreground)] [--tw-prose-quote-borders:rgba(255,255,255,0.3)]"
                        : "[--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-headings:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)] [--tw-prose-bullets:var(--color-secondary-foreground)] [--tw-prose-counters:var(--color-secondary-foreground)] [--tw-prose-code:var(--color-secondary-foreground)] [--tw-prose-links:var(--color-primary)] [--tw-prose-quotes:var(--color-secondary-foreground)] [--tw-prose-quote-borders:var(--color-border)]"
                    }`}>
                      <Markdown>{formatMessage(msg.content)}</Markdown>
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
                  </div>
                  {isUser && userAvatarUrl && (
                    <button onClick={() => setShowUserAvatarModal(true)} className="cursor-pointer flex-shrink-0 mb-1">
                      <img src={userAvatarUrl} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
                    </button>
                  )}
                </div>
              </div>
            );
          })}
          {isSending && !isGeneratingNarrative && !isGeneratingIllustration && !isGeneratingVideo && (
            <div className="flex items-end gap-2 justify-start">
              <div className="flex -space-x-4 flex-shrink-0 mb-1">
                {groupCharacters.map((ch, i) => {
                  const p = store.activePortraits[ch.character_id];
                  return p?.data_url ? (
                    <img key={ch.character_id} src={p.data_url} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-background" style={{ zIndex: groupCharacters.length - i }} />
                  ) : (
                    <span key={ch.character_id} className="w-[72px] h-[72px] rounded-full ring-2 ring-background" style={{ backgroundColor: ch.avatar_color, zIndex: groupCharacters.length - i }} />
                  );
                })}
              </div>
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
        <div className="flex gap-2 max-w-2xl mx-auto items-end">
          <div className="relative group/talk flex-shrink-0">
            <Button
              variant="ghost"
              size="icon"
              className="text-primary/70 hover:text-primary hover:bg-primary/10 h-10 w-10 rounded-xl"
              onClick={() => setShowGroupTalkPicker(true)}
              disabled={isSending || !store.apiKey || store.messages.length === 0}
            >
              <MessageSquare size={16} />
            </Button>
            <span className="absolute bottom-full left-1/2 -translate-x-1/2 -mb-0.5 px-2.5 py-1 text-[11px] font-medium text-white bg-black rounded-lg shadow-lg whitespace-nowrap opacity-0 group-hover/talk:opacity-100 pointer-events-none transition-opacity duration-150">
              Talk to Me
            </span>
          </div>
          <div className="relative group/narr flex-shrink-0">
            <Button
              variant="ghost"
              size="icon"
              className="text-amber-500/70 hover:text-amber-400 hover:bg-amber-500/10 h-10 w-10 rounded-xl"
              onClick={() => store.generateNarrative()}
              disabled={isSending || !store.apiKey || store.messages.length === 0}
            >
              <BookOpen size={16} />
            </Button>
            <span className="absolute bottom-full left-1/2 -translate-x-1/2 -mb-0.5 px-2.5 py-1 text-[11px] font-medium text-white bg-black rounded-lg shadow-lg whitespace-nowrap opacity-0 group-hover/narr:opacity-100 pointer-events-none transition-opacity duration-150">
              + Narrative
            </span>
          </div>
          <div className="relative group/illus flex-shrink-0">
            <Button
              variant="ghost"
              size="icon"
              className="text-emerald-500/70 hover:text-emerald-400 hover:bg-emerald-500/10 h-10 w-10 rounded-xl"
              onClick={() => setShowIllustrationPicker(true)}
              disabled={isSending || !store.apiKey || store.messages.length === 0}
            >
              <Image size={16} />
            </Button>
            <span className="absolute bottom-full left-1/2 -translate-x-1/2 -mb-0.5 px-2.5 py-1 text-[11px] font-medium text-white bg-black rounded-lg shadow-lg whitespace-nowrap opacity-0 group-hover/illus:opacity-100 pointer-events-none transition-opacity duration-150">
              Illustration
            </span>
          </div>
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              e.target.style.height = "auto";
              e.target.style.height = Math.min(e.target.scrollHeight, 200) + "px";
              // Keep chat scrolled to bottom as textarea grows
              requestAnimationFrame(() => {
                const el = scrollRef.current;
                if (el) el.scrollTop = el.scrollHeight;
              });
            }}
            onKeyDown={handleKeyDown}
            placeholder={`Talk to ${store.activeGroupChat?.display_name ?? "the group"}...`}
            className="flex-1 min-h-[40px] max-h-[200px] resize-none rounded-xl border border-input bg-transparent px-4 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]"
            rows={1}
            disabled={isSending || (store.autoRespond && !store.apiKey)}
          />
          <Button
            size="icon"
            className="rounded-xl h-10 w-10 flex-shrink-0"
            onClick={handleSend}
            disabled={!input.trim() || isSending || (store.autoRespond && !store.apiKey)}
          >
            {isSending ? <Loader2 size={16} className="animate-spin" /> : <Send size={16} />}
          </Button>
        </div>
      </div>

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
        onClearHistory={store.activeGroupChat ? () => {
          store.clearGroupChatHistory(store.activeGroupChat!.group_chat_id);
          setShowNarrationSettings(false);
        } : undefined}
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
          setIncludeSceneSummary(true);
        }}
        illustrationInstructions={illustrationInstructions}
        setIllustrationInstructions={setIllustrationInstructions}
        usePreviousScene={usePreviousScene}
        setUsePreviousScene={setUsePreviousScene}
        includeSceneSummary={includeSceneSummary}
        setIncludeSceneSummary={setIncludeSceneSummary}
        hasPreviousIllustration={store.messages.some((m) => m.role === "illustration")}
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
            store.generateVideo(videoModalId, videoPrompt.trim() || undefined, videoDuration, videoStyle);
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
        uploadingVideo={uploadingVideo}
      />

      {illustrationModalId && (() => {
        const selId = modalSelectedId ?? illustrationModalId;
        // Use modalIllustrations if loaded, fall back to current messages
        const allIllustrations = modalIllustrations.length > 0
          ? modalIllustrations
          : store.messages.filter((m) => m.role === "illustration").map((m) => ({ id: m.message_id, content: m.content }));
        const selectedItem = allIllustrations.find((i) => i.id === selId);
        if (!selectedItem) return null;
        const modalVideoFile = videoFiles[selId];
        const modalVideoUrl = videoDataUrls[selId];
        return (
          <Dialog open onClose={() => { setIllustrationModalId(null); setModalPlayingVideo(false); }} className="max-w-[90vw]">
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
                    loop
                    playsInline
                    className="max-w-full max-h-[75vh] object-contain rounded-t-2xl"
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
                  onClick={() => { setIllustrationModalId(null); setModalPlayingVideo(false); }}
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
                        // All messages are already loaded — just scroll to the illustration
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
                </div>
                {modalVideoFile && !modalPlayingVideo && (
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
                {modalPlayingVideo && (
                  <button
                    onClick={() => setModalPlayingVideo(false)}
                    className="absolute bottom-4 right-4 z-20 w-12 h-12 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm"
                  >
                    <Square size={16} fill="white" />
                  </button>
                )}
                {allIllustrations.length > 1 && (<>
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
              </div>
              {allIllustrations.length > 1 && (
                <div className="flex-shrink-0 bg-card/80 backdrop-blur-sm rounded-b-2xl px-3 py-2 border-t border-border/30">
                  <div className="flex gap-1.5 overflow-x-auto scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]">
                    {allIllustrations.map((illus) => (
                      <button
                        key={illus.id}
                        onClick={() => {
                          setModalSelectedId(illus.id);
                          setModalImageLoading(true);
                          setModalPlayingVideo(false);
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

      {/* Group Talk to Me picker */}
      <GroupTalkPickerModal
        open={showGroupTalkPicker}
        onClose={() => setShowGroupTalkPicker(false)}
        characters={groupCharacters}
        portraits={store.activePortraits}
        onSelect={(characterId) => {
          store.promptGroupCharacter(characterId);
          setShowGroupTalkPicker(false);
        }}
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

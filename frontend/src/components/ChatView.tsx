import { useRef, useEffect, useState, useCallback } from "react";
import Markdown from "react-markdown";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Send, Loader2, SmilePlus, X, Check, Copy, ExternalLink, BookOpen, RotateCcw, MessageSquare, Settings } from "lucide-react";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type Reaction } from "@/lib/tauri";

const QUICK_EMOJIS = ["❤️", "😂", "😮", "😢", "🔥", "👍", "👎", "💀", "🙏", "✨", "👀", "💯"];



interface Props {
  store: ReturnType<typeof useAppStore>;
}

function ReactionBubbles({
  reactions,
  isUser,
}: {
  reactions: Reaction[];
  isUser: boolean;
}) {
  if (reactions.length === 0) return null;

  const grouped: Record<string, { emoji: string; reactors: string[] }> = {};
  for (const r of reactions) {
    if (!grouped[r.emoji]) grouped[r.emoji] = { emoji: r.emoji, reactors: [] };
    grouped[r.emoji].reactors.push(r.reactor);
  }

  return (
    <div className={`flex gap-1 mt-0.5 ${isUser ? "justify-end" : "justify-start"}`}>
      {Object.values(grouped).map(({ emoji, reactors }) => (
        <span
          key={emoji}
          className="inline-flex items-center gap-0.5 text-xs bg-secondary/80 border border-border rounded-full px-1.5 py-0.5 backdrop-blur-sm"
          title={reactors.map((r) => (r === "user" ? "You" : "Character")).join(", ")}
        >
          <span className="text-sm leading-none">{emoji}</span>
          {reactors.length > 1 && (
            <span className="text-[10px] text-muted-foreground">{reactors.length}</span>
          )}
        </span>
      ))}
    </div>
  );
}

function EmojiPicker({
  onSelect,
  onClose,
}: {
  onSelect: (emoji: string) => void;
  onClose: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [onClose]);

  return (
    <div
      ref={ref}
      className="absolute z-50 bg-card border border-border rounded-xl shadow-xl p-2 grid grid-cols-6 gap-1 w-[200px] animate-in fade-in zoom-in-95 duration-150"
    >
      {QUICK_EMOJIS.map((emoji) => (
        <button
          key={emoji}
          onClick={() => {
            onSelect(emoji);
            onClose();
          }}
          className="w-8 h-8 flex items-center justify-center text-lg rounded-lg hover:bg-accent transition-colors cursor-pointer"
        >
          {emoji}
        </button>
      ))}
    </div>
  );
}

export function ChatView({ store }: Props) {
  const [input, setInput] = useState("");
  const [pickerMessageId, setPickerMessageId] = useState<string | null>(null);
  const [showPortraitModal, setShowPortraitModal] = useState(false);
  const [showUserAvatarModal, setShowUserAvatarModal] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const charPortrait = store.activeCharacter ? store.activePortraits[store.activeCharacter.character_id] : undefined;
  const [userAvatarUrl, setUserAvatarUrl] = useState("");
  const [copiedError, setCopiedError] = useState(false);
  const [resetConfirmId, setResetConfirmId] = useState<string | null>(null);
  const [showIdentityPopover, setShowIdentityPopover] = useState(false);
  const [showNarrationSettings, setShowNarrationSettings] = useState(false);
  const [narrationTone, setNarrationTone] = useState("Auto");
  const [narrationInstructions, setNarrationInstructions] = useState("");
  const [responseLength, setResponseLength] = useState("Auto");
  const [narrationDirty, setNarrationDirty] = useState(false);

  const charId = store.activeCharacter?.character_id;

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

  const prevScrollHeightRef = useRef(0);
  const isLoadingOlderRef = useRef(false);
  const messageCountRef = useRef(0);

  // Scroll to bottom only when new messages arrive at the end (not when older ones are prepended)
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const prevCount = messageCountRef.current;
    const newCount = store.messages.length;
    messageCountRef.current = newCount;

    if (isLoadingOlderRef.current) {
      // Older messages were prepended — restore scroll position
      const addedHeight = el.scrollHeight - prevScrollHeightRef.current;
      el.scrollTop = addedHeight;
      isLoadingOlderRef.current = false;
    } else {
      // New messages at the end — scroll to bottom
      el.scrollTop = el.scrollHeight;
    }
  }, [store.messages, store.sending]);

  // Auto-focus input after AI response arrives
  useEffect(() => {
    if (!store.sending) {
      inputRef.current?.focus();
    }
  }, [store.sending]);

  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el || store.loadingOlder || !store.hasMoreMessages) return;
    if (el.scrollTop < 80) {
      isLoadingOlderRef.current = true;
      prevScrollHeightRef.current = el.scrollHeight;
      store.loadOlderMessages();
    }
  }, [store.loadingOlder, store.hasMoreMessages, store.loadOlderMessages]);

  const handleSend = async () => {
    const text = input.trim();
    if (!text || store.sending) return;
    store.clearChatError();
    setInput("");
    if (inputRef.current) inputRef.current.style.height = "auto";
    await store.sendMessage(text);
    inputRef.current?.focus();
  };

  const handleRetry = async () => {
    if (!store.lastFailedContent || store.sending) return;
    const content = store.lastFailedContent;
    store.clearChatError();
    await store.sendMessage(content);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  if (!store.activeCharacter) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <p>Select or create a character to start chatting</p>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col min-h-0 relative">
      {charPortrait?.data_url && (
        <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden">
          <img
            src={charPortrait.data_url}
            alt=""
            className="w-full h-full object-cover"
          />
          <div className="absolute inset-0 bg-background/60" />
        </div>
      )}
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
        ) : (
          <span
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: store.activeCharacter.avatar_color }}
          />
        )}
        <h1 className="font-semibold">{store.activeCharacter.display_name}</h1>
        {store.activeCharacter.identity && (
          <div className="relative flex-1 min-w-0">
            <span
              className="text-xs text-muted-foreground truncate block cursor-default"
              onMouseEnter={() => setShowIdentityPopover(true)}
              onMouseLeave={() => setShowIdentityPopover(false)}
            >
              {store.activeCharacter.identity.slice(0, 60)}...
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
                <p className="font-semibold text-sm mb-1">{store.activeCharacter.display_name}</p>
                <p className="text-xs text-muted-foreground leading-relaxed whitespace-pre-wrap">{store.activeCharacter.identity}</p>
              </div>
            )}
          </div>
        )}
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
        <ScrollArea ref={scrollRef} className="h-full px-4 py-3" onScroll={handleScroll}>
        <div>
        {store.loadingOlder && (
          <div className="flex justify-center py-3">
            <Loader2 size={18} className="animate-spin text-muted-foreground" />
          </div>
        )}
        {store.hasMoreMessages && !store.loadingOlder && store.messages.length > 0 && (
          <div className="flex justify-center py-2">
            <button
              onClick={() => {
                isLoadingOlderRef.current = true;
                prevScrollHeightRef.current = scrollRef.current?.scrollHeight ?? 0;
                store.loadOlderMessages();
              }}
              className="text-xs text-muted-foreground/60 hover:text-muted-foreground transition-colors cursor-pointer"
            >
              Load older messages
            </button>
          </div>
        )}
        {store.messages.length === 0 && (
          <div className="text-center text-muted-foreground py-12">
            <p className="text-lg mb-1">Start a conversation</p>
            <p className="text-sm">
              Send a message to {store.activeCharacter.display_name}
            </p>
          </div>
        )}
        <div className="space-y-3 max-w-2xl mx-auto">
          {store.messages.map((msg) => {
            const isUser = msg.role === "user";
            const isNarrative = msg.role === "narrative";
            const isPending = msg.message_id.startsWith("pending-");
            const reactions = store.reactions[msg.message_id] ?? [];
            const showPicker = pickerMessageId === msg.message_id;

            if (isNarrative) {
              return (
                <div key={msg.message_id} className="flex justify-center my-2">
                  <div className="relative group max-w-[90%] rounded-xl px-5 py-3.5 text-sm leading-relaxed bg-gradient-to-br from-amber-950/40 to-amber-900/20 border border-amber-700/30 text-amber-100/90 italic backdrop-blur-sm">
                    <div className="flex items-center gap-1.5 mb-1.5 text-[10px] uppercase tracking-wider text-amber-500/70 font-semibold not-italic">
                      <BookOpen size={12} />
                      <span>Narrative</span>
                    </div>
                    <div className="prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:var(--color-amber-100)] [--tw-prose-bold:rgb(252,211,77)]">
                      <Markdown>{msg.content}</Markdown>
                    </div>
                    <p className="text-[10px] mt-1.5 text-amber-500/50 not-italic flex items-center gap-2">
                      {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                      {!isPending && (
                        <button
                          onClick={() => setResetConfirmId(msg.message_id)}
                          className="opacity-0 group-hover:opacity-100 transition-opacity text-amber-500/40 hover:text-amber-400 cursor-pointer"
                        >
                          Reset to Here
                        </button>
                      )}
                    </p>
                  </div>
                </div>
              );
            }

            return (
              <div key={msg.message_id}>
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
                        : "bg-secondary text-secondary-foreground rounded-bl-md max-w-[80%]"
                    }`}
                  >
                    {/* Reaction button — overlaps the top corner of the bubble */}
                    {!isPending && (
                      <button
                        onClick={() => setPickerMessageId(showPicker ? null : msg.message_id)}
                        className={`absolute -top-2.5 z-10 w-7 h-7 flex items-center justify-center rounded-full bg-white shadow-md border border-border/50 text-muted-foreground hover:text-foreground hover:scale-110 opacity-0 group-hover:opacity-100 transition-all cursor-pointer ${
                          isUser ? "-left-2.5" : "-right-2.5"
                        }`}
                      >
                        <SmilePlus size={16} strokeWidth={2} />
                      </button>
                    )}

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
                      <Markdown>{msg.content}</Markdown>
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

                <div className={!isUser ? "pl-20" : userAvatarUrl ? "pr-20" : ""}>
                  <ReactionBubbles reactions={reactions} isUser={isUser} />
                </div>
              </div>
            );
          })}
          {store.sending && !store.generatingNarrative && (
            <div className="flex items-end gap-2 justify-start">
              {charPortrait?.data_url ? (
                <img src={charPortrait.data_url} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border flex-shrink-0 mb-1" />
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
          {store.generatingNarrative && (
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
        </div>
        </div>
      </ScrollArea>
      </div>

      {store.chatError && (
        <div className="px-4 py-2.5 bg-background border-t border-destructive/30 flex items-center gap-3 relative z-10">
          <div className="flex-1 min-w-0">
            <p className="text-xs text-destructive font-medium truncate">{store.chatError}</p>
          </div>
          <button
            onClick={() => {
              navigator.clipboard.writeText(store.chatError!);
              setCopiedError(true);
              setTimeout(() => setCopiedError(false), 2000);
            }}
            className="flex-shrink-0 text-destructive/60 hover:text-destructive transition-colors cursor-pointer"
            title="Copy full error"
          >
            {copiedError ? <Check size={14} /> : <Copy size={14} />}
          </button>
          {store.lastFailedContent && (
            <Button
              size="sm"
              variant="outline"
              className="flex-shrink-0 border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
              onClick={handleRetry}
              disabled={store.sending}
            >
              Try Again
            </Button>
          )}
          <button
            onClick={() => store.clearChatError()}
            className="flex-shrink-0 text-destructive/60 hover:text-destructive transition-colors cursor-pointer"
          >
            <X size={14} />
          </button>
        </div>
      )}

      <div className="px-4 py-3 border-t border-border relative z-10 bg-background">
        <div className="flex gap-2 max-w-2xl mx-auto items-end">
          <div className="relative group/talk flex-shrink-0">
            <Button
              variant="ghost"
              size="icon"
              className="text-primary/70 hover:text-primary hover:bg-primary/10 h-10 w-10 rounded-xl"
              onClick={() => store.promptCharacter()}
              disabled={store.sending || !store.apiKey || store.messages.length === 0}
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
              disabled={store.sending || !store.apiKey || store.messages.length === 0}
            >
              <BookOpen size={16} />
            </Button>
            <span className="absolute bottom-full left-1/2 -translate-x-1/2 -mb-0.5 px-2.5 py-1 text-[11px] font-medium text-white bg-black rounded-lg shadow-lg whitespace-nowrap opacity-0 group-hover/narr:opacity-100 pointer-events-none transition-opacity duration-150">
              + Narrative
            </span>
          </div>
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              e.target.style.height = "auto";
              e.target.style.height = Math.min(e.target.scrollHeight, 200) + "px";
            }}
            onKeyDown={handleKeyDown}
            placeholder={`Message ${store.activeCharacter.display_name}...`}
            className="flex-1 min-h-[40px] max-h-[200px] resize-none rounded-xl border border-input bg-transparent px-4 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]"
            rows={1}
            disabled={store.sending || (store.autoRespond && !store.apiKey)}
          />
          <Button
            size="icon"
            className="rounded-xl h-10 w-10 flex-shrink-0"
            onClick={handleSend}
            disabled={!input.trim() || store.sending || (store.autoRespond && !store.apiKey)}
          >
            {store.sending ? <Loader2 size={16} className="animate-spin" /> : <Send size={16} />}
          </Button>
        </div>
      </div>

      {charPortrait?.data_url && (
        <Dialog open={showPortraitModal} onClose={() => setShowPortraitModal(false)} className="max-w-md">
          <div className="relative">
            <img
              src={charPortrait.data_url}
              alt={store.activeCharacter.display_name}
              className="w-full rounded-2xl shadow-2xl shadow-black/50"
            />
            <button
              onClick={() => setShowPortraitModal(false)}
              className="absolute top-3 right-3 w-8 h-8 flex items-center justify-center rounded-full bg-black/50 text-white hover:bg-black/70 transition-colors cursor-pointer backdrop-blur-sm"
            >
              <X size={16} />
            </button>
            <div className="absolute inset-x-0 bottom-0 rounded-b-2xl bg-gradient-to-t from-black/70 to-transparent px-5 pb-4 pt-10">
              <p className="text-white font-semibold text-lg">{store.activeCharacter.display_name}</p>
            </div>
          </div>
        </Dialog>
      )}

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

      <Dialog open={!!resetConfirmId} onClose={() => setResetConfirmId(null)} className="max-w-sm">
        <div className="p-5 space-y-4">
          <div className="flex items-center gap-2">
            <RotateCcw size={18} className="text-destructive" />
            <h3 className="font-semibold">Reset to Here</h3>
          </div>
          <p className="text-sm text-muted-foreground">
            This will permanently delete all messages after this point, including their associated memories and embeddings.
            {store.messages.find((m) => m.message_id === resetConfirmId)?.role === "user" && (
              <span className="block mt-1.5 text-foreground/80">A new response will be generated from {store.activeCharacter?.display_name}.</span>
            )}
          </p>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setResetConfirmId(null)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => {
                if (resetConfirmId) {
                  store.resetToMessage(resetConfirmId);
                  setResetConfirmId(null);
                }
              }}
            >
              Reset
            </Button>
          </div>
        </div>
      </Dialog>

      <Dialog open={showNarrationSettings} onClose={() => { setShowNarrationSettings(false); setNarrationDirty(false); }} className="max-w-md">
        <div className="p-5 space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <BookOpen size={18} className="text-amber-500" />
              <h3 className="font-semibold">Narration Settings</h3>
            </div>
            <button
              onClick={() => { setShowNarrationSettings(false); setNarrationDirty(false); }}
              className="w-8 h-8 flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
            >
              <X size={16} />
            </button>
          </div>

          <div className="space-y-3">
            <div>
              <label className="text-xs font-medium text-muted-foreground block mb-1.5">Tone</label>
              <select
                value={narrationTone}
                onChange={(e) => { setNarrationTone(e.target.value); setNarrationDirty(true); }}
                className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
              >
                {[
                  "Auto",
                  "Humorous", "Romantic", "Action & Adventure", "Dark & Gritty",
                  "Suspenseful", "Whimsical", "Melancholic", "Heroic",
                  "Horror", "Noir", "Surreal", "Cozy & Warm",
                  "Tense & Paranoid", "Poetic", "Cinematic",
                  "Mythic", "Playful", "Bittersweet", "Ethereal", "Gritty Realism",
                ].map((t) => (
                  <option key={t} value={t}>{t}</option>
                ))}
              </select>
            </div>

            <div>
              <label className="text-xs font-medium text-muted-foreground block mb-1.5">Response Length</label>
              <select
                value={responseLength}
                onChange={(e) => { setResponseLength(e.target.value); setNarrationDirty(true); }}
                className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
              >
                {["Auto", "Short", "Medium", "Long"].map((l) => (
                  <option key={l} value={l}>{l}</option>
                ))}
              </select>
              <p className="text-[10px] text-muted-foreground mt-1">
                {responseLength === "Auto" && "The character decides how much to say."}
                {responseLength === "Short" && "Brief replies, 2\u20133 sentences."}
                {responseLength === "Medium" && "Moderate replies, 4\u20136 sentences."}
                {responseLength === "Long" && "Detailed replies, 7+ sentences with rich detail."}
              </p>
            </div>

            <div>
              <label className="text-xs font-medium text-muted-foreground block mb-1.5">Custom Instructions</label>
              <textarea
                value={narrationInstructions}
                onChange={(e) => { setNarrationInstructions(e.target.value); setNarrationDirty(true); }}
                placeholder="e.g. Describe the weather shifting. Include background characters reacting. Let the scene move to a new location..."
                className="w-full min-h-[100px] max-h-[200px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                rows={4}
              />
            </div>
          </div>

          <div className="flex justify-end gap-2 pt-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => { setShowNarrationSettings(false); setNarrationDirty(false); }}
            >
              Cancel
            </Button>
            <Button
              size="sm"
              disabled={!narrationDirty}
              onClick={async () => {
                if (!charId) return;
                await Promise.all([
                  api.setSetting(`narration_tone.${charId}`, narrationTone),
                  api.setSetting(`narration_instructions.${charId}`, narrationInstructions),
                  api.setSetting(`response_length.${charId}`, responseLength),
                ]);
                setNarrationDirty(false);
                setShowNarrationSettings(false);
              }}
            >
              Save
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

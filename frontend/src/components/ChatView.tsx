import { useRef, useEffect, useState, useCallback } from "react";
import Markdown from "react-markdown";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Send, Loader2, SmilePlus, X, Paintbrush, Check } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type Reaction, type ChatBackground, type GalleryItem } from "@/lib/tauri";

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
  const [chatBg, setChatBg] = useState<ChatBackground | null>(null);
  const [showBgPicker, setShowBgPicker] = useState(false);
  const [showBgImagePicker, setShowBgImagePicker] = useState(false);
  const [galleryItems, setGalleryItems] = useState<GalleryItem[]>([]);
  const [bgImageUrl, setBgImageUrl] = useState<string>("");
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const bgPickerRef = useRef<HTMLDivElement>(null);
  const charPortrait = store.activeCharacter ? store.activePortraits[store.activeCharacter.character_id] : undefined;
  const [userAvatarUrl, setUserAvatarUrl] = useState("");

  useEffect(() => {
    if (!store.activeWorld) { setUserAvatarUrl(""); return; }
    api.getUserAvatar(store.activeWorld.world_id).then((url) => setUserAvatarUrl(url || ""));
  }, [store.activeWorld?.world_id, store.userProfile?.avatar_file]);

  const charId = store.activeCharacter?.character_id;
  const worldId = store.activeWorld?.world_id;

  useEffect(() => {
    if (!charId) return;
    api.getChatBackground(charId).then((bg) => setChatBg(bg));
  }, [charId]);

  useEffect(() => {
    if (!worldId) return;
    api.listWorldGallery(worldId).then(setGalleryItems).catch(() => {});
  }, [worldId, store.activeWorldImage?.image_id, store.activePortraits, store.userProfile?.avatar_file]);

  useEffect(() => {
    if (chatBg?.bg_type === "world_image" && chatBg.bg_image_id) {
      const cached = galleryItems.find((i) => i.id === chatBg.bg_image_id);
      if (cached?.data_url) {
        setBgImageUrl(cached.data_url);
      }
    } else if (chatBg?.bg_type === "world_image" && store.activeWorldImage?.data_url) {
      setBgImageUrl(store.activeWorldImage.data_url);
    } else {
      setBgImageUrl("");
    }
  }, [chatBg?.bg_type, chatBg?.bg_image_id, galleryItems, store.activeWorldImage]);

  const saveBg = useCallback((patch: Partial<ChatBackground>) => {
    if (!charId) return;
    const updated: ChatBackground = {
      character_id: charId,
      bg_type: patch.bg_type ?? chatBg?.bg_type ?? "color",
      bg_color: patch.bg_color ?? chatBg?.bg_color ?? "",
      bg_image_id: patch.bg_image_id ?? chatBg?.bg_image_id ?? "",
      bg_blur: patch.bg_blur ?? chatBg?.bg_blur ?? 0,
      updated_at: new Date().toISOString(),
    };
    setChatBg(updated);
    api.updateChatBackground(updated);
  }, [charId, chatBg]);

  const resetBg = useCallback(() => {
    if (!charId) return;
    const cleared: ChatBackground = {
      character_id: charId, bg_type: "color", bg_color: "", bg_image_id: "", bg_blur: 0, updated_at: new Date().toISOString(),
    };
    setChatBg(cleared);
    api.updateChatBackground(cleared);
  }, [charId]);

  useEffect(() => {
    if (!showBgPicker) return;
    const handler = (e: MouseEvent) => {
      if (bgPickerRef.current && !bgPickerRef.current.contains(e.target as Node)) {
        setShowBgPicker(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [showBgPicker]);

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

  const bgStyle = chatBg?.bg_type === "color" && chatBg.bg_color ? { backgroundColor: chatBg.bg_color } : undefined;

  const hasExplicitBg = chatBg && (
    (chatBg.bg_type === "color" && chatBg.bg_color) ||
    (chatBg.bg_type === "world_image" && (bgImageUrl || chatBg.bg_image_id))
  );
  const defaultAvatarBg = !hasExplicitBg ? charPortrait?.data_url : undefined;

  return (
    <div className="flex-1 flex flex-col min-h-0 relative" style={bgStyle}>
      {chatBg?.bg_type === "world_image" && bgImageUrl && (
        <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden">
          <img
            src={bgImageUrl}
            alt=""
            className="w-full h-full object-cover"
            style={{ filter: chatBg.bg_blur ? `blur(${chatBg.bg_blur}px)` : undefined, transform: chatBg.bg_blur ? "scale(1.1)" : undefined }}
          />
          <div className="absolute inset-0 bg-background/40" />
        </div>
      )}
      {defaultAvatarBg && (
        <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden">
          <img
            src={defaultAvatarBg}
            alt=""
            className="w-full h-full object-cover"
            style={{}}
          />
          <div className="absolute inset-0 bg-background/60" />
        </div>
      )}
      <div className="px-4 py-3 border-b border-border flex items-center gap-3 relative z-30 bg-background">
        {charPortrait?.data_url ? (
          <button onClick={() => setShowPortraitModal(true)} className="cursor-pointer flex-shrink-0">
            <img src={charPortrait.data_url} alt="" className="w-9 h-9 rounded-full object-cover ring-2 ring-border hover:ring-primary/50 transition-all" />
          </button>
        ) : (
          <span
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: store.activeCharacter.avatar_color }}
          />
        )}
        <h1 className="font-semibold">{store.activeCharacter.display_name}</h1>
        {store.activeCharacter.identity && (
          <span className="text-xs text-muted-foreground truncate flex-1">
            {store.activeCharacter.identity.slice(0, 60)}...
          </span>
        )}
        <div className="relative ml-auto flex-shrink-0">
          <button
            onClick={() => setShowBgPicker(!showBgPicker)}
            className="w-8 h-8 rounded-lg flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
            title="Chat background color"
          >
            <Paintbrush size={16} />
          </button>
          {showBgPicker && (
            <div ref={bgPickerRef} className="absolute right-0 top-full mt-2 z-50 w-56 bg-card border border-border rounded-xl shadow-xl p-3 animate-in fade-in zoom-in-95 duration-150">
              <p className="text-[10px] text-muted-foreground mb-2 font-semibold uppercase tracking-wider">Chat background</p>

              <div className="space-y-3">
                <div>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="bgtype"
                      checked={!chatBg || chatBg.bg_type === "color"}
                      onChange={() => saveBg({ bg_type: "color" })}
                      className="accent-primary"
                    />
                    <span className="text-xs">Solid color</span>
                  </label>
                  {(!chatBg || chatBg.bg_type === "color") && (
                    <div className="flex items-center gap-2 mt-1.5 ml-5">
                      <input
                        type="color"
                        value={chatBg?.bg_color || "#0a0a0f"}
                        onChange={(e) => saveBg({ bg_type: "color", bg_color: e.target.value })}
                        className="w-7 h-7 rounded-lg border border-input cursor-pointer bg-transparent p-0.5"
                      />
                      <span className="text-[10px] font-mono text-muted-foreground">{chatBg?.bg_color || "default"}</span>
                    </div>
                  )}
                </div>

                {galleryItems.length > 0 && (
                  <div>
                    <label className="flex items-center gap-2 cursor-pointer">
                      <input
                        type="radio"
                        name="bgtype"
                        checked={chatBg?.bg_type === "world_image"}
                        onChange={() => {
                          const first = galleryItems[0];
                          saveBg({ bg_type: "world_image", bg_image_id: first?.id ?? "", bg_blur: 0 });
                        }}
                        className="accent-primary"
                      />
                      <span className="text-xs">Gallery image</span>
                    </label>
                    {chatBg?.bg_type === "world_image" && (
                      <div className="mt-1.5 ml-5 space-y-2">
                        {bgImageUrl && (
                          <img src={bgImageUrl} alt="" className="w-full rounded-lg ring-1 ring-border" />
                        )}
                        <button
                          onClick={() => setShowBgImagePicker(true)}
                          className="text-[11px] text-primary hover:text-primary/80 transition-colors cursor-pointer"
                        >
                          Choose image ({galleryItems.length})
                        </button>
                        <div className="flex items-center gap-2">
                          <span className="text-[10px] text-muted-foreground w-8">Blur</span>
                          <input
                            type="range"
                            min={0}
                            max={20}
                            value={chatBg?.bg_blur ?? 0}
                            onChange={(e) => saveBg({ bg_blur: Number(e.target.value) })}
                            className="flex-1 accent-primary h-1"
                          />
                          <span className="text-[10px] font-mono text-muted-foreground w-5 text-right">{chatBg?.bg_blur ?? 0}</span>
                        </div>
                      </div>
                    )}
                  </div>
                )}

                <button
                  onClick={resetBg}
                  className="text-[11px] text-muted-foreground hover:text-foreground transition-colors cursor-pointer pt-1 border-t border-border/50 w-full text-left"
                >
                  Reset to default
                </button>
              </div>
            </div>
          )}
        </div>
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
            const isPending = msg.message_id.startsWith("pending-");
            const reactions = store.reactions[msg.message_id] ?? [];
            const showPicker = pickerMessageId === msg.message_id;

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

                    <div className={`prose prose-sm max-w-none prose-p:my-1 prose-ul:my-1 prose-ol:my-1 prose-li:my-0.5 prose-headings:my-2 prose-pre:my-2 prose-blockquote:my-2 prose-hr:my-2 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 ${
                      isUser
                        ? "[--tw-prose-body:var(--color-primary-foreground)] [--tw-prose-headings:var(--color-primary-foreground)] [--tw-prose-bold:var(--color-primary-foreground)] [--tw-prose-bullets:var(--color-primary-foreground)] [--tw-prose-counters:var(--color-primary-foreground)] [--tw-prose-code:var(--color-primary-foreground)] [--tw-prose-links:var(--color-primary-foreground)] [--tw-prose-quotes:var(--color-primary-foreground)] [--tw-prose-quote-borders:rgba(255,255,255,0.3)]"
                        : "[--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-headings:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)] [--tw-prose-bullets:var(--color-secondary-foreground)] [--tw-prose-counters:var(--color-secondary-foreground)] [--tw-prose-code:var(--color-secondary-foreground)] [--tw-prose-links:var(--color-primary)] [--tw-prose-quotes:var(--color-secondary-foreground)] [--tw-prose-quote-borders:var(--color-border)]"
                    }`}>
                      <Markdown>{msg.content}</Markdown>
                    </div>
                    <p className={`text-[10px] mt-1 ${
                      isUser ? "text-primary-foreground/50" : "text-muted-foreground"
                    }`}>
                      {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </p>
                  </div>
                  {isUser && userAvatarUrl && (
                    <img src={userAvatarUrl} alt="" className="w-[72px] h-[72px] rounded-full object-cover ring-2 ring-border flex-shrink-0 mb-1" />
                  )}
                </div>

                <div className={!isUser ? "pl-20" : userAvatarUrl ? "pr-20" : ""}>
                  <ReactionBubbles reactions={reactions} isUser={isUser} />
                </div>
              </div>
            );
          })}
          {store.sending && (
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
        </div>
        </div>
      </ScrollArea>
      </div>

      {store.chatError && (
        <div className="px-4 py-2.5 bg-background border-t border-destructive/30 flex items-center gap-3 relative z-10">
          <div className="flex-1 min-w-0">
            <p className="text-xs text-destructive font-medium truncate">{store.chatError}</p>
          </div>
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
            className="flex-1 min-h-[40px] max-h-[200px] resize-none rounded-xl border border-input bg-transparent px-4 py-2.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            rows={1}
            disabled={store.sending || !store.apiKey}
          />
          <Button
            size="icon"
            className="rounded-xl h-10 w-10 flex-shrink-0"
            onClick={handleSend}
            disabled={!input.trim() || store.sending || !store.apiKey}
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

      <Dialog open={showBgImagePicker} onClose={() => setShowBgImagePicker(false)} className="max-w-2xl">
        <div className="p-5 space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="font-semibold text-lg">Choose Background Image</h3>
            <button
              onClick={() => setShowBgImagePicker(false)}
              className="w-8 h-8 flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
            >
              <X size={16} />
            </button>
          </div>
          <div className="grid grid-cols-3 gap-3 max-h-[60vh] overflow-y-auto pr-1">
            {galleryItems.map((item) => (
              <button
                key={item.id}
                onClick={() => {
                  saveBg({ bg_type: "world_image", bg_image_id: item.id });
                  setShowBgImagePicker(false);
                }}
                className={`relative rounded-xl overflow-hidden ring-2 transition-all cursor-pointer ${
                  chatBg?.bg_image_id === item.id
                    ? "ring-primary shadow-lg"
                    : "ring-transparent hover:ring-border"
                }`}
              >
                {item.data_url && (
                  <img src={item.data_url} alt="" className={`w-full object-cover ${item.category === "character" || item.category === "user" ? "aspect-square" : "aspect-video"}`} />
                )}
                {chatBg?.bg_image_id === item.id && (
                  <div className="absolute top-2 right-2 w-6 h-6 rounded-full bg-primary flex items-center justify-center">
                    <Check size={14} className="text-primary-foreground" />
                  </div>
                )}
                <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/60 to-transparent px-3 pb-2 pt-6">
                  <p className="text-white/80 text-[10px] line-clamp-1">{item.label}</p>
                  <p className="text-white/50 text-[9px] mt-0.5">{new Date(item.created_at).toLocaleDateString()}</p>
                </div>
              </button>
            ))}
          </div>
          {galleryItems.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-8">
              No images yet. Generate or upload images in the Gallery.
            </p>
          )}
        </div>
      </Dialog>
    </div>
  );
}

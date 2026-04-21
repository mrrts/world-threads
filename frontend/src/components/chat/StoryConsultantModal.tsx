import { useState, useRef, useEffect, useCallback } from "react";
import Markdown from "react-markdown";
import { Dialog } from "@/components/ui/dialog";
import { X, Loader2, Send, Lightbulb, Sparkles, Trash2, ChevronDown, Pencil, Plus, PanelLeftClose, PanelLeftOpen, Download, BookOpen } from "lucide-react";
import { formatMessage, markdownComponents, remarkPlugins, rehypePlugins } from "./formatMessage";
import { listen } from "@tauri-apps/api/event";
import { api, type ConsultantChat } from "@/lib/tauri";
import { playChime } from "@/lib/chime";
import { Button } from "@/components/ui/button";
import { chatFontPx } from "@/lib/chat-font";

interface ConsultantMessage {
  // "import" is a marker role used when the user pulls in recent thread
  // messages as context — it's rendered as a collapsed bar, not a bubble.
  role: "user" | "assistant" | "import";
  content: string;
}

interface Props {
  open: boolean;
  onClose: () => void;
  apiKey: string;
  characterId: string | null;
  groupChatId: string | null;
  threadId: string;
  characterNames: string[];
  worldImageUrl?: string;
  /** Portrait URLs keyed by character_id */
  portraits: Record<string, string>;
  /** User avatar URL */
  userAvatarUrl: string;
  /** Whether to play a chime on first token */
  notifyOnMessage: boolean;
  /** Chat font size ("sm" | "md" | "lg" etc.) — shared with ChatView/GroupChatView */
  chatFontSize: string;
}

interface PromptCategory {
  name: string;
  prompts: Array<{ label: string; prompt: string }>;
}

function buildCategories(names: string[]): PromptCategory[] {
  return [
    {
      name: "What's Next",
      prompts: [
        { label: "What should I do?", prompt: "What should I do next? Give me a few options." },
        { label: "How should I respond to that?", prompt: "How should I respond to what just happened? Give me a few options." },
        { label: "What could I say here?", prompt: "Help me think of what to say. What would feel right given everything that's happened?" },
        { label: "Something feels like it's about to shift", prompt: "Something feels like it's about to shift. What do you think is coming?" },
        { label: "What haven't I thought of?", prompt: "What's an angle here I haven't considered?" },
      ],
    },
    {
      name: "The People",
      prompts: [
        ...names.map((n) => ({ label: `What's going on with ${n}?`, prompt: `What's going on with ${n} right now? What do you think they're feeling?` })),
        ...names.map((n) => ({ label: `What am I missing about ${n}?`, prompt: `Am I missing something about ${n}? About what they just said or did?` })),
        { label: "Where do things stand between us?", prompt: "Where do things stand between us right now? What's changed recently?" },
        ...names.map((n) => ({ label: `What might ${n} do next?`, prompt: `What do you think ${n} might do next? What would you expect from them here?` })),
      ],
    },
    {
      name: "Stepping Back",
      prompts: [
        { label: "What's really going on here?", prompt: "Step back with me — what's really going on here? What's the bigger picture?" },
        { label: "What's the subtext right now?", prompt: "What's the subtext of what just happened? What's going on beneath the surface?" },
        { label: "What moment mattered most recently?", prompt: "Pick a moment from recently and tell me why it mattered." },
        { label: "What should I be paying more attention to?", prompt: "Is there something I should be paying more attention to? Something I'm glossing over?" },
        { label: "What would you do in my position?", prompt: "Honestly — what would you do if you were me right now?" },
      ],
    },
    {
      name: "The Tension",
      prompts: [
        { label: "How can I push this further?", prompt: "How can I push this further? Raise the stakes a little?" },
        { label: "How can I ease things?", prompt: "How can I ease things? Bring some warmth or calm to this?" },
        { label: "Is there something unresolved?", prompt: "Is there something unresolved hanging in the air between us that I should address?" },
        { label: "What am I avoiding?", prompt: "Am I avoiding something? Something I should probably say or do but haven't?" },
        { label: "What should I be asking you?", prompt: "What question should I be asking you right now?" },
      ],
    },
  ];
}

export function StoryConsultantModal({ open, onClose, apiKey, characterId, groupChatId, threadId, characterNames, worldImageUrl, portraits, userAvatarUrl, notifyOnMessage, chatFontSize }: Props) {
  const [chats, setChats] = useState<ConsultantChat[]>([]);
  const [activeChatId, setActiveChatId] = useState<string | null>(null);
  const [messages, setMessages] = useState<ConsultantMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [showPrompts, setShowPrompts] = useState(false);
  const [showClearConfirm, setShowClearConfirm] = useState(false);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const [editingIdx, setEditingIdx] = useState<number | null>(null);
  const [editContent, setEditContent] = useState("");
  const [deleteIdx, setDeleteIdx] = useState<number | null>(null);
  const [deleteChatId, setDeleteChatId] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [importPreview, setImportPreview] = useState<{ role: string; content: string; speaker_name: string; character_id: string | null; avatar_color: string | null } | null>(null);
  const [showImportPreview, setShowImportPreview] = useState(false);
  const importHoverTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const lastAssistantRef = useRef<HTMLDivElement>(null);
  // True while we're waiting for the first token and the prompt is likely
  // in prompt-ingest phase on the local model (large context = long wait
  // before the first token). Shown for the first response in a chat and
  // for the first response after "Import Latest".
  const [showReadingLabel, setShowReadingLabel] = useState(false);

  const categories = buildCategories(characterNames);

  // Load chats list when opened
  const loadChats = useCallback(async () => {
    if (!threadId) return;
    const list = await api.listConsultantChats(threadId);
    setChats(list);
    return list;
  }, [threadId]);

  useEffect(() => {
    if (open && threadId) {
      loadChats();
      setActiveChatId(null);
      setMessages([]);
    }
  }, [open, threadId]);

  // Load messages when active chat changes (skip if currently sending — send manages state itself)
  useEffect(() => {
    if (loading) return;
    setImportPreview(null);
    if (!activeChatId) { setMessages([]); return; }
    api.loadConsultantChat(activeChatId).then((msgs) => {
      setMessages(msgs as ConsultantMessage[]);
    }).catch(() => setMessages([]));
  }, [activeChatId]);

  // Scroll tracking
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const checkBottom = () => setIsAtBottom(el.scrollHeight - el.scrollTop - el.clientHeight < 40);
    el.addEventListener("scroll", checkBottom);
    checkBottom();
    return () => el.removeEventListener("scroll", checkBottom);
  }, [open, activeChatId]);

  // Re-check scroll position when messages change
  useEffect(() => {
    const el = scrollRef.current;
    if (el) setIsAtBottom(el.scrollHeight - el.scrollTop - el.clientHeight < 40);
  }, [messages]);

  // Scroll to bottom only when switching chats or loading history
  useEffect(() => {
    const el = scrollRef.current;
    if (el) setTimeout(() => { el.scrollTop = el.scrollHeight; setIsAtBottom(true); }, 50);
  }, [activeChatId]);

  // Focus input
  useEffect(() => {
    if (open) setTimeout(() => inputRef.current?.focus(), 100);
  }, [open, activeChatId]);

  const createNewChat = useCallback(async () => {
    const chat = await api.createConsultantChat(threadId);
    setChats((prev) => [chat, ...prev]);
    setActiveChatId(chat.chat_id);
    setMessages([]);
    setShowPrompts(false);
  }, [threadId]);

  const send = useCallback(async (text: string) => {
    const trimmed = text.trim();
    if (!trimmed || loading) return;

    let chatId = activeChatId;

    // Auto-create a chat if none exists
    if (!chatId) {
      const chat = await api.createConsultantChat(threadId);
      setChats((prev) => [chat, ...prev]);
      chatId = chat.chat_id;
      setActiveChatId(chatId);
    }

    // Decide whether to show the "Reading the latest messages..." label.
    // True for the first consultant response in a chat (no prior assistant
    // bubble with content), OR when the most recent non-user message was an
    // import — both are the cases where the local model chews on a large
    // fresh prompt before emitting any tokens.
    const priorAssistant = messages.some((m) => m.role === "assistant" && m.content);
    const lastNonUser = [...messages].reverse().find((m) => m.role !== "user");
    const shouldShowReading = !priorAssistant || lastNonUser?.role === "import";
    setShowReadingLabel(shouldShowReading);

    setMessages((prev) => [...prev, { role: "user", content: trimmed }, { role: "assistant", content: "" }]);
    setInput("");
    setLoading(true);
    // Scroll to show the user's message
    setTimeout(() => { const el = scrollRef.current; if (el) el.scrollTo({ top: el.scrollHeight, behavior: "smooth" }); }, 50);
    if (inputRef.current) inputRef.current.style.height = "auto";

    // Listen for streaming tokens
    let chimePlayed = false;
    const unlisten = await listen<string>("consultant-token", (event) => {
      if (!chimePlayed && notifyOnMessage) { playChime(); chimePlayed = true; }
      // First token arrived → we're out of the ingest phase, hide the label.
      setShowReadingLabel(false);
      setMessages((prev) => {
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last?.role === "assistant") {
          updated[updated.length - 1] = { ...last, content: last.content + event.payload };
        }
        // Auto-scroll while streaming — but only until ~300px of the assistant
        // message has entered view, so the reader can keep their eyes on the
        // first paragraph while the rest streams in below the fold.
        // Skip if ref is null: a RAF can fire after loading flips false (the
        // last token races with setLoading), and we don't want that stray
        // scroll-to-bottom at end of stream.
        const el = scrollRef.current;
        if (el) requestAnimationFrame(() => {
          const msgEl = lastAssistantRef.current;
          if (msgEl && msgEl.offsetHeight < 300) el.scrollTop = el.scrollHeight;
        });
        return updated;
      });
    });

    try {
      await api.storyConsultant(apiKey, chatId, characterId, groupChatId, trimmed);

      // Generate title if this is the first message
      const currentChat = chats.find((c) => c.chat_id === chatId);
      if (currentChat?.title === "New Chat" || !currentChat) {
        api.generateConsultantTitle(apiKey, trimmed).then((title) => {
          api.updateConsultantChatTitle(chatId!, title);
          setChats((prev) => prev.map((c) => c.chat_id === chatId ? { ...c, title } : c));
        }).catch(() => {});
      }
    } catch (e) {
      setMessages((prev) => {
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last?.role === "assistant" && !last.content) {
          updated[updated.length - 1] = { ...last, content: `Error: ${e}` };
        } else {
          updated.push({ role: "assistant", content: `Error: ${e}` });
        }
        return updated;
      });
    } finally {
      unlisten();
      setLoading(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [apiKey, characterId, groupChatId, loading, activeChatId, threadId, chats]);

  const handleImport = useCallback(async () => {
    let chatId = activeChatId;
    if (!chatId) {
      const chat = await api.createConsultantChat(threadId);
      setChats((prev) => [chat, ...prev]);
      chatId = chat.chat_id;
      setActiveChatId(chatId);
    }
    try {
      const msg = await api.importChatMessages(chatId, characterId, groupChatId);
      setMessages((prev) => [...prev, msg as ConsultantMessage]);
      setImportPreview(null); // Clear cache so next hover fetches updated last seen
      setTimeout(() => { const el = scrollRef.current; if (el) el.scrollTo({ top: el.scrollHeight, behavior: "smooth" }); }, 50);
    } catch (e) {
      // Show "no new messages" inline rather than as an error
      const errMsg = String(e);
      if (errMsg.includes("No new messages")) {
        setMessages((prev) => [...prev, { role: "assistant" as const, content: "No new messages to import — you're already caught up." }]);
      }
    }
  }, [activeChatId, threadId, characterId, groupChatId]);

  const handleEditSave = async () => {
    if (editingIdx == null || !activeChatId) return;
    const updated = [...messages];
    updated[editingIdx] = { ...updated[editingIdx], content: editContent };
    await api.saveConsultantMessages(activeChatId, updated);
    setMessages(updated);
    setEditingIdx(null);
  };

  const handleDeleteMessage = async () => {
    if (deleteIdx == null || !activeChatId) return;
    const updated = messages.filter((_, idx) => idx !== deleteIdx);
    await api.saveConsultantMessages(activeChatId, updated);
    setMessages(updated);
    setDeleteIdx(null);
  };

  const handleDeleteChat = async () => {
    if (!deleteChatId) return;
    await api.deleteConsultantChat(deleteChatId);
    setChats((prev) => prev.filter((c) => c.chat_id !== deleteChatId));
    if (activeChatId === deleteChatId) {
      const remaining = chats.filter((c) => c.chat_id !== deleteChatId);
      setActiveChatId(remaining[0]?.chat_id ?? null);
    }
    setDeleteChatId(null);
  };

  return (<>
    <Dialog open={open} onClose={onClose} className="max-w-[90vw]">
      <div className="flex h-[88vh] bg-card border border-border rounded-xl shadow-2xl shadow-black/40 overflow-hidden relative">
        {worldImageUrl && (
          <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden">
            <img src={worldImageUrl} alt="" className="w-full h-full object-cover" />
            <div className="absolute inset-0 bg-background/75" />
          </div>
        )}

        {/* Sidebar */}
        {sidebarOpen && (
          <div className="w-56 flex-shrink-0 border-r border-border/30 bg-card/90 backdrop-blur-sm flex flex-col relative z-[1]">
            <div className="px-3 py-3 border-b border-border/30 flex items-center justify-between">
              <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Chats</h3>
              <div className="flex items-center gap-0.5">
                <button
                  onClick={createNewChat}
                  className="w-6 h-6 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
                  title="New chat"
                >
                  <Plus size={14} />
                </button>
                <button
                  onClick={() => setSidebarOpen(false)}
                  className="w-6 h-6 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
                  title="Collapse sidebar"
                >
                  <PanelLeftClose size={14} />
                </button>
              </div>
            </div>
            <div className="flex-1 overflow-y-auto py-1">
              {chats.map((chat) => (
                <div
                  key={chat.chat_id}
                  className={`group/chat relative flex items-center px-3 py-2 cursor-pointer transition-colors ${
                    chat.chat_id === activeChatId ? "bg-accent" : "hover:bg-accent/50"
                  }`}
                  onClick={() => { setActiveChatId(chat.chat_id); setShowPrompts(false); }}
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-xs font-medium truncate">{chat.title}</p>
                    <p className="text-[10px] font-medium text-foreground/70 hidden group-hover/chat:block whitespace-normal leading-snug mt-0.5">{chat.title}</p>
                    <p className="text-[10px] text-muted-foreground/60">
                      {new Date(chat.created_at).toLocaleDateString([], { month: "short", day: "numeric" })}
                      {" · "}
                      {new Date(chat.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </p>
                  </div>
                  <button
                    onClick={(e) => { e.stopPropagation(); setDeleteChatId(chat.chat_id); }}
                    className="flex-shrink-0 w-5 h-5 rounded flex items-center justify-center opacity-0 group-hover/chat:opacity-100 transition-opacity text-muted-foreground hover:text-destructive cursor-pointer"
                  >
                    <Trash2 size={10} />
                  </button>
                </div>
              ))}
              {chats.length === 0 && (
                <p className="text-xs text-muted-foreground/40 px-3 py-4 text-center">No chats yet</p>
              )}
            </div>
          </div>
        )}

        {/* Main chat area */}
        <div className="flex-1 flex flex-col relative z-[1]">
          {/* Header */}
          <div className="flex items-center justify-between px-5 py-3 border-b border-border bg-card/95 relative z-[1]">
            <div className="flex items-center gap-2">
              {!sidebarOpen && (
                <button
                  onClick={() => setSidebarOpen(true)}
                  className="w-7 h-7 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer mr-1"
                  title="Show sidebar"
                >
                  <PanelLeftOpen size={15} />
                </button>
              )}
              <Sparkles size={16} className="text-primary" />
              <h3 className="font-semibold text-sm">Story Consultant</h3>
            </div>
            <button
              onClick={onClose}
              className="w-7 h-7 flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
            >
              <X size={14} />
            </button>
          </div>

          {/* Content area */}
          <div className="flex-1 overflow-hidden relative z-[1]">
            {/* Ideas overlay */}
            {showPrompts && (
              <div className="absolute inset-0 z-10 bg-card overflow-y-auto px-5 py-4">
                <div className="grid grid-cols-2 gap-4 max-w-3xl mx-auto">
                  {categories.map((cat) => (
                    <div key={cat.name}>
                      <h4 className="text-[10px] uppercase tracking-wider font-semibold text-muted-foreground/60 mb-2 px-1">{cat.name}</h4>
                      <div className="space-y-0.5">
                        {cat.prompts.map((p, i) => (
                          <button
                            key={i}
                            onClick={() => { setShowPrompts(false); send(p.prompt); }}
                            className="w-full text-left px-3 py-2 text-[13px] leading-snug rounded-lg hover:bg-accent transition-colors cursor-pointer text-foreground/80 hover:text-foreground"
                          >
                            {p.label}
                          </button>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Messages */}
            <div
              ref={scrollRef}
              className="h-full overflow-y-auto px-5 py-4"
              style={!isAtBottom ? { maskImage: "linear-gradient(to bottom, black 92%, transparent 100%)", WebkitMaskImage: "linear-gradient(to bottom, black 92%, transparent 100%)" } : undefined}
            >
              <div className="max-w-3xl mx-auto space-y-4">
                {messages.length === 0 && !loading && !showPrompts && (
                  <div className="text-center py-12">
                    <Sparkles size={28} className="mx-auto text-muted-foreground/30 mb-3" />
                    <p className="text-sm text-muted-foreground/60">Ask me anything about your story.</p>
                    <p className="text-xs text-muted-foreground/40 mt-1">Click Ideas for inspiration, or type your own question.</p>
                  </div>
                )}
                {messages.map((msg, i) => {
                  // Hide empty assistant message (typing indicator covers this state)
                  if (msg.role === "assistant" && !msg.content) return null;
                  // Import messages render as a collapsed bar
                  if (msg.role === "import") {
                    const label = msg.content.split("\n---\n")[0] || "Imported latest messages";
                    return (
                      <div key={i} className="flex justify-center my-2">
                        <div className="flex items-center gap-2 px-4 py-2 rounded-full bg-sky-500/10 border border-sky-500/20 text-sky-400 text-xs">
                          <Download size={12} />
                          <span>{label}</span>
                        </div>
                      </div>
                    );
                  }
                  const isStreamingAssistant = loading && msg.role === "assistant" && i === messages.length - 1;
                  return (
                  <div key={i} className={`group flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}>
                    <div
                      ref={isStreamingAssistant ? lastAssistantRef : undefined}
                      style={{ fontSize: `${chatFontPx(chatFontSize)}px` }}
                      className={`relative max-w-[85%] rounded-2xl px-4 py-2.5 leading-relaxed ${
                      msg.role === "user"
                        ? "bg-primary text-primary-foreground rounded-br-md"
                        : "bg-secondary/60 text-secondary-foreground rounded-bl-md border border-border/30 backdrop-blur-sm"
                    }`}>
                      {msg.role === "user" && (
                        <div className="absolute top-2 right-8 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                          <button
                            onClick={() => { setEditingIdx(i); setEditContent(msg.content); }}
                            className="w-7 h-7 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm"
                          >
                            <Pencil size={12} />
                          </button>
                          <button
                            onClick={() => setDeleteIdx(i)}
                            className="w-7 h-7 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-destructive transition-colors backdrop-blur-sm"
                          >
                            <Trash2 size={12} />
                          </button>
                        </div>
                      )}
                      {msg.role === "assistant" ? (
                        <div className="prose prose-sm max-w-none [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-headings:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)] [--tw-prose-bullets:var(--color-secondary-foreground)] [--tw-prose-counters:var(--color-secondary-foreground)] [--tw-prose-links:var(--color-primary)]">
                          <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(msg.content)}</Markdown>
                        </div>
                      ) : (
                        <p>{msg.content}</p>
                      )}
                      <button
                        onClick={async () => {
                          if (!activeChatId || loading) return;
                          if (msg.role === "user") {
                            // Truncate to before this message, then re-send it
                            await api.truncateConsultantChat(activeChatId, i);
                            setMessages(messages.slice(0, i));
                            send(msg.content);
                          } else {
                            await api.truncateConsultantChat(activeChatId, i + 1);
                            setMessages(messages.slice(0, i + 1));
                          }
                        }}
                        className={`text-[10px] mt-1 opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer ${
                          msg.role === "user"
                            ? "text-primary-foreground/40 hover:text-primary-foreground/70"
                            : "text-muted-foreground/40 hover:text-muted-foreground"
                        }`}
                      >
                        Reset to Here
                      </button>
                    </div>
                  </div>
                  );
                })}
                {/* Typing indicator shows only before first token arrives */}
                {loading && messages.length > 0 && messages[messages.length - 1]?.role === "assistant" && !messages[messages.length - 1]?.content && (
                  <div className="flex flex-col items-start gap-1.5 -mt-4">
                    <div className="bg-secondary/60 rounded-2xl rounded-bl-md px-4 py-3 flex items-center gap-1.5 border border-border/30 backdrop-blur-sm">
                      <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:0ms]" />
                      <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:150ms]" />
                      <span className="w-1.5 h-1.5 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:300ms]" />
                    </div>
                    {showReadingLabel && (
                      <span className="text-[13px] text-muted-foreground/60 ml-3 animate-pulse">Reading the latest messages...</span>
                    )}
                  </div>
                )}
              </div>
            </div>
            {/* Scroll to bottom */}
            {!isAtBottom && (
              <button
                onClick={() => { const el = scrollRef.current; if (el) el.scrollTo({ top: el.scrollHeight, behavior: "smooth" }); }}
                className="absolute bottom-4 left-1/2 -translate-x-1/2 w-8 h-8 rounded-full bg-card/80 backdrop-blur-sm shadow-lg shadow-black/20 border border-border/30 flex items-center justify-center cursor-pointer hover:bg-card transition-colors text-muted-foreground hover:text-foreground"
              >
                <ChevronDown size={16} />
              </button>
            )}
          </div>

          {/* Input area */}
          <div className="flex-shrink-0 border-t border-border px-4 py-3 relative z-[1]">
            <div className="max-w-3xl mx-auto flex items-end gap-2">
              <button
                onClick={() => setShowPrompts(!showPrompts)}
                className={`flex-shrink-0 h-9 rounded-lg flex items-center gap-1.5 px-3 text-sm font-medium transition-colors cursor-pointer ${
                  showPrompts ? "bg-amber-500/20 text-amber-400" : "text-muted-foreground hover:text-foreground hover:bg-accent"
                }`}
              >
                <Lightbulb size={18} />
                <span>Ideas</span>
              </button>
              <textarea
                ref={inputRef}
                value={input}
                onChange={(e) => {
                  setInput(e.target.value);
                  e.target.style.height = "auto";
                  if (e.target.scrollHeight > e.target.offsetHeight) {
                    e.target.style.height = Math.min(e.target.scrollHeight, 120) + "px";
                  }
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    send(input);
                  }
                }}
                placeholder="Ask about your story..."
                style={{ fontSize: `${chatFontPx(chatFontSize)}px` }}
                className="flex-1 max-h-[120px] resize-none rounded-lg border border-input bg-transparent px-3 py-2 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                rows={1}
                disabled={loading}
              />
              <button
                onClick={() => send(input)}
                disabled={!input.trim() || loading}
                className="flex-shrink-0 w-9 h-9 rounded-lg flex items-center justify-center bg-primary text-primary-foreground transition-colors cursor-pointer hover:bg-primary/90 disabled:opacity-40 disabled:cursor-not-allowed"
              >
                {loading ? <Loader2 size={16} className="animate-spin" /> : <Send size={16} />}
              </button>
              <div className="relative"
                onMouseEnter={() => {
                  if (importHoverTimer.current) clearTimeout(importHoverTimer.current);
                  setShowImportPreview(true);
                  if (activeChatId && !importPreview) {
                    api.getLastSeenMessage(activeChatId).then((p) => setImportPreview(p));
                  }
                }}
                onMouseLeave={() => {
                  importHoverTimer.current = setTimeout(() => setShowImportPreview(false), 200);
                }}
              >
                <button
                  onClick={handleImport}
                  disabled={loading}
                  className="flex-shrink-0 h-9 rounded-lg flex items-center gap-1.5 px-3 text-sm font-medium transition-colors cursor-pointer text-muted-foreground hover:text-sky-400 hover:bg-sky-500/10 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  <Download size={14} />
                  <span>Import Latest</span>
                </button>
                {showImportPreview && (
                  <div className="absolute bottom-full right-0 mb-2 w-96 bg-card border border-border rounded-xl shadow-2xl shadow-black/40 overflow-hidden animate-in fade-in zoom-in-95 duration-150">
                    <div className="px-3 py-2 border-b border-border/50">
                      <p className="text-[10px] uppercase tracking-wider font-semibold text-muted-foreground/60">Import Latest Messages</p>
                      <p className="text-[10px] text-muted-foreground/40 mt-0.5">Last seen:</p>
                    </div>
                    {importPreview ? (
                      <div className="p-3 max-h-64 overflow-y-auto">
                        {importPreview.role === "narrative" ? (
                          <div className="rounded-lg px-4 py-3 bg-gradient-to-br from-amber-950/40 to-amber-900/20 border border-amber-700/30 text-amber-100/90 italic text-sm leading-relaxed">
                            <div className="flex items-center gap-1.5 mb-1.5 text-[10px] uppercase tracking-wider text-amber-500/70 font-semibold not-italic">
                              <BookOpen size={12} />
                              <span>Narrative</span>
                            </div>
                            <div className="prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:rgb(252,211,77,0.9)] [--tw-prose-bold:rgb(252,211,77)]">
                              <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(importPreview.content)}</Markdown>
                            </div>
                          </div>
                        ) : (
                          <div className={`rounded-lg px-4 py-3 text-sm leading-relaxed ${
                            importPreview.role === "user"
                              ? "bg-primary text-primary-foreground"
                              : "bg-secondary/40 text-secondary-foreground border border-border/30"
                          }`}>
                            <div className="flex items-center gap-2 mb-1.5">
                              {importPreview.role === "user" && userAvatarUrl ? (
                                <img src={userAvatarUrl} alt="" className="w-10 h-10 rounded-full object-cover ring-1 ring-border flex-shrink-0" />
                              ) : importPreview.character_id && portraits[importPreview.character_id] ? (
                                <img src={portraits[importPreview.character_id]} alt="" className="w-10 h-10 rounded-full object-cover ring-1 ring-border flex-shrink-0" />
                              ) : importPreview.avatar_color ? (
                                <span className="w-10 h-10 rounded-full flex-shrink-0 ring-1 ring-white/10" style={{ backgroundColor: importPreview.avatar_color }} />
                              ) : null}
                              <p className="text-[10px] font-semibold text-muted-foreground/70">{importPreview.speaker_name}</p>
                            </div>
                            <div className={`prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_em]:italic ${
                              importPreview.role === "user"
                                ? "[--tw-prose-body:var(--color-primary-foreground)] [--tw-prose-bold:var(--color-primary-foreground)]"
                                : "[--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)]"
                            }`}>
                              <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(importPreview.content)}</Markdown>
                            </div>
                          </div>
                        )}
                      </div>
                    ) : (
                      <div className="p-3 text-xs text-muted-foreground/50 text-center">No previous context</div>
                    )}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </Dialog>

    {/* Delete chat confirmation */}
    {deleteChatId && (
      <div className="fixed inset-0 z-[60]">
        <Dialog open onClose={() => setDeleteChatId(null)} className="max-w-xs">
          <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
            <div className="flex items-center gap-2">
              <Trash2 size={18} className="text-destructive" />
              <h3 className="font-semibold">Delete Chat</h3>
            </div>
            <p className="text-sm text-muted-foreground">
              Delete this consultant conversation? This cannot be undone.
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="ghost" size="sm" onClick={() => setDeleteChatId(null)}>Cancel</Button>
              <Button variant="destructive" size="sm" onClick={handleDeleteChat}>Delete</Button>
            </div>
          </div>
        </Dialog>
      </div>
    )}

    {/* Edit message modal */}
    {editingIdx != null && (
      <div className="fixed inset-0 z-[70]">
        <Dialog open onClose={() => setEditingIdx(null)} className="max-w-lg">
          <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
            <div className="flex items-center gap-2">
              <Pencil size={18} className="text-primary" />
              <h3 className="font-semibold">Edit Message</h3>
            </div>
            <textarea
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              className="w-full min-h-[120px] max-h-[300px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm font-mono focus:outline-none focus:ring-1 focus:ring-ring"
              autoFocus
            />
            <div className="flex justify-end gap-2">
              <Button variant="ghost" size="sm" onClick={() => setEditingIdx(null)}>Cancel</Button>
              <Button size="sm" disabled={!editContent.trim() || editContent === messages[editingIdx]?.content} onClick={handleEditSave}>
                <Pencil size={14} className="mr-1.5" />
                Update
              </Button>
            </div>
          </div>
        </Dialog>
      </div>
    )}

    {/* Delete message confirmation */}
    {deleteIdx != null && (
      <div className="fixed inset-0 z-[70]">
        <Dialog open onClose={() => setDeleteIdx(null)} className="max-w-xs">
          <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
            <div className="flex items-center gap-2">
              <Trash2 size={18} className="text-destructive" />
              <h3 className="font-semibold">Delete Message</h3>
            </div>
            <p className="text-sm text-muted-foreground">
              Delete this message from the conversation?
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="ghost" size="sm" onClick={() => setDeleteIdx(null)}>Cancel</Button>
              <Button variant="destructive" size="sm" onClick={handleDeleteMessage}>Delete</Button>
            </div>
          </div>
        </Dialog>
      </div>
    )}
  </>);
}

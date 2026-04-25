import { useState, useRef, useEffect, useCallback } from "react";
import Markdown from "react-markdown";
import { Dialog } from "@/components/ui/dialog";
import { X, Loader2, Send, Lightbulb, Sparkles, Trash2, ChevronDown, Pencil, Plus, PanelLeftClose, PanelLeftOpen, Download, BookOpen, Drama, Eye, Feather, Wand2, RotateCcw, Sprout, Map, Hammer, ArrowRight, Users, Flame, type LucideIcon } from "lucide-react";
import { formatMessage, markdownComponents, consultantMarkdownComponents, consultantStreamingMarkdownComponents, transformConsultantIcons, remarkPlugins, rehypePlugins } from "./formatMessage";
import { listen } from "@tauri-apps/api/event";
import { api, type ConsultantChat } from "@/lib/tauri";
import { BackstageActionCard, parseBackstageSegments, type BackstageActionContext } from "./BackstageActionCard";

const CONSULTANT_MODE_KEY = "worldthreads:consultant-mode";
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
  /** Active world id — required for Backstage actions that create
   *  world-scoped entities (e.g. new group chats). */
  worldId: string;
  /** When set, the modal auto-sends this text as a user message exactly
   *  once after `open` becomes true. Used by per-message "How do I
   *  react!?" buttons in ChatView/GroupChatView. Parent should clear it
   *  via `onAutoSendConsumed` after firing so reopening the modal
   *  later doesn't re-send the same prompt. */
  autoSendOnOpen?: string | null;
  onAutoSendConsumed?: () => void;
}

interface PromptCategory {
  name: string;
  /// Lucide icon shown next to the category heading. Picks a verb-shape
  /// for the category (Eye for diagnostic, Feather for craft, Wand for
  /// action, etc.) so the heading row reads at a glance.
  icon: LucideIcon;
  prompts: Array<{ label: string; prompt: string }>;
}

/// Backstage-specific ideas. Covers the superset of what Backstage
/// uniquely answers well that Immersive can't — state of the world,
/// craft/worldbuilding, action proposals, post-mortem, maintenance.
/// Prompts are plainspoken (the user types in their voice; the theatre
/// register is Backstage's voice, not the user's). Tuned to be bold —
/// these should provoke, name what's been ducked, and push toward the
/// brave move rather than the safe survey.
function buildBackstageCategories(names: string[]): PromptCategory[] {
  return [
    {
      name: "State of the World",
      icon: Eye,
      prompts: [
        { label: "What's gone stale and won't admit it?", prompt: "Be honest with me — what in this world has gone stale lately and is coasting on past momentum? Name it specifically." },
        { label: "Which character is being protected from harm?", prompt: "Which character have I been quietly protecting from anything difficult? Who needs something to actually go wrong for them?" },
        { label: "Where am I avoiding a confrontation?", prompt: "Where in this world am I dodging a confrontation that should have already happened? Point to the chat and the unsaid thing." },
      ],
    },
    {
      name: "Craft & Worldbuilding",
      icon: Feather,
      prompts: [
        ...names.slice(0, 1).map((n) => ({ label: `What about ${n} am I afraid to write?`, prompt: `What about ${n} have I been treating with kid gloves? What dimension of them would I write if I weren't worried about being kind?` })),
        { label: "What would a brutal editor cut?", prompt: "If a brutal editor read the last month of this world's chats, what would they cut, and what would they tell me to write more of?" },
        { label: "What truth about this world am I avoiding?", prompt: "What truth about this world have I been circling but not writing? Name it." },
      ],
    },
    {
      name: "Stage a Move",
      icon: Drama,
      prompts: [
        ...names.slice(0, 1).map((n) => ({ label: `Stage a costly move for ${n}`, prompt: `Look at recent activity with ${n} and propose a canon update that would actually cost them something — not a flattering note, a real shift.` })),
        { label: "Where am I hedging? Stage the line that stops.", prompt: "Pick the chat where I've been hedging the most and draft the message that stops hedging. Don't soften it." },
        { label: "What's the secret I should let surface?", prompt: "Is there a secret or unsaid thing in this world that I've kept buried too long? Suggest where it surfaces and who notices first." },
      ],
    },
    {
      name: "Look Back",
      icon: RotateCcw,
      prompts: [
        { label: "What story am I actually telling?", prompt: "Step back. Across the last weeks of this world, what story am I actually telling — not what I think I'm telling, what's on the page?" },
        { label: "Which moment did I rush past that should have hurt?", prompt: "Looking back, which moment did I rush past that should have actually hurt — a death, a betrayal, a goodbye I underplayed?" },
        { label: "What's the through-line costing me?", prompt: "What's the through-line of this world right now — and what's it costing me to keep telling it that way? What might be on the other side of changing it?" },
      ],
    },
    {
      name: "Tending the World",
      icon: Sprout,
      prompts: [
        { label: "Which canon entry is now a lie?", prompt: "Which canonized entry (description weave, kept record) is now a lie about who this character actually is? Suggest the rewrite." },
        { label: "Which character description is flattering, not honest?", prompt: "Which character's description is flattering them rather than describing them as they actually behave now? Name it and propose the honest version." },
        { label: "Where would an ending serve this world?", prompt: "Where in this world would a real ending — a death, a departure, a relationship breaking — serve the larger story right now? Don't be precious about it." },
      ],
    },
    {
      // Backstage as a wayfinding companion — the user can ask "where
      // is X" or "how do I do Y" and get a direct answer about the
      // app's UI rather than craft advice. Different register from the
      // bolder categories: matter-of-fact, helpful, location-aware.
      name: "Find Your Way Around",
      icon: Map,
      prompts: [
        ...names.slice(0, 1).map((n) => ({ label: `Where do I change ${n}'s portrait?`, prompt: `Where in the app do I change ${n}'s portrait — generate a new one, pick from the gallery, or upload my own? Walk me through it.` })),
        { label: "How do I start a group chat?", prompt: "How do I start a group chat — picking which characters are in it, naming it, and getting the first message going?" },
        { label: "Where do I change the chat background or world image?", prompt: "Where do I change the chat background, manage the world image gallery, or set the active world image? Walk me through the world-image controls." },
        { label: "How do I undo a canon entry I just made?", prompt: "I just canonized something and I want to undo it — where do I go to find that kept record and remove it, and what does removing it actually change about the character?" },
        { label: "Where are my backups — and how do I restore one?", prompt: "Where does the app store its backups, how do I trigger one manually, and how do I restore from a previous backup if I need to?" },
        { label: "How do I switch which AI model is being used?", prompt: "Where do I change which AI model the app is using — globally, and per-chat? Walk me through both controls." },
        { label: "Where do I find all my Imagined Chapters?", prompt: "Where do I open the Imagined Chapters modal for a chat, and how do I see (and revisit) every chapter I've already made for a thread?" },
      ],
    },
    {
      // Backstage's other strength: it knows the apparatus. This category
      // surfaces the app's actual tools (canonization, imagined chapters,
      // meanwhile events, portraits, illustrations, journals, group chats,
      // inventories) so the user can ASK Backstage to recommend which
      // tool to reach for and why. The bold register stays — these are
      // pointed asks, not menu items.
      name: "Reach for the Apparatus",
      icon: Hammer,
      prompts: [
        { label: "What moment is overdue for canonization?", prompt: "Look at recent activity — which specific message or exchange is overdue for canonization? Tell me which to click 'Remember this' on and which deserves 'This changes them.'" },
        { label: "What scene is begging for an Imagined Chapter?", prompt: "Which moment from recent chats is begging to be turned into an imagined chapter — something that would gain weight from being rendered as image + prose?" },
        ...names.slice(0, 1).map((n) => ({ label: `Has ${n}'s portrait fallen behind?`, prompt: `Has ${n} shifted in a way the current portrait no longer carries — emotionally, physically, in bearing? Should I generate a new one, and what should it capture?` })),
        ...names.slice(0, 1).map((n) => ({ label: `Should ${n}'s inventory be refreshed?`, prompt: `Look at ${n}'s recent activity — has anything changed about what they're carrying, wearing, or holding that the inventory should now reflect? If so, tell me what to update.` })),
        { label: "Two characters who should be in a group chat", prompt: "Which two (or three) characters should be put in a group chat together that haven't been? What's the encounter that would surface, and which world-day setting would land it best?" },
        { label: "Whose user-journal entry am I overdue for?", prompt: "Looking at what I've been doing in this world recently, what would my own user-journal entry need to say honestly? Draft the shape of it." },
      ],
    },
  ];
}

function buildCategories(names: string[]): PromptCategory[] {
  return [
    {
      name: "What's Next",
      icon: ArrowRight,
      prompts: [
        { label: "What should I do?", prompt: "What should I do next? Give me a few options." },
        { label: "How should I respond to that?", prompt: "How should I respond to what just happened? Give me a few options." },
        { label: "What haven't I thought of?", prompt: "What's an angle here I haven't considered?" },
      ],
    },
    {
      name: "The People",
      icon: Users,
      prompts: [
        ...names.slice(0, 1).map((n) => ({ label: `What's going on with ${n}?`, prompt: `What's going on with ${n} right now? What do you think they're feeling?` })),
        { label: "Where do things stand between us?", prompt: "Where do things stand between us right now? What's changed recently?" },
        ...names.slice(0, 1).map((n) => ({ label: `What might ${n} do next?`, prompt: `What do you think ${n} might do next? What would you expect from them here?` })),
      ],
    },
    {
      name: "Stepping Back",
      icon: Eye,
      prompts: [
        { label: "What's really going on here?", prompt: "Step back with me — what's really going on here? What's the bigger picture?" },
        { label: "What's the subtext right now?", prompt: "What's the subtext of what just happened? What's going on beneath the surface?" },
        { label: "What would you do in my position?", prompt: "Honestly — what would you do if you were me right now?" },
      ],
    },
    {
      name: "The Tension",
      icon: Flame,
      prompts: [
        { label: "How can I push this further?", prompt: "How can I push this further? Raise the stakes a little?" },
        { label: "How can I ease things?", prompt: "How can I ease things? Bring some warmth or calm to this?" },
        { label: "What am I avoiding?", prompt: "Am I avoiding something? Something I should probably say or do but haven't?" },
      ],
    },
  ];
}

export function StoryConsultantModal({ open, onClose, apiKey, characterId, groupChatId, threadId, characterNames, worldImageUrl, portraits, userAvatarUrl, notifyOnMessage, chatFontSize, worldId, autoSendOnOpen, onAutoSendConsumed }: Props) {
  const [chats, setChats] = useState<ConsultantChat[]>([]);
  const [activeChatId, setActiveChatId] = useState<string | null>(null);
  // Which tab the sidebar is showing. Also determines the mode of any
  // new chat created from the sidebar "+" button. Persisted across
  // sessions in localStorage so users who live in one mode don't have
  // to re-select it every time they open the modal.
  const [activeMode, setActiveMode] = useState<"immersive" | "backstage">(() => {
    try {
      const stored = localStorage.getItem(CONSULTANT_MODE_KEY);
      return stored === "backstage" ? "backstage" : "immersive";
    } catch { return "immersive"; }
  });
  useEffect(() => {
    try { localStorage.setItem(CONSULTANT_MODE_KEY, activeMode); } catch { /* private mode etc. */ }
  }, [activeMode]);
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

  const categories = activeMode === "backstage"
    ? buildBackstageCategories(characterNames)
    : buildCategories(characterNames);

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
    const chat = await api.createConsultantChat(threadId, undefined, activeMode);
    setChats((prev) => [chat, ...prev]);
    setActiveChatId(chat.chat_id);
    setMessages([]);
    setShowPrompts(false);
  }, [threadId, activeMode]);

  const send = useCallback(async (text: string) => {
    const trimmed = text.trim();
    if (!trimmed || loading) return;

    let chatId = activeChatId;

    // Auto-create a chat if none exists. Uses the active tab's mode so
    // pressing send on an empty Backstage tab starts a Backstage chat.
    if (!chatId) {
      const chat = await api.createConsultantChat(threadId, undefined, activeMode);
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
  }, [apiKey, characterId, groupChatId, loading, activeChatId, threadId, chats, activeMode]);

  const handleImport = useCallback(async () => {
    let chatId = activeChatId;
    if (!chatId) {
      const chat = await api.createConsultantChat(threadId, undefined, activeMode);
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
  }, [activeChatId, threadId, characterId, groupChatId, activeMode]);

  // Auto-send-on-open: per-message "How do I react!?" buttons set
  // `autoSendOnOpen` and flip `open` true in the same render. After the
  // modal mounts, fire send(text) once and notify the parent so the
  // trigger gets cleared.
  //
  // CRITICAL — refs over deps: `send` is a useCallback whose identity
  // changes whenever its deps move (loading, chats, activeChatId, ...).
  // Several of those deps change within ~10-50ms of the modal opening
  // (loadChats resolves → setChats fires → send identity changes). If
  // we put `send` in this effect's dep array, the new identity causes
  // a re-run, the cleanup cancels the pending setTimeout, and the
  // 50ms callback never fires — auto-send silently no-ops. Ref the
  // function and depend only on the trigger flags.
  const autoSendFiredRef = useRef<string | null>(null);
  const sendRef = useRef(send);
  sendRef.current = send;
  const onAutoSendConsumedRef = useRef(onAutoSendConsumed);
  onAutoSendConsumedRef.current = onAutoSendConsumed;
  useEffect(() => {
    if (!open) { autoSendFiredRef.current = null; return; }
    if (!autoSendOnOpen) return;
    if (autoSendFiredRef.current === autoSendOnOpen) return;
    autoSendFiredRef.current = autoSendOnOpen;
    // Tiny defer so the modal mount + chat-load effects settle before
    // send() runs. send() auto-creates a chat if there isn't one.
    const t = setTimeout(() => {
      sendRef.current(autoSendOnOpen);
      onAutoSendConsumedRef.current?.();
    }, 100);
    return () => clearTimeout(t);
  }, [open, autoSendOnOpen]);

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

  const isBackstageMode = activeMode === "backstage";

  // Fade-through-solid mode-bg crossfade. When activeMode changes:
  // 1. The currently-displayed bg fades out to opacity 0 (350ms),
  //    revealing the solid bg-card behind.
  // 2. displayedMode is swapped to the new mode while the bg is invisible.
  // 3. The new bg fades in from opacity 0 to 1 (350ms).
  // Net effect: bg → solidColor → bg, exactly the requested transition shape.
  const [displayedMode, setDisplayedMode] = useState<"immersive" | "backstage">(activeMode);
  const [bgOpacity, setBgOpacity] = useState(1);
  useEffect(() => {
    if (activeMode === displayedMode) return;
    setBgOpacity(0);
    const t = setTimeout(() => {
      setDisplayedMode(activeMode);
      setBgOpacity(1);
    }, 350);
    return () => clearTimeout(t);
  }, [activeMode, displayedMode]);
  const isBackstageDisplayed = displayedMode === "backstage";

  return (<>
    <Dialog open={open} onClose={onClose} className="max-w-[90vw]">
      <div className="flex h-[88vh] bg-card border border-border rounded-xl shadow-2xl shadow-black/40 overflow-hidden relative">
        {/* World-image bg only renders in IMMERSIVE mode (background of the
            whole modal including sidebar). In backstage mode the world
            image is suppressed and the viewport itself carries the
            theatrical bg + inner glow (so the sidebar doesn't eat the
            left edge of the glow). Opacity tied to bgOpacity so the
            crossfade dissolves through the solid bg-card cleanly when
            switching modes. */}
        {worldImageUrl && (
          <div
            className="absolute inset-0 z-0 pointer-events-none overflow-hidden transition-opacity duration-[350ms] ease-in-out"
            style={{ opacity: !isBackstageDisplayed ? bgOpacity : 0 }}
          >
            <img src={worldImageUrl} alt="" className="w-full h-full object-cover" />
            <div className="absolute inset-0 bg-background/75" />
          </div>
        )}

        {/* Sidebar */}
        {sidebarOpen && (() => {
          const visibleChats = chats.filter((c) => (c.mode ?? "immersive") === activeMode);
          return (
          <div className="w-56 flex-shrink-0 border-r border-border/30 bg-card/90 backdrop-blur-sm flex flex-col relative z-[1]">
            {/* Sidebar mode tabs removed — mode toggle now lives prominently
                in the header. The sidebar just lists chats for the
                currently-active mode. */}
            <div className="px-3 py-2.5 border-b border-border/30 flex items-center justify-between">
              <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Chats</h3>
              <div className="flex items-center gap-0.5">
                <button
                  onClick={createNewChat}
                  className="w-6 h-6 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
                  title={`New ${activeMode} chat`}
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
              {visibleChats.map((chat) => {
                const isBackstage = chat.mode === "backstage";
                return (
                <div
                  key={chat.chat_id}
                  className={`group/chat relative flex items-center px-3 py-2 cursor-pointer transition-colors border-l-2 ${
                    chat.chat_id === activeChatId
                      ? (isBackstage ? "bg-amber-500/10 border-amber-400/70" : "bg-accent border-primary/60")
                      : (isBackstage ? "border-amber-500/20 hover:bg-amber-500/5" : "border-transparent hover:bg-accent/50")
                  }`}
                  onClick={() => { setActiveChatId(chat.chat_id); setShowPrompts(false); }}
                >
                  <div className="flex-shrink-0 mr-2 opacity-70">
                    {isBackstage ? <Drama size={11} className="text-amber-400/80" /> : <Sparkles size={11} className="text-primary/70" />}
                  </div>
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
                );
              })}
              {visibleChats.length === 0 && (
                <p className="text-xs text-muted-foreground/40 px-3 py-4 text-center">
                  {activeMode === "backstage" ? "No Backstage chats yet" : "No chats yet"}
                </p>
              )}
            </div>
          </div>
          );
        })()}

        {/* Main chat area */}
        <div className="flex-1 flex flex-col relative z-[1]">
          {/* Mode-themed bg + inner-glow layer — lives INSIDE the
              viewport so the sidebar doesn't eat the left edge of the
              glow. Two layered themes (immersive indigo/violet, backstage
              deep-amber + gold) keyed on displayedMode; opacity follows
              bgOpacity for the fade-through-solid crossfade. */}
          <div
            className="absolute inset-0 z-0 pointer-events-none overflow-hidden transition-opacity duration-[350ms] ease-in-out"
            style={{ opacity: bgOpacity }}
          >
            {isBackstageDisplayed ? (
              <>
                <div className="w-full h-full bg-gradient-to-br from-amber-900/60 via-amber-950/80 to-amber-900/60" />
                <div className="absolute inset-0 [box-shadow:inset_0_0_120px_rgba(252,211,77,0.25),inset_0_0_60px_rgba(252,211,77,0.35)] pointer-events-none" />
              </>
            ) : (
              <div className="absolute inset-0 [box-shadow:inset_0_0_120px_rgba(99,102,241,0.18),inset_0_0_60px_rgba(165,180,252,0.22)] pointer-events-none" />
            )}
          </div>
          {/* Header — Big Mode-switch. The toggle IS the modal's title.
              Each mode is themed distinctly so the switch feels like
              flipping a lens, not picking a tab:
                Immersive → indigo/violet, soft sparkle, "in-the-story"
                Backstage → amber, theatrical, "fourth-wall"
              Active mode fills with its accent + label tagline; inactive
              mode is plain text with the icon, clearly subordinate but
              one click away. */}
          <div className="flex items-center px-5 py-3 border-b border-border bg-gradient-to-b from-card/95 to-card/80 relative z-[1]">
            {/* Left slot — sidebar toggle (or spacer to keep center true). */}
            <div className="flex items-center w-10 flex-shrink-0">
              {!sidebarOpen && (
                <button
                  onClick={() => setSidebarOpen(true)}
                  className="w-8 h-8 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
                  title="Show sidebar"
                >
                  <PanelLeftOpen size={16} />
                </button>
              )}
            </div>
            {/* Centered big-brassy mode toggle. The toggle IS the modal's
                title — wider buttons, proudly placed, so the choice reads
                at a glance. */}
            <div className="flex-1 flex justify-center">
              <div className="inline-flex rounded-xl overflow-hidden border border-border/80 bg-background/40 shadow-inner shadow-black/20 ring-1 ring-white/5">
                <button
                  onClick={() => {
                    setActiveMode("immersive");
                    if (activeChatId && chats.find((c) => c.chat_id === activeChatId)?.mode === "backstage") {
                      setActiveChatId(null);
                      setMessages([]);
                    }
                  }}
                  className={`group/im flex items-center justify-center gap-3 px-9 py-3 min-w-[200px] transition-all cursor-pointer ${
                    activeMode === "immersive"
                      ? "bg-gradient-to-br from-indigo-500/30 via-violet-500/20 to-indigo-500/15 text-indigo-100 shadow-[inset_0_-2px_0_rgba(99,102,241,0.5)]"
                      : "text-muted-foreground hover:text-foreground hover:bg-accent/40"
                  }`}
                  title="Immersive — in-the-story confidant"
                >
                  <Sparkles
                    size={20}
                    className={activeMode === "immersive" ? "text-indigo-300 drop-shadow-[0_0_5px_rgba(165,180,252,0.7)]" : ""}
                  />
                  <span className="flex flex-col items-start leading-tight">
                    <span className="text-base font-bold tracking-tight">
                      Immersive
                    </span>
                    <span className={`text-[10px] uppercase tracking-[0.14em] ${activeMode === "immersive" ? "text-indigo-300/80" : "text-muted-foreground/50"}`}>
                      in-the-story
                    </span>
                  </span>
                </button>
                <button
                  onClick={() => {
                    setActiveMode("backstage");
                    if (activeChatId && chats.find((c) => c.chat_id === activeChatId)?.mode !== "backstage") {
                      setActiveChatId(null);
                      setMessages([]);
                    }
                  }}
                  className={`group/bs flex items-center justify-center gap-3 px-9 py-3 min-w-[200px] transition-all cursor-pointer border-l border-border/80 ${
                    activeMode === "backstage"
                      ? "bg-gradient-to-br from-amber-500/30 via-orange-500/20 to-amber-500/15 text-amber-100 shadow-[inset_0_-2px_0_rgba(245,158,11,0.5)]"
                      : "text-muted-foreground hover:text-foreground hover:bg-accent/40"
                  }`}
                  title="Backstage — fourth-wall stage manager"
                >
                  <Drama
                    size={20}
                    className={activeMode === "backstage" ? "text-amber-300 drop-shadow-[0_0_5px_rgba(252,211,77,0.7)]" : ""}
                  />
                  <span className="flex flex-col items-start leading-tight">
                    <span className="text-base font-bold tracking-tight">
                      Backstage
                    </span>
                    <span className={`text-[10px] uppercase tracking-[0.14em] ${activeMode === "backstage" ? "text-amber-300/80" : "text-muted-foreground/50"}`}>
                      fourth-wall
                    </span>
                  </span>
                </button>
              </div>
            </div>
            {/* Right slot — close button. Same width as left slot so the
                toggle stays visually centered. */}
            <div className="flex items-center justify-end w-10 flex-shrink-0">
              <button
                onClick={onClose}
                className="w-8 h-8 flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
              >
                <X size={15} />
              </button>
            </div>
          </div>

          {/* Content area */}
          <div className="flex-1 overflow-hidden relative z-[1]">
            {/* Ideas overlay */}
            {showPrompts && (
              <div className="absolute inset-0 z-10 bg-card overflow-y-auto px-5 py-4">
                <div className="grid grid-cols-2 gap-4 max-w-3xl mx-auto">
                  {categories.map((cat) => (
                    <div key={cat.name}>
                      <h4 className="text-[10px] uppercase tracking-wider font-semibold text-muted-foreground/60 mb-2 px-1 flex items-center gap-1.5">
                        <cat.icon size={11} className="text-muted-foreground/70" />
                        <span>{cat.name}</span>
                      </h4>
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
                    {activeMode === "backstage" ? (
                      <>
                        <Drama size={28} className="mx-auto text-amber-400/40 mb-3" />
                        <p className="text-sm text-muted-foreground/70">I've been watching from the wings.</p>
                        <p className="text-xs text-muted-foreground/40 mt-1">Ask what I'm noticing, or for a suggestion on what to try next.</p>
                      </>
                    ) : (
                      <>
                        <Sparkles size={28} className="mx-auto text-muted-foreground/30 mb-3" />
                        <p className="text-sm text-muted-foreground/60">Ask me anything about your story.</p>
                        <p className="text-xs text-muted-foreground/40 mt-1">Click Ideas for inspiration, or type your own question.</p>
                      </>
                    )}
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
                        <div
                          style={{ fontSize: `${chatFontPx(chatFontSize)}px` }}
                          className="prose prose-sm max-w-none [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-headings:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)] [--tw-prose-bullets:var(--color-secondary-foreground)] [--tw-prose-counters:var(--color-secondary-foreground)] [--tw-prose-links:var(--color-primary)] [--tw-prose-quotes:var(--color-secondary-foreground)] [--tw-prose-quote-borders:rgba(255,255,255,0.25)]"
                        >
                          {/* Both modes can emit ```action cards now —
                              Immersive's prompt teaches a small in-world
                              subset (staged_message, canon_entry,
                              new_group_chat). The renderer is mode-agnostic;
                              what differs is what the prompt asks for. */}
                          {!isStreamingAssistant ? (
                            <>
                              {parseBackstageSegments(msg.content).map((seg, segIdx) => (
                                seg.kind === "text"
                                  ? (seg.value.trim()
                                      ? <Markdown key={segIdx} components={consultantMarkdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{transformConsultantIcons(formatMessage(seg.value))}</Markdown>
                                      : null)
                                  : <BackstageActionCard
                                      key={segIdx}
                                      block={seg.block}
                                      ctx={{
                                        activeThreadId: threadId,
                                        groupChatId,
                                        onAppliedClose: onClose,
                                        apiKey,
                                        worldId,
                                      } as BackstageActionContext}
                                    />
                              ))}
                            </>
                          ) : (
                            <Markdown components={consultantStreamingMarkdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{transformConsultantIcons(formatMessage(msg.content))}</Markdown>
                          )}
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

import React, { useState, useEffect } from "react";
import Markdown from "react-markdown";
import { BookOpen, Link2, Image, Loader2, Trash2, BookText, Sparkles, RotateCw, Pencil, MessageSquare } from "lucide-react";
import { formatMessage, markdownComponents, remarkPlugins, rehypePlugins } from "./formatMessage";
import { TimeDivider } from "./TimeDivider";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogBody } from "@/components/ui/dialog";
import { CyclingLoadingMessages } from "@/components/ui/cycling-loading-messages";
import { listen } from "@tauri-apps/api/event";
import { api, type Message, type NovelEntry } from "@/lib/tauri";
import { playChime } from "@/lib/chime";

// Cute flavor for the long ingest-before-first-token wait when writing a
// chapter. "Reading messages..." is index 0 so it's what the user sees first
// — it also happens to be what the model is literally doing at that moment.
const NOVELIST_LOADING_MESSAGES = [
  "Reading messages...",
  "Calibrating typewriter...",
  "Making tea...",
  "Getting the creative juices flowing...",
  "Checking notes...",
  "Lighting candles...",
  "Imagining details...",
  "Procrastinating...",
  "Sharpening pencils...",
  "Staring out the window...",
  "Consulting the muse...",
  "Rearranging index cards...",
  "Listening for the right voice...",
  "Pacing the study...",
  "Pouring another cup...",
  "Finding the first sentence...",
  "Cracking knuckles...",
  "Settling in...",
  "Dusting off the thesaurus...",
  "Remembering what happened...",
  "Flipping through the dictionary...",
  "Putting on the writing jumper...",
  "Closing the door...",
  "Tuning out the world...",
  "Cueing up the record player...",
  "Wrestling with adverbs...",
  "Petting the cat...",
  "Finding the right metaphor...",
  "Rereading the last chapter...",
  "Chasing a stray thought...",
  "Organizing the desk...",
  "Drawing the curtains...",
  "Cleaning the reading glasses...",
  "Biting the pencil...",
  "Conjuring the opening image...",
  "Listening to the rain...",
  "Weighing word choices...",
  "Letting the silence settle...",
  "Gathering the thread...",
  "Arguing with the narrator...",
];

// Markdown components for the novel text. Extends the default formatter
// with a decorative `<hr>` that renders as a large centered ornament —
// kept as a fallback in case the model emits a horizontal rule mid-section.
// Primary section dividers are handled by splitting novel content at the
// "\n\n* * *\n\n" divider string and rendering each section in its own
// <article> (so each section's first letter gets its own drop-cap).
const novelMarkdownComponents = {
  ...markdownComponents,
  hr: () => <NovelDivider />,
};

function NovelDivider() {
  return (
    <div
      className="text-center my-10 text-[5.625rem] leading-none text-amber-400/70 select-none"
      aria-hidden="true"
    >
      ❧
    </div>
  );
}

// Split novel prose on the backend's section divider and render each section
// in its own <article>. Passing cursor renders it at the end of the last
// section (during active streaming). endsWithDivider means the backend has
// emitted a divider but the next section hasn't started yet — we render an
// extra divider at the end so the parent can show the "preparing next
// section" spinner below it.
function NovelSections({
  content,
  articleClassName,
  cursor,
}: {
  content: string;
  articleClassName: string;
  cursor?: React.ReactNode;
}) {
  const parts = content.split("\n\n* * *\n\n");
  const endsWithDivider = parts.length > 0 && parts[parts.length - 1] === "";
  const sections = endsWithDivider ? parts.slice(0, -1) : parts;
  return (
    <>
      {sections.map((section, i) => {
        const isLast = i === sections.length - 1;
        return (
          <React.Fragment key={i}>
            {i > 0 && <NovelDivider />}
            <article className={articleClassName}>
              <Markdown components={novelMarkdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>
                {section}
              </Markdown>
              {cursor && isLast && !endsWithDivider && cursor}
            </article>
          </React.Fragment>
        );
      })}
      {endsWithDivider && <NovelDivider />}
    </>
  );
}

const NOVEL_ARTICLE_CLASS_LG =
  "prose prose-lg prose-invert max-w-none leading-[1.9] [--tw-prose-body:var(--color-foreground)] [--tw-prose-headings:var(--color-foreground)] [--tw-prose-bold:var(--color-foreground)] [--tw-prose-links:var(--color-primary)] first-letter:text-5xl first-letter:font-serif first-letter:font-bold first-letter:float-left first-letter:mr-2 first-letter:mt-1 first-letter:leading-none first-letter:text-amber-400";

const NOVEL_ARTICLE_CLASS_SM =
  "prose prose-sm prose-invert max-w-none leading-relaxed [--tw-prose-body:var(--color-foreground)] [--tw-prose-bold:var(--color-foreground)] first-letter:text-4xl first-letter:font-serif first-letter:font-bold first-letter:float-left first-letter:mr-2 first-letter:mt-1 first-letter:leading-none first-letter:text-amber-400";

interface Props {
  day: number;
  messages: Message[];
  portraits: Record<string, string>;
  characterColors: Record<string, string>;
  characterNames: Record<string, string>;
  userAvatarUrl: string;
  backgroundPortraits: string[];
  videoFiles: Record<string, string>;
  videoDataUrls: Record<string, string>;
  playVideo: (messageId: string) => void;
  playingVideo: string | null;
  setPlayingVideo: (v: string | null) => void;
  loopVideo: Record<string, boolean>;
  setLoopVideo: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
  /** Thread ID for novel entry lookups */
  threadId: string;
  /** API key for LLM calls */
  apiKey: string;
  /** Whether this is a group chat */
  isGroup: boolean;
  /** Pre-loaded novel entry for this day (if any) */
  novelEntry: NovelEntry | null;
  /** Callback when novel entry is saved or deleted */
  onNovelChange: () => void;
  notifyOnMessage?: boolean;
  /** In a 1:1 chat, assistant messages are saved with
   *  sender_character_id = null. Pass the active character's id here so the
   *  chat view can still resolve a portrait for them. Leave undefined for
   *  group chats (which populate sender_character_id per-message). */
  defaultCharacterId?: string;
}

export function DayPageSlide({
  day, messages, portraits, characterColors, characterNames,
  userAvatarUrl, backgroundPortraits, videoFiles, videoDataUrls, playVideo,
  playingVideo, setPlayingVideo, loopVideo, setLoopVideo, defaultCharacterId,
  threadId, apiKey, isGroup, novelEntry, onNovelChange, notifyOnMessage,
}: Props) {
  const [showNovelView, setShowNovelView] = useState(!!novelEntry);
  const [novelModalOpen, setNovelModalOpen] = useState(false);
  const [novelGenerating, setNovelGenerating] = useState(false);
  const [novelDraft, setNovelDraft] = useState("");
  const [novelTab, setNovelTab] = useState<"read" | "edit">("read");
  const [clearConfirmOpen, setClearConfirmOpen] = useState(false);
  const [regenerateConfirmOpen, setRegenerateConfirmOpen] = useState(false);
  // True during inter-section gaps — after a section finishes streaming but
  // before the next section's first token arrives. Drives the inline
  // "Writing next day part..." spinner.
  const [preparingSection, setPreparingSection] = useState(false);

  // Sync novel view with entry existence
  useEffect(() => {
    if (novelEntry) setShowNovelView(true);
    else setShowNovelView(false);
  }, [novelEntry]);

  const handleNovelize = async () => {
    setNovelGenerating(true);
    setNovelDraft("");
    setShowNovelView(true);
    setPreparingSection(false);

    let chimePlayed = false;
    const unlisten = await listen<string>("novel-token", (event) => {
      if (!chimePlayed && notifyOnMessage) { playChime(); chimePlayed = true; }
      setNovelDraft((prev) => prev + event.payload);
      // First token of any section — we're streaming again, clear the gap spinner.
      setPreparingSection(false);
    });
    // Backend emits novel-phase events to signal transitions between sections
    // (and beats-extraction sub-phases). While those are in flight we show the
    // "Writing next day part..." state inline.
    const unlistenPhase = await listen<{ phase: string; section_index?: number }>(
      "novel-phase",
      (event) => {
        const { phase, section_index } = event.payload ?? {};
        if (phase === "beats" || (phase === "section" && (section_index ?? 0) > 0)) {
          setPreparingSection(true);
        }
      },
    );

    try {
      const content = await api.generateNovelEntry(apiKey, threadId, day, isGroup);
      // Auto-save the final chapter so the inline view transitions seamlessly
      // from "streaming" to "saved novel entry" without a modal step.
      await api.saveNovelEntry(threadId, day, content);
      onNovelChange();
      setNovelDraft("");
    } catch (e) {
      setNovelDraft(`Error generating chapter: ${e}`);
    } finally {
      unlisten();
      unlistenPhase();
      setNovelGenerating(false);
      setPreparingSection(false);
    }
  };

  // Open the modal in edit mode with the saved chapter content loaded.
  const handleEdit = () => {
    if (!novelEntry) return;
    setNovelDraft(novelEntry.content);
    setNovelTab("edit");
    setNovelModalOpen(true);
  };

  const handleSave = async () => {
    await api.saveNovelEntry(threadId, day, novelDraft);
    setNovelModalOpen(false);
    onNovelChange();
  };

  const handleClear = async () => {
    await api.deleteNovelEntry(threadId, day);
    setClearConfirmOpen(false);
    setShowNovelView(false);
    onNovelChange();
  };

  const handleRegenerate = async () => {
    setRegenerateConfirmOpen(false);
    await api.deleteNovelEntry(threadId, day);
    onNovelChange();
    await handleNovelize();
  };

  // Collect illustrations for the day
  const dayIllustrations = messages.filter((m) => m.role === "illustration");

  return (
    <div className="w-full h-full flex flex-col bg-background rounded-t-2xl overflow-hidden relative">
      {/* Portrait background */}
      {backgroundPortraits.length > 0 && (
        <div className="absolute inset-0 z-0 pointer-events-none overflow-hidden flex">
          {backgroundPortraits.map((url, i) => (
            <div key={i} className="flex-1 relative">
              <img src={url} alt="" className="w-full h-full object-cover" />
            </div>
          ))}
          <div className="absolute inset-0 bg-background/65" />
        </div>
      )}

      {/* Sticky header */}
      <div className="flex-shrink-0 sticky top-0 z-10 bg-card/95 backdrop-blur-sm border-b border-border/30 px-6 py-3 relative flex items-center justify-center gap-3">
        <h2 className="text-lg font-bold text-foreground tracking-tight">Day {day}</h2>
        {novelEntry ? (
          <>
            <div className="inline-flex rounded-full overflow-hidden border border-amber-600/30">
              <button
                onClick={() => setShowNovelView(true)}
                className={`px-3 py-1 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5 ${
                  showNovelView
                    ? "bg-amber-600 text-white"
                    : "bg-amber-600/10 text-amber-400/80 hover:bg-amber-600/20 hover:text-amber-400"
                }`}
              >
                <BookText size={12} />
                Novel
              </button>
              <button
                onClick={() => setShowNovelView(false)}
                className={`px-3 py-1 text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5 border-l border-amber-600/30 ${
                  !showNovelView
                    ? "bg-amber-600 text-white"
                    : "bg-amber-600/10 text-amber-400/80 hover:bg-amber-600/20 hover:text-amber-400"
                }`}
              >
                <MessageSquare size={12} />
                Chat
              </button>
            </div>
            <button
              onClick={() => setRegenerateConfirmOpen(true)}
              disabled={!apiKey || novelGenerating}
              title="Regenerate chapter"
              className="px-2 py-1 text-xs font-medium rounded-full bg-amber-600/10 text-amber-400/80 hover:bg-amber-600/20 hover:text-amber-400 transition-colors cursor-pointer flex items-center gap-1 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <RotateCw size={12} />
            </button>
            <button
              onClick={handleEdit}
              disabled={novelGenerating}
              title="Edit chapter"
              className="px-2 py-1 text-xs font-medium rounded-full bg-amber-600/10 text-amber-400/80 hover:bg-amber-600/20 hover:text-amber-400 transition-colors cursor-pointer flex items-center gap-1 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              <Pencil size={12} />
            </button>
          </>
        ) : (
          <button
            onClick={handleNovelize}
            disabled={!apiKey || novelGenerating}
            className="px-3 py-1 text-xs font-medium rounded-full bg-primary/20 text-primary hover:bg-primary/30 transition-colors cursor-pointer flex items-center gap-1.5 disabled:opacity-60 disabled:cursor-not-allowed"
          >
            {novelGenerating ? <Loader2 size={12} className="animate-spin" /> : <Sparkles size={12} />}
            {novelGenerating ? "Novelizing" : "Novelize"}
          </button>
        )}
        {novelGenerating && (
          <span className="text-[11px] text-muted-foreground/80 italic">
            Novelizing… do not close gallery.
          </span>
        )}
      </div>

      {/* Content area */}
      {(showNovelView && novelEntry) || novelGenerating ? (
        /* ── Novel view: chapter + gallery side by side ── */
        <div className="flex-1 overflow-hidden relative z-[1] flex">
          {/* Chapter text */}
          <div className={`overflow-y-auto px-8 py-8 ${dayIllustrations.length > 0 ? "flex-1" : "w-full"}`}>
            <div className="max-w-prose mx-auto">
              {novelGenerating && !novelDraft ? (
                // Pre-first-token: centered spinner + cycling novelist messages.
                <div className="flex flex-col items-center justify-center py-24 gap-3">
                  <Loader2 size={28} className="animate-spin text-primary" />
                  <p className="text-sm text-muted-foreground">
                    <CyclingLoadingMessages messages={NOVELIST_LOADING_MESSAGES} />
                  </p>
                </div>
              ) : (
                <>
                  <NovelSections
                    content={novelGenerating ? novelDraft : (novelEntry?.content ?? "")}
                    articleClassName={NOVEL_ARTICLE_CLASS_LG}
                    cursor={
                      novelGenerating && !preparingSection && novelDraft ? (
                        <span className="inline-block w-1.5 h-5 bg-primary/60 animate-pulse ml-0.5 align-text-bottom" />
                      ) : null
                    }
                  />
                  {novelGenerating && preparingSection && (
                    // Between-section gap: backend is extracting beats or
                    // about to start the next section. Show the spinner +
                    // cycling messages with a header.
                    <div className="flex flex-col items-center justify-center py-10 gap-3">
                      <p className="text-sm text-muted-foreground font-medium">Writing next day part...</p>
                      <Loader2 size={24} className="animate-spin text-primary" />
                      <p className="text-xs text-muted-foreground">
                        <CyclingLoadingMessages messages={NOVELIST_LOADING_MESSAGES} />
                      </p>
                    </div>
                  )}
                  {!novelGenerating && novelEntry && (
                    <div className="mt-8 flex justify-center">
                      <button
                        onClick={() => setClearConfirmOpen(true)}
                        className="text-xs text-muted-foreground/50 hover:text-destructive transition-colors cursor-pointer flex items-center gap-1"
                      >
                        <Trash2 size={10} />
                        Clear Novel Entry
                      </button>
                    </div>
                  )}
                </>
              )}
            </div>
          </div>
          {/* Image gallery */}
          {dayIllustrations.length > 0 && (
            <div className="w-[320px] flex-shrink-0 overflow-y-auto border-l border-border/20 bg-black/20 p-3 space-y-3">
              {dayIllustrations.map((msg) => (
                <div key={msg.message_id} className="rounded-lg overflow-hidden">
                  <img
                    src={msg.content}
                    alt="Scene illustration"
                    loading="lazy"
                    className="w-full rounded-lg"
                  />
                </div>
              ))}
            </div>
          )}
        </div>
      ) : (
        /* ── Chat message view ── */
        <div className="flex-1 overflow-y-auto px-4 py-4 relative z-[1]">
          <div className="space-y-3 max-w-2xl mx-auto">
            {messages.map((msg, idx) => {
              const prevMsg = idx > 0 ? messages[idx - 1] : undefined;
              const timeDivider = <TimeDivider current={msg} previous={prevMsg} />;

              if (msg.role === "narrative") {
                return (
                  <React.Fragment key={msg.message_id}>
                    {timeDivider}
                    <div className="flex justify-center my-2">
                      <div className="max-w-[90%] rounded-xl px-5 py-3.5 text-sm leading-relaxed bg-gradient-to-br from-amber-950/40 to-amber-900/20 border border-amber-700/30 text-amber-100/90 italic backdrop-blur-sm">
                        <div className="flex items-center gap-1.5 mb-1.5 text-[10px] uppercase tracking-wider text-amber-500/70 font-semibold not-italic">
                          <BookOpen size={12} />
                          <span>Narrative</span>
                        </div>
                        <div className="prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:rgb(252,211,77,0.9)] [--tw-prose-bold:rgb(252,211,77)] [&_em]:italic">
                          <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(msg.content)}</Markdown>
                        </div>
                        <p className="text-[10px] mt-1.5 text-amber-500/50">
                          {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                        </p>
                      </div>
                    </div>
                  </React.Fragment>
                );
              }

              if (msg.role === "context") {
                return (
                  <React.Fragment key={msg.message_id}>
                    {timeDivider}
                    <div className="flex justify-center my-2">
                      <div className="max-w-[90%] rounded-xl px-5 py-3.5 text-sm leading-relaxed bg-gradient-to-br from-sky-950/40 to-sky-900/20 border border-sky-700/30 text-sky-100/90 backdrop-blur-sm">
                        <div className="flex items-center gap-1.5 mb-1.5 text-[10px] uppercase tracking-wider text-sky-500/70 font-semibold">
                          <Link2 size={12} />
                          <span>Cross-Chat Context</span>
                        </div>
                        <div className="prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:var(--color-sky-100)] [--tw-prose-bold:rgb(125,211,252)]">
                          <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(msg.content)}</Markdown>
                        </div>
                        <p className="text-[10px] mt-1.5 text-sky-500/50">
                          {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                        </p>
                      </div>
                    </div>
                  </React.Fragment>
                );
              }

              if (msg.role === "illustration") {
                const isPlaying = playingVideo === msg.message_id;
                const hasVideo = !!videoFiles[msg.message_id];
                const videoUrl = videoDataUrls[msg.message_id];
                return (
                  <React.Fragment key={msg.message_id}>
                    {timeDivider}
                    <div className="flex justify-center my-3">
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
                            className={`w-full rounded-lg ${isPlaying && videoUrl ? "invisible" : ""}`}
                          />
                          {isPlaying && videoUrl && (
                            <>
                              <video
                                src={videoUrl}
                                autoPlay
                                loop={!!loopVideo[msg.message_id]}
                                playsInline
                                className="absolute inset-2 w-[calc(100%-16px)] h-[calc(100%-16px)] object-contain rounded-lg"
                                onEnded={() => { if (!loopVideo[msg.message_id]) setPlayingVideo(null); }}
                              />
                              <button
                                onClick={() => setPlayingVideo(null)}
                                className="absolute bottom-4 right-4 w-8 h-8 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm opacity-0 group-hover/illus:opacity-100"
                              >
                                <span className="text-xs">&#9632;</span>
                              </button>
                            </>
                          )}
                          {!isPlaying && hasVideo && (
                            <button
                              onClick={() => playVideo(msg.message_id)}
                              className="absolute bottom-4 right-4 w-8 h-8 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-purple-600 transition-colors backdrop-blur-sm"
                            >
                              <span className="text-sm ml-0.5">&#9654;</span>
                            </button>
                          )}
                        </div>
                        <p className="text-[10px] px-4 pb-3 text-emerald-500/50">
                          {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                        </p>
                      </div>
                    </div>
                  </React.Fragment>
                );
              }

              const isUser = msg.role === "user";
              // 1:1 chats don't set sender_character_id on assistant rows —
              // fall back to the chat's default character so we can still
              // look up a portrait / color / name for them.
              const charId = msg.sender_character_id ?? defaultCharacterId ?? null;
              const portraitUrl = charId ? portraits[charId] : undefined;
              const avatarColor = charId ? characterColors[charId] : "#c4a882";
              const charName = charId ? characterNames[charId] : undefined;

              return (
                <React.Fragment key={msg.message_id}>
                  {timeDivider}
                  <div>
                    <div className={`flex items-end gap-2 ${isUser ? "justify-end" : "justify-start"}`}>
                      {!isUser && (
                        portraitUrl ? (
                          <img src={portraitUrl} alt="" className="w-20 h-20 rounded-full object-cover ring-1 ring-border flex-shrink-0 mb-1" />
                        ) : (
                          <span
                            className="w-20 h-20 rounded-full flex-shrink-0 mb-1 ring-1 ring-white/10"
                            style={{ backgroundColor: avatarColor }}
                          />
                        )
                      )}
                      <div
                        className={`rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${
                          isUser
                            ? "bg-primary text-primary-foreground rounded-br-md max-w-[80%]"
                            : "bg-secondary/40 text-secondary-foreground rounded-bl-md max-w-[80%] border border-border/30"
                        }`}
                      >
                        {!isUser && charName && (
                          <p className="text-[10px] font-semibold text-muted-foreground/70 mb-0.5">{charName}</p>
                        )}
                        <div className={`prose prose-sm max-w-none prose-p:my-1 prose-ul:my-1 prose-ol:my-1 prose-li:my-0.5 prose-headings:my-2 prose-pre:my-2 prose-blockquote:my-2 prose-hr:my-2 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_em]:italic ${
                          isUser
                            ? "[--tw-prose-body:var(--color-primary-foreground)] [--tw-prose-headings:var(--color-primary-foreground)] [--tw-prose-bold:var(--color-primary-foreground)] [--tw-prose-bullets:var(--color-primary-foreground)] [--tw-prose-counters:var(--color-primary-foreground)] [--tw-prose-code:var(--color-primary-foreground)] [--tw-prose-links:var(--color-primary-foreground)] [--tw-prose-quotes:var(--color-primary-foreground)] [--tw-prose-quote-borders:rgba(255,255,255,0.3)]"
                            : "[--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-headings:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)] [--tw-prose-bullets:var(--color-secondary-foreground)] [--tw-prose-counters:var(--color-secondary-foreground)] [--tw-prose-code:var(--color-secondary-foreground)] [--tw-prose-links:var(--color-primary)] [--tw-prose-quotes:var(--color-secondary-foreground)] [--tw-prose-quote-borders:var(--color-border)]"
                        }`}>
                          <Markdown components={markdownComponents} remarkPlugins={remarkPlugins} rehypePlugins={rehypePlugins}>{formatMessage(msg.content)}</Markdown>
                        </div>
                        <p className={`text-[10px] mt-1 ${isUser ? "text-primary-foreground/50" : "text-muted-foreground"}`}>
                          {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                        </p>
                      </div>
                      {isUser && userAvatarUrl && (
                        <img src={userAvatarUrl} alt="" className="w-20 h-20 rounded-full object-cover ring-1 ring-border flex-shrink-0 mb-1" />
                      )}
                    </div>
                  </div>
                </React.Fragment>
              );
            })}
          </div>
        </div>
      )}

      {/* ── Novelize modal ── */}
      <Dialog open={novelModalOpen} onClose={() => { if (!novelGenerating) setNovelModalOpen(false); }}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle>Novelize — Day {day}</DialogTitle>
          </DialogHeader>
          {!novelGenerating && (
            <div className="flex border-b border-border px-6">
              <button
                onClick={() => setNovelTab("read")}
                className={`px-4 py-2 text-sm font-medium transition-colors cursor-pointer ${
                  novelTab === "read" ? "text-foreground border-b-2 border-primary" : "text-muted-foreground hover:text-foreground"
                }`}
              >
                Read
              </button>
              <button
                onClick={() => setNovelTab("edit")}
                className={`px-4 py-2 text-sm font-medium transition-colors cursor-pointer ${
                  novelTab === "edit" ? "text-foreground border-b-2 border-primary" : "text-muted-foreground hover:text-foreground"
                }`}
              >
                Edit
              </button>
            </div>
          )}
          <DialogBody className="!p-0">
            {novelGenerating && !novelDraft ? (
              <div className="flex flex-col items-center justify-center py-20 gap-3">
                <Loader2 size={28} className="animate-spin text-primary" />
                <p className="text-sm text-muted-foreground">
                  <CyclingLoadingMessages messages={NOVELIST_LOADING_MESSAGES} />
                </p>
              </div>
            ) : novelTab === "read" || novelGenerating ? (
              <div className="max-h-[60vh] overflow-y-auto px-6 py-5">
                <NovelSections
                  content={novelDraft}
                  articleClassName={NOVEL_ARTICLE_CLASS_SM}
                  cursor={
                    novelGenerating ? (
                      <span className="inline-block w-1.5 h-4 bg-primary/60 animate-pulse ml-0.5 align-text-bottom" />
                    ) : null
                  }
                />
              </div>
            ) : (
              <div className="px-6 py-4">
                <textarea
                  value={novelDraft}
                  onChange={(e) => setNovelDraft(e.target.value)}
                  className="w-full min-h-[50vh] max-h-[60vh] resize-y rounded-lg border border-input bg-transparent px-4 py-3 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                />
              </div>
            )}
            {!novelGenerating && (
              <div className="flex justify-end gap-2 px-6 py-4 border-t border-border">
                <Button variant="ghost" size="sm" onClick={() => setNovelModalOpen(false)}>
                  Cancel
                </Button>
                <Button size="sm" onClick={handleSave} disabled={!novelDraft.trim()}>
                  Save
                </Button>
              </div>
            )}
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* ── Clear confirmation ── */}
      <Dialog open={clearConfirmOpen} onClose={() => setClearConfirmOpen(false)}>
        <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50 max-w-xs mx-auto">
          <div className="flex items-center gap-2">
            <Trash2 size={18} className="text-destructive" />
            <h3 className="font-semibold">Clear Novel Entry</h3>
          </div>
          <p className="text-sm text-muted-foreground">
            This will permanently delete the novel chapter for Day {day}.
          </p>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setClearConfirmOpen(false)}>
              Cancel
            </Button>
            <Button variant="destructive" size="sm" onClick={handleClear}>
              Clear
            </Button>
          </div>
        </div>
      </Dialog>

      {/* ── Regenerate confirmation ── */}
      <Dialog open={regenerateConfirmOpen} onClose={() => setRegenerateConfirmOpen(false)}>
        <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50 max-w-xs mx-auto">
          <div className="flex items-center gap-2">
            <RotateCw size={18} className="text-amber-500" />
            <h3 className="font-semibold">Regenerate Chapter</h3>
          </div>
          <p className="text-sm text-muted-foreground">
            This will clear the current chapter for Day {day} and write a new one from the day's messages.
          </p>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setRegenerateConfirmOpen(false)}>
              Cancel
            </Button>
            <Button size="sm" className="bg-amber-600 hover:bg-amber-700 text-white" onClick={handleRegenerate}>
              Regenerate
            </Button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}

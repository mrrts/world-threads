import React from "react";
import Markdown from "react-markdown";
import { BookOpen, Link2, Image } from "lucide-react";
import { formatMessage, markdownComponents } from "./formatMessage";
import type { Message } from "@/lib/tauri";

interface Props {
  day: number;
  timePart: string | null;
  messages: Message[];
  /** Character portrait URLs keyed by character_id */
  portraits: Record<string, string>;
  /** Character colors keyed by character_id */
  characterColors: Record<string, string>;
  /** Character display names keyed by character_id */
  characterNames: Record<string, string>;
  /** User avatar data URL */
  userAvatarUrl: string;
  /** Portrait URLs for the background (tiled side by side) */
  backgroundPortraits: string[];
  /** Video files keyed by message_id */
  videoFiles: Record<string, string>;
  /** Video data URLs keyed by message_id */
  videoDataUrls: Record<string, string>;
  /** Play video callback */
  playVideo: (messageId: string) => void;
  /** Currently playing video */
  playingVideo: string | null;
  setPlayingVideo: (v: string | null) => void;
  loopVideo: Record<string, boolean>;
  setLoopVideo: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
}

export function DayPageSlide({
  day, timePart, messages, portraits, characterColors, characterNames,
  userAvatarUrl, backgroundPortraits, videoFiles, videoDataUrls, playVideo,
  playingVideo, setPlayingVideo, loopVideo, setLoopVideo,
}: Props) {
  // Group consecutive time-of-day sections
  let lastTime: string | null = null;

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
      <div className="flex-shrink-0 sticky top-0 z-10 bg-card/95 backdrop-blur-sm border-b border-border/30 px-6 py-3 relative">
        <h2 className="text-lg font-bold text-foreground tracking-tight text-center">
          Day {day}{timePart ? ` · ${timePart.split(" ").map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase()).join(" ")}` : ""}
        </h2>
      </div>
      {/* Scrollable message list */}
      <div className="flex-1 overflow-y-auto px-4 py-4 relative z-[1]">
        <div className="space-y-3 max-w-2xl mx-auto">
          {messages.map((msg, idx) => {
            // Time divider
            let timeDivider: React.ReactNode = null;
            if (msg.world_time && msg.world_time !== lastTime) {
              const formatted = msg.world_time
                .split(" ")
                .map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
                .join(" ");
              timeDivider = (
                <div className="flex items-center gap-4 my-5 px-2">
                  <div className="flex-1 h-[1.5px] bg-zinc-700/50" />
                  <span className="text-xs font-semibold text-zinc-500 uppercase tracking-wider px-3 py-1 rounded-full bg-zinc-800/60 border border-zinc-700/40">
                    {formatted}
                  </span>
                  <div className="flex-1 h-[1.5px] bg-zinc-700/50" />
                </div>
              );
              lastTime = msg.world_time;
            }

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
                      <div className="prose prose-sm max-w-none prose-p:my-1 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [--tw-prose-body:rgb(252,211,77,0.9)] [--tw-prose-bold:rgb(252,211,77)] [&_em]:italic [&_em]:block [&_em]:border-l-2 [&_em]:border-current/20 [&_em]:pl-3 [&_em]:my-1.5 [&_em]:opacity-80">
                        <Markdown components={markdownComponents}>{formatMessage(msg.content)}</Markdown>
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
                        <Markdown components={markdownComponents}>{formatMessage(msg.content)}</Markdown>
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

            // User / Assistant messages
            const isUser = msg.role === "user";
            const charId = msg.sender_character_id;
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
                        <img src={portraitUrl} alt="" className="w-10 h-10 rounded-full object-cover ring-1 ring-border flex-shrink-0 mb-1" />
                      ) : (
                        <span
                          className="w-10 h-10 rounded-full flex-shrink-0 mb-1 ring-1 ring-white/10"
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
                      <div className={`prose prose-sm max-w-none prose-p:my-1 prose-ul:my-1 prose-ol:my-1 prose-li:my-0.5 prose-headings:my-2 prose-pre:my-2 prose-blockquote:my-2 prose-hr:my-2 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_em]:italic [&_em]:block [&_em]:border-l-2 [&_em]:border-current/20 [&_em]:pl-3 [&_em]:my-1.5 [&_em]:opacity-80 ${
                        isUser
                          ? "[--tw-prose-body:var(--color-primary-foreground)] [--tw-prose-headings:var(--color-primary-foreground)] [--tw-prose-bold:var(--color-primary-foreground)] [--tw-prose-bullets:var(--color-primary-foreground)] [--tw-prose-counters:var(--color-primary-foreground)] [--tw-prose-code:var(--color-primary-foreground)] [--tw-prose-links:var(--color-primary-foreground)] [--tw-prose-quotes:var(--color-primary-foreground)] [--tw-prose-quote-borders:rgba(255,255,255,0.3)]"
                          : "[--tw-prose-body:var(--color-secondary-foreground)] [--tw-prose-headings:var(--color-secondary-foreground)] [--tw-prose-bold:var(--color-secondary-foreground)] [--tw-prose-bullets:var(--color-secondary-foreground)] [--tw-prose-counters:var(--color-secondary-foreground)] [--tw-prose-code:var(--color-secondary-foreground)] [--tw-prose-links:var(--color-primary)] [--tw-prose-quotes:var(--color-secondary-foreground)] [--tw-prose-quote-borders:var(--color-border)]"
                      }`}>
                        <Markdown components={markdownComponents}>{formatMessage(msg.content)}</Markdown>
                      </div>
                      <p className={`text-[10px] mt-1 ${isUser ? "text-primary-foreground/50" : "text-muted-foreground"}`}>
                        {new Date(msg.created_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                      </p>
                    </div>
                    {isUser && userAvatarUrl && (
                      <img src={userAvatarUrl} alt="" className="w-10 h-10 rounded-full object-cover ring-1 ring-border flex-shrink-0 mb-1" />
                    )}
                  </div>
                </div>
              </React.Fragment>
            );
          })}
        </div>
      </div>
    </div>
  );
}

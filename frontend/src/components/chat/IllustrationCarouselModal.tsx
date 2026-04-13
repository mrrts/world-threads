import React, { useState, useMemo } from "react";
import { Dialog } from "@/components/ui/dialog";
import { X, Check, Download, Crosshair, ChevronLeft, ChevronRight, Play, Pause, Square, BookOpen, Image } from "lucide-react";
import { api, type Message } from "@/lib/tauri";
import type { useSlideshow } from "@/hooks/use-slideshow";
import { DayPageSlide } from "./DayPageSlide";

// ─── Types ──────────────────────────────────────────────────────────────────

export type CarouselSlide =
  | { type: "illustration"; id: string; content: string }
  | { type: "day-page"; day: number; timePart: string | null; messages: Message[] };

// ─── Props ──────────────────────────────────────────────────────────────────

export interface IllustrationCarouselModalProps {
  illustrationModalId: string | null;
  setIllustrationModalId: (id: string | null) => void;
  modalSelectedId: string | null;
  setModalSelectedId: (id: string | null) => void;
  modalPlayingVideo: boolean;
  setModalPlayingVideo: (v: boolean) => void;
  modalImageLoading: boolean;
  setModalImageLoading: (v: boolean) => void;
  modalIllustrations: Array<{ id: string; content: string }>;
  videoFiles: Record<string, string>;
  videoDataUrls: Record<string, string>;
  setVideoDataUrls: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  loadVideoBlobUrl: (videoFile: string) => Promise<string>;
  downloadedId: string | null;
  setDownloadedId: (id: string | null) => void;
  modalSlideshow: ReturnType<typeof useSlideshow>;
  fallbackIllustrations: Array<{ id: string; content: string }>;
  /** All messages for the chat — used to build day pages */
  allMessages: Message[];
  /** Character portrait URLs keyed by character_id */
  portraits: Record<string, string>;
  /** Character colors keyed by character_id */
  characterColors: Record<string, string>;
  /** Character display names keyed by character_id */
  characterNames: Record<string, string>;
  /** User avatar URL */
  userAvatarUrl: string;
  /** Portrait URLs for day page backgrounds */
  backgroundPortraits: string[];
  /** Play video in day page */
  playVideo: (messageId: string) => void;
  playingVideo: string | null;
  setPlayingVideo: (v: string | null) => void;
  loopVideo: Record<string, boolean>;
  setLoopVideo: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/** Composite key for day + time-of-day grouping */
function dayTimeKey(day: number, time: string | null): string {
  return `${day}::${time ?? ""}`;
}

/** Build interleaved slide list: day-part page before first illustration of each day-part.
 *  Every day-part gets a page regardless of whether it has illustrations. */
function buildSlides(
  illustrations: Array<{ id: string; content: string }>,
  allMessages: Message[],
): CarouselSlide[] {
  // Group messages by day+time composite key, preserving insertion order
  const partMap = new Map<string, { day: number; time: string | null; messages: Message[] }>();
  const illustrationParts = new Map<string, string>(); // message_id → dayTimeKey

  for (const msg of allMessages) {
    if (msg.world_day == null) continue;
    const key = dayTimeKey(msg.world_day, msg.world_time ?? null);
    if (!partMap.has(key)) partMap.set(key, { day: msg.world_day, time: msg.world_time ?? null, messages: [] });
    partMap.get(key)!.messages.push(msg);
    if (msg.role === "illustration") {
      illustrationParts.set(msg.message_id, key);
    }
  }

  // Sort parts by day, then by chronological order of first message
  const sortedParts = [...partMap.entries()].sort((a, b) => {
    if (a[1].day !== b[1].day) return a[1].day - b[1].day;
    const aFirst = a[1].messages[0]?.created_at ?? "";
    const bFirst = b[1].messages[0]?.created_at ?? "";
    return aFirst.localeCompare(bFirst);
  });

  // Track which illustrations belong to which day-part
  const illusByPart = new Map<string, Array<{ id: string; content: string }>>();
  for (const illus of illustrations) {
    const key = illustrationParts.get(illus.id);
    if (key != null) {
      if (!illusByPart.has(key)) illusByPart.set(key, []);
      illusByPart.get(key)!.push(illus);
    }
  }

  // Illustrations with no day assigned (predate the day system) — come before day pages
  const orphanIllustrations = illustrations.filter(
    (i) => !illustrationParts.has(i.id),
  );

  const slides: CarouselSlide[] = [];

  // Orphan illustrations first (they precede the day system)
  for (const illus of orphanIllustrations) {
    slides.push({ type: "illustration", id: illus.id, content: illus.content });
  }

  for (const [key, part] of sortedParts) {
    // Day-part page
    slides.push({ type: "day-page", day: part.day, timePart: part.time, messages: part.messages });
    // Illustrations for this day-part
    const partIllus = illusByPart.get(key) ?? [];
    for (const illus of partIllus) {
      slides.push({ type: "illustration", id: illus.id, content: illus.content });
    }
  }

  return slides;
}

// ─── Component ──────────────────────────────────────────────────────────────

export function IllustrationCarouselModal({
  illustrationModalId,
  setIllustrationModalId,
  modalSelectedId,
  setModalSelectedId,
  modalPlayingVideo,
  setModalPlayingVideo,
  modalImageLoading,
  setModalImageLoading,
  modalIllustrations,
  videoFiles,
  videoDataUrls,
  setVideoDataUrls,
  loadVideoBlobUrl,
  downloadedId,
  setDownloadedId,
  modalSlideshow,
  fallbackIllustrations,
  allMessages,
  portraits,
  characterColors,
  characterNames,
  userAvatarUrl,
  backgroundPortraits,
  playVideo,
  playingVideo,
  setPlayingVideo,
  loopVideo,
  setLoopVideo,
}: IllustrationCarouselModalProps) {
  const [showDayPages, setShowDayPages] = useState(true);

  const allIllustrations = modalIllustrations.length > 0
    ? modalIllustrations
    : fallbackIllustrations;

  // Build the full slide list (day pages + illustrations)
  const mixedSlides = useMemo(
    () => buildSlides(allIllustrations, allMessages),
    [allIllustrations, allMessages],
  );

  // The active slide list depends on the toggle
  const slides: CarouselSlide[] = useMemo(() => {
    if (showDayPages && allMessages.length > 0) return mixedSlides;
    return allIllustrations.map((i) => ({ type: "illustration" as const, ...i }));
  }, [showDayPages, mixedSlides, allIllustrations, allMessages.length]);

  if (!illustrationModalId) return null;

  // Find the current slide index based on modalSelectedId
  const selId = modalSelectedId ?? illustrationModalId;
  let currentSlideIdx = slides.findIndex(
    (s) => s.type === "illustration" && s.id === selId,
  );
  if (currentSlideIdx < 0) currentSlideIdx = 0;
  const currentSlide = slides[currentSlideIdx];

  if (!currentSlide) return null;

  const isIllustration = currentSlide.type === "illustration";
  const isDayPage = currentSlide.type === "day-page";

  const selectedItem = isIllustration
    ? allIllustrations.find((i) => i.id === currentSlide.id)
    : null;

  const modalVideoFile = isIllustration ? videoFiles[currentSlide.id] : undefined;
  const modalVideoUrl = isIllustration ? videoDataUrls[currentSlide.id] : undefined;

  const closeModal = () => {
    setIllustrationModalId(null);
    setModalPlayingVideo(false);
    if (modalSlideshow.active) modalSlideshow.toggle();
  };

  const navigateTo = (idx: number) => {
    const target = slides[idx];
    if (!target) return;
    if (target.type === "illustration") {
      setModalSelectedId(target.id);
      setModalImageLoading(true);
    } else {
      // For day pages, set a synthetic id so we know which slide we're on
      setModalSelectedId(`__daypart_${target.day}_${target.timePart ?? ""}`);
      setModalImageLoading(false);
    }
    setModalPlayingVideo(false);
  };

  const goNext = () => navigateTo(currentSlideIdx >= slides.length - 1 ? 0 : currentSlideIdx + 1);
  const goPrev = () => navigateTo(currentSlideIdx <= 0 ? slides.length - 1 : currentSlideIdx - 1);

  // Override slide index finding for day pages (synthetic IDs)
  if (isDayPage && modalSelectedId?.startsWith("__daypart_")) {
    // Already on the right slide via the slides array lookup
  }

  // Recalculate index for day-page synthetic IDs
  const resolvedIdx = (() => {
    if (modalSelectedId?.startsWith("__daypart_")) {
      const rest = modalSelectedId.replace("__daypart_", "");
      const underIdx = rest.indexOf("_");
      const dayNum = parseInt(rest.slice(0, underIdx), 10);
      const timePart = rest.slice(underIdx + 1) || null;
      return slides.findIndex((s) => s.type === "day-page" && s.day === dayNum && s.timePart === timePart);
    }
    return currentSlideIdx;
  })();
  const activeIdx = resolvedIdx >= 0 ? resolvedIdx : currentSlideIdx;
  const activeSlide = slides[activeIdx] ?? currentSlide;

  return (
    <Dialog open onClose={closeModal} className="max-w-[90vw]">
      <div className="flex flex-col max-h-[90vh]">
        <div className="relative flex items-center justify-center min-h-0 flex-1 overflow-hidden group/modal">
          {/* Day Page view */}
          {activeSlide.type === "day-page" ? (
            <div className="w-full" style={{ height: "75vh" }}>
              <DayPageSlide
                day={activeSlide.day}
                timePart={activeSlide.timePart}
                messages={activeSlide.messages}
                portraits={portraits}
                characterColors={characterColors}
                characterNames={characterNames}
                backgroundPortraits={backgroundPortraits}
                userAvatarUrl={userAvatarUrl}
                videoFiles={videoFiles}
                videoDataUrls={videoDataUrls}
                playVideo={playVideo}
                playingVideo={playingVideo}
                setPlayingVideo={setPlayingVideo}
                loopVideo={loopVideo}
                setLoopVideo={setLoopVideo}
              />
            </div>
          ) : (
            <>
              {/* Illustration view */}
              {modalImageLoading && !modalPlayingVideo && (
                <div className="absolute inset-0 flex items-center justify-center z-10">
                  <div className="animate-spin w-6 h-6 border-2 border-white/20 border-t-white rounded-full" />
                </div>
              )}
              {modalPlayingVideo && modalVideoUrl ? (
                <video
                  key={`modal-video-${activeSlide.id}`}
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
              ) : selectedItem ? (
                <img
                  key={`modal-img-${activeSlide.id}`}
                  src={selectedItem.content}
                  alt="Illustration"
                  className={`max-w-full max-h-[75vh] object-contain rounded-t-2xl ${modalImageLoading ? "opacity-0" : "opacity-100"} transition-opacity`}
                  onLoad={() => setModalImageLoading(false)}
                />
              ) : null}
            </>
          )}

          {/* Close button */}
          <button
            onClick={closeModal}
            className="absolute top-3 right-3 z-20 w-8 h-8 flex items-center justify-center rounded-full bg-black/50 text-white hover:bg-black/70 transition-colors cursor-pointer backdrop-blur-sm"
          >
            <X size={16} />
          </button>

          {/* Top-left controls: download, go-to, slideshow, day pages toggle */}
          <div className="absolute top-3 left-3 z-20 flex gap-1.5 opacity-0 group-hover/modal:opacity-100 transition-opacity">
            {/* Download (illustrations only) */}
            {activeSlide.type === "illustration" && (
              <div className="relative group/mdl-dl">
                <button
                  onClick={async () => {
                    await api.downloadIllustration(activeSlide.id);
                    setDownloadedId(activeSlide.id);
                    setTimeout(() => setDownloadedId(null), 1500);
                  }}
                  className="w-8 h-8 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm"
                >
                  {downloadedId === activeSlide.id ? <Check size={14} /> : <Download size={14} />}
                </button>
                <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mdl-dl:opacity-100 pointer-events-none transition-opacity">{downloadedId === activeSlide.id ? "Saved!" : "Download"}</span>
              </div>
            )}
            {/* Go to image (illustrations only) */}
            {activeSlide.type === "illustration" && (
              <div className="relative group/mdl-goto">
                <button
                  onClick={async () => {
                    closeModal();
                    await new Promise((r) => setTimeout(r, 100));
                    const el = document.querySelector(`[data-message-id="${activeSlide.id}"]`);
                    if (el) el.scrollIntoView({ behavior: "smooth", block: "center" });
                  }}
                  className="w-8 h-8 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-colors backdrop-blur-sm"
                >
                  <Crosshair size={14} />
                </button>
                <span className="absolute top-full left-1/2 -translate-x-1/2 mt-1.5 px-2 py-0.5 text-[10px] font-medium text-white bg-black rounded-md shadow-lg whitespace-nowrap opacity-0 group-hover/mdl-goto:opacity-100 pointer-events-none transition-opacity">Go to Image</span>
              </div>
            )}
            {/* Slideshow */}
            {allIllustrations.length > 1 && (
              <div className="relative group/mdl-ss">
                <button
                  onClick={() => {
                    if (!modalSlideshow.active && activeSlide.type === "illustration") {
                      modalSlideshow.jumpTo(activeSlide.id);
                    }
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
            {/* Day Pages toggle */}
            {allMessages.length > 0 && (
              <div className="flex rounded-full overflow-hidden border border-white/20 backdrop-blur-sm">
                <button
                  onClick={() => setShowDayPages(true)}
                  className={`px-2.5 py-1.5 text-[10px] font-medium transition-colors cursor-pointer flex items-center gap-1 ${
                    showDayPages
                      ? "bg-primary/80 text-white"
                      : "bg-black/50 text-white/60 hover:text-white hover:bg-black/70"
                  }`}
                >
                  <BookOpen size={10} />
                  Day Pages
                </button>
                <button
                  onClick={() => setShowDayPages(false)}
                  className={`px-2.5 py-1.5 text-[10px] font-medium transition-colors cursor-pointer flex items-center gap-1 ${
                    !showDayPages
                      ? "bg-primary/80 text-white"
                      : "bg-black/50 text-white/60 hover:text-white hover:bg-black/70"
                  }`}
                >
                  <Image size={10} />
                  Images Only
                </button>
              </div>
            )}
          </div>

          {/* Video play/stop buttons (illustrations only) */}
          {activeSlide.type === "illustration" && modalVideoFile && !modalPlayingVideo && !modalSlideshow.active && (
            <button
              onClick={async () => {
                if (!modalVideoUrl) {
                  try {
                    const url = await loadVideoBlobUrl(modalVideoFile);
                    setVideoDataUrls((prev) => ({ ...prev, [activeSlide.id]: url }));
                  } catch { return; }
                }
                setModalPlayingVideo(true);
              }}
              className="absolute bottom-4 right-4 z-20 w-12 h-12 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-purple-600 transition-colors backdrop-blur-sm"
            >
              <span className="text-xl ml-0.5">&#9654;</span>
            </button>
          )}
          {activeSlide.type === "illustration" && modalPlayingVideo && !modalSlideshow.active && (
            <button
              onClick={() => setModalPlayingVideo(false)}
              className="absolute bottom-4 right-4 z-20 w-12 h-12 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm"
            >
              <Square size={16} fill="white" />
            </button>
          )}

          {/* Navigation arrows */}
          {slides.length > 1 && !modalSlideshow.active && (<>
            <button
              onClick={goPrev}
              className="absolute left-2 top-1/2 -translate-y-1/2 z-20 w-10 h-10 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-all backdrop-blur-sm opacity-0 group-hover/modal:opacity-100"
            >
              <ChevronLeft size={20} />
            </button>
            <button
              onClick={goNext}
              className="absolute right-2 top-1/2 -translate-y-1/2 z-20 w-10 h-10 rounded-full bg-black/50 text-white flex items-center justify-center cursor-pointer hover:bg-black/70 transition-all backdrop-blur-sm opacity-0 group-hover/modal:opacity-100"
            >
              <ChevronRight size={20} />
            </button>
          </>)}

          {/* Slideshow progress bar */}
          {modalSlideshow.active && (
            <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/10 z-30">
              <div
                className="h-full bg-primary transition-none"
                style={{ width: `${modalSlideshow.progress * 100}%` }}
              />
            </div>
          )}
        </div>

        {/* Thumbnail strip */}
        {slides.length > 1 && (
          <div className="flex-shrink-0 bg-card/80 backdrop-blur-sm rounded-b-2xl px-3 py-2 border-t border-border/30">
            <div className="flex gap-1.5 overflow-x-auto scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]">
              {slides.map((slide, idx) => {
                const isActive = idx === activeIdx;
                if (slide.type === "day-page") {
                  if (!slide.timePart) return null;
                  const timeLabel = slide.timePart.split(" ").map((w) => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase()).join(" ");
                  return (
                    <button
                      key={`day-${slide.day}-${slide.timePart}`}
                      ref={isActive ? (el) => {
                        if (!el) return;
                        const c = el.parentElement;
                        if (c) c.scrollTo({ left: el.offsetLeft - c.offsetWidth / 2 + el.offsetWidth / 2, behavior: "smooth" });
                      } : undefined}
                      onClick={() => {
                        if (modalSlideshow.active) return;
                        navigateTo(idx);
                      }}
                      className={`relative flex-shrink-0 w-16 h-11 rounded-lg overflow-hidden transition-all cursor-pointer flex items-center justify-center ${
                        isActive
                          ? "ring-2 ring-primary ring-offset-1 ring-offset-card bg-card"
                          : "ring-1 ring-border opacity-60 hover:opacity-100 bg-card/60"
                      }`}
                    >
                      <div className="text-center px-0.5">
                        <span className="text-[9px] font-bold text-muted-foreground leading-tight block">Day {slide.day}</span>
                        <span className="text-[8px] text-muted-foreground/70 leading-tight block">{timeLabel}</span>
                      </div>
                    </button>
                  );
                }
                return (
                  <button
                    key={slide.id}
                    ref={isActive ? (el) => {
                      if (!el) return;
                      const c = el.parentElement;
                      if (c) c.scrollTo({ left: el.offsetLeft - c.offsetWidth / 2 + el.offsetWidth / 2, behavior: "smooth" });
                    } : undefined}
                    onClick={() => {
                      if (modalSlideshow.active) {
                        modalSlideshow.jumpTo(slide.id);
                      } else {
                        navigateTo(idx);
                      }
                    }}
                    className={`relative flex-shrink-0 w-16 h-11 rounded-lg overflow-hidden transition-all cursor-pointer ${
                      isActive
                        ? "ring-2 ring-primary ring-offset-1 ring-offset-card"
                        : "ring-1 ring-border opacity-60 hover:opacity-100"
                    }`}
                  >
                    <img src={slide.content} alt="" className="w-full h-full object-cover" />
                    {videoFiles[slide.id] && (
                      <div className="absolute bottom-0.5 right-0.5 w-3.5 h-3.5 rounded-full bg-purple-600 flex items-center justify-center">
                        <span className="text-white text-[6px]">&#9654;</span>
                      </div>
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </Dialog>
  );
}

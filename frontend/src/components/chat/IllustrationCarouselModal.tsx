import React from "react";
import { Dialog } from "@/components/ui/dialog";
import { X, Check, Download, Crosshair, ChevronLeft, ChevronRight, Play, Pause, Square } from "lucide-react";
import { api } from "@/lib/tauri";
import type { useSlideshow } from "@/hooks/use-slideshow";

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
}

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
}: IllustrationCarouselModalProps) {
  if (!illustrationModalId) return null;

  const selId = modalSelectedId ?? illustrationModalId;
  const allIllustrations = modalIllustrations.length > 0
    ? modalIllustrations
    : fallbackIllustrations;
  const selectedItem = allIllustrations.find((i) => i.id === selId);
  if (!selectedItem) return null;

  const modalVideoFile = videoFiles[selId];
  const modalVideoUrl = videoDataUrls[selId];

  const closeModal = () => {
    setIllustrationModalId(null);
    setModalPlayingVideo(false);
    if (modalSlideshow.active) modalSlideshow.toggle();
  };

  return (
    <Dialog open onClose={closeModal} className="max-w-[90vw]">
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
              loop={!modalSlideshow.active}
              playsInline
              className="max-w-full max-h-[75vh] object-contain rounded-t-2xl"
              onTimeUpdate={modalSlideshow.active ? (e) => {
                const v = e.currentTarget;
                modalSlideshow.onVideoTimeUpdate(v.currentTime, v.duration);
              } : undefined}
              onEnded={modalSlideshow.active ? modalSlideshow.onVideoEnded : undefined}
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
            onClick={closeModal}
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
                  closeModal();
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
            {allIllustrations.length > 1 && (
              <div className="relative group/mdl-ss">
                <button
                  onClick={() => {
                    if (!modalSlideshow.active) modalSlideshow.jumpTo(selId);
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
          </div>
          {modalVideoFile && !modalPlayingVideo && !modalSlideshow.active && (
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
          {modalPlayingVideo && !modalSlideshow.active && (
            <button
              onClick={() => setModalPlayingVideo(false)}
              className="absolute bottom-4 right-4 z-20 w-12 h-12 rounded-full bg-black/70 text-white flex items-center justify-center cursor-pointer hover:bg-red-600 transition-colors backdrop-blur-sm"
            >
              <Square size={16} fill="white" />
            </button>
          )}
          {allIllustrations.length > 1 && !modalSlideshow.active && (<>
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
        {allIllustrations.length > 1 && (
          <div className="flex-shrink-0 bg-card/80 backdrop-blur-sm rounded-b-2xl px-3 py-2 border-t border-border/30">
            <div className="flex gap-1.5 overflow-x-auto scrollbar-none [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none]">
              {allIllustrations.map((illus) => (
                <button
                  key={illus.id}
                  ref={illus.id === selId ? (el) => {
                    if (!el) return;
                    const c = el.parentElement;
                    if (c) c.scrollTo({ left: el.offsetLeft - c.offsetWidth / 2 + el.offsetWidth / 2, behavior: "smooth" });
                  } : undefined}
                  onClick={() => {
                    if (modalSlideshow.active) {
                      modalSlideshow.jumpTo(illus.id);
                    } else {
                      setModalSelectedId(illus.id);
                      setModalImageLoading(true);
                      setModalPlayingVideo(false);
                    }
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
}

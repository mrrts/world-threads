import { useState, useEffect, useCallback, useRef } from "react";

export interface SlideshowSlide {
  illustrationId: string;
  type: "image" | "video";
}

interface UseSlideshowOptions {
  illustrations: Array<{ id: string; data_url: string }>;
  videoDataUrls: Record<string, string>;
  videoFiles: Record<string, string>;
  /** Called with (illustrationId, videoFilePath). Should load the video and store in videoDataUrls[illustrationId]. */
  loadVideoUrl: (illustrationId: string, videoFile: string) => Promise<void>;
  imageDuration?: number; // ms, default 8000
}

export function useSlideshow({
  illustrations,
  videoDataUrls,
  videoFiles,
  loadVideoUrl,
  imageDuration = 8000,
}: UseSlideshowOptions) {
  const [active, setActive] = useState(false);
  const [slideIndex, setSlideIndex] = useState(0);
  const [progress, setProgress] = useState(0); // 0-1 for progress bar
  const startRef = useRef(0);
  const rafRef = useRef(0);
  const loadingRef = useRef<Set<string>>(new Set());

  // Build flat slide list: video if available, otherwise image
  const slides: SlideshowSlide[] = [];
  for (const illus of illustrations) {
    if (videoFiles[illus.id]) {
      slides.push({ illustrationId: illus.id, type: "video" });
    } else {
      slides.push({ illustrationId: illus.id, type: "image" });
    }
  }

  const currentSlide = slides[slideIndex] ?? null;

  const advance = useCallback(() => {
    setSlideIndex((i) => (i + 1) % (slides.length || 1));
    setProgress(0);
  }, [slides.length]);

  // Preload video for the NEXT video slide (not all at once)
  useEffect(() => {
    if (!active) return;
    // Find the next video slide from current position
    for (let offset = 1; offset <= slides.length; offset++) {
      const idx = (slideIndex + offset) % slides.length;
      const slide = slides[idx];
      if (slide?.type === "video") {
        const id = slide.illustrationId;
        if (videoFiles[id] && !videoDataUrls[id] && !loadingRef.current.has(id)) {
          loadingRef.current.add(id);
          loadVideoUrl(id, videoFiles[id]).finally(() => loadingRef.current.delete(id));
        }
        break; // only preload the next one
      }
    }
  }, [active, slideIndex, slides, videoFiles, videoDataUrls, loadVideoUrl]);

  // If current slide is a video that hasn't loaded yet, load it now
  useEffect(() => {
    if (!active || !currentSlide || currentSlide.type !== "video") return;
    const id = currentSlide.illustrationId;
    if (videoFiles[id] && !videoDataUrls[id] && !loadingRef.current.has(id)) {
      loadingRef.current.add(id);
      loadVideoUrl(id, videoFiles[id]).finally(() => loadingRef.current.delete(id));
    }
  }, [active, slideIndex, currentSlide, videoFiles, videoDataUrls, loadVideoUrl]);

  // Image timer with animated progress
  useEffect(() => {
    if (!active || !currentSlide || currentSlide.type !== "image") {
      if (active && currentSlide?.type === "video") {
        // Don't reset progress for video — it's driven by onTimeUpdate
      } else {
        setProgress(0);
      }
      return;
    }

    startRef.current = performance.now();

    const tick = () => {
      const elapsed = performance.now() - startRef.current;
      const p = Math.min(elapsed / imageDuration, 1);
      setProgress(p);
      if (p >= 1) {
        advance();
      } else {
        rafRef.current = requestAnimationFrame(tick);
      }
    };

    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, [active, slideIndex, currentSlide?.type, imageDuration, advance]);

  // Video: progress is driven externally via onTimeUpdate, auto-advance via onEnded
  const onVideoTimeUpdate = useCallback(
    (current: number, duration: number) => {
      if (active && duration > 0) {
        setProgress(current / duration);
      }
    },
    [active],
  );

  const onVideoEnded = useCallback(() => {
    if (active) advance();
  }, [active, advance]);

  const toggle = useCallback(() => {
    setActive((prev) => {
      if (!prev) setProgress(0);
      return !prev;
    });
  }, []);

  // Jump to a specific illustration (by id) — finds the image slide for it
  const jumpTo = useCallback(
    (illustrationId: string) => {
      const idx = slides.findIndex(
        (s) => s.illustrationId === illustrationId && s.type === "image",
      );
      if (idx >= 0) {
        setSlideIndex(idx);
        setProgress(0);
      }
    },
    [slides],
  );

  // Cleanup on unmount
  useEffect(() => {
    return () => cancelAnimationFrame(rafRef.current);
  }, []);

  return {
    active,
    toggle,
    currentSlide,
    slideIndex,
    progress,
    advance,
    jumpTo,
    onVideoTimeUpdate,
    onVideoEnded,
    slides,
  };
}

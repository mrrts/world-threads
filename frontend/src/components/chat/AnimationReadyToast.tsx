import { useEffect } from "react";
import { Video, X } from "lucide-react";

interface AnimationReadyToastProps {
  animationReadyId: string | null;
  onGo: () => void;
  onDismiss: () => void;
}

export function AnimationReadyToast({ animationReadyId, onGo, onDismiss }: AnimationReadyToastProps) {
  useEffect(() => {
    if (!animationReadyId) return;
    const timer = setTimeout(onDismiss, 5000);
    return () => clearTimeout(timer);
  }, [animationReadyId, onDismiss]);

  if (!animationReadyId) return null;

  return (
    <div className="absolute bottom-4 right-4 z-20 bg-card border border-purple-500/30 rounded-xl shadow-xl shadow-black/30 px-4 py-3 flex items-center gap-3 animate-in fade-in slide-in-from-bottom-2 duration-200">
      <Video size={16} className="text-purple-400 flex-shrink-0" />
      <span className="text-sm font-medium">Animation is ready!</span>
      <button
        onClick={onGo}
        className="px-2.5 py-1 text-xs font-medium bg-purple-600 hover:bg-purple-700 text-white rounded-lg cursor-pointer transition-colors"
      >
        Go
      </button>
      <button
        onClick={onDismiss}
        className="text-muted-foreground hover:text-foreground cursor-pointer transition-colors"
      >
        <X size={14} />
      </button>
    </div>
  );
}

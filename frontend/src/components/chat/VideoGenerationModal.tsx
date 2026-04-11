import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Video, X, Loader2 } from "lucide-react";

interface VideoGenerationModalProps {
  open: boolean;
  onClose: () => void;
  onGenerate: () => void;
  onUpload: (file: File) => Promise<void>;
  videoTab: "generate" | "upload";
  setVideoTab: (v: "generate" | "upload") => void;
  videoStyle: string;
  setVideoStyle: (v: string) => void;
  videoPrompt: string;
  setVideoPrompt: (v: string) => void;
  videoDuration: number;
  setVideoDuration: (v: number) => void;
  uploadingVideo: boolean;
}

export function VideoGenerationModal({
  open,
  onClose,
  onGenerate,
  onUpload,
  videoTab,
  setVideoTab,
  videoStyle,
  setVideoStyle,
  videoPrompt,
  setVideoPrompt,
  videoDuration,
  setVideoDuration,
  uploadingVideo,
}: VideoGenerationModalProps) {
  return (
    <Dialog open={open} onClose={onClose} className="max-w-sm">
      <div className="p-5 space-y-4 bg-card/95 backdrop-blur-md border border-border rounded-xl shadow-2xl shadow-black/50">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Video size={18} className="text-purple-500" />
            <h3 className="font-semibold">Animate Illustration</h3>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
          >
            <X size={16} />
          </button>
        </div>

        <div className="flex border-b border-border">
          <button
            onClick={() => setVideoTab("generate")}
            className={`flex-1 pb-2 text-xs font-medium text-center border-b-2 transition-colors cursor-pointer ${
              videoTab === "generate" ? "border-purple-500 text-purple-400" : "border-transparent text-muted-foreground hover:text-foreground"
            }`}
          >
            Generate
          </button>
          <button
            onClick={() => setVideoTab("upload")}
            className={`flex-1 pb-2 text-xs font-medium text-center border-b-2 transition-colors cursor-pointer ${
              videoTab === "upload" ? "border-purple-500 text-purple-400" : "border-transparent text-muted-foreground hover:text-foreground"
            }`}
          >
            Upload
          </button>
        </div>

        {videoTab === "generate" ? (
          <>
            <div>
              <label className="text-xs font-medium text-muted-foreground block mb-1.5">Style</label>
              <div className="grid grid-cols-2 gap-1.5">
                {([
                  { value: "still", label: "Still" },
                  { value: "dialogue", label: "Dialogue" },
                  { value: "action-no-dialogue", label: "Action (Silent)" },
                  { value: "action-dialogue", label: "Action + Dialogue" },
                ] as const).map(({ value, label }) => (
                  <button
                    key={value}
                    onClick={() => setVideoStyle(value)}
                    className={`rounded-lg px-3 py-1.5 text-xs font-medium transition-all cursor-pointer ${
                      videoStyle === value
                        ? "bg-purple-600 text-white"
                        : "border border-border hover:border-purple-500/40 hover:bg-purple-500/5"
                    }`}
                  >
                    {label}
                  </button>
                ))}
              </div>
            </div>

            <div>
              <label className="text-xs font-medium text-muted-foreground block mb-1.5">Custom Direction (optional)</label>
              <textarea
                value={videoPrompt}
                onChange={(e) => setVideoPrompt(e.target.value)}
                placeholder="e.g. She turns to look out the window as rain begins to fall..."
                className="w-full min-h-[60px] max-h-[120px] resize-y rounded-lg border border-input bg-transparent px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                rows={2}
              />
              <p className="text-[10px] text-muted-foreground mt-1">Leave blank to auto-generate from conversation context.</p>
            </div>

            <div>
              <label className="text-xs font-medium text-muted-foreground block mb-1.5">Duration: {videoDuration}s</label>
              <input
                type="range"
                min={4}
                max={8}
                value={videoDuration}
                onChange={(e) => setVideoDuration(Number(e.target.value))}
                className="w-full accent-purple-500"
              />
              <div className="flex justify-between text-[10px] text-muted-foreground/50 mt-0.5">
                <span>4s</span>
                <span>8s</span>
              </div>
            </div>

            <div className="flex justify-end gap-2">
              <Button variant="ghost" size="sm" onClick={onClose}>
                Cancel
              </Button>
              <Button
                size="sm"
                className="bg-purple-600 hover:bg-purple-700 text-white"
                onClick={onGenerate}
              >
                Generate Video
              </Button>
            </div>
          </>
        ) : (
          <>
            <div>
              <p className="text-xs text-muted-foreground mb-3">Upload a video file to attach to this illustration.</p>
              <label className="flex flex-col items-center justify-center w-full h-32 border-2 border-dashed border-border rounded-xl cursor-pointer hover:border-purple-500/40 hover:bg-purple-500/5 transition-all">
                <Video size={24} className="text-muted-foreground/50 mb-2" />
                <span className="text-xs text-muted-foreground">Click to select a video file</span>
                <span className="text-[10px] text-muted-foreground/50 mt-0.5">MP4, WebM, or MOV</span>
                <input
                  type="file"
                  accept="video/mp4,video/webm,video/quicktime,.mp4,.webm,.mov"
                  className="hidden"
                  onChange={async (e) => {
                    const file = e.target.files?.[0];
                    if (!file) return;
                    await onUpload(file);
                  }}
                />
              </label>
            </div>

            {uploadingVideo && (
              <div className="flex items-center justify-center gap-2 text-purple-400">
                <Loader2 size={14} className="animate-spin" />
                <span className="text-xs">Uploading video...</span>
              </div>
            )}

            <div className="flex justify-end">
              <Button variant="ghost" size="sm" onClick={onClose}>
                Cancel
              </Button>
            </div>
          </>
        )}
      </div>
    </Dialog>
  );
}

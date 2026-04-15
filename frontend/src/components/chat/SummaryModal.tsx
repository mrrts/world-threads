import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Loader2, Send, Check } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { api, type Character, type GroupChat, type PortraitInfo } from "@/lib/tauri";

interface ChatTarget {
  type: "char" | "group";
  id: string;
  label: string;
  portraits: Array<{ data_url?: string; color: string }>;
}

interface Props {
  open: boolean;
  onClose: () => void;
  title: string;
  generateSummary: () => Promise<string>;
  /** All characters in the world (for the send-to picker) */
  characters?: Character[];
  groupChats?: GroupChat[];
  activePortraits?: Record<string, PortraitInfo>;
  /** The current chat's character_id or group_chat_id to exclude from targets */
  currentCharacterId?: string;
  currentGroupChatId?: string;
  /** Called after sending context to another chat */
  onContextSent?: () => void;
}

export function SummaryModal({
  open, onClose, title, generateSummary,
  characters = [], groupChats = [], activePortraits = {},
  currentCharacterId, currentGroupChatId, onContextSent,
}: Props) {
  const [summary, setSummary] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedTargets, setSelectedTargets] = useState<Set<string>>(new Set());
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);

  useEffect(() => {
    if (!open) return;
    setSummary("");
    setError(null);
    setSelectedTargets(new Set());
    setSent(false);
    setLoading(true);

    let unlisten: (() => void) | null = null;

    (async () => {
      unlisten = await listen<string>("summary-token", (event) => {
        setSummary((prev) => (prev ?? "") + event.payload);
      });
      try {
        const result = await generateSummary();
        setSummary(result);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
        unlisten?.();
      }
    })();

    return () => { unlisten?.(); };
  }, [open]);

  // Build target list excluding current chat
  const targets: ChatTarget[] = [];
  for (const ch of characters) {
    if (ch.character_id === currentCharacterId) continue;
    const p = activePortraits[ch.character_id];
    targets.push({
      type: "char",
      id: ch.character_id,
      label: ch.display_name,
      portraits: [{ data_url: p?.data_url, color: ch.avatar_color }],
    });
  }
  for (const gc of groupChats) {
    if (gc.group_chat_id === currentGroupChatId) continue;
    const charIds: string[] = Array.isArray(gc.character_ids) ? gc.character_ids : [];
    const portraits = charIds.map((id) => {
      const ch = characters.find((c) => c.character_id === id);
      const p = activePortraits[id];
      return { data_url: p?.data_url, color: ch?.avatar_color ?? "#888" };
    });
    const names = charIds.map((id) => characters.find((c) => c.character_id === id)?.display_name).filter(Boolean);
    targets.push({
      type: "group",
      id: gc.group_chat_id,
      label: names.join(" & "),
      portraits,
    });
  }

  const toggleTarget = (key: string) => {
    setSelectedTargets((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
    setSent(false);
  };

  const handleSend = async () => {
    if (!summary || selectedTargets.size === 0) return;
    setSending(true);
    try {
      for (const key of selectedTargets) {
        const target = targets.find((t) => `${t.type}:${t.id}` === key);
        if (!target) continue;
        if (target.type === "char") {
          await api.createContextMessage(target.id, undefined, summary);
        } else {
          await api.createContextMessage(undefined, target.id, summary);
        }
      }
      setSent(true);
      onContextSent?.();
    } catch (e) {
      setError(String(e));
    } finally {
      setSending(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>On-demand summary of the current conversation.</DialogDescription>
        </DialogHeader>
        <DialogBody>
          {loading && !summary ? (
            <div className="flex items-center justify-center py-8 gap-3">
              <Loader2 size={18} className="animate-spin text-muted-foreground" />
              <span className="text-sm text-muted-foreground">Generating summary...</span>
            </div>
          ) : error ? (
            <div className="text-sm text-destructive py-4">{error}</div>
          ) : summary ? (
            <>
              <p className="text-sm text-foreground leading-relaxed whitespace-pre-wrap">{summary}{loading ? <span className="inline-block w-1.5 h-4 bg-primary/60 animate-pulse ml-0.5 align-text-bottom" /> : null}</p>

              {targets.length > 0 && (
                <div className="mt-4 pt-4 border-t border-border">
                  <p className="text-xs font-medium text-muted-foreground mb-2">Send as context to other chats</p>
                  <div className="space-y-1 mb-3 max-h-40 overflow-y-auto">
                    {targets.map((t) => {
                      const key = `${t.type}:${t.id}`;
                      return (
                        <label key={key} className="flex items-center gap-2.5 px-2 py-1.5 rounded-lg hover:bg-accent/50 cursor-pointer select-none">
                          <input
                            type="checkbox"
                            checked={selectedTargets.has(key)}
                            onChange={() => toggleTarget(key)}
                            className="accent-primary w-3.5 h-3.5 flex-shrink-0"
                          />
                          <div className="flex items-center gap-1.5 flex-shrink-0">
                            {t.portraits.map((p, i) => (
                              p.data_url
                                ? <img key={i} src={p.data_url} alt="" className="w-6 h-6 rounded-full object-cover ring-1 ring-border" style={t.portraits.length > 1 ? { marginLeft: i > 0 ? -8 : 0, zIndex: t.portraits.length - i } : undefined} />
                                : <div key={i} className="w-6 h-6 rounded-full ring-1 ring-border" style={{ backgroundColor: p.color, marginLeft: i > 0 ? -8 : 0, zIndex: t.portraits.length - i }} />
                            ))}
                          </div>
                          <span className="text-xs truncate">{t.type === "group" ? `Group: ${t.label}` : t.label}</span>
                        </label>
                      );
                    })}
                  </div>
                  <Button
                    size="sm"
                    className="w-full"
                    disabled={selectedTargets.size === 0 || sending || sent}
                    onClick={handleSend}
                  >
                    {sending ? <Loader2 size={14} className="mr-1.5 animate-spin" /> : sent ? <Check size={14} className="mr-1.5" /> : <Send size={14} className="mr-1.5" />}
                    {sent ? `Sent to ${selectedTargets.size} chat${selectedTargets.size !== 1 ? "s" : ""}` : `Send to ${selectedTargets.size} chat${selectedTargets.size !== 1 ? "s" : ""}`}
                  </Button>
                </div>
              )}
            </>
          ) : null}
          <div className="flex justify-end mt-4">
            <Button variant="ghost" size="sm" onClick={onClose}>Close</Button>
          </div>
        </DialogBody>
      </DialogContent>
    </Dialog>
  );
}

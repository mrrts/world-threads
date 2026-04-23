import { useEffect, useState } from "react";
import { Dialog } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ArrowLeft, BookOpen, Feather, Loader2, RotateCw, ScrollText, X } from "lucide-react";
import {
  api,
  type KeptRecord,
  type Character,
  type Message,
  type UserProfile,
  type World,
  type ProposedCanonUpdate,
  type AppliedCanonUpdate,
  type CanonKind,
  type CanonAction,
} from "@/lib/tauri";

/// Props kept compatible with the previous modal so existing callers
/// (ChatView, GroupChatView, NarrativeMessage) don't need to change.
/// `onSaved` now fires once per applied update — one row per kind, as
/// before, so toast/notification logic still works per-record.
export function KeepRecordModal({
  open,
  onOpenChange,
  sourceMessage,
  sourceSpeakerLabel,
  world,
  userProfile,
  characters,
  apiKey,
  onSaved,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sourceMessage: Message | null;
  sourceSpeakerLabel: string;
  world: World | null;
  userProfile: UserProfile | null;
  characters: Character[];
  apiKey: string;
  onSaved: (saved: { entry: KeptRecord; subjectLabel: string }) => void;
}) {
  // Unused in the auto flow but kept to satisfy the prop signature.
  void world;
  void userProfile;
  void characters;

  // Two-act gate phase precedes the classifier call. The user picks
  // "light" (remember this) or "heavy" (this changes them); the same
  // modal branches the ceremony from that choice.
  type Phase = "gate" | "proposing" | "preview" | "committing" | "applied";
  type Act = "light" | "heavy";
  const [phase, setPhase] = useState<Phase>("gate");
  const [act, setAct] = useState<Act | null>(null);
  const [userHint, setUserHint] = useState("");
  const [userNote, setUserNote] = useState("");
  const [proposals, setProposals] = useState<ProposedCanonUpdate[]>([]);
  const [applied, setApplied] = useState<AppliedCanonUpdate[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Reset modal state each time it opens or closes.
  useEffect(() => {
    if (!open) {
      setPhase("gate");
      setAct(null);
      setUserHint("");
      setUserNote("");
      setProposals([]);
      setApplied([]);
      setError(null);
    }
  }, [open]);

  async function runPropose(pickedAct: Act) {
    if (!sourceMessage) return;
    setAct(pickedAct);
    setPhase("proposing");
    setError(null);
    try {
      const got = await api.proposeAutoCanon(apiKey, {
        sourceMessageId: sourceMessage.message_id,
        act: pickedAct,
        userHint: userHint.trim() || undefined,
      });
      setProposals(got);
      setPhase("preview");
    } catch (e) {
      setError(String(e));
      setPhase("gate");
    }
  }

  function backToGate() {
    setPhase("gate");
    setProposals([]);
    setApplied([]);
    setError(null);
  }

  async function runCommit() {
    if (!sourceMessage) return;
    // Strip proposals whose content was cleared — the user effectively
    // removed them from the batch by emptying the field.
    const cleaned = proposals
      .map((p) => ({ ...p, new_content: p.new_content.trim() }))
      .filter((p) => p.new_content.length > 0);
    if (cleaned.length === 0) {
      setError("All proposed updates are empty — edit content or regenerate.");
      return;
    }
    setPhase("committing");
    setError(null);
    try {
      const got = await api.commitAutoCanon(apiKey, {
        sourceMessageId: sourceMessage.message_id,
        updates: cleaned,
        userNote: userNote.trim() || undefined,
      });
      setApplied(got);
      setPhase("applied");
      // Fire onSaved per applied update that wrote a kept_records row.
      // Remove ops don't write a ledger entry (see canon_cmds.rs), so
      // they're reported to the UI but not as "saved records."
      for (const a of got) {
        if (!a.kept_id) continue;
        onSaved({
          entry: {
            kept_id: a.kept_id,
            source_message_id: sourceMessage.message_id,
            source_thread_id: null,
            source_world_day: null,
            source_created_at: null,
            subject_type: a.subject_type,
            subject_id: a.subject_id,
            record_type: a.kind,
            content: a.new_content,
            user_note: "",
            created_at: new Date().toISOString(),
          },
          subjectLabel: a.subject_label,
        });
      }
    } catch (e) {
      setError(String(e));
      setPhase("preview");
    }
  }

  function updateProposalAt(i: number, patch: Partial<ProposedCanonUpdate>) {
    setProposals((prev) => prev.map((p, idx) => (idx === i ? { ...p, ...patch } : p)));
  }

  if (!open || !sourceMessage) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <div className="fixed inset-0 z-50 flex items-start justify-center p-6 overflow-y-auto">
        <div className="w-full max-w-2xl my-8 bg-card border border-border rounded-xl shadow-2xl shadow-black/40 p-5 space-y-4 animate-in fade-in zoom-in-95 duration-150">
          <div className="flex items-center gap-2">
            <ScrollText size={18} className="text-primary" />
            <h2 className="text-base font-semibold">
              {phase === "applied"
                ? "Canonized"
                : phase === "gate"
                ? "Canonize this moment"
                : act === "heavy"
                ? "This changes them"
                : "Remember this"}
            </h2>
          </div>

          {/* Source message preview — shown in every phase */}
          <div className="rounded-lg border border-border/60 bg-secondary/30 p-3">
            <div className="text-[11px] uppercase tracking-wide text-muted-foreground mb-1">
              Source — {sourceSpeakerLabel}
              {sourceMessage.world_day != null && sourceMessage.world_time ? (
                <span> · Day {sourceMessage.world_day}, {sourceMessage.world_time}</span>
              ) : null}
            </div>
            <div className="text-sm whitespace-pre-wrap line-clamp-6">{sourceMessage.content}</div>
          </div>

          {/* Phase: gate — the two-act primary choice */}
          {phase === "gate" && (
            <>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1.5">
                  Optional hint (applies to either act)
                </label>
                <textarea
                  value={userHint}
                  onChange={(e) => setUserHint(e.target.value)}
                  rows={2}
                  placeholder={`e.g. "add this as a boundary for Darren" or "remember Anna likes her coffee with a splash of cream, no sugar"`}
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring resize-y"
                />
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <button
                  onClick={() => runPropose("light")}
                  disabled={!apiKey}
                  className="text-left rounded-xl border border-border bg-secondary/30 hover:bg-secondary/60 hover:border-primary/50 transition-colors p-4 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer group"
                >
                  <div className="flex items-center gap-2 mb-2">
                    <Feather size={16} className="text-primary" />
                    <span className="text-sm font-semibold">Remember this</span>
                  </div>
                  <div className="text-xs text-muted-foreground leading-snug">
                    A fact, boundary, tic, or open thread worth carrying. Does not reshape who they fundamentally are.
                  </div>
                </button>
                <button
                  onClick={() => runPropose("heavy")}
                  disabled={!apiKey}
                  className="text-left rounded-xl border border-border bg-secondary/30 hover:bg-secondary/60 hover:border-primary/50 transition-colors p-4 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer group"
                >
                  <div className="flex items-center gap-2 mb-2">
                    <BookOpen size={16} className="text-primary" />
                    <span className="text-sm font-semibold">This changes them</span>
                  </div>
                  <div className="text-xs text-muted-foreground leading-snug">
                    A load-bearing revelation the canon needs to hold. Reshapes how this character is carried from here forward.
                  </div>
                </button>
              </div>
            </>
          )}

          {/* Phase: proposing / committing — spinner states */}
          {(phase === "proposing" || phase === "committing") && (
            <div className="flex items-center justify-center py-8 gap-2 text-sm text-muted-foreground">
              <Loader2 size={16} className="animate-spin" />
              <span>{phase === "proposing" ? "Classifying the moment…" : "Applying updates…"}</span>
            </div>
          )}

          {/* Phase: preview — show proposed updates, editable */}
          {phase === "preview" && proposals.length > 0 && (
            <div className="space-y-3">
              <div className="text-xs text-muted-foreground">
                {proposals.length === 1
                  ? "The classifier proposed 1 update. Edit the content below if you want to tweak it, then Commit."
                  : `The classifier proposed ${proposals.length} updates. Edit any content below if you want to tweak it, then Commit.`}
              </div>
              {proposals.map((p, i) => (
                <ProposalCard
                  key={i}
                  proposal={p}
                  onContentChange={(next) => updateProposalAt(i, { new_content: next })}
                  onSkip={() => setProposals((prev) => prev.filter((_, idx) => idx !== i))}
                />
              ))}
              {proposals.length === 0 && (
                <div className="rounded-lg border border-border/60 bg-secondary/10 p-3 text-xs text-muted-foreground text-center">
                  All proposals skipped. Regenerate to get a new set, or cancel.
                </div>
              )}
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1.5">
                  Optional note (why this matters to you — stored on every record)
                </label>
                <input
                  value={userNote}
                  onChange={(e) => setUserNote(e.target.value)}
                  placeholder="A private note stored with each kept record."
                  className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
                />
              </div>
            </div>
          )}

          {/* Phase: applied — final report */}
          {phase === "applied" && (
            <div className="space-y-2">
              <div className="text-xs text-muted-foreground">
                {applied.length === 1 ? "1 update applied:" : `${applied.length} updates applied:`}
              </div>
              {applied.map((a) => (
                <AppliedCard key={a.kept_id} applied={a} />
              ))}
            </div>
          )}

          {error && (
            <div className="rounded-lg border border-destructive/50 bg-destructive/10 text-destructive text-xs p-2">
              {error}
            </div>
          )}

          {/* Actions — phase-dependent */}
          <div className="flex items-center justify-end gap-2 pt-1">
            {phase === "gate" && (
              <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
            )}
            {phase === "preview" && (
              <>
                <Button variant="ghost" onClick={backToGate}>
                  <ArrowLeft size={14} className="mr-1.5" />
                  Back
                </Button>
                <Button variant="outline" onClick={() => act && runPropose(act)}>
                  <RotateCw size={14} className="mr-1.5" />
                  Regenerate
                </Button>
                <Button onClick={runCommit}>Commit</Button>
              </>
            )}
            {phase === "applied" && (
              <Button onClick={() => onOpenChange(false)}>Done</Button>
            )}
          </div>
        </div>
      </div>
    </Dialog>
  );
}

/// Render one proposed update with edit controls that match the
/// CharacterEditor shape for that kind:
/// - description_weave → big textarea (with "before" collapsible)
/// - voice_rule / boundary / known_fact / open_loop → single-line input
/// Destructive actions (update / remove) are visually distinguished:
/// - update: amber accent, "before" shown above the editable replacement
/// - remove: rose accent, target shown struck-through, no editable field
/// - add (default): neutral accent, new content in an editable input
function ProposalCard({
  proposal,
  onContentChange,
  onSkip,
}: {
  proposal: ProposedCanonUpdate;
  onContentChange: (next: string) => void;
  onSkip: () => void;
}) {
  const label = KIND_LABEL[proposal.kind];
  const isWeave = proposal.kind === "description_weave";
  const actionStyle = ACTION_STYLES[proposal.action];
  const isRemove = proposal.action === "remove";
  return (
    <div className={`rounded-lg border ${actionStyle.border} ${actionStyle.bg} p-3 space-y-2 relative`}>
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-[10px] uppercase tracking-wider font-semibold text-primary bg-primary/10 border border-primary/30 rounded px-1.5 py-0.5">
            {label}
          </span>
          {proposal.action !== "add" && (
            <span className={`text-[10px] uppercase tracking-wider font-semibold ${actionStyle.badgeText} ${actionStyle.badgeBg} ${actionStyle.badgeBorder} border rounded px-1.5 py-0.5`}>
              {ACTION_LABEL[proposal.action]}
            </span>
          )}
          <span className="text-xs text-muted-foreground">for {proposal.subject_label}</span>
        </div>
        <button
          onClick={onSkip}
          title="Skip this proposal"
          aria-label="Skip this proposal"
          className="text-muted-foreground hover:text-foreground transition-colors cursor-pointer p-0.5 -mt-0.5 -mr-0.5"
        >
          <X size={14} />
        </button>
      </div>
      {proposal.justification && (
        <div className="text-[11px] text-muted-foreground italic leading-snug">
          {proposal.justification}
        </div>
      )}
      {isWeave ? (
        <>
          <textarea
            value={proposal.new_content}
            onChange={(e) => onContentChange(e.target.value)}
            rows={8}
            className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring resize-y"
          />
          {proposal.prior_content && proposal.prior_content.trim() && (
            <details className="text-xs">
              <summary className="cursor-pointer text-muted-foreground hover:text-foreground select-none">
                Show current description (before)
              </summary>
              <div className="mt-2 rounded-lg border border-border/60 bg-secondary/10 p-2 text-sm whitespace-pre-wrap max-h-40 overflow-y-auto">
                {proposal.prior_content}
              </div>
            </details>
          )}
        </>
      ) : isRemove ? (
        <div className="rounded-md border border-rose-500/30 bg-rose-500/5 px-3 py-2 text-sm">
          <span className="text-[10px] uppercase tracking-wider text-rose-700 dark:text-rose-400 mr-2">will remove:</span>
          <span className="line-through text-muted-foreground">{proposal.target_existing_text ?? ""}</span>
        </div>
      ) : proposal.action === "update" ? (
        <>
          {proposal.target_existing_text && (
            <div className="rounded-md border border-border/40 bg-secondary/10 px-3 py-1.5 text-xs">
              <span className="text-[10px] uppercase tracking-wider text-muted-foreground mr-2">current:</span>
              <span className="text-muted-foreground">{proposal.target_existing_text}</span>
            </div>
          )}
          <input
            value={proposal.new_content}
            onChange={(e) => onContentChange(e.target.value)}
            placeholder="Replacement text"
            className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
          />
        </>
      ) : (
        <input
          value={proposal.new_content}
          onChange={(e) => onContentChange(e.target.value)}
          className="w-full rounded-lg border border-input bg-transparent px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
        />
      )}
    </div>
  );
}

/// Applied-state row — read-only summary of what just happened.
/// Uses the same action-color scheme as the preview card so "what was
/// proposed" and "what was applied" are visually consistent.
function AppliedCard({ applied }: { applied: AppliedCanonUpdate }) {
  const label = KIND_LABEL[applied.kind];
  const isWeave = applied.kind === "description_weave";
  const style = ACTION_STYLES[applied.action];
  return (
    <div className={`rounded-lg border ${style.border} ${style.bg} p-3 space-y-1.5`}>
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-[10px] uppercase tracking-wider font-semibold text-primary bg-primary/10 border border-primary/30 rounded px-1.5 py-0.5">
          {label}
        </span>
        <span className={`text-[10px] uppercase tracking-wider font-semibold ${style.badgeText} ${style.badgeBg} ${style.badgeBorder} border rounded px-1.5 py-0.5`}>
          {ACTION_LABEL[applied.action]}
        </span>
        <span className="text-xs text-muted-foreground">for {applied.subject_label}</span>
      </div>
      {applied.action === "remove" ? (
        <div className="text-sm">
          <span className="text-[10px] uppercase tracking-wider text-rose-700 dark:text-rose-400 mr-2">removed:</span>
          <span className="line-through text-muted-foreground">{applied.prior_content ?? applied.target_existing_text ?? ""}</span>
        </div>
      ) : isWeave ? (
        <>
          <div className="text-sm whitespace-pre-wrap">{applied.new_content}</div>
          {applied.prior_content && applied.prior_content.trim() && (
            <details className="text-xs">
              <summary className="cursor-pointer text-muted-foreground hover:text-foreground select-none">
                Show before
              </summary>
              <div className="mt-1.5 text-muted-foreground/80 whitespace-pre-wrap max-h-32 overflow-y-auto">
                {applied.prior_content}
              </div>
            </details>
          )}
        </>
      ) : applied.action === "update" ? (
        <>
          {applied.prior_content && (
            <div className="text-xs text-muted-foreground line-through">{applied.prior_content}</div>
          )}
          <div className="text-sm">{applied.new_content}</div>
        </>
      ) : (
        <div className="text-sm">{applied.new_content}</div>
      )}
    </div>
  );
}

const KIND_LABEL: Record<CanonKind, string> = {
  description_weave: "Description",
  voice_rule: "Voice rule",
  boundary: "Boundary",
  known_fact: "Known fact",
  open_loop: "Open loop",
};

const ACTION_LABEL: Record<CanonAction, string> = {
  add: "Add",
  update: "Update",
  remove: "Remove",
};

const ACTION_STYLES: Record<CanonAction, {
  border: string;
  bg: string;
  badgeText: string;
  badgeBg: string;
  badgeBorder: string;
}> = {
  add: {
    border: "border-border/60",
    bg: "bg-secondary/20",
    badgeText: "text-emerald-700 dark:text-emerald-400",
    badgeBg: "bg-emerald-500/10",
    badgeBorder: "border-emerald-500/30",
  },
  update: {
    border: "border-amber-500/30",
    bg: "bg-amber-500/5",
    badgeText: "text-amber-700 dark:text-amber-400",
    badgeBg: "bg-amber-500/10",
    badgeBorder: "border-amber-500/30",
  },
  remove: {
    border: "border-rose-500/30",
    bg: "bg-rose-500/5",
    badgeText: "text-rose-700 dark:text-rose-400",
    badgeBg: "bg-rose-500/10",
    badgeBorder: "border-rose-500/30",
  },
};

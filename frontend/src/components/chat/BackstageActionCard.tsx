import { useState } from "react";
import { Check, X, Loader2, Feather, Send, ImagePlus, Palette, Users } from "lucide-react";
import { api } from "@/lib/tauri";

/// A parsed Backstage action — extracted from a ```action fenced block
/// in the assistant's response. Two flavors are shipping in v1:
///
///   - `canon_entry` — AI drafted a replacement identity for a
///     character (or the user). Clicking "Save" persists it via
///     `saveKeptRecord` with record_type=description_weave.
///   - `staged_message` — AI drafted a message to put in the active
///     chat's input. Clicking "Stage" dispatches a window event that
///     ChatView / GroupChatView listens for and uses to fill input.
///
/// Unknown `type` values render as a raw JSON pre-block so nothing is
/// silently dropped when the prompt evolves faster than this parser.
export type BackstageActionBlock =
  | {
      type: "canon_entry";
      subject_type: "character" | "user";
      subject_id: string;
      /// Always "description_weave" — the only record_type the save
      /// path supports. Optional in the JSON (defaults).
      record_type?: string;
      label: string;
      content: string;
    }
  | {
      type: "staged_message";
      label: string;
      content: string;
    }
  | {
      type: "portrait_regen";
      /// Character whose portrait is regenerating.
      subject_id: string;
      /// Freeform pose / mood / detail description passed to
      /// generate_portrait_with_pose_cmd as the pose_description arg.
      pose_description: string;
      label: string;
    }
  | {
      type: "illustration";
      /// Character whose solo chat the illustration gets attached to.
      character_id: string;
      /// Scene description → generate_illustration_cmd custom_instructions.
      custom_instructions: string;
      label: string;
    }
  | {
      type: "new_group_chat";
      /// Exactly two character ids — backend rejects anything else.
      character_ids: string[];
      label: string;
    }
  | {
      type: string;
      label?: string;
      content?: string;
      [key: string]: unknown;
    };

export interface BackstageActionContext {
  /// Thread id of the chat the user was in when Backstage opened.
  /// Staged messages route here. Illustrations also commit to this
  /// thread (whether it's a solo or group thread is resolved by
  /// `groupChatId` below).
  activeThreadId: string;
  /// When the active chat is a group chat, its id. Lets the
  /// illustration flow paint with the full group cast and commit to
  /// the group_messages table instead of the (wrong) solo character's
  /// thread.
  groupChatId: string | null;
  /// Close the Backstage modal after a successful stage/save. Matches
  /// the "fire-and-forget" feel — the card did its job, return user to
  /// their chat.
  onAppliedClose: () => void;
  /// API key for actions that trigger LLM/image generation (portrait
  /// regen, illustration). Missing key → action shows an error instead
  /// of silently failing.
  apiKey: string;
  /// World id — required for new_group_chat creation.
  worldId: string;
}

interface Props {
  block: BackstageActionBlock;
  ctx: BackstageActionContext;
}

export function BackstageActionCard({ block, ctx }: Props) {
  const [state, setState] = useState<"idle" | "applying" | "applied" | "dismissed" | "error" | "previewing" | "preview-shown" | "attaching">("idle");
  const [error, setError] = useState<string | null>(null);
  // Illustration card preview state — set after the preview command
  // returns; cleared on discard/try-again. The image is already saved to
  // disk + world_images at preview time; the imageId is what attach uses.
  const [previewedImage, setPreviewedImage] = useState<{ imageId: string; dataUrl: string; aspectRatio: number } | null>(null);

  if (state === "dismissed") {
    return (
      <div className="my-3 px-3 py-2 rounded-lg border border-border/30 bg-muted/20 text-[11px] text-muted-foreground/70 italic">
        Dismissed.
      </div>
    );
  }

  if (state === "applied") {
    const label =
      block.type === "canon_entry" ? "Saved to Canon." :
      block.type === "staged_message" ? "Staged in your chat." :
      block.type === "portrait_regen" ? "Portrait painted — find it in the character editor." :
      block.type === "illustration" ? "Illustration added to the chat." :
      block.type === "new_group_chat" ? "Group chat created — find it in your sidebar." :
      "Applied.";
    return (
      <div className="my-3 px-3 py-2 rounded-lg border border-emerald-500/30 bg-emerald-500/10 text-[11px] text-emerald-400 flex items-center gap-2">
        <Check size={12} />
        <span>{label}</span>
      </div>
    );
  }

  // Canon entry card
  if (block.type === "canon_entry") {
    const onSave = async () => {
      setState("applying");
      setError(null);
      try {
        await api.saveKeptRecord({
          subjectType: block.subject_type,
          subjectId: block.subject_id,
          recordType: block.record_type || "description_weave",
          content: block.content,
        });
        setState("applied");
        setTimeout(() => ctx.onAppliedClose(), 600);
      } catch (e: any) {
        setError(String(e));
        setState("error");
      }
    };
    return (
      <div className="my-3 rounded-xl border border-amber-400/40 bg-amber-500/5 overflow-hidden">
        <div className="px-4 py-2.5 border-b border-amber-400/20 bg-amber-500/10 flex items-center gap-2">
          <Feather size={13} className="text-amber-400" />
          <span className="text-[11px] uppercase tracking-wider font-semibold text-amber-300">
            Propose Canon entry
          </span>
        </div>
        <div className="px-4 py-3">
          <p className="text-[11px] text-muted-foreground/80 mb-2 italic">{block.label}</p>
          <div className="text-sm leading-relaxed whitespace-pre-wrap text-foreground/90 bg-background/40 rounded-md p-3 border border-border/40 max-h-[320px] overflow-y-auto">
            {block.content}
          </div>
          {error && <p className="text-xs text-destructive mt-2">{error}</p>}
          <div className="flex items-center gap-2 mt-3">
            <button
              onClick={onSave}
              disabled={state === "applying"}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-amber-500/90 hover:bg-amber-500 text-black text-xs font-medium transition-colors cursor-pointer disabled:opacity-60 disabled:cursor-wait"
            >
              {state === "applying" ? <Loader2 size={12} className="animate-spin" /> : <Check size={12} />}
              Save as Canon
            </button>
            <button
              onClick={() => setState("dismissed")}
              disabled={state === "applying"}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border/50 text-muted-foreground hover:text-foreground hover:bg-accent text-xs transition-colors cursor-pointer"
            >
              <X size={12} />
              Dismiss
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Staged message card
  if (block.type === "staged_message") {
    const onStage = () => {
      window.dispatchEvent(new CustomEvent("backstage:stage-message", {
        detail: { threadId: ctx.activeThreadId, text: block.content },
      }));
      setState("applied");
      setTimeout(() => ctx.onAppliedClose(), 400);
    };
    return (
      <div className="my-3 rounded-xl border border-amber-400/40 bg-amber-500/5 overflow-hidden">
        <div className="px-4 py-2.5 border-b border-amber-400/20 bg-amber-500/10 flex items-center gap-2">
          <Send size={13} className="text-amber-400" />
          <span className="text-[11px] uppercase tracking-wider font-semibold text-amber-300">
            Stage a draft
          </span>
        </div>
        <div className="px-4 py-3">
          <p className="text-[11px] text-muted-foreground/80 mb-2 italic">{block.label}</p>
          <div className="text-sm leading-relaxed whitespace-pre-wrap text-foreground/90 bg-background/40 rounded-md p-3 border border-border/40 max-h-[240px] overflow-y-auto">
            {block.content}
          </div>
          <div className="flex items-center gap-2 mt-3">
            <button
              onClick={onStage}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-amber-500/90 hover:bg-amber-500 text-black text-xs font-medium transition-colors cursor-pointer"
            >
              <Send size={12} />
              Stage in chat
            </button>
            <button
              onClick={() => setState("dismissed")}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border/50 text-muted-foreground hover:text-foreground hover:bg-accent text-xs transition-colors cursor-pointer"
            >
              <X size={12} />
              Dismiss
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Portrait regeneration card
  if (block.type === "portrait_regen") {
    const onApply = async () => {
      if (!ctx.apiKey) { setError("No API key configured."); setState("error"); return; }
      setState("applying");
      setError(null);
      try {
        await api.generatePortraitWithPose(ctx.apiKey, block.subject_id, block.pose_description);
        setState("applied");
        // Portrait creation is slow — leave the success state visible a
        // beat longer before auto-close so the user sees what happened.
        setTimeout(() => ctx.onAppliedClose(), 1200);
      } catch (e: any) {
        setError(String(e));
        setState("error");
      }
    };
    return (
      <div className="my-3 rounded-xl border border-amber-400/40 bg-amber-500/5 overflow-hidden">
        <div className="px-4 py-2.5 border-b border-amber-400/20 bg-amber-500/10 flex items-center gap-2">
          <Palette size={13} className="text-amber-400" />
          <span className="text-[11px] uppercase tracking-wider font-semibold text-amber-300">
            Propose portrait variation
          </span>
        </div>
        <div className="px-4 py-3">
          <p className="text-[11px] text-muted-foreground/80 mb-2 italic">{block.label}</p>
          <div className="text-sm leading-relaxed whitespace-pre-wrap text-foreground/90 bg-background/40 rounded-md p-3 border border-border/40">
            {block.pose_description}
          </div>
          {error && <p className="text-xs text-destructive mt-2">{error}</p>}
          <div className="flex items-center gap-2 mt-3">
            <button
              onClick={onApply}
              disabled={state === "applying"}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-amber-500/90 hover:bg-amber-500 text-black text-xs font-medium transition-colors cursor-pointer disabled:opacity-60 disabled:cursor-wait"
            >
              {state === "applying" ? <Loader2 size={12} className="animate-spin" /> : <Palette size={12} />}
              {state === "applying" ? "Painting…" : "Paint this portrait"}
            </button>
            <button
              onClick={() => setState("dismissed")}
              disabled={state === "applying"}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border/50 text-muted-foreground hover:text-foreground hover:bg-accent text-xs transition-colors cursor-pointer"
            >
              <X size={12} />
              Dismiss
            </button>
          </div>
          {state === "applying" && (
            <p className="text-[10px] text-muted-foreground/60 mt-2 italic">This usually takes 20-40 seconds.</p>
          )}
        </div>
      </div>
    );
  }

  // Illustration card — two-step flow:
  //   1. "Generate preview" → backend renders the image without inserting
  //      a chat message, returns the data URL. Card shows the preview
  //      inline + "Add to chat" / "Try again" / "Discard" buttons.
  //   2. "Add to chat" → backend commits the previewed image into the
  //      active thread (solo or group, routed by ctx.groupChatId).
  //   3. "Discard" → backend deletes the file + world_images row.
  // The earlier one-step flow lost the image into the wrong thread when
  // Backstage was opened from a group chat.
  if (block.type === "illustration") {
    const onPreview = async () => {
      if (!ctx.apiKey) { setError("No API key configured."); setState("error"); return; }
      setState("previewing");
      setError(null);
      try {
        const result = await api.previewBackstageIllustration(ctx.apiKey, block.character_id, ctx.groupChatId, block.custom_instructions);
        setPreviewedImage({ imageId: result.image_id, dataUrl: result.data_url, aspectRatio: result.aspect_ratio });
        setState("preview-shown");
      } catch (e: any) {
        setError(String(e));
        setState("error");
      }
    };
    const onAttach = async () => {
      if (!previewedImage) return;
      setState("attaching");
      setError(null);
      try {
        const msg = await api.attachPreviewedIllustration(previewedImage.imageId, ctx.activeThreadId, ctx.groupChatId !== null);
        // Tell the chat view (which is mounted under the modal) to
        // refresh its message list so the illustration appears.
        window.dispatchEvent(new CustomEvent("backstage:illustration-added", {
          detail: { threadId: ctx.activeThreadId, message: msg },
        }));
        setState("applied");
        setTimeout(() => ctx.onAppliedClose(), 800);
      } catch (e: any) {
        setError(String(e));
        setState("preview-shown");
      }
    };
    const onDiscard = async () => {
      if (!previewedImage) { setState("dismissed"); return; }
      try {
        await api.discardPreviewedIllustration(previewedImage.imageId);
      } catch { /* best effort cleanup */ }
      setPreviewedImage(null);
      setState("dismissed");
    };
    const onTryAgain = async () => {
      if (previewedImage) {
        try { await api.discardPreviewedIllustration(previewedImage.imageId); } catch { /* best effort */ }
        setPreviewedImage(null);
      }
      onPreview();
    };
    return (
      <div className="my-3 rounded-xl border border-amber-400/40 bg-amber-500/5 overflow-hidden">
        <div className="px-4 py-2.5 border-b border-amber-400/20 bg-amber-500/10 flex items-center gap-2">
          <ImagePlus size={13} className="text-amber-400" />
          <span className="text-[11px] uppercase tracking-wider font-semibold text-amber-300">
            Propose an illustration
          </span>
        </div>
        <div className="px-4 py-3">
          <p className="text-[11px] text-muted-foreground/80 mb-2 italic">{block.label}</p>
          <div className="text-sm leading-relaxed whitespace-pre-wrap text-foreground/90 bg-background/40 rounded-md p-3 border border-border/40 max-h-[240px] overflow-y-auto">
            {block.custom_instructions}
          </div>
          {previewedImage && (
            <div className="mt-3 rounded-lg overflow-hidden border border-amber-400/30 bg-black/20">
              <img
                src={previewedImage.dataUrl}
                alt="Preview"
                className="block w-full h-auto"
                style={{ aspectRatio: String(previewedImage.aspectRatio) }}
                draggable={false}
              />
            </div>
          )}
          {state === "previewing" && (
            <div className="mt-3 flex items-center gap-2 text-xs text-muted-foreground italic">
              <Loader2 size={12} className="animate-spin" />
              <span>Painting preview… (20–40 seconds)</span>
            </div>
          )}
          {error && <p className="text-xs text-destructive mt-2">{error}</p>}
          <div className="flex items-center gap-2 mt-3 flex-wrap">
            {state === "preview-shown" || state === "attaching" ? (
              <>
                <button
                  onClick={onAttach}
                  disabled={state === "attaching"}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-emerald-500/90 hover:bg-emerald-500 text-black text-xs font-medium transition-colors cursor-pointer disabled:opacity-60 disabled:cursor-wait"
                >
                  {state === "attaching" ? <Loader2 size={12} className="animate-spin" /> : <Check size={12} />}
                  {state === "attaching" ? "Adding…" : "Add to chat"}
                </button>
                <button
                  onClick={onTryAgain}
                  disabled={state === "attaching"}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border/50 text-muted-foreground hover:text-foreground hover:bg-accent text-xs transition-colors cursor-pointer"
                >
                  <ImagePlus size={12} />
                  Try again
                </button>
                <button
                  onClick={onDiscard}
                  disabled={state === "attaching"}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border/50 text-muted-foreground hover:text-foreground hover:bg-accent text-xs transition-colors cursor-pointer"
                >
                  <X size={12} />
                  Discard
                </button>
              </>
            ) : (
              <>
                <button
                  onClick={onPreview}
                  disabled={state === "previewing"}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-amber-500/90 hover:bg-amber-500 text-black text-xs font-medium transition-colors cursor-pointer disabled:opacity-60 disabled:cursor-wait"
                >
                  {state === "previewing" ? <Loader2 size={12} className="animate-spin" /> : <ImagePlus size={12} />}
                  {state === "previewing" ? "Painting…" : "Generate preview"}
                </button>
                <button
                  onClick={onDiscard}
                  disabled={state === "previewing"}
                  className="flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border/50 text-muted-foreground hover:text-foreground hover:bg-accent text-xs transition-colors cursor-pointer"
                >
                  <X size={12} />
                  Dismiss
                </button>
              </>
            )}
          </div>
        </div>
      </div>
    );
  }

  // New group chat card
  if (block.type === "new_group_chat") {
    const onApply = async () => {
      if (!ctx.worldId) { setError("Missing world context."); setState("error"); return; }
      if (!Array.isArray(block.character_ids) || block.character_ids.length !== 2) {
        setError("Group chats need exactly 2 characters.");
        setState("error");
        return;
      }
      setState("applying");
      setError(null);
      try {
        await api.createGroupChat(ctx.worldId, block.character_ids);
        setState("applied");
        setTimeout(() => ctx.onAppliedClose(), 800);
      } catch (e: any) {
        setError(String(e));
        setState("error");
      }
    };
    return (
      <div className="my-3 rounded-xl border border-amber-400/40 bg-amber-500/5 overflow-hidden">
        <div className="px-4 py-2.5 border-b border-amber-400/20 bg-amber-500/10 flex items-center gap-2">
          <Users size={13} className="text-amber-400" />
          <span className="text-[11px] uppercase tracking-wider font-semibold text-amber-300">
            Propose a new group chat
          </span>
        </div>
        <div className="px-4 py-3">
          <p className="text-sm leading-relaxed text-foreground/90">{block.label}</p>
          {error && <p className="text-xs text-destructive mt-2">{error}</p>}
          <div className="flex items-center gap-2 mt-3">
            <button
              onClick={onApply}
              disabled={state === "applying"}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-amber-500/90 hover:bg-amber-500 text-black text-xs font-medium transition-colors cursor-pointer disabled:opacity-60 disabled:cursor-wait"
            >
              {state === "applying" ? <Loader2 size={12} className="animate-spin" /> : <Users size={12} />}
              {state === "applying" ? "Creating…" : "Start this group chat"}
            </button>
            <button
              onClick={() => setState("dismissed")}
              disabled={state === "applying"}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-border/50 text-muted-foreground hover:text-foreground hover:bg-accent text-xs transition-colors cursor-pointer"
            >
              <X size={12} />
              Dismiss
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Unknown action type — render the raw JSON so nothing silently drops.
  return (
    <div className="my-3 px-3 py-2 rounded-lg border border-border/30 bg-muted/20 text-[11px] text-muted-foreground/70">
      <p className="mb-1 italic">Unknown action type: <code>{block.type}</code></p>
      <pre className="whitespace-pre-wrap break-all text-[10px]">{JSON.stringify(block, null, 2)}</pre>
    </div>
  );
}

/// Scan an assistant message for ```action fenced JSON blocks and
/// return an interleaved array of text and action segments — preserves
/// original ordering so cards render where they appeared in the prose.
/// Partial / malformed JSON (common during streaming) is yielded as
/// plain text so nothing is hidden.
export type BackstageSegment =
  | { kind: "text"; value: string }
  | { kind: "action"; block: BackstageActionBlock };

export function parseBackstageSegments(content: string): BackstageSegment[] {
  if (!content) return [];
  const segments: BackstageSegment[] = [];
  const re = /```action\s*\n([\s\S]*?)\n```/g;
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = re.exec(content)) !== null) {
    if (m.index > last) {
      segments.push({ kind: "text", value: content.slice(last, m.index) });
    }
    const jsonBody = m[1].trim();
    try {
      const parsed = JSON.parse(jsonBody);
      if (parsed && typeof parsed === "object" && typeof parsed.type === "string") {
        segments.push({ kind: "action", block: parsed as BackstageActionBlock });
      } else {
        segments.push({ kind: "text", value: m[0] });
      }
    } catch {
      // Malformed — keep as text so the user sees it rather than losing it.
      segments.push({ kind: "text", value: m[0] });
    }
    last = m.index + m[0].length;
  }
  if (last < content.length) {
    segments.push({ kind: "text", value: content.slice(last) });
  }
  return segments;
}

import { Package, Heart } from "lucide-react";
import type { InventoryItem } from "@/lib/tauri";

/**
 * Compact read-only display of a character's kept items. Physical
 * items (up to 3) render as a bulleted list with the Package icon;
 * the single interior item — a non-physical thing they're carrying
 * inside, a memory / core truth / profound feeling — renders as a
 * distinct labeled line below. Used in character popovers, cards,
 * and anywhere we show a character's state.
 *
 * Empty inventory → renders nothing.
 */
export function InventoryStrip({
  inventory,
  compact = false,
}: {
  inventory: InventoryItem[] | undefined | null;
  /** Compact mode drops descriptions and uses tighter padding — use in
   *  dense popovers / cards. Default shows names + descriptions. */
  compact?: boolean;
}) {
  const items = (inventory ?? []).filter((it) => it?.name?.trim());
  const physical = items.filter((it) => (it.kind ?? "physical") !== "interior");
  const interior = items.find((it) => it.kind === "interior");
  if (physical.length === 0 && !interior) return null;

  return (
    <div className="space-y-2">
      {physical.length > 0 && (
        <div className="space-y-1">
          <div className="flex items-center gap-1.5 text-[10px] uppercase tracking-wider text-muted-foreground/70 font-semibold">
            <Package size={10} />
            <span>In their keeping</span>
          </div>
          <ul className="space-y-1">
            {physical.map((it, i) => (
              <li key={i} className="text-xs leading-snug">
                <span className="text-foreground/90">{it.name}</span>
                {!compact && it.description?.trim() ? (
                  <span className="text-muted-foreground/70 italic"> — {it.description}</span>
                ) : null}
              </li>
            ))}
          </ul>
        </div>
      )}
      {interior && (
        <div className="space-y-1">
          <div className="flex items-center gap-1.5 text-[10px] uppercase tracking-wider text-muted-foreground/70 font-semibold">
            <Heart size={10} />
            <span>Carried inside</span>
          </div>
          <p className="text-xs leading-snug italic">
            <span className="text-foreground/90">{interior.name}</span>
            {!compact && interior.description?.trim() ? (
              <span className="text-muted-foreground/70"> — {interior.description}</span>
            ) : null}
          </p>
        </div>
      )}
    </div>
  );
}

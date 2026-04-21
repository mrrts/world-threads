import { Package } from "lucide-react";
import type { InventoryUpdateRecord } from "@/lib/tauri";

interface Props {
  records: InventoryUpdateRecord[] | undefined;
}

/// Small badge under any message that has been used as a trigger for
/// an inventory update. Shorthand only — just names, no descriptions —
/// so the user can see at a glance that this message has already been
/// used without the full detail (which lives in the inventory_update
/// message card that follows in chat history). Clicking the Package
/// button again produces a fresh update and overwrites this record.
export function InventoryUpdateBadge({ records }: Props) {
  if (!records || records.length === 0) return null;

  const clauses = records
    .map(renderShorthand)
    .filter((c): c is string => c !== null);

  if (clauses.length === 0) return null;

  return (
    <div className="mt-2 px-2.5 py-1.5 rounded-md bg-amber-500/15 border border-amber-500/25 flex items-center gap-1.5 text-[11px] text-amber-100 italic leading-snug">
      <Package size={11} className="flex-shrink-0 text-amber-400" />
      <span>
        <span className="font-medium not-italic text-amber-300">Inventory updated</span>
        <span className="text-amber-400/70"> · </span>
        <span className="not-italic">{clauses.join("; ")}</span>
      </span>
    </div>
  );
}

function renderShorthand(r: InventoryUpdateRecord): string | null {
  const parts: string[] = [];
  if (r.added.length > 0) parts.push(`+${quote(r.added)}`);
  if (r.updated.length > 0) parts.push(`~${quote(r.updated)}`);
  // Only show removes when they exceed adds (asymmetric swap). Balanced
  // removes are implicit in the adds.
  if (r.removed.length > r.added.length) {
    const tail = r.removed.slice(r.added.length);
    parts.push(`−${quote(tail)}`);
  }
  if (parts.length === 0) return null;
  return `${r.character_name}: ${parts.join(", ")}`;
}

function quote(names: string[]): string {
  return names.map((n) => `"${n}"`).join(", ");
}

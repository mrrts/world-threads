import { useEffect, useState } from "react";
import { Loader2, X, Package, Heart, Trash2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { api, type InventoryItem } from "@/lib/tauri";

/** Max inventory slots. Mirror of orchestrator::INVENTORY_MAX_ITEMS. */
const MAX_SLOTS = 10;

type Kind = "physical" | "interior";

/**
 * Editable inventory UI for the character settings page. Up to
 * MAX_SLOTS items, each tagged physical or interior via a small
 * toggle on the slot card. Saves are debounced via
 * `setCharacterInventory`, which also stamps the current world-day so
 * the next focus-refresh doesn't blow the user's edit away.
 */
export function InventoryEditor({
  characterId,
  initial,
  onSaved,
}: {
  characterId: string | undefined;
  initial: InventoryItem[];
  onSaved?: (inventory: InventoryItem[]) => void;
}) {
  const normalize = (arr: InventoryItem[]): InventoryItem[] =>
    arr.slice(0, MAX_SLOTS).map((it) => ({
      name: it?.name ?? "",
      description: it?.description ?? "",
      kind: (it?.kind === "interior" ? "interior" : "physical") as Kind,
    }));

  const [slots, setSlots] = useState<InventoryItem[]>(normalize(initial));
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setSlots(normalize(initial));
    setDirty(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [characterId, JSON.stringify(initial)]);

  useEffect(() => {
    if (!dirty || !characterId) return;
    const timer = setTimeout(async () => {
      const payload = slots
        .map((it) => ({
          name: (it.name ?? "").trim(),
          description: (it.description ?? "").trim(),
          kind: (it.kind === "interior" ? "interior" : "physical") as Kind,
        }))
        .filter((it) => it.name.length > 0);
      try {
        setSaving(true);
        const saved = await api.setCharacterInventory(characterId, payload);
        setDirty(false);
        onSaved?.(saved);
      } catch {
        // ignore — next edit retries
      } finally {
        setSaving(false);
      }
    }, 600);
    return () => clearTimeout(timer);
  }, [slots, dirty, characterId, onSaved]);

  const update = (index: number, patch: Partial<InventoryItem>) => {
    setSlots((prev) => {
      const next = prev.slice();
      next[index] = { ...next[index], ...patch };
      return next;
    });
    setDirty(true);
  };

  const addSlot = (kind: Kind) => {
    if (slots.length >= MAX_SLOTS) return;
    setSlots((prev) => [...prev, { name: "", description: "", kind }]);
    setDirty(true);
  };

  const removeSlot = (index: number) => {
    setSlots((prev) => prev.filter((_, i) => i !== index));
    setDirty(true);
  };

  // Two-step confirm: first click arms, second click commits. Arming
  // auto-disarms after 3s so a stray click doesn't sit loaded forever.
  const [clearArmed, setClearArmed] = useState(false);
  useEffect(() => {
    if (!clearArmed) return;
    const t = setTimeout(() => setClearArmed(false), 3000);
    return () => clearTimeout(t);
  }, [clearArmed]);
  const clearAll = () => {
    if (!clearArmed) { setClearArmed(true); return; }
    setClearArmed(false);
    setSlots([]);
    setDirty(true);
  };

  const physicalCount = slots.filter((s) => (s.kind ?? "physical") !== "interior" && s.name.trim()).length;
  const interiorCount = slots.filter((s) => s.kind === "interior" && s.name.trim()).length;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between text-[10px] uppercase tracking-wider text-muted-foreground/70 font-semibold min-h-[14px]">
        <span>{slots.length}/{MAX_SLOTS} slots · {physicalCount} physical · {interiorCount} interior</span>
        {saving ? (
          <span className="flex items-center gap-1 text-muted-foreground"><Loader2 size={10} className="animate-spin" /> saving…</span>
        ) : dirty ? (
          <span className="text-muted-foreground/60">unsaved…</span>
        ) : null}
      </div>
      <div className="space-y-2">
        {slots.map((slot, i) => {
          const kind: Kind = (slot.kind === "interior" ? "interior" : "physical");
          const filled = !!slot.name.trim();
          return (
            <div
              key={i}
              className={`rounded-lg border p-3 space-y-2 transition-colors ${
                filled ? "border-border bg-card/50" : "border-dashed border-border/50 bg-transparent"
              }`}
            >
              <div className="flex items-center gap-2">
                <div className="flex rounded-md overflow-hidden border border-input flex-shrink-0">
                  <button
                    onClick={() => update(i, { kind: "physical" })}
                    className={`px-2 py-1 text-[10px] font-medium flex items-center gap-1 transition-colors cursor-pointer ${
                      kind === "physical"
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                    }`}
                    title="Physical item — something they carry"
                  >
                    <Package size={10} /> physical
                  </button>
                  <button
                    onClick={() => update(i, { kind: "interior" })}
                    className={`px-2 py-1 text-[10px] font-medium flex items-center gap-1 transition-colors cursor-pointer ${
                      kind === "interior"
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                    }`}
                    title="Interior — a memory, truth, or feeling they carry inside"
                  >
                    <Heart size={10} /> interior
                  </button>
                </div>
                <Input
                  value={slot.name}
                  onChange={(e) => update(i, { name: e.target.value })}
                  placeholder={kind === "interior" ? "e.g. the ache of being seen" : "e.g. folded map to the ferry"}
                  className="flex-1"
                />
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-9 w-9 text-muted-foreground hover:text-destructive flex-shrink-0"
                  onClick={() => removeSlot(i)}
                  title="Remove slot"
                >
                  <X size={14} />
                </Button>
              </div>
              <Textarea
                value={slot.description}
                onChange={(e) => update(i, { description: e.target.value })}
                placeholder={kind === "interior"
                  ? "One sentence — the shape of the feeling or memory"
                  : "One sentence — context, use, or wear"}
                rows={2}
                className="text-xs"
                disabled={!filled}
              />
            </div>
          );
        })}
      </div>
      {slots.length < MAX_SLOTS && (
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => addSlot("physical")} className="flex-1">
            <Package size={12} className="mr-1.5" /> Add physical
          </Button>
          <Button variant="outline" size="sm" onClick={() => addSlot("interior")} className="flex-1">
            <Heart size={12} className="mr-1.5" /> Add interior
          </Button>
        </div>
      )}
      {slots.length > 0 && (
        <div className="pt-1 flex justify-end">
          <Button
            variant="ghost"
            size="sm"
            onClick={clearAll}
            className={`text-xs ${clearArmed ? "text-destructive hover:bg-destructive/10" : "text-muted-foreground/60 hover:text-destructive hover:bg-destructive/10"}`}
          >
            <Trash2 size={12} className="mr-1.5" />
            {clearArmed ? "Click again to confirm" : "Clear all items"}
          </Button>
        </div>
      )}
    </div>
  );
}

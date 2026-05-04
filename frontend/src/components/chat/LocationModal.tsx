import { useEffect, useMemo, useRef, useState } from "react";
import { MapPin, X, Trash2, ChevronDown, Library } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { Message } from "@/lib/tauri";

interface SavedPlace {
  saved_place_id: string;
  world_id: string;
  name: string;
  created_at: string;
  /// Last time this place was set as a chat's current location. Backend
  /// orders the dropdown by this DESC, so the most-recently-used place
  /// always appears first.
  last_used_at: string;
  // Forward-compatible optional fields. Backend doesn't ship these
  // yet — when it does (image upload + description + facts), the
  // dropdown rows below will pick them up without further refactor.
  image_url?: string | null;
  description?: string | null;
  facts?: string[] | null;
}

interface ChatLocationResponse {
  current_location: string | null;
  message: Message | null;
  saved_place: SavedPlace | null;
}

interface Props {
  open: boolean;
  onClose: () => void;
  worldId: string;
  /// Exactly one of characterId / groupChatId should be set, matching
  /// the chat surface this modal is attached to.
  characterId?: string;
  groupChatId?: string;
  currentLocation: string | null;
  /// Optional API key. When supplied, set_chat_location_cmd fires a
  /// background derivation refresh on real (non-no-op) location changes
  /// so the (world, name) entry in the location_derivations cache is
  /// populated for future dialogue calls. Skipped silently when null
  /// (the prompt path renders gracefully without a derivation).
  apiKey?: string | null;
  /// Called after a successful update with the new location string and
  /// the inserted location_change message (when one was inserted —
  /// null on no-op same-as-previous updates).
  onUpdated: (newLocation: string, insertedMessage: Message | null) => void;
}

/// Modal for setting / changing the current chat location. Includes:
///   - one-line input (initialized to currentLocation if set)
///   - dropdown of saved places (selecting populates input)
///   - "save this place" checkbox (DISABLED when trimmed input matches
///     any existing saved place — keeps the UNIQUE(world_id, name)
///     constraint unreachable in normal use)
export function LocationModal({
  open,
  onClose,
  worldId,
  characterId,
  groupChatId,
  currentLocation,
  apiKey,
  onUpdated,
}: Props) {
  const [input, setInput] = useState(currentLocation ?? "");
  const [saveToLibrary, setSaveToLibrary] = useState(false);
  const [places, setPlaces] = useState<SavedPlace[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    setInput(currentLocation ?? "");
    setSaveToLibrary(false);
    setError(null);
    setDropdownOpen(false);
    invoke<SavedPlace[]>("list_saved_places_cmd", { worldId })
      .then(setPlaces)
      .catch((e) => setError(String(e)));
    setTimeout(() => inputRef.current?.focus(), 30);
  }, [open, worldId, currentLocation]);

  // Close the dropdown when clicking outside its trigger / panel.
  useEffect(() => {
    if (!dropdownOpen) return;
    const handler = (e: MouseEvent) => {
      if (!dropdownRef.current) return;
      if (!dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [dropdownOpen]);

  const trimmed = input.trim();
  const matchesSavedPlace = useMemo(
    () => trimmed.length > 0 && places.some((p) => p.name.toLowerCase() === trimmed.toLowerCase()),
    [trimmed, places],
  );

  // Auto-uncheck Save when the input matches a saved place.
  useEffect(() => {
    if (matchesSavedPlace && saveToLibrary) setSaveToLibrary(false);
  }, [matchesSavedPlace, saveToLibrary]);

  if (!open) return null;

  const handleUpdate = async () => {
    if (!trimmed) {
      setError("Location must not be empty");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const res = await invoke<ChatLocationResponse>("set_chat_location_cmd", {
        characterId,
        groupChatId,
        location: trimmed,
        saveToLibrary,
        apiKey: apiKey ?? null,
      });
      onUpdated(trimmed, res.message ?? null);
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleDeletePlace = async (id: string) => {
    try {
      await invoke("delete_saved_place_cmd", { savedPlaceId: id });
      setPlaces((prev) => prev.filter((p) => p.saved_place_id !== id));
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div
      className="fixed inset-0 z-[80] flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="bg-card border border-border rounded-xl shadow-2xl shadow-black/50 w-full max-w-md mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-5 py-3 border-b border-border/50">
          <div className="flex items-center gap-2">
            <MapPin size={16} className="text-emerald-400" />
            <h2 className="text-sm font-semibold">Update location</h2>
          </div>
          <button
            onClick={onClose}
            className="text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
          >
            <X size={16} />
          </button>
        </div>

        <div className="p-5 space-y-4">
          {currentLocation && (
            <p className="text-xs text-muted-foreground">
              Currently: <span className="text-foreground/80">{currentLocation}</span>
            </p>
          )}
          <div>
            <label className="text-xs font-medium text-muted-foreground block mb-1.5">
              New location
            </label>
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !loading) handleUpdate();
                if (e.key === "Escape") onClose();
              }}
              placeholder="Ryan's House, the porch, the kitchen table…"
              className="w-full px-3 py-2 text-sm rounded-md border border-border bg-background focus:outline-none focus:ring-2 focus:ring-emerald-500/40 focus:border-emerald-500/40"
            />
          </div>

          <label
            className={`flex items-center gap-2 text-xs ${
              matchesSavedPlace ? "opacity-50 cursor-not-allowed" : "cursor-pointer"
            }`}
            title={matchesSavedPlace ? "Already in your saved places" : undefined}
          >
            <input
              type="checkbox"
              checked={saveToLibrary}
              disabled={matchesSavedPlace}
              onChange={(e) => setSaveToLibrary(e.target.checked)}
              className="rounded border-border cursor-pointer disabled:cursor-not-allowed"
            />
            <span>Save this place to your world's library</span>
          </label>

          {places.length > 0 && (
            <div>
              <label className="text-xs font-medium text-muted-foreground block mb-1.5">
                Saved places
              </label>
              <div className="relative" ref={dropdownRef}>
                <button
                  type="button"
                  onClick={() => setDropdownOpen((v) => !v)}
                  className="w-full flex items-center justify-between gap-2 px-3 py-2 text-sm rounded-md border border-border bg-background hover:bg-accent/40 transition-colors cursor-pointer focus:outline-none focus:ring-2 focus:ring-emerald-500/40 focus:border-emerald-500/40"
                >
                  <span className="flex items-center gap-2 text-muted-foreground/90">
                    <Library size={14} className="text-emerald-400/80" />
                    <span>Choose from {places.length} saved {places.length === 1 ? "place" : "places"}</span>
                  </span>
                  <ChevronDown
                    size={14}
                    className={`text-muted-foreground/60 transition-transform ${dropdownOpen ? "rotate-180" : ""}`}
                  />
                </button>

                {dropdownOpen && (
                  <div className="absolute left-0 right-0 top-full mt-1.5 z-50 rounded-lg border border-border bg-card shadow-2xl shadow-black/40 overflow-hidden animate-in fade-in slide-in-from-top-1 duration-150">
                    <div className="max-h-72 overflow-y-auto py-1">
                      {places.map((p) => (
                        <SavedPlaceRow
                          key={p.saved_place_id}
                          place={p}
                          onPick={() => {
                            setInput(p.name);
                            setDropdownOpen(false);
                            inputRef.current?.focus();
                          }}
                          onDelete={() => handleDeletePlace(p.saved_place_id)}
                        />
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {error && <p className="text-xs text-destructive">{error}</p>}
        </div>

        <div className="flex items-center justify-end gap-2 px-5 py-3 border-t border-border/50 bg-card/40">
          <button
            onClick={onClose}
            className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
          >
            Cancel
          </button>
          <button
            onClick={handleUpdate}
            disabled={loading || !trimmed}
            className="px-4 py-1.5 text-sm font-medium rounded-md bg-emerald-600 hover:bg-emerald-500 text-white disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer transition-colors"
          >
            {loading ? "Updating…" : "Update Location"}
          </button>
        </div>
      </div>
    </div>
  );
}

/// Row inside the saved-places dropdown. Built rich enough to host
/// future per-place metadata: a left-side image slot (square thumbnail
/// or fallback colored monogram), the place name (always present),
/// and a reserved area below for an optional one-line description and
/// a row of fact-pill chips. When those backend fields ship, the row
/// already has the layout to present them — no further refactor.
function SavedPlaceRow({
  place,
  onPick,
  onDelete,
}: {
  place: SavedPlace;
  onPick: () => void;
  onDelete: () => void;
}) {
  const monogram = (place.name.match(/[A-Za-z]/)?.[0] ?? "•").toUpperCase();
  const description = place.description?.trim() || null;
  const facts = (place.facts ?? []).filter((f) => f && f.trim().length > 0);

  return (
    <div className="group/row flex items-stretch gap-3 px-3 py-2 hover:bg-accent/40 transition-colors">
      <button
        type="button"
        onClick={onPick}
        className="flex-1 flex items-stretch gap-3 text-left cursor-pointer min-w-0"
      >
        <div className="flex-shrink-0 w-10 h-10 rounded-md bg-gradient-to-br from-emerald-700/30 to-emerald-900/40 border border-emerald-700/30 flex items-center justify-center overflow-hidden">
          {place.image_url ? (
            <img src={place.image_url} alt="" className="w-full h-full object-cover" />
          ) : (
            <span className="text-emerald-300/80 text-sm font-semibold">{monogram}</span>
          )}
        </div>
        <div className="flex-1 min-w-0 flex flex-col justify-center">
          <span className="text-sm text-foreground/90 truncate group-hover/row:text-foreground transition-colors">
            {place.name}
          </span>
          {description && (
            <span className="text-[11px] text-muted-foreground/80 truncate">
              {description}
            </span>
          )}
          {facts.length > 0 && (
            <div className="mt-1 flex flex-wrap gap-1">
              {facts.slice(0, 3).map((f, i) => (
                <span
                  key={i}
                  className="text-[10px] px-1.5 py-0.5 rounded-full bg-emerald-950/50 text-emerald-200/80 border border-emerald-700/30"
                >
                  {f}
                </span>
              ))}
            </div>
          )}
        </div>
      </button>
      <button
        type="button"
        onClick={onDelete}
        className="flex-shrink-0 self-center opacity-0 group-hover/row:opacity-100 text-muted-foreground hover:text-destructive transition-all p-1.5 rounded cursor-pointer"
        title="Remove from saved places"
      >
        <Trash2 size={13} />
      </button>
    </div>
  );
}

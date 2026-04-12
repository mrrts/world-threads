import { useState, useRef, useCallback, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter } from "@/components/ui/dialog";
import { Plus, Archive, ArchiveRestore, ChevronRight, Globe, Sparkles, User, Settings2 } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import { api, type WorldImageInfo } from "@/lib/tauri";

interface Props {
  store: ReturnType<typeof useAppStore>;
  onNavigate?: (view: string) => void;
}

export function Sidebar({ store, onNavigate }: Props) {
  const [showNewWorld, setShowNewWorld] = useState(false);
  const [worldName, setWorldName] = useState("");
  const [showNewChar, setShowNewChar] = useState(false);
  const [charName, setCharName] = useState("");
  const [showArchived, setShowArchived] = useState(false);
  const [showGroupPicker, setShowGroupPicker] = useState(false);
  const [selectedGroupMembers, setSelectedGroupMembers] = useState<string[]>([]);
  const [userAvatarUrl, setUserAvatarUrl] = useState("");

  useEffect(() => {
    if (!store.activeWorld) { setUserAvatarUrl(""); return; }
    api.getUserAvatar(store.activeWorld.world_id).then((url) => setUserAvatarUrl(url || ""));
  }, [store.activeWorld?.world_id, store.userProfile?.avatar_file]);
  const [hoverWorld, setHoverWorld] = useState<string | null>(null);
  const [hoverChar, setHoverChar] = useState<string | null>(null);
  const [hoverGroup, setHoverGroup] = useState<string | null>(null);
  const [worldImageCache, setWorldImageCache] = useState<Record<string, WorldImageInfo | null>>({});
  const hoverTimerRef = useRef<ReturnType<typeof setTimeout>>();

  const showWorldTooltip = useCallback((worldId: string) => {
    clearTimeout(hoverTimerRef.current);
    hoverTimerRef.current = setTimeout(() => setHoverWorld(worldId), 400);
    if (!(worldId in worldImageCache)) {
      api.getActiveWorldImage(worldId).then((img) =>
        setWorldImageCache((c) => ({ ...c, [worldId]: img }))
      ).catch(() => {});
    }
  }, [worldImageCache]);

  const hideWorldTooltip = useCallback(() => {
    clearTimeout(hoverTimerRef.current);
    hoverTimerRef.current = setTimeout(() => setHoverWorld(null), 300);
  }, []);

  const showCharTooltip = useCallback((charId: string) => {
    clearTimeout(hoverTimerRef.current);
    hoverTimerRef.current = setTimeout(() => setHoverChar(charId), 400);
  }, []);

  const hideCharTooltip = useCallback(() => {
    clearTimeout(hoverTimerRef.current);
    setHoverChar(null);
  }, []);

  const showGroupTooltip = useCallback((groupId: string) => {
    clearTimeout(hoverTimerRef.current);
    hoverTimerRef.current = setTimeout(() => setHoverGroup(groupId), 400);
  }, []);

  const hideGroupTooltip = useCallback(() => {
    clearTimeout(hoverTimerRef.current);
    setHoverGroup(null);
  }, []);

  const submitWorld = async () => {
    if (!worldName.trim()) return;
    await store.createWorld(worldName.trim());
    setWorldName("");
    setShowNewWorld(false);
  };

  const submitChar = async () => {
    if (!charName.trim()) return;
    await store.createCharacter(charName.trim());
    setCharName("");
    setShowNewChar(false);
  };

  return (
    <>
      <div className="w-56 flex-shrink-0 bg-card/50 border-r border-border flex flex-col">
        <div className="p-3 border-b border-border">
          <div className="flex items-center justify-between mb-2">
            <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Worlds</h2>
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => { setWorldName(""); setShowNewWorld(true); }}>
              <Plus size={14} />
            </Button>
          </div>
          <div className="space-y-0.5">
            {store.worlds.length === 0 && (
              <p className="text-xs text-muted-foreground/50 italic px-2 py-3 text-center">No worlds yet</p>
            )}
            {store.worlds.map((w) => (
              <div key={w.world_id} className="relative"
                onMouseEnter={() => showWorldTooltip(w.world_id)}
                onMouseLeave={hideWorldTooltip}
              >
                <button
                  onClick={() => store.selectWorld(w)}
                  className={`w-full text-left rounded-lg text-sm transition-all cursor-pointer overflow-hidden ${
                    store.activeWorld?.world_id === w.world_id
                      ? "bg-primary/15 text-primary font-medium"
                      : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                  }`}
                >
                  <span className="block px-2.5 py-1.5">{w.name}</span>
                  {store.activeWorld?.world_id === w.world_id && store.activeWorldImage?.data_url && (
                    <img
                      src={store.activeWorldImage.data_url}
                      alt=""
                      className="w-full h-20 object-cover rounded-b-lg"
                    />
                  )}
                </button>
                {hoverWorld === w.world_id && (
                  <div
                    onMouseEnter={() => clearTimeout(hoverTimerRef.current)}
                    onMouseLeave={hideWorldTooltip}
                    className="absolute left-full top-0 ml-2 z-50 w-64 bg-card border border-border rounded-xl shadow-2xl shadow-black/40 overflow-hidden animate-in fade-in zoom-in-95 duration-150">
                    {(worldImageCache[w.world_id] ?? (w.world_id === store.activeWorld?.world_id ? store.activeWorldImage : null))?.data_url && (
                      <img
                        src={(worldImageCache[w.world_id] ?? store.activeWorldImage)!.data_url}
                        alt=""
                        className="w-full h-32 object-cover"
                      />
                    )}
                    <div className="p-3">
                      <p className="font-semibold text-sm">{w.name}</p>
                      {w.description ? (
                        <p className="text-xs text-muted-foreground mt-1 leading-relaxed whitespace-pre-wrap">
                          {w.description}
                        </p>
                      ) : (
                        <p className="text-xs text-muted-foreground/50 italic mt-1">No description</p>
                      )}
                      <button
                        onClick={() => {
                          if (store.activeWorld?.world_id !== w.world_id) store.selectWorld(w);
                          onNavigate?.("summary");
                          setHoverWorld(null);
                        }}
                        className="mt-2 text-xs font-medium text-primary hover:text-primary/80 hover:underline cursor-pointer"
                      >
                        View World
                      </button>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>

        {store.activeWorld && (
          <div className="flex-1 flex flex-col min-h-0">
            <div className="p-3 border-b border-border">
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Characters</h2>
                <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => { setCharName(""); setShowNewChar(true); }}>
                  <Plus size={14} />
                </Button>
              </div>
              <div className="space-y-0.5">
                <button
                  onClick={() => { store.selectUserProfile(); onNavigate?.("character"); }}
                  className={`w-full text-left px-2.5 py-1.5 rounded-lg text-sm transition-all flex items-center gap-2.5 cursor-pointer ${
                    store.editingUserProfile
                      ? "bg-primary/15 text-primary font-medium"
                      : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                  }`}
                >
                  {userAvatarUrl ? (
                    <img src={userAvatarUrl} alt="" className="w-6 h-6 rounded-full object-cover flex-shrink-0" />
                  ) : (
                    <div className="w-6 h-6 rounded-full bg-primary/20 flex items-center justify-center flex-shrink-0">
                      <User size={13} className="text-primary" />
                    </div>
                  )}
                  {store.userProfile?.display_name || "Me"}
                  <span className="ml-auto text-[10px] font-medium bg-primary/15 text-primary px-1.5 py-0.5 rounded-full leading-none">Me</span>
                </button>

                <div className="border-b border-border/50 my-1" />

                {store.characters.map((ch) => {
                  const portrait = store.activePortraits[ch.character_id];
                  const isActive = store.activeCharacter?.character_id === ch.character_id;
                  return (
                  <div key={ch.character_id} className="relative flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg group"
                    onMouseEnter={() => showCharTooltip(ch.character_id)}
                    onMouseLeave={hideCharTooltip}
                  >
                    {portrait?.data_url ? (
                      <img src={portrait.data_url} alt="" className="w-8 h-8 rounded-full object-cover flex-shrink-0 ring-1 ring-border" />
                    ) : (
                      <span
                        className="w-8 h-8 rounded-full flex-shrink-0 ring-1 ring-white/10"
                        style={{ backgroundColor: ch.avatar_color }}
                      />
                    )}
                    <button
                      onClick={() => { store.selectCharacter(ch); onNavigate?.("chat"); }}
                      className={`text-sm flex-1 truncate text-left cursor-pointer hover:underline ${isActive && !store.editingUserProfile ? "text-primary font-medium" : "text-muted-foreground"}`}
                    >
                      {ch.display_name}
                    </button>
                    <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 flex-shrink-0 transition-opacity">
                      <button
                        onClick={() => { store.selectCharacter(ch); onNavigate?.("character"); }}
                        className="h-6 w-6 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors cursor-pointer"
                        title="Settings"
                      >
                        <Settings2 size={12} />
                      </button>
                      <button
                        onClick={() => store.archiveCharacter(ch.character_id)}
                        className="h-6 w-6 rounded-md flex items-center justify-center text-muted-foreground hover:text-amber-500 hover:bg-accent/50 transition-colors cursor-pointer"
                        title="Archive"
                      >
                        <Archive size={10} />
                      </button>
                    </div>
                    {hoverChar === ch.character_id && (
                      <div className="absolute left-full top-0 ml-2 z-50 w-56 bg-card border border-border rounded-xl shadow-2xl shadow-black/40 overflow-hidden animate-in fade-in zoom-in-95 duration-150 pointer-events-none">
                        <div className="flex items-center gap-3 p-3">
                          {portrait?.data_url ? (
                            <img src={portrait.data_url} alt="" className="w-14 h-14 rounded-full object-cover ring-2 ring-border flex-shrink-0" />
                          ) : (
                            <div className="w-14 h-14 rounded-full flex-shrink-0 ring-2 ring-white/10" style={{ backgroundColor: ch.avatar_color }} />
                          )}
                          <div className="flex-1 min-w-0">
                            <p className="font-semibold text-sm truncate">{ch.display_name}</p>
                          </div>
                        </div>
                        {ch.identity && (
                          <div className="px-3 pb-3 -mt-1">
                            <p className="text-xs text-muted-foreground leading-relaxed whitespace-pre-wrap">
                              {ch.identity}
                            </p>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                  );
                })}

                {/* Group Chats */}
                <div className="border-b border-border/50 my-1" />
                <button
                  onClick={() => setShowGroupPicker(true)}
                  className="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60 hover:text-muted-foreground transition-colors cursor-pointer px-2.5 py-1 w-full"
                >
                  <Plus size={10} />
                  Group
                </button>

                {store.groupChats.map((gc) => {
                  const isActive = store.activeGroupChat?.group_chat_id === gc.group_chat_id;
                  const charIds: string[] = Array.isArray(gc.character_ids) ? gc.character_ids : [];
                  const charNames = charIds.map((cid) => store.characters.find((c) => c.character_id === cid)?.display_name).filter(Boolean);
                  const groupChars = charIds.map((cid) => store.characters.find((c) => c.character_id === cid)).filter(Boolean) as typeof store.characters;
                  return (
                    <div key={gc.group_chat_id} className="relative"
                      onMouseEnter={() => showGroupTooltip(gc.group_chat_id)}
                      onMouseLeave={hideGroupTooltip}
                    >
                      <button
                        onClick={() => { store.selectGroupChat(gc); onNavigate?.("chat"); }}
                        className={`flex items-center gap-2.5 px-2.5 py-2 rounded-lg w-full text-left transition-colors cursor-pointer ${
                          isActive ? "bg-accent" : "hover:bg-accent/50"
                        }`}
                      >
                        <div className="flex -space-x-2.5 flex-shrink-0">
                          {charIds.map((cid, i) => {
                            const p = store.activePortraits[cid];
                            return p?.data_url ? (
                              <img key={cid} src={p.data_url} alt="" className="w-8 h-8 rounded-full object-cover ring-2 ring-card" style={{ zIndex: charIds.length - i }} />
                            ) : (
                              <span key={cid} className="w-8 h-8 rounded-full ring-2 ring-card bg-muted" style={{ zIndex: charIds.length - i }} />
                            );
                          })}
                        </div>
                        <div className="flex-1 min-w-0">
                          <span className={`text-[10px] uppercase tracking-wider ${isActive ? "text-primary/60" : "text-muted-foreground/50"}`}>Group</span>
                          <p className={`text-sm truncate leading-tight ${isActive ? "text-primary font-medium" : "text-muted-foreground"}`}>
                            {charNames.join(" & ")}
                          </p>
                        </div>
                      </button>
                      {hoverGroup === gc.group_chat_id && (
                        <div className="absolute left-full top-0 ml-2 z-50 w-[420px] bg-card border border-border rounded-xl shadow-2xl shadow-black/40 overflow-hidden animate-in fade-in zoom-in-95 duration-150 pointer-events-none">
                          <div className="grid grid-cols-2 divide-x divide-border">
                            {groupChars.map((ch) => {
                              const portrait = store.activePortraits[ch.character_id];
                              return (
                                <div key={ch.character_id} className="p-3">
                                  <div className="flex flex-col items-center mb-2">
                                    {portrait?.data_url ? (
                                      <img src={portrait.data_url} alt="" className="w-16 h-16 rounded-full object-cover ring-2 ring-border" />
                                    ) : (
                                      <div className="w-16 h-16 rounded-full ring-2 ring-white/10" style={{ backgroundColor: ch.avatar_color }} />
                                    )}
                                    <p className="font-semibold text-sm mt-2">{ch.display_name}</p>
                                  </div>
                                  {ch.identity && (
                                    <p className="text-xs text-muted-foreground leading-relaxed whitespace-pre-wrap">
                                      {ch.identity}
                                    </p>
                                  )}
                                </div>
                              );
                            })}
                          </div>
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              {store.archivedCharacters.length > 0 && (
                <div className="mt-2 pt-2 border-t border-border/50">
                  <button
                    onClick={() => setShowArchived((v) => !v)}
                    className="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60 hover:text-muted-foreground transition-colors cursor-pointer px-2.5 w-full"
                  >
                    <ChevronRight size={10} className={`transition-transform ${showArchived ? "rotate-90" : ""}`} />
                    Archived ({store.archivedCharacters.length})
                  </button>
                  {showArchived && (
                    <div className="space-y-0.5 mt-1">
                      {store.archivedCharacters.map((ch) => {
                        const portrait = store.activePortraits[ch.character_id];
                        return (
                          <div key={ch.character_id} className="flex items-center gap-2 px-2.5 py-1 rounded-lg group">
                            {portrait?.data_url ? (
                              <img src={portrait.data_url} alt="" className="w-5 h-5 rounded-full object-cover flex-shrink-0 ring-1 ring-border opacity-50" />
                            ) : (
                              <span
                                className="w-2 h-2 rounded-full flex-shrink-0 ring-1 ring-white/10 opacity-50"
                                style={{ backgroundColor: ch.avatar_color }}
                              />
                            )}
                            <span className="text-xs flex-1 truncate text-muted-foreground/50">
                              {ch.display_name}
                            </span>
                            <button
                              onClick={() => store.unarchiveCharacter(ch.character_id)}
                              className="h-5 w-5 rounded-md flex items-center justify-center text-muted-foreground/50 hover:text-primary hover:bg-accent/50 transition-all opacity-0 group-hover:opacity-100 cursor-pointer flex-shrink-0"
                              title="Restore"
                            >
                              <ArchiveRestore size={10} />
                            </button>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              )}
            </div>

            <ScrollArea className="flex-1 p-3">
              <div className="space-y-3">
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Globe size={12} />
                  <span className="text-[10px] font-semibold uppercase tracking-wider">World State</span>
                </div>
                {store.activeWorld.state.time && (
                  <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                    <Sparkles size={10} className="text-primary/50" />
                    <span>Day {store.activeWorld.state.time.day_index}</span>
                    <button
                      onClick={() => {
                        if (!store.activeWorld) return;
                        const newState = structuredClone(store.activeWorld.state);
                        newState.time.day_index += 1;
                        store.updateWorldState(newState);
                      }}
                      className="w-4 h-4 rounded flex items-center justify-center text-muted-foreground/50 hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
                      title="Advance day"
                    >
                      <Plus size={10} />
                    </button>
                    <span className="text-border">·</span>
                    <select
                      value={store.activeWorld.state.time.time_of_day}
                      onChange={(e) => {
                        if (!store.activeWorld) return;
                        const newState = structuredClone(store.activeWorld.state);
                        newState.time.time_of_day = e.target.value;
                        store.updateWorldState(newState);
                      }}
                      className="bg-transparent border-none text-xs text-muted-foreground cursor-pointer focus:outline-none hover:text-foreground transition-colors p-0"
                    >
                      {["DAWN", "MORNING", "MIDDAY", "AFTERNOON", "EVENING", "NIGHT", "LATE NIGHT"].map((t) => (
                        <option key={t} value={t}>{t}</option>
                      ))}
                    </select>
                  </div>
                )}
              </div>
            </ScrollArea>
          </div>
        )}
      </div>

      {/* New World Modal */}
      <Dialog open={showNewWorld} onClose={() => setShowNewWorld(false)}>
        <DialogContent>
          <DialogHeader onClose={() => setShowNewWorld(false)}>
            <DialogTitle>Create a New World</DialogTitle>
            <DialogDescription>Give your world a name. You can edit all the details after.</DialogDescription>
          </DialogHeader>
          <DialogBody>
            <Input
              autoFocus
              placeholder="e.g. The Floating Isles, Nightfall Harbor..."
              value={worldName}
              onChange={(e) => setWorldName(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") submitWorld(); }}
            />
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowNewWorld(false)}>Cancel</Button>
            <Button onClick={submitWorld} disabled={!worldName.trim()}>Create World</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* New Character Modal */}
      <Dialog open={showNewChar} onClose={() => setShowNewChar(false)}>
        <DialogContent>
          <DialogHeader onClose={() => setShowNewChar(false)}>
            <DialogTitle>Add a Character</DialogTitle>
            <DialogDescription>Name your new character. You can flesh out their canon later.</DialogDescription>
          </DialogHeader>
          <DialogBody>
            <Input
              autoFocus
              placeholder="e.g. Mara, Ion, Wren..."
              value={charName}
              onChange={(e) => setCharName(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") submitChar(); }}
            />
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowNewChar(false)}>Cancel</Button>
            <Button onClick={submitChar} disabled={!charName.trim()}>Add Character</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Group Chat Picker */}
      <Dialog open={showGroupPicker} onClose={() => { setShowGroupPicker(false); setSelectedGroupMembers([]); }}>
        <DialogContent>
          <DialogHeader onClose={() => { setShowGroupPicker(false); setSelectedGroupMembers([]); }}>
            <DialogTitle>Create Group Chat</DialogTitle>
            <DialogDescription>Select 2 characters to start a group conversation.</DialogDescription>
          </DialogHeader>
          <DialogBody>
            <div className="grid grid-cols-2 gap-2">
              {store.characters.map((ch) => {
                const portrait = store.activePortraits[ch.character_id];
                const selected = selectedGroupMembers.includes(ch.character_id);
                return (
                  <button
                    key={ch.character_id}
                    onClick={() => {
                      setSelectedGroupMembers((prev) =>
                        selected
                          ? prev.filter((id) => id !== ch.character_id)
                          : prev.length >= 2 ? prev : [...prev, ch.character_id]
                      );
                    }}
                    className={`flex items-center gap-2.5 p-2.5 rounded-xl border-2 transition-all cursor-pointer ${
                      selected ? "border-primary bg-primary/10" : "border-border hover:border-primary/40"
                    }`}
                  >
                    {portrait?.data_url ? (
                      <img src={portrait.data_url} alt="" className="w-10 h-10 rounded-full object-cover flex-shrink-0" />
                    ) : (
                      <div className="w-10 h-10 rounded-full flex-shrink-0" style={{ backgroundColor: ch.avatar_color }} />
                    )}
                    <span className="text-sm font-medium truncate">{ch.display_name}</span>
                  </button>
                );
              })}
            </div>
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => { setShowGroupPicker(false); setSelectedGroupMembers([]); }}>Cancel</Button>
            <Button
              disabled={selectedGroupMembers.length < 2}
              onClick={async () => {
                await store.createGroupChat(selectedGroupMembers);
                setShowGroupPicker(false);
                setSelectedGroupMembers([]);
                onNavigate?.("chat");
              }}
            >
              Create ({selectedGroupMembers.length}/2)
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

    </>
  );
}

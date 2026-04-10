import { useState, useEffect } from "react";
import { api, type PortraitInfo } from "@/lib/tauri";

interface Props {
  characterId: string;
}

export function PortraitPopout({ characterId }: Props) {
  const [portrait, setPortrait] = useState<PortraitInfo | null>(null);
  const [characterName, setCharacterName] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const [p, ch] = await Promise.all([
          api.getActivePortrait(characterId),
          api.getCharacter(characterId),
        ]);
        setPortrait(p);
        setCharacterName(ch.display_name);
      } catch {
        // ignore
      } finally {
        setLoading(false);
      }
    })();
  }, [characterId]);

  if (loading) {
    return (
      <div className="h-screen bg-background flex items-center justify-center">
        <div className="animate-pulse text-primary text-2xl">...</div>
      </div>
    );
  }

  if (!portrait?.data_url) {
    return (
      <div className="h-screen bg-background flex items-center justify-center text-muted-foreground text-sm">
        No portrait available
      </div>
    );
  }

  return (
    <div className="h-screen bg-background flex flex-col overflow-hidden">
      <div
        data-tauri-drag-region
        className="h-8 flex-shrink-0 flex items-center pl-[72px] pr-3 bg-card border-b border-border select-none"
      >
        <span className="text-xs text-muted-foreground truncate">{characterName}</span>
      </div>
      <div className="flex-1 flex items-center justify-center p-2 min-h-0">
        <img
          src={portrait.data_url}
          alt={characterName}
          className="max-w-full max-h-full object-contain rounded-lg"
        />
      </div>
    </div>
  );
}

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";
import { Undo2, Clock, Zap } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";

interface Props {
  store: ReturnType<typeof useAppStore>;
}

export function WorldFeed({ store }: Props) {
  const [showRetcon, setShowRetcon] = useState(false);

  if (!store.activeWorld) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <div className="text-center space-y-2">
          <p className="text-lg">No world selected</p>
          <p className="text-sm text-muted-foreground/60">Select a world to see its timeline</p>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="flex-1 flex flex-col min-h-0">
        <div className="px-6 py-3 border-b border-border flex items-center justify-between">
          <div className="flex items-center gap-2">
            <h1 className="font-semibold">World Feed</h1>
            <Badge variant="secondary" className="text-[10px]">{store.worldEvents.length} events</Badge>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowRetcon(true)}
            disabled={store.worldEvents.length === 0}
          >
            <Undo2 size={13} className="mr-1.5" /> Retcon Last Tick
          </Button>
        </div>

        <ScrollArea className="flex-1 px-6 py-4">
          {store.worldEvents.length === 0 ? (
            <div className="text-center text-muted-foreground py-16">
              <Clock size={32} className="mx-auto mb-3 text-muted-foreground/30" />
              <p className="font-medium">No world events yet</p>
              <p className="text-sm mt-1 text-muted-foreground/60">Events will appear here as you chat with characters</p>
            </div>
          ) : (
            <div className="max-w-2xl space-y-3">
              {[...store.worldEvents].reverse().map((evt) => (
                <div key={evt.event_id} className="bg-card border border-border rounded-xl p-4 hover:border-border/80 transition-colors">
                  <div className="flex items-center gap-2 mb-2.5">
                    <Badge variant="secondary" className="text-[10px] font-mono">
                      Day {evt.day_index}
                    </Badge>
                    <Badge variant="outline" className="text-[10px]">
                      {evt.time_of_day}
                    </Badge>
                    <div className="flex-1" />
                    <div className="flex items-center gap-1 text-[10px] text-muted-foreground/50">
                      <Zap size={8} />
                      {evt.trigger_type.replace(/_/g, " ")}
                    </div>
                  </div>
                  <p className="text-sm leading-relaxed">{evt.summary}</p>
                  {evt.hooks && evt.hooks.length > 0 && (
                    <div className="mt-3 pt-2.5 border-t border-border/50 flex flex-wrap gap-1.5">
                      {evt.hooks.map((hook, i) => (
                        <span key={i} className="text-[10px] text-primary/60 bg-primary/5 px-2 py-0.5 rounded-full">
                          ↪ {hook}
                        </span>
                      ))}
                    </div>
                  )}
                  <p className="text-[10px] text-muted-foreground/40 mt-2.5">
                    {new Date(evt.created_at).toLocaleString()}
                  </p>
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
      </div>

      <Dialog open={showRetcon} onClose={() => setShowRetcon(false)}>
        <DialogContent>
          <DialogHeader onClose={() => setShowRetcon(false)}>
            <DialogTitle>Retcon Last Tick</DialogTitle>
            <DialogDescription>
              This will undo the most recent world tick event. The event and its effects will be removed. Use this if something went wrong.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowRetcon(false)}>Cancel</Button>
            <Button variant="destructive" onClick={async () => {
              await store.retconLastTick();
              setShowRetcon(false);
            }}>
              Undo Last Tick
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

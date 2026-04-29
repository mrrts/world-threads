import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Dialog } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import type { Message } from "@/lib/tauri";
import { Gamepad2, Sparkles, Trophy, X } from "lucide-react";

const STORAGE = (chatId: string) => `arcade_v1_${chatId}`;

type ArcadeStats = {
  jewels: number;
  crowns: number;
  dollars: number;
  combo: number;
  comboUntil: number;
  hype: number;
  riddlesSolved: number;
  secrets: string[];
};

const DEFAULT_STATS: ArcadeStats = {
  jewels: 0,
  crowns: 0,
  dollars: 0,
  combo: 0,
  comboUntil: 0,
  hype: 0,
  riddlesSolved: 0,
  secrets: [],
};

const RIDDLES: { q: string; a: string }[] = [
  { q: "I have keys but no locks. I have space but no room. You can enter but not go outside. What am I?", a: "A keyboard (arcade edition) ⌨️" },
  { q: "What gets wetter as it dries?", a: "A towel — dry humor, wet truth." },
  { q: "Speak without a mouth, hear without ears. I have no body, but come alive with wind.", a: "An echo… or a really good system prompt." },
  { q: "The more you take, the more you leave behind. What?", a: "Footsteps — and unread lore tabs." },
  { q: "I’m tall when young, short when old. I glow with life, then ash with cold.", a: "A candle — and also your hype meter if you stop feeding it." },
];

function loadStats(chatId: string): ArcadeStats {
  try {
    const raw = localStorage.getItem(STORAGE(chatId));
    if (!raw) return { ...DEFAULT_STATS };
    const o = JSON.parse(raw) as Partial<ArcadeStats>;
    return { ...DEFAULT_STATS, ...o, secrets: Array.isArray(o.secrets) ? o.secrets : [] };
  } catch {
    return { ...DEFAULT_STATS };
  }
}

function saveStats(chatId: string, s: ArcadeStats) {
  try {
    localStorage.setItem(STORAGE(chatId), JSON.stringify(s));
  } catch { /* private mode */ }
}

function riddleIndex(chatId: string): number {
  let h = 0;
  for (let i = 0; i < chatId.length; i++) h = (h * 31 + chatId.charCodeAt(i)) | 0;
  return Math.abs(h) % RIDDLES.length;
}

function playableTail(messages: Message[]): Message | null {
  for (let i = messages.length - 1; i >= 0; i--) {
    const m = messages[i];
    if (m.role === "user" || m.role === "assistant" || m.role === "narrative") return m;
  }
  return null;
}

type Props = {
  chatId: string;
  messages: Message[];
};

export function ArcadeGameModeHUD({ chatId, messages }: Props) {
  const [stats, setStats] = useState<ArcadeStats>(() => loadStats(chatId));
  const [toast, setToast] = useState<string | null>(null);
  const [burst, setBurst] = useState<string[]>([]);
  const [dashboard, setDashboard] = useState(false);
  const [riddleOpen, setRiddleOpen] = useState(false);
  const [riddleSolved, setRiddleSolved] = useState(false);
  const crownClicks = useRef<number[]>([]);
  const lastPlayableId = useRef<string | null>(null);

  const riddle = useMemo(() => RIDDLES[riddleIndex(chatId)], [chatId]);

  useEffect(() => {
    setStats(loadStats(chatId));
    setRiddleSolved(false);
    setRiddleOpen(false);
    lastPlayableId.current = null;
  }, [chatId]);

  const pushToast = useCallback((t: string) => {
    setToast(t);
    window.setTimeout(() => setToast(null), 2600);
  }, []);

  const fireworks = useCallback((emojis: string) => {
    const arr = emojis.split("");
    setBurst(arr);
    window.setTimeout(() => setBurst([]), 900);
  }, []);

  const apply = useCallback(
    (fn: (s: ArcadeStats) => ArcadeStats) => {
      setStats((prev) => {
        const next = fn(prev);
        saveStats(chatId, next);
        return next;
      });
    },
    [chatId]
  );

  useEffect(() => {
    const tail = playableTail(messages);
    if (!tail?.message_id) return;

    if (lastPlayableId.current === null) {
      lastPlayableId.current = tail.message_id;
      return;
    }
    if (lastPlayableId.current === tail.message_id) return;
    lastPlayableId.current = tail.message_id;

    const now = Date.now();
    let toastMsg: string | null = null;
    let burstStr = "";

    setStats((s) => {
      const comboAlive = now < s.comboUntil;
      let next = { ...s };

      if (tail.role === "user") {
        const gain = 5 + Math.floor(Math.random() * 21);
        next = {
          ...next,
          dollars: next.dollars + gain,
          combo: comboAlive ? next.combo + 1 : 1,
          comboUntil: now + 45_000,
          hype: Math.min(100, next.hype + 4),
        };
        toastMsg = `+${gain} ARCADE BUCKS`;
        burstStr = "💵✨🎮";
      } else if (tail.role === "assistant" || tail.role === "narrative") {
        const bonus = 1 + Math.floor((comboAlive ? next.combo : 0) / 3);
        const prevJ = next.jewels;
        const jewels = prevJ + bonus;
        const crownGain = Math.floor(jewels / 15) - Math.floor(prevJ / 15);
        const crowns = next.crowns + crownGain;
        if (crownGain > 0) {
          toastMsg = crownGain > 1 ? `${crownGain} CROWN DROPS!` : "CROWN DROP — royal combo!";
          burstStr = "👑💎🔥";
        } else {
          toastMsg = `+${bonus} jewel${bonus > 1 ? "s" : ""}`;
          burstStr = "💎⚡";
        }
        next = {
          ...next,
          jewels,
          crowns,
          combo: comboAlive ? next.combo + 1 : 1,
          comboUntil: now + 50_000,
          hype: Math.min(100, next.hype + 7),
        };
      }
      saveStats(chatId, next);
      return next;
    });

    if (toastMsg) pushToast(toastMsg);
    if (burstStr) fireworks(burstStr);
  }, [messages, chatId, fireworks, pushToast]);

  const onCrownTap = () => {
    const now = Date.now();
    crownClicks.current = crownClicks.current.filter((t) => now - t < 2000);
    crownClicks.current.push(now);
    if (crownClicks.current.length >= 3) {
      crownClicks.current = [];
      apply((s) => {
        if (s.secrets.includes("crown_triple")) return s;
        const secrets = [...s.secrets, "crown_triple"];
        fireworks("🤐👑💰✨");
        pushToast("SECRET UNLOCKED: Crown Konami — vault spills 500!");
        return { ...s, secrets, dollars: s.dollars + 500, crowns: s.crowns + 1 };
      });
    }
  };

  const solveRiddle = () => {
    if (riddleSolved) return;
    setRiddleSolved(true);
    apply((s) => ({
      ...s,
      riddlesSolved: s.riddlesSolved + 1,
      dollars: s.dollars + 42,
      jewels: s.jewels + 3,
      hype: Math.min(100, s.hype + 12),
    }));
    fireworks("🧩✨🏆");
    pushToast("Riddle cracked — brain loot acquired");
  };

  const achievements = useMemo(() => {
    const a: { id: string; label: string; done: boolean; emoji: string }[] = [
      { id: "j1", label: "First sparkle", done: stats.jewels >= 1, emoji: "💎" },
      { id: "j50", label: "Bling runner", done: stats.jewels >= 50, emoji: "✨" },
      { id: "c3", label: "Triple crown", done: stats.crowns >= 3, emoji: "👑" },
      { id: "d1k", label: "High roller", done: stats.dollars >= 1000, emoji: "💵" },
      { id: "r1", label: "Riddle scholar", done: stats.riddlesSolved >= 1, emoji: "🧩" },
      { id: "sec", label: "Secret society", done: stats.secrets.length > 0, emoji: "🤐" },
    ];
    return a;
  }, [stats]);

  const hypePct = stats.hype;

  return (
    <>
      <div
        className="mb-3 rounded-xl border-2 border-fuchsia-500/60 bg-gradient-to-r from-violet-950/90 via-fuchsia-950/85 to-cyan-950/90 px-3 py-2 shadow-[0_0_24px_rgba(217,70,239,0.25)] font-mono text-[11px] leading-tight text-fuchsia-100/95 relative overflow-hidden"
        role="region"
        aria-label="Arcade game mode HUD"
      >
        {burst.length > 0 && (
          <div className="pointer-events-none absolute inset-0 flex items-center justify-center gap-1 text-2xl animate-pulse" aria-hidden>
            {burst.map((e, i) => (
              <span key={`${e}-${i}`} className="drop-shadow-[0_0_8px_rgba(255,255,255,0.8)]">{e}</span>
            ))}
          </div>
        )}
        <div className="flex flex-wrap items-center justify-between gap-2 relative z-10">
          <div className="flex items-center gap-2 text-[10px] uppercase tracking-widest text-fuchsia-200/90">
            <Gamepad2 className="w-3.5 h-3.5 text-cyan-300" />
            <span>Arcade HUD</span>
            <span className="text-cyan-300/80">LIVE</span>
          </div>
          <button
            type="button"
            onClick={() => setDashboard(true)}
            className="rounded-md bg-black/40 px-2 py-0.5 text-[10px] uppercase tracking-wide text-amber-200 border border-amber-400/40 hover:bg-amber-500/20 transition-colors"
          >
            🎰 Dashboard
          </button>
        </div>
        <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 relative z-10">
          <button type="button" onClick={onCrownTap} className="flex items-center gap-0.5 hover:scale-105 transition-transform cursor-pointer" title="Triple-tap for a secret…">
            <span>👑</span>
            <span className="text-amber-200">{stats.crowns}</span>
          </button>
          <span className="flex items-center gap-0.5">
            <span>💎</span>
            <span className="text-cyan-200">{stats.jewels}</span>
          </span>
          <span className="flex items-center gap-0.5">
            <span>💵</span>
            <span className="text-emerald-200">{stats.dollars}</span>
          </span>
          <span className="flex items-center gap-0.5 text-orange-200">
            <span>🔥</span>
            <span>×{stats.combo}</span>
          </span>
          <span className="flex items-center gap-0.5 text-pink-200">
            <span>⚡</span>
            <span>HYPE {hypePct}%</span>
          </span>
        </div>
        <div className="mt-1.5 h-1.5 rounded-full bg-black/50 overflow-hidden border border-white/10 relative z-10">
          <div
            className="h-full bg-gradient-to-r from-fuchsia-400 via-amber-300 to-cyan-400 transition-all duration-500"
            style={{ width: `${hypePct}%` }}
          />
        </div>
        <div className="mt-2 flex items-start justify-between gap-2 relative z-10">
          <div className="flex-1 min-w-0">
            <p className="text-[10px] uppercase tracking-wide text-fuchsia-300/80 mb-0.5">Side quest — riddle</p>
            <p className="text-[11px] text-white/90 leading-snug">{riddle.q}</p>
            <div className="mt-1 flex flex-wrap gap-1">
              <button
                type="button"
                onClick={() => setRiddleOpen((v) => !v)}
                className="text-[10px] px-1.5 py-0.5 rounded bg-white/10 hover:bg-white/20 border border-white/20"
              >
                {riddleOpen ? "Hide" : "Peek"} 🧿
              </button>
              {!riddleSolved && (
                <button
                  type="button"
                  onClick={solveRiddle}
                  className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-600/80 hover:bg-emerald-500 border border-emerald-300/40"
                >
                  Claim solve (+loot)
                </button>
              )}
              {riddleSolved && <span className="text-[10px] text-emerald-300">Cleared ✓</span>}
            </div>
            {riddleOpen && (
              <p className="mt-1 text-[10px] text-cyan-100/90 border-l-2 border-cyan-400/50 pl-2 italic">
                {riddle.a}
              </p>
            )}
          </div>
          <span className="text-xl shrink-0" title="Jewel stash mood">
            {stats.jewels > 80 ? "🏆" : stats.jewels > 30 ? "🎪" : "🕹️"}
          </span>
        </div>
      </div>

      {toast && (
        <div className="fixed bottom-24 left-1/2 -translate-x-1/2 z-[100] px-4 py-2 rounded-full bg-black/85 border border-fuchsia-500/50 text-sm text-fuchsia-100 shadow-lg animate-in fade-in zoom-in-95 duration-200 font-mono">
          {toast}
        </div>
      )}

      <Dialog open={dashboard} onClose={() => setDashboard(false)} className="max-w-lg w-[calc(100vw-2rem)]">
        <div className="p-5 space-y-4 bg-gradient-to-b from-violet-950/95 to-black/95 border-2 border-fuchsia-500/40 rounded-2xl shadow-2xl text-fuchsia-50 max-h-[85vh] overflow-y-auto">
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <Trophy className="w-5 h-5 text-amber-300" />
              <h3 className="font-bold text-lg tracking-tight">Arcade Command Center</h3>
            </div>
            <button type="button" onClick={() => setDashboard(false)} className="p-1 rounded-lg hover:bg-white/10" aria-label="Close">
              <X className="w-5 h-5" />
            </button>
          </div>
          <p className="text-xs text-fuchsia-200/80 leading-relaxed">
            Play money, jewels, and crowns are <strong className="text-amber-200">cosmetic fun</strong> for this thread — they do not touch billing, canon, or the model. Triple-tap the 👑 on the HUD (three times within two seconds) for a secret payout.
          </p>
          <div className="grid grid-cols-3 gap-2 text-center">
            <div className="rounded-xl bg-black/40 border border-fuchsia-500/30 p-3">
              <div className="text-2xl mb-1">💎</div>
              <div className="text-xl font-bold text-cyan-200">{stats.jewels}</div>
              <div className="text-[10px] uppercase text-fuchsia-300/80">Jewels</div>
            </div>
            <div className="rounded-xl bg-black/40 border border-amber-500/30 p-3">
              <div className="text-2xl mb-1">👑</div>
              <div className="text-xl font-bold text-amber-200">{stats.crowns}</div>
              <div className="text-[10px] uppercase text-amber-300/80">Crowns</div>
            </div>
            <div className="rounded-xl bg-black/40 border border-emerald-500/30 p-3">
              <div className="text-2xl mb-1">💵</div>
              <div className="text-xl font-bold text-emerald-200">{stats.dollars}</div>
              <div className="text-[10px] uppercase text-emerald-300/80">Arcade bucks</div>
            </div>
          </div>
          <div>
            <div className="flex items-center gap-1 text-xs uppercase tracking-wide text-fuchsia-300/90 mb-2">
              <Sparkles className="w-3.5 h-3.5" />
              Achievements
            </div>
            <ul className="space-y-1.5">
              {achievements.map((x) => (
                <li
                  key={x.id}
                  className={`flex items-center gap-2 text-sm rounded-lg px-2 py-1.5 border ${
                    x.done ? "border-emerald-500/40 bg-emerald-950/40" : "border-white/10 bg-black/30 opacity-60"
                  }`}
                >
                  <span>{x.emoji}</span>
                  <span className={x.done ? "text-emerald-100" : "text-fuchsia-200/70"}>{x.label}</span>
                  {x.done && <span className="ml-auto text-emerald-300 text-xs">✓</span>}
                </li>
              ))}
            </ul>
          </div>
          <div className="border-t border-white/10 pt-3 space-y-2">
            <p className="text-xs font-semibold text-amber-200/90">Bonus riddles (just for kicks)</p>
            {RIDDLES.slice(0, 3).map((rd, i) => (
              <details key={i} className="text-xs bg-black/30 rounded-lg border border-white/10 px-2 py-1">
                <summary className="cursor-pointer text-fuchsia-200/90">{rd.q}</summary>
                <p className="mt-1 text-cyan-100/90 pl-2 border-l border-cyan-500/40">{rd.a}</p>
              </details>
            ))}
          </div>
          <Button variant="secondary" className="w-full" onClick={() => setDashboard(false)}>
            Back to the scene
          </Button>
        </div>
      </Dialog>
    </>
  );
}

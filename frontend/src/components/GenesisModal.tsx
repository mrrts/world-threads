import { useState, useEffect, useRef } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogBody, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Compass, Loader2, Sparkles, X, ChevronDown, ChevronRight, User, ImagePlus, Plus } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { Input } from "@/components/ui/input";
import { api, type GenesisStageEvent, type GenesisResult, type GenesisHints } from "@/lib/tauri";
import { WEATHER_OPTIONS } from "@/lib/weather";

const TIME_OF_DAY_OPTIONS: Array<{ id: string; label: string }> = [
  { id: "",            label: "Any (let the world decide)" },
  { id: "morning",     label: "Morning" },
  { id: "midday",      label: "Midday" },
  { id: "afternoon",   label: "Afternoon" },
  { id: "evening",     label: "Evening" },
  { id: "late night",  label: "Late night" },
];

interface Props {
  open: boolean;
  onClose: () => void;
  apiKey: string;
  googleApiKey: string;
  setApiKey: (key: string) => Promise<void>;
  setGoogleApiKey: (key: string) => Promise<void>;
  onWorldAccepted: (worldId: string) => void;
}

type Phase = "keys" | "idle" | "generating" | "error" | "define_self" | "reaching" | "reflecting" | "offering";

interface CharacterReveal {
  character_id: string;
  name: string;
  identity: string;
  avatar_color: string;
  portraitUrl?: string;
}

interface WorldReveal {
  name: string;
  description: string;
  imageUrl?: string;
}

/// The first-run experience + on-demand "dream me a world" path. The
/// user meets their world progressively as it's rendered — name + two
/// character names appear first (from the LLM's JSON), then the world
/// landscape image fades in, then each character's portrait as it
/// finishes painting. By the time generation completes, the user has
/// spent the wait meeting the place rather than staring at a spinner.
///
/// On completion the modal pivots to a commitment-ceremony phase —
/// "What do you want to build while you're here?" — agency-shaped,
/// builder-shaped (deliberately distinct from QuestAcceptanceDialog's
/// yearning-shaped "what are you reaching for here?"; the asymmetry
/// matches the asymmetry between accepting a pursuit and authoring
/// a place). The answer is saved as the world's first user-authored
/// quest, turning a model-generated world into a chosen home.
export function GenesisModal({ open, onClose, apiKey, googleApiKey: _googleApiKey, setApiKey, setGoogleApiKey, onWorldAccepted }: Props) {
  // If no OpenAI key is stored, start in the keys phase — the user
  // must provide one before they can dream a world. Google key is
  // always optional and surfaced alongside.
  const [phase, setPhase] = useState<Phase>(() => apiKey.trim() ? "idle" : "keys");
  const [openaiInput, setOpenaiInput] = useState("");
  const [googleInput, setGoogleInput] = useState("");
  const [savingKeys, setSavingKeys] = useState(false);
  const [keyError, setKeyError] = useState<string | null>(null);
  const [stageDetail, setStageDetail] = useState("Sketching the shape of a world…");
  const [progress, setProgress] = useState(0);
  const [history, setHistory] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<GenesisResult | null>(null);
  const [world, setWorld] = useState<WorldReveal | null>(null);
  const [characters, setCharacters] = useState<CharacterReveal[]>([]);
  // Define-self phase state. The primary "what do you look like?" is
  // kept distinct from the advanced "about you" so users who skip the
  // advanced section still get a clean appearance-only description
  // driving the avatar generation.
  const [selfAppearance, setSelfAppearance] = useState("");
  const [selfName, setSelfName] = useState("");
  const [selfAbout, setSelfAbout] = useState("");
  const [selfFacts, setSelfFacts] = useState<string[]>([]);
  const [selfFactInput, setSelfFactInput] = useState("");
  const [showAdvancedSelf, setShowAdvancedSelf] = useState(false);
  const [userAvatarUrl, setUserAvatarUrl] = useState("");
  const [painting, setPainting] = useState(false);
  const [selfError, setSelfError] = useState<string | null>(null);
  // Existing avatars from the user's other worlds — surfaces an "import
  // me from another world" affordance so a user who already has a
  // canon portrait of themselves elsewhere doesn't have to re-paint.
  // Loaded lazily when define_self phase opens. Empty for first-world
  // users; the import row simply won't render.
  const [importableAvatars, setImportableAvatars] = useState<Array<{ world_id: string; world_name: string; avatar_file: string; data_url: string }>>([]);
  const [importing, setImporting] = useState(false);

  const [reaching, setReaching] = useState("");
  const [nobleOffering, setNobleOffering] = useState("");
  const [committing, setCommitting] = useState(false);
  const unlistenRef = useRef<(() => void) | null>(null);

  // Pre-gen hint controls (optional, collapsed by default).
  const [showHints, setShowHints] = useState(false);
  const [toneHint, setToneHint] = useState("");
  const [timeHint, setTimeHint] = useState("");
  const [weatherHint, setWeatherHint] = useState("");

  useEffect(() => {
    if (!open) return;
    setPhase(apiKey.trim() ? "idle" : "keys");
    setOpenaiInput("");
    setGoogleInput("");
    setSavingKeys(false);
    setKeyError(null);
    setStageDetail("Sketching the shape of a world…");
    setProgress(0);
    setHistory([]);
    setError(null);
    setResult(null);
    setWorld(null);
    setCharacters([]);
    setSelfAppearance("");
    setSelfName("");
    setSelfAbout("");
    setSelfFacts([]);
    setSelfFactInput("");
    setShowAdvancedSelf(false);
    setUserAvatarUrl("");
    setPainting(false);
    setSelfError(null);
    setReaching("");
    setNobleOffering("");
    setCommitting(false);
    setShowHints(false);
    setToneHint("");
    setTimeHint("");
    setWeatherHint("");
  }, [open]);

  useEffect(() => () => {
    if (unlistenRef.current) { unlistenRef.current(); unlistenRef.current = null; }
  }, []);

  const startGeneration = async () => {
    if (!apiKey) { setError("No API key configured. Set one in Settings first."); setPhase("error"); return; }
    setPhase("generating");
    setError(null);
    setHistory([]);
    setProgress(0);
    setStageDetail("Sketching the shape of a world…");
    setWorld(null);
    setCharacters([]);

    const unlisten = await listen<GenesisStageEvent>("genesis-stage", (event) => {
      const { detail, progress: p, reveal } = event.payload;
      setStageDetail(detail);
      setProgress(p);
      setHistory((prev) => prev[prev.length - 1] === detail ? prev : [...prev, detail]);

      if (reveal) {
        if (reveal.kind === "world_named") {
          setWorld({ name: reveal.name, description: reveal.description });
        } else if (reveal.kind === "character_named") {
          setCharacters((prev) => {
            if (prev.some((c) => c.character_id === reveal.character_id)) return prev;
            return [...prev, {
              character_id: reveal.character_id,
              name: reveal.name,
              identity: reveal.identity,
              avatar_color: reveal.avatar_color,
            }];
          });
        } else if (reveal.kind === "world_image_ready") {
          // Fetch the world image and reveal it in the card
          api.getActiveWorldImage(reveal.world_id).then((img) => {
            if (img?.data_url) setWorld((w) => w ? { ...w, imageUrl: img.data_url } : w);
          }).catch(() => {});
        } else if (reveal.kind === "portrait_ready") {
          const charId = reveal.character_id;
          api.getActivePortrait(charId).then((p) => {
            if (p?.data_url) {
              setCharacters((prev) => prev.map((c) =>
                c.character_id === charId ? { ...c, portraitUrl: p.data_url } : c
              ));
            }
          }).catch(() => {});
        }
      }
    });
    unlistenRef.current = unlisten;

    try {
      const hints: GenesisHints = {
        tone: toneHint.trim() || null,
        time_of_day: timeHint.trim() || null,
        weather_key: weatherHint.trim() || null,
      };
      const anyHint = !!(hints.tone || hints.time_of_day || hints.weather_key);
      const res = await api.autoGenerateWorldWithCharacters(apiKey, anyHint ? hints : undefined);
      setResult(res);
      // Ensure a default UserProfile row exists for this world so the
      // subsequent avatar-generation call has something to update.
      // Quiet-safe — if profile already exists (shouldn't here, but), upsert
      // just overwrites with defaults before the user edits them.
      try {
        await api.updateUserProfile({
          world_id: res.world_id,
          display_name: "Me",
          description: "",
          facts: [],
          boundaries: [],
          avatar_file: "",
          updated_at: "",
        });
      } catch (err) { console.warn("[Genesis] could not seed default user profile:", err); }
      // Lazy-load any portraits the user has built in other worlds so
      // the import-from-another-world affordance can render. Filter out
      // the freshly-created world (which won't have a portrait yet).
      api.listAllUserAvatars().then((avatars) => {
        setImportableAvatars(avatars.filter((a) => a.world_id !== res.world_id));
      }).catch(() => { setImportableAvatars([]); });
      setPhase("define_self");
    } catch (e: any) {
      setError(String(e));
      setPhase("error");
    } finally {
      if (unlistenRef.current) { unlistenRef.current(); unlistenRef.current = null; }
    }
  };

  // Merge the primary appearance field with the optional advanced
  // "about you" into a single description — the avatar prompt reads
  // the description field, and a user who only fills the primary
  // appearance field shouldn't have to also duplicate it into advanced.
  const composedSelfDescription = () => {
    const appearance = selfAppearance.trim();
    const about = selfAbout.trim();
    if (appearance && about) return `${appearance}\n\n${about}`;
    return appearance || about;
  };

  const saveSelfProfile = async (): Promise<void> => {
    if (!result) return;
    await api.updateUserProfile({
      world_id: result.world_id,
      display_name: selfName.trim() || "Me",
      description: composedSelfDescription(),
      facts: selfFacts.filter((f) => f.trim()),
      boundaries: [],
      avatar_file: "",
      updated_at: "",
    });
  };

  // Import a portrait + identity from one of the user's other worlds.
  // Copies the avatar via setUserAvatarFromGallery (cheap — just
  // re-references the existing file by name; no re-painting), then
  // pre-fills appearance/name/facts from the source profile, but only
  // for fields the user hasn't already typed into. We don't overwrite
  // their in-progress entries.
  const onImportFromWorld = async (avatar: { world_id: string; avatar_file: string }) => {
    if (!result || importing) return;
    setImporting(true);
    setSelfError(null);
    try {
      const dataUrl = await api.setUserAvatarFromGallery(result.world_id, avatar.avatar_file);
      setUserAvatarUrl(dataUrl || "");
      const sourceProfile = await api.getUserProfile(avatar.world_id).catch(() => null);
      if (sourceProfile) {
        // Pre-fill ONLY blank fields — never clobber what the user
        // already typed. Treat appearance/name/about/facts as separate
        // imports so partial-edits don't get reset.
        if (!selfAppearance.trim() && sourceProfile.description) {
          setSelfAppearance(sourceProfile.description);
        }
        if (!selfName.trim() && sourceProfile.display_name && sourceProfile.display_name !== "Me") {
          setSelfName(sourceProfile.display_name);
        }
        const existingFacts = (sourceProfile.facts as unknown);
        const factsArr: string[] = Array.isArray(existingFacts)
          ? (existingFacts as unknown[]).filter((x): x is string => typeof x === "string")
          : [];
        if (selfFacts.filter((f) => f.trim()).length === 0 && factsArr.length > 0) {
          setSelfFacts(factsArr);
          // Open the advanced section so the imported facts are visible
          // and the user can see what got carried over.
          setShowAdvancedSelf(true);
        }
      }
      // Persist the freshly-imported state so the world has a real
      // profile row (mirrors what onPaintSelf does after a paint).
      await saveSelfProfile();
    } catch (e: any) {
      setSelfError(String(e));
    } finally {
      setImporting(false);
    }
  };

  const onPaintSelf = async () => {
    if (!result || painting) return;
    if (!selfAppearance.trim()) {
      setSelfError("Tell me what you look like first, then I'll paint you.");
      return;
    }
    setPainting(true);
    setSelfError(null);
    try {
      await saveSelfProfile();
      await api.generateUserAvatar(apiKey, result.world_id, {
        display_name: selfName.trim() || "Me",
        description: composedSelfDescription(),
      });
      // Fetch the freshly-generated data URL so we can render it in-place.
      const url = await api.getUserAvatar(result.world_id).catch(() => "");
      setUserAvatarUrl(url || "");
    } catch (e: any) {
      setSelfError(String(e));
    } finally {
      setPainting(false);
    }
  };

  const onContinueFromSelf = async () => {
    if (!result || committing) return;
    setCommitting(true);
    try {
      // Save whatever the user has entered (safe even if they didn't
      // click Paint — the profile just has no avatar).
      await saveSelfProfile();
      setPhase("reaching");
    } catch (e: any) {
      setSelfError(String(e));
    } finally {
      setCommitting(false);
    }
  };

  const onOfferForReflection = async () => {
    if (!result || committing) return;
    const text = reaching.trim();
    if (!text) return;
    setCommitting(true);
    setError(null);
    setPhase("reflecting");
    try {
      const offering = await api.reflectReachingAsNobleQuest(apiKey, result.world_id, text);
      setNobleOffering(offering);
      setPhase("offering");
    } catch (e: any) {
      setError(String(e));
      setPhase("reaching");
    } finally {
      setCommitting(false);
    }
  };

  const onAccept = async () => {
    if (!result || committing) return;
    setCommitting(true);
    try {
      // Title = the noble reflection (the dignified naming that gets
      // shown in the Quests list). Description = the user's own words
      // (so their voice persists alongside the reflection that named
      // them back).
      await api.createQuest(
        result.world_id,
        nobleOffering.trim() || "What I want to build while I'm here",
        reaching.trim(),
        "user_authored",
        undefined,
      );
      onWorldAccepted(result.world_id);
      onClose();
    } catch (e: any) {
      setError(String(e));
      setCommitting(false);
    }
  };

  const onRevise = () => {
    setPhase("reaching");
    setNobleOffering("");
    setError(null);
  };

  const onSkip = () => {
    if (!result) return;
    onWorldAccepted(result.world_id);
    onClose();
  };

  const onSaveKeys = async () => {
    const openai = openaiInput.trim();
    if (!openai) { setKeyError("Please paste your OpenAI API key to continue."); return; }
    if (!openai.startsWith("sk-")) { setKeyError('OpenAI keys start with "sk-". Please check what you pasted.'); return; }
    setSavingKeys(true);
    setKeyError(null);
    try {
      await setApiKey(openai);
      const google = googleInput.trim();
      if (google) { await setGoogleApiKey(google); }
      setPhase("idle");
    } catch (e: any) {
      setKeyError(String(e));
    } finally {
      setSavingKeys(false);
    }
  };

  return (
    <Dialog open={open} onClose={phase === "generating" || phase === "reflecting" ? () => {} : onClose} className="max-w-2xl">
      <DialogContent>
        {phase === "keys" && (
          <>
            <DialogHeader onClose={onClose}>
              <DialogTitle>
                <Sparkles size={16} className="inline mr-2 text-amber-400" />
                Let's get the worst part over with
              </DialogTitle>
            </DialogHeader>
            <DialogBody className="space-y-5">
              <p className="text-sm text-foreground/90 leading-relaxed">
                WorldThreads doesn't run its own AI — you bring your own key. That means the app is free
                and your data stays on your machine, but it also means the first five minutes involve
                a small dance with OpenAI's billing page. There's no way around it. Let's do it now and
                not look back.
              </p>

              {/* OpenAI (required) */}
              <div className="space-y-2.5 rounded-xl border-2 border-amber-400/40 bg-amber-500/5 p-4">
                <div className="flex items-center gap-2">
                  <span className="text-[10px] uppercase tracking-wider font-semibold text-amber-400">Required</span>
                  <p className="text-sm font-semibold">OpenAI API key</p>
                </div>
                <ol className="text-xs text-foreground/85 leading-relaxed space-y-1 list-decimal list-inside ml-1">
                  <li>Go to <a href="https://platform.openai.com" target="_blank" rel="noreferrer" className="text-amber-400 underline">platform.openai.com</a> and sign in (or create an account).</li>
                  <li>Open <span className="font-medium">Billing</span> and add a payment method. Put $5-$10 of credit on the account — this lasts a long time in normal play.</li>
                  <li>Open <span className="font-medium">API keys</span> and create a new secret key. Copy it immediately — OpenAI won't show it again.</li>
                  <li>Paste it below. It'll be stored securely in your system keychain, never sent anywhere but OpenAI.</li>
                </ol>
                <Input
                  autoFocus
                  type="password"
                  value={openaiInput}
                  onChange={(e) => setOpenaiInput(e.target.value)}
                  placeholder="sk-..."
                  className="font-mono text-xs"
                />
              </div>

              {/* Google (optional) */}
              <div className="space-y-2.5 rounded-xl border border-border/60 bg-card/30 p-4">
                <div className="flex items-center gap-2">
                  <span className="text-[10px] uppercase tracking-wider text-muted-foreground/70">Optional</span>
                  <p className="text-sm font-semibold text-foreground/80">Google AI API key</p>
                  <span className="text-[11px] text-muted-foreground">— unlocks video generation</span>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  If you'd like to animate illustrations into short videos, you'll need a key from
                  Google AI Studio. If you don't want that feature right now, leave this blank —
                  you can add it later from Settings.
                </p>
                <ol className="text-xs text-foreground/75 leading-relaxed space-y-1 list-decimal list-inside ml-1">
                  <li>Visit <a href="https://aistudio.google.com/apikey" target="_blank" rel="noreferrer" className="text-amber-400 underline">aistudio.google.com/apikey</a>.</li>
                  <li>Create a new API key. Video generation via Veo requires a billing-enabled Google Cloud project.</li>
                  <li>Paste it below.</li>
                </ol>
                <Input
                  type="password"
                  value={googleInput}
                  onChange={(e) => setGoogleInput(e.target.value)}
                  placeholder="(optional — leave blank if you don't want video)"
                  className="font-mono text-xs"
                />
              </div>

              {keyError && <p className="text-xs text-destructive">{keyError}</p>}

              <p className="text-[11px] text-muted-foreground/60 italic">
                That's the worst part. After this, the app just makes worlds for you.
              </p>
            </DialogBody>
            <DialogFooter>
              <Button variant="ghost" onClick={onClose} disabled={savingKeys}>Leave setup for now</Button>
              <Button
                onClick={onSaveKeys}
                disabled={savingKeys || !openaiInput.trim()}
                className="bg-amber-500/90 hover:bg-amber-500 text-black"
              >
                {savingKeys ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <Sparkles size={14} className="mr-1.5" />}
                {savingKeys ? "Saving…" : "Save keys and keep going"}
              </Button>
            </DialogFooter>
          </>
        )}

        {phase === "idle" && (
          <>
            <DialogHeader onClose={onClose}>
              <DialogTitle>
                <Sparkles size={16} className="inline mr-2 text-amber-400" />
                A world is waiting to be dreamt
              </DialogTitle>
            </DialogHeader>
            <DialogBody className="space-y-4">
              <p className="text-sm text-foreground/90 leading-relaxed">
                In about a minute, this will dream up a new world — somewhere with weather and
                invariants and two people living in it. A hand-painted portrait for each.
                A landscape for the world itself. Their interior lives already populated.
                It'll surprise you; that's the point.
              </p>
              <div className="rounded-md border border-amber-400/30 bg-amber-500/5 p-3">
                <p className="text-xs text-muted-foreground/90 leading-relaxed italic">
                  The register the app reaches for: compelling, dramatic, varied, gently holy,
                  deeply fun. You can keep it or discard it and try another. You can edit
                  anything afterward.
                </p>
              </div>
              {/* Optional, collapsed by default. Users who just want a
                  surprise skip this entirely; users who have a specific
                  vibe in mind can steer without losing randomness on
                  everything else. */}
              <div>
                <button
                  onClick={() => setShowHints((v) => !v)}
                  className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                >
                  {showHints ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                  Set tone, time of day, or weather
                  <span className="text-[10px] text-muted-foreground/60">(optional)</span>
                </button>
                {showHints && (
                  <div className="mt-3 space-y-3 rounded-lg border border-border/50 bg-card/40 p-3">
                    <div className="space-y-1">
                      <label className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Tone</label>
                      <Input
                        value={toneHint}
                        onChange={(e) => setToneHint(e.target.value)}
                        placeholder='e.g. "melancholy winter" or "warm and musical"'
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-3">
                      <div className="space-y-1">
                        <label className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Time of day</label>
                        <select
                          value={timeHint}
                          onChange={(e) => setTimeHint(e.target.value)}
                          className="w-full h-9 px-3 rounded-md border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                        >
                          {TIME_OF_DAY_OPTIONS.map((o) => (
                            <option key={o.id} value={o.id}>{o.label}</option>
                          ))}
                        </select>
                      </div>
                      <div className="space-y-1">
                        <label className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Weather</label>
                        <select
                          value={weatherHint}
                          onChange={(e) => setWeatherHint(e.target.value)}
                          className="w-full h-9 px-3 rounded-md border border-input bg-background text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                        >
                          <option value="">Any (let the world decide)</option>
                          {WEATHER_OPTIONS.map((w) => (
                            <option key={w.id} value={w.id}>{w.emoji} {w.label}</option>
                          ))}
                        </select>
                      </div>
                    </div>
                    <p className="text-[10px] text-muted-foreground/60 italic">
                      Anything you leave as "Any" stays a surprise.
                    </p>
                  </div>
                )}
              </div>
              <p className="text-xs text-muted-foreground/70">
                Uses your OpenAI key. Takes 30-90 seconds.
              </p>
            </DialogBody>
            <DialogFooter>
              <Button variant="ghost" onClick={onClose}>Not yet</Button>
              <Button
                onClick={startGeneration}
                disabled={!apiKey}
                className="bg-amber-500/90 hover:bg-amber-500 text-black"
              >
                <Sparkles size={14} className="mr-1.5" />
                Dream a world
              </Button>
            </DialogFooter>
          </>
        )}

        {phase === "generating" && (
          <>
            <DialogHeader onClose={() => {}}>
              <DialogTitle>
                <Loader2 size={16} className="inline mr-2 text-amber-400 animate-spin" />
                {stageDetail}
              </DialogTitle>
            </DialogHeader>
            <DialogBody className="space-y-4">
              <div className="w-full h-1.5 rounded-full bg-muted/40 overflow-hidden">
                <div
                  className="h-full bg-amber-500 transition-all duration-500 ease-out"
                  style={{ width: `${Math.max(5, Math.round(progress * 100))}%` }}
                />
              </div>

              {/* Progressive reveal. The user meets their world as it
                  lands — not waiting on a load bar, but reading names
                  and descriptions that materialize in order. */}
              <div className="space-y-3">
                {world && (
                  <div className="rounded-xl overflow-hidden border border-amber-400/30 bg-amber-500/5 animate-in fade-in slide-in-from-bottom-2 duration-500">
                    {world.imageUrl ? (
                      <div className="relative w-full h-72 overflow-hidden">
                        <img src={world.imageUrl} alt="" className="w-full h-full object-cover animate-in fade-in duration-700" />
                        <div className="absolute inset-0 bg-gradient-to-t from-background/80 via-transparent to-transparent" />
                      </div>
                    ) : (
                      <div className="w-full h-72 bg-gradient-to-br from-amber-500/20 to-amber-500/5 flex items-center justify-center">
                        <div className="text-xs text-amber-300/60 italic flex items-center gap-2">
                          <Loader2 size={11} className="animate-spin" />
                          the land is being painted…
                        </div>
                      </div>
                    )}
                    <div className="p-3">
                      <p className="text-sm font-semibold text-amber-300">{world.name}</p>
                      <p className="text-xs text-foreground/80 mt-1 leading-relaxed">{world.description}</p>
                    </div>
                  </div>
                )}

                {characters.length > 0 && (
                  <div className="space-y-2">
                    {characters.map((c) => (
                      <div
                        key={c.character_id}
                        className="flex items-start gap-3 rounded-lg border border-border/50 bg-card/50 p-3 animate-in fade-in slide-in-from-bottom-2 duration-500"
                      >
                        {c.portraitUrl ? (
                          <img
                            src={c.portraitUrl}
                            alt=""
                            className="w-14 h-14 rounded-lg object-cover flex-shrink-0 ring-1 ring-border animate-in fade-in duration-700"
                          />
                        ) : (
                          <div
                            className="w-14 h-14 rounded-lg flex-shrink-0 ring-1 ring-white/10 flex items-center justify-center"
                            style={{ backgroundColor: c.avatar_color }}
                          >
                            <Loader2 size={14} className="animate-spin text-white/70" />
                          </div>
                        )}
                        <div className="flex-1 min-w-0">
                          <p className="text-sm font-medium text-foreground">{c.name}</p>
                          <p className="text-xs text-foreground/75 mt-0.5 leading-snug italic">
                            {c.identity.length > 180 ? `${c.identity.slice(0, 180).trimEnd()}…` : c.identity}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {!world && characters.length === 0 && (
                  <div className="flex items-center justify-center py-12 text-xs text-muted-foreground/60 italic">
                    {history[history.length - 1] ?? "Sketching…"}
                  </div>
                )}
              </div>

              <p className="text-[11px] text-muted-foreground/50 italic text-center">
                You're meeting them while they come into focus. This takes about a minute.
              </p>
            </DialogBody>
          </>
        )}

        {phase === "error" && (
          <>
            <DialogHeader onClose={onClose}>
              <DialogTitle>Something didn't land</DialogTitle>
            </DialogHeader>
            <DialogBody>
              <p className="text-sm text-destructive">{error ?? "Unknown error."}</p>
              <p className="text-xs text-muted-foreground mt-2">
                You can try again; each attempt samples fresh seeds, so the world will be different.
              </p>
            </DialogBody>
            <DialogFooter>
              <Button variant="ghost" onClick={onClose}>Leave this for now</Button>
              <Button onClick={startGeneration}>Dream a different world</Button>
            </DialogFooter>
          </>
        )}

        {phase === "define_self" && world && result && (
          <>
            <DialogHeader onClose={onContinueFromSelf}>
              <DialogTitle>
                <User size={16} className="inline mr-2 text-amber-400" />
                And you? Who walks in?
              </DialogTitle>
            </DialogHeader>
            {/* max-h + overflow-y-auto keeps the dialog from growing past
                the viewport when the form expands — importing a portrait
                from another world auto-opens the advanced section, which
                otherwise pushes the footer off-screen on shorter laptops. */}
            <DialogBody className="space-y-4 max-h-[65vh] overflow-y-auto">
              <p className="text-sm text-foreground/90 leading-relaxed">
                The world's been dreamt. The people in it are waiting. One small piece left —
                you, in their eyes. Short and easy; fuller if you want.
              </p>

              {/* Import-from-another-world affordance — only renders if the
                  user has at least one portrait in another world. Lets them
                  carry their already-canon self over instead of re-painting.
                  Renders as a small horizontal row above the appearance
                  field; on click, imports the portrait + pre-fills any
                  blank profile fields from the source world. */}
              {importableAvatars.length > 0 && (
                <div className="rounded-xl border border-border/60 bg-card/40 p-3 space-y-2">
                  <p className="text-xs text-muted-foreground">
                    Or carry yourself over from another world — pick a portrait you've already painted of yourself.
                  </p>
                  <div className="flex gap-2 overflow-x-auto pb-1">
                    {importableAvatars.map((a) => (
                      <button
                        key={`${a.world_id}-${a.avatar_file}`}
                        onClick={() => onImportFromWorld(a)}
                        disabled={importing || painting}
                        className="flex flex-col items-center gap-1 flex-shrink-0 group cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                        title={`Import from ${a.world_name}`}
                      >
                        <img
                          src={a.data_url}
                          alt=""
                          className="w-14 h-14 rounded-full object-cover ring-1 ring-border group-hover:ring-amber-400/60 transition-all"
                        />
                        <span className="text-[10px] text-muted-foreground/70 max-w-[60px] truncate">{a.world_name}</span>
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {/* Primary: appearance + paint action */}
              <div className="rounded-xl border border-amber-400/40 bg-amber-500/5 p-4 space-y-3">
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-foreground/90">What do you look like?</label>
                  <Textarea
                    autoFocus
                    className="min-h-[80px]"
                    value={selfAppearance}
                    onChange={(e) => { setSelfAppearance(e.target.value); setSelfError(null); }}
                    placeholder="A sentence or two — a face, a posture, a way of dressing. Plain description works."
                  />
                </div>

                <div className="flex items-start gap-4">
                  {userAvatarUrl ? (
                    <img
                      src={userAvatarUrl}
                      alt=""
                      className="w-28 h-28 rounded-xl object-cover ring-2 ring-amber-400/40 flex-shrink-0 animate-in fade-in duration-700"
                    />
                  ) : (
                    <div className="w-28 h-28 rounded-xl bg-muted/30 flex items-center justify-center flex-shrink-0 ring-1 ring-border">
                      {painting ? <Loader2 size={20} className="animate-spin text-amber-400" /> : <User size={26} className="text-muted-foreground/40" />}
                    </div>
                  )}
                  <div className="flex-1 space-y-1.5">
                    <Button
                      onClick={onPaintSelf}
                      disabled={painting || !selfAppearance.trim()}
                      className="bg-amber-500/90 hover:bg-amber-500 text-black"
                    >
                      {painting ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <ImagePlus size={14} className="mr-1.5" />}
                      {painting ? "Painting you…" : userAvatarUrl ? "Paint again" : "Paint me"}
                    </Button>
                    <p className="text-[11px] text-muted-foreground/70 leading-relaxed">
                      Uses what you wrote above. You can rewrite and repaint as many times as you want.
                    </p>
                  </div>
                </div>

                {selfError && <p className="text-xs text-destructive">{selfError}</p>}
              </div>

              {/* Advanced (collapsed by default) */}
              <div>
                <button
                  onClick={() => setShowAdvancedSelf((v) => !v)}
                  className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                >
                  {showAdvancedSelf ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                  More about me
                  <span className="text-[10px] text-muted-foreground/60">(optional)</span>
                </button>
                {showAdvancedSelf && (
                  <div className="mt-3 space-y-3 rounded-lg border border-border/50 bg-card/40 p-3">
                    <div className="space-y-1">
                      <label className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Your name</label>
                      <Input
                        value={selfName}
                        onChange={(e) => setSelfName(e.target.value)}
                        placeholder='What characters will call you (default: "Me")'
                      />
                    </div>
                    <div className="space-y-1">
                      <label className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Anything else about you</label>
                      <Textarea
                        className="min-h-[80px]"
                        value={selfAbout}
                        onChange={(e) => setSelfAbout(e.target.value)}
                        placeholder="Personality, vibe, what matters to you. Gets folded into your portrait and shown to characters."
                      />
                    </div>
                    <div className="space-y-1.5">
                      <label className="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Things characters should know</label>
                      <div className="space-y-1.5">
                        {selfFacts.map((f, i) => (
                          <div key={i} className="flex items-center gap-2 group">
                            <span className="w-1.5 h-1.5 rounded-full bg-primary/40 flex-shrink-0" />
                            <Input
                              className="flex-1"
                              value={f}
                              onChange={(e) => setSelfFacts(selfFacts.map((v, j) => j === i ? e.target.value : v))}
                            />
                            <button
                              onClick={() => setSelfFacts(selfFacts.filter((_, j) => j !== i))}
                              className="h-7 w-7 rounded-md flex items-center justify-center text-muted-foreground/50 hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
                            >
                              <X size={12} />
                            </button>
                          </div>
                        ))}
                        <div className="flex items-center gap-2">
                          <span className="w-1.5 h-1.5 rounded-full bg-border flex-shrink-0" />
                          <Input
                            className="flex-1"
                            value={selfFactInput}
                            onChange={(e) => setSelfFactInput(e.target.value)}
                            placeholder="e.g. Lives near the coast. Writes with a fountain pen."
                            onKeyDown={(e) => {
                              if (e.key === "Enter" && selfFactInput.trim()) {
                                e.preventDefault();
                                setSelfFacts([...selfFacts, selfFactInput.trim()]);
                                setSelfFactInput("");
                              }
                            }}
                          />
                          <Button
                            variant="outline"
                            size="sm"
                            className="h-9 flex-shrink-0"
                            disabled={!selfFactInput.trim()}
                            onClick={() => {
                              if (!selfFactInput.trim()) return;
                              setSelfFacts([...selfFacts, selfFactInput.trim()]);
                              setSelfFactInput("");
                            }}
                          >
                            <Plus size={12} />
                          </Button>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </DialogBody>
            <DialogFooter>
              <Button
                onClick={onContinueFromSelf}
                disabled={committing}
                className="bg-amber-500/90 hover:bg-amber-500 text-black"
              >
                {committing ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <Compass size={14} className="mr-1.5" />}
                Step into what you want to build
              </Button>
            </DialogFooter>
          </>
        )}

        {phase === "reaching" && world && (
          <>
            <DialogHeader onClose={onSkip}>
              <DialogTitle>
                <Compass size={16} className="inline mr-2 text-amber-400" />
                What do you want to build while you're here?
              </DialogTitle>
            </DialogHeader>
            <DialogBody className="space-y-4">
              {/* Keep the world card visible — the user's looking at
                  the place they're about to commit to, not a blank prompt. */}
              <div className="rounded-xl overflow-hidden border border-amber-400/30 bg-amber-500/5">
                {world.imageUrl && (
                  <div className="relative w-full h-28 overflow-hidden">
                    <img src={world.imageUrl} alt="" className="w-full h-full object-cover" />
                    <div className="absolute inset-0 bg-gradient-to-t from-background/80 via-transparent to-transparent" />
                  </div>
                )}
                <div className="p-3">
                  <p className="text-sm font-semibold text-amber-300">{world.name}</p>
                  {characters.length > 0 && (
                    <p className="text-[11px] text-muted-foreground mt-0.5">
                      with {characters.map((c) => c.name).join(" and ")}
                    </p>
                  )}
                </div>
              </div>

              <p className="text-sm text-foreground/90 leading-relaxed">
                Before you step in: one honest sentence about what you want to build, do, or make happen here.
              </p>
              <p className="text-xs text-muted-foreground italic leading-relaxed">
                Not a goal — a project worth your attention. Whatever you write becomes the first quest in this world,
                waiting there for you. You can skip if nothing's ready to say yet.
              </p>
              <Textarea
                autoFocus
                className="min-h-[100px]"
                value={reaching}
                onChange={(e) => setReaching(e.target.value)}
                placeholder="Plain, specific, honest. Doesn't have to be clever."
              />
              {error && <p className="text-xs text-destructive">{error}</p>}
            </DialogBody>
            <DialogFooter>
              <Button variant="ghost" onClick={onSkip} disabled={committing}>
                <X size={14} className="mr-1.5" />
                Skip — just let me in
              </Button>
              <Button
                onClick={onOfferForReflection}
                disabled={committing || !reaching.trim()}
                className="bg-amber-500/90 hover:bg-amber-500 text-black"
              >
                {committing ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <Compass size={14} className="mr-1.5" />}
                {committing ? "Offering…" : "Offer this as a quest"}
              </Button>
            </DialogFooter>
          </>
        )}

        {phase === "reflecting" && world && (
          <>
            <DialogHeader onClose={() => {}}>
              <DialogTitle>
                <Loader2 size={16} className="inline mr-2 text-amber-400 animate-spin" />
                Hearing it named back to you…
              </DialogTitle>
            </DialogHeader>
            <DialogBody className="space-y-4">
              <p className="text-sm text-muted-foreground italic leading-relaxed">
                What you wrote is being spoken back as a real pursuit.
              </p>
              <div className="rounded-md border border-border/40 bg-card/30 p-3">
                <p className="text-[11px] uppercase tracking-wider text-muted-foreground/70 mb-1">In your own words</p>
                <p className="text-sm text-foreground/80 italic">{reaching.trim()}</p>
              </div>
            </DialogBody>
          </>
        )}

        {phase === "offering" && world && (
          <>
            <DialogHeader onClose={onRevise}>
              <DialogTitle>
                <Compass size={16} className="inline mr-2 text-amber-400" />
                The quest, named
              </DialogTitle>
            </DialogHeader>
            <DialogBody className="space-y-4">
              <p className="text-xs text-muted-foreground italic leading-relaxed">
                Here is what you wrote, named as a real pursuit. Accept it, or revise.
              </p>
              <div className="rounded-xl border-2 border-amber-400/50 bg-gradient-to-br from-amber-500/10 via-amber-500/5 to-transparent p-5">
                <p
                  className="text-amber-100 leading-relaxed"
                  style={{ fontSize: "1.05rem", fontWeight: 500, letterSpacing: "0.005em" }}
                >
                  {nobleOffering}
                </p>
              </div>
              <div className="rounded-md border border-border/40 bg-card/30 p-3">
                <p className="text-[11px] uppercase tracking-wider text-muted-foreground/70 mb-1">From what you wrote</p>
                <p className="text-sm text-foreground/80 italic">{reaching.trim()}</p>
              </div>
              {error && <p className="text-xs text-destructive">{error}</p>}
            </DialogBody>
            <DialogFooter>
              <Button variant="ghost" onClick={onRevise} disabled={committing}>
                Go back and revise it
              </Button>
              <Button
                onClick={onAccept}
                disabled={committing}
                className="bg-amber-500/90 hover:bg-amber-500 text-black"
              >
                {committing ? <Loader2 size={14} className="animate-spin mr-1.5" /> : <Compass size={14} className="mr-1.5" />}
                {committing ? "Entering…" : "Accept this quest and enter"}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

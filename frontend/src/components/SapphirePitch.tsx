import { useEffect, useState } from "react";
import { api, type World, type WorldImageInfo } from "@/lib/tauri";

const README_URL = "https://github.com/mrrts/world-threads";
const LANDING_REPORT_URL = "https://github.com/mrrts/world-threads/blob/main/reports/2026-04-27-0030-public-release-landing.md";

type PitchState = {
  world: World | null;
  image: WorldImageInfo | null;
  loading: boolean;
  error: string | null;
};

export function SapphirePitch() {
  const params = new URLSearchParams(window.location.search);
  const requestedWorldId = params.get("world");
  const [state, setState] = useState<PitchState>({
    world: null,
    image: null,
    loading: true,
    error: null,
  });

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      try {
        const worlds = await api.listWorlds();
        if (!worlds.length) {
          if (!cancelled) {
            setState({ world: null, image: null, loading: false, error: "No worlds found." });
          }
          return;
        }

        const world =
          worlds.find((w) => w.world_id === requestedWorldId) ??
          worlds[0];

        let image = await api.getActiveWorldImage(world.world_id);
        if (!image) {
          const images = await api.listWorldImages(world.world_id);
          image = images.find((img) => img.is_active) ?? images[0] ?? null;
        }

        if (!cancelled) {
          setState({ world, image, loading: false, error: null });
        }
      } catch (error) {
        if (!cancelled) {
          setState({
            world: null,
            image: null,
            loading: false,
            error: error instanceof Error ? error.message : "Failed to load pitch surface.",
          });
        }
      }
    };

    void load();
    return () => {
      cancelled = true;
    };
  }, [requestedWorldId]);

  const worldName = state.world?.name ?? "WorldThreads";
  const worldDescription = state.world?.description?.trim() ?? "";
  const heroImage = state.image?.data_url ?? "";
  const toneTags = state.world?.tone_tags?.slice(0, 4) ?? [];
  const proofPoints = [
    "Bring your own OpenAI key.",
    "Keep your conversations on your own disk.",
    "Read the doctrine and prompt stack in the open.",
  ];

  return (
    <div className="min-h-screen bg-[#090c14] text-[#f6f1e8]">
      <style>{`
        .sapphire-shell {
          min-height: 100vh;
          background:
            radial-gradient(circle at 15% 20%, rgba(57, 109, 224, 0.32), transparent 30%),
            radial-gradient(circle at 85% 12%, rgba(84, 186, 255, 0.18), transparent 28%),
            radial-gradient(circle at 50% 100%, rgba(9, 71, 142, 0.35), transparent 45%),
            linear-gradient(180deg, #09101a 0%, #0e1726 42%, #0b1220 100%);
        }
        .sapphire-grid {
          background-image:
            linear-gradient(rgba(148, 196, 255, 0.06) 1px, transparent 1px),
            linear-gradient(90deg, rgba(148, 196, 255, 0.06) 1px, transparent 1px);
          background-size: 32px 32px;
        }
        .crown-glow {
          box-shadow:
            0 0 0 1px rgba(153, 204, 255, 0.14),
            0 24px 80px rgba(17, 91, 181, 0.35),
            inset 0 1px 0 rgba(255,255,255,0.1);
        }
        .image-frost::after {
          content: "";
          position: absolute;
          inset: 0;
          background:
            linear-gradient(180deg, rgba(3, 10, 20, 0.02) 0%, rgba(3, 10, 20, 0.3) 68%, rgba(3, 10, 20, 0.82) 100%);
          pointer-events: none;
        }
        .title-font {
          font-family: Georgia, "Times New Roman", serif;
          letter-spacing: -0.03em;
        }
        .body-font {
          font-family: "Iowan Old Style", "Palatino Linotype", "Book Antiqua", Palatino, serif;
        }
        .sapphire-button {
          background:
            linear-gradient(135deg, #72c1ff 0%, #0d5fe7 44%, #062e84 100%);
          color: #f8fbff;
          box-shadow:
            0 16px 42px rgba(9, 82, 191, 0.4),
            inset 0 1px 0 rgba(255,255,255,0.24);
        }
        .sapphire-button:hover {
          transform: translateY(-1px);
          box-shadow:
            0 20px 52px rgba(9, 82, 191, 0.48),
            inset 0 1px 0 rgba(255,255,255,0.26);
        }
        .sapphire-secondary {
          border: 1px solid rgba(142, 195, 255, 0.2);
          background: rgba(10, 22, 38, 0.7);
          color: #ddecfb;
          box-shadow: inset 0 1px 0 rgba(255,255,255,0.05);
        }
        .sapphire-secondary:hover {
          border-color: rgba(142, 195, 255, 0.34);
          background: rgba(12, 28, 48, 0.86);
        }
      `}</style>

      <div className="sapphire-shell sapphire-grid">
        <main className="mx-auto flex min-h-screen max-w-7xl flex-col px-5 py-6 sm:px-8 lg:px-10">
          <section className="crown-glow relative overflow-hidden rounded-[2rem] border border-[#81baff33] bg-[#07101a]/72">
            <div className="grid min-h-[88vh] grid-cols-1 lg:grid-cols-[1.06fr_0.94fr]">
              <div className="relative z-10 flex flex-col justify-between p-7 sm:p-10 lg:p-14">
                <div className="space-y-6">
                  <div className="inline-flex w-fit items-center gap-3 rounded-full border border-[#91c8ff3d] bg-[#0e2036b0] px-4 py-2 text-[11px] uppercase tracking-[0.3em] text-[#c6e5ff]">
                    <span className="inline-block h-2.5 w-2.5 rounded-full bg-[#74c6ff]" />
                    Crown-Grade Convergence
                  </div>

                  <div className="space-y-5">
                    <p className="body-font max-w-xl text-sm uppercase tracking-[0.28em] text-[#93c8ff]">
                      Characters with grain. Worlds with weight.
                    </p>
                    <h1 className="title-font max-w-4xl text-5xl leading-[0.95] text-[#f7f6f2] sm:text-6xl lg:text-7xl">
                      Enter the
                      {" "}
                      <span className="bg-gradient-to-r from-[#e7f5ff] via-[#86cfff] to-[#2f82ff] bg-clip-text text-transparent">
                        Crown of Sapphire
                      </span>
                      {", "}
                      where characters have grain and worlds have weight.
                    </h1>
                    <p className="body-font max-w-2xl text-lg leading-8 text-[#d7e7f5] sm:text-xl">
                      Meet AI characters with distinct voices, boundaries, and resistance, in AI worlds that feel inhabited rather than sketched.
                      Underneath, the system keeps voice, continuity, and boundaries intact,
                      so the writing can stay vivid without drifting into mush, therapy-voice, or fake intimacy.
                    </p>
                  </div>

                  <div className="grid gap-4 sm:grid-cols-3">
                    <div className="rounded-[1.35rem] border border-white/10 bg-white/5 p-4">
                      <div className="text-xs uppercase tracking-[0.22em] text-[#8ec3ff]">Inspectable Trust</div>
                      <p className="body-font mt-3 text-sm leading-6 text-[#e8eef6]">
                        BYOK-first, on-disk storage, and an openly readable doctrine layer so the trust story can be checked instead of merely admired.
                      </p>
                    </div>
                    <div className="rounded-[1.35rem] border border-white/10 bg-white/5 p-4">
                      <div className="text-xs uppercase tracking-[0.22em] text-[#8ec3ff]">Worlds With Weight</div>
                      <p className="body-font mt-3 text-sm leading-6 text-[#e8eef6]">
                        Places with weather, objects, friction, memory, and moral atmosphere, so the encounter feels located instead of floating.
                      </p>
                    </div>
                    <div className="rounded-[1.35rem] border border-white/10 bg-white/5 p-4">
                      <div className="text-xs uppercase tracking-[0.22em] text-[#8ec3ff]">Characters With Grain</div>
                      <p className="body-font mt-3 text-sm leading-6 text-[#e8eef6]">
                        Not chatbots in costume. Distinct presences with their own limits, humor, tenderness, and resistance to fake intimacy.
                      </p>
                    </div>
                  </div>

                  <div className="rounded-[1.5rem] border border-[#91c8ff24] bg-[#081524bf] p-5">
                    <div className="text-[11px] uppercase tracking-[0.24em] text-[#8ec3ff]">What You Can Verify Right Now</div>
                    <div className="mt-4 grid gap-3 sm:grid-cols-3">
                      {proofPoints.map((point) => (
                        <div
                          key={point}
                          className="rounded-[1rem] border border-white/10 bg-[#0c1b2db5] px-4 py-3 text-sm leading-6 text-[#e3edf7]"
                        >
                          {point}
                        </div>
                      ))}
                    </div>
                  </div>
                </div>

                <div className="mt-8 flex flex-col gap-5 sm:flex-row sm:items-end sm:justify-between">
                  <div className="max-w-xl space-y-3">
                    <a
                      href={README_URL}
                      target="_blank"
                      rel="noreferrer"
                      className="sapphire-button inline-flex rounded-full px-7 py-4 text-base font-semibold transition-all duration-200"
                    >
                      Read the WorldThreads README
                    </a>
                    <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap">
                      <a
                        href={LANDING_REPORT_URL}
                        target="_blank"
                        rel="noreferrer"
                        className="sapphire-secondary inline-flex rounded-full px-6 py-3 text-sm font-medium transition-all duration-200"
                      >
                        Take the 15-minute orientation
                      </a>
                      <a
                        href="/"
                        className="inline-flex rounded-full border border-transparent px-2 py-3 text-sm font-medium text-[#b9d8f6] transition-colors duration-200 hover:text-white"
                      >
                        Or step straight into the app
                      </a>
                    </div>
                    <p className="body-font max-w-lg text-sm leading-6 text-[#c1d4e8]">
                      If this page catches your eye, the README is the next honest surface. It names the worldview plainly and lets the work prove itself in the open.
                    </p>
                  </div>

                  <div className="rounded-[1.2rem] border border-[#9ed0ff24] bg-[#0d1828c8] px-5 py-4 text-right">
                    <div className="text-[11px] uppercase tracking-[0.24em] text-[#86bbf5]">Craft Aim</div>
                    <div className="title-font mt-1 text-2xl text-[#f5f8fd]">𝓕 → speech → world</div>
                    <div className="body-font mt-2 text-sm leading-6 text-[#c3d6ea]">
                      Compression that does not flatten.
                      Structure that does not kill wonder.
                    </div>
                  </div>
                </div>
              </div>

              <div className="relative min-h-[28rem] border-t border-white/10 lg:min-h-full lg:border-l lg:border-t-0">
                {heroImage ? (
                  <div className="image-frost absolute inset-0">
                    <img
                      src={heroImage}
                      alt={worldName}
                      className="h-full w-full object-cover"
                    />
                  </div>
                ) : (
                  <div className="absolute inset-0 bg-[radial-gradient(circle_at_35%_25%,rgba(110,189,255,0.2),transparent_28%),linear-gradient(160deg,#13233b_0%,#0c1626_100%)]" />
                )}

                <div className="relative z-10 flex h-full flex-col justify-end p-7 sm:p-10">
                  <div className="crown-glow max-w-2xl rounded-[1.6rem] border border-[#b7dcff30] bg-[#07111bd8] p-6 sm:p-7">
                    <div className="flex flex-wrap gap-2">
                      {toneTags.map((tag) => (
                        <span
                          key={tag}
                          className="rounded-full border border-[#8abfff38] bg-[#0d2036] px-3 py-1 text-[11px] uppercase tracking-[0.18em] text-[#b9dbfb]"
                        >
                          {tag}
                        </span>
                      ))}
                      <span className="rounded-full border border-[#8abfff38] bg-[#0d2036] px-3 py-1 text-[11px] uppercase tracking-[0.18em] text-[#b9dbfb]">
                        World image from SQLite
                      </span>
                    </div>

                    <h2 className="title-font mt-5 text-3xl text-[#fbf8f1] sm:text-4xl">
                      {state.loading ? "Loading a world…" : worldName}
                    </h2>
                    <p className="body-font mt-4 text-base leading-7 text-[#d3e2f1] sm:text-lg">
                      {state.error
                        ? state.error
                        : worldDescription || "A world with enough grain, weather, and mystery to make the next encounter feel chosen instead of generated."}
                    </p>

                    <div className="mt-6 grid gap-4 border-t border-white/10 pt-5 sm:grid-cols-2">
                      <div>
                        <div className="text-[11px] uppercase tracking-[0.24em] text-[#88c2ff]">Pitch</div>
                        <p className="body-font mt-2 text-sm leading-6 text-[#e6eef6]">
                          The worlds are not there to backdrop the characters.
                          They are there to answer with them.
                        </p>
                      </div>
                      <div>
                        <div className="text-[11px] uppercase tracking-[0.24em] text-[#88c2ff]">Promise</div>
                        <p className="body-font mt-2 text-sm leading-6 text-[#e6eef6]">
                          You are not chasing novelty for its own sake.
                          You are seeking wonder sturdy enough to survive daylight and still feel human in the hand.
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </section>
        </main>
      </div>
    </div>
  );
}

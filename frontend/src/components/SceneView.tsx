import { useRef, useMemo, useState, useEffect } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { Float, Stars, Billboard } from "@react-three/drei";
import * as THREE from "three";
import { MessageSquare, Settings2 } from "lucide-react";
import type { useAppStore } from "@/hooks/use-app-store";
import type { Character } from "@/lib/tauri";

interface Props {
  store: ReturnType<typeof useAppStore>;
  onChat?: (characterId: string) => void;
  onSettings?: (characterId: string) => void;
}

const TIME_COLORS: Record<string, { sky: string; ground: string; ambient: string; sun: string; fog: string; intensity: number }> = {
  DAWN:      { sky: "#7a6a8a", ground: "#5a5060", ambient: "#e8b090", sun: "#ffb070", fog: "#6a5a70", intensity: 0.6 },
  MORNING:   { sky: "#88b4d8", ground: "#7a9a6a", ambient: "#fff4e0", sun: "#ffe8b0", fog: "#8ab0d0", intensity: 0.85 },
  NOON:      { sky: "#90c0e8", ground: "#8aaa78", ambient: "#fffff0", sun: "#fff8e0", fog: "#90b8e0", intensity: 1.0 },
  AFTERNOON: { sky: "#80b0d8", ground: "#80a070", ambient: "#fff0d0", sun: "#ffe0a0", fog: "#80a8c8", intensity: 0.9 },
  DUSK:      { sky: "#6a5070", ground: "#5a4a50", ambient: "#e8a070", sun: "#ff9050", fog: "#6a4a60", intensity: 0.55 },
  NIGHT:     { sky: "#141828", ground: "#1a1a28", ambient: "#6080b0", sun: "#4060a0", fog: "#101020", intensity: 0.25 },
};

export function SceneView({ store, onChat, onSettings }: Props) {
  const timeOfDay = store.activeWorld?.state?.time?.time_of_day ?? "MORNING";
  const colors = TIME_COLORS[timeOfDay] ?? TIME_COLORS.MORNING;
  const [selectedChar, setSelectedChar] = useState<Character | null>(null);

  const charPositions = useMemo(() => {
    const count = store.characters.length;
    if (count === 0) return [];
    const spacing = Math.min(3.5, 10 / count);
    return store.characters.map((_, i) => {
      const x = i * spacing - (count - 1) * spacing * 0.5;
      return [x, 0.6, 0] as [number, number, number];
    });
  }, [store.characters.length]);

  return (
    <div className="flex-1 relative">
      <div className="absolute inset-0">
        <Canvas camera={{ position: [0, 3, 10], fov: 45 }}>
          <fog attach="fog" args={[colors.fog, 8, 35]} />
          <ambientLight intensity={colors.intensity * 0.6} color={colors.ambient} />
          <directionalLight position={[5, 10, 5]} intensity={colors.intensity * 0.8} color={colors.sun} />
          <directionalLight position={[-3, 6, -2]} intensity={colors.intensity * 0.2} color={colors.ambient} />
          <hemisphereLight args={[colors.sky, colors.ground, colors.intensity * 0.4]} />

          {timeOfDay === "NIGHT" || timeOfDay === "DUSK" || timeOfDay === "DAWN" ? (
            <Stars radius={60} depth={50} count={timeOfDay === "NIGHT" ? 1500 : 400} factor={2} fade speed={0.3} />
          ) : null}

          <Ground color={colors.ground} />

          {store.characters.map((ch, i) => {
            const portrait = store.activePortraits[ch.character_id];
            return (
              <CharacterAvatar
                key={ch.character_id}
                avatarUrl={portrait?.data_url}
                fallbackColor={ch.avatar_color}
                position={charPositions[i] ?? [0, 0.6, 0]}
                active={ch.character_id === store.activeCharacter?.character_id}
                onClick={() => setSelectedChar(ch)}
              />
            );
          })}

          <Particles count={30} timeOfDay={timeOfDay} />
        </Canvas>
      </div>

      {/* Bottom info bar */}
      <div className="absolute bottom-4 left-4 bg-card/80 backdrop-blur-sm border border-border rounded-lg px-4 py-2">
        <p className="text-xs text-muted-foreground">
          {store.activeWorld?.state?.location?.current_scene ?? "Unknown"} · Day {store.activeWorld?.state?.time?.day_index ?? 1} · {timeOfDay}
        </p>
        <div className="flex gap-3 mt-1">
          {store.characters.map((ch) => {
            const portrait = store.activePortraits[ch.character_id];
            return (
              <button
                key={ch.character_id}
                className="flex items-center gap-1.5 text-xs cursor-pointer hover:text-primary transition-colors"
                onClick={() => setSelectedChar(ch)}
              >
                {portrait?.data_url ? (
                  <img src={portrait.data_url} alt="" className="w-4 h-4 rounded-full object-cover" />
                ) : (
                  <span className="w-3 h-3 rounded-full" style={{ backgroundColor: ch.avatar_color }} />
                )}
                <span className={ch.character_id === store.activeCharacter?.character_id ? "text-primary font-medium" : "text-muted-foreground"}>
                  {ch.display_name}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Character card popup */}
      {selectedChar && (
        <div className="absolute inset-0 z-20 flex items-center justify-center" onClick={() => setSelectedChar(null)}>
          <div
            className="bg-card border border-border rounded-2xl shadow-2xl shadow-black/40 w-80 overflow-hidden animate-in zoom-in-95 fade-in duration-150"
            onClick={(e) => e.stopPropagation()}
          >
            {(() => {
              const portrait = store.activePortraits[selectedChar.character_id];
              return (
                <>
                  <div className="p-5 flex items-center gap-4">
                    {portrait?.data_url ? (
                      <img src={portrait.data_url} alt="" className="w-20 h-20 rounded-full object-cover ring-2 ring-border flex-shrink-0" />
                    ) : (
                      <div
                        className="w-20 h-20 rounded-full flex-shrink-0 ring-2 ring-white/10"
                        style={{ backgroundColor: selectedChar.avatar_color }}
                      />
                    )}
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold text-base truncate">{selectedChar.display_name}</h3>
                      {selectedChar.identity && (
                        <p className="text-xs text-muted-foreground mt-1 line-clamp-3 leading-relaxed">
                          {selectedChar.identity.slice(0, 140)}{selectedChar.identity.length > 140 ? "..." : ""}
                        </p>
                      )}
                    </div>
                  </div>
                  <div className="flex border-t border-border">
                    <button
                      onClick={() => { setSelectedChar(null); onChat?.(selectedChar.character_id); }}
                      className="flex-1 flex items-center justify-center gap-2 py-3 text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors cursor-pointer border-r border-border"
                    >
                      <MessageSquare size={15} />
                      Chat
                    </button>
                    <button
                      onClick={() => { setSelectedChar(null); onSettings?.(selectedChar.character_id); }}
                      className="flex-1 flex items-center justify-center gap-2 py-3 text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors cursor-pointer"
                    >
                      <Settings2 size={15} />
                      Settings
                    </button>
                  </div>
                </>
              );
            })()}
          </div>
        </div>
      )}
    </div>
  );
}

function CharacterAvatar({
  avatarUrl,
  fallbackColor,
  position,
  active,
  onClick,
}: {
  avatarUrl?: string;
  fallbackColor: string;
  position: [number, number, number];
  active: boolean;
  onClick: () => void;
}) {
  const groupRef = useRef<THREE.Group>(null);
  const texture = useAvatarTexture(avatarUrl);
  const scale = active ? 1.3 : 1.0;

  useFrame((_, delta) => {
    if (groupRef.current) {
      const targetScale = active ? 1.3 : 1.0;
      groupRef.current.scale.lerp(new THREE.Vector3(targetScale, targetScale, targetScale), delta * 4);
    }
  });

  return (
    <Float speed={1.2} rotationIntensity={0.05} floatIntensity={0.4}>
      <group ref={groupRef} position={position} onClick={(e) => { e.stopPropagation(); onClick(); }}>
        <Billboard follow lockX={false} lockY={false} lockZ={false}>
          {texture ? (
            <mesh>
              <circleGeometry args={[0.7, 48]} />
              <meshBasicMaterial
                map={texture}
                side={THREE.DoubleSide}
                toneMapped={false}
              />
            </mesh>
          ) : (
            <mesh>
              <circleGeometry args={[0.7, 48]} />
              <meshBasicMaterial
                color={fallbackColor}
                side={THREE.DoubleSide}
              />
            </mesh>
          )}
          {/* Ring */}
          <mesh>
            <ringGeometry args={[0.7, 0.78, 48]} />
            <meshStandardMaterial
              color={active ? "#ffffff" : "#aaaaaa"}
              emissive={active ? "#ffffff" : "#666666"}
              emissiveIntensity={active ? 0.3 : 0.05}
              roughness={0.2}
              metalness={0.5}
              transparent
              opacity={active ? 0.9 : 0.4}
              side={THREE.DoubleSide}
            />
          </mesh>
        </Billboard>
        <pointLight
          color={active ? "#ffffff" : fallbackColor}
          intensity={active ? 1.0 : 0.2}
          distance={3}
        />
      </group>
    </Float>
  );
}

function useAvatarTexture(dataUrl?: string): THREE.Texture | null {
  const [texture, setTexture] = useState<THREE.Texture | null>(null);
  const texRef = useRef<THREE.Texture | null>(null);

  useEffect(() => {
    if (texRef.current) { texRef.current.dispose(); texRef.current = null; }
    if (!dataUrl) { setTexture(null); return; }

    const loader = new THREE.TextureLoader();
    loader.load(dataUrl, (tex) => {
      tex.colorSpace = THREE.SRGBColorSpace;
      tex.needsUpdate = true;
      texRef.current = tex;
      setTexture(tex);
    });

    return () => {
      if (texRef.current) { texRef.current.dispose(); texRef.current = null; }
    };
  }, [dataUrl]);

  return texture;
}

function Ground({ color }: { color: string }) {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.5, 0]} receiveShadow>
      <planeGeometry args={[60, 60]} />
      <meshStandardMaterial color={color} roughness={0.95} metalness={0.0} />
    </mesh>
  );
}

function Particles({ count, timeOfDay }: { count: number; timeOfDay: string }) {
  const meshRef = useRef<THREE.InstancedMesh>(null);

  const particles = useMemo(() => {
    return Array.from({ length: count }, () => ({
      x: (Math.random() - 0.5) * 18,
      y: Math.random() * 5 + 0.5,
      z: (Math.random() - 0.5) * 18,
      speed: 0.08 + Math.random() * 0.2,
      offset: Math.random() * Math.PI * 2,
    }));
  }, [count]);

  const dummy = useMemo(() => new THREE.Object3D(), []);

  useFrame(({ clock }) => {
    if (!meshRef.current) return;
    const t = clock.getElapsedTime();

    particles.forEach((p, i) => {
      dummy.position.set(
        p.x + Math.sin(t * 0.08 + p.offset) * 0.4,
        p.y + Math.sin(t * p.speed + p.offset) * 0.6,
        p.z,
      );
      dummy.updateMatrix();
      meshRef.current!.setMatrixAt(i, dummy.matrix);
    });
    meshRef.current.instanceMatrix.needsUpdate = true;
  });

  const particleColor = timeOfDay === "NIGHT" ? "#6080ff" : timeOfDay === "DUSK" || timeOfDay === "DAWN" ? "#ffa060" : "#fff8d0";

  return (
    <instancedMesh ref={meshRef} args={[undefined, undefined, count]}>
      <sphereGeometry args={[0.025, 6, 6]} />
      <meshStandardMaterial color={particleColor} emissive={particleColor} emissiveIntensity={1.5} transparent opacity={0.5} />
    </instancedMesh>
  );
}

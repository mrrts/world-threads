// Chiptune Score-Phrase synth (Phase 2 of Real User Held arc).
// Reads phrase JSON conforming to Score-Phrase Protocol v1 (4 tracks of MIDI
// instruments) and renders via Web Audio.
//
// Available instruments (cheap to render, MIDI-pitched):
//   square     — 50% pulse (built-in 'square' oscillator)
//   pulse_25   — 25% pulse via PeriodicWave (nasal lead)
//   pulse_125  — 12.5% pulse via PeriodicWave (thin/hollow)
//   triangle   — built-in 'triangle' oscillator (warm bass)
//   sawtooth   — built-in 'sawtooth' oscillator (string-like pad)
//   sine       — built-in 'sine' oscillator (soft bell/lead)
//   noise      — white-noise buffer with playback-rate pitching (rhythm)

export type Instrument =
  | 'square'
  | 'pulse_25'
  | 'pulse_125'
  | 'triangle'
  | 'sawtooth'
  | 'sine'
  | 'noise';

export const INSTRUMENTS: readonly Instrument[] = [
  'square',
  'pulse_25',
  'pulse_125',
  'triangle',
  'sawtooth',
  'sine',
  'noise',
] as const;

// Tagged-union track event. `rest` is a first-class primitive — explicit silence
// the AI can deliberately compose, distinct from "no event scheduled." Synth-time
// behavior: notes are scheduled; rests are no-ops (they advance the AI's mental
// timeline without producing sound).
export interface NoteEvent {
  type: "note";
  tick: number;
  duration: number;
  midi: number;
  velocity?: number;
}

export interface RestEvent {
  type: "rest";
  tick: number;
  duration: number;
}

export type TrackEvent = NoteEvent | RestEvent;

// Legacy + permissive: untyped event with `midi: number | null` is read as
// rest when midi is null/undefined, note otherwise. Lets older phrase JSON
// (or relaxed LLM output) still play.
export interface LegacyEvent {
  tick: number;
  duration: number;
  midi?: number | null;
  velocity?: number;
  type?: "note" | "rest";
}

function normalizeEvent(e: LegacyEvent | TrackEvent): TrackEvent {
  if ((e as TrackEvent).type === "rest") {
    return { type: "rest", tick: e.tick, duration: e.duration };
  }
  if ((e as TrackEvent).type === "note") {
    const n = e as NoteEvent;
    return { type: "note", tick: n.tick, duration: n.duration, midi: n.midi, velocity: n.velocity };
  }
  // Untyped — discriminate by midi presence.
  const midi = (e as LegacyEvent).midi;
  if (midi == null) {
    return { type: "rest", tick: e.tick, duration: e.duration };
  }
  return { type: "note", tick: e.tick, duration: e.duration, midi, velocity: (e as LegacyEvent).velocity };
}

export interface Track {
  name: string;          // labels for the AI: "lead" / "harmony" / "bass" / "rhythm"
  instrument: Instrument;
  notes: (TrackEvent | LegacyEvent)[];
}

export interface ScorePhrase {
  protocol_version: string;
  phrase_id: string;
  previous_phrase_id: string | null;
  tempo_bpm: number;
  time_signature: [number, number];
  subdivision: number;
  bars: number;
  key: string;
  mood_descriptor: string;
  momentstamp_basis: string;
  tracks: Track[]; // expected length 4; spec allows any length 1–4
}

let audioCtx: AudioContext | null = null;
const pulseWaveCache = new Map<number, PeriodicWave>();
let noiseBuffer: AudioBuffer | null = null;

function getContext(): AudioContext {
  if (!audioCtx) audioCtx = new AudioContext();
  return audioCtx;
}

// Expose the audio clock so callers can schedule phrases back-to-back without
// gaps (chain mode). Returns 0 if no AudioContext has been created yet.
export function getAudioContextTime(): number {
  return audioCtx?.currentTime ?? 0;
}

function midiToFreq(midi: number): number {
  return 440 * Math.pow(2, (midi - 69) / 12);
}

// Pulse-wave Fourier coefficients for arbitrary duty d ∈ (0, 1):
//   real[n] = (2/πn) sin(2πnd)
//   imag[n] = (2/πn) (1 − cos(2πnd))
// Verified at d=0.5: real[n]=0; imag[n]=4/(πn) for odd n — classic 50% square wave.
function getPulsePeriodicWave(ctx: AudioContext, duty: number): PeriodicWave {
  const key = Math.round(duty * 1000) / 1000;
  const cached = pulseWaveCache.get(key);
  if (cached) return cached;
  const harmonics = 64;
  const real = new Float32Array(harmonics + 1);
  const imag = new Float32Array(harmonics + 1);
  for (let n = 1; n <= harmonics; n++) {
    real[n] = (2 / (Math.PI * n)) * Math.sin(2 * Math.PI * n * key);
    imag[n] = (2 / (Math.PI * n)) * (1 - Math.cos(2 * Math.PI * n * key));
  }
  const wave = ctx.createPeriodicWave(real, imag, { disableNormalization: false });
  pulseWaveCache.set(key, wave);
  return wave;
}

function getNoiseBuffer(ctx: AudioContext): AudioBuffer {
  if (noiseBuffer) return noiseBuffer;
  const len = ctx.sampleRate * 2;
  const buf = ctx.createBuffer(1, len, ctx.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < len; i++) data[i] = Math.random() * 2 - 1;
  noiseBuffer = buf;
  return buf;
}

// secondsPerTick = 1 / (beatsPerSecond × ticksPerBeat).
// ticksPerBeat = subdivision / time_signature[1].
// 4/4 with subdivision=16 → 4 ticks/beat (sixteenth-notes).
export function secondsPerTick(phrase: ScorePhrase): number {
  const beatsPerSecond = phrase.tempo_bpm / 60;
  const ticksPerBeat = phrase.subdivision / phrase.time_signature[1];
  return 1 / (beatsPerSecond * ticksPerBeat);
}

// Per-instrument gain trim — keeps the 4-track mix balanced. Triangle and
// sawtooth read low and need more headroom; noise needs less or it dominates.
function trimFor(inst: Instrument): number {
  switch (inst) {
    case 'triangle':  return 0.30;
    case 'sawtooth':  return 0.16;
    case 'sine':      return 0.28;
    case 'square':    return 0.16;
    case 'pulse_25':  return 0.18;
    case 'pulse_125': return 0.20;
    case 'noise':     return 0.12;
  }
}

function scheduleNote(
  ctx: AudioContext,
  inst: Instrument,
  note: NoteEvent,
  startTime: number,
  endTime: number,
  destination: AudioNode,
): void {
  const velocity = (note.velocity ?? 80) / 127;
  const peak = velocity * trimFor(inst);

  const attack = 0.005;
  const release = 0.025;
  const sustainStart = startTime + attack;
  const releaseStart = Math.max(sustainStart, endTime - release);

  const gain = ctx.createGain();
  gain.gain.setValueAtTime(0, startTime);
  gain.gain.linearRampToValueAtTime(peak, sustainStart);
  gain.gain.setValueAtTime(peak, releaseStart);
  gain.gain.exponentialRampToValueAtTime(0.0001, endTime);
  gain.connect(destination);

  if (inst === 'noise') {
    const src = ctx.createBufferSource();
    src.buffer = getNoiseBuffer(ctx);
    src.loop = true;
    // midi=60 → 1.0×; ±12 → ±octave brightness shift.
    src.playbackRate.value = Math.pow(2, (note.midi - 60) / 12);
    src.connect(gain);
    src.start(startTime);
    src.stop(endTime + 0.05);
    return;
  }

  const osc = ctx.createOscillator();
  switch (inst) {
    case 'square':
    case 'triangle':
    case 'sawtooth':
    case 'sine':
      osc.type = inst;
      break;
    case 'pulse_25':
      osc.setPeriodicWave(getPulsePeriodicWave(ctx, 0.25));
      break;
    case 'pulse_125':
      osc.setPeriodicWave(getPulsePeriodicWave(ctx, 0.125));
      break;
  }
  osc.frequency.value = midiToFreq(note.midi);
  osc.connect(gain);
  osc.start(startTime);
  osc.stop(endTime + 0.05);
}

export interface PlayHandle {
  endsAt: number;
  stop: () => void;
}

export function playPhrase(
  phrase: ScorePhrase,
  opts?: { masterVolume?: number; startAt?: number },
): PlayHandle {
  const ctx = getContext();
  if (ctx.state === 'suspended') void ctx.resume();
  const sptick = secondsPerTick(phrase);
  const masterVolume = opts?.masterVolume ?? 0.5;
  const start = opts?.startAt ?? ctx.currentTime + 0.05;

  const master = ctx.createGain();
  master.gain.value = masterVolume;
  master.connect(ctx.destination);

  let phraseEnd = start;
  for (const track of phrase.tracks ?? []) {
    if (!INSTRUMENTS.includes(track.instrument)) continue;
    for (const raw of track.notes ?? []) {
      const ev = normalizeEvent(raw);
      const evStart = start + ev.tick * sptick;
      const evEnd = evStart + ev.duration * sptick;
      if (ev.type === "note") {
        scheduleNote(ctx, track.instrument, ev, evStart, evEnd, master);
      }
      // rests are no-ops — they're a notational primitive, not sound.
      if (evEnd > phraseEnd) phraseEnd = evEnd;
    }
  }

  return {
    endsAt: phraseEnd,
    stop: () => {
      try {
        master.gain.cancelScheduledValues(ctx.currentTime);
        master.gain.setTargetAtTime(0, ctx.currentTime, 0.01);
        setTimeout(() => master.disconnect(), 200);
      } catch {
        // ctx torn down — ignore
      }
    },
  };
}

// Hand-authored demo phrase. 4 tracks, mixed instrument palette — proves the
// synth honors the protocol's track-and-instrument shape across more than just
// the NES-original four. Mood: "open hesitation."
export const DEMO_PHRASE: ScorePhrase = {
  protocol_version: '1.0',
  phrase_id: 'demo-phrase-001',
  previous_phrase_id: null,
  tempo_bpm: 96,
  time_signature: [4, 4],
  subdivision: 16,
  bars: 2,
  key: 'C major',
  mood_descriptor: 'open hesitation',
  momentstamp_basis: 'demo (hand-authored, not from real momentstamp)',
  tracks: [
    {
      name: 'lead',
      instrument: 'pulse_25',
      notes: [
        { type: 'note', tick: 0,  duration: 6, midi: 64, velocity: 65 },
        { type: 'rest', tick: 6,  duration: 2 },
        { type: 'note', tick: 8,  duration: 4, midi: 67, velocity: 75 },
        { type: 'note', tick: 12, duration: 4, midi: 65, velocity: 60 },
        { type: 'note', tick: 16, duration: 6, midi: 64, velocity: 70 },
        { type: 'rest', tick: 22, duration: 2 },
        { type: 'note', tick: 24, duration: 8, midi: 60, velocity: 70 },
      ],
    },
    {
      name: 'harmony',
      instrument: 'sine',
      notes: [
        { type: 'note', tick: 0,  duration: 16, midi: 55, velocity: 50 },
        { type: 'note', tick: 16, duration: 16, midi: 57, velocity: 50 },
      ],
    },
    {
      name: 'bass',
      instrument: 'triangle',
      notes: [
        { type: 'note', tick: 0,  duration: 16, midi: 36, velocity: 100 },
        { type: 'note', tick: 16, duration: 16, midi: 41, velocity: 100 },
      ],
    },
    {
      name: 'rhythm',
      instrument: 'noise',
      notes: [
        { type: 'note', tick: 4,  duration: 1, midi: 72, velocity: 35 },
        { type: 'note', tick: 12, duration: 1, midi: 72, velocity: 35 },
        { type: 'note', tick: 20, duration: 1, midi: 72, velocity: 35 },
        { type: 'note', tick: 28, duration: 1, midi: 72, velocity: 35 },
      ],
    },
  ],
};

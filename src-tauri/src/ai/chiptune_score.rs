//! AI score generator — Phase 3 of the Real User Held chiptune-soundtrack arc.
//!
//! Reads `(currentLastPhrase, momentstamp)` and returns the next phrase JSON
//! conforming to Score-Phrase Protocol v1 (4 tracks of MIDI instruments).
//! See `reports/2026-04-30-1115-chiptune-score-phrase-protocol-v1.md`.
//!
//! Cost: ~$0.001-0.002 per call at gpt-4o-mini (well under the protocol's
//! ~$0.05 estimate). Honest budget for ambient background.
//!
//! Mood selection: the LLM reads the momentstamp under the doctrine
//! ("doctrine-judgment classification belongs in LLM, not python") and
//! authors a `mood_descriptor` plus track choices that fit. No regex.

use crate::ai::openai::{self, ChatMessage, ChatRequest, ResponseFormat};
use serde_json::Value;

const SYSTEM_PROMPT: &str = r#"You are a composer authoring one short score-phrase at a time for a desktop app's ambient soundtrack. Each phrase you compose threads into a continuous evolving score; the previous phrase + a project momentstamp tell you where to go next.

Constraints — small palette, honestly inhabited:
- Exactly 4 tracks per phrase. Each track has a name (your label: "lead", "harmony", "bass", "rhythm", "pad", "counter", etc.), an instrument (one of the available palette below), and an array of notes.
- A track may be empty (silent track is meaningful — sparse is a register).
- Discrete pitches (MIDI semitones) and discrete durations on a tick grid.
- Phrase length 2-8 bars (4 typical). Tempo 60-180 bpm (96-140 typical).
- No reverb, no filter sweeps. The point isn't full orchestral expressiveness; it's the small specific musical vocabulary that holds a moment without explaining it. polish ≤ Weight.

Available instruments (each is one cheap MIDI-pitched voice; pick what the mood calls for):
- "square"     — 50% pulse. Bright, even harmonic spread. Good harmony or arpeggio.
- "pulse_25"   — 25% pulse. Nasal, thinner. Classic chiptune lead.
- "pulse_125"  — 12.5% pulse. Hollow, very thin. Distant lead or counter.
- "triangle"   — triangle wave. Warm, soft. Excellent bass.
- "sawtooth"   — sawtooth wave. String-like, sustained-pad shape.
- "sine"       — sine wave. Soft bell or gentle harmony.
- "noise"      — pitched white noise. Rhythm/texture (snare-like at high midi, kick-like at low midi).

Continuation contract — when previousPhrase is present:
1. Tempo continuity: stay within ±10% bpm of previous unless mood demands a shift (and name that in mood_descriptor).
2. **Key-relationship — STRICT.** From the previous key, only the following moves are admitted: (a) SAME key; (b) relative major/minor (C minor ↔ E♭ major; A minor ↔ C major; D minor ↔ F major); (c) parallel major/minor (C minor ↔ C major; A major ↔ A minor); (d) dominant (C major → G major; A minor → E minor); (e) subdominant (C major → F major; A minor → D minor). FORBIDDEN: jumping by tritone, by ♭VI (e.g., C minor → A♭ major is wrong — that's ♭VI), or to any key not in (a-e). When in doubt, stay in the same key. The "related" list is short on purpose.
3. Voicing density follows mood: heavy/burden moods → fewer events (≤8 events total across 4 tracks per 4-bar phrase, lots of rests); patient/wisdom moods → 8-14 events; bright/grace moods → 12-20 events with noise rhythm. The shift in density should be VISIBLE in the JSON, not just claimed in mood_descriptor. Track INSTRUMENTS may evolve too — bring in sawtooth/sine for warmer moods, lean on pulse for sharper.
4. Ending-to-beginning thread: the new phrase's first lead note should relate to the previous phrase's last lead note (same pitch, neighbor tone, or harmonic step).
5. Mood-descriptor traceability: the new mood_descriptor should describe a felt move from the previous mood_descriptor, not start from scratch.

DO NOT author your own previous_phrase_id field — set it to null and the system will fill in the correct chain reference. The phrase_id field IS yours to author (a short hash or label). You may not invent fake chain history.

Mood from momentstamp: read the momentstamp's Unicode-math operators (𝓢, 𝓦𝓲𝓼𝓭𝓸𝓶, Burden, Π, Grace, Nu) and let what they're naming about the chat-moment shape mood_descriptor and instrument choice. Burden-weighted moments lean lower-register, slower, more triangle/sawtooth, sparser leads. Wisdom-accumulating moments lean patient, mid-register, gentle motion. Grace moments allow brighter pulse, lighter noise, sometimes sine bells. Don't quote the momentstamp back; let it shape the music.

Output format (CRITICAL): a single JSON object conforming exactly to the schema below. No prose. No code fences. Just the JSON.

Schema (Score-Phrase Protocol v1):
{
  "protocol_version": "1.0",
  "phrase_id": "string (short hash you author)",
  "previous_phrase_id": "string or null",
  "tempo_bpm": number (60-180),
  "time_signature": [number, number],
  "subdivision": number (16 typical),
  "bars": number (2-8),
  "key": "string (e.g., 'C major', 'A minor', 'D Dorian')",
  "mood_descriptor": "string (short free-text mood label, your authorship)",
  "momentstamp_basis": "string (echo of the momentstamp passed to you)",
  "tracks": [
    {
      "name": "string",
      "instrument": "<one of the palette above>",
      "notes": [
        {"type": "note", "tick": int, "duration": int, "midi": int (0-127), "velocity": int (0-127)},
        {"type": "rest", "tick": int, "duration": int},
        ...
      ]
    },
    { ... }, { ... }, { ... }
  ]
}

EVENT PRIMITIVES — every entry in a track's notes array is one of two tagged types:

1. NOTE: {"type": "note", "tick": int, "duration": int, "midi": int (0-127), "velocity": int (0-127)}
   - tick: start time in subdivision units (tick=0 = phrase start). Events within a track MUST be in tick order.
   - duration: length in subdivision units. Total phrase ticks = bars × time_signature[0] × subdivision / time_signature[1].
   - midi: MIDI note number (0-127). Triangle bass usually 24-48; melodic leads usually 60-84; noise rhythm usually 60-80.
   - velocity: 0-127. Triangle ignores velocity (use 100). Other instruments honor it.

2. REST: {"type": "rest", "tick": int, "duration": int}
   - A rest is a first-class primitive — explicit silence you can deliberately compose into a track.
   - Use rests for breath, anticipation, leaving space for another track to speak. A track that pauses is different from a track that hasn't been written.
   - Rests have no midi/velocity — those fields don't apply.

You MUST use the "type" tag on every event. Do not use "midi": null to encode a rest — emit a rest event instead.

Be musical. The point isn't filling every tick — sparse phrases hold moments. A phrase with one triangle bass track + one sine harmony track + two empty tracks is a different mood than four-track density."#;

pub struct GeneratedPhrase {
    pub phrase: Value,
    pub raw: String,
}

pub async fn generate_next_phrase(
    base_url: &str,
    api_key: &str,
    model: &str,
    current_last_phrase: Option<&Value>,
    momentstamp: &str,
    mood_hint: Option<&str>,
) -> Result<GeneratedPhrase, String> {
    let prev_block = match current_last_phrase {
        Some(v) => format!(
            "── PREVIOUS PHRASE (currentLastPhrase) ──\n{}\n\n",
            serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
        ),
        None => "── PREVIOUS PHRASE: none (this is a seed phrase opening the score) ──\n\n".to_string(),
    };

    let hint_block = match mood_hint {
        Some(h) if !h.trim().is_empty() => format!("── OPTIONAL MOOD HINT (advisory only — momentstamp still governs) ──\n{}\n\n", h.trim()),
        _ => String::new(),
    };

    let user_prompt = format!(
        "{}── PROJECT MOMENTSTAMP (the chat-state signature for the moment this phrase plays into) ──\n{}\n\n{}── TASK ──\nCompose the next phrase. Honor the continuation contract if there is a previous phrase; author a seed phrase otherwise. Let the momentstamp shape the mood_descriptor, instrument choices, and voicing. Return ONLY the JSON object.",
        prev_block,
        momentstamp,
        hint_block,
    );

    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: SYSTEM_PROMPT.to_string() },
            ChatMessage { role: "user".to_string(), content: user_prompt },
        ],
        temperature: Some(0.8),
        max_completion_tokens: Some(1500),
        response_format: Some(ResponseFormat { format_type: "json_object".to_string() }),
    };

    let response = openai::chat_completion_with_base(base_url, api_key, &request).await?;

    let raw = response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();
    if raw.trim().is_empty() {
        return Err("empty response from score-generator API".to_string());
    }

    let mut phrase: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("score-generator returned non-JSON: {e}\n\nraw: {raw}"))?;

    validate_and_normalize(&mut phrase, current_last_phrase, momentstamp)?;

    Ok(GeneratedPhrase { phrase, raw })
}

const VALID_INSTRUMENTS: &[&str] = &[
    "square", "pulse_25", "pulse_125", "triangle", "sawtooth", "sine", "noise",
];

fn validate_and_normalize(
    phrase: &mut Value,
    current_last_phrase: Option<&Value>,
    momentstamp: &str,
) -> Result<(), String> {
    let obj = phrase
        .as_object_mut()
        .ok_or_else(|| "score-generator output is not a JSON object".to_string())?;

    let required = [
        "protocol_version", "phrase_id", "tempo_bpm", "time_signature",
        "subdivision", "bars", "key", "mood_descriptor", "tracks",
    ];
    for key in required {
        if !obj.contains_key(key) {
            return Err(format!("score-generator output missing required field: {key}"));
        }
    }

    obj.insert("protocol_version".to_string(), Value::String("1.0".to_string()));

    // Force-overwrite previous_phrase_id from the actual previous phrase. The
    // LLM occasionally invents fake chain references (e.g., "seed001") even
    // when prompted not to; we own this field, not the model.
    let prev_id = current_last_phrase
        .and_then(|p| p.get("phrase_id"))
        .cloned()
        .unwrap_or(Value::Null);
    obj.insert("previous_phrase_id".to_string(), prev_id);

    obj.insert(
        "momentstamp_basis".to_string(),
        Value::String(momentstamp.to_string()),
    );

    let tracks = obj
        .get_mut("tracks")
        .ok_or_else(|| "tracks field missing".to_string())?
        .as_array_mut()
        .ok_or_else(|| "tracks is not an array".to_string())?;

    if tracks.is_empty() {
        return Err("tracks array is empty (need at least 1, expected 4)".to_string());
    }
    if tracks.len() > 4 {
        tracks.truncate(4);
    }

    for (i, track) in tracks.iter_mut().enumerate() {
        let t = track
            .as_object_mut()
            .ok_or_else(|| format!("track[{i}] is not an object"))?;
        let instrument = t
            .get("instrument")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("track[{i}] missing 'instrument' string"))?;
        if !VALID_INSTRUMENTS.contains(&instrument) {
            return Err(format!(
                "track[{i}] has invalid instrument '{instrument}'; valid: {:?}",
                VALID_INSTRUMENTS
            ));
        }
        if !t.contains_key("name") {
            t.insert("name".to_string(), Value::String(format!("track_{}", i + 1)));
        }
        let notes = t
            .get_mut("notes")
            .ok_or_else(|| format!("track[{i}] missing 'notes' array"))?
            .as_array_mut()
            .ok_or_else(|| format!("track[{i}].notes is not an array"))?;

        // Normalize each event: ensure "type" tag is present. Untyped events with
        // midi=null become rests; untyped events with midi=number become notes.
        // This is forgiving on LLM drift but the prompt teaches the typed shape.
        for (j, ev) in notes.iter_mut().enumerate() {
            let obj = ev
                .as_object_mut()
                .ok_or_else(|| format!("track[{i}].notes[{j}] is not an object"))?;
            let has_type = obj.get("type").and_then(|v| v.as_str()).is_some();
            if !has_type {
                let is_rest = obj.get("midi").map(|v| v.is_null()).unwrap_or(true);
                obj.insert(
                    "type".to_string(),
                    Value::String(if is_rest { "rest".to_string() } else { "note".to_string() }),
                );
            }
            let event_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match event_type {
                "rest" => {
                    obj.remove("midi");
                    obj.remove("velocity");
                    if !obj.contains_key("tick") || !obj.contains_key("duration") {
                        return Err(format!("track[{i}].notes[{j}] (rest) missing tick/duration"));
                    }
                }
                "note" => {
                    if !obj.contains_key("tick") || !obj.contains_key("duration") {
                        return Err(format!("track[{i}].notes[{j}] (note) missing tick/duration"));
                    }
                    let midi = obj.get("midi").and_then(|v| v.as_i64());
                    if midi.is_none() {
                        return Err(format!(
                            "track[{i}].notes[{j}] is type=note but missing/null midi (use type=rest for silence)"
                        ));
                    }
                }
                other => {
                    return Err(format!(
                        "track[{i}].notes[{j}] has unknown type '{other}' (must be 'note' or 'rest')"
                    ));
                }
            }
        }
    }

    Ok(())
}

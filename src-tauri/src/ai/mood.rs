use crate::db::queries::{Character, CharacterMood, Message, World};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const MAX_HISTORY: usize = 20;
const DEFAULT_DRIFT_RATE: f64 = 0.15;
const MAX_DELTA_PER_STEP: f64 = 0.25;
const NOISE_AMPLITUDE: f64 = 0.03;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodVector {
    pub valence: f64,
    pub energy: f64,
    pub tension: f64,
}

impl MoodVector {
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            energy: 0.0,
            tension: 0.0,
        }
    }

    fn clamp(&mut self) {
        self.valence = self.valence.clamp(-1.0, 1.0);
        self.energy = self.energy.clamp(-1.0, 1.0);
        self.tension = self.tension.clamp(0.0, 1.0);
    }
}

impl From<&CharacterMood> for MoodVector {
    fn from(m: &CharacterMood) -> Self {
        Self {
            valence: m.valence,
            energy: m.energy,
            tension: m.tension,
        }
    }
}

/// Deterministic target mood based on world state, character profile, and recent messages.
pub fn compute_mood_target(
    world: &World,
    character: &Character,
    recent_messages: &[Message],
) -> MoodVector {
    let mut target = MoodVector::neutral();

    // --- World time of day influence ---
    if let Some(time_of_day) = world
        .state
        .get("time")
        .and_then(|t| t.get("time_of_day"))
        .and_then(|v| v.as_str())
    {
        match time_of_day.to_uppercase().as_str() {
            "NIGHT" | "LATE_NIGHT" => {
                target.energy -= 0.3;
                target.tension += 0.05;
            }
            "MORNING" | "DAWN" => {
                target.energy += 0.15;
                target.valence += 0.1;
            }
            "AFTERNOON" => {
                target.energy += 0.05;
            }
            "EVENING" | "DUSK" => {
                target.energy -= 0.15;
                target.valence += 0.05;
            }
            _ => {}
        }
    }

    // --- World tone tags influence ---
    if let Some(tags) = world.tone_tags.as_array() {
        for tag in tags.iter().filter_map(|v| v.as_str()) {
            let t = tag.to_lowercase();
            if t.contains("peaceful") || t.contains("calm") || t.contains("serene") {
                target.tension -= 0.15;
                target.valence += 0.1;
            } else if t.contains("tense") || t.contains("dark") || t.contains("grim") {
                target.tension += 0.2;
                target.valence -= 0.1;
            } else if t.contains("whimsical") || t.contains("playful") || t.contains("lighthearted")
            {
                target.valence += 0.15;
                target.energy += 0.1;
            } else if t.contains("melanchol") || t.contains("sad") || t.contains("somber") {
                target.valence -= 0.2;
                target.energy -= 0.15;
            }
        }
    }

    // --- Character mood/trust from state ---
    if let Some(mood_val) = character.state.get("mood").and_then(|v| v.as_f64()) {
        let normalized = (mood_val / 5.0).clamp(-1.0, 1.0);
        target.valence += normalized * 0.2;
    }
    if let Some(trust) = character.state.get("trust_user").and_then(|v| v.as_f64()) {
        let normalized = (trust / 5.0).clamp(-1.0, 1.0);
        target.tension -= normalized * 0.1;
        target.valence += normalized * 0.1;
    }

    // --- Recent conversation sentiment analysis (keyword heuristic) ---
    let window = recent_messages.iter().rev().take(6);
    let mut pos_count = 0i32;
    let mut neg_count = 0i32;
    let mut heavy_count = 0i32;

    for msg in window {
        let lower = msg.content.to_lowercase();
        let positive = [
            "haha",
            "lol",
            "love",
            "happy",
            "great",
            "amazing",
            "beautiful",
            "thank",
            "glad",
            "nice",
            "funny",
            "laugh",
            "joy",
            "excited",
            "wonderful",
        ];
        let negative = [
            "sad", "angry", "upset", "hurt", "hate", "sorry", "terrible", "awful", "worried",
            "scared", "afraid", "lonely", "miss you", "crying",
        ];
        let heavy = [
            "death",
            "dying",
            "grief",
            "trauma",
            "abuse",
            "war",
            "disease",
            "hospital",
            "funeral",
            "cancer",
            "suicide",
            "crisis",
            "emergency",
        ];

        pos_count += positive.iter().filter(|w| lower.contains(*w)).count() as i32;
        neg_count += negative.iter().filter(|w| lower.contains(*w)).count() as i32;
        heavy_count += heavy.iter().filter(|w| lower.contains(*w)).count() as i32;
    }

    let sentiment = ((pos_count - neg_count) as f64 * 0.08).clamp(-0.3, 0.3);
    target.valence += sentiment;
    target.energy += sentiment * 0.5;

    if heavy_count > 0 {
        let weight = (heavy_count as f64 * 0.1).min(0.3);
        target.tension += weight;
        target.energy -= weight * 0.5;
    }

    target.clamp();
    target
}

/// Smooth drift: exponential lerp from current toward target, with tiny noise and delta cap.
pub fn drift_mood(
    current: &MoodVector,
    target: &MoodVector,
    drift_rate: Option<f64>,
) -> MoodVector {
    let rate = drift_rate.unwrap_or(DEFAULT_DRIFT_RATE);
    let seed = (current.valence * 1000.0 + current.energy * 777.0 + current.tension * 333.0).abs();

    let noise = |i: u32| -> f64 {
        let x = ((seed + i as f64) * 12345.6789).sin() * 43758.5453;
        (x - x.floor() - 0.5) * 2.0 * NOISE_AMPLITUDE
    };

    let lerp_clamped = |cur: f64, tgt: f64, n: f64| -> f64 {
        let raw = cur + (tgt - cur) * rate + n;
        let delta = (raw - cur).clamp(-MAX_DELTA_PER_STEP, MAX_DELTA_PER_STEP);
        cur + delta
    };

    let mut result = MoodVector {
        valence: lerp_clamped(current.valence, target.valence, noise(0)),
        energy: lerp_clamped(current.energy, target.energy, noise(1)),
        tension: lerp_clamped(current.tension, target.tension, noise(2)),
    };
    result.clamp();
    result
}

/// Convert mood vector into a short style directive string for the LLM prompt.
pub fn mood_to_style_directive(mood: &MoodVector) -> String {
    let mut qualities = Vec::new();

    match (mood.valence > 0.3, mood.valence < -0.3) {
        (true, _) => qualities.push("warm"),
        (_, true) => qualities.push("subdued"),
        _ => {}
    }

    match (mood.energy > 0.3, mood.energy < -0.3) {
        (true, _) => qualities.push("bright and quick"),
        (_, true) => qualities.push("quiet and slow"),
        _ => {}
    }

    if mood.tension > 0.5 {
        qualities.push("careful and clipped");
    } else if mood.tension > 0.25 {
        qualities.push("slightly guarded");
    } else if mood.tension < 0.1 && mood.valence > 0.0 {
        qualities.push("at ease");
    }

    // Compound moods
    if mood.energy < -0.2 && mood.valence > 0.2 {
        qualities.retain(|q| *q != "warm");
        qualities.push("gentle, quiet warmth");
    }
    if mood.energy > 0.2 && mood.valence > 0.2 && mood.tension < 0.2 {
        qualities.retain(|q| *q != "warm" && *q != "bright and quick");
        qualities.push("bright and playful");
    }
    if mood.energy < -0.2 && mood.valence < -0.2 {
        qualities.retain(|q| *q != "subdued" && *q != "quiet and slow");
        qualities.push("heavy, withdrawn");
    }
    if mood.tension > 0.4 && mood.energy > 0.2 {
        qualities.retain(|q| *q != "careful and clipped" && *q != "bright and quick");
        qualities.push("tense and sharp");
    }

    if qualities.is_empty() {
        return String::new();
    }

    format!("Current conversational tone: {}. Let this color your word choice and pacing subtly — don't announce it.", qualities.join(", "))
}

/// Append the current mood to history (max 20 entries), return updated history JSON.
pub fn append_mood_history(history: &Value, mood: &MoodVector) -> Value {
    let mut arr = history.as_array().cloned().unwrap_or_default();
    arr.push(json!({
        "v": (mood.valence * 100.0).round() / 100.0,
        "e": (mood.energy * 100.0).round() / 100.0,
        "t": (mood.tension * 100.0).round() / 100.0,
    }));
    while arr.len() > MAX_HISTORY {
        arr.remove(0);
    }
    Value::Array(arr)
}

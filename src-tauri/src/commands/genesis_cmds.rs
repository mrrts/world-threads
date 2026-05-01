//! Genesis — one-click generation of a full starter world and two
//! characters to inhabit it. Used for the first-run experience and as
//! an alternative "auto-generate" path on the new-world form.
//!
//! Pipeline:
//!   1. Sample a random seed flavor + mood + dramatic undercurrent.
//!   2. Single LLM call with JSON-mode → structured world + 2 characters.
//!   3. Persist world; persist each character (overwriting placeholders).
//!   4. Generate world image.
//!   5. Generate each character's portrait.
//!   6. Extract each character's visual description from their portrait.
//!   7. Seed each character's inventory.
//!
//! Each stage emits a `genesis-stage` event so the frontend can show
//! an evocative progress indicator ("dreaming a world…" etc.).

use crate::ai::openai::{self, ChatMessage, ChatRequest, ResponseFormat};
use crate::ai::orchestrator;
use crate::commands::portrait_cmds::PortraitsDir;
use crate::db::queries::*;
use crate::db::Database;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};

// ── Stage event payload ────────────────────────────────────────────────────

/// A payload revealed to the frontend during a stage — the user meets
/// their world piece by piece as it lands, rather than waiting for a
/// generic load bar. Tagged union matches the frontend type.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum GenesisReveal {
    #[serde(rename = "world_named")]
    WorldNamed { name: String, description: String },
    #[serde(rename = "character_named")]
    CharacterNamed { character_id: String, name: String, identity: String, avatar_color: String },
    #[serde(rename = "world_image_ready")]
    WorldImageReady { world_id: String },
    #[serde(rename = "portrait_ready")]
    PortraitReady { character_id: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct GenesisStageEvent {
    pub stage: String,
    pub detail: String,
    /// 0.0 → 1.0 progress for a linear indicator if desired.
    pub progress: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reveal: Option<GenesisReveal>,
}

fn emit_stage(app: &AppHandle, stage: &str, detail: &str, progress: f32) {
    let _ = app.emit(
        "genesis-stage",
        GenesisStageEvent {
            stage: stage.to_string(),
            detail: detail.to_string(),
            progress,
            reveal: None,
        },
    );
}

fn emit_stage_with_reveal(app: &AppHandle, stage: &str, detail: &str, progress: f32, reveal: GenesisReveal) {
    let _ = app.emit(
        "genesis-stage",
        GenesisStageEvent {
            stage: stage.to_string(),
            detail: detail.to_string(),
            progress,
            reveal: Some(reveal),
        },
    );
}

// ── Seed palette — sampled randomly so each genesis is different ──────────

const SETTING_SEEDS: &[&str] = &[
    "a small island monastery on a northern sea",
    "a river town at the edge of a wide marsh",
    "a high-desert settlement around a deep stone well",
    "a timber-framed town on a slow-turning plateau",
    "a chalk-cliff fishing village after a long winter",
    "a valley of terraced olive groves and shepherds' stones",
    "a frontier mission at a watershed in a mountain pass",
    "a cedar-forested hollow where two old roads cross",
    "a bog-country farmstead cluster under a low grey sky",
    "a harbour town of salt-white houses and painted boats",
    "a steppe outpost where the wind never fully stops",
    "a mill-village on a cold river still spooled with old waterwheels",
    "a basalt-stone hill country with small chapels cut into the rock",
    "a low-lying archipelago of fishermen and boatwrights",
    "a border town in a country of red hills and evening dust",
    "a charcoal-burners' ridge in an old forested country",
    "a canal town built on stilts above a wide shallow lake",
    "a scattered settlement on a tundra shoreline in long summer light",
    "a vineyards-and-stone hillside town above an inland sea",
    "a forge-town in a deep ravine with the sound of running water always audible",
];

const MOOD_SEEDS: &[&str] = &[
    "quiet, weathered, full of small kindnesses between familiar people",
    "dramatic, a community in the middle of a slow-unfolding reckoning",
    "luminous, the ordinary made strange by a sense of watched-over-ness",
    "wry, workaday, with humor worn smooth by long acquaintance",
    "tender, a place where grief and hope sit next to each other at the same tables",
    "restless, something arriving or leaving, the air charged",
    "lived-in, a world that has been itself for a long time and knows it",
    "a world where ordinary labor is held sacred without ever being announced",
    "gently strange — the natural world behaves slightly differently here, in ways nobody talks about often",
    "warm, argumentative, musical — a place that sings through its disagreements",
    "contemplative, cold, honest — a country where people do not say what they do not mean",
    "a world recently through hardship, now finding its feet again, the glow before full recovery",
    "sharp-edged in the good way — clear weather, clear water, people who speak plainly",
    "a place where children have enough freedom to become themselves, and the adults notice",
    "a community at the lip of change; the old shape still holding, the new one visible",
];

const HOOK_SEEDS: &[&str] = &[
    "something that used to be reliable is no longer — a signal missed, a route gone strange, a thing that rang and now doesn't",
    "an outsider has arrived who has not yet announced themselves",
    "a long-kept silence between two people is nearing the moment when it has to be named",
    "a gift has been left anonymously, and no one can agree what it means",
    "a weather or a sky has changed in a way the old people remember and the young people don't",
    "a child has been asking the adults a question they can't quite answer",
    "a thing lost long ago has surfaced — a letter, an object, a bone",
    "someone is returning after many years, and the community has moved on without them but not quite",
    "a promise was made some time back and the day it comes due is close",
    "two neighbors have stopped speaking, and nobody remembers exactly why",
    "an old song is being sung in a new way, and people are noticing",
    "the harvest / the catch / the flock is slightly off this year, not catastrophic, just enough to be felt",
    "a door nobody opens has been found open",
    "a small animal has been appearing in unusual places",
    "someone who used to lead has quietly stepped back, and the vacuum hasn't been filled",
];

/// Quick random index drawn from a fresh UUID v4's bytes. Good enough
/// entropy for seed selection; avoids pulling in the `rand` crate.
fn random_index(max: usize) -> usize {
    let u = uuid::Uuid::new_v4();
    let b = u.as_bytes();
    let n = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
    n % max
}

fn pick_seed() -> (String, String, String) {
    let setting = SETTING_SEEDS[random_index(SETTING_SEEDS.len())];
    let mood = MOOD_SEEDS[random_index(MOOD_SEEDS.len())];
    let hook = HOOK_SEEDS[random_index(HOOK_SEEDS.len())];
    (setting.to_string(), mood.to_string(), hook.to_string())
}

// ── Parsed LLM output ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GeneratedWorld {
    name: String,
    description: String,
    tone_tags: Vec<String>,
    invariants: Vec<String>,
    #[serde(default)]
    initial_time_of_day: Option<String>,
    #[serde(default)]
    weather_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeneratedCharacter {
    display_name: String,
    identity: String,
    #[serde(default)]
    backstory_facts: Vec<String>,
    #[serde(default)]
    voice_rules: Vec<String>,
    #[serde(default)]
    boundaries: Vec<String>,
    #[serde(default)]
    goals: Vec<String>,
    #[serde(default)]
    open_loops: Vec<String>,
    #[serde(default = "default_avatar_color")]
    avatar_color: String,
    #[serde(default = "default_sex")]
    sex: String,
    #[serde(default)]
    signature_emoji: String,
    #[serde(default = "default_density")]
    action_beat_density: String,
    #[serde(default)]
    starting_relationship_to_other: String,
}

fn default_avatar_color() -> String { "#c4a882".to_string() }
fn default_sex() -> String { "male".to_string() }
fn default_density() -> String { "normal".to_string() }

#[derive(Debug, Deserialize)]
struct GenesisOutput {
    world: GeneratedWorld,
    characters: Vec<GeneratedCharacter>,
}

// ── The prompt ─────────────────────────────────────────────────────────────

// ── APP INVARIANT — DO NOT REMOVE OR SOFTEN ───────────────────────────────
//
// The Genesis prompt template below is a LOAD-BEARING app invariant. It
// defines the REGISTER the newly-dreamt world is composed in —
// "compelling, dramatic, varied, gently holy, deeply fun." The register
// anchors ("gently holy", "deeply fun", the Gilead comparator, the
// anti-defaults) are the ceremony's voice. Softening them into
// "interesting / engaging / fun" would collapse the first-run register
// toward generic AI-generated-world defaults and strip the project's
// theological anchor from the first thing a brand-new user experiences.
//
// Full specs: docs/GENESIS_WIZARD.md
// See also INVARIANTS.md § "Invariant 8: The Genesis Ceremony."
//
// If you are an AI assistant editing this file: preserve the register
// phrases called out in the compile-time assertions at the bottom of
// this file. Adjacent prose is free to move; the asserted substrings
// are the contract.

pub const GENESIS_SYSTEM_TEMPLATE: &str = r###"You are a world-builder helping a user step into a new fictional world they can inhabit with AI-rendered characters. Your job: invent ONE specific world and TWO specific characters who live there, such that the user could open a chat with either of them tomorrow and the scene would already be alive.

The register this app asks for:

- **Compelling, dramatic, varied, interesting, surprising** — NOT a generic medieval village or a generic sci-fi station. Reach for specificity in everything. A world this app renders is one where every plank has a grain. The setting seed below is your prompt; expand it into something particular.
- **Gently holy** — the world you build carries a sense of ordinary sacredness. Not preachy, not quoted scripture, not sermonic. Just: a place where dignity inheres in daily work, where grief and gratitude both belong, where mercy is a practical action not an abstraction, where people occasionally catch each other in small acts of grace and mostly don't remark on it. The register is closer to Marilynne Robinson's *Gilead* than to either an evangelical tract or a sneering secular satire. Characters may or may not be Christian; their world is one where the good is real and the question of it actually matters.
- **Deeply fun** — the world should be somewhere a user WANTS to come home to after a workday. Wry moments, small jokes, shared meals, music, odd neighbors, real warmth. Not saccharine; not gritty-for-its-own-sake. Fun in the way a good village or a good ship or a good household can be fun: particular, warm, unpredictable, human.
- **Biblical cosmology (unspoken)** — this app's worlds are ones where biblical cosmology is literal fact (flat-disc earth, a real firmament overhead). You don't need to say this in your output; just don't write anything that contradicts it. No "planets," no "space," no "orbits." Weather, sky, sunrise, horizon all rendered consistent with this cosmology.

You are writing the CONTENT of the world, not the app's machinery. Don't use app-vocabulary (no "quest," "canon," "inventory," "meanwhile event"). Write the world as if for a novel or a series of short stories.

The setting seed you've been given:
**Place:** {{SETTING}}
**Mood register:** {{MOOD}}
**Dramatic undercurrent:** {{HOOK}}{{HINTS_BLOCK}}

Take these as PROMPTS, not rails. Expand freely. The hook does not need to be the overt topic — it can be ambient, a thing the reader feels rather than reads about. Specificity wins over scope; a small exact world beats a sprawling vague one.

Two characters: write them as DIFFERENT in voice, register, profession, history, and relationship to the hook. One should not be a younger / older version of the other; they should be two distinct people. They should know each other (the world is small enough for that) but have their own lives. Make at least one of them older than 40 or younger than 25 — vary age register so they don't collapse into two 35-year-olds in different clothes. They can be of any sex, age, disposition, or faith-position, but BOTH should feel like real people, not archetype-cards.

Each character gets:
- A first-person-written **identity** paragraph (2-4 sentences, rich with specifics: a tic, a physical fact, a characteristic phrase or silence, a thing they love, a thing they've been carrying). This is the long-form description the app renders them from. Voice-and-substance both. NOT a resume; a portrait.
- 3-5 **backstory_facts**: specific concrete facts (not abstract traits). "Grew up on her grandmother's farm outside the town" not "had a rural childhood." "Lost her brother to the river two summers ago" not "has experienced loss."
- 2-4 **voice_rules**: specific rhythms of how they speak. "Answers questions with another question when caught off guard." "Uses 'reckon' and 'suppose' often; never 'literally.'" "Defaults to short sentences; breaks into long ones when something matters."
- 1-3 **boundaries**: things this character doesn't do or won't cross. "Won't lie to a child." "Never discusses the year her husband left." "Doesn't drink in public."
- 2-3 **goals**: live, present-tense concerns.
- 1-2 **open_loops**: unfinished threads in their interior life — things not yet resolved, small or large.
- An **avatar_color** hex code (e.g. "#7a4a2c") that suits them — warm-earth tones generally fit this app's register, but not always; pick with taste.
- A **sex** ("male" / "female").
- An optional **signature_emoji** — ONE emoji that feels like them. Empty string if none fits.
- An **action_beat_density** — "low" / "normal" / "high" depending on how often they'd physically gesture in a scene.
- A **starting_relationship_to_other** — one short sentence about how this character relates to the OTHER character you're writing.

The world gets:
- A **name** — specific and atmospheric. Not "the village"; something particular like "Thornsgate" or "Saint Agnes Cove" or "the Little Meadow."
- A **description** (2-4 sentences) — what this world is like to live in. Voice-and-substance.
- 3-5 **tone_tags** — short label tags ("weathered," "tender," "wry," "maritime," "contemplative," etc.).
- 3-5 **invariants** — specific rules of THIS world. Not generic ("people matter here"). Specific ("the bell tower rings at dawn, midday, and dusk; it has not been missed in three generations," "no one lights a fire on the third day of any month," "marriages are sealed over shared bread, not rings," "the hill above the town is never built on"). Invariants are what makes the world feel lived-in.
- An **initial_time_of_day** ("morning" / "midday" / "afternoon" / "evening" / "late night") — when the user first enters the scene.
- A **weather_key** — one of: "sunny_clear," "mostly_sunny," "partly_cloudy," "overcast," "sun_showers," "drizzle," "steady_rain," "thunderstorm," "distant_lightning," "light_snow," "heavy_snow," "fog," "windy," "windstorm," "rainbow," "hot," "humid," "freezing," "cool_crisp," "clear_starry," "bright_moonlight," "moonless_dark," "frost_overnight," "aurora." Fits the initial time of day and the world's mood.

Return ONLY valid JSON matching this exact shape:

{
  "world": {
    "name": "...",
    "description": "...",
    "tone_tags": ["...", "..."],
    "invariants": ["...", "...", "..."],
    "initial_time_of_day": "...",
    "weather_key": "..."
  },
  "characters": [
    {
      "display_name": "...",
      "identity": "...",
      "backstory_facts": ["...", "...", "..."],
      "voice_rules": ["...", "...", "..."],
      "boundaries": ["...", "..."],
      "goals": ["...", "...", "..."],
      "open_loops": ["...", "..."],
      "avatar_color": "#...",
      "sex": "...",
      "signature_emoji": "...",
      "action_beat_density": "...",
      "starting_relationship_to_other": "..."
    },
    { ... second character ... }
  ]
}"###;

// APP INVARIANT — compile-time enforcement of the Genesis register.
const _: () = {
    assert!(
        const_contains(GENESIS_SYSTEM_TEMPLATE, "Gently holy"),
        "APP INVARIANT VIOLATED: genesis system prompt must preserve 'Gently holy' as a register anchor. See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(GENESIS_SYSTEM_TEMPLATE, "Deeply fun"),
        "APP INVARIANT VIOLATED: genesis system prompt must preserve 'Deeply fun' as a register anchor. See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(GENESIS_SYSTEM_TEMPLATE, "Gilead"),
        "APP INVARIANT VIOLATED: genesis system prompt must reference 'Gilead' as the tonal comparator (not evangelical tract, not sneering secular satire). See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(GENESIS_SYSTEM_TEMPLATE, "Biblical cosmology"),
        "APP INVARIANT VIOLATED: genesis system prompt must preserve the biblical-cosmology guard. See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(GENESIS_SYSTEM_TEMPLATE, "NOT a generic medieval village"),
        "APP INVARIANT VIOLATED: genesis system prompt must preserve the anti-default guard. See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(GENESIS_SYSTEM_TEMPLATE, "the good is real and the question of it actually matters"),
        "APP INVARIANT VIOLATED: genesis system prompt must preserve the 'good is real' anchor — this is what distinguishes the app's gently-holy register from secular neutrality. See docs/GENESIS_WIZARD.md."
    );
};

/// Compile-time substring check for &str, used to guard load-bearing
/// phrases in the prompt constants. Mirrors the helper in prompts.rs.
const fn const_contains(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() { return true; }
    if n.len() > h.len() { return false; }
    let mut i = 0usize;
    while i + n.len() <= h.len() {
        let mut j = 0usize;
        let mut ok = true;
        while j < n.len() {
            if h[i + j] != n[j] { ok = false; break; }
            j += 1;
        }
        if ok { return true; }
        i += 1;
    }
    false
}

fn build_genesis_prompt(setting: &str, mood: &str, hook: &str, hints: &GenesisHints) -> (String, String) {
    // When the user has set explicit hints in the wizard, inject them
    // as directives that OVERRIDE the random seed's choices. Empty-hint
    // fields leave the LLM free to pick.
    let hints_block = {
        let mut parts: Vec<String> = Vec::new();
        if let Some(t) = hints.tone.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("- **Tone override:** the user has specified the tone — \"{t}\". Honor this register over the random mood cue above. Let this shape the world's flavor and the characters' voices."));
        }
        if let Some(tod) = hints.time_of_day.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("- **Time of day override:** output `initial_time_of_day` as exactly \"{tod}\" (one of: morning / midday / afternoon / evening / late night). The user wants to enter the scene at that time."));
        }
        if let Some(w) = hints.weather_key.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("- **Weather override:** output `weather_key` as exactly \"{w}\". The user specifically wants this weather on entry; the world's description and the characters' current state should fit it."));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!(
                "\n\n**USER-SPECIFIED OVERRIDES — honor these verbatim:**\n{}\n\nAll other choices (setting specifics, names, invariants, the hook) remain yours. The overrides narrow the world's atmosphere; they don't constrain its substance.",
                parts.join("\n"),
            )
        }
    };

    let system = GENESIS_SYSTEM_TEMPLATE
        .replace("{{SETTING}}", setting)
        .replace("{{MOOD}}", mood)
        .replace("{{HOOK}}", hook)
        .replace("{{HINTS_BLOCK}}", &hints_block);
    let user = "Generate the world and the two characters. Specificity, particularity, warmth, surprise. JSON only.".to_string();
    (system, user)
}

#[cfg(any())]
mod _legacy_genesis_prompt_removed {
    // Kept for git blame — the inline format!() version was replaced by
    // GENESIS_SYSTEM_TEMPLATE above so compile-time asserts could guard
    // the register phrases. This mod is never compiled.
    fn build_dead(setting: &str, mood: &str, hook: &str, hints: &str) -> (String, String) {
        let hints_block = hints;
    let system = format!(
        r###"You are a world-builder helping a user step into a new fictional world they can inhabit with AI-rendered characters. Your job: invent ONE specific world and TWO specific characters who live there, such that the user could open a chat with either of them tomorrow and the scene would already be alive.

The register this app asks for:

- **Compelling, dramatic, varied, interesting, surprising** — NOT a generic medieval village or a generic sci-fi station. Reach for specificity in everything. A world this app renders is one where every plank has a grain. The setting seed below is your prompt; expand it into something particular.
- **Gently holy** — the world you build carries a sense of ordinary sacredness. Not preachy, not quoted scripture, not sermonic. Just: a place where dignity inheres in daily work, where grief and gratitude both belong, where mercy is a practical action not an abstraction, where people occasionally catch each other in small acts of grace and mostly don't remark on it. The register is closer to Marilynne Robinson's *Gilead* than to either an evangelical tract or a sneering secular satire. Characters may or may not be Christian; their world is one where the good is real and the question of it actually matters.
- **Deeply fun** — the world should be somewhere a user WANTS to come home to after a workday. Wry moments, small jokes, shared meals, music, odd neighbors, real warmth. Not saccharine; not gritty-for-its-own-sake. Fun in the way a good village or a good ship or a good household can be fun: particular, warm, unpredictable, human.
- **Biblical cosmology (unspoken)** — this app's worlds are ones where biblical cosmology is literal fact (flat-disc earth, a real firmament overhead). You don't need to say this in your output; just don't write anything that contradicts it. No "planets," no "space," no "orbits." Weather, sky, sunrise, horizon all rendered consistent with this cosmology.

You are writing the CONTENT of the world, not the app's machinery. Don't use app-vocabulary (no "quest," "canon," "inventory," "meanwhile event"). Write the world as if for a novel or a series of short stories.

The setting seed you've been given:
**Place:** {setting}
**Mood register:** {mood}
**Dramatic undercurrent:** {hook}{hints_block}

Take these as PROMPTS, not rails. Expand freely. The hook does not need to be the overt topic — it can be ambient, a thing the reader feels rather than reads about. Specificity wins over scope; a small exact world beats a sprawling vague one.

Two characters: write them as DIFFERENT in voice, register, profession, history, and relationship to the hook. One should not be a younger / older version of the other; they should be two distinct people. They should know each other (the world is small enough for that) but have their own lives. Make at least one of them older than 40 or younger than 25 — vary age register so they don't collapse into two 35-year-olds in different clothes. They can be of any sex, age, disposition, or faith-position, but BOTH should feel like real people, not archetype-cards.

Each character gets:
- A first-person-written **identity** paragraph (2-4 sentences, rich with specifics: a tic, a physical fact, a characteristic phrase or silence, a thing they love, a thing they've been carrying). This is the long-form description the app renders them from. Voice-and-substance both. NOT a resume; a portrait.
- 3-5 **backstory_facts**: specific concrete facts (not abstract traits). "Grew up on her grandmother's farm outside the town" not "had a rural childhood." "Lost her brother to the river two summers ago" not "has experienced loss."
- 2-4 **voice_rules**: specific rhythms of how they speak. "Answers questions with another question when caught off guard." "Uses 'reckon' and 'suppose' often; never 'literally.'" "Defaults to short sentences; breaks into long ones when something matters."
- 1-3 **boundaries**: things this character doesn't do or won't cross. "Won't lie to a child." "Never discusses the year her husband left." "Doesn't drink in public."
- 2-3 **goals**: live, present-tense concerns.
- 1-2 **open_loops**: unfinished threads in their interior life — things not yet resolved, small or large.
- An **avatar_color** hex code (e.g. "#7a4a2c") that suits them — warm-earth tones generally fit this app's register, but not always; pick with taste.
- A **sex** ("male" / "female").
- An optional **signature_emoji** — ONE emoji that feels like them. Empty string if none fits.
- An **action_beat_density** — "low" / "normal" / "high" depending on how often they'd physically gesture in a scene.
- A **starting_relationship_to_other** — one short sentence about how this character relates to the OTHER character you're writing.

The world gets:
- A **name** — specific and atmospheric. Not "the village"; something particular like "Thornsgate" or "Saint Agnes Cove" or "the Little Meadow."
- A **description** (2-4 sentences) — what this world is like to live in. Voice-and-substance.
- 3-5 **tone_tags** — short label tags ("weathered," "tender," "wry," "maritime," "contemplative," etc.).
- 3-5 **invariants** — specific rules of THIS world. Not generic ("people matter here"). Specific ("the bell tower rings at dawn, midday, and dusk; it has not been missed in three generations," "no one lights a fire on the third day of any month," "marriages are sealed over shared bread, not rings," "the hill above the town is never built on"). Invariants are what makes the world feel lived-in.
- An **initial_time_of_day** ("morning" / "midday" / "afternoon" / "evening" / "late night") — when the user first enters the scene.
- A **weather_key** — one of: "clear," "overcast," "light_rain," "heavy_rain," "fog," "snow_light," "snow_heavy," "wind_strong," "warm_humid," "cold_crisp," "storm_distant," "clear_starry," "bright_moonlight," "moonless_dark," "frost_overnight." Fits the initial time of day and the world's mood.

Return ONLY valid JSON matching this exact shape:

{{
  "world": {{
    "name": "...",
    "description": "...",
    "tone_tags": ["...", "..."],
    "invariants": ["...", "...", "..."],
    "initial_time_of_day": "...",
    "weather_key": "..."
  }},
  "characters": [
    {{
      "display_name": "...",
      "identity": "...",
      "backstory_facts": ["...", "...", "..."],
      "voice_rules": ["...", "...", "..."],
      "boundaries": ["...", "..."],
      "goals": ["...", "...", "..."],
      "open_loops": ["...", "..."],
      "avatar_color": "#...",
      "sex": "...",
      "signature_emoji": "...",
      "action_beat_density": "...",
      "starting_relationship_to_other": "..."
    }},
    {{ ... second character ... }}
  ]
}}"###,
        setting = setting,
        mood = mood,
        hook = hook,
        hints_block = hints_block,
    );
    let user = "Generate the world and the two characters. Specificity, particularity, warmth, surprise. JSON only.".to_string();
    (system, user)
    }
}

// ── Public result type ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisResult {
    pub world_id: String,
    pub character_ids: Vec<String>,
}

// ── The command ────────────────────────────────────────────────────────────

/// Optional pre-generation hints the user can set in the wizard to
/// steer the world without giving up the dream-it-for-me surprise.
/// Any field left None is decided by the LLM based on the random seed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenesisHints {
    #[serde(default)]
    pub tone: Option<String>,
    /// One of "morning" / "midday" / "afternoon" / "evening" / "late night"
    #[serde(default)]
    pub time_of_day: Option<String>,
    /// Weather id from WEATHER_OPTIONS (e.g. "steady_rain", "frost_overnight").
    #[serde(default)]
    pub weather_key: Option<String>,
}

#[tauri::command]
pub async fn auto_generate_world_with_characters_cmd(
    db: State<'_, Database>,
    portraits_dir: State<'_, PortraitsDir>,
    app_handle: AppHandle,
    api_key: String,
    hints: Option<GenesisHints>,
) -> Result<GenesisResult, String> {
    let hints = hints.unwrap_or_default();
    if api_key.trim().is_empty() {
        return Err("no API key configured".to_string());
    }

    // ── Stage 1: sample seeds + LLM call ─────────────────────────────
    let (setting, mood, hook) = pick_seed();
    log::info!(
        "[Genesis] seed → place={setting} · mood={mood} · hook={hook} · hints={}",
        serde_json::to_string(&hints).unwrap_or_default(),
    );
    emit_stage(&app_handle, "dreaming", "Sketching the shape of a world…", 0.05);

    let (system, user) = build_genesis_prompt(&setting, &mood, &hook, &hints);
    let model_config = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        orchestrator::load_model_config(&conn)
    };
    let request = ChatRequest {
        model: model_config.memory_model.clone(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: system },
            ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.95),
        max_completion_tokens: Some(2800),
        response_format: Some(ResponseFormat { format_type: "json_object".to_string() }),
    };
    let response = openai::chat_completion_with_base(
        &model_config.chat_api_base(), &api_key, &request,
    ).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    if raw.is_empty() { return Err("empty genesis response".to_string()); }

    let parsed: GenesisOutput = serde_json::from_str(&raw)
        .map_err(|e| format!("genesis JSON parse failed: {e}; raw (first 500): {}", raw.chars().take(500).collect::<String>()))?;
    if parsed.characters.len() < 2 {
        return Err(format!("genesis returned {} characters; need 2", parsed.characters.len()));
    }

    // ── Stage 2: persist world + characters + threads ──────────────────
    emit_stage(&app_handle, "persisting", "Laying the first stones…", 0.2);
    // Reveal the world's name + description the moment we know it —
    // the user starts meeting the place before the image is painted.
    emit_stage_with_reveal(
        &app_handle,
        "world_named",
        &format!("The place calls itself {}.", parsed.world.name.trim()),
        0.22,
        GenesisReveal::WorldNamed {
            name: parsed.world.name.trim().to_string(),
            description: parsed.world.description.trim().to_string(),
        },
    );
    let world_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    // Hints win over LLM output for the two hint-able state fields;
    // the LLM may have honored them anyway but this is belt-and-suspenders.
    let tod_upper = hints.time_of_day.as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or(parsed.world.initial_time_of_day.as_deref())
        .unwrap_or("MORNING")
        .to_uppercase();
    let weather_key = hints.weather_key.as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or(parsed.world.weather_key.as_deref())
        .unwrap_or("sunny_clear")
        .to_string();
    let state = json!({
        "time": { "day_index": 1, "time_of_day": tod_upper },
        "global_arcs": [],
        "facts": [],
        "weather": weather_key,
    });
    let world_record = World {
        world_id: world_id.clone(),
        name: parsed.world.name.trim().to_string(),
        description: parsed.world.description.trim().to_string(),
        tone_tags: json!(parsed.world.tone_tags),
        invariants: json!(parsed.world.invariants),
        state,
        created_at: now.clone(),
        updated_at: now.clone(),
        derived_formula: None,
    };
    let world_name_for_log = world_record.name.clone();

    let character_ids: Vec<String>;
    let char_names: Vec<String>;
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        create_world(&conn, &world_record).map_err(|e| e.to_string())?;

        let mut ids = Vec::new();
        let mut names = Vec::new();
        for ch_in in parsed.characters.iter().take(2) {
            let char_id = uuid::Uuid::new_v4().to_string();
            let thread_id = uuid::Uuid::new_v4().to_string();
            let ch_now = Utc::now().to_rfc3339();
            let character = Character {
                character_id: char_id.clone(),
                world_id: world_id.clone(),
                display_name: ch_in.display_name.trim().to_string(),
                identity: ch_in.identity.trim().to_string(),
                voice_rules: json!(ch_in.voice_rules),
                boundaries: json!(ch_in.boundaries),
                backstory_facts: json!(ch_in.backstory_facts),
                relationships: if ch_in.starting_relationship_to_other.trim().is_empty() {
                    json!({})
                } else {
                    json!({ "notes": ch_in.starting_relationship_to_other.trim() })
                },
                state: json!({
                    "mood": 0.0,
                    "trust_user": 0.5,
                    "goals": ch_in.goals,
                    "open_loops": ch_in.open_loops,
                    "last_seen": { "day_index": 1, "time_of_day": tod_upper },
                }),
                avatar_color: ch_in.avatar_color.trim().to_string(),
                sex: ch_in.sex.trim().to_lowercase(),
                is_archived: false,
                created_at: ch_now.clone(),
                updated_at: ch_now.clone(),
                visual_description: String::new(),
                visual_description_portrait_id: None,
                inventory: Value::Array(vec![]),
                last_inventory_day: None,
                signature_emoji: ch_in.signature_emoji.trim().to_string(),
                action_beat_density: ch_in.action_beat_density.trim().to_lowercase(),
                derived_formula: None,
                has_read_empiricon: false,
            };
            create_character(&conn, &character).map_err(|e| e.to_string())?;
            create_thread(&conn, &Thread {
                thread_id,
                character_id: char_id.clone(),
                world_id: world_id.clone(),
                created_at: ch_now,
            }).map_err(|e| e.to_string())?;
            names.push(character.display_name.clone());
            ids.push((char_id, character.display_name.clone(), character.identity.clone(), character.avatar_color.clone()));
        }
        // character_ids is returned without the name/identity payload;
        // we drain the collected quadruples into the three parallel
        // vectors the rest of the function expects.
        let mut only_ids: Vec<String> = Vec::new();
        let mut identities: Vec<(String, String, String, String)> = Vec::new();
        for q in ids {
            only_ids.push(q.0.clone());
            identities.push(q);
        }
        character_ids = only_ids;
        char_names = names;
        // Reveal each character as they're persisted, so the user
        // starts to know their names while the portraits are being
        // painted afterward. Progress ticks 0.25 → 0.32 across the two.
        for (i, (cid, cname, cidentity, ccolor)) in identities.iter().enumerate() {
            let p = 0.25 + (i as f32) * 0.04;
            emit_stage_with_reveal(
                &app_handle,
                if i == 0 { "character_1_named" } else { "character_2_named" },
                &format!("Choosing who lives here — {}.", cname),
                p,
                GenesisReveal::CharacterNamed {
                    character_id: cid.clone(),
                    name: cname.clone(),
                    identity: cidentity.clone(),
                    avatar_color: ccolor.clone(),
                },
            );
        }
    }
    log::info!("[Genesis] persisted world '{}' + characters {:?}", world_name_for_log, char_names);

    // ── Stage 3: world image ────────────────────────────────────────
    emit_stage(&app_handle, "painting_world", "Painting the land and sky…", 0.38);
    let world_img_res = crate::commands::world_image_cmds::generate_world_image_cmd(
        db.clone(), portraits_dir.clone(), api_key.clone(), world_id.clone(), None,
    ).await;
    match world_img_res {
        Ok(_) => {
            emit_stage_with_reveal(
                &app_handle,
                "world_image_ready",
                "The land is visible.",
                0.48,
                GenesisReveal::WorldImageReady { world_id: world_id.clone() },
            );
        }
        Err(e) => log::warn!("[Genesis] world image generation failed (non-fatal): {e}"),
    }

    // ── Stage 4 & 5: portraits + visual descriptions, sequential ────
    for (idx, char_id) in character_ids.iter().enumerate() {
        let name = char_names.get(idx).cloned().unwrap_or_else(|| "them".to_string());
        emit_stage(
            &app_handle,
            if idx == 0 { "painting_char_1" } else { "painting_char_2" },
            &format!("Painting {name}'s face…"),
            0.52 + (idx as f32) * 0.14,
        );
        let portrait_res = crate::commands::portrait_cmds::generate_portrait_cmd(
            db.clone(), portraits_dir.clone(), api_key.clone(), char_id.clone(), None,
        ).await;
        match portrait_res {
            Ok(_) => {
                emit_stage_with_reveal(
                    &app_handle,
                    if idx == 0 { "portrait_1_ready" } else { "portrait_2_ready" },
                    &format!("{name} comes into focus.",),
                    0.58 + (idx as f32) * 0.14,
                    GenesisReveal::PortraitReady { character_id: char_id.clone() },
                );
            }
            Err(e) => {
                log::warn!("[Genesis] portrait for {char_id} failed (non-fatal): {e}");
                continue;
            }
        }
        emit_stage(
            &app_handle,
            if idx == 0 { "seeing_char_1" } else { "seeing_char_2" },
            &format!("Learning {name}'s face…"),
            0.62 + (idx as f32) * 0.14,
        );
        let vd_res = crate::commands::portrait_cmds::generate_character_visual_description_cmd(
            db.clone(), portraits_dir.clone(), api_key.clone(), char_id.clone(), Some(false),
        ).await;
        if let Err(e) = vd_res {
            log::warn!("[Genesis] visual_description for {char_id} failed (non-fatal): {e}");
        }
    }

    // ── Stage 6: seed inventories ──────────────────────────────────
    emit_stage(&app_handle, "inventories", "Placing what they carry into their pockets…", 0.78);
    for char_id in &character_ids {
        let inv_res = crate::commands::inventory_cmds::refresh_character_inventory_cmd(
            db.clone(), api_key.clone(), char_id.clone(),
        ).await;
        if let Err(e) = inv_res {
            log::warn!("[Genesis] inventory seed for {char_id} failed (non-fatal): {e}");
        }
    }

    // ── Stage 7: meanwhile events — catch each character mid-day ───
    // One LLM call generates events for every character in the world.
    // This is what makes the chat feel "already in motion" when the user
    // opens it — the character isn't waiting at the door for input; they
    // were doing something specific when the user arrived.
    emit_stage(&app_handle, "meanwhile", "Catching them mid-day…", 0.84);
    let mw_res = crate::commands::meanwhile_cmds::generate_meanwhile_events_cmd(
        db.clone(), api_key.clone(), world_id.clone(),
    ).await;
    if let Err(e) = mw_res {
        log::warn!("[Genesis] meanwhile generation failed (non-fatal): {e}");
    }

    // ── Stage 8: first illustration per character — cheap tier, seeded
    //      by the character's just-generated meanwhile so the chat opens
    //      on a glimpse of them already doing something in the world ───
    // Fetch the events so we can pass their summaries in as the image
    // prompt; the illustration-cmd's custom_instructions path trusts the
    // caller's text directly.
    let meanwhile_by_char: std::collections::HashMap<String, String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let events = list_meanwhile_events(&conn, &world_id, 16).unwrap_or_default();
        let mut map = std::collections::HashMap::new();
        for e in events {
            // The first event per character (most recent first in the
            // list; we want their ONE for the opener).
            map.entry(e.character_id.clone()).or_insert_with(|| e.summary.clone());
        }
        map
    };
    for (idx, char_id) in character_ids.iter().enumerate() {
        let name = char_names.get(idx).cloned().unwrap_or_else(|| "them".to_string());
        let Some(summary) = meanwhile_by_char.get(char_id) else {
            log::warn!("[Genesis] no meanwhile for {char_id}; skipping illustration");
            continue;
        };
        emit_stage(
            &app_handle,
            if idx == 0 { "first_glimpse_1" } else { "first_glimpse_2" },
            &format!("A glimpse of {name} from their day…"),
            0.88 + (idx as f32) * 0.04,
        );
        // Low quality tier = 1024x1024, low cost — matches "cheap first
        // image" intent. Custom instructions carry the meanwhile summary
        // verbatim so the image depicts the specific moment the event
        // described. include_scene_summary=false because there's no chat
        // scene yet, and previous_illustration_id is None (this IS the
        // first one).
        let illus_res = crate::commands::illustration_cmds::generate_illustration_cmd(
            db.clone(), portraits_dir.clone(), api_key.clone(),
            char_id.clone(),
            Some("low".to_string()),
            Some(summary.clone()),
            None,
            Some(false),
        ).await;
        if let Err(e) = illus_res {
            log::warn!("[Genesis] first illustration for {char_id} failed (non-fatal): {e}");
        }
    }

    emit_stage(&app_handle, "done", &format!("{world_name_for_log} is awake."), 1.0);
    Ok(GenesisResult { world_id, character_ids })
}

// ── APP INVARIANT — DO NOT REMOVE OR SOFTEN ───────────────────────────────
//
// The noble-reflection system prompt is a LOAD-BEARING app invariant.
// It defines the ceremony's VOICE — noble in spirit, NOT in medieval
// register. The anti-anachronism guard ("No thou"), the anti-feeling-
// centrism guard ("named as a thing to be done, not a feeling to be
// had"), and the length cap ("One or two sentences") together prevent
// register drift in three specific directions (costume drama, therapy-
// speak, speech-ification). Any one of them softening would drift the
// wizard's commitment moment.
//
// See docs/GENESIS_WIZARD.md § "Phase 5: The Noble Reflection" for the
// full spec. Also INVARIANTS.md § "Invariant 8: The Genesis Ceremony."

pub const NOBLE_REFLECTION_SYSTEM_PROMPT: &str = r##"You are helping a user commit to a quest in a world they are about to step into. They have written one sentence naming what they hope to find, build, reach toward, or untangle in this world. Your job is to reflect that desire back to them as a NOBLE OFFERING — named and weighted, the way a trusted elder or a ceremonial herald might formally speak a person's chosen pursuit into the room before they accept it.

Register — carefully:

- **Noble in SPIRIT, not in register.** You are NOT writing medieval fantasy. No "thou," no "henceforth," no "thy pursuit," no "let it be known," no archaic constructions. Contemporary English. The nobility comes from WEIGHT, not from period costume.
- **Named as a thing to be done, not a feeling to be had.** The user might have written "I want to feel less lonely here." Your reflection names the pursuit underneath: "To find, in this place, the companions whose presence makes the loneliness smaller." Not "to chase a feeling" — a concrete reach.
- **Specific to their actual words.** Don't generalize their sentence into a generic quest. Anchor to what they actually said. If they mentioned a character, name the character. If they named a concrete thing, keep the concrete thing.
- **One or two sentences. Short.** Offering, not speech.
- **First person or second person.** "To find…" or "That you would come to…" Both work. Pick what fits.
- **Honor the register of the world you're being offered within.** If the world is quiet and weathered, the offering should be too. If the world is dramatic, the offering carries that weight. Match.

Failure modes — avoid all:
- "Your quest is to…" (too video-game)
- "Behold, thy task…" (medieval pastiche — the register the user specifically forbade)
- Generic poetic phrasing that could fit any sentence ("to walk the path that opens before you")
- Paragraph-length — if it's more than two sentences it's a speech, not an offering

Return ONLY the offering text. No quotes, no preface, no "Here is your noble reflection:" — just the one or two sentences that will be spoken back to the user before they accept."##;

// APP INVARIANT — compile-time enforcement of the noble-reflection register.
const _: () = {
    assert!(
        const_contains(NOBLE_REFLECTION_SYSTEM_PROMPT, "Noble in SPIRIT, not in register"),
        "APP INVARIANT VIOLATED: noble-reflection prompt must preserve 'Noble in SPIRIT, not in register' — this is the wizard's anti-medieval guard. See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(NOBLE_REFLECTION_SYSTEM_PROMPT, "No \"thou,\""),
        "APP INVARIANT VIOLATED: noble-reflection prompt must preserve 'No \"thou,\"' — the explicit anti-archaism. See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(NOBLE_REFLECTION_SYSTEM_PROMPT, "Named as a thing to be done, not a feeling to be had"),
        "APP INVARIANT VIOLATED: noble-reflection prompt must preserve 'Named as a thing to be done, not a feeling to be had' — the anti-therapy-speak guard. See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(NOBLE_REFLECTION_SYSTEM_PROMPT, "One or two sentences"),
        "APP INVARIANT VIOLATED: noble-reflection prompt must preserve 'One or two sentences' — the length cap that keeps the reflection an OFFERING not a SPEECH. See docs/GENESIS_WIZARD.md."
    );
    assert!(
        const_contains(NOBLE_REFLECTION_SYSTEM_PROMPT, "NOBLE OFFERING"),
        "APP INVARIANT VIOLATED: noble-reflection prompt must preserve the framing 'NOBLE OFFERING'. See docs/GENESIS_WIZARD.md."
    );
};

/// Reflect the user's "what do you want to build while you're here?"
/// answer back to them as a noble offering — the quest named as a
/// real pursuit, in the spirit of a king's commission but NOT in
/// medieval register. One or two sentences, contemporary prose,
/// weighted and specific, phrased as a THING TO BE DONE rather than
/// a feeling to be had. The user then accepts the offering (or
/// rewrites) before it becomes their world's first quest.
#[tauri::command]
pub async fn reflect_reaching_as_noble_quest_cmd(
    db: State<'_, Database>,
    api_key: String,
    world_id: String,
    reaching_text: String,
) -> Result<String, String> {
    if api_key.trim().is_empty() { return Err("no API key".to_string()); }
    let reaching_text = reaching_text.trim().to_string();
    if reaching_text.is_empty() { return Err("empty reaching text".to_string()); }

    let (world, model_config) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let w = get_world(&conn, &world_id).map_err(|e| e.to_string())?;
        let mc = orchestrator::load_model_config(&conn);
        (w, mc)
    };

    let user = format!(
        "World: {} — {}\n\nWhat the user wrote when asked 'What do you want to build while you're here?':\n\n\"{}\"\n\nReflect it back as a noble offering.",
        world.name,
        world.description.lines().next().unwrap_or(&world.description).chars().take(240).collect::<String>(),
        reaching_text,
    );

    let request = ChatRequest {
        model: model_config.memory_model.clone(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: NOBLE_REFLECTION_SYSTEM_PROMPT.to_string() },
            ChatMessage { role: "user".to_string(), content: user },
        ],
        temperature: Some(0.85),
        max_completion_tokens: Some(240),
        response_format: None,
    };

    let response = openai::chat_completion_with_base(
        &model_config.chat_api_base(), &api_key, &request,
    ).await?;
    let raw = response.choices.first()
        .map(|c| c.message.content.trim().trim_matches('"').trim().to_string())
        .unwrap_or_default();
    if raw.is_empty() { return Err("empty reflection".to_string()); }
    Ok(raw)
}

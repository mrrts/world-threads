use app_lib::ai::prompts::{self, PromptOverrides};
use app_lib::db::queries::{Character, UserProfile, World};
use clap::Parser;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "character-identity-runtime-probe",
    about = "Emit the live dialogue system prompt for a fixture character under prose or compressed identity mode."
)]
struct Args {
    /// Fixture name under tests/fixtures/character_identity, or a direct JSON path.
    #[arg(long)]
    fixture: String,

    /// Identity carrier mode for the emitted prompt.
    #[arg(long, default_value = "prose")]
    mode: String,

    /// User message paired with the system prompt for downstream probes.
    #[arg(long)]
    user_message: String,

    /// Display name used for the synthetic user profile.
    #[arg(long, default_value = "Ryan")]
    user_name: String,

    /// Emit structured JSON rather than plain text.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct ProbeOutput {
    fixture: String,
    character_id: String,
    display_name: String,
    mode: String,
    user_name: String,
    user_message: String,
    system_prompt: String,
}

fn main() {
    let args = Args::parse();
    let character = load_fixture(&args.fixture);
    let system_prompt = build_probe_prompt(&character, &args.mode, &args.user_name);
    let out = ProbeOutput {
        fixture: args.fixture.clone(),
        character_id: character.character_id.clone(),
        display_name: character.display_name.clone(),
        mode: args.mode.clone(),
        user_name: args.user_name.clone(),
        user_message: args.user_message.clone(),
        system_prompt,
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&out).expect("probe output serializes")
        );
    } else {
        println!(
            "fixture: {}\ncharacter: {} [{}]\nmode: {}\nuser: {}\n\n=== SYSTEM PROMPT ===\n{}\n\n=== USER MESSAGE ===\n{}",
            out.fixture,
            out.display_name,
            out.character_id,
            out.mode,
            out.user_name,
            out.system_prompt,
            out.user_message
        );
    }
}

fn build_probe_prompt(character: &Character, mode: &str, user_name: &str) -> String {
    let world = minimal_world(&character.world_id);
    let profile = minimal_profile(&character.world_id, user_name);
    let mut overrides = PromptOverrides::new();
    if mode.eq_ignore_ascii_case("compressed") {
        overrides.insert("character_identity_mode", "compressed");
    }
    prompts::build_dialogue_system_prompt_with_overrides(
        &world,
        character,
        Some(&profile),
        None,
        Some("Auto"),
        None,
        None,
        false,
        &[],
        None,
        &[],
        None,
        &[],
        None,
        &[],
        None,
        None,
        Some(&overrides),
    )
}

fn load_fixture(input: &str) -> Character {
    let path = if input.ends_with(".json") || input.contains('/') {
        PathBuf::from(input)
    } else {
        Path::new("tests/fixtures/character_identity").join(format!("{input}.json"))
    };
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse fixture {}: {e}", path.display()))
}

fn minimal_world(world_id: &str) -> World {
    World {
        world_id: world_id.to_string(),
        name: "Probe World".into(),
        description: String::new(),
        tone_tags: serde_json::json!([]),
        invariants: serde_json::json!([]),
        state: serde_json::json!({}),
        created_at: String::new(),
        updated_at: String::new(),
        derived_formula: None,
    }
}

fn minimal_profile(world_id: &str, display_name: &str) -> UserProfile {
    UserProfile {
        world_id: world_id.to_string(),
        display_name: display_name.to_string(),
        description: String::new(),
        facts: serde_json::json!([]),
        boundaries: serde_json::json!([]),
        avatar_file: String::new(),
        updated_at: String::new(),
        derived_formula: None,
        derived_summary: None,
    }
}

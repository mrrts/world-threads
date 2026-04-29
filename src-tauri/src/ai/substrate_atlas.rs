//! Substrate atlas — single registry of every `pub fn build_*` entry point
//! under `src/ai/` that participates in LLM-shaped work (v1), plus a
//! discovery audit so new public builders cannot land without updating the
//! registry (v2).
//!
//! **v1:** `worldcli substrates` prints a markdown table (or `--json`).
//! **v2:** `cargo test` + `worldcli substrates --audit` compare this registry
//! to `include_str!` scans of the four AI source files; any drift fails.

use serde::Serialize;
use std::collections::BTreeSet;

/// One row in the atlas — discriminant doubles as the stable registry key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuildSubstrate {
    AnimationPrompt,
    ChapterFromImageSystemPrompt,
    CharacterDerivePrompt,
    ConsultantSystemPrompt,
    CorrectionNote,
    DialogueMessages,
    DialogueSystemPrompt,
    DialogueSystemPromptWithOverrides,
    DreamMessages,
    DreamSystemPrompt,
    MemoryUpdatePrompt,
    NarrativeSystemPrompt,
    ProactivePingMessages,
    ProactivePingSystemPrompt,
    SceneDescriptionPrompt,
    SceneInventionPrompt,
    UserInWorldDerivePrompt,
    UserInWorldDerivePromptWithChoices,
    WorldDerivePrompt,
}

impl BuildSubstrate {
    pub const ALL: &'static [BuildSubstrate] = &[
        BuildSubstrate::AnimationPrompt,
        BuildSubstrate::ChapterFromImageSystemPrompt,
        BuildSubstrate::CharacterDerivePrompt,
        BuildSubstrate::ConsultantSystemPrompt,
        BuildSubstrate::CorrectionNote,
        BuildSubstrate::DialogueMessages,
        BuildSubstrate::DialogueSystemPrompt,
        BuildSubstrate::DialogueSystemPromptWithOverrides,
        BuildSubstrate::DreamMessages,
        BuildSubstrate::DreamSystemPrompt,
        BuildSubstrate::MemoryUpdatePrompt,
        BuildSubstrate::NarrativeSystemPrompt,
        BuildSubstrate::ProactivePingMessages,
        BuildSubstrate::ProactivePingSystemPrompt,
        BuildSubstrate::SceneDescriptionPrompt,
        BuildSubstrate::SceneInventionPrompt,
        BuildSubstrate::UserInWorldDerivePrompt,
        BuildSubstrate::UserInWorldDerivePromptWithChoices,
        BuildSubstrate::WorldDerivePrompt,
    ];

    pub fn rust_fn(self) -> &'static str {
        match self {
            BuildSubstrate::AnimationPrompt => "build_animation_prompt",
            BuildSubstrate::ChapterFromImageSystemPrompt => "build_chapter_from_image_system_prompt",
            BuildSubstrate::CharacterDerivePrompt => "build_character_prompt",
            BuildSubstrate::ConsultantSystemPrompt => "build_consultant_system_prompt",
            BuildSubstrate::CorrectionNote => "build_correction_note",
            BuildSubstrate::DialogueMessages => "build_dialogue_messages",
            BuildSubstrate::DialogueSystemPrompt => "build_dialogue_system_prompt",
            BuildSubstrate::DialogueSystemPromptWithOverrides => "build_dialogue_system_prompt_with_overrides",
            BuildSubstrate::DreamMessages => "build_dream_messages",
            BuildSubstrate::DreamSystemPrompt => "build_dream_system_prompt",
            BuildSubstrate::MemoryUpdatePrompt => "build_memory_update_prompt",
            BuildSubstrate::NarrativeSystemPrompt => "build_narrative_system_prompt",
            BuildSubstrate::ProactivePingMessages => "build_proactive_ping_messages",
            BuildSubstrate::ProactivePingSystemPrompt => "build_proactive_ping_system_prompt",
            BuildSubstrate::SceneDescriptionPrompt => "build_scene_description_prompt",
            BuildSubstrate::SceneInventionPrompt => "build_scene_invention_prompt",
            BuildSubstrate::UserInWorldDerivePrompt => "build_user_in_world_prompt_owned",
            BuildSubstrate::UserInWorldDerivePromptWithChoices => "build_user_in_world_prompt_with_choices",
            BuildSubstrate::WorldDerivePrompt => "build_world_prompt",
        }
    }

    fn source_file(self) -> &'static str {
        match self {
            BuildSubstrate::CharacterDerivePrompt
            | BuildSubstrate::WorldDerivePrompt
            | BuildSubstrate::UserInWorldDerivePrompt
            | BuildSubstrate::UserInWorldDerivePromptWithChoices => "derivation.rs",
            BuildSubstrate::ConsultantSystemPrompt => "consultant.rs",
            BuildSubstrate::CorrectionNote => "conscience.rs",
            _ => "prompts.rs",
        }
    }

    fn family(self) -> &'static str {
        match self {
            BuildSubstrate::DialogueSystemPrompt | BuildSubstrate::DialogueSystemPromptWithOverrides => {
                "dialogue system (solo/group dispatch)"
            }
            BuildSubstrate::DialogueMessages => "dialogue messages",
            BuildSubstrate::ProactivePingSystemPrompt => "proactive ping system",
            BuildSubstrate::ProactivePingMessages => "proactive ping messages",
            BuildSubstrate::DreamSystemPrompt => "dream system",
            BuildSubstrate::DreamMessages => "dream messages",
            BuildSubstrate::MemoryUpdatePrompt => "memory tier",
            BuildSubstrate::NarrativeSystemPrompt => "immersive narrative",
            BuildSubstrate::SceneDescriptionPrompt | BuildSubstrate::SceneInventionPrompt => "novel / scene tools",
            BuildSubstrate::AnimationPrompt => "animation beat",
            BuildSubstrate::ChapterFromImageSystemPrompt => "imagined chapter",
            BuildSubstrate::CharacterDerivePrompt | BuildSubstrate::WorldDerivePrompt => {
                "derivation (worldcli / UI)"
            }
            BuildSubstrate::UserInWorldDerivePrompt | BuildSubstrate::UserInWorldDerivePromptWithChoices => {
                "user Me-character derivation"
            }
            BuildSubstrate::ConsultantSystemPrompt => "backstage consultant",
            BuildSubstrate::CorrectionNote => "conscience (non-LLM string; registry for completeness)",
        }
    }

    fn voice_pov(self) -> &'static str {
        match self {
            BuildSubstrate::DialogueSystemPrompt
            | BuildSubstrate::DialogueSystemPromptWithOverrides
            | BuildSubstrate::ProactivePingSystemPrompt => "in-character reply",
            BuildSubstrate::DreamSystemPrompt => "dream-prose (subconscious)",
            BuildSubstrate::NarrativeSystemPrompt => "second-person narrator to user",
            BuildSubstrate::ConsultantSystemPrompt => "consultant / craft voice",
            BuildSubstrate::MemoryUpdatePrompt => "memory-update schema voice",
            BuildSubstrate::SceneDescriptionPrompt
            | BuildSubstrate::SceneInventionPrompt
            | BuildSubstrate::AnimationPrompt
            | BuildSubstrate::ChapterFromImageSystemPrompt => "tool/narration hybrid (see block)",
            BuildSubstrate::CharacterDerivePrompt
            | BuildSubstrate::WorldDerivePrompt
            | BuildSubstrate::UserInWorldDerivePrompt
            | BuildSubstrate::UserInWorldDerivePromptWithChoices => "derivation synthesis",
            BuildSubstrate::CorrectionNote => "n/a (template fragment)",
            BuildSubstrate::DialogueMessages
            | BuildSubstrate::DreamMessages
            | BuildSubstrate::ProactivePingMessages => "message assembly (wraps system + history)",
        }
    }

    fn user_shape(self) -> &'static str {
        match self {
            BuildSubstrate::DialogueSystemPrompt
            | BuildSubstrate::DialogueSystemPromptWithOverrides
            | BuildSubstrate::ProactivePingSystemPrompt
            | BuildSubstrate::DreamSystemPrompt
            | BuildSubstrate::NarrativeSystemPrompt => "UserProfile + hidden-motive steer where shipped",
            BuildSubstrate::MemoryUpdatePrompt => "conversation text only",
            BuildSubstrate::ConsultantSystemPrompt => "thread + character context",
            BuildSubstrate::UserInWorldDerivePrompt | BuildSubstrate::UserInWorldDerivePromptWithChoices => {
                "Me-character fields + wizard choices"
            }
            BuildSubstrate::CharacterDerivePrompt => "character record",
            BuildSubstrate::WorldDerivePrompt => "world record",
            BuildSubstrate::SceneDescriptionPrompt
            | BuildSubstrate::SceneInventionPrompt
            | BuildSubstrate::AnimationPrompt
            | BuildSubstrate::ChapterFromImageSystemPrompt => "scene + cast + user as needed",
            BuildSubstrate::CorrectionNote => "verdict payload",
            BuildSubstrate::DialogueMessages
            | BuildSubstrate::DreamMessages
            | BuildSubstrate::ProactivePingMessages => "inherits upstream system prompt",
        }
    }

    fn parity(self) -> &'static str {
        match self {
            BuildSubstrate::DialogueSystemPrompt | BuildSubstrate::DialogueSystemPromptWithOverrides => {
                "solo ↔ group via overrides; ChatView ↔ GroupChatView app parity"
            }
            BuildSubstrate::ProactivePingSystemPrompt => "wraps solo dialogue builder + ping block",
            BuildSubstrate::DreamSystemPrompt | BuildSubstrate::NarrativeSystemPrompt => {
                "hidden-motive line ships; output craft still substrate-specific (CLAUDE)"
            }
            _ => "—",
        }
    }

    fn craft_anchor(self) -> &'static str {
        match self {
            BuildSubstrate::DialogueSystemPrompt | BuildSubstrate::DialogueSystemPromptWithOverrides => {
                "FUNDAMENTAL_SYSTEM_PREAMBLE + FORMAT_SECTION + fence stack"
            }
            BuildSubstrate::DreamSystemPrompt => "dream_preamble + dream_craft_block",
            BuildSubstrate::NarrativeSystemPrompt => "NARRATIVE_SYSTEM_PREAMBLE + POV block",
            BuildSubstrate::ProactivePingSystemPrompt => "solo stack + proactive_ping_block",
            BuildSubstrate::MemoryUpdatePrompt => "memory schema + conversation",
            BuildSubstrate::ConsultantSystemPrompt => "consultant preamble + thread pack",
            BuildSubstrate::CharacterDerivePrompt | BuildSubstrate::WorldDerivePrompt => "derivation templates",
            BuildSubstrate::UserInWorldDerivePrompt | BuildSubstrate::UserInWorldDerivePromptWithChoices => {
                "user derivation headers"
            }
            BuildSubstrate::SceneDescriptionPrompt
            | BuildSubstrate::SceneInventionPrompt
            | BuildSubstrate::AnimationPrompt
            | BuildSubstrate::ChapterFromImageSystemPrompt => "per-fn craft blocks in prompts.rs",
            BuildSubstrate::CorrectionNote => "conscience template",
            BuildSubstrate::DialogueMessages
            | BuildSubstrate::DreamMessages
            | BuildSubstrate::ProactivePingMessages => "orchestrator message assembly",
        }
    }

    fn enforcement(self) -> &'static str {
        match self {
            BuildSubstrate::DialogueSystemPrompt | BuildSubstrate::DialogueSystemPromptWithOverrides => {
                "const_contains invariants + craft registry tests + hidden_motive guards"
            }
            BuildSubstrate::DreamSystemPrompt | BuildSubstrate::NarrativeSystemPrompt => {
                "hidden_motive_guard_tests"
            }
            _ => "manual / bite-tests (add tests when substrate gains invariants)",
        }
    }
}

const PROMPTS_RS: &str = include_str!("prompts.rs");
const DERIVATION_RS: &str = include_str!("derivation.rs");
const CONSULTANT_RS: &str = include_str!("consultant.rs");
const CONSCIENCE_RS: &str = include_str!("conscience.rs");

/// Extract `build_*` function names after `pub fn ` … `(` (single-line or
/// split signature — handles `pub fn foo(\n` by scanning until `(`).
pub fn discover_build_fns_in_source(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for raw in src.split("pub fn ") {
        if raw.is_empty() {
            continue;
        }
        let rest = raw.trim_start();
        if !rest.starts_with("build_") {
            continue;
        }
        let end = rest.find('(').unwrap_or(0);
        if end == 0 {
            continue;
        }
        let head = rest[..end].trim();
        let name = head.split_whitespace().next().unwrap_or("");
        if name.starts_with("build_") {
            out.insert(name.to_string());
        }
    }
    out
}

pub fn discovered_all_ai_builders() -> BTreeSet<String> {
    let mut u = BTreeSet::new();
    u.extend(discover_build_fns_in_source(PROMPTS_RS));
    u.extend(discover_build_fns_in_source(DERIVATION_RS));
    u.extend(discover_build_fns_in_source(CONSULTANT_RS));
    u.extend(discover_build_fns_in_source(CONSCIENCE_RS));
    u
}

pub fn registered_rust_fns() -> BTreeSet<String> {
    BuildSubstrate::ALL
        .iter()
        .map(|s| s.rust_fn().to_string())
        .collect()
}

/// **v2 audit:** registry must exactly match discovered `pub fn build_*` in
/// the four scanned AI files.
pub fn audit_registry_matches_discovered() -> Result<(), String> {
    let disc = discovered_all_ai_builders();
    let reg = registered_rust_fns();
    let missing: Vec<_> = disc.difference(&reg).cloned().collect();
    let orphan: Vec<_> = reg.difference(&disc).cloned().collect();
    if !missing.is_empty() || !orphan.is_empty() {
        return Err(format!(
            "substrate atlas drift:\n  discovered but not registered: {missing:?}\n  registered but not in sources: {orphan:?}"
        ));
    }
    Ok(())
}

pub fn format_atlas_markdown() -> String {
    let mut s = String::new();
    s.push_str("<!-- Do not edit this file by hand. Regenerate after changing any `pub fn build_*` in `src-tauri/src/ai/` or the registry in `substrate_atlas.rs`. -->\n\n");
    s.push_str("# Substrate atlas (generated contract)\n\n");
    s.push_str("**Regenerate:** from repo root, with a local app DB available to worldcli (default path):\n\n");
    s.push_str("```bash\ncd src-tauri && cargo run --bin worldcli -- substrates --emit-markdown ../docs/SUBSTRATE_ATLAS.md\n```\n\n");
    s.push_str("Print to stdout instead: `cargo run --bin worldcli -- substrates` · JSON: add `--json` · CI drift check: add `--audit`.\n\n");
    s.push_str("Auto-generated field guide for `pub fn build_*` under `src-tauri/src/ai/`. ");
    s.push_str("Registry lives in `substrate_atlas::BuildSubstrate`; drift fails `substrate_atlas::audit_registry_matches_discovered`.\n\n");
    s.push_str("| Substrate | `rust fn` | file | family | voice / POV | user / payload | parity | craft anchor | enforcement |\n");
    s.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for e in BuildSubstrate::ALL {
        s.push_str(&format!(
            "| `{:?}` | `{}` | `{}` | {} | {} | {} | {} | {} | {} |\n",
            e,
            e.rust_fn(),
            e.source_file(),
            e.family(),
            e.voice_pov(),
            e.user_shape(),
            e.parity(),
            e.craft_anchor(),
            e.enforcement()
        ));
    }
    s
}

#[derive(Serialize)]
struct AtlasJsonRow {
    substrate: String,
    rust_fn: String,
    source_file: String,
    family: String,
    voice_pov: String,
    user_shape: String,
    parity: String,
    craft_anchor: String,
    enforcement: String,
}

pub fn format_atlas_json() -> Result<String, serde_json::Error> {
    let rows: Vec<AtlasJsonRow> = BuildSubstrate::ALL
        .iter()
        .map(|e| AtlasJsonRow {
            substrate: format!("{e:?}"),
            rust_fn: e.rust_fn().to_string(),
            source_file: e.source_file().to_string(),
            family: e.family().to_string(),
            voice_pov: e.voice_pov().to_string(),
            user_shape: e.user_shape().to_string(),
            parity: e.parity().to_string(),
            craft_anchor: e.craft_anchor().to_string(),
            enforcement: e.enforcement().to_string(),
        })
        .collect();
    serde_json::to_string_pretty(&rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_count_matches_all_variant_const() {
        assert_eq!(BuildSubstrate::ALL.len(), 19);
    }

    #[test]
    fn v2_audit_no_drift_between_registry_and_ai_sources() {
        audit_registry_matches_discovered().unwrap_or_else(|e| panic!("{e}"));
    }
}

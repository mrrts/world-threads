//! Substrate atlas — single registry of every `pub fn build_*` entry point
//! in the **atlas scan roots** (`src/ai/*.rs` + selected `src/commands/*.rs`
//! that ship prompt-shaped or context-pack builders), plus a discovery audit
//! so new public builders cannot land without updating the registry (v2).
//!
//! **v1:** `worldcli substrates` prints a markdown table (or `--json`).
//! **v2:** `cargo test` + `worldcli substrates --audit` compare this registry
//! to `include_str!` scans of those roots; any drift fails.

use serde::Serialize;
use std::collections::BTreeSet;

/// One row in the atlas — discriminant doubles as the stable registry key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuildSubstrate {
    AnimationPrompt,
    CanonizationInputs,
    ChapterFromImageSystemPrompt,
    CharacterDerivePrompt,
    ConsultantSystemPrompt,
    CorrectionNote,
    CrossThreadSnippet,
    DialogueMessages,
    DialogueSystemPrompt,
    DialogueSystemPromptWithOverrides,
    DreamMessages,
    DreamSystemPrompt,
    LocationDerivePrompt,
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
        BuildSubstrate::CanonizationInputs,
        BuildSubstrate::ChapterFromImageSystemPrompt,
        BuildSubstrate::CharacterDerivePrompt,
        BuildSubstrate::ConsultantSystemPrompt,
        BuildSubstrate::CorrectionNote,
        BuildSubstrate::CrossThreadSnippet,
        BuildSubstrate::DialogueMessages,
        BuildSubstrate::DialogueSystemPrompt,
        BuildSubstrate::DialogueSystemPromptWithOverrides,
        BuildSubstrate::DreamMessages,
        BuildSubstrate::DreamSystemPrompt,
        BuildSubstrate::LocationDerivePrompt,
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
            BuildSubstrate::CanonizationInputs => "build_canonization_inputs",
            BuildSubstrate::ChapterFromImageSystemPrompt => "build_chapter_from_image_system_prompt",
            BuildSubstrate::CharacterDerivePrompt => "build_character_prompt",
            BuildSubstrate::ConsultantSystemPrompt => "build_consultant_system_prompt",
            BuildSubstrate::CorrectionNote => "build_correction_note",
            BuildSubstrate::CrossThreadSnippet => "build_cross_thread_snippet",
            BuildSubstrate::DialogueMessages => "build_dialogue_messages",
            BuildSubstrate::DialogueSystemPrompt => "build_dialogue_system_prompt",
            BuildSubstrate::DialogueSystemPromptWithOverrides => "build_dialogue_system_prompt_with_overrides",
            BuildSubstrate::DreamMessages => "build_dream_messages",
            BuildSubstrate::DreamSystemPrompt => "build_dream_system_prompt",
            BuildSubstrate::LocationDerivePrompt => "build_location_prompt",
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
            | BuildSubstrate::LocationDerivePrompt
            | BuildSubstrate::UserInWorldDerivePrompt
            | BuildSubstrate::UserInWorldDerivePromptWithChoices => "derivation.rs",
            BuildSubstrate::ConsultantSystemPrompt => "consultant.rs",
            BuildSubstrate::CorrectionNote => "conscience.rs",
            BuildSubstrate::CanonizationInputs => "canon_cmds.rs",
            BuildSubstrate::CrossThreadSnippet => "chat_cmds.rs",
            _ => "prompts.rs",
        }
    }

    fn family(self) -> &'static str {
        match self {
            BuildSubstrate::CanonizationInputs => "canonization (classifier / propose pipeline)",
            BuildSubstrate::CrossThreadSnippet => "cross-thread continuity (dialogue context pack)",
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
            BuildSubstrate::CharacterDerivePrompt | BuildSubstrate::WorldDerivePrompt | BuildSubstrate::LocationDerivePrompt => {
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
            BuildSubstrate::CanonizationInputs => {
                "subject + context assembly (pre-classifier; not character voice)"
            }
            BuildSubstrate::CrossThreadSnippet => {
                "retrieved continuity text (injected upstream of character reply)"
            }
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
            | BuildSubstrate::LocationDerivePrompt
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
            BuildSubstrate::CanonizationInputs => {
                "source message + speaker label + context window + canon subjects"
            }
            BuildSubstrate::CrossThreadSnippet => {
                "other-thread message blocks; optional UserProfile for display name"
            }
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
            BuildSubstrate::LocationDerivePrompt => "location name + world context",
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
            BuildSubstrate::CanonizationInputs => {
                "worldcli classify-canonization ↔ Tauri propose path (`canon_internals`)"
            }
            BuildSubstrate::CrossThreadSnippet => {
                "same retrieval shape wherever cross-thread continuity is injected"
            }
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
            BuildSubstrate::CanonizationInputs => {
                "find_message + surrounding_messages + CanonizationSubject list"
            }
            BuildSubstrate::CrossThreadSnippet => {
                "list_cross_thread_recent_for_character + PICKING UP WHERE YOU LEFT OFF header"
            }
            BuildSubstrate::DialogueSystemPrompt | BuildSubstrate::DialogueSystemPromptWithOverrides => {
                "FUNDAMENTAL_SYSTEM_PREAMBLE + FORMAT_SECTION + fence stack"
            }
            BuildSubstrate::DreamSystemPrompt => "dream_preamble + dream_craft_block",
            BuildSubstrate::NarrativeSystemPrompt => "NARRATIVE_SYSTEM_PREAMBLE + POV block",
            BuildSubstrate::ProactivePingSystemPrompt => "solo stack + proactive_ping_block",
            BuildSubstrate::MemoryUpdatePrompt => "memory schema + conversation",
            BuildSubstrate::ConsultantSystemPrompt => "consultant preamble + thread pack",
            BuildSubstrate::CharacterDerivePrompt | BuildSubstrate::WorldDerivePrompt | BuildSubstrate::LocationDerivePrompt => "derivation templates",
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
            BuildSubstrate::CanonizationInputs => {
                "worldcli classify-canonization + manual path tests when classifier changes"
            }
            BuildSubstrate::CrossThreadSnippet => {
                "manual / retrieval bite-tests when cross-thread window changes"
            }
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

fn has_explicit_automation(enforcement: &str) -> bool {
    let e = enforcement.to_ascii_lowercase();
    e.contains("test")
        || e.contains("guard")
        || e.contains("audit")
        || e.contains("invariant")
}

fn is_manual_heavy(enforcement: &str) -> bool {
    enforcement.to_ascii_lowercase().starts_with("manual /")
}

const PROMPTS_RS: &str = include_str!("prompts.rs");
const DERIVATION_RS: &str = include_str!("derivation.rs");
const CONSULTANT_RS: &str = include_str!("consultant.rs");
const CONSCIENCE_RS: &str = include_str!("conscience.rs");
const CANON_CMDS_RS: &str = include_str!("../commands/canon_cmds.rs");
const CHAT_CMDS_RS: &str = include_str!("../commands/chat_cmds.rs");

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

pub fn discovered_atlas_scope_builders() -> BTreeSet<String> {
    let mut u = BTreeSet::new();
    u.extend(discover_build_fns_in_source(PROMPTS_RS));
    u.extend(discover_build_fns_in_source(DERIVATION_RS));
    u.extend(discover_build_fns_in_source(CONSULTANT_RS));
    u.extend(discover_build_fns_in_source(CONSCIENCE_RS));
    u.extend(discover_build_fns_in_source(CANON_CMDS_RS));
    u.extend(discover_build_fns_in_source(CHAT_CMDS_RS));
    u
}

pub fn registered_rust_fns() -> BTreeSet<String> {
    BuildSubstrate::ALL
        .iter()
        .map(|s| s.rust_fn().to_string())
        .collect()
}

/// **v2 audit:** registry must exactly match discovered `pub fn build_*` in
/// the atlas scan roots (`include_str!` sources above).
pub fn audit_registry_matches_discovered() -> Result<(), String> {
    let disc = discovered_atlas_scope_builders();
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
    s.push_str("<!-- Do not edit this file by hand. Regenerate after changing any `pub fn build_*` in the atlas scan roots (`src-tauri/src/ai/`, `canon_cmds.rs`, `chat_cmds.rs` for registered builders) or the registry in `substrate_atlas.rs`. -->\n\n");
    s.push_str("# Substrate Atlas: The Score\n\n");
    s.push_str("**Regenerate:** from repo root (no DB required for this command):\n\n");
    s.push_str("```bash\ncd src-tauri && cargo run --bin worldcli -- substrates --emit-markdown ../docs/SUBSTRATE_ATLAS.md\n```\n\n");
    s.push_str("Print to stdout instead: `cargo run --bin worldcli -- substrates` · JSON: add `--json` · CI drift check: add `--audit`.\n\n");
    s.push_str("This is the living score of every registered `pub fn build_*` in the atlas scan roots (see module doc on `substrate_atlas`). ");
    s.push_str("Each row captures role, voice, payload shape, parity edges, and enforcement so implementation and intent stay in tune.\n\n");
    s.push_str("Registry source of truth: `substrate_atlas::BuildSubstrate`.\n");
    s.push_str("Drift gate: `substrate_atlas::audit_registry_matches_discovered`.\n\n");
    let total = BuildSubstrate::ALL.len();
    let parity_sensitive = BuildSubstrate::ALL
        .iter()
        .filter(|e| e.parity() != "—")
        .count();
    let automated = BuildSubstrate::ALL
        .iter()
        .filter(|e| has_explicit_automation(e.enforcement()))
        .count();
    let manual_heavy = BuildSubstrate::ALL
        .iter()
        .filter(|e| is_manual_heavy(e.enforcement()))
        .count();
    s.push_str("## Chorus\n\n");
    s.push_str(&format!(
        "- **Total registered substrates:** {} (`BuildSubstrate::ALL`)\n",
        total
    ));
    s.push_str(
        "- **Scan roots:** `src-tauri/src/ai/*.rs` + selected `src-tauri/src/commands/*.rs`\n",
    );
    s.push_str(&format!(
        "- **Parity-sensitive substrates:** {} (explicit cross-surface obligations)\n",
        parity_sensitive
    ));
    s.push_str(&format!(
        "- **Explicit automation in enforcement notes:** {} / {}\n",
        automated, total
    ));
    s.push_str(&format!(
        "- **Manual-heavy enforcement notes:** {} / {} (best candidates for stronger tests)\n\n",
        manual_heavy, total
    ));
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

/// Backstage consultant lens: compact, documentary snapshot of the atlas.
///
/// Plain-language first, technical identifiers second. This is meant to help
/// Backstage reason about risk and parity without dumping the full table.
pub fn format_backstage_lens() -> String {
    format_backstage_lens_with_focus(&[])
}

pub fn format_backstage_lens_with_focus(focus: &[BuildSubstrate]) -> String {
    let total = BuildSubstrate::ALL.len();
    let parity_rows: Vec<BuildSubstrate> = BuildSubstrate::ALL
        .iter()
        .copied()
        .filter(|e| e.parity() != "—")
        .collect();
    let manual_rows: Vec<BuildSubstrate> = BuildSubstrate::ALL
        .iter()
        .copied()
        .filter(|e| is_manual_heavy(e.enforcement()))
        .collect();
    let automated_count = BuildSubstrate::ALL
        .iter()
        .filter(|e| has_explicit_automation(e.enforcement()))
        .count();

    let mut focus_ranked: Vec<BuildSubstrate> = focus
        .iter()
        .copied()
        .filter(|e| parity_rows.contains(e))
        .collect();
    for row in &parity_rows {
        if !focus_ranked.contains(row) {
            focus_ranked.push(*row);
        }
    }

    let mut s = String::new();
    s.push_str("═══════════════════════════════════════════════\n");
    s.push_str("ATLAS LENS (documentary, backstage-only)\n\n");
    s.push_str("Use this as a quiet craft map. Default to plain-language guidance; only surface technical labels if ");
    s.push_str("the user explicitly asks for internals.\n\n");
    s.push_str("Plain read:\n");
    s.push_str(&format!(
        "- Registered substrates: {total}\n- Cross-surface parity seams: {}\n- Explicit automation markers in enforcement: {} / {}\n- Manual-heavy enforcement notes: {} / {}\n\n",
        parity_rows.len(),
        automated_count,
        total,
        manual_rows.len(),
        total
    ));

    s.push_str("Current parity hotspots (plain-language first):\n");
    for e in focus_ranked.iter().take(5) {
        s.push_str(&format!(
            "- {:?}: {}\n  Guardrail shape: {}\n",
            e,
            e.parity(),
            e.enforcement()
        ));
    }
    s.push('\n');

    s.push_str("Technical lookup (only if asked):\n");
    for e in focus_ranked.iter().take(5) {
        s.push_str(&format!(
            "- {:?} => `{}` in `{}`\n",
            e,
            e.rust_fn(),
            e.source_file()
        ));
    }
    s.push_str("═══════════════════════════════════════════════");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_count_matches_all_variant_const() {
        assert_eq!(BuildSubstrate::ALL.len(), 22);
    }

    #[test]
    fn v2_audit_no_drift_between_registry_and_ai_sources() {
        audit_registry_matches_discovered().unwrap_or_else(|e| panic!("{e}"));
    }

    #[test]
    fn backstage_lens_respects_focus_ranking_order() {
        let lens = format_backstage_lens_with_focus(&[
            BuildSubstrate::CrossThreadSnippet,
            BuildSubstrate::DialogueSystemPromptWithOverrides,
        ]);
        let cross_idx = lens
            .find("CrossThreadSnippet")
            .expect("expected CrossThreadSnippet in lens");
        let dialogue_idx = lens
            .find("DialogueSystemPromptWithOverrides")
            .expect("expected DialogueSystemPromptWithOverrides in lens");
        assert!(
            cross_idx < dialogue_idx,
            "focused substrate should be ranked before non-focused parity rows"
        );
    }

    #[test]
    fn backstage_lens_marks_technical_mode_as_opt_in() {
        let lens = format_backstage_lens();
        assert!(
            lens.contains("only surface technical labels if the user explicitly asks for internals"),
            "lens should enforce technical opt-in wording"
        );
        assert!(
            lens.contains("Technical lookup (only if asked):"),
            "lens should clearly scope technical section to on-request use"
        );
    }
}

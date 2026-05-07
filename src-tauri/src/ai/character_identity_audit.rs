//! Offline character-identity audit scaffolding.
//!
//! This module is intentionally separate from prompt assembly and
//! from the live `has_read_empiricon` runtime switch. It exists so the
//! proposed encoder/decoder can be rehearsed and audited without
//! affecting character prompts.

use crate::ai::character_identity_payload::{
    character_identity_is_lossless, decode_character_identity_payload,
    render_character_identity_payload, split_character_identity,
};
use crate::db::queries::Character;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditVerdict {
    NotRun,
    Pass,
    SoftFail,
    HardFail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterIdentityAuditResult {
    pub character_id: String,
    pub display_name: String,
    pub verdict: AuditVerdict,
    pub preserved: Vec<String>,
    pub missing: Vec<String>,
    pub notes: Vec<String>,
}

pub fn audit_character_identity(character: &Character) -> CharacterIdentityAuditResult {
    match render_character_identity_payload(character) {
        Some(payload) => audit_character_identity_payload(character, &payload),
        None => CharacterIdentityAuditResult {
            character_id: character.character_id.clone(),
            display_name: character.display_name.clone(),
            verdict: AuditVerdict::HardFail,
            preserved: Vec::new(),
            missing: vec!["payload_render".to_string()],
            notes: vec!["failed to render payload".to_string()],
        },
    }
}

pub fn audit_character_identity_payload(
    character: &Character,
    payload: &str,
) -> CharacterIdentityAuditResult {
    let parsed = match decode_character_identity_payload(payload) {
        Ok(parsed) => parsed,
        Err(err) => {
            return CharacterIdentityAuditResult {
                character_id: character.character_id.clone(),
                display_name: character.display_name.clone(),
                verdict: AuditVerdict::HardFail,
                preserved: Vec::new(),
                missing: vec!["decode".to_string()],
                notes: vec![err.to_string()],
            }
        }
    };

    let expected = split_character_identity(character);
    let mut preserved = Vec::new();
    let mut missing = Vec::new();

    compare_opt_string(
        "role_frame",
        &parsed.buckets.role_frame,
        &expected.role_frame,
        &mut preserved,
        &mut missing,
    );
    compare_opt_string(
        "relation_anchor",
        &parsed.buckets.relation_anchor,
        &expected.relation_anchor,
        &mut preserved,
        &mut missing,
    );
    compare_vec(
        "voice_lift",
        &parsed.buckets.voice_lift,
        &expected.voice_lift,
        &mut preserved,
        &mut missing,
    );
    compare_vec(
        "embodied_marker",
        &parsed.buckets.embodied_marker,
        &expected.embodied_marker,
        &mut preserved,
        &mut missing,
    );
    compare_vec(
        "attachment_node",
        &parsed.buckets.attachment_node,
        &expected.attachment_node,
        &mut preserved,
        &mut missing,
    );
    compare_opt_string(
        "wound_longing",
        &parsed.buckets.wound_longing,
        &expected.wound_longing,
        &mut preserved,
        &mut missing,
    );
    compare_vec(
        "refusal_shape",
        &parsed.buckets.refusal_shape,
        &expected.refusal_shape,
        &mut preserved,
        &mut missing,
    );
    compare_opt_string(
        "moral_theological_position",
        &parsed.buckets.moral_theological_position,
        &expected.moral_theological_position,
        &mut preserved,
        &mut missing,
    );
    compare_vec(
        "fact_atom",
        &parsed.buckets.fact_atom,
        &expected.fact_atom,
        &mut preserved,
        &mut missing,
    );

    let lossless = character_identity_is_lossless(character, &parsed);
    let mut notes = Vec::new();
    if let Some(g) = classify_gravity_pressure(character) {
        notes.push(format!("gravity_line: {g}"));
    }
    if !lossless {
        notes.push(
            "source snapshot or bucket reconstruction did not match expected character shape"
                .to_string(),
        );
    }

    let verdict = if missing.is_empty() && lossless {
        AuditVerdict::Pass
    } else if preserved.is_empty() {
        AuditVerdict::HardFail
    } else {
        AuditVerdict::SoftFail
    };

    CharacterIdentityAuditResult {
        character_id: character.character_id.clone(),
        display_name: character.display_name.clone(),
        verdict,
        preserved,
        missing,
        notes,
    }
}

pub fn classify_gravity_pressure(character: &Character) -> Option<String> {
    crate::ai::character_identity_payload::classify_gravity_pressure(character)
}

fn compare_opt_string(
    label: &str,
    actual: &Option<String>,
    expected: &Option<String>,
    preserved: &mut Vec<String>,
    missing: &mut Vec<String>,
) {
    match (actual, expected) {
        (Some(a), Some(e)) if a == e => preserved.push(label.to_string()),
        (None, None) => preserved.push(label.to_string()),
        _ => missing.push(label.to_string()),
    }
}

fn compare_vec(
    label: &str,
    actual: &[String],
    expected: &[String],
    preserved: &mut Vec<String>,
    missing: &mut Vec<String>,
) {
    if actual == expected {
        preserved.push(label.to_string());
    } else {
        missing.push(label.to_string());
    }
}

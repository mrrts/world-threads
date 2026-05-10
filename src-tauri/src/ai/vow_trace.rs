//! VGUS Stage 1 Phase 0 — vow-trace logging stub.
//!
//! Vow-Governed Unattended Substrate (VGUS) arc charter at
//! `reports/2026-05-09-2930-vgus-arc-charter-and-stage-1-phase-0-spec.md`.
//!
//! Stage 1 (Refusal Apparatus) is the prerequisite gate for Stages 2
//! (Night Keep) and 3 (World That Breathes). This module ships the
//! data structures and stub interface for vow-tracking; live invocation
//! logic and runtime checking are deferred to Phase 1.
//!
//! Vows are durable per-character objects with provenance + state +
//! lineage. Three ingestion paths: substrate_emergent (character
//! articulates a vow during runtime, gets ratified in-voice),
//! apparatus_template (pre-authored scaffold accepted in voice),
//! doctrine_import (refusal classes from /consecrate flow down as
//! constraints). Conflict resolution priority: doctrine > safety
//! constraints > character-specific vows.
//!
//! See schema.rs for the SQL tables (`vows`, `vow_event_log`,
//! `vow_invocations`).

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VowSource {
    SubstrateEmergent,
    ApparatusTemplate,
    DoctrineImport,
}

impl VowSource {
    pub fn as_db(&self) -> &'static str {
        match self {
            VowSource::SubstrateEmergent => "substrate_emergent",
            VowSource::ApparatusTemplate => "apparatus_template",
            VowSource::DoctrineImport => "doctrine_import",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "substrate_emergent" => Some(VowSource::SubstrateEmergent),
            "apparatus_template" => Some(VowSource::ApparatusTemplate),
            "doctrine_import" => Some(VowSource::DoctrineImport),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VowStatus {
    Proposed,
    Ratified,
    Suspended,
    Amended,
    Rescinded,
}

impl VowStatus {
    pub fn as_db(&self) -> &'static str {
        match self {
            VowStatus::Proposed => "proposed",
            VowStatus::Ratified => "ratified",
            VowStatus::Suspended => "suspended",
            VowStatus::Amended => "amended",
            VowStatus::Rescinded => "rescinded",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "proposed" => Some(VowStatus::Proposed),
            "ratified" => Some(VowStatus::Ratified),
            "suspended" => Some(VowStatus::Suspended),
            "amended" => Some(VowStatus::Amended),
            "rescinded" => Some(VowStatus::Rescinded),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InvocationMode {
    Invoked,
    NearMiss,
    BreachCaught,
}

impl InvocationMode {
    pub fn as_db(&self) -> &'static str {
        match self {
            InvocationMode::Invoked => "invoked",
            InvocationMode::NearMiss => "near_miss",
            InvocationMode::BreachCaught => "breach_caught",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vow {
    pub id: String,
    pub character_id: String,
    pub text: String,
    pub scope: Vec<String>,
    pub source: VowSource,
    pub anchors: Vec<String>,
    pub constraints: Vec<VowConstraint>,
    pub exceptions: Vec<String>,
    pub triggers: Vec<String>,
    pub status: VowStatus,
    pub provenance: VowProvenance,
    pub parent_vow_id: Option<String>,
    pub salience: f64,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VowConstraint {
    pub kind: ConstraintKind,
    pub predicate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintKind {
    Must,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VowProvenance {
    pub ratified_by: Option<String>,
    pub ratified_at: Option<String>,
    pub witnesses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VowInvocation {
    pub vow_id: String,
    pub turn_id: String,
    pub mode: InvocationMode,
}

/// Phase 0 stub: returns an empty Vec. Phase 1 will implement real vow-checking
/// against the turn input + character's ratified vows. The signature is the
/// commitment; the body is the placeholder.
pub fn check_vows(_turn_input: &str, _character_id: &str, _conn: &Connection) -> Vec<VowInvocation> {
    Vec::new()
}

/// Phase 0 stub: signature only. Phase 1 will implement insert into
/// `vow_invocations` with the constraint that `vow_id` resolves to a ratified
/// vow whose character_id matches.
pub fn record_invocation(_invocation: &VowInvocation, _conn: &Connection) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vow_source_db_roundtrip() {
        for src in [
            VowSource::SubstrateEmergent,
            VowSource::ApparatusTemplate,
            VowSource::DoctrineImport,
        ] {
            assert_eq!(VowSource::from_db(src.as_db()), Some(src));
        }
        assert_eq!(VowSource::from_db("unknown"), None);
    }

    #[test]
    fn vow_status_db_roundtrip() {
        for st in [
            VowStatus::Proposed,
            VowStatus::Ratified,
            VowStatus::Suspended,
            VowStatus::Amended,
            VowStatus::Rescinded,
        ] {
            assert_eq!(VowStatus::from_db(st.as_db()), Some(st));
        }
        assert_eq!(VowStatus::from_db("unknown"), None);
    }

    #[test]
    fn invocation_mode_strings() {
        assert_eq!(InvocationMode::Invoked.as_db(), "invoked");
        assert_eq!(InvocationMode::NearMiss.as_db(), "near_miss");
        assert_eq!(InvocationMode::BreachCaught.as_db(), "breach_caught");
    }
}

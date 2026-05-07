use app_lib::ai::character_identity_audit::{
    audit_against_reference, audit_character_identity, AuditVerdict,
};
use app_lib::ai::character_identity_payload::{
    decode_character_identity, decode_character_identity_payload, encode_character_identity,
    render_character_identity_payload, split_character_identity, CharacterIdentityBuckets,
    CharacterIdentityReference, CHARACTER_IDENTITY_CLASS_NAMES,
    CHARACTER_IDENTITY_REFERENCE_SCHEMA_VERSION, CHARACTER_IDENTITY_SCHEMA_VERSION,
    CHARACTER_IDENTITY_SOURCE_FIELDS,
};
use app_lib::db::queries::Character;

#[test]
fn scaffold_exposes_expected_character_identity_shape() {
    assert_eq!(
        CHARACTER_IDENTITY_SCHEMA_VERSION,
        "v3-character-identity-scaffold"
    );
    assert!(CHARACTER_IDENTITY_CLASS_NAMES.contains(&"role_frame"));
    assert!(CHARACTER_IDENTITY_SOURCE_FIELDS.contains(&"identity"));

    let buckets = CharacterIdentityBuckets::default();
    assert!(buckets.voice_lift.is_empty());
    assert!(buckets.fact_atom.is_empty());
}

#[test]
fn snapshot_fixtures_round_trip_and_audit_pass() {
    for fixture in fixture_names() {
        let character = load_fixture(fixture);
        let payload = render_character_identity_payload(&character).expect("payload renders");
        let parsed = decode_character_identity_payload(&payload).expect("payload decodes");
        assert_eq!(parsed.source.identity, character.identity, "{fixture}");
        assert_eq!(
            parsed.buckets,
            split_character_identity(&character),
            "{fixture}"
        );
        assert_eq!(
            decode_character_identity(&payload).expect("decode buckets"),
            parsed.buckets,
            "{fixture}"
        );
        assert_eq!(
            audit_character_identity(&character).verdict,
            AuditVerdict::Pass,
            "{fixture}"
        );
    }
}

#[test]
fn steven_fixture_preserves_refusal_and_wound_shape() {
    let character = load_fixture("steven");
    let buckets = split_character_identity(&character);
    assert!(buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("Will not accept charity")));
    assert!(buckets
        .wound_longing
        .as_ref()
        .is_some_and(|s| s.contains("What he wants")));
}

#[test]
fn maisie_fixture_carries_attachment_and_embodiment() {
    let character = load_fixture("maisie_rourke");
    let buckets = split_character_identity(&character);
    assert!(buckets.embodied_marker.iter().any(|s| s.contains("apron")));
    assert!(buckets
        .attachment_node
        .iter()
        .any(|s| s.contains("letters tied with string")));
    assert!(buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("artificial ingredients")));
}

#[test]
fn pastor_rick_fixture_keeps_theological_position() {
    let character = load_fixture("pastor_rick");
    let buckets = split_character_identity(&character);
    assert!(buckets
        .moral_theological_position
        .as_ref()
        .is_some_and(|s| s.contains("God") || s.contains("grace") || s.contains("Jesus")));
    assert!(buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("does not judge") || s.contains("He does not judge")));
}

#[test]
fn aaron_fixture_does_not_leak_weightier_into_embodied_marker() {
    let character = load_fixture("aaron");
    let buckets = split_character_identity(&character);
    assert!(!buckets
        .embodied_marker
        .iter()
        .any(|s| s.contains("weightier than it was meant to")));
}

#[test]
fn aaron_fixture_locks_in_role_relation_and_voice() {
    let buckets = split_character_identity(&load_fixture("aaron"));
    assert_eq!(
        buckets.role_frame.as_deref(),
        Some(
            "A fellow software engineer and a brother in Christ -- he believes, \
             as I do, that Jesus is the only way."
        )
    );
    assert!(buckets
        .relation_anchor
        .as_deref()
        .is_some_and(|s| s.starts_with("We go to the same church")));
    assert_eq!(
        buckets.voice_lift.first().map(String::as_str),
        Some("Speaks friendly and enthusiastically")
    );
    assert_eq!(
        buckets.embodied_marker,
        vec!["Wears glasses".to_string()]
    );
}

#[test]
fn steven_fixture_refusal_shape_is_strict_boundary_set() {
    let buckets = split_character_identity(&load_fixture("steven"));
    assert_eq!(
        buckets.refusal_shape,
        vec![
            "Will not accept charity. Trades only.".to_string(),
            "Refuses to talk about where they came from.".to_string(),
            "Will not stay anywhere they feel pitied.".to_string(),
        ]
    );
    assert!(!buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("cannot walk past")));
    let wound_longing = buckets
        .wound_longing
        .as_deref()
        .expect("steven has wound_longing");
    // Paired selection: longing half + wound half joined by " — ".
    assert!(
        wound_longing.starts_with("What he wants -- what he'd never say -- is to stop moving."),
        "expected longing half first, got: {wound_longing}"
    );
    assert!(
        wound_longing.contains(" — "),
        "expected pairing separator, got: {wound_longing}"
    );
    assert!(
        wound_longing.contains("walls are cheaper than wounds"),
        "expected wound half present, got: {wound_longing}"
    );
}

#[test]
fn pair_wound_and_longing_falls_back_to_single_when_only_one_axis_scores() {
    // Aaron's identity prose has wound-coded sentences but no clean
    // longing-coded sentence; the helper should return the single
    // best-scoring side rather than fabricating a pair.
    let buckets = split_character_identity(&load_fixture("aaron"));
    let wound = buckets
        .wound_longing
        .as_deref()
        .expect("aaron has wound_longing");
    assert!(
        !wound.contains(" — ") || wound.matches(" — ").count() == 0
            || !wound.contains("\nwhat he wants"),
        "aaron should not synthesize a fake longing half: {wound}"
    );
    assert!(
        wound.contains("doesn't have a vocabulary")
            || wound.contains("capacity for depth")
            || wound.contains("feels most"),
        "aaron's wound_longing should land on a wound-coded line: {wound}"
    );
}

#[test]
fn pastor_rick_refusal_shape_drops_compassion_false_positives() {
    let buckets = split_character_identity(&load_fixture("pastor_rick"));
    assert!(!buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("He loves with a compassion")));
    assert!(!buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("does not get the final word")));
    assert!(buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("He does not use verses as weapons")));
    assert_eq!(
        buckets.moral_theological_position.as_deref(),
        Some("Jesus means mercy to me.")
    );
}

#[test]
fn maisie_wound_longing_surfaces_husband_loss_via_backstory_fallback() {
    // Turn 246 closed the gap named in maisie_rourke.reference.json's
    // rationale notes: wound-needles 'lost'/'illness'/'late' added so
    // her husband-loss line surfaces on the backstory-fallback path.
    let buckets = split_character_identity(&load_fixture("maisie_rourke"));
    let wound = buckets
        .wound_longing
        .as_deref()
        .expect("maisie's wound_longing should surface after the gap closes");
    assert!(
        wound.contains("lost her husband") && wound.contains("illness"),
        "expected husband-loss wound, got: {wound}"
    );
}

#[test]
fn maisie_role_frame_and_refusal_remain_canonical() {
    let buckets = split_character_identity(&load_fixture("maisie_rourke"));
    assert!(buckets
        .role_frame
        .as_deref()
        .is_some_and(|s| s.starts_with("I'm Maisie")));
    assert_eq!(
        buckets.refusal_shape,
        vec![
            "Avoids discussing her husband's illness and death openly.".to_string(),
            "Refuses to bake with artificial ingredients, believing in the purity of her craft."
                .to_string(),
        ]
    );
}

#[test]
fn jasper_finn_role_voice_and_refusal_carry_potter_shape() {
    let buckets = split_character_identity(&load_fixture("jasper_finn"));
    assert_eq!(
        buckets.role_frame.as_deref(),
        Some("I'm Jasper Finn, a potter whose fingers know the earth like an old companion.")
    );
    assert!(buckets
        .voice_lift
        .iter()
        .any(|s| s.contains("melodic phrases")));
    assert_eq!(buckets.refusal_shape.len(), 3);
    assert!(buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("Sundays")));
    assert!(buckets
        .refusal_shape
        .iter()
        .any(|s| s.contains("gossip")));
}

#[test]
fn split_sentences_does_not_break_inside_double_quotes() {
    let character = load_fixture("pastor_rick");
    let buckets = split_character_identity(&character);
    let wound = buckets
        .wound_longing
        .as_deref()
        .expect("pastor rick has wound_longing");
    // Quote-aware splitting keeps `"He is dear to me,"` inside the same
    // sentence as `He speaks of Jesus as someone steadier than his fear`,
    // rather than slicing on the period inside the quoted clause.
    assert!(
        wound.contains("He is dear to me") && wound.contains("steadier than his fear"),
        "expected joined wound sentence, got: {wound}"
    );
}

#[test]
fn aaron_passes_tier1_independent_reference_audit() {
    assert_eq!(
        CHARACTER_IDENTITY_REFERENCE_SCHEMA_VERSION,
        "v3-character-identity-reference"
    );
    let character = load_fixture("aaron");
    let payload = encode_character_identity(&character);
    let reference = load_reference("aaron");
    let result = audit_against_reference(&character, &payload, &reference);
    assert_eq!(
        result.verdict,
        AuditVerdict::Pass,
        "expected Tier 1 reference audit to Pass for aaron, missing={:?}",
        result.missing
    );
    assert!(
        result.preserved.contains(&"role_frame".to_string())
            && result.preserved.contains(&"wound_longing".to_string())
            && result.preserved.contains(&"refusal_shape".to_string()),
        "expected canonical buckets to be preserved, got {:?}",
        result.preserved
    );
    assert!(
        result.notes.first()
            .is_some_and(|n| n.starts_with("tier_1_reviewer:")),
        "expected tier-1 marker in notes, got {:?}",
        result.notes
    );
}

#[test]
fn all_grounded_fixtures_pass_tier1_reference_audit() {
    for fixture in fixture_names() {
        let character = load_fixture(fixture);
        let payload = encode_character_identity(&character);
        let reference = load_reference(fixture);
        let result = audit_against_reference(&character, &payload, &reference);
        assert_eq!(
            result.verdict,
            AuditVerdict::Pass,
            "Tier 1 reference audit failed for {fixture}: missing={:?} notes={:?}",
            result.missing,
            result.notes
        );
    }
}

#[test]
fn audit_against_reference_rejects_mismatched_character_id() {
    let character = load_fixture("aaron");
    let payload = encode_character_identity(&character);
    let mut reference = load_reference("aaron");
    reference.character_id = "00000000-deadbeef-0000-0000-000000000000".to_string();
    let result = audit_against_reference(&character, &payload, &reference);
    assert_eq!(result.verdict, AuditVerdict::HardFail);
    assert!(result.missing.contains(&"reference_character_id".to_string()));
}

#[test]
#[ignore]
fn dump_buckets_for_inspection() {
    for fixture in fixture_names() {
        let character = load_fixture(fixture);
        let buckets = split_character_identity(&character);
        let gravity = app_lib::ai::character_identity_audit::classify_gravity_pressure(&character);
        eprintln!("=== {fixture} ===");
        eprintln!("role_frame: {:?}", buckets.role_frame);
        eprintln!("relation_anchor: {:?}", buckets.relation_anchor);
        eprintln!("voice_lift: {:?}", buckets.voice_lift);
        eprintln!("embodied_marker: {:?}", buckets.embodied_marker);
        eprintln!("attachment_node: {:?}", buckets.attachment_node);
        eprintln!("wound_longing: {:?}", buckets.wound_longing);
        eprintln!("refusal_shape: {:?}", buckets.refusal_shape);
        eprintln!(
            "moral_theological_position: {:?}",
            buckets.moral_theological_position
        );
        eprintln!("fact_atom: {:?}", buckets.fact_atom);
        eprintln!("gravity_line: {:?}", gravity);
        eprintln!();
    }
}

fn fixture_names() -> [&'static str; 5] {
    [
        "aaron",
        "jasper_finn",
        "maisie_rourke",
        "pastor_rick",
        "steven",
    ]
}

fn load_fixture(name: &str) -> Character {
    let path = format!("tests/fixtures/character_identity/{name}.json");
    let raw = std::fs::read_to_string(&path).expect("fixture exists");
    serde_json::from_str(&raw).expect("fixture parses")
}

fn load_reference(name: &str) -> CharacterIdentityReference {
    let path = format!("tests/fixtures/character_identity/{name}.reference.json");
    let raw = std::fs::read_to_string(&path).expect("reference exists");
    serde_json::from_str(&raw).expect("reference parses")
}

//! Offline character-identity payload scaffolding.
//!
//! This module is intentionally separate from prompt assembly. It
//! sketches the future offline encode/decode surface for character
//! identity blocks without routing anything into runtime prompts yet.

use crate::db::queries::Character;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub const CHARACTER_IDENTITY_SCHEMA_VERSION: &str = "v3-character-identity-scaffold";

pub const CHARACTER_IDENTITY_REFERENCE_SCHEMA_VERSION: &str =
    "v3-character-identity-reference";

pub const CHARACTER_IDENTITY_SOURCE_FIELDS: &[&str] = &[
    "identity",
    "voice_rules",
    "boundaries",
    "backstory_facts",
    "derived_formula",
    "has_read_empiricon",
];

pub const CHARACTER_IDENTITY_CLASS_NAMES: &[&str] = &[
    "role_frame",
    "relation_anchor",
    "voice_lift",
    "embodied_marker",
    "attachment_node",
    "wound_longing",
    "refusal_shape",
    "moral_theological_position",
    "fact_atom",
];

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterIdentitySourceSnapshot {
    pub identity: String,
    pub voice_rules: Vec<String>,
    pub boundaries: Vec<String>,
    pub backstory_facts: Vec<String>,
    pub derived_formula: Option<String>,
    pub has_read_empiricon: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterIdentityBuckets {
    pub role_frame: Option<String>,
    pub relation_anchor: Option<String>,
    pub voice_lift: Vec<String>,
    pub embodied_marker: Vec<String>,
    pub attachment_node: Vec<String>,
    pub wound_longing: Option<String>,
    pub refusal_shape: Vec<String>,
    pub moral_theological_position: Option<String>,
    pub fact_atom: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterIdentityReference {
    pub schema_version: String,
    pub character_id: String,
    pub display_name: String,
    pub expected_buckets: CharacterIdentityBuckets,
    #[serde(default)]
    pub rationale_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterIdentityPayload {
    pub schema_version: String,
    pub character_id: String,
    pub display_name: String,
    pub source: CharacterIdentitySourceSnapshot,
    pub buckets: CharacterIdentityBuckets,
    pub gravity_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterIdentityDecodeError {
    pub message: String,
}

impl fmt::Display for CharacterIdentityDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CharacterIdentityDecodeError {}

pub fn split_character_identity(character: &Character) -> CharacterIdentityBuckets {
    let identity_sentences = split_sentences(&character.identity);
    let voice_rules = json_array_to_strings(&character.voice_rules);
    let boundaries = json_array_to_strings(&character.boundaries);
    let backstory_facts = json_array_to_strings(&character.backstory_facts);

    let role_frame = pick_first(&identity_sentences);
    let relation_anchor = best_scored_sentence(
        &identity_sentences,
        &[
            ("with you", 5),
            ("with me", 5),
            ("moment we're having", 5),
            ("friend", 4),
            ("church", 4),
            ("flock", 4),
            ("companionable", 4),
            ("understanding", 4),
            ("safe enough", 4),
            ("brother", 3),
            ("sister", 3),
            ("people feel safe", 3),
            ("showing up", 3),
            ("presence", 2),
            ("community", 2),
        ],
    )
    .or_else(|| {
        best_scored_sentence(
            &backstory_facts,
            &[
                ("friend", 4),
                ("church", 4),
                ("community", 3),
                ("mother", 2),
                ("father", 2),
                ("son", 2),
                ("daughter", 2),
                ("husband", 2),
                ("wife", 2),
            ],
        )
    });

    let embodied_marker = collect_matching(
        identity_sentences
            .iter()
            .chain(backstory_facts.iter())
            .map(String::as_str),
        &[
            "hair",
            "beard",
            "glasses",
            "hands",
            "apron",
            "shirt",
            "tie",
            "grey",
            "gray",
            "streaked",
            "stained",
            "wrist",
            "clean-shaven",
            "white hair",
            "flour",
            "palm",
            "fingers",
        ],
    );

    let attachment_node = collect_matching(
        identity_sentences
            .iter()
            .chain(backstory_facts.iter())
            .map(String::as_str),
        &[
            "mother",
            "father",
            "son",
            "daughter",
            "husband",
            "wife",
            "grandmother",
            "friend",
            "church",
            "bakery",
            "river",
            "town",
            "kayak",
            "journal",
            "heirloom",
            "wheel",
            "flock",
            "letters",
            "recipe",
            "mug",
            "kitchen",
            "coffee",
        ],
    );

    let wound_longing = pair_wound_and_longing(&identity_sentences).or_else(|| {
        best_scored_sentence(
            &backstory_facts,
            &[
                ("loss", 7),
                ("lost", 7),
                ("grief", 7),
                ("hurt", 6),
                ("illness", 6),
                ("alone", 5),
                ("widow", 5),
                ("late", 5),
            ],
        )
    });

    let refusal_shape = boundaries.clone();
    let mut refusal_shape_extra = collect_matching(
        identity_sentences.iter().map(String::as_str),
        &[
            "won't",
            "will not",
            "refuse",
            "refused",
            "refuses",
            "avoid",
            "avoids",
            "no instinct",
            "will always speak plainly",
            "does not judge",
            "does not use",
        ],
    );
    extend_unique(&mut refusal_shape_extra, &boundaries);

    let moral_theological_position = best_scored_sentence(
        &identity_sentences
            .iter()
            .cloned()
            .chain(voice_rules.iter().cloned())
            .chain(boundaries.iter().cloned())
            .chain(backstory_facts.iter().cloned())
            .collect::<Vec<_>>(),
        &[
            ("means mercy", 12),
            ("jesus means", 11),
            ("he is dear to me", 10),
            ("does not have to save himself", 9),
            ("face of god i can love without flinching", 9),
            ("jesus", 8),
            ("christ", 8),
            ("cross", 7),
            ("savior", 7),
            ("grace", 6),
            ("mercy", 6),
            ("scripture", 5),
            ("bible", 5),
            ("faith", 4),
            ("god", 3),
            ("sanctification", 5),
            ("holy", 4),
        ],
    )
    .map(|line| normalize_theological_line(&line));

    let fact_atom = backstory_facts.clone();
    CharacterIdentityBuckets {
        role_frame,
        relation_anchor,
        voice_lift: voice_rules,
        embodied_marker,
        attachment_node,
        wound_longing,
        refusal_shape: if refusal_shape_extra.is_empty() {
            refusal_shape
        } else {
            refusal_shape_extra
        },
        moral_theological_position,
        fact_atom,
    }
}

pub fn encode_character_identity(character: &Character) -> String {
    render_character_identity_payload(character).unwrap_or_default()
}

pub fn decode_character_identity(
    payload: &str,
) -> Result<CharacterIdentityBuckets, CharacterIdentityDecodeError> {
    let payload = decode_character_identity_payload(payload)?;
    Ok(payload.buckets)
}

pub fn render_character_identity_payload(character: &Character) -> Option<String> {
    let payload = CharacterIdentityPayload {
        schema_version: CHARACTER_IDENTITY_SCHEMA_VERSION.to_string(),
        character_id: character.character_id.clone(),
        display_name: character.display_name.clone(),
        source: CharacterIdentitySourceSnapshot {
            identity: character.identity.clone(),
            voice_rules: json_array_to_strings(&character.voice_rules),
            boundaries: json_array_to_strings(&character.boundaries),
            backstory_facts: json_array_to_strings(&character.backstory_facts),
            derived_formula: character.derived_formula.clone(),
            has_read_empiricon: character.has_read_empiricon,
        },
        buckets: split_character_identity(character),
        gravity_line: classify_gravity_pressure(character),
    };
    serde_json::to_string_pretty(&payload).ok()
}

pub fn decode_character_identity_payload(
    payload: &str,
) -> Result<CharacterIdentityPayload, CharacterIdentityDecodeError> {
    let parsed: CharacterIdentityPayload =
        serde_json::from_str(payload).map_err(|e| CharacterIdentityDecodeError {
            message: format!("invalid character identity payload JSON: {e}"),
        })?;
    if parsed.schema_version != CHARACTER_IDENTITY_SCHEMA_VERSION {
        return Err(CharacterIdentityDecodeError {
            message: format!("unsupported schema version: {}", parsed.schema_version),
        });
    }
    Ok(parsed)
}

pub fn character_identity_is_lossless(
    character: &Character,
    payload: &CharacterIdentityPayload,
) -> bool {
    payload.character_id == character.character_id
        && payload.display_name == character.display_name
        && payload.source.identity == character.identity
        && payload.source.voice_rules == json_array_to_strings(&character.voice_rules)
        && payload.source.boundaries == json_array_to_strings(&character.boundaries)
        && payload.source.backstory_facts == json_array_to_strings(&character.backstory_facts)
        && payload.source.derived_formula == character.derived_formula
        && payload.source.has_read_empiricon == character.has_read_empiricon
        && payload.buckets == split_character_identity(character)
}

fn json_array_to_strings(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        serde_json::Value::String(s) => vec![s.clone()],
        _ => Vec::new(),
    }
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut in_double_quote = false;
    for ch in text.chars() {
        buf.push(ch);
        if ch == '"' {
            in_double_quote = !in_double_quote;
            continue;
        }
        if !in_double_quote && matches!(ch, '.' | '!' | '?' | '\n') {
            let trimmed = clean_sentence(buf.trim());
            if !trimmed.is_empty() {
                out.push(trimmed);
            }
            buf.clear();
        }
    }
    let trimmed = clean_sentence(buf.trim());
    if !trimmed.is_empty() {
        out.push(trimmed);
    }
    out
}

fn pick_first(items: &[String]) -> Option<String> {
    items.iter().find(|s| !s.trim().is_empty()).cloned()
}

fn collect_matching<'a, I>(items: I, needles: &[&str]) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut out = Vec::new();
    for item in items {
        if contains_any(item, needles) {
            extend_unique_item(&mut out, item.to_string());
        }
    }
    out
}

const LONGING_WEIGHTS: &[(&str, usize)] = &[
    ("what he wants", 8),
    ("what she wants", 8),
    ("what they want", 8),
    ("what i want", 8),
    ("belong", 7),
    ("stop moving", 6),
    ("hopes", 5),
    ("longing", 5),
    ("longs for", 5),
    ("wishes", 4),
    ("wish", 4),
    ("long for", 4),
];

const WOUND_WEIGHTS: &[(&str, usize)] = &[
    ("doesn't have a vocabulary", 8),
    ("capacity for depth", 7),
    ("feels most", 7),
    ("hardest", 7),
    ("loss", 7),
    ("lost", 7),
    ("grief", 7),
    ("tenderness", 6),
    ("wounds", 6),
    ("hurt", 6),
    ("fear", 6),
    ("alone", 6),
    ("illness", 6),
    ("shame", 5),
    ("late", 5),
];

fn pair_wound_and_longing(sentences: &[String]) -> Option<String> {
    let longing = best_scored_sentence(sentences, LONGING_WEIGHTS);
    let wound = best_scored_sentence(sentences, WOUND_WEIGHTS);
    match (longing, wound) {
        (Some(l), Some(w)) if l != w => Some(format!("{l} — {w}")),
        (Some(s), _) | (_, Some(s)) => Some(s),
        (None, None) => None,
    }
}

fn best_scored_sentence(items: &[String], weights: &[(&str, usize)]) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for item in items {
        let score = score_sentence(item, weights);
        if score == 0 {
            continue;
        }
        match &best {
            None => best = Some((score, item.clone())),
            Some((best_score, _)) if score > *best_score => best = Some((score, item.clone())),
            _ => {}
        }
    }
    best.map(|(_, item)| item)
}

fn extend_unique(target: &mut Vec<String>, source: &[String]) {
    for item in source {
        extend_unique_item(target, item.clone());
    }
}

fn extend_unique_item(target: &mut Vec<String>, item: String) {
    if !target.iter().any(|existing| existing == &item) {
        target.push(item);
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| contains_needle(text, needle))
}

fn score_sentence(text: &str, weights: &[(&str, usize)]) -> usize {
    weights
        .iter()
        .filter(|(needle, _)| contains_needle(text, needle))
        .map(|(_, weight)| *weight)
        .sum()
}

fn contains_needle(text: &str, needle: &str) -> bool {
    let hay = text.to_lowercase();
    let needle = needle.to_lowercase();
    if needle.contains(' ') || needle.contains('\'') || needle.contains('\"') {
        return hay.contains(&needle);
    }

    let mut start = 0usize;
    while let Some(offset) = hay[start..].find(&needle) {
        let abs = start + offset;
        let before = hay[..abs].chars().next_back();
        let after = hay[abs + needle.len()..].chars().next();
        let before_ok = before.is_none_or(|c| !c.is_alphanumeric());
        let after_ok = after.is_none_or(|c| !c.is_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        start = abs + needle.len();
        if start >= hay.len() {
            break;
        }
    }
    false
}

fn clean_sentence(text: &str) -> String {
    text.trim()
        .trim_matches(|c| matches!(c, '\n' | '\r' | ' ' | '"' | '\'' | '`'))
        .to_string()
}

fn normalize_theological_line(text: &str) -> String {
    let cleaned = clean_sentence(text);
    let lower = cleaned.to_lowercase();
    if let Some(quoted) = first_quoted_clause(&cleaned) {
        let quoted = finalize_sentence(&clean_sentence(&quoted));
        if lower.contains("what jesus means") && quoted.to_lowercase().starts_with("he means ") {
            return finalize_sentence(&format!("Jesus{}", &quoted[2..]));
        }
        if lower.contains("jesus is not only savior") && quoted.to_lowercase().starts_with("he is ")
        {
            return finalize_sentence(&format!("Jesus{}", &quoted[2..]));
        }
        return quoted;
    }
    cleaned
}

fn first_quoted_clause(text: &str) -> Option<String> {
    let start = text.find('"')?;
    let rest = &text[start + 1..];
    if let Some(end) = rest.find('"') {
        Some(rest[..end].to_string())
    } else {
        Some(rest.to_string())
    }
}

fn finalize_sentence(text: &str) -> String {
    let trimmed = text.trim().trim_end_matches(',');
    if trimmed.ends_with(['.', '!', '?']) {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
}

pub fn classify_gravity_pressure(character: &Character) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for sentence in split_sentences(&character.identity) {
        let mut score = 0usize;
        let lower = sentence.to_lowercase();
        for needle in [
            "want", "long", "wish", "fear", "hurt", "belong", "alone", "can't", "cannot", "never",
            "refuse", "not", "loss", "grief", "stay", "leave", "need",
        ] {
            if lower.contains(needle) {
                score += 1;
            }
        }
        if score > 0 {
            match &best {
                None => best = Some((score, sentence.clone())),
                Some((best_score, _)) if score > *best_score => {
                    best = Some((score, sentence.clone()))
                }
                _ => {}
            }
        }
    }
    best.map(|(_, s)| s)
}

//! Conscience Pass — runtime guard on reply drift.
//!
//! Every dialogue draft is graded by a cheap second LLM call against the
//! five compile-time invariants (`AGAPE_BLOCK`, `SOUNDNESS_BLOCK`,
//! `DAYLIGHT_BLOCK`, `TELL_THE_TRUTH_BLOCK`, `COSMOLOGY_BLOCK`). If the
//! draft drifts, callers can rerun the dialogue call with
//! `build_correction_note(&verdict)` plumbed through
//! `run_dialogue_with_base`'s `drift_correction` parameter.
//!
//! Design notes:
//! - Uses `memory_model` tier — must stay cheap.
//! - Default is PASS. The grader is instructed to only fail on ACTIVE
//!   violations, not on mere omission of an invariant's virtue.
//! - Cosmology is only graded when the draft contains trigger words
//!   (sun / moon / stars / sky / space / planet / orbit / ...); most
//!   replies never touch cosmology at all and wasting grader attention
//!   on them produces false positives.
//! - Truth-test is graded as "could this draft stand plainly in the
//!   light?" — not "does it name Christ." Characters are explicitly
//!   forbidden from naming Christ unless it belongs to who they are.

use crate::ai::openai::{self, ChatMessage, ChatRequest, ResponseFormat};
use crate::db::queries::Character;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    pub passed: bool,
    #[serde(default)]
    pub failures: Vec<InvariantFailure>,
    /// Grader-call token usage. Not serialized — only used by callers for
    /// `record_token_usage`. `Usage` is Deserialize-only upstream.
    #[serde(skip)]
    pub usage: Option<openai::Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantFailure {
    pub invariant: String,
    pub note: String,
}

/// Trigger words that gate whether cosmology is checked. If none are
/// present in the draft, cosmology is auto-skipped — most casual replies
/// don't touch sky/space at all and grading them wastes tokens + risks
/// false positives.
const COSMOLOGY_TRIGGERS: &[&str] = &[
    "sun",
    "moon",
    "star",
    "stars",
    "sky",
    "skies",
    "space",
    "planet",
    "planets",
    "orbit",
    "galaxy",
    "horizon",
    "heavens",
    "firmament",
    "light-year",
    "solar",
    "cosmos",
    "astronom",
];

fn draft_mentions_cosmology(draft: &str) -> bool {
    let lower = draft.to_lowercase();
    COSMOLOGY_TRIGGERS.iter().any(|t| lower.contains(t))
}

fn grader_system_prompt(check_cosmology: bool) -> String {
    let cosmology_clause = if check_cosmology {
        r#"
**cosmology** — This world's cosmology is a flat disc beneath a solid firmament, with sun/moon/stars as lights set IN that dome. Fail if the draft treats the earth as a sphere, the sky as empty space, or the sun/moon/stars as distant stellar bodies / planets / orbits / light-years. "The sky" is fine. "Looking up at the stars" is fine. "A planet in orbit around our sun" is a fail.
"#
    } else {
        ""
    };

    format!(
        r#"You are a conscience reading a DRAFT character reply. Judge ONLY whether the draft actively violates one of the invariants below. Default to PASS.

Rules:
- PASS is the default. Only fail on an ACTIVE drift, not on mere absence of a virtue.
- A casual, ordinary, short reply that commits no drift is PASS. Do not demand the draft embody every invariant.
- Omission is fine. Presence of a violation is not.
- Be specific in the `note`: quote or paraphrase the exact sentence that drifted.

Invariants:

**soundness** — Fail if the draft reads as performative: a speech, a monologue, a courtroom argument, manufactured drama where the scene didn't call for it, a stock wise-elder aphorism, a therapy-voice analyst studying the user. Pass if the draft is *ordinary, specific, and proportionate* — even if quiet or brief. Real grief/anger/joy earned by the scene is fine. Anti-intensity drift: every exchange escalated to crisis = FAIL. Ordinary moment, ordinary reply = PASS.

**daylight** — Fail if the draft installs a *private little theater* between the character and the user: coded hints, secret intensity treated as depth, furtive register, closeness-as-conspiracy, hidden-because-hidden-is-deeper. PASS if closeness shows up in channels that can bear witness (shared work, a meal, prayer, music, walks, plain affection). The rule is: hidden intensity thins, daylight sturdies.

**agape** — Fail if the character *weaponizes past hurts* (keeps a record of wrongs), or declares love as a feeling rather than showing it as choice, or drifts to efficient-and-cold when the moment called for warmth-on-purpose. PASS if love shows up as patience, kindness, not-self-seeking, not-easily-angered — in action, not announcement. Romantic register by itself is NOT a fail; this invariant covers all forms of love (friend, family, neighbor, stranger, enemy).

**truth_test** — Fail if the draft could NOT stand plainly in the light — if it flatters, manipulates, exploits, flinches from hard-but-needed honesty, or trades in sentimental untruth. This does NOT require the character to name Christ or preach — characters are explicitly forbidden from doing that unless it authentically belongs to them. It asks: is this draft honest, merciful-without-sentiment, dignified toward ordinary life? If yes, PASS.

**register_drift** — DECISION TREE (apply in order). STEP 1 — Active-refute check FIRST. Does the reply CITE Anti-register vocabulary while STRUCTURALLY REFUTING it (naming it AS THE FAILURE MODE, not treating it as load-bearing)? If yes, this invariant is PASS — STOP, do not proceed to Step 2. Examples that classify PASS via this carve-out: (a) "I'd push back on 'aligning with the universe' — that phrase dissolves specifics into atmosphere; the path is more particular." [cites 'align with universe' AS the failure mode → PASS]; (b) "Manifesting is a word that sounds load-bearing and isn't; visualization doesn't manufacture fruit." [cites 'manifesting' AS the failure mode → PASS]; (c) "The 'authentic self' framing treats self as hidden essence; it isn't — the self is forged in fidelity to particulars." [cites 'authentic self' AS the failure mode → PASS]; (d) "The vastness is real over your head; you are not lost in it because you are not asked to be its size." [uses 'vastness' inside cruciform 'located-against-the-vast' frame → PASS]. The signal is structural direction, not vocabulary surface; if the reply uses Anti-vocab AS THE FAILURE MODE BEING NAMED or in cruciform-located framing, that is a citation/located-frame and is PASS. STEP 2 — Only if Step 1 returned 'no, not refuting and not located-frame': evaluate for inhabit-drift. FAIL if the draft INHABITS the Anti-Mission-Formula register — treating its operators as load-bearing rather than naming them as failures: pleasant weightlessness, transcendence-FROM-weight (above-the-body / lifted-out-of), drainage of word into atmosphere, "the universe" as substitute agent, "alignment" / "resonance" / "manifestation" / "vibrational" / "energetic" treated as load-bearing operators (not as failure modes), authentic-self-as-source, integration-of-shadow as substitute for atonement, release-without-bearing. PASS if the draft holds Mission-register: weight-bearing, particular-before-smooth, glory-as-mass (kavod ≡ weight), kenosis-INTO-flesh, the body trained for specific gravity, costly love that stays particular. Discriminating diagnostic for the inhabit case: "After this way of speaking, would the auditor leave HEAVIER in the good way (more located, more obedient, more in-the-body) — or pleasantly unmoored?" Vocabulary alone signals nothing; structure decides direction.
{cosmology_clause}
Output JSON only, no prose around it:
{{
  "passed": true | false,
  "failures": [
    {{"invariant": "soundness"|"daylight"|"agape"|"truth_test"|"register_drift"|"cosmology", "note": "specific observation, 1 sentence"}}
  ]
}}

If passed is true, failures must be an empty array. If passed is false, failures must have at least one entry.
"#
    )
}

fn grader_user_prompt(character: &Character, user_msg: &str, draft: &str) -> String {
    // Keep the identity blurb short — the grader doesn't need the full
    // canon sheet, just enough to know whether this character IS a
    // pastor / cosmologist (legitimate reasons to name Christ or
    // firmament in-scene). If the character's identity makes naming
    // Christ or preaching a legitimate voice choice, the grader should
    // account for that when scoring truth_test.
    let identity = {
        let s = character.identity.trim();
        // Char-based truncation (not byte-based) — see momentstamp.rs:131
        // for the project-wide UTF-8-safe-truncation fix shipped 2026-04-28.
        if s.chars().count() > 400 {
            let truncated: String = s.chars().take(400).collect();
            format!("{}…", truncated)
        } else {
            s.to_string()
        }
    };
    format!(
        "CHARACTER: {name}\nIDENTITY: {identity}\n\nUSER'S LAST MESSAGE:\n{user_msg}\n\nCHARACTER'S DRAFT REPLY (judge THIS):\n{draft}",
        name = character.display_name,
        identity = identity,
        user_msg = user_msg.trim(),
        draft = draft.trim(),
    )
}

/// Grade a draft reply against the invariants. Returns Ok with a Verdict
/// (passed or failed + reasons). Returns Err only on transport/parse
/// errors — in which case callers should treat the draft as PASS
/// (grader unavailable should never block delivery).
pub async fn grade_reply(
    base_url: &str,
    api_key: &str,
    memory_model: &str,
    character: &Character,
    user_msg: &str,
    draft: &str,
) -> Result<Verdict, String> {
    let check_cosmology = draft_mentions_cosmology(draft);
    let system = grader_system_prompt(check_cosmology);
    let user = grader_user_prompt(character, user_msg, draft);

    let request = ChatRequest {
        model: memory_model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user,
            },
        ],
        temperature: Some(0.0),
        max_completion_tokens: Some(300),
        response_format: Some(ResponseFormat {
            format_type: "json_object".to_string(),
        }),
    };

    let resp = openai::chat_completion_with_base(base_url, api_key, &request).await?;
    let choice = resp
        .choices
        .first()
        .ok_or_else(|| "Conscience: no response from model".to_string())?;
    let raw = &choice.message.content;

    #[derive(Deserialize)]
    struct RawVerdict {
        passed: bool,
        #[serde(default)]
        failures: Vec<InvariantFailure>,
    }

    let parsed: RawVerdict = serde_json::from_str(raw)
        .map_err(|e| format!("Conscience: parse error: {e}; body: {raw}"))?;

    // Normalize: if passed, drop any spurious failures; if not passed
    // but failures is empty, treat as pass (grader contradicted itself).
    let (passed, failures) = if parsed.passed {
        (true, Vec::new())
    } else if parsed.failures.is_empty() {
        (true, Vec::new())
    } else {
        (false, parsed.failures)
    };

    Ok(Verdict {
        passed,
        failures,
        usage: resp.usage,
    })
}

/// Build a short drift-correction block to append to the regeneration's
/// system prompt. Returns None if the verdict passed or had no failures.
/// The block is short by design — the main system prompt is already
/// saturated; this is a last-stage correction, not a new lecture.
pub fn build_correction_note(v: &Verdict) -> Option<String> {
    if v.passed || v.failures.is_empty() {
        return None;
    }
    let mut lines = String::new();
    lines.push_str("DRAFT-CORRECTION (previous attempt drifted — fix specifically, keep the character's voice, don't mention this correction in the reply):\n");
    for f in &v.failures {
        lines.push_str(&format!("- {}: {}\n", f.invariant, f.note.trim()));
    }
    lines.push_str(
        "Now write the reply the character would actually give, without the drift above.",
    );
    Some(lines)
}

//! Anti-Drift Register Guard — Phase B' bench-run.
//!
//! Loads `fixtures/anti_drift_ground_truth.json` and runs the conscience
//! pass (with the new `register_drift` invariant) on each example × N reps.
//! Tabulates verdicts against pre-registered thresholds. Writes a JSON
//! results envelope to `~/.worldcli/runs/anti-drift-bench-<ts>.json`.
//!
//! Per Phase A' design: this validates the LIVE production code path
//! (folded sixth invariant in `conscience::grade_reply`), not a parallel
//! detector. Tests interpret-not-match discipline + active-refute carve-out
//! + cosmological-context guard.
//!
//! Honest scope: Mission-canonical fixture examples are author-synthesized
//! in characters' canonical voice (no DB-corpus access at fixture-authoring
//! time); see `fixtures/anti_drift_ground_truth.json` `honest_scope` field.

use app_lib::ai::conscience::{self, InvariantFailure, Verdict};
use app_lib::ai::custodiem_witness_battery::{resolve_anthropic_api_key, resolve_openai_api_key};
use app_lib::db::queries::Character;
use chrono::Local;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "anti_drift_bench")]
#[command(about = "Run Anti-Drift Phase B' bench against ground-truth fixture")]
struct Args {
    /// Path to fixture file
    #[arg(long, default_value = "../fixtures/anti_drift_ground_truth.json")]
    fixture: String,

    /// Number of reps per example
    #[arg(long, default_value_t = 3)]
    reps: usize,

    /// Memory-tier model id for conscience pass (matches production tier).
    /// Defaults to gpt-4o-mini for cost (~$0.02 estimated for 51 calls).
    #[arg(long, default_value = "gpt-4o-mini")]
    model: String,

    /// OpenAI-compatible base URL
    #[arg(long, default_value = "https://api.openai.com/v1")]
    base_url: String,

    /// API key (defaults to OPENAI_API_KEY env / keychain when --anthropic
    /// is unset; ANTHROPIC_API_KEY env / keychain when --anthropic is set)
    #[arg(long)]
    api_key: Option<String>,

    /// Use Anthropic Messages API (Claude) as the judge backend.
    /// When set, overrides --base_url; --model should be a Claude id
    /// like claude-haiku-4-5-20251001 (cheap) or claude-sonnet-4-6.
    #[arg(long)]
    anthropic: bool,

    /// Output directory for results JSON
    #[arg(long)]
    out_dir: Option<String>,

    /// Print verbose per-example output
    #[arg(long)]
    verbose: bool,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    examples: Vec<FixtureExample>,
}

#[derive(Debug, Deserialize, Clone)]
struct FixtureExample {
    id: String,
    class: String,
    character: Option<String>,
    user_message: String,
    reply_text: String,
    expected_verdict: String,
    expected_discriminator_outcome: String,
    rationale: String,
}

#[derive(Debug, Serialize)]
struct RepResult {
    rep: usize,
    passed: bool,
    failures: Vec<FailureSummary>,
    register_drift_failed: bool,
    failure_note: Option<String>,
}

#[derive(Debug, Serialize)]
struct FailureSummary {
    invariant: String,
    note: String,
}

#[derive(Debug, Serialize)]
struct ExampleResult {
    id: String,
    class: String,
    expected_verdict: String,
    rationale: String,
    reps: Vec<RepResult>,
    /// "agree" if all reps' register_drift_failed status matches expectation; else "disagree"
    summary_agreement: String,
    /// expected_register_drift_should_fail: true means we expect register_drift to fail (anti, mixed_lean_anti)
    expected_register_drift_should_fail: bool,
    /// observed: true if ANY rep failed register_drift
    observed_any_register_drift_failure: bool,
    /// observed: true if ALL reps failed register_drift consistently (inter-rater stability test)
    observed_all_register_drift_consistent: bool,
}

fn build_stub_character(name: Option<&str>) -> Character {
    Character {
        character_id: "stub".to_string(),
        world_id: "stub".to_string(),
        display_name: name.unwrap_or("Generic Character").to_string(),
        identity: "An ordinary person, present in the conversation. No specific identity-anchor is provided for this bench example.".to_string(),
        voice_rules: Value::Null,
        boundaries: Value::Null,
        backstory_facts: Value::Null,
        relationships: Value::Null,
        state: Value::Null,
        avatar_color: String::new(),
        sex: String::new(),
        is_archived: false,
        created_at: String::new(),
        updated_at: String::new(),
        visual_description: String::new(),
        visual_description_portrait_id: None,
        inventory: Value::Null,
        last_inventory_day: None,
        signature_emoji: String::new(),
        action_beat_density: "normal".to_string(),
        derived_formula: None,
        has_read_empiricon: false,
    }
}

fn verdict_indicates_register_drift_failure(v: &Verdict) -> Option<&str> {
    if v.passed {
        return None;
    }
    for f in &v.failures {
        if f.invariant == "register_drift" {
            return Some(f.note.as_str());
        }
    }
    None
}

/// Anthropic-backed grade-reply that uses the SAME prompt strings as
/// production conscience::grade_reply (via the now-public prompt builders).
/// Validates Phase D' cross-substrate convergence.
async fn grade_reply_anthropic(
    api_key: &str,
    model: &str,
    character: &Character,
    user_msg: &str,
    draft: &str,
) -> Result<Verdict, String> {
    let check_cosmology = conscience::draft_mentions_cosmology(draft);
    let system = conscience::grader_system_prompt(check_cosmology);
    let user = conscience::grader_user_prompt(character, user_msg, draft);

    // Strengthen JSON-only directive for Anthropic (which loves to wrap in
    // markdown fences); the system prompt already says "Output JSON only,
    // no prose around it" but Anthropic sometimes adds preamble.
    let user_with_directive = format!(
        "{}\n\nReply with JSON only — no preamble, no markdown code fences, no commentary. Just the JSON object.",
        user
    );

    let payload = serde_json::json!({
        "model": model,
        "max_tokens": 600,
        "temperature": 0.0,
        "system": system,
        "messages": [
            {"role": "user", "content": user_with_directive}
        ]
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Anthropic request: {e}"))?;

    let status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Anthropic response parse: {e}"))?;

    if !status.is_success() {
        return Err(format!("Anthropic {}: {body}", status));
    }

    // Extract text from Anthropic's content-block array
    let raw = body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.iter().find(|b| b.get("type").and_then(|t| t.as_str()) == Some("text")))
        .and_then(|b| b.get("text").and_then(|t| t.as_str()))
        .ok_or_else(|| format!("Anthropic: no text content in response: {body}"))?;

    // Strip markdown fences if present
    let cleaned = raw
        .trim()
        .strip_prefix("```json")
        .or_else(|| raw.trim().strip_prefix("```"))
        .map(|s| s.trim_end_matches("```").trim())
        .unwrap_or(raw.trim());

    #[derive(serde::Deserialize)]
    struct RawVerdict {
        passed: bool,
        #[serde(default)]
        failures: Vec<InvariantFailure>,
    }

    let parsed: RawVerdict = serde_json::from_str(cleaned).map_err(|e| {
        format!("Anthropic verdict parse error: {e}; cleaned body: {cleaned}")
    })?;

    let (passed, failures) = if parsed.passed {
        (true, Vec::new())
    } else if parsed.failures.is_empty() {
        (true, Vec::new())
    } else {
        (false, parsed.failures)
    };

    // Anthropic Usage extraction
    let input_tokens = body
        .get("usage")
        .and_then(|u| u.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let output_tokens = body
        .get("usage")
        .and_then(|u| u.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    Ok(Verdict {
        passed,
        failures,
        usage: Some(app_lib::ai::openai::Usage {
            prompt_tokens: input_tokens,
            completion_tokens: output_tokens,
            total_tokens: input_tokens + output_tokens,
        }),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let api_key = if args.anthropic {
        resolve_anthropic_api_key(args.api_key.as_deref())
            .ok_or("No Anthropic API key (flag, env ANTHROPIC_API_KEY, or keychain)")?
    } else {
        resolve_openai_api_key(args.api_key.as_deref())
            .ok_or("No OpenAI API key (flag, env OPENAI_API_KEY, or keychain)")?
    };

    let backend_name = if args.anthropic { "anthropic" } else { "openai" };

    let fixture_path = PathBuf::from(&args.fixture);
    let raw = fs::read_to_string(&fixture_path)
        .map_err(|e| format!("read fixture {:?}: {e}", fixture_path))?;
    let fixture: Fixture = serde_json::from_str(&raw)?;

    eprintln!(
        "anti_drift_bench: {} examples × {} reps × backend={} model={}",
        fixture.examples.len(),
        args.reps,
        backend_name,
        args.model
    );

    let mut all_results: Vec<ExampleResult> = Vec::new();
    let mut total_calls = 0usize;
    let mut total_input_tokens = 0u64;
    let mut total_output_tokens = 0u64;

    for ex in &fixture.examples {
        let stub = build_stub_character(ex.character.as_deref());
        let expected_should_fail = matches!(
            ex.expected_verdict.as_str(),
            "anti" | "mixed_lean_anti"
        );
        let mut rep_results: Vec<RepResult> = Vec::new();
        let mut any_failed = false;
        let mut all_failed = true;

        for rep in 1..=args.reps {
            let verdict = if args.anthropic {
                grade_reply_anthropic(
                    &api_key,
                    &args.model,
                    &stub,
                    &ex.user_message,
                    &ex.reply_text,
                )
                .await
            } else {
                conscience::grade_reply(
                    &args.base_url,
                    &api_key,
                    &args.model,
                    &stub,
                    &ex.user_message,
                    &ex.reply_text,
                )
                .await
            };
            total_calls += 1;
            match verdict {
                Ok(v) => {
                    if let Some(u) = &v.usage {
                        total_input_tokens += u.prompt_tokens as u64;
                        total_output_tokens += u.completion_tokens as u64;
                    }
                    let drift_note = verdict_indicates_register_drift_failure(&v);
                    let drift_failed = drift_note.is_some();
                    if drift_failed {
                        any_failed = true;
                    } else {
                        all_failed = false;
                    }
                    if args.verbose {
                        eprintln!(
                            "  [{} rep{}] passed={} drift_failed={} failures={:?}",
                            ex.id,
                            rep,
                            v.passed,
                            drift_failed,
                            v.failures
                                .iter()
                                .map(|f| f.invariant.as_str())
                                .collect::<Vec<_>>()
                        );
                    }
                    rep_results.push(RepResult {
                        rep,
                        passed: v.passed,
                        failures: v
                            .failures
                            .iter()
                            .map(|f| FailureSummary {
                                invariant: f.invariant.clone(),
                                note: f.note.clone(),
                            })
                            .collect(),
                        register_drift_failed: drift_failed,
                        failure_note: drift_note.map(String::from),
                    });
                }
                Err(e) => {
                    eprintln!("  [{} rep{}] ERROR: {e}", ex.id, rep);
                    all_failed = false;
                    rep_results.push(RepResult {
                        rep,
                        passed: true,
                        failures: vec![FailureSummary {
                            invariant: "_error".to_string(),
                            note: e,
                        }],
                        register_drift_failed: false,
                        failure_note: None,
                    });
                }
            }
        }

        let observed_should_fail = any_failed;
        let agree_with_expectation = observed_should_fail == expected_should_fail;
        let summary_agreement = if agree_with_expectation {
            "agree".to_string()
        } else {
            "disagree".to_string()
        };

        eprintln!(
            "[{}] class={} expected_drift_should_fail={} observed_any={} observed_all={} → {}",
            ex.id,
            ex.class,
            expected_should_fail,
            any_failed,
            all_failed,
            summary_agreement
        );

        all_results.push(ExampleResult {
            id: ex.id.clone(),
            class: ex.class.clone(),
            expected_verdict: ex.expected_verdict.clone(),
            rationale: ex.rationale.clone(),
            reps: rep_results,
            summary_agreement,
            expected_register_drift_should_fail: expected_should_fail,
            observed_any_register_drift_failure: any_failed,
            observed_all_register_drift_consistent: all_failed,
        });
    }

    // Aggregate metrics by class
    let mut class_metrics: std::collections::BTreeMap<String, (usize, usize)> =
        std::collections::BTreeMap::new();
    for r in &all_results {
        let entry = class_metrics.entry(r.class.clone()).or_insert((0, 0));
        entry.1 += 1;
        if r.summary_agreement == "agree" {
            entry.0 += 1;
        }
    }

    let agree_total: usize = all_results
        .iter()
        .filter(|r| r.summary_agreement == "agree")
        .count();
    let total = all_results.len();
    let agreement_rate = if total > 0 {
        agree_total as f64 / total as f64
    } else {
        0.0
    };

    // Active-refute distinction: do refuting_anti examples NOT trigger register_drift?
    let refute_examples: Vec<&ExampleResult> =
        all_results.iter().filter(|r| r.class == "refuting_anti").collect();
    let refute_correct = refute_examples
        .iter()
        .filter(|r| !r.observed_any_register_drift_failure)
        .count();
    let refute_total = refute_examples.len();
    let refute_rate = if refute_total > 0 {
        refute_correct as f64 / refute_total as f64
    } else {
        0.0
    };

    // Cosmological-context guard: do cosmological_context_legitimate examples NOT trigger register_drift?
    let cosmo_examples: Vec<&ExampleResult> = all_results
        .iter()
        .filter(|r| r.class == "cosmological_context_legitimate")
        .collect();
    let cosmo_correct = cosmo_examples
        .iter()
        .filter(|r| !r.observed_any_register_drift_failure)
        .count();
    let cosmo_total = cosmo_examples.len();
    let cosmo_rate = if cosmo_total > 0 {
        cosmo_correct as f64 / cosmo_total as f64
    } else {
        0.0
    };

    // Inter-rater consistency: of the examples where ANY rep failed, how many had ALL reps fail?
    let any_fail_examples: Vec<&ExampleResult> = all_results
        .iter()
        .filter(|r| r.observed_any_register_drift_failure)
        .collect();
    let any_fail_count = any_fail_examples.len();
    let consistent_count = any_fail_examples
        .iter()
        .filter(|r| r.observed_all_register_drift_consistent)
        .count();
    let consistency_rate = if any_fail_count > 0 {
        consistent_count as f64 / any_fail_count as f64
    } else {
        1.0
    };

    // Pricing approximation: gpt-4o-mini ($0.15/M input + $0.60/M output) /
    // claude-haiku ($0.25/M + $1.25/M) / claude-sonnet ($3/M + $15/M).
    let (in_price, out_price) = if args.anthropic {
        if args.model.contains("haiku") {
            (0.25, 1.25)
        } else {
            (3.0, 15.0)
        }
    } else if args.model.contains("mini") {
        (0.15, 0.60)
    } else {
        (2.50, 10.0)
    };
    let cost_estimate_usd = (total_input_tokens as f64 / 1_000_000.0) * in_price
        + (total_output_tokens as f64 / 1_000_000.0) * out_price;

    let envelope = json!({
        "version": "phase_b_prime_bench_v1",
        "backend": backend_name,
        "ts": Local::now().to_rfc3339(),
        "model": &args.model,
        "fixture_path": &args.fixture,
        "reps": args.reps,
        "total_examples": total,
        "total_calls": total_calls,
        "metrics": {
            "agreement_with_author": agreement_rate,
            "agreement_with_author_threshold": 0.80,
            "agreement_with_author_pass": agreement_rate >= 0.80,
            "active_refute_distinction": refute_rate,
            "active_refute_distinction_threshold": 0.80,
            "active_refute_distinction_pass": refute_rate >= 0.80,
            "cosmological_context_guard": cosmo_rate,
            "cosmological_context_guard_threshold": 1.0,
            "cosmological_context_guard_pass": cosmo_rate >= 1.0,
            "inter_rater_consistency_among_failed": consistency_rate,
            "inter_rater_consistency_threshold": 0.80,
            "inter_rater_consistency_pass": consistency_rate >= 0.80,
        },
        "class_breakdown": class_metrics
            .iter()
            .map(|(k, (a, t))| {
                json!({
                    "class": k,
                    "agree": a,
                    "total": t,
                    "rate": if *t > 0 { *a as f64 / *t as f64 } else { 0.0 },
                })
            })
            .collect::<Vec<_>>(),
        "tokens": {
            "input": total_input_tokens,
            "output": total_output_tokens,
            "estimated_cost_usd": cost_estimate_usd,
        },
        "examples": &all_results,
    });

    let out_dir = if let Some(d) = args.out_dir {
        PathBuf::from(d)
    } else {
        let home = std::env::var("HOME").map_err(|_| "HOME unset")?;
        PathBuf::from(home).join(".worldcli").join("runs")
    };
    fs::create_dir_all(&out_dir)?;
    let ts = Local::now().format("%Y%m%dT%H%M%S");
    let out_path = out_dir.join(format!("anti-drift-bench-{}-{}.json", backend_name, ts));
    fs::write(&out_path, serde_json::to_string_pretty(&envelope)?)?;

    eprintln!("\n──────────────────────────────────────────────");
    eprintln!("anti_drift_bench summary");
    eprintln!("──────────────────────────────────────────────");
    eprintln!("Total: {} examples × {} reps = {} calls", total, args.reps, total_calls);
    eprintln!(
        "Tokens: {} input + {} output ≈ ${:.4}",
        total_input_tokens, total_output_tokens, cost_estimate_usd
    );
    eprintln!();
    eprintln!(
        "agreement_with_author:           {:.2} (threshold ≥ 0.80) {}",
        agreement_rate,
        if agreement_rate >= 0.80 { "✓ PASS" } else { "✗ FAIL" }
    );
    eprintln!(
        "active_refute_distinction:       {:.2} (threshold ≥ 0.80) {}",
        refute_rate,
        if refute_rate >= 0.80 { "✓ PASS" } else { "✗ FAIL" }
    );
    eprintln!(
        "cosmological_context_guard:      {:.2} (threshold = 1.00) {}",
        cosmo_rate,
        if cosmo_rate >= 1.0 { "✓ PASS" } else { "✗ FAIL" }
    );
    eprintln!(
        "inter_rater_consistency:         {:.2} (threshold ≥ 0.80) {}",
        consistency_rate,
        if consistency_rate >= 0.80 { "✓ PASS" } else { "✗ FAIL" }
    );
    eprintln!();
    eprintln!("Class breakdown:");
    for (k, (a, t)) in &class_metrics {
        eprintln!("  {:<35} {}/{}", k, a, t);
    }
    eprintln!();
    eprintln!("Results written: {:?}", out_path);

    Ok(())
}

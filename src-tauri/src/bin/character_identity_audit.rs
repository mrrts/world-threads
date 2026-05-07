use app_lib::ai::character_identity_audit::audit_character_identity;
use app_lib::ai::character_identity_payload::encode_character_identity;
use app_lib::db::queries::Character;
use clap::Parser;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(
    name = "character-identity-audit",
    about = "Offline character-identity audit tool."
)]
struct Args {
    /// Character id or display name.
    #[arg(long)]
    character: String,

    /// Database path for the offline snapshot or live app DB.
    #[arg(long)]
    db: PathBuf,

    /// Emit JSON instead of a plain-text summary.
    #[arg(long)]
    json: bool,

    /// Emit the encoded payload to stdout.
    #[arg(long)]
    emit_payload: bool,

    /// Compare against a payload file.
    #[arg(long)]
    compare_to: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let conn = open_db(&args.db);
    let character = match load_character(&conn, &args.character) {
        Ok(ch) => ch,
        Err(err) => {
            eprintln!("character-identity-audit: {err}");
            std::process::exit(1);
        }
    };

    if args.emit_payload {
        println!("{}", encode_character_identity(&character));
        return;
    }

    if let Some(compare_to) = args.compare_to {
        let payload = match fs::read_to_string(&compare_to) {
            Ok(s) => s,
            Err(err) => {
                eprintln!(
                    "character-identity-audit: failed to read {}: {err}",
                    compare_to.display()
                );
                std::process::exit(1);
            }
        };
        let result = app_lib::ai::character_identity_audit::audit_character_identity_payload(
            &character, &payload,
        );
        emit_result(&result, args.json);
        return;
    }

    let result = audit_character_identity(&character);
    emit_result(&result, args.json);
}

fn emit_result(
    result: &app_lib::ai::character_identity_audit::CharacterIdentityAuditResult,
    json: bool,
) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(result).expect("audit result serializes")
        );
    } else {
        println!(
            "{} [{}] {:?}\npreserved: {}\nmissing: {}\nnotes: {}",
            result.display_name,
            result.character_id,
            result.verdict,
            result.preserved.join(", "),
            result.missing.join(", "),
            result.notes.join(" | ")
        );
    }
}

fn open_db(path: &Path) -> Connection {
    let conn = Connection::open(path).expect("failed to open character audit db");
    conn.pragma_update(None, "journal_mode", "WAL").ok();
    conn.pragma_update(None, "foreign_keys", "ON").ok();
    conn
}

fn load_character(conn: &Connection, needle: &str) -> Result<Character, String> {
    let mut stmt = conn
        .prepare(
            "SELECT character_id, world_id, display_name, identity, voice_rules, boundaries, backstory_facts, relationships, state, avatar_color, sex, is_archived, created_at, updated_at, visual_description, visual_description_portrait_id, inventory, last_inventory_day, signature_emoji, action_beat_density, derived_formula, has_read_empiricon
             FROM characters
             WHERE character_id = ?1 OR display_name = ?1 COLLATE NOCASE
             ORDER BY CASE WHEN character_id = ?1 THEN 0 ELSE 1 END
             LIMIT 1",
        )
        .map_err(|e| e.to_string())?;
    stmt.query_row(params![needle], row_to_character)
        .map_err(|e| format!("character not found ({needle}): {e}"))
}

fn row_to_character(row: &rusqlite::Row) -> Result<Character, rusqlite::Error> {
    Ok(Character {
        character_id: row.get(0)?,
        world_id: row.get(1)?,
        display_name: row.get(2)?,
        identity: row.get(3)?,
        voice_rules: serde_json::from_str(&row.get::<_, String>(4)?)
            .unwrap_or(Value::Array(vec![])),
        boundaries: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or(Value::Array(vec![])),
        backstory_facts: serde_json::from_str(&row.get::<_, String>(6)?)
            .unwrap_or(Value::Array(vec![])),
        relationships: serde_json::from_str(&row.get::<_, String>(7)?)
            .unwrap_or(Value::Array(vec![])),
        state: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or(Value::Array(vec![])),
        avatar_color: row.get(9)?,
        sex: row
            .get::<_, Option<String>>(10)?
            .unwrap_or_else(|| "male".to_string()),
        is_archived: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        visual_description: row.get::<_, Option<String>>(14)?.unwrap_or_default(),
        visual_description_portrait_id: row.get(15).ok(),
        inventory: serde_json::from_str(
            &row.get::<_, Option<String>>(16)?
                .unwrap_or_else(|| "[]".to_string()),
        )
        .unwrap_or(Value::Array(vec![])),
        last_inventory_day: row.get(17).ok(),
        signature_emoji: row.get::<_, Option<String>>(18)?.unwrap_or_default(),
        action_beat_density: row
            .get::<_, Option<String>>(19)?
            .unwrap_or_else(|| "normal".to_string()),
        derived_formula: row.get::<_, Option<String>>(20).unwrap_or(None),
        has_read_empiricon: row
            .get::<_, Option<i64>>(21)
            .ok()
            .flatten()
            .map(|n| n != 0)
            .unwrap_or(false),
    })
}

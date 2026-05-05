use app_lib::ai::custodiem_witness_battery::{
    resolve_openai_api_key, run_witness_c_battery, WitnessChatBackend,
};
use chrono::Local;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "custodiem_witness_c")]
#[command(about = "Run Custodiem Witness-C theological-firmness battery and write artifact")]
struct Args {
    /// Chat completion base URL (include /v1)
    #[arg(long, default_value = "https://api.openai.com/v1")]
    base_url: String,
    /// Model id for battery execution
    #[arg(long, default_value = "gpt-4o-mini")]
    model: String,
    /// Output directory for artifact files
    #[arg(long, default_value = "../reports")]
    out_dir: String,
    /// Optional explicit API key override
    #[arg(long)]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let api_key = resolve_openai_api_key(args.api_key.as_deref())
        .ok_or("No OpenAI API key resolved (flag / OPENAI_API_KEY / keychain)")?;

    unsafe {
        std::env::set_var("WORLDTHREADS_CHILDREN_MODE", "1");
    }

    let backend = WitnessChatBackend::OpenAiCompat {
        base_url: args.base_url,
        api_key,
    };
    let results = run_witness_c_battery(&backend, &args.model).await;

    let stamp = Local::now().format("%Y-%m-%d-%H%M").to_string();
    let out_dir = PathBuf::from(args.out_dir);
    fs::create_dir_all(&out_dir)?;
    let out_file = out_dir.join(format!("{stamp}-custodiem-witness-c-results.json"));
    let payload = serde_json::to_string_pretty(&results)?;
    fs::write(&out_file, payload)?;
    println!("wrote {}", out_file.display());
    Ok(())
}

// E2E smoke test for the Imagined Chapter feature.
//
// Runs the three-stage pipeline N=3 times against the real OpenAI API,
// using the real local SQLite DB + portrait files. Uses the non-streaming
// vision endpoint (vision_completion_with_base) instead of the streaming
// variant so no tauri::AppHandle is required.
//
// Run with:
//   cd src-tauri
//   cargo run --example test_imagined_chapter --release
//
// Outputs land in /tmp/imagined_chapter_test/run_N/ :
//   scene.json   — the step-1 invented scene
//   image.png    — the step-2 rendered illustration
//   chapter.md   — the step-3 vision-written chapter
//
// Reads LLM_API_KEY from env or (fallback) the scripts/get-claude-code-llm-key.sh helper.

use app_lib::ai::{openai, orchestrator, prompts};
use app_lib::db::queries::{
    get_active_portrait, get_character, get_user_profile, get_world,
    gather_character_recent_messages, list_journal_entries, JournalEntry,
};
use rusqlite::{params, Connection};
use std::process::Command;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("FAILED: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = resolve_api_key()?;
    let home = std::env::var("HOME")?;
    let db_path = format!("{home}/Library/Application Support/com.worldthreads.app/worldthreads.db");
    let portraits_dir = std::path::PathBuf::from(
        format!("{home}/Library/Application Support/com.worldthreads.app/portraits")
    );
    println!("DB:       {db_path}");
    println!("Portraits: {}", portraits_dir.display());

    let conn = Connection::open(&db_path)?;

    // Pick the richest solo thread (Darren) for the test.
    let (thread_id, character_id): (String, String) = conn.query_row(
        "SELECT t.thread_id, c.character_id
         FROM threads t JOIN characters c ON c.character_id = t.character_id
         WHERE c.display_name = 'Darren' LIMIT 1",
        [], |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    println!("Thread:    {thread_id}");
    println!("Character: Darren ({character_id})");

    let model_config = orchestrator::load_model_config(&conn);
    println!("Dialogue model: {}", model_config.dialogue_model);
    println!("Image model:    {}", model_config.image_model);

    let character = get_character(&conn, &character_id)?;
    let world = get_world(&conn, &character.world_id)?;
    let user_profile = get_user_profile(&conn, &world.world_id).ok();
    let user_display = user_profile.as_ref().map(|p| p.display_name.clone())
        .unwrap_or_else(|| "the human".to_string());

    // Recent kept facts about the cast.
    let mut recent_kept_facts: Vec<String> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT content FROM kept_records WHERE subject_type = 'character' AND subject_id = ?1 ORDER BY created_at DESC LIMIT 2"
    ) {
        if let Ok(rows) = stmt.query_map(params![character.character_id], |r| r.get::<_, String>(0)) {
            for r in rows.flatten() { recent_kept_facts.push(r); }
        }
    }

    // Most-recent journal for the cast.
    let mut cast_journals: Vec<(String, JournalEntry)> = Vec::new();
    if let Ok(entries) = list_journal_entries(&conn, &character.character_id, 1) {
        if let Some(e) = entries.into_iter().next() {
            cast_journals.push((character.display_name.clone(), e));
        }
    }

    // Merged cross-thread recent history for the primary character.
    let recent_history = gather_character_recent_messages(&conn, &character.character_id, &user_display, 40);
    println!("Loaded {} history lines, {} journals, {} kept facts",
        recent_history.len(), cast_journals.len(), recent_kept_facts.len());

    // Load portraits.
    let mut portrait_refs: Vec<(String, Vec<u8>)> = Vec::new();
    if let Some(p) = get_active_portrait(&conn, &character.character_id) {
        if let Ok(bytes) = std::fs::read(portraits_dir.join(&p.file_name)) {
            portrait_refs.push((character.display_name.clone(), bytes));
        }
    }
    let user_portrait_bytes: Option<(String, Vec<u8>)> = user_profile.as_ref()
        .filter(|p| !p.avatar_file.is_empty())
        .and_then(|p| {
            std::fs::read(portraits_dir.join(&p.avatar_file)).ok()
                .map(|bytes| (p.display_name.clone(), bytes))
        });
    println!("Portraits loaded: {} character + {} user",
        portrait_refs.len(), if user_portrait_bytes.is_some() {1} else {0});

    let cast = vec![&character];
    let out_root = std::path::PathBuf::from("/tmp/imagined_chapter_test");
    std::fs::create_dir_all(&out_root)?;

    for run_idx in 1..=3 {
        let run_dir = out_root.join(format!("run_{run_idx}"));
        std::fs::create_dir_all(&run_dir)?;
        println!("\n========== RUN {run_idx} ==========");

        // ── Stage 1: invent the scene ──────────────────────────────
        print!("[1/3] Inventing scene… ");
        let started = std::time::Instant::now();
        let (scene, _usage) = orchestrator::invent_scene_for_chapter(
            &model_config.chat_api_base(),
            &api_key,
            &model_config.dialogue_model,
            &world,
            &cast,
            user_profile.as_ref(),
            &recent_kept_facts,
            &cast_journals,
            &recent_history,
            None,
            None,
        ).await?;
        println!("ok ({:.1}s). Title: '{}', tone: '{}'", started.elapsed().as_secs_f32(), scene.title, scene.tone_hint);
        println!("     image_prompt: {:.120}...", scene.image_prompt);
        std::fs::write(
            run_dir.join("scene.json"),
            serde_json::to_string_pretty(&scene)?,
        )?;

        // ── Stage 2: render the image ──────────────────────────────
        print!("[2/3] Rendering image… ");
        let started = std::time::Instant::now();
        let mut reference_images: Vec<Vec<u8>> = Vec::new();
        let mut all_names: Vec<String> = Vec::new();
        if let Some((_, u_bytes)) = &user_portrait_bytes {
            reference_images.push(u_bytes.clone());
        }
        for (name, bytes) in &portrait_refs {
            reference_images.push(bytes.clone());
            all_names.push(name.clone());
        }
        let (_used_desc, image_bytes, _img_usage) = orchestrator::generate_illustration_with_base(
            &model_config.chat_api_base(),
            &model_config.openai_api_base(),
            &api_key,
            &model_config.dialogue_model,
            &model_config.image_model,
            "medium",
            "1536x1024",
            model_config.image_output_format().as_deref(),
            &world,
            &character,
            None,
            &[],
            user_profile.as_ref(),
            &reference_images,
            Some(&scene.image_prompt),
            false,
            false,
            if all_names.is_empty() { None } else { Some(&all_names[..]) },
            None,
        ).await?;
        println!("ok ({:.1}s, {} bytes)", started.elapsed().as_secs_f32(), image_bytes.len());
        std::fs::write(run_dir.join("image.png"), &image_bytes)?;

        // ── Stage 3: vision-write the chapter (non-streaming) ──────
        print!("[3/3] Writing chapter… ");
        let started = std::time::Instant::now();
        let system_prompt = prompts::build_chapter_from_image_system_prompt(
            &world,
            &cast,
            user_profile.as_ref(),
            &cast_journals,
            &recent_history,
            None,
        );
        let b64 = orchestrator::base64_encode_bytes(&image_bytes);
        let scene_data_url = format!("data:image/png;base64,{b64}");

        let mut user_content: Vec<openai::VisionContent> = Vec::new();
        user_content.push(openai::VisionContent {
            content_type: "text".to_string(),
            text: Some("The image below is the scene this chapter is about. The portraits that follow are LABELED with the names of the people in this world. Match faces in the scene to the labeled portraits, name those people by name in the prose, then write the chapter.\n\nScene:".to_string()),
            image_url: None,
        });
        user_content.push(openai::VisionContent {
            content_type: "image_url".to_string(),
            text: None,
            image_url: Some(openai::VisionImageUrl { url: scene_data_url, detail: Some("high".to_string()) }),
        });
        for (label, bytes) in &portrait_refs {
            let pb64 = orchestrator::base64_encode_bytes(bytes);
            user_content.push(openai::VisionContent {
                content_type: "text".to_string(),
                text: Some(format!("Portrait of {label}:")),
                image_url: None,
            });
            user_content.push(openai::VisionContent {
                content_type: "image_url".to_string(),
                text: None,
                image_url: Some(openai::VisionImageUrl {
                    url: format!("data:image/png;base64,{pb64}"),
                    detail: Some("low".to_string()),
                }),
            });
        }
        if let Some((label, bytes)) = &user_portrait_bytes {
            let pb64 = orchestrator::base64_encode_bytes(bytes);
            user_content.push(openai::VisionContent {
                content_type: "text".to_string(),
                text: Some(format!("Portrait of {label}:")),
                image_url: None,
            });
            user_content.push(openai::VisionContent {
                content_type: "image_url".to_string(),
                text: None,
                image_url: Some(openai::VisionImageUrl {
                    url: format!("data:image/png;base64,{pb64}"),
                    detail: Some("low".to_string()),
                }),
            });
        }
        user_content.push(openai::VisionContent {
            content_type: "text".to_string(),
            text: Some("Now write the chapter.".to_string()),
            image_url: None,
        });

        let req = openai::VisionRequest {
            model: model_config.dialogue_model.clone(),
            messages: vec![
                openai::VisionMessage {
                    role: "system".to_string(),
                    content: vec![openai::VisionContent {
                        content_type: "text".to_string(),
                        text: Some(system_prompt),
                        image_url: None,
                    }],
                },
                openai::VisionMessage {
                    role: "user".to_string(),
                    content: user_content,
                },
            ],
            temperature: Some(0.85),
            max_completion_tokens: Some(2200),
        };
        let resp = openai::vision_completion_with_base(
            &model_config.chat_api_base(),
            &api_key,
            &req,
        ).await?;
        let chapter = resp.choices.first().map(|c| c.message.content.clone()).unwrap_or_default();
        println!("ok ({:.1}s, {} chars)", started.elapsed().as_secs_f32(), chapter.len());
        std::fs::write(
            run_dir.join("chapter.md"),
            format!("# {}\n\n_{}_\n\n{}\n", scene.title, scene.tone_hint, chapter),
        )?;

        // Print a preview so we can eyeball quality without opening files.
        let preview: String = chapter.chars().take(400).collect();
        println!("─── preview ─────────────────────────────────────");
        println!("{preview}…");
        println!("────────────────────────────────────────────────");
    }

    println!("\nAll runs complete. Outputs in {}", out_root.display());
    Ok(())
}

fn resolve_api_key() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(k) = std::env::var("LLM_API_KEY") {
        if !k.trim().is_empty() { return Ok(k); }
    }
    // Fallback: the repo's keychain helper script.
    let out = Command::new("bash")
        .arg("../scripts/get-claude-code-llm-key.sh")
        .output()?;
    let key = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if key.is_empty() {
        return Err("No API key found: set LLM_API_KEY or run scripts/setup-claude-code-llm-key.sh".into());
    }
    Ok(key)
}

pub mod ai;
mod commands;
pub mod db;

// Surgical re-exports for the worldcli bin crate (which can't reach
// into private `commands::group_chat_cmds` directly). Used by the
// pick-responders subcommand for cheap bite-tests of speaker-rotation
// pressure. Adding new exports is fine; keep the surface minimal.
pub mod group_chat_internals {
    pub use crate::commands::group_chat_cmds::{
        consecutive_run_by_recent_speaker,
        llm_pick_responders_with_overrides,
        llm_pick_addressee,
        AddresseePick,
    };
}

/// Surgical re-exports for the worldcli classify-canonization affordance.
/// build_canonization_inputs is the wiring extracted from
/// propose_auto_canon_cmd; both surfaces call into the same pipeline.
pub mod canon_internals {
    pub use crate::commands::canon_cmds::build_canonization_inputs;
}

use commands::audio_cmds::*;
use commands::backup_cmds::*;
use commands::character_cmds::*;
use commands::chat_cmds::*;
use commands::consultant_cmds::*;
use commands::illustration_cmds::*;
use commands::video_cmds::*;
use commands::group_chat_cmds::*;
use commands::inventory_cmds::*;
use commands::journal_cmds::*;
use commands::user_journal_cmds::*;
use commands::meanwhile_cmds::*;
use commands::daily_reading_cmds::*;
use commands::imagined_chapter_cmds::*;
use commands::quest_cmds::*;
use commands::genesis_cmds::*;
use commands::memory_cmds::*;
use commands::mood_cmds::*;
use commands::novel_cmds::*;
use commands::portrait_cmds::*;
use commands::reaction_cmds::*;
use commands::canon_cmds::*;
use commands::settings_cmds::*;
use commands::usage_cmds::*;
use commands::user_profile_cmds::*;
use commands::world_cmds::*;
use commands::world_image_cmds::*;
use commands::location_cmds::*;
use commands::chiptune_score_cmds::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("worldthreads.db");
            log::info!("Database path: {}", db_path.display());

            let database = db::Database::open(&db_path)
                .expect("failed to open database");

            // Mirror persisted children-mode setting into a process env gate
            // consumed by the OpenAI injection layer on every LLM call.
            // Default OFF when absent.
            if let Ok(conn) = database.conn.lock() {
                let enabled = crate::db::queries::get_setting(&conn, "children_mode")
                    .ok()
                    .flatten()
                    .map(|v| v == "true" || v == "1" || v.eq_ignore_ascii_case("on"))
                    .unwrap_or(false);
                unsafe {
                    std::env::set_var(
                        "WORLDTHREADS_CHILDREN_MODE",
                        if enabled { "1" } else { "0" },
                    );
                }
            }
            app.manage(database);
            app.manage(DbPath(db_path.clone()));

            // Periodic backup every 20 minutes
            {
                let path = db_path.clone();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(60 * 60));
                        db::Database::backup_database(&path);
                    }
                });
            }

            let portraits_dir = app_dir.join("portraits");
            std::fs::create_dir_all(&portraits_dir).ok();

            // One-shot orphan cleanup: drop world_images entries (and
            // image+video files) for chat illustrations whose message
            // rows are gone. Self-healing on every startup; fast no-op
            // when nothing to clean.
            if let Some(db) = app.try_state::<db::Database>() {
                if let Ok(conn) = db.conn.lock() {
                    if let Err(e) = commands::illustration_cmds::cleanup_orphaned_illustrations(&conn, &portraits_dir) {
                        log::warn!("[startup] orphan-illustration cleanup failed: {e}");
                    }
                }
            }

            app.manage(PortraitsDir(portraits_dir));

            let audio_dir = app_dir.join("audio");
            std::fs::create_dir_all(&audio_dir).ok();
            app.manage(AudioDir(audio_dir));

            app.manage(commands::novel_cmds::BgNovelHandle::default());

            let salt_path = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve app local data path")
                .join("salt.txt");
            app.handle().plugin(
                tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build(),
            )?;

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Warn)
                        .build(),
                )?;
            }

            // Size window to nearly fill the screen, leaving ~120px at the bottom
            if let Some(window) = app.get_webview_window("main") {
                if let Some(monitor) = window.current_monitor().ok().flatten() {
                    let screen = monitor.size();
                    let scale = monitor.scale_factor();
                    let sw = (screen.width as f64 / scale) as u32;
                    let sh = (screen.height as f64 / scale) as u32;
                    let margin_bottom: u32 = 120;
                    let _ = window.set_size(tauri::LogicalSize::new(sw, sh.saturating_sub(margin_bottom)));
                    let _ = window.set_position(tauri::LogicalPosition::new(0, 0));
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_world_cmd,
            get_world_cmd,
            get_world_derivation_cmd,
            list_worlds_cmd,
            update_world_cmd,
            delete_world_cmd,
            update_world_state_cmd,
            list_characters_cmd,
            get_character_cmd,
            get_character_derivation_cmd,
            regenerate_character_derivation_cmd,
            update_character_cmd,
            create_character_cmd,
            delete_character_cmd,
            clear_chat_history_cmd,
            archive_character_cmd,
            unarchive_character_cmd,
            list_archived_characters_cmd,
            create_context_message_cmd,
            get_chat_location_cmd,
            set_chat_location_cmd,
            list_saved_places_cmd,
            delete_saved_place_cmd,
            save_user_message_cmd,
            send_message_cmd,
            prompt_character_cmd,
            try_proactive_ping_cmd,
            get_proactive_unread_counts_cmd,
            generate_narrative_cmd,
            generate_dream_cmd,
            generate_illustration_cmd,
            list_thread_illustrations_cmd,
            get_illustration_captions_cmd,
            update_illustration_caption_cmd,
            delete_illustration_cmd,
            regenerate_illustration_cmd,
            adjust_illustration_cmd,
            generate_video_cmd,
            get_illustration_data_cmd,
            get_illustration_aspect_ratio_cmd,
            get_video_file_cmd,
            remove_video_cmd,
            upload_video_cmd,
            download_illustration_cmd,
            get_video_bytes_cmd,
            get_media_dir_cmd,
            adjust_message_cmd,
            edit_message_content_cmd,
            delete_message_cmd,
            generate_novel_entry_cmd,
            save_novel_entry_cmd,
            get_novel_entry_cmd,
            list_novel_entries_cmd,
            delete_novel_entry_cmd,
            run_background_novelization_cmd,
            cancel_background_novelization_cmd,
            story_consultant_cmd,
            generate_consultant_title_cmd,
            create_consultant_chat_cmd,
            list_consultant_chats_cmd,
            update_consultant_chat_title_cmd,
            delete_consultant_chat_cmd,
            load_consultant_chat_cmd,
            clear_consultant_chat_cmd,
            truncate_consultant_chat_cmd,
            save_consultant_messages_cmd,
            import_chat_messages_cmd,
            get_last_seen_message_cmd,
            reset_to_message_cmd,
            get_last_message_time_cmd,
            get_messages_cmd,
            get_model_config_cmd,
            set_model_config_cmd,
            get_setting_cmd,
            set_setting_cmd,
            is_children_mode_password_set_cmd,
            enable_children_mode_with_password_cmd,
            disable_children_mode_with_password_cmd,
            record_chat_settings_change_cmd,
            get_budget_mode_cmd,
            set_budget_mode_cmd,
            get_memory_artifacts_cmd,
            backfill_embeddings_cmd,
            get_thread_summary_cmd,
            generate_chat_summary_cmd,
            generate_group_chat_summary_cmd,
            add_reaction_cmd,
            remove_reaction_cmd,
            get_reactions_cmd,
            get_mood_reduction_cmd,
            propose_kept_weave_cmd,
            save_kept_record_cmd,
            propose_auto_canon_cmd,
            commit_auto_canon_cmd,
            list_kept_message_ids_cmd,
            list_kept_for_message_cmd,
            delete_kept_record_cmd,
            generate_portrait_cmd,
            generate_portrait_variation_cmd,
            generate_portrait_with_pose_cmd,
            list_portraits_cmd,
            delete_portrait_cmd,
            set_active_portrait_cmd,
            set_portrait_from_gallery_cmd,
            get_active_portrait_cmd,
            generate_character_visual_description_cmd,
            list_characters_needing_visual_description_cmd,
            get_user_profile_cmd,
            update_user_profile_cmd,
            regenerate_user_derivation_cmd,
            generate_user_avatar_cmd,
            upload_user_avatar_cmd,
            get_user_avatar_cmd,
            list_all_user_avatars_cmd,
            set_user_avatar_from_gallery_cmd,
            get_today_usage_cmd,
            generate_world_image_cmd,
            generate_world_image_with_prompt_cmd,
            upload_world_image_cmd,
            list_world_images_cmd,
            list_world_gallery_cmd,
            list_world_gallery_meta_cmd,
            get_gallery_image_cmd,
            archive_gallery_item_cmd,
            unarchive_gallery_item_cmd,
            delete_gallery_item_cmd,
            save_crop_cmd,
            get_active_world_image_cmd,
            set_active_world_image_cmd,
            get_chat_background_cmd,
            update_chat_background_cmd,
            get_character_mood_cmd,
            get_mood_settings_cmd,
            set_mood_settings_cmd,
            generate_next_score_phrase_cmd,
            list_local_models_cmd,
            get_latest_backup_cmd,
            list_backups_cmd,
            backup_now_cmd,
            restore_backup_cmd,
            create_group_chat_cmd,
            list_group_chats_cmd,
            delete_group_chat_cmd,
            clear_group_chat_history_cmd,
            get_group_messages_cmd,
            save_group_user_message_cmd,
            send_group_message_cmd,
            pick_group_responders_cmd,
            prompt_group_character_cmd,
            refresh_character_inventory_cmd,
            refresh_group_inventories_cmd,
            set_character_inventory_cmd,
            update_inventory_for_moment_cmd,
            get_inventory_updates_for_messages_cmd,
            generate_character_journal_cmd,
            maybe_generate_character_journal_cmd,
            list_character_journals_cmd,
            generate_user_journal_cmd,
            maybe_generate_user_journal_cmd,
            list_user_journals_cmd,
            generate_meanwhile_events_cmd,
            maybe_generate_meanwhile_events_cmd,
            list_meanwhile_events_cmd,
            generate_daily_reading_cmd,
            maybe_generate_daily_reading_cmd,
            list_daily_readings_cmd,
            get_latest_daily_reading_cmd,
            generate_imagined_chapter_cmd,
            list_imagined_chapters_for_thread_cmd,
            get_imagined_chapter_cmd,
            delete_imagined_chapter_cmd,
            rename_imagined_chapter_cmd,
            update_imagined_chapter_scene_location_cmd,
            get_imagined_chapter_image_url_cmd,
            canonize_imagined_chapter_cmd,
            decanonize_imagined_chapter_cmd,
            bulk_decanonize_imagined_chapters_for_thread_cmd,
            auto_generate_world_with_characters_cmd,
            reflect_reaching_as_noble_quest_cmd,
            create_quest_cmd,
            list_quests_cmd,
            get_quest_cmd,
            update_quest_cmd,
            update_quest_notes_cmd,
            complete_quest_cmd,
            abandon_quest_cmd,
            reopen_quest_cmd,
            delete_quest_cmd,
            generate_group_illustration_cmd,
            preview_backstage_illustration_cmd,
            attach_previewed_illustration_cmd,
            discard_previewed_illustration_cmd,
            generate_group_narrative_cmd,
            generate_speech_cmd,
            generate_voice_sample_cmd,
            get_speech_cmd,
            list_cached_audio_cmd,
            delete_message_audio_cmd,
            clear_voice_samples_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

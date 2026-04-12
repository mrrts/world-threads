mod ai;
mod commands;
mod db;

use commands::audio_cmds::*;
use commands::backup_cmds::*;
use commands::character_cmds::*;
use commands::chat_cmds::*;
use commands::group_chat_cmds::*;
use commands::memory_cmds::*;
use commands::mood_cmds::*;
use commands::portrait_cmds::*;
use commands::reaction_cmds::*;
use commands::settings_cmds::*;
use commands::usage_cmds::*;
use commands::user_profile_cmds::*;
use commands::world_cmds::*;
use commands::world_image_cmds::*;
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
            app.manage(database);
            app.manage(DbPath(db_path.clone()));

            // Periodic backup every 20 minutes
            {
                let path = db_path.clone();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(20 * 60));
                        db::Database::backup_database(&path);
                    }
                });
            }

            let portraits_dir = app_dir.join("portraits");
            std::fs::create_dir_all(&portraits_dir).ok();
            app.manage(PortraitsDir(portraits_dir));

            let audio_dir = app_dir.join("audio");
            std::fs::create_dir_all(&audio_dir).ok();
            app.manage(AudioDir(audio_dir));

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
                        .level(log::LevelFilter::Error)
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
            list_worlds_cmd,
            update_world_cmd,
            delete_world_cmd,
            update_world_state_cmd,
            list_characters_cmd,
            get_character_cmd,
            update_character_cmd,
            create_character_cmd,
            delete_character_cmd,
            clear_chat_history_cmd,
            archive_character_cmd,
            unarchive_character_cmd,
            list_archived_characters_cmd,
            save_user_message_cmd,
            send_message_cmd,
            prompt_character_cmd,
            generate_narrative_cmd,
            generate_illustration_cmd,
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
            reset_to_message_cmd,
            get_last_message_time_cmd,
            get_messages_cmd,
            get_model_config_cmd,
            set_model_config_cmd,
            get_setting_cmd,
            set_setting_cmd,
            get_budget_mode_cmd,
            set_budget_mode_cmd,
            get_memory_artifacts_cmd,
            get_thread_summary_cmd,
            generate_chat_summary_cmd,
            generate_group_chat_summary_cmd,
            add_reaction_cmd,
            remove_reaction_cmd,
            get_reactions_cmd,
            generate_portrait_cmd,
            generate_portrait_variation_cmd,
            generate_portrait_with_pose_cmd,
            list_portraits_cmd,
            delete_portrait_cmd,
            set_active_portrait_cmd,
            set_portrait_from_gallery_cmd,
            get_active_portrait_cmd,
            get_user_profile_cmd,
            update_user_profile_cmd,
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
            list_local_models_cmd,
            get_latest_backup_cmd,
            backup_now_cmd,
            restore_backup_cmd,
            create_group_chat_cmd,
            list_group_chats_cmd,
            delete_group_chat_cmd,
            clear_group_chat_history_cmd,
            get_group_messages_cmd,
            save_group_user_message_cmd,
            send_group_message_cmd,
            prompt_group_character_cmd,
            generate_group_illustration_cmd,
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

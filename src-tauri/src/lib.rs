mod ai;
mod commands;
mod db;

use commands::character_cmds::*;
use commands::chat_cmds::*;
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

            let portraits_dir = app_dir.join("portraits");
            std::fs::create_dir_all(&portraits_dir).ok();
            app.manage(PortraitsDir(portraits_dir));

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
            reset_to_message_cmd,
            get_messages_cmd,
            get_model_config_cmd,
            set_model_config_cmd,
            get_setting_cmd,
            set_setting_cmd,
            get_budget_mode_cmd,
            set_budget_mode_cmd,
            get_memory_artifacts_cmd,
            get_thread_summary_cmd,
            add_reaction_cmd,
            remove_reaction_cmd,
            get_reactions_cmd,
            generate_portrait_cmd,
            generate_portrait_variation_cmd,
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
            set_user_avatar_from_gallery_cmd,
            get_today_usage_cmd,
            generate_world_image_cmd,
            generate_world_image_with_prompt_cmd,
            upload_world_image_cmd,
            list_world_images_cmd,
            list_world_gallery_cmd,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

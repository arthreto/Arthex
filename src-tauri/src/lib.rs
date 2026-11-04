mod app;
mod modules;
mod services;

use tauri_plugin_autostart::MacosLauncher;

pub fn run() {
    let builder = tauri::Builder::default()
        // Must stay first: a second launch only reopens the overlay of the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            app::overlay::show(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![app::autostart::LAUNCH_ARG]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init());
    #[cfg(not(windows))]
    let builder = builder.plugin(tauri_plugin_global_shortcut::Builder::new().build());

    builder
        .invoke_handler(tauri::generate_handler![
            app::commands::settings_get,
            app::commands::settings_update,
            app::commands::overlay_hide,
            app::commands::hotkey_record,
            app::commands::app_info,
            app::commands::app_open_data_dir,
            app::commands::app_uninstall,
            modules::shortcuts::commands::shortcuts_list,
            modules::shortcuts::commands::shortcuts_save_site,
            modules::shortcuts::commands::shortcuts_save_app,
            modules::shortcuts::commands::shortcuts_remove,
            modules::shortcuts::commands::shortcuts_restore,
            modules::shortcuts::commands::shortcuts_reorder,
            modules::shortcuts::commands::shortcuts_programs,
            modules::shortcuts::commands::shortcuts_browse_program,
            modules::shortcuts::commands::shortcuts_pick_icon,
            modules::shortcuts::commands::shortcuts_open,
            modules::apps::commands::apps_list,
            modules::file_search::commands::file_search_query,
            modules::file_search::commands::file_search_open,
            modules::file_search::commands::file_search_reveal,
            modules::file_search::commands::file_search_status,
            modules::file_search::commands::file_search_rescan,
            modules::file_search::commands::file_search_add_folder,
            modules::file_search::commands::file_search_remove_folder,
            modules::file_search::commands::file_search_remove_exclusion,
            modules::system::commands::system_run,
            modules::system::commands::system_recycle_bin_size,
            modules::system::commands::system_schedule,
            modules::system::commands::system_schedule_cancel,
            modules::system::commands::system_schedule_get,
            modules::apps::commands::apps_open,
            modules::recent_apps::commands::recent_apps_list,
            modules::recent_apps::commands::recent_apps_open,
            modules::recent_apps::commands::recent_apps_clear,
        ])
        .setup(|tauri_app| {
            app::setup(tauri_app.handle())?;
            modules::setup(tauri_app.handle())?;
            Ok(())
        })
        .on_window_event(app::overlay::on_window_event)
        .on_page_load(app::overlay::on_page_load)
        .run(tauri::generate_context!())
        .expect("Arthex failed to start");
}

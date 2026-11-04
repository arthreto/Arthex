mod app;

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
        .plugin(tauri_plugin_opener::init());
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
        ])
        .setup(|tauri_app| Ok(app::setup(tauri_app.handle())?))
        .on_window_event(app::overlay::on_window_event)
        .run(tauri::generate_context!())
        .expect("Arthex failed to start");
}

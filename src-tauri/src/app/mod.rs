pub mod autostart;
pub mod commands;
pub mod dialogs;
mod error;
pub mod events;
mod hotkey;
pub mod i18n;
mod logging;
pub mod overlay;
mod settings;
pub mod storage;
mod system;
mod tray;
pub mod updates;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

pub use error::{AppError, Result};
use settings::SettingsStore;
pub use settings::{Element, Settings, is_module_id};

pub struct AppState {
    settings: SettingsStore,
}

pub fn setup(app: &AppHandle) -> Result<()> {
    logging::init(&app.path().app_log_dir()?);
    let store = SettingsStore::load(app.path().app_config_dir()?.join("settings.json"));
    let settings = store.get();
    logging::set_debug(settings.debug_log);
    system::log_display();
    app.manage(AppState { settings: store });

    autostart::apply(app, settings.autostart);
    tray::create(app)?;
    overlay::prepare(app);
    hotkey::start(app, &settings.hotkey);
    updates::start(app);
    if !autostart::launched_at_startup() {
        overlay::show(app);
    }
    tracing::info!(version = %app.package_info().version, "Arthex started");
    Ok(())
}

pub fn settings(app: &AppHandle) -> Settings {
    app.state::<AppState>().settings.get()
}

/// Saves a partial update, then applies its side effects (startup entry, hotkey, tray menu) and tells the UI.
pub fn update_settings(app: &AppHandle, patch: &Value) -> Result<Settings> {
    let store = &app.state::<AppState>().settings;
    let before = store.get();
    let after = store.update(patch)?;

    if before.autostart != after.autostart {
        autostart::apply(app, after.autostart);
    }
    if before.translucent != after.translucent
        && let Some(window) = app.get_webview_window(overlay::LABEL)
    {
        overlay::see_through(&window);
    }
    if before.debug_log != after.debug_log {
        logging::set_debug(after.debug_log);
    }
    if before.hotkey != after.hotkey {
        hotkey::apply(app, &after.hotkey);
    }
    if (before.autostart != after.autostart || before.language != after.language)
        && let Err(error) = tray::refresh(app)
    {
        tracing::warn!(%error, "could not refresh the tray menu");
    }
    app.emit(events::SETTINGS_CHANGED, &after)?;
    Ok(after)
}

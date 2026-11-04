//! Rust side of the modules in `src/modules/`, one folder per module id.

pub mod apps;
pub mod clipboard;
pub mod file_search;
pub mod filesize;
pub mod notes;
pub mod recent_apps;
pub mod shortcuts;
pub mod system;

use tauri::{AppHandle, Listener};

use crate::{
    app::{self, Result, Settings, events},
    services::file_index,
};

pub fn setup(app: &AppHandle) -> Result<()> {
    apps::setup(app);
    file_index::setup(app)?;
    shortcuts::setup(app)?;
    system::setup(app);
    clipboard::setup(app);
    notes::setup(app)?;
    recent_apps::setup(app)?;
    sync(app, &app::settings(app));
    let handle = app.clone();
    app.listen_any(events::SETTINGS_CHANGED, move |event| {
        if let Ok(settings) = serde_json::from_str::<Settings>(event.payload()) {
            sync(&handle, &settings);
        }
    });
    Ok(())
}

/// Starts or stops the background work of each module to match its switch in Settings.
fn sync(app: &AppHandle, settings: &Settings) {
    recent_apps::set_enabled(app, settings.is_module_enabled(recent_apps::ID));
    clipboard::set_enabled(app, settings.is_module_enabled(clipboard::ID));
    file_index::set_wanted(
        app,
        settings.is_module_enabled(file_search::ID) || settings.is_module_enabled(filesize::ID),
    );
}

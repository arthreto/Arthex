//! Rust side of the modules in `src/modules/`, one folder per module id.

pub mod apps;
pub mod clipboard;
pub mod file_search;
pub mod filesize;
pub mod notes;
pub mod recent_apps;
pub mod shortcuts;
pub mod system;
pub mod web_search;

use tauri::{AppHandle, Listener};

use crate::{
    app::{self, Element, Result, Settings, events},
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

/// Starts or stops the background work of each module to match its switches in Settings.
/// The elements listed here are the ones each module's manifest declares.
fn sync(app: &AppHandle, settings: &Settings) {
    use Element::{Page, Search, Widget};
    recent_apps::set_enabled(app, settings.is_module_active(recent_apps::ID, &[Widget]));
    clipboard::set_enabled(
        app,
        settings.is_module_active(clipboard::ID, &[Page, Widget]),
    );
    clipboard::set_keep(app, settings.keep_clipboard);
    file_index::set_wanted(
        app,
        settings.is_module_active(file_search::ID, &[Search])
            || settings.is_module_active(filesize::ID, &[Page, Widget]),
    );
}

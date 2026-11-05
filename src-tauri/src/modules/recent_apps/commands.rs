use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use super::RecentApps;
use crate::{
    app::{AppError, Result, dialogs::blocking, overlay},
    services::{icons, module_host::is_module_program},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentApp {
    path: String,
    name: String,
    last_used: u64,
    running: bool,
    icon: Option<String>,
}

#[tauri::command]
pub async fn recent_apps_list(app: AppHandle) -> Result<Vec<RecentApp>> {
    let recent = app.state::<RecentApps>();
    let entries: Vec<_> = recent
        .entries()
        .into_iter()
        .filter(|entry| !is_module_program(&app, &entry.path))
        .map(|entry| {
            let window = recent.window_of(&entry.path);
            (entry, window)
        })
        .collect();
    blocking(move || {
        entries
            .into_iter()
            .map(|(entry, window)| RecentApp {
                running: window.is_some_and(is_open),
                icon: icons::file_icon(std::path::Path::new(&entry.path)),
                path: entry.path,
                name: entry.name,
                last_used: entry.last_used,
            })
            .collect()
    })
    .await
}

/// Brings the app back if it still runs, otherwise starts it again.
#[tauri::command]
pub fn recent_apps_open(app: AppHandle, path: String) -> Result<()> {
    let recent = app.state::<RecentApps>();
    let entry = recent
        .entries()
        .into_iter()
        .find(|entry| entry.path == path)
        .ok_or_else(|| AppError::Invalid("not a recent app".into()))?;
    let restored = recent.window_of(&entry.path).is_some_and(bring_to_front);
    if !restored {
        app.opener()
            .open_path(&entry.path, None::<&str>)
            .map_err(|error| AppError::Failed(error.to_string()))?;
    }
    overlay::hide(&app);
    Ok(())
}

#[tauri::command]
pub fn recent_apps_clear(app: AppHandle) -> Result<()> {
    app.state::<RecentApps>().clear()
}

#[cfg(windows)]
use super::watch::{bring_to_front, is_open};

#[cfg(not(windows))]
fn is_open(_window: isize) -> bool {
    false
}

#[cfg(not(windows))]
fn bring_to_front(_window: isize) -> bool {
    false
}

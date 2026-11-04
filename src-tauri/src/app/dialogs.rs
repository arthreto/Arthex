//! Native file pickers, opened above the overlay without closing it.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use super::{AppError, Result, i18n::Translator, overlay};

/// Asks for one file. `filter` is the locale key naming the accepted files.
pub async fn pick_file(
    app: &AppHandle,
    filter: &str,
    extensions: &'static [&'static str],
) -> Result<Option<PathBuf>> {
    let app = app.clone();
    let label = Translator::new(super::settings(&app).language.as_deref()).t(filter);
    tauri::async_runtime::spawn_blocking(move || {
        let _hold = overlay::hold(&app);
        let mut dialog = app.dialog().file().add_filter(label, extensions);
        if let Some(window) = app.get_webview_window(overlay::LABEL) {
            dialog = dialog.set_parent(&window);
        }
        dialog
            .blocking_pick_file()
            .and_then(|file| file.into_path().ok())
    })
    .await
    .map_err(|error| AppError::Failed(error.to_string()))
}

/// Runs slow work (disk, shell) off the interface thread.
pub async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| AppError::Failed(error.to_string()))
}

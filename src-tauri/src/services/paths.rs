//! Checks and shell actions on paths sent by the interface.

use std::path::Path;

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::app::{AppError, Result};

/// Accepts only an absolute path to something that exists.
pub fn existing(path: &str) -> Result<&Path> {
    let path = Path::new(path);
    if path.is_absolute() && path.exists() {
        Ok(path)
    } else {
        Err(AppError::Invalid("this file no longer exists".into()))
    }
}

/// Opens a file or folder with its default program.
pub fn open(app: &AppHandle, path: &Path) -> Result<()> {
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Failed(error.to_string()))
}

/// Shows an item selected in its folder, in the file manager.
pub fn reveal(app: &AppHandle, path: &Path) -> Result<()> {
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|error| AppError::Failed(error.to_string()))
}

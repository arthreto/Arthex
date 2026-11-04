use std::path::PathBuf;

use tauri::{AppHandle, Manager};

#[cfg(windows)]
use super::windows::recycle;
use super::{Drive, Folder, LargeFile};
use crate::{
    app::{
        AppError, Result,
        dialogs::{blocking, pick_folder},
    },
    services::{file_index::FileIndex, paths},
};

#[tauri::command]
pub async fn filesize_drives() -> Result<Vec<Drive>> {
    blocking(super::drives).await
}

#[tauri::command]
pub async fn filesize_folder(app: AppHandle, path: String) -> Result<Option<Folder>> {
    blocking(move || super::folder(&app.state::<FileIndex>(), &PathBuf::from(path))).await
}

#[tauri::command]
pub async fn filesize_largest(app: AppHandle, path: String) -> Result<Vec<LargeFile>> {
    blocking(move || super::largest(&app.state::<FileIndex>(), &PathBuf::from(path))).await
}

#[tauri::command]
pub fn filesize_rescan(app: AppHandle) {
    app.state::<FileIndex>().rescan();
}

#[tauri::command]
pub fn filesize_reveal(app: AppHandle, path: String) -> Result<()> {
    paths::reveal(&app, paths::existing(&path)?)
}

/// Picks a folder to look at. One outside the index is added to it, and shows once scanned.
#[tauri::command]
pub async fn filesize_pick_folder(app: AppHandle) -> Result<Option<String>> {
    let Some(folder) = pick_folder(&app).await? else {
        return Ok(None);
    };
    let files = app.state::<FileIndex>();
    let path = folder.to_string_lossy().into_owned();
    if files.find(&folder).is_none() {
        files.update_config(|config| {
            if !config.folders.contains(&path) {
                config.folders.push(path.clone());
            }
        })?;
    }
    Ok(Some(path))
}

/// Sends an indexed, unprotected item to the Recycle Bin. The interface asks first.
#[tauri::command]
pub async fn filesize_trash(app: AppHandle, path: String) -> Result<()> {
    blocking(move || {
        let path = paths::existing(&path)?.to_path_buf();
        let files = app.state::<FileIndex>();
        let at = files
            .find(&path)
            .ok_or_else(|| AppError::Invalid("this item is not indexed".into()))?;
        if files.read().is_protected(at) {
            return Err(AppError::Invalid("this item is protected".into()));
        }
        recycle(&path)?;
        files.remove(at);
        Ok(())
    })
    .await?
}

#[cfg(not(windows))]
fn recycle(_path: &std::path::Path) -> Result<()> {
    Err(AppError::Unsupported)
}

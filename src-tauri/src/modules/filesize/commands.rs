use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::{Drive, Folder, LargeFile};
use crate::{
    app::{
        AppError, Result,
        dialogs::{blocking, pick_folder},
        overlay,
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

/// Shows an indexed item in the file manager.
#[tauri::command]
pub async fn filesize_reveal(app: AppHandle, path: String) -> Result<()> {
    blocking(move || {
        let path = paths::existing(&path)?;
        if app.state::<FileIndex>().find(path).is_none() {
            return Err(AppError::Invalid("this item is not indexed".into()));
        }
        paths::reveal(&app, path)
    })
    .await?
}

/// Picks a folder to look at. One the index does not hold yet shows once scanned: the first scan
/// may still be running, the folder may be new, or it may lie outside the indexed roots.
#[tauri::command]
pub async fn filesize_pick_folder(app: AppHandle) -> Result<Option<String>> {
    let Some(folder) = pick_folder(&app).await? else {
        return Ok(None);
    };
    let files = app.state::<FileIndex>();
    let path = folder.to_string_lossy().into_owned();
    if files.find(&folder).is_none() {
        if files.excludes(&folder) {
            return Err(AppError::Invalid(
                "this folder is left out of the index by an exclusion".into(),
            ));
        }
        if !files.covers(&folder) {
            files.update_config(|config| {
                if !config.folders.contains(&path) {
                    config.folders.push(path.clone());
                }
            })?;
        } else if files.building().is_none() {
            files.rescan();
        }
    }
    Ok(Some(path))
}

/// Sends an indexed, unprotected item on a fixed drive to the Recycle Bin. The interface asks
/// first.
#[tauri::command]
pub async fn filesize_trash(app: AppHandle, path: String) -> Result<()> {
    let owner = owner_window(&app);
    blocking(move || {
        let path = paths::existing(&path)?.to_path_buf();
        let files = app.state::<FileIndex>();
        match files.may_remove(&path) {
            None => return Err(AppError::Invalid("this item is not indexed".into())),
            Some(false) => return Err(AppError::Invalid("this item is protected".into())),
            Some(true) => {}
        }
        // The shell may ask before deleting for good; keep the overlay up under its question.
        let _hold = overlay::hold(&app);
        recycle(&path, owner)?;
        files.remove(&path);
        Ok(())
    })
    .await?
}

#[cfg(windows)]
fn recycle(path: &Path, owner: isize) -> Result<()> {
    if !super::windows::has_recycle_bin(path) {
        return Err(AppError::Invalid(
            "this drive has no Recycle Bin, so nothing was deleted".into(),
        ));
    }
    super::windows::recycle(path, owner)
}

#[cfg(not(windows))]
fn recycle(_path: &Path, _owner: isize) -> Result<()> {
    Err(AppError::Unsupported)
}

/// The overlay window, so the shell's questions open above it.
fn owner_window(app: &AppHandle) -> isize {
    #[cfg(windows)]
    if let Some(hwnd) = app
        .get_webview_window("main")
        .and_then(|window| window.hwnd().ok())
    {
        return hwnd.0 as isize;
    }
    #[cfg(not(windows))]
    let _ = app;
    0
}

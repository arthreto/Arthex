use std::path::Path;

use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use super::{FileHit, Status, search, status};
use crate::{
    app::{
        AppError, Result,
        dialogs::{blocking, pick_folder},
        overlay,
    },
    services::file_index::FileIndex,
};

/// Results shown per search; the interface adds them under other groups.
const LIMIT: usize = 8;

#[tauri::command]
pub async fn file_search_query(app: AppHandle, query: String) -> Result<Vec<FileHit>> {
    blocking(move || search(&app.state::<FileIndex>(), &query, LIMIT)).await
}

#[tauri::command]
pub fn file_search_open(app: AppHandle, path: String) -> Result<()> {
    let path = existing(&path)?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|error| AppError::Failed(error.to_string()))?;
    overlay::hide(&app);
    Ok(())
}

/// Opens the folder of a result in the file manager, with the item selected.
#[tauri::command]
pub fn file_search_reveal(app: AppHandle, path: String) -> Result<()> {
    let path = existing(&path)?;
    app.opener()
        .reveal_item_in_dir(path)
        .map_err(|error| AppError::Failed(error.to_string()))?;
    overlay::hide(&app);
    Ok(())
}

#[tauri::command]
pub fn file_search_status(app: AppHandle) -> Status {
    status(&app.state::<FileIndex>())
}

#[tauri::command]
pub fn file_search_rescan(app: AppHandle) {
    app.state::<FileIndex>().rescan();
}

#[tauri::command]
pub async fn file_search_add_folder(app: AppHandle) -> Result<Status> {
    if let Some(folder) = pick_folder(&app).await? {
        let folder = folder.to_string_lossy().into_owned();
        app.state::<FileIndex>().update_config(|config| {
            if !config.folders.contains(&folder) {
                config.folders.push(folder);
            }
        })?;
    }
    Ok(status(&app.state::<FileIndex>()))
}

#[tauri::command]
pub fn file_search_remove_folder(app: AppHandle, folder: String) -> Result<Status> {
    let files = app.state::<FileIndex>();
    files.update_config(|config| config.folders.retain(|item| *item != folder))?;
    Ok(status(&files))
}

#[tauri::command]
pub fn file_search_remove_exclusion(app: AppHandle, exclusion: String) -> Result<Status> {
    let files = app.state::<FileIndex>();
    files.update_config(|config| config.exclusions.retain(|item| *item != exclusion))?;
    Ok(status(&files))
}

fn existing(path: &str) -> Result<&Path> {
    let path = Path::new(path);
    if path.is_absolute() && path.exists() {
        Ok(path)
    } else {
        Err(AppError::Invalid("this file no longer exists".into()))
    }
}

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use super::{FileHit, Status, search, status};
use crate::{
    app::{
        AppError, Result,
        dialogs::{blocking, pick_folder},
        overlay,
    },
    services::file_index::Building,
    services::{file_index::FileIndex, paths},
};

/// Results shown per search; the interface adds them under other groups.
const LIMIT: usize = 8;

#[tauri::command]
pub async fn file_search_query(app: AppHandle, query: String) -> Result<Vec<FileHit>> {
    blocking(move || search(&app.state::<FileIndex>(), &query, LIMIT)).await
}

/// Opens a search result. Only indexed items, so the command cannot start any file.
#[tauri::command]
pub async fn file_search_open(app: AppHandle, path: String) -> Result<()> {
    let handle = app.clone();
    blocking(move || paths::open(&handle, &indexed(&handle, &path)?)).await??;
    overlay::hide(&app);
    Ok(())
}

/// Opens the folder of a result in the file manager, with the item selected.
#[tauri::command]
pub async fn file_search_reveal(app: AppHandle, path: String) -> Result<()> {
    let handle = app.clone();
    blocking(move || paths::reveal(&handle, &indexed(&handle, &path)?)).await??;
    overlay::hide(&app);
    Ok(())
}

fn indexed(app: &AppHandle, path: &str) -> Result<PathBuf> {
    let path = paths::existing(path)?.to_path_buf();
    if app.state::<FileIndex>().find(&path).is_some() {
        Ok(path)
    } else {
        Err(AppError::Invalid("this item is not indexed".into()))
    }
}

#[tauri::command]
pub fn file_search_status(app: AppHandle) -> Status {
    status(&app.state::<FileIndex>())
}

/// How far the first indexing has gone, or `None` once files can be searched.
#[tauri::command]
pub fn file_search_building(app: AppHandle) -> Option<Building> {
    app.state::<FileIndex>().building()
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

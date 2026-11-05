//! Commands of the module host, granted in `capabilities/main.json` and registered in `lib.rs`.

use std::fs;

use tauri::{AppHandle, Manager};

use super::{Installed, ModuleHost, catalog, package::MAX_PACKAGE_BYTES};
use crate::app::{
    AppError, Result,
    dialogs::{blocking, pick_file},
};

#[tauri::command]
pub fn modules_list(app: AppHandle) -> Vec<Installed> {
    app.state::<ModuleHost>().list()
}

#[tauri::command]
pub async fn modules_catalog(app: AppHandle) -> Result<Vec<catalog::Offer>> {
    blocking(move || catalog::offers(&app)).await?
}

/// Installs or updates a module from the catalog.
#[tauri::command]
pub async fn modules_install(app: AppHandle, id: String) -> Result<Installed> {
    blocking(move || catalog::install(&app, &id)).await?
}

/// Installs a `.arthex` file the user picks, without the network.
#[tauri::command]
pub async fn modules_install_file(app: AppHandle) -> Result<Option<Installed>> {
    let Some(path) = pick_file(&app, "files.modules", &["arthex"]).await? else {
        return Ok(None);
    };
    blocking(move || {
        if fs::metadata(&path)?.len() > MAX_PACKAGE_BYTES {
            return Err(AppError::Invalid("this file is too large".into()));
        }
        super::install(&app, &fs::read(&path)?, None).map(Some)
    })
    .await?
}

#[tauri::command]
pub fn modules_uninstall(app: AppHandle, id: String, delete_data: bool) -> Result<()> {
    super::uninstall(&app, &id, delete_data)
}

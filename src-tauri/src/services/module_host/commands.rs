//! Commands of the module host, granted in `capabilities/main.json` and registered in `lib.rs`.

use std::fs;

use serde_json::Value;
use tauri::{AppHandle, Manager};

use super::{Installed, ModuleHost, catalog, opened, package::MAX_PACKAGE_BYTES, program};
use crate::app::{
    AppError, Result,
    dialogs::{blocking, pick_file},
    overlay,
};

#[tauri::command]
pub fn modules_list(app: AppHandle) -> Vec<Installed> {
    app.state::<ModuleHost>().list()
}

#[tauri::command]
pub async fn modules_catalog(app: AppHandle) -> Result<Vec<catalog::Offer>> {
    blocking(move || catalog::offers(&app)).await?
}

/// The last catalog read, without the network.
#[tauri::command]
pub async fn modules_catalog_kept(app: AppHandle) -> Result<Option<Vec<catalog::Offer>>> {
    blocking(move || catalog::kept_offers(&app)).await?
}

/// The image of a module of the catalog.
#[tauri::command]
pub async fn modules_preview(app: AppHandle, id: String) -> Result<Option<String>> {
    blocking(move || catalog::preview(&app, &id)).await?
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

/// Relays a call to a module's program with the overlay out of the way, for as long as it takes.
#[tauri::command]
pub async fn modules_call_hidden(
    app: AppHandle,
    id: String,
    method: String,
    params: Value,
) -> Result<Value> {
    blocking(move || program::call_hidden(&app, &id, &method, params)).await?
}

#[tauri::command]
pub fn modules_hide_overlay(app: AppHandle) {
    overlay::hide(&app);
}

#[tauri::command]
pub fn modules_show_overlay(app: AppHandle) {
    overlay::show(&app);
}

/// Relays a call of a module's interface to its program.
#[tauri::command]
pub async fn modules_call(
    app: AppHandle,
    id: String,
    method: String,
    params: Value,
) -> Result<Value> {
    blocking(move || super::call(&app, &id, &method, params)).await?
}

/// The `.arthex` file opened from Explorer, waiting for the user's answer.
#[tauri::command]
pub fn modules_opened(app: AppHandle) -> Result<Option<opened::InstallRequest>> {
    opened::pending(&app)
}

#[tauri::command]
pub async fn modules_answer_opened(app: AppHandle, install: bool) -> Result<Option<Installed>> {
    blocking(move || opened::answer(&app, install)).await?
}

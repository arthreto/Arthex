//! Commands callable from the interface. Each one must be granted in `capabilities/main.json`
//! and registered in `lib.rs`.

use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;

use super::{Result, i18n, overlay, settings::Settings, system};

#[tauri::command]
pub fn settings_get(app: AppHandle) -> Settings {
    super::settings(&app)
}

#[tauri::command]
pub fn settings_update(app: AppHandle, patch: Value) -> Result<Settings> {
    super::update_settings(&app, &patch)
}

#[tauri::command]
pub fn overlay_hide(app: AppHandle) {
    overlay::hide(&app);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    system_locale: Option<String>,
}

#[tauri::command]
pub fn app_info(app: AppHandle) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        system_locale: i18n::system_locale(),
    }
}

#[tauri::command]
pub fn app_open_data_dir(app: AppHandle) -> Result<()> {
    system::open_data_dir(&app)
}

#[tauri::command]
pub fn app_uninstall(app: AppHandle) -> Result<()> {
    system::uninstall(&app)
}

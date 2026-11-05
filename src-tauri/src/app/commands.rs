//! Commands callable from the interface. Each one must be granted in `capabilities/main.json`
//! and registered in `lib.rs`.

use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;

use super::{Result, hotkey, i18n, overlay, settings::Settings, system, updates};

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
    overlay::conceal(&app);
}

/// Returns `false` where the interface has to read the keys itself.
#[tauri::command]
pub fn hotkey_record(active: bool) -> bool {
    hotkey::record(active)
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
pub fn app_open_install_dir(app: AppHandle) -> Result<()> {
    system::open_install_dir(&app)
}

/// The graphics the page renders with, for the debug log: software rendering cannot be
/// see-through.
#[tauri::command]
pub fn app_note_renderer(renderer: String) {
    let renderer: String = renderer.chars().take(200).collect();
    tracing::debug!(renderer, "webview graphics");
}

#[tauri::command]
pub fn app_open_log_dir(app: AppHandle) -> Result<()> {
    system::open_log_dir(&app)
}

#[tauri::command]
pub fn app_uninstall(app: AppHandle) -> Result<()> {
    system::uninstall(&app)
}

#[tauri::command]
pub async fn updates_check(app: AppHandle) -> Result<Option<updates::Available>> {
    updates::check(&app).await
}

#[tauri::command]
pub async fn updates_install(app: AppHandle) -> Result<()> {
    updates::install(&app).await
}

/// The release notes to show once after an update, if this launch follows one.
#[tauri::command]
pub fn updates_whats_new(app: AppHandle) -> Option<updates::WhatsNew> {
    updates::whats_new(&app)
}

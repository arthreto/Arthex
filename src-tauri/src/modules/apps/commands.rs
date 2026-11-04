use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

use super::{App, Apps};
use crate::app::{AppError, Result, dialogs::blocking, overlay};

#[tauri::command]
pub async fn apps_list(app: AppHandle) -> Result<Vec<App>> {
    blocking(move || app.state::<Apps>().list()).await
}

/// Starts an app from the list, through the shell, as the Start menu does.
#[tauri::command]
pub fn apps_open(app: AppHandle, id: String) -> Result<()> {
    if !app.state::<Apps>().knows(&id) {
        return Err(AppError::Invalid("unknown app".into()));
    }
    app.opener()
        .open_path(format!(r"shell:AppsFolder\{id}"), None::<&str>)
        .map_err(|error| AppError::Failed(error.to_string()))?;
    overlay::hide(&app);
    Ok(())
}

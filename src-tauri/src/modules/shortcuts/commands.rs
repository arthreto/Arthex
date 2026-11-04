use std::thread;

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use super::{CHANGED, Shortcut, Shortcuts, Target, clean_name, favicon, new_id, site};
use crate::app::{AppError, Result, overlay};

#[tauri::command]
pub fn shortcuts_list(shortcuts: State<'_, Shortcuts>) -> Vec<Shortcut> {
    shortcuts.list()
}

/// Adds the site right away; its icon follows through `shortcuts://changed`.
#[tauri::command]
pub fn shortcuts_add_site(
    app: AppHandle,
    shortcuts: State<'_, Shortcuts>,
    url: String,
    name: Option<String>,
) -> Result<Vec<Shortcut>> {
    let url = site::parse(&url).ok_or_else(|| AppError::Invalid("not a website address".into()))?;
    let shortcut = Shortcut {
        id: new_id(),
        name: clean_name(name).unwrap_or_else(|| site::default_name(&url)),
        target: Target::Site {
            url: url.to_string(),
        },
        icon: None,
    };
    let id = shortcut.id.clone();
    let list = shortcuts.add(shortcut)?;

    thread::spawn(move || {
        let Some(icon) = favicon::fetch(&url) else {
            return;
        };
        match app.state::<Shortcuts>().set_icon(&id, icon) {
            Ok(list) => {
                let _ = app.emit(CHANGED, list);
            }
            Err(error) => tracing::warn!(%error, "could not save a site icon"),
        }
    });
    Ok(list)
}

#[tauri::command]
pub fn shortcuts_open(app: AppHandle, shortcuts: State<'_, Shortcuts>, id: String) -> Result<()> {
    let shortcut = shortcuts
        .get(&id)
        .ok_or_else(|| AppError::Invalid("unknown shortcut".into()))?;
    let opened = match &shortcut.target {
        Target::Site { url } => app.opener().open_url(url, None::<&str>),
        Target::App { path } => app.opener().open_path(path, None::<&str>),
    };
    opened.map_err(|error| AppError::Failed(error.to_string()))?;
    overlay::hide(&app);
    Ok(())
}

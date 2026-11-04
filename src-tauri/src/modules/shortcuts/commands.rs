use std::{fs::File, io::Read, path::Path, thread};

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use super::{CHANGED, Shortcut, Shortcuts, Target, clean_name, favicon, new_id, site};
use crate::{
    app::{
        AppError, Result,
        dialogs::{blocking, pick_file},
        overlay,
    },
    services::{
        icons,
        programs::{self, Program},
    },
};

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "ico", "webp", "svg"];
const MAX_IMAGE_BYTES: u64 = 4 * 1024 * 1024;

#[tauri::command]
pub fn shortcuts_list(shortcuts: State<'_, Shortcuts>) -> Vec<Shortcut> {
    shortcuts.list()
}

/// Adds the site right away. Without a chosen icon, the site's own icon follows through
/// `shortcuts://changed`.
#[tauri::command]
pub fn shortcuts_add_site(
    app: AppHandle,
    shortcuts: State<'_, Shortcuts>,
    url: String,
    name: Option<String>,
    icon: Option<String>,
) -> Result<Vec<Shortcut>> {
    let url = site::parse(&url).ok_or_else(|| AppError::Invalid("not a website address".into()))?;
    if icon
        .as_deref()
        .is_some_and(|icon| !icons::is_icon_data_url(icon))
    {
        return Err(AppError::Invalid("not an icon".into()));
    }
    let fetch_icon = icon.is_none();
    let shortcut = Shortcut {
        id: new_id(),
        name: clean_name(name).unwrap_or_else(|| site::default_name(&url)),
        target: Target::Site {
            url: url.to_string(),
        },
        icon,
    };
    let id = shortcut.id.clone();
    let list = shortcuts.add(shortcut)?;

    if fetch_icon {
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
    }
    Ok(list)
}

#[tauri::command]
pub async fn shortcuts_add_app(
    app: AppHandle,
    path: String,
    name: Option<String>,
) -> Result<Vec<Shortcut>> {
    let program = blocking(move || programs::describe(Path::new(&path)))
        .await?
        .ok_or_else(|| AppError::Invalid("not a program".into()))?;
    app.state::<Shortcuts>().add(Shortcut {
        id: new_id(),
        name: clean_name(name).unwrap_or(program.name),
        target: Target::App { path: program.path },
        icon: program.icon,
    })
}

/// Programs of the Start menu, for the "Software" picker.
#[tauri::command]
pub async fn shortcuts_programs() -> Result<Vec<Program>> {
    blocking(programs::installed).await
}

#[tauri::command]
pub async fn shortcuts_browse_program(app: AppHandle) -> Result<Option<Program>> {
    let Some(path) = pick_file(&app, "files.programs", &["exe", "lnk"]).await? else {
        return Ok(None);
    };
    let program = blocking(move || programs::describe(&path)).await?;
    program
        .map(Some)
        .ok_or_else(|| AppError::Invalid("not a program".into()))
}

/// A custom icon picked from disk, as a data URL ready for `shortcuts_add_site`.
#[tauri::command]
pub async fn shortcuts_pick_icon(app: AppHandle) -> Result<Option<String>> {
    let Some(path) = pick_file(&app, "files.images", IMAGE_EXTENSIONS).await? else {
        return Ok(None);
    };
    let icon = blocking(move || {
        let mut bytes = Vec::new();
        File::open(&path)
            .ok()?
            .take(MAX_IMAGE_BYTES)
            .read_to_end(&mut bytes)
            .ok()?;
        let is_svg = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"));
        if is_svg {
            icons::svg_data_url(&bytes)
        } else {
            icons::png_data_url(&bytes)
        }
    })
    .await?;
    icon.map(Some)
        .ok_or_else(|| AppError::Invalid("not an image".into()))
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

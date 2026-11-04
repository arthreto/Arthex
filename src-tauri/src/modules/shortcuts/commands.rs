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

/// Adds a site, or edits the one with `id`. Without a chosen icon, a new address gets the site's
/// own icon a moment later through `shortcuts://changed`.
#[tauri::command]
pub fn shortcuts_save_site(
    app: AppHandle,
    shortcuts: State<'_, Shortcuts>,
    id: Option<String>,
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
    let previous = id
        .as_deref()
        .map(|id| existing(&shortcuts, id))
        .transpose()?;
    let same_site = previous.as_ref().is_some_and(
        |previous| matches!(&previous.target, Target::Site { url: old } if *old == url.to_string()),
    );
    let icon = icon.or_else(|| {
        previous
            .filter(|_| same_site)
            .and_then(|previous| previous.icon)
    });
    let fetch_icon = icon.is_none();
    let shortcut = Shortcut {
        id: id.unwrap_or_else(new_id),
        name: clean_name(name).unwrap_or_else(|| site::default_name(&url)),
        target: Target::Site {
            url: url.to_string(),
        },
        icon,
    };
    let saved_id = shortcut.id.clone();
    let list = shortcuts.save(shortcut)?;
    if fetch_icon {
        fetch_icon_later(app, saved_id, url);
    }
    Ok(list)
}

/// Adds a program, or edits the shortcut with `id`.
#[tauri::command]
pub async fn shortcuts_save_app(
    app: AppHandle,
    id: Option<String>,
    path: String,
    name: Option<String>,
) -> Result<Vec<Shortcut>> {
    if let Some(id) = &id {
        existing(&app.state::<Shortcuts>(), id)?;
    }
    let program = blocking(move || programs::describe(Path::new(&path)))
        .await?
        .ok_or_else(|| AppError::Invalid("not a program".into()))?;
    app.state::<Shortcuts>().save(Shortcut {
        id: id.unwrap_or_else(new_id),
        name: clean_name(name).unwrap_or(program.name),
        target: Target::App { path: program.path },
        icon: program.icon,
    })
}

#[tauri::command]
pub fn shortcuts_remove(shortcuts: State<'_, Shortcuts>, id: String) -> Result<Vec<Shortcut>> {
    shortcuts.remove(&id)
}

/// Undoes a removal: the shortcut is checked again, as it comes back from the interface.
#[tauri::command]
pub async fn shortcuts_restore(
    app: AppHandle,
    shortcut: Shortcut,
    index: usize,
) -> Result<Vec<Shortcut>> {
    let checked = blocking(move || checked(shortcut))
        .await?
        .ok_or_else(|| AppError::Invalid("not a valid shortcut".into()))?;
    app.state::<Shortcuts>().insert(index, checked)
}

#[tauri::command]
pub fn shortcuts_reorder(
    shortcuts: State<'_, Shortcuts>,
    ids: Vec<String>,
) -> Result<Vec<Shortcut>> {
    shortcuts.reorder(&ids)
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

fn existing(shortcuts: &Shortcuts, id: &str) -> Result<Shortcut> {
    shortcuts
        .get(id)
        .ok_or_else(|| AppError::Invalid("unknown shortcut".into()))
}

fn fetch_icon_later(app: AppHandle, id: String, url: tauri::Url) {
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

/// Rebuilds a shortcut sent by the interface from what Rust itself accepts.
fn checked(shortcut: Shortcut) -> Option<Shortcut> {
    let target = match shortcut.target {
        Target::Site { url } => Target::Site {
            url: site::parse(&url)?.to_string(),
        },
        Target::App { path } => Target::App {
            path: programs::describe(Path::new(&path))?.path,
        },
    };
    let icon = match shortcut.icon {
        Some(icon) if !icons::is_icon_data_url(&icon) => return None,
        icon => icon,
    };
    Some(Shortcut {
        id: shortcut.id,
        name: clean_name(Some(shortcut.name))?,
        target,
        icon,
    })
}

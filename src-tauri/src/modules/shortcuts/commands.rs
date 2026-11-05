use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    thread,
};

use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use super::{
    CHANGED, Shortcut, Shortcuts, Symbol, Target, clean_keyword, clean_line, clean_name, favicon,
    new_id, site,
};
use crate::{
    app::{
        AppError, Result,
        dialogs::{blocking, pick_file, pick_folder},
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

/// How a shortcut looks: its usual icon (site, app), a pictogram or an image the user picked.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Look {
    Auto,
    Symbol { name: String, tint: String },
    Image { icon: String },
}

/// A shortcut as the dialog sends it, checked here before it is saved.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    #[serde(flatten)]
    target: Target,
    name: Option<String>,
    look: Look,
    keyword: Option<String>,
    on_home: bool,
    in_search: bool,
}

/// Adds a shortcut, or edits the one with `id`. A site without a chosen look gets its own icon a
/// moment later through `shortcuts://changed`.
#[tauri::command]
pub async fn shortcuts_save(
    app: AppHandle,
    id: Option<String>,
    draft: Draft,
) -> Result<Vec<Shortcut>> {
    let previous = id
        .as_deref()
        .map(|id| existing(&app.state::<Shortcuts>(), id))
        .transpose()?;
    let (shortcut, fetch) = blocking(move || build(previous, id, draft)).await??;
    let saved_id = shortcut.id.clone();
    let list = app.state::<Shortcuts>().save(shortcut)?;
    if let Some(url) = fetch {
        fetch_icon_later(app, saved_id, url);
    }
    Ok(list)
}

/// The shortcut to save, and the site whose icon to fetch afterwards, if any.
fn build(
    previous: Option<Shortcut>,
    id: Option<String>,
    draft: Draft,
) -> Result<(Shortcut, Option<tauri::Url>)> {
    let invalid = |reason: &str| AppError::Invalid(reason.into());
    let name = clean_name(draft.name);
    let (target, name, usual_icon, fetch) = match draft.target {
        Target::Site { url } => {
            let url = site::parse(&url).ok_or_else(|| invalid("not a website address"))?;
            let same = previous.as_ref().is_some_and(
                |previous| matches!(&previous.target, Target::Site { url: old } if *old == url.to_string()),
            );
            let kept = previous.filter(|_| same).and_then(|previous| previous.icon);
            let fetch = kept.is_none().then(|| url.clone());
            let name = name.unwrap_or_else(|| site::default_name(&url));
            (
                Target::Site {
                    url: url.to_string(),
                },
                name,
                kept,
                fetch,
            )
        }
        Target::App { path } => {
            let program =
                programs::describe(Path::new(&path)).ok_or_else(|| invalid("not a program"))?;
            let name = name.unwrap_or(program.name);
            (Target::App { path: program.path }, name, program.icon, None)
        }
        Target::Path { path } => {
            let path = Path::new(&path)
                .canonicalize()
                .map_err(|_| invalid("this folder or file does not exist"))?;
            let name = name
                .or_else(|| {
                    clean_name(
                        path.file_name()
                            .map(|name| name.to_string_lossy().into_owned()),
                    )
                })
                .ok_or_else(|| invalid("give this shortcut a name"))?;
            (
                Target::Path {
                    path: path.to_string_lossy().into_owned(),
                },
                name,
                None,
                None,
            )
        }
        Target::Command { line } => {
            let line = clean_line(&line).ok_or_else(|| invalid("write the command on one line"))?;
            let name = name.ok_or_else(|| invalid("give this command a name"))?;
            (Target::Command { line }, name, None, None)
        }
    };
    let (icon, symbol, fetch) = match draft.look {
        Look::Auto => (usual_icon, None, fetch),
        Look::Symbol { name, tint } => {
            let symbol = Symbol { name, tint };
            if !symbol.is_valid() {
                return Err(invalid("not a pictogram"));
            }
            (None, Some(symbol), None)
        }
        Look::Image { icon } if icons::is_icon_data_url(&icon) => (Some(icon), None, None),
        Look::Image { .. } => return Err(invalid("not an icon")),
    };
    let shortcut = Shortcut {
        id: id.unwrap_or_else(new_id),
        name,
        target,
        icon,
        symbol,
        keyword: clean_keyword(draft.keyword),
        on_home: draft.on_home,
        in_search: draft.in_search,
    };
    Ok((shortcut, fetch))
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

/// A folder for a shortcut; a file path can also be typed.
#[tauri::command]
pub async fn shortcuts_browse_folder(app: AppHandle) -> Result<Option<String>> {
    Ok(pick_folder(&app)
        .await?
        .map(|folder| folder.to_string_lossy().into_owned()))
}

/// A custom icon picked from disk, as a data URL ready for `shortcuts_save`.
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
    let failed = |error: &dyn std::fmt::Display| AppError::Failed(error.to_string());
    match &shortcut.target {
        Target::Site { url } => app
            .opener()
            .open_url(url, None::<&str>)
            .map_err(|error| failed(&error)),
        Target::App { path } | Target::Path { path } => app
            .opener()
            .open_path(path, None::<&str>)
            .map_err(|error| failed(&error)),
        Target::Command { line } => {
            run(line, app.path().home_dir()?).map_err(|error| failed(&error))
        }
    }?;
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

/// Starts a command the user wrote in a shortcut, through the system shell, from the home folder.
fn run(line: &str, home: PathBuf) -> std::io::Result<()> {
    #[cfg(windows)]
    let mut child = {
        use std::os::windows::process::CommandExt;
        // `raw_arg`: cmd.exe reads the line exactly as typed, quotes included.
        Command::new("cmd")
            .arg("/C")
            .raw_arg(line)
            .current_dir(home)
            .spawn()?
    };
    #[cfg(not(windows))]
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(line)
        .current_dir(home)
        .spawn()?;
    thread::spawn(move || child.wait());
    Ok(())
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
        Target::Path { path } => Target::Path {
            path: Path::new(&path)
                .canonicalize()
                .ok()?
                .to_string_lossy()
                .into_owned(),
        },
        Target::Command { line } => Target::Command {
            line: clean_line(&line)?,
        },
    };
    let icon = match shortcut.icon {
        Some(icon) if !icons::is_icon_data_url(&icon) => return None,
        icon => icon,
    };
    if shortcut
        .symbol
        .as_ref()
        .is_some_and(|symbol| !symbol.is_valid())
    {
        return None;
    }
    Some(Shortcut {
        id: shortcut.id,
        name: clean_name(Some(shortcut.name))?,
        target,
        icon,
        symbol: shortcut.symbol,
        keyword: clean_keyword(shortcut.keyword),
        on_home: shortcut.on_home,
        in_search: shortcut.in_search,
    })
}

//! `.arthex` files opened from Explorer. Windows starts Arthex with the file's path; the package
//! is checked at once, but installed only after the user confirms in the overlay.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, PoisonError},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::{
    Installed, ModuleHost,
    package::{self, MAX_PACKAGE_BYTES, Texts},
    public_key,
};
use crate::app::{AppError, Result};

/// Sent once an opened file was checked; the interface then asks `modules_opened`.
pub const OPENED: &str = "modules://opened";

/// What the overlay shows before installing an opened file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallRequest {
    id: String,
    version: String,
    icon: String,
    tint: String,
    elements: Vec<String>,
    texts: BTreeMap<String, Texts>,
    size: u64,
}

type Checked = Result<(PathBuf, InstallRequest)>;

/// The last opened file: waiting for an answer, or why it cannot be installed.
#[derive(Default)]
pub struct Opened(Mutex<Option<Checked>>);

/// Looks for a `.arthex` file among the arguments of a launch.
pub fn open_from_args(app: &AppHandle, args: &[String]) {
    let Some(path) = args.iter().skip(1).map(PathBuf::from).find(|path| {
        path.extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("arthex"))
    }) else {
        return;
    };
    let app = app.clone();
    std::thread::spawn(move || {
        let Some(host) = app.try_state::<ModuleHost>() else {
            return;
        };
        let checked = check(&app, &path);
        *host.opened.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(checked);
        app.emit(OPENED, ()).ok();
    });
}

fn check(app: &AppHandle, path: &Path) -> Checked {
    let path = path.canonicalize()?;
    let size = fs::metadata(&path)?.len();
    if size > MAX_PACKAGE_BYTES {
        return Err(AppError::Invalid("this file is too large".into()));
    }
    let info = package::open(&fs::read(&path)?, &public_key(app)?)?.info;
    let request = InstallRequest {
        id: info.id,
        version: info.version,
        icon: info.icon,
        tint: info.tint,
        elements: info.elements,
        texts: info.texts,
        size,
    };
    Ok((path, request))
}

/// The opened file waiting for an answer. An unusable file is reported once, then forgotten.
pub fn pending(app: &AppHandle) -> Result<Option<InstallRequest>> {
    let host = app.state::<ModuleHost>();
    let mut opened = host.opened.0.lock().unwrap_or_else(PoisonError::into_inner);
    match opened.take() {
        None => Ok(None),
        Some(Err(error)) => Err(error),
        Some(Ok((path, request))) => {
            *opened = Some(Ok((path, request.clone())));
            Ok(Some(request))
        }
    }
}

/// Installs the opened file if the user confirmed, and forgets it either way.
pub fn answer(app: &AppHandle, install: bool) -> Result<Option<Installed>> {
    let taken = app
        .state::<ModuleHost>()
        .opened
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    match taken {
        Some(Ok((path, request))) if install => {
            super::install(app, &fs::read(path)?, Some(&request.id)).map(Some)
        }
        _ => Ok(None),
    }
}

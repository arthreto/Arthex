//! Installable modules: verified `.arthex` packages unpacked in the app's local data folder,
//! served to the interface through the `arthex-module` protocol. Built-in modules live in the
//! app itself and never go through here.

mod catalog;
pub mod commands;
pub mod context_menu;
mod opened;
mod package;
mod program;
pub mod protocol;
mod shared;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{PoisonError, RwLock},
    time::Duration,
};

use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Listener, Manager};

use crate::app::{self, AppError, Result, Settings, events};
use package::{ENTRY, INFO, ModuleInfo};

pub use opened::open_from_args;
pub use package::HOST_API;
pub use program::call;

pub const SCHEME: &str = "arthex-module";
/// Sent after a module was installed, updated or uninstalled.
pub const CHANGED: &str = "modules://changed";
/// Modules are looked for in the catalog once the start of Windows is over.
const UPDATE_CHECK: Duration = Duration::from_secs(40);

/// An installed module, as the interface sees it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    pub id: String,
    pub version: String,
    /// Where its files are served from, ending with `/`.
    pub url: String,
    pub style: bool,
    /// Loaded unsigned from `ARTHEX_DEV_MODULES`, in development builds only.
    pub dev: bool,
    #[serde(skip)]
    dir: PathBuf,
    #[serde(skip)]
    info: ModuleInfo,
}

pub struct ModuleHost {
    /// `<local data>/modules/<id>/`: the unpacked packages.
    dir: PathBuf,
    /// `<data>/module-data/<id>/`: what each module keeps, which survives updates.
    data: PathBuf,
    /// `<local data>/shared/<id>/`: what several modules use, kept once.
    shared: PathBuf,
    installed: RwLock<BTreeMap<String, Installed>>,
    programs: program::Programs,
    opened: opened::Opened,
}

impl ModuleHost {
    fn installed(&self) -> std::sync::RwLockReadGuard<'_, BTreeMap<String, Installed>> {
        self.installed
            .read()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn installed_mut(&self) -> std::sync::RwLockWriteGuard<'_, BTreeMap<String, Installed>> {
        self.installed
            .write()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn get(&self, id: &str) -> Option<Installed> {
        self.installed().get(id).cloned()
    }

    pub fn list(&self) -> Vec<Installed> {
        self.installed().values().cloned().collect()
    }
}

pub fn setup(app: &AppHandle) -> Result<()> {
    let dir = app.path().app_local_data_dir()?.join("modules");
    let data = app.path().app_data_dir()?.join("module-data");
    let shared = app.path().app_local_data_dir()?.join("shared");
    fs::create_dir_all(&dir)?;
    let mut installed = scan(&dir, false);
    installed.extend(dev_modules());
    app.manage(ModuleHost {
        dir,
        data,
        shared,
        installed: RwLock::new(installed),
        programs: program::Programs::default(),
        opened: opened::Opened::default(),
    });
    let args: Vec<String> = std::env::args().collect();
    open_from_args(app, &args);

    let settings = app::settings(app);
    program::sync(app, &settings);
    context_menu::sync(app, &settings);
    let handle = app.clone();
    app.listen_any(events::SETTINGS_CHANGED, move |event| {
        if let Ok(settings) = serde_json::from_str::<Settings>(event.payload()) {
            program::sync(&handle, &settings);
            context_menu::sync(&handle, &settings);
        }
    });

    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(UPDATE_CHECK);
        catalog::update_at_launch(&handle);
    });
    Ok(())
}

fn is_inside(path: &str, dir: &Path) -> bool {
    let dir = dir.display().to_string().to_lowercase();
    let path = path.to_lowercase();
    path.strip_prefix(&dir)
        .is_some_and(|rest| rest.starts_with(['\\', '/']))
}

/// Whether a program belongs to a module: it is part of Arthex, not an app of the user.
pub fn is_module_program(app: &AppHandle, path: &str) -> bool {
    app.try_state::<ModuleHost>().is_some_and(|host| {
        is_inside(path, &host.dir)
            || is_inside(path, &host.shared)
            || host
                .list()
                .iter()
                .any(|module| is_inside(path, &module.dir))
    })
}

/// Reads what was unpacked earlier. Each package was verified before it was written.
fn scan(dir: &Path, dev: bool) -> BTreeMap<String, Installed> {
    let Ok(entries) = fs::read_dir(dir) else {
        return BTreeMap::new();
    };
    entries
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let info: ModuleInfo = app::storage::read_json(&path.join(INFO))?;
            let usable = path.file_name().is_some_and(|name| *name == *info.id)
                && info.api <= HOST_API
                && path.join(ENTRY).is_file();
            usable.then(|| (info.id.clone(), describe(&path, info, dev)))
        })
        .collect()
}

/// Lets a module be developed without packaging or signing it. Never in release builds.
fn dev_modules() -> BTreeMap<String, Installed> {
    if !cfg!(debug_assertions) {
        return BTreeMap::new();
    }
    std::env::var_os("ARTHEX_DEV_MODULES")
        .map(|dir| scan(Path::new(&dir), true))
        .unwrap_or_default()
}

fn describe(dir: &Path, info: ModuleInfo, dev: bool) -> Installed {
    Installed {
        id: info.id.clone(),
        version: info.version.clone(),
        url: protocol::base_url(&info.id),
        style: dir.join("style.css").is_file(),
        dev,
        dir: dir.to_path_buf(),
        info,
    }
}

/// The public key of the updater: modules are signed with the same key as the app.
fn public_key(app: &AppHandle) -> Result<String> {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|updater| updater.get("pubkey"))
        .and_then(Value::as_str)
        .filter(|key| !key.trim().is_empty())
        .map(String::from)
        .ok_or(AppError::Unsupported)
}

/// Checks a package, then replaces any previous version in one rename.
pub fn install(app: &AppHandle, bytes: &[u8], expected: Option<&str>) -> Result<Installed> {
    let package = package::open(bytes, &public_key(app)?)?;
    let host = app.state::<ModuleHost>();
    let id = package.info.id.clone();
    if expected.is_some_and(|expected| expected != id) {
        return Err(AppError::Invalid(format!("the package holds {id}")));
    }
    if host.get(&id).is_some_and(|current| current.dev) {
        return Err(AppError::Invalid(format!("{id} is being developed")));
    }
    catalog::ensure_shared(app, &package.info.uses)?;

    host.programs.stop(&id);
    let target = replace_folder(&host.dir, &id, &package.files)?;
    #[cfg(unix)]
    if let Some(native) = &package.info.native {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(target.join(native), fs::Permissions::from_mode(0o755))?;
    }

    let installed = describe(&target, package.info, false);
    host.installed_mut().insert(id.clone(), installed.clone());
    shared::prune(&host);
    tracing::info!(module = %id, version = %installed.version, "module installed");
    let settings = app::settings(app);
    program::sync(app, &settings);
    context_menu::sync(app, &settings);
    app.emit(CHANGED, ()).ok();
    Ok(installed)
}

pub fn uninstall(app: &AppHandle, id: &str, delete_data: bool) -> Result<()> {
    let host = app.state::<ModuleHost>();
    let Some(installed) = host.get(id).filter(|module| !module.dev) else {
        return Err(AppError::Invalid(format!("{id} is not installed")));
    };
    host.programs.stop(id);
    fs::remove_dir_all(&installed.dir)?;
    host.installed_mut().remove(id);
    shared::prune(&host);
    if delete_data {
        remove_if_present(&host.data.join(id))?;
    }
    tracing::info!(module = %id, delete_data, "module uninstalled");
    context_menu::sync(app, &app::settings(app));
    app.emit(CHANGED, ()).ok();
    Ok(())
}

/// Writes a folder next to the old one, then swaps them in one rename: never half of each.
fn replace_folder(parent: &Path, id: &str, files: &BTreeMap<String, Vec<u8>>) -> Result<PathBuf> {
    let target = parent.join(id);
    let staging = parent.join(format!("{id}.new"));
    let previous = parent.join(format!("{id}.old"));
    remove_if_present(&staging)?;
    for (path, content) in files {
        let file = staging.join(path);
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder)?;
        }
        fs::write(&file, content)?;
    }
    remove_if_present(&previous)?;
    if target.exists() {
        fs::rename(&target, &previous)?;
    }
    fs::rename(&staging, &target)?;
    let _ = fs::remove_dir_all(&previous);
    Ok(target)
}

fn remove_if_present(dir: &Path) -> Result<()> {
    match fs::remove_dir_all(dir) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_what_is_inside_a_folder() {
        let dir = Path::new(r"C:\Data\modules");
        assert!(is_inside(r"c:\data\MODULES\color\native\color.exe", dir));
        assert!(!is_inside(r"C:\Data\modules-old\color.exe", dir));
        assert!(!is_inside(r"C:\Program Files\App\app.exe", dir));
    }
}

//! Installable modules: verified `.arthex` packages unpacked in the app's local data folder,
//! served to the interface through the `arthex-module` protocol. Built-in modules live in the
//! app itself and never go through here.

mod catalog;
pub mod commands;
mod package;
mod program;
pub mod protocol;

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
    installed: RwLock<BTreeMap<String, Installed>>,
    programs: program::Programs,
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
    fs::create_dir_all(&dir)?;
    let mut installed = scan(&dir, false);
    installed.extend(dev_modules());
    app.manage(ModuleHost {
        dir,
        data,
        installed: RwLock::new(installed),
        programs: program::Programs::default(),
    });

    program::sync(app, &app::settings(app));
    let handle = app.clone();
    app.listen_any(events::SETTINGS_CHANGED, move |event| {
        if let Ok(settings) = serde_json::from_str::<Settings>(event.payload()) {
            program::sync(&handle, &settings);
        }
    });

    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(UPDATE_CHECK);
        catalog::update_at_launch(&handle);
    });
    Ok(())
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

    host.programs.stop(&id);
    let target = host.dir.join(&id);
    let staging = host.dir.join(format!("{id}.new"));
    let previous = host.dir.join(format!("{id}.old"));
    remove_if_present(&staging)?;
    for (path, content) in &package.files {
        let file = staging.join(path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file, content)?;
    }
    #[cfg(unix)]
    if let Some(native) = &package.info.native {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(staging.join(native), fs::Permissions::from_mode(0o755))?;
    }
    remove_if_present(&previous)?;
    if target.exists() {
        fs::rename(&target, &previous)?;
    }
    fs::rename(&staging, &target)?;
    let _ = fs::remove_dir_all(&previous);

    let installed = describe(&target, package.info, false);
    host.installed_mut().insert(id.clone(), installed.clone());
    tracing::info!(module = %id, version = %installed.version, "module installed");
    program::sync(app, &app::settings(app));
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
    if delete_data {
        remove_if_present(&host.data.join(id))?;
    }
    tracing::info!(module = %id, delete_data, "module uninstalled");
    app.emit(CHANGED, ()).ok();
    Ok(())
}

fn remove_if_present(dir: &Path) -> Result<()> {
    match fs::remove_dir_all(dir) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}
